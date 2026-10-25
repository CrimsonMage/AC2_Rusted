//! The Asheron's Call 2 client in a window: log on to a server, choose a character on the
//! retail character selection screen (or name one to enter at once, as `-startchar` did), enter
//! the world and play with the retail controls (the DATs' key maps and action map: W/S/A/D or
//! the arrows to move, Q/E to strafe, the wheel to zoom, the middle button for mouse look, ...),
//! drawn through the shared scene and seen from the retail camera.
//!
//! ```text
//! ac2 --dat-dir DIR [--server 127.0.0.1:9000] [--account local] [--version 7160]
//!     [--character NAME] [--client-exe PATH] [--radius 1]
//! ```

mod keys;
mod pregame;

use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_client::pregame_view::PregameView;
use ac2_client::view::ClientView;
use ac2_client::{frame_input, Client, Command, DatWorld};
use ac2_gpu::{render_to, Gpu, DEPTH_FORMAT};
use ac2_net::Credential;
use ac2_ui::{Controls, InputEvent, SelectionUi};
use ac2_weenie::{CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent};
use winit::application::ApplicationHandler;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

/// Frame-thread time spent taking finished loads (`DBCache::UseTime`'s pump).
const LOAD_BUDGET: Duration = Duration::from_millis(4);
/// `WHEEL_DELTA`: one notch.
const WHEEL_DELTA: f32 = 120.0;

struct Args {
    dat_dir: String,
    server: String,
    account: String,
    version: String,
    /// `gmClient::m_startChar`: enter with this character, skipping character selection.
    character: Option<String>,
    client_exe: Option<String>,
    radius: i32,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        dat_dir: std::env::var("AC2_DAT_DIR").unwrap_or_default(),
        server: "127.0.0.1:9000".into(),
        account: "local".into(),
        version: ac2_net::LEGIONS_VERSION.into(),
        character: None,
        client_exe: None,
        radius: 1,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it.next().ok_or(format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--dat-dir" => args.dat_dir = value,
            "--server" => args.server = value,
            "--account" => args.account = value,
            "--version" => args.version = value,
            "--character" => args.character = Some(value),
            "--client-exe" => args.client_exe = Some(value),
            "--radius" => args.radius = value.parse().map_err(|e| format!("--radius: {e}"))?,
            other => return Err(format!("unknown option {other}")),
        }
    }
    if args.dat_dir.is_empty() {
        return Err("set AC2_DAT_DIR or pass --dat-dir".into());
    }
    Ok(args)
}

/// Everything the program owns once the window exists.
struct State {
    window: Arc<Window>,
    gpu: Gpu,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    format: wgpu::TextureFormat,
    /// The world and HUD, once the player is in the world.
    view: Option<ClientView>,
    /// The pregame screens until then (none when a start character enters at once).
    pregame: Option<PregameView>,
    controls: Controls,
    /// `gmSelectionUI` and the commands its finished selections send next frame.
    selection: SelectionUi,
    commands: Vec<Command>,
    /// `SoundManager`: the UI's sounds, played through the speakers.
    audio: ac2_audio::SoundManager,
    /// Client time origin (`Timer::cur_time` for the interpreter, materials and input stamps).
    started: Instant,
    grabbed: bool,
}

struct App {
    client: Client,
    world: Arc<ac2_world::Ac2WorldSource>,
    controls: Option<Controls>,
    radius: i32,
    state: Option<State>,
    store: Arc<ac2_db::ResourceStore>,
    /// Whether the pregame screens show (no start character).
    pregame: bool,
    /// Character generation's `CharacterGenSystem`, handed to the pregame screens.
    chargen: Option<ac2_client::ScriptCharGen>,
}

impl State {
    fn now(&self) -> (u32, f64) {
        let elapsed = self.started.elapsed();
        (elapsed.as_millis() as u32, elapsed.as_secs_f64())
    }

    fn input(&mut self, event: InputEvent) {
        let (stamp, now) = self.now();
        self.controls.event(event, stamp, now);
    }

