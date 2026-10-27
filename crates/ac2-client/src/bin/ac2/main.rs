//! The Asheron's Call 2 client in a window: log on to a server, choose a character on the
//! retail character selection screen (or name one to enter at once, as `-startchar` did), enter
//! the world and play with the retail controls (the DATs' key maps and action map: W/S/A/D or
//! the arrows to move, Q/E to strafe, the wheel to zoom, the middle button for mouse look, ...),
//! drawn through the shared scene and seen from the retail camera.
//!
//! ```text
//! ac2 --dat-dir DIR [--server 127.0.0.1:9000] [--account local] [--version 7160]
//!     [--character NAME] [--radius 1] [--chargen FILE]
//!
//! ac2 --dat-dir DIR --host icefire.portalspace.net --port 8610 --rodat on \
//!     --account ACCOUNT --glsticketdirect PASSWORD --version 1802
//! ```
//!
//! The Pyrealium launcher runs this program as its client executable, as it runs retail's:
//!
//! ```text
//! ac2 --dat-dir DIR --account NAME --host H --port P --rodat --width 1024 --height 768 /glsticket
//! ```
//!
//! `glsticket` without a value reads the ticket the launcher left (the `GLSTicket` registry value
//! on Windows, a mode 0600 `GLSTicket` file in the AC2 configuration directory elsewhere) and
//! deletes it; `--glsticket FILE` reads and deletes that file instead. The ticket is the
//! password.
//!
//! The second usage form above is the retail command line (`Turbine::Client::Init`'s arguments); `--host` and
//! `--port` win over `--server`, and the ticket is the password. Retail's single-dash and `/`
//! forms (`-host`, `/host`, `-h`) are accepted too. Servers for the 2012-2017 relaunch client
//! check its data version, `1802` ([`ac2_net::RELAUNCH_VERSION`]).

mod cmdline;
mod instance;
mod keys;
mod pregame;
mod screenshot;

use cmdline::{parse_args, Args};

use std::io;
use std::path::PathBuf;
use std::sync::{mpsc::Receiver, Arc};
use std::time::{Duration, Instant};

use ac2_client::pregame_view::PregameView;
use ac2_client::view::ClientView;
use ac2_client::{frame_input, pick_targets, Client, Command, DatWorld};
use ac2_gpu::{render_to, Gpu, DEPTH_FORMAT};
use ac2_net::Credential;
use ac2_ui::{Controls, InputEvent, SelectionUi};
use ac2_weenie::{CharacterChoice, GameplayNotice, OnlineConfig, OnlineSession, SessionEvent};
use winit::application::ApplicationHandler;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{CursorGrabMode, Window, WindowId};

/// Frame-thread time spent taking finished loads (`DBCache::UseTime`'s pump).
const LOAD_BUDGET: Duration = Duration::from_millis(4);
/// `WHEEL_DELTA`: one notch.
const WHEEL_DELTA: f32 = 120.0;

/// The script library and script data character generation runs over (`ScriptCharGen`'s host is
/// consumed by its view, so each pregame view builds its own).
struct ChargenSource {
    program: Arc<ac2_script::vm::Program>,
    data: ac2_script::data::DatScriptData,
}

/// A pregame view with its character generation host, and the `--chargen` capture.
fn new_pregame_view(
    store: &Arc<ac2_db::ResourceStore>,
    world: &Arc<ac2_world::Ac2WorldSource>,
    chargen: &ChargenSource,
    capture: Option<&std::path::Path>,
    gpu: &Gpu,
    format: wgpu::TextureFormat,
    size: (u32, u32),
) -> PregameView {
    let host = ac2_script::host::ScriptHost::new(
        Arc::clone(&chargen.program),
        Box::new(chargen.data.clone()),
    );
    let mut view =
        PregameView::new(Arc::clone(store), gpu, format, size).with_scenes(Arc::clone(world));
    view.set_char_gen_system(Box::new(ac2_client::ScriptCharGen::new(host)));
    view.set_capture(ac2_client::CharGenCapture::new(
        capture.map(std::path::Path::to_path_buf),
    ));
    view
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
    /// The character left the world (`CharExitGame`): the world gives way to character selection
    /// after this frame.
    leave_world: bool,
    /// Client time origin (`Timer::cur_time` for the interpreter, materials and input stamps).
    started: Instant,
    /// Whether the OS input method is enabled (`CIme`'s active state; see `sync_ime`).
    ime_allowed: bool,
    grabbed: bool,
    /// Screenshots being encoded and written on a worker (`Device::SaveScreenshot`'s file).
    screenshots: Vec<Receiver<io::Result<PathBuf>>>,
}

struct App {
    client: Client,
    world: Arc<ac2_world::Ac2WorldSource>,
    controls: Option<Controls>,
    radius: i32,
    /// The scene's graphics options from `Preferences.ini` (`SmartBox::InitPreferences`), applied
    /// to each world view as it is made.
    scene_options: ac2_render::prefs::SceneOptions,
    state: Option<State>,
    store: Arc<ac2_db::ResourceStore>,
    /// Whether the pregame screens show (no start character).
    pregame: bool,
    /// The script library and script data character generation's `CharacterGenSystem` runs over:
    /// each pregame view gets a host of its own over them.
    chargen: ChargenSource,
    /// The `--chargen` capture file (`CharGenData`), made into the pregame view's capture.
    chargen_file: Option<std::path::PathBuf>,
    /// `width` / `height` (`Client+0x60` / `Client+100`): the initial window size, when given.
    window_size: Option<winit::dpi::PhysicalSize<u32>>,
}

impl State {
    fn now(&self) -> (u32, f64) {
        let elapsed = self.started.elapsed();
        (elapsed.as_millis() as u32, elapsed.as_secs_f64())
    }

    /// `CIme::Activate` / `CIme::Deactivate`: the OS input method is on while the keyboard types
    /// into a text element (`set_ime_allowed`), so composed text reaches the window.
    fn sync_ime(&mut self) {
        let on = self.controls.input.text_mode();
        if on != self.ime_allowed {
            self.window.set_ime_allowed(on);
            self.ime_allowed = on;
        }
    }