    /// `CInputManager::SetMouseLookMode` follows the camera: the pointer is captured and hidden
    /// in mouse look.
    fn sync_mouse_look(&mut self, mouse_look: bool) {
        self.controls.input.set_mouse_look(mouse_look);
        if mouse_look != self.grabbed {
            let grab = if mouse_look {
                self.window
                    .set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined))
            } else {
                self.window.set_cursor_grab(CursorGrabMode::None)
            };
            if grab.is_ok() {
                self.window.set_cursor_visible(!mouse_look);
                self.grabbed = mouse_look;
            }
        }
    }
}

/// The speakers, or silence when there is no output device.
fn speakers() -> Box<dyn ac2_audio::SoundSink> {
    match ac2_audio::SpeakerSink::open() {
        Ok(sink) => Box::new(sink),
        Err(error) => {
            eprintln!("no sound: {error}");
            Box::new(ac2_audio::NullSink)
        }
    }
}

fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("ac2 depth"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

impl App {
    /// One client frame: the controls' frame into `Client::use_time`, then the view.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let Some(mut view) = state.view.take() else {
            self.pregame_frame(event_loop);
            return;
        };
        self.world_frame(event_loop, &mut view);
        if let Some(state) = self.state.as_mut() {
            state.view = Some(view);
        }
    }

    /// One frame in the world.
    fn world_frame(&mut self, event_loop: &ActiveEventLoop, view: &mut ClientView) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        // The UI tree takes the UI actions first; its messages reach the camera UI.
        let ui_input = state.controls.take_ui_input();
        let modifiers = state.controls.modifiers();
        let mut messages = view.ui_input(&ui_input, modifiers);
        let (actions, characters, _) = state.controls.take_text_input();
        messages.extend(view.text_input(&actions, &characters, modifiers));
        for request in view.take_chat_requests() {
            match request {
                ac2_ui::ChatRequest::Line(line) => state.commands.push(Command::ChatLine(line)),
                // `ChatWindow::DisplayCmdHistory` / `DisplayRetellHistory` /
                // `DisplayReplyHistory`.
                ac2_ui::ChatRequest::History { history, prev } => {
                    let kind = match history {
                        ac2_ui::ChatHistory::Command => ac2_weenie::ChatHistory::Command,
                        ac2_ui::ChatHistory::Retell => ac2_weenie::ChatHistory::Retell,
                        ac2_ui::ChatHistory::Reply => ac2_weenie::ChatHistory::Reply,
                    };
                    let Some(item) = self.client.session_mut().chat_history(kind, prev) else {
                        continue;
                    };
                    if history == ac2_ui::ChatHistory::Command {
                        view.set_chat_entry(&item);
                    } else {
                        view.assisted_tell(&item);
                    }
                }
            }
        }
        let (stamp, now) = state.now();
        state.controls.set_text_mode(view.text_mode(), stamp, now);
        state.controls.ui_messages(&messages, now);
        for message in &messages {
            state.selection.message(message);
        }
        let mut input = frame_input(state.controls.frame());
        input.commands.append(&mut state.commands);
        let events = match self.client.use_time(&input) {
            Ok(events) => events,
            Err(error) => {
                eprintln!("ac2: {error}");
                event_loop.exit();
                return;
            }
        };
        for event in &events {
            match event {
                SessionEvent::Notice(GameplayNotice::DisplayStringInfo { kind, text }) => {
                    view.chat_display_info(*kind, text);
                }
                SessionEvent::ShellCommand { text, .. } => {
                    eprintln!("ac2: chat command not ported: {text}");
                }
                _ => {}
            }
        }
        for notice in self.client.take_notices() {
            if notice == ac2_client::Notice::ConnectionLost {
                eprintln!("ac2: connection lost");
                event_loop.exit();
                return;
            }
        }
        state.sync_mouse_look(self.client.camera().movement_state().mouse_look);
        let frame = self.client.frame();
        // The HUD frameworks that ask the player: skills panel, shortcut bar, chat options.
        let script_notices = self.client.take_script_notices();
        let asked = view.frameworks(&mut self.client, &frame, &events, &script_notices);
        state.commands.extend(asked);
        // The selection render: the object under the pointer, selected (or the selection
        // cleared) by `gmSelectionUI::Finish3DSelection`.
        if state.selection.wants_pick() {
            let screen = (state.config.width, state.config.height);
            let picked = view.pick(&frame, screen, view.pointer());
            let outcome = state.selection.finish(picked);
            state.commands.push(Command::Select(outcome.select));
            if outcome.use_selection {
                state.commands.push(Command::UseSelection);
            }
            if outcome.open_examination {
                view.open_examination();
            }
        }
        view.update(&state.gpu, &frame, LOAD_BUDGET);
        ac2_client::play_ui_sounds(&mut state.audio, &view.take_ui_sounds());
        let (_, time) = state.now();
        state.audio.use_time(time);
        let Ok(output) = state.surface.get_current_texture() else {
            state.surface.configure(&state.gpu.device, &state.config);
            return;
        };
        let target = output.texture.create_view(&Default::default());
        let aspect = state.config.width as f32 / state.config.height as f32;
        let drawn = view.prepare(&state.gpu, &frame, aspect, time);
        if view.take_deselect() {
            state.commands.push(Command::Select(None));
        }
        render_to(
            &state.gpu.device,
            &state.gpu.queue,
            &target,
            &state.depth,
            [0.0; 3],
            |pass| {
                if drawn {
                    view.draw(pass);
                }
            },
        );
        output.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Asheron's Call 2"))
                .expect("window"),
        );
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).expect("surface");
        let gpu = Gpu::new(Some(&surface), instance).expect("GPU adapter");
        let caps = surface.get_capabilities(&gpu.adapter);
        // Gamma-space output like the retail D3D back buffer.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&gpu.device, &config);
        let depth = depth_view(&gpu.device, &config);
        let pregame = self.pregame.then(|| {
            let mut view = PregameView::new(
                Arc::clone(&self.store),
                &gpu,
                format,
                (config.width, config.height),
            );
            if let Some(chargen) = self.chargen.take() {
                view.set_char_gen_system(Box::new(chargen));
            }
            view
        });
        self.state = Some(State {
            window,
            gpu,
            surface,
            config,
            depth,
            format,
            view: None,
            pregame,
            selection: SelectionUi::default(),
            commands: Vec::new(),
            controls: {
                // The HUD's tree takes the UI actions (mouse over panels, the world's wheel).
                let mut controls = self.controls.take().expect("controls");
                controls.ui_attached = true;
                controls
            },
            audio: ac2_client::sound_manager(Arc::clone(self.world.resource_store()), speakers()),
            started: Instant::now(),
            grabbed: false,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::RedrawRequested) {
            self.frame(event_loop);
            return;
        }
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                self.client.session_mut().log_off(Duration::from_secs(3));
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                state.config.width = size.width.max(1);
                state.config.height = size.height.max(1);
                state.surface.configure(&state.gpu.device, &state.config);
                state.depth = depth_view(&state.gpu.device, &state.config);
                let (width, height) = (state.config.width, state.config.height);
                if let Some(view) = &mut state.view {
                    view.resize(width, height);
                }
                if let Some(pregame) = &mut state.pregame {
                    pregame.resize(width, height);
                }
            }
            WindowEvent::Focused(focused) => {
                let (_, now) = state.now();
                state.controls.focus(focused, now);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Outside text mode a held key is one press (DirectInput); in text mode key
                // repeats move the cursor and type again (`WM_KEYDOWN` / `WM_CHAR`).
                let text_mode = state.controls.input.text_mode();
                if event.repeat && !text_mode {
                    return;
                }
                let pressed = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(scan) = keys::dik(code) {
                        state.input(InputEvent::Key { scan, pressed });
                    }
                }
                if pressed {
                    if let Some(text) = &event.text {
                        for ch in text.encode_utf16() {
                            state.input(InputEvent::Text { ch });
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state: pressed,
                button,
                ..
            } => {
                let button = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    _ => return,
                };
                state.input(InputEvent::Button {
                    button,
                    pressed: pressed == ElementState::Pressed,
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let notches = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => (p.y / 40.0) as f32,
                };
                if notches != 0.0 {
                    state.input(InputEvent::Wheel {
                        delta: (notches * WHEEL_DELTA) as i32,
                    });
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                state.input(InputEvent::Cursor {
                    x: position.x as i32,
                    y: position.y as i32,
                });
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let (Some(state), DeviceEvent::MouseMotion { delta }) = (self.state.as_mut(), event) {
            state.input(InputEvent::Motion {
                dx: delta.0 as i32,
                dy: delta.1 as i32,
            });
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    let dataset = ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(&args.dat_dir))
        .map_err(|e| e.to_string())?;
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
    ac2_db::install_ac2_property_types(&store).map_err(|e| e.to_string())?;
    let dat_world = DatWorld::new(Arc::clone(&store))?;
    let world = Arc::clone(dat_world.world());
    let controls = Controls::load(&store)
        .map_err(|e| e.to_string())?
        .ok_or("the DATs hold no action map")?;
    let server_key = match &args.client_exe {
        Some(path) => {
            let exe = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            ac2_net::ServerPublicKey::find_in_client(&exe)
        }
        None => None,
    };
    let mut session = OnlineSession::connect(OnlineConfig {
        server: args.server.parse().map_err(|e| format!("--server: {e}"))?,
        account: args.account,
        credential: Credential::Account,
        version: args.version,
        // Without a start character the session waits at the character set for the
        // selection screen (`gmCharacterSelectionUI`).
        character: match &args.character {
            Some(name) => CharacterChoice::Name(name.clone()),
            None => CharacterChoice::Wait,
        },
        server_key,
        language: 0,
        dat_iterations: ac2_weenie::dat_iterations(&dataset)?,
    })
    .map_err(|e| e.to_string())?;
    // The string tables chat and usage lines name (`StringInfo::SetTableEnum`), by DID, read
    // through the client's own store so the script host's reads share its cache.
    let script_data = ac2_script::data::DatScriptData::new(Arc::clone(&dataset), (*store).clone());
    session.set_string_tables(ac2_client::string_table_ids(&script_data));
    // `CWInterface`'s registered chat commands, emote names and usage texts.
    session.set_chat_commands(ac2_client::chat_commands(&script_data));
    // The loader threads serve the script VM's missed reads (the frame thread never reads).
    let dat_world = dat_world.with_script_data(script_data.clone());
    let mut client = Client::new(session, Arc::new(dat_world));
    // The weenies' scripts on the VM (`CWDistributor`), over the script library.
    let wlib = ac2_script::load_wlib(&store, ac2_core::DataId::new(ac2_script::LEGIONS_WLIB_DID))
        .map_err(|e| e.to_string())?
        .ok_or("the DATs hold no script library")?;
    let program = Arc::new(ac2_script::vm::Program::from_stream(&wlib.stream));
    // Before the frame loop: what the interpreter reads between instructions stays resident,
    // and from then on the VM only reads what is resident.
    script_data.preload_tables();
    script_data.preload_states();
    script_data.preload_skill_states(&program);
    script_data.set_cache_only(true);
    // Character generation asks the `CharacterGenSystem` singleton on a host of its own over
    // the same resident data.
    let chargen = ac2_client::ScriptCharGen::new(ac2_script::host::ScriptHost::new(
        Arc::clone(&program),
        Box::new(script_data.clone()),
    ));
    let host = ac2_script::host::ScriptHost::new(program, Box::new(script_data));
    client.attach_scripts(host, ac2_client::SCRIPT_VM_FUNCTIONS);
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let mut app = App {
        client,
        world,
        controls: Some(controls),
        radius: args.radius,
        state: None,
        store,
        pregame: args.character.is_none(),
        chargen: Some(chargen),
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())
}

fn main() {
    let result = parse_args().and_then(run);
    if let Err(message) = result {
        eprintln!("ac2: {message}");
        std::process::exit(1);
    }
}