    /// Copies the presented frame and queues its save on a worker (`Device::SaveScreenshot`). The
    /// read-back waits for the GPU; the encode and the file write do not run on this thread.
    fn save_screenshot(&mut self, texture: &wgpu::Texture) {
        let (width, height) = (self.config.width, self.config.height);
        match screenshot::read_surface(&self.gpu, texture, self.format, width, height) {
            Ok(rgba) => {
                let (sender, receiver) = std::sync::mpsc::channel();
                let dir = ac2_prefs::paths::config_dir();
                std::thread::spawn(move || {
                    let _ = sender.send(screenshot::save_frame(&dir, width, height, &rgba));
                });
                self.screenshots.push(receiver);
            }
            Err(error) => {
                // The failure takes the same path as a failed write, to the chat.
                let (sender, receiver) = std::sync::mpsc::channel();
                let _ = sender.send(Err(io::Error::other(error)));
                self.screenshots.push(receiver);
            }
        }
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

/// `gmClient::RecvNotice_WorldName`: the window title carries the world's name, a modern stand-in
/// for the HUD's display. Retail reads the name through the efun
/// `gTurbineGunUtils::CallEFun_GetWorldName`, which the script VM does not bind yet.
fn world_title(info: &ac2_net::protocol::StringInfo) -> Option<String> {
    info.literal
        .as_deref()
        .filter(|name| !name.is_empty())
        .map(|name| format!("Asheron's Call 2 - {name}"))
}

fn show_world_name(window: &Window, info: &ac2_net::protocol::StringInfo) {
    match world_title(info) {
        Some(title) => window.set_title(&title),
        None => eprintln!("ac2: world name is not a literal; the title keeps its default"),
    }
}

/// The world names among a frame's session events (they arrive before character selection too).
pub(crate) fn show_world_names(window: &Window, events: &[SessionEvent]) {
    for event in events {
        if let SessionEvent::WorldName(info) = event {
            show_world_name(window, info);
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
        if self.state.as_ref().is_some_and(|s| s.leave_world) {
            self.back_to_character_selection();
        } else if let Some(state) = self.state.as_mut() {
            state.view = Some(view);
        }
    }

    /// `UIFlow::QueueUIMode(characterselection)` after the session ended: the world and its HUD
    /// go, and the pregame screens open at character selection with the character set the
    /// session kept (`gmCharacterSelectionUI`, the list `UIPersistantData` holds).
    fn back_to_character_selection(&mut self) {
        let resume = self.client.session().characters().cloned();
        let Some(state) = self.state.as_mut() else {
            return;
        };
        state.leave_world = false;
        state.commands.clear();
        state.selection = SelectionUi::default();
        state.sync_mouse_look(false);
        let mut view = new_pregame_view(
            &self.store,
            &self.world,
            &self.chargen,
            self.chargen_file.as_deref(),
            &state.gpu,
            state.format,
            (state.config.width, state.config.height),
        );
        if let Some(set) = resume {
            view = view.resume_at_character_selection(set);
        }
        state.pregame = Some(view);
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
        // `gmUIFlow::ListenToMessage`: the screenshot key's release saves the frame below.
        let screenshot = messages.iter().any(|message| {
            message.message == ac2_ui::GLOBAL_KEY_RELEASE
                && message.element.is_none()
                && message.data as u32 == screenshot::SCREENSHOT_KEY
        });
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
        show_world_names(&state.window, &events);
        for saved in screenshot::finished(&mut state.screenshots) {
            // Retail's SaveScreenshot shows no chat line, on success or failure.
            match saved {
                Ok(path) => eprintln!("ac2: screenshot {}", path.display()),
                Err(error) => eprintln!("ac2: screenshot: {error}"),
            }
        }
        for event in &events {
            match event {
                SessionEvent::Notice(GameplayNotice::DisplayStringInfo { kind, text }) => {
                    view.chat_display_info(*kind, text);
                }
                SessionEvent::ShellCommand { text, .. } => {
                    eprintln!("ac2: chat command not ported: {text}");
                }
                SessionEvent::LeftWorld => state.leave_world = true,
                _ => {}
            }
        }
        if state.leave_world {
            return;
        }
        for notice in self.client.take_notices() {
            if notice == ac2_client::Notice::ConnectionLost {
                eprintln!("ac2: connection lost");
                event_loop.exit();
                return;
            }
        }
        state.sync_mouse_look(self.client.camera().mouse_look);
        let frame = self.client.frame();
        // The HUD frameworks that ask the player: skills panel, shortcut bar, chat options.
        let script_notices = self.client.take_script_notices();
        let asked = view.frameworks(&mut self.client, &frame, &events, &script_notices);
        state.commands.extend(asked);
        // The options pages' applies: the scene's options, and the window's fullscreen state (set
        // only by a verified display change).
        for effect in view.take_options_effects() {
            match effect {
                ac2_client::OptionsEffect::Scene(options) => {
                    view.set_scene_options(options);
                    self.scene_options = options;
                }
                ac2_client::OptionsEffect::Fullscreen(on) => state
                    .window
                    .set_fullscreen(on.then_some(winit::window::Fullscreen::Borderless(None))),
            }
        }
        // The selection render: the object under the pointer, selected (or the selection
        // cleared) by `gmSelectionUI::Finish3DSelection`.
        if state.selection.wants_pick() {
            let screen = (state.config.width, state.config.height);
            let potential = view.pick_list(&frame, screen, view.pointer());
            let (targets, player_type) = pick_targets(&self.client, &frame, &potential);
            if let Some(outcome) = state.selection.finish(&targets, player_type) {
                state.commands.push(Command::Select(outcome.select));
                if outcome.use_selection {
                    state.commands.push(Command::UseSelection);
                }
                if outcome.open_examination {
                    view.open_examination();
                }
            }
        }
        // The particle systems the objects' FX started and stopped since the last frame.
        let starts = self.client.take_object_particle_starts();
        let stops = self.client.take_object_particle_stops();
        view.set_object_particles(&starts, &stops);
        view.update(&state.gpu, &frame, LOAD_BUDGET);
        // The HUD's sounds play from the client's sound manager on its next frame.
        if let Some(audio) = self.client.audio_mut() {
            ac2_client::play_ui_sounds(audio, &view.take_ui_sounds());
        }
        let (_, time) = state.now();
        let Some(output) = ac2_gpu::acquire_frame(&state.surface, &state.gpu.device, &state.config)
        else {
            return;
        };
        let target = output.texture.create_view(&Default::default());
        let aspect = state.config.width as f32 / state.config.height as f32;
        let drawn = view.prepare(&state.gpu, &frame, aspect, time);
        if view.take_deselect() {
            state.commands.push(Command::Select(None));
        }
        if drawn {
            view.render(&state.gpu, &target, &state.depth, true);
        } else {
            render_to(
                &state.gpu.device,
                &state.gpu.queue,
                &target,
                &state.depth,
                [0.0; 3],
                |_| {},
            );
        }
        if screenshot {
            state.save_screenshot(&output.texture);
        }
        state.gpu.queue.present(output);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let mut attributes = Window::default_attributes().with_title("Asheron's Call 2");
        if let Some(size) = self.window_size {
            attributes = attributes.with_inner_size(size);
        }
        let window = Arc::new(event_loop.create_window(attributes).expect("window"));
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
        // `COPY_SRC` lets the screenshot key read the presented frame back.
        let copy = if caps.usages.contains(wgpu::TextureUsages::COPY_SRC) {
            wgpu::TextureUsages::COPY_SRC
        } else {
            wgpu::TextureUsages::empty()
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | copy,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            color_space: Default::default(),
        };
        surface.configure(&gpu.device, &config);
        let depth = depth_view(&gpu.device, &config);
        let pregame = self.pregame.then(|| {
            new_pregame_view(
                &self.store,
                &self.world,
                &self.chargen,
                self.chargen_file.as_deref(),
                &gpu,
                format,
                (config.width, config.height),
            )
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
            leave_world: false,
            controls: {
                // The HUD's tree takes the UI actions (mouse over panels, the world's wheel).
                let mut controls = self.controls.take().expect("controls");
                controls.ui_attached = true;
                controls
            },
            started: Instant::now(),
            ime_allowed: false,
            grabbed: false,
            screenshots: Vec::new(),
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
        state.sync_ime();
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
                        for event in ac2_ui::typed_text(text) {
                            state.input(event);
                        }
                    }
                }
            }
            // `CIme::ProcessMessage`: the OS input method's committed text types like keyboard
            // text. The composition (preedit) is drawn by the OS and not shown in the entry line.
            WindowEvent::Ime(Ime::Commit(text)) => {
                for event in ac2_ui::typed_text(&text) {
                    state.input(event);
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
    // `Turbine::Client::IsAlreadyRunning`: a second client stops here; the lock lasts for the run.
    let _instance = instance::claim()?;
    // `Turbine::Client::SetLanguage` -> `CLCache::SetLanguageInternal`: the language's local DAT
    // (its string tables) replaces the English one the mount opens by default.
    let mut dat_options = ac2_dat::DatOpenOptions::new(&args.dat_dir);
    if let Some(language) = &args.language {
        dat_options.local_dat_file_name =
            ac2_dat::local_dat_for_language(args.dat_dir.as_ref(), language)?;
    }
    let dataset = ac2_dat::DatDataset::open(dat_options).map_err(|e| e.to_string())?;
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
    ac2_db::install_ac2_property_types(&store).map_err(|e| e.to_string())?;
    // The preferences are read here, before the frame loop; the frame thread never reads the file.
    let prefs_path = match &args.prefs {
        Some(name) => ac2_prefs::paths::resolve(name),
        None => ac2_prefs::paths::default_file(),
    };
    let prefs = ac2_client::ClientPrefs::load(prefs_path, Arc::clone(&store))
        .map_err(|e| format!("preferences: {e}"))?;
    let dat_world = DatWorld::new(Arc::clone(&store))?;
    let world = Arc::clone(dat_world.world());
    // `gmClient::InitKeymap`: the player's keymap files, else the DATs' maps.
    let keymap_files = ac2_ui::keymap_file::KeymapFiles::user();
    let controls = Controls::load_with(&store, Some(&keymap_files))
        .map_err(|e| e.to_string())?
        .ok_or("the DATs hold no action map")?;
    let mut session = OnlineSession::connect(OnlineConfig {
        server: args.server_address()?,
        account: args.account,
        credential: Credential::from_ticket(&args.ticket),
        version: args.version,
        // Without a start character the session waits at the character set for the
        // selection screen (`gmCharacterSelectionUI`).
        character: match &args.character {
            Some(name) => CharacterChoice::Name(name.clone()),
            None => CharacterChoice::Wait,
        },
        // Retail always exports an RSA-wrapped session key (`ClientNet::Init`); the servers we
        // talk to cannot unwrap it, so the logon carries none (auth flags 0).
        server_key: None,
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
    let script_data_for_chargen = script_data.clone();
    let host = ac2_script::host::ScriptHost::new(Arc::clone(&program), Box::new(script_data));
    client.attach_scripts(host, ac2_client::SCRIPT_VM_FUNCTIONS);
    // `SoundManager`: the speakers, or silence without an output device.
    client.attach_audio(ac2_client::sound_manager(Arc::clone(&store), speakers()));
    client.set_portal_sound_ids(ac2_client::portal_sound_ids(&store));
    client.set_rain_sound_ids(ac2_client::rain_sound_ids(&store));
    client.set_fx_scripts(ac2_client::load_fx_scripts(&store));
    let scene_options = prefs.scene_options();
    client.apply_sound_preferences(prefs.bank());
    client.attach_prefs(prefs);
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let mut app = App {
        client,
        world,
        controls: Some(controls),
        radius: args.radius,
        scene_options,
        state: None,
        store,
        pregame: args.character.is_none(),
        chargen: ChargenSource {
            program: Arc::clone(&program),
            data: script_data_for_chargen,
        },
        chargen_file: args.chargen.as_deref().map(ac2_prefs::paths::config_file),
        // `Client+0x60` / `+100` start at 0x400 x 0x300; either argument alone keeps the other.
        window_size: (args.width.is_some() || args.height.is_some()).then(|| {
            winit::dpi::PhysicalSize::new(args.width.unwrap_or(1024), args.height.unwrap_or(768))
        }),
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

#[cfg(test)]
mod tests {
    use super::world_title;
    use ac2_net::protocol::StringInfo;

    #[test]
    fn the_title_carries_a_literal_world_name() {
        assert_eq!(
            world_title(&StringInfo::literal("Darktide")),
            Some("Asheron's Call 2 - Darktide".to_owned())
        );
        assert_eq!(world_title(&StringInfo::literal("")), None);
        assert_eq!(world_title(&StringInfo::default()), None);
    }
}
