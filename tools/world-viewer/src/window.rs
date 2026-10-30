//! The interactive window: a wgpu surface, the client's own view fed an offline frame, the
//! player (or the fly camera) and the egui panel over it.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_client::view::ClientView;
use ac2_core::DataId;
use ac2_gpu::{acquire_frame, Gpu, DEPTH_FORMAT};
use ac2_render::scene::DAY_SECONDS;
use ac2_world::Ac2WorldSource;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::fly::{FlyCamera, FlyInput};
use crate::frame::client_frame;
use crate::gui::Gui;
use crate::hud_input::HudInput;
use crate::panel::{Command, Mode, Panel, PanelView};
use crate::races::PlayerModel;
use crate::session::Session;
use crate::walk::{actions, Landing, Walk, WalkInput};

/// Frame-thread time a frame spends taking finished scene loads.
const LOAD_BUDGET: Duration = Duration::from_millis(4);

/// What the window needs from `main`.
pub struct Setup {
    /// The DATs, which the world map (M) opens with a store of its own.
    pub dat_dir: std::path::PathBuf,
    pub world: Arc<Ac2WorldSource>,
    pub store: Arc<ac2_db::ResourceStore>,
    pub models: Vec<PlayerModel>,
    /// Why there are no models, shown in the panel.
    pub models_error: Option<String>,
    pub landing: Landing,
    pub radius: i32,
    pub fly_first: bool,
    pub fly: FlyCamera,
    pub time_of_day: f64,
    /// The AC1 creatures of `--ace-weenies`.
    pub ac1: Option<crate::ac1::Ac1Showcase>,
}

pub fn run(setup: Setup) {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App {
        setup: Some(setup),
        state: None,
    };
    event_loop.run_app(&mut app).expect("event loop runs");
}

struct App {
    setup: Option<Setup>,
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    gpu: Gpu,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    view: ClientView,
    gui: Gui,
    panel: Panel,
    walk: Walk,
    fly: FlyCamera,
    time_of_day: f64,
    keys: HashSet<KeyCode>,
    looking: bool,
    last_frame: Instant,
    /// Client time origin (`Timer::local_time`) for animated materials and physics.
    started: Instant,
    fps: f32,
    models_error: Option<String>,
    dat_dir: std::path::PathBuf,
    /// The world map under test, made on its first opening; whether its window is open.
    map: Option<ac2hero_map::WorldMap>,
    map_open: bool,
    /// The HUD's session (`session.rs`); `None` when it could not start, and the HUD then runs
    /// only its session-free parts (the guide, the world map, the Tracker).
    session: Option<Session>,
    /// The HUD's input through the retail key maps (`hud_input.rs`); `None` without an action
    /// map in the DATs.
    hud_input: Option<HudInput>,
    ac1: Option<crate::ac1::Ac1Showcase>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(mut setup) = self.setup.take() else {
            return;
        };
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes().with_title("AC2 world viewer (offline)"),
                )
                .expect("window"),
        );
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).expect("surface");
        let gpu = Gpu::new(Some(&surface), instance).expect("GPU adapter");
        ac2_render::set_gpu_block_compression(gpu.block_compression());
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
            color_space: Default::default(),
        };
        surface.configure(&gpu.device, &config);
        let depth = depth_view(&gpu.device, &config);
        let mut view = ClientView::new(
            Arc::clone(&setup.world),
            &gpu,
            format,
            setup.radius,
            (config.width, config.height),
        );
        if let Some(showcase) = &setup.ac1 {
            view.set_foreign_models(showcase.models());
        }
        let mut walk = Walk::new(&setup.world, setup.store.clone(), 0.0);
        let mode = if setup.fly_first || setup.models.is_empty() {
            Mode::Fly
        } else {
            Mode::Walk
        };
        if let Some(model) = setup.models.first() {
            walk.set_model(model.visual);
            walk.teleport(setup.landing);
        }
        let session = match Session::start(&setup.store, &setup.world, setup.landing) {
            Ok(mut session) => {
                // The AC1 weapons go in the pack; the hand shows what the inventory wields.
                if let Some(showcase) = setup.ac1.as_mut() {
                    session.give_items(&showcase.weapons.items());
                    showcase.weapons.current = None;
                }
                Some(session)
            }
            Err(error) => {
                eprintln!("world-viewer: no HUD session: {error}");
                None
            }
        };
        let hud_input = HudInput::load(&setup.store);
        let gui = Gui::new(&window, &gpu, format);
        let panel = Panel::new(setup.models, mode, setup.landing);
        self.state = Some(State {
            window,
            gpu,
            surface,
            config,
            depth,
            view,
            gui,
            panel,
            walk,
            fly: setup.fly,
            time_of_day: setup.time_of_day,
            keys: Default::default(),
            looking: false,
            last_frame: Instant::now(),
            started: Instant::now(),
            fps: 0.0,
            models_error: setup.models_error,
            dat_dir: setup.dat_dir,
            map: None,
            map_open: false,
            session,
            hud_input,
            ac1: setup.ac1,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let consumed = state.gui.event(&state.window, &event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.config.width = size.width.max(1);
                state.config.height = size.height.max(1);
                state.surface.configure(&state.gpu.device, &state.config);
                state.depth = depth_view(&state.gpu.device, &state.config);
                state.view.resize(state.config.width, state.config.height);
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(hud) = &mut state.hud_input {
                    hud.cursor(position.x, position.y);
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(hud) = &mut state.hud_input {
                    hud.focus(focused);
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(hud) = &mut state.hud_input {
                    hud.modifiers(modifiers.state().alt_key(), modifiers.state().super_key());
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // The HUD's key maps see every key; a HUD text field (the chat's entry, the
                // guide's search, the Tracker's pattern) keeps the typing from the viewer's keys.
                if let Some(hud) = &mut state.hud_input {
                    let typing = hud.text_mode();
                    hud.key(&event);
                    if typing {
                        return;
                    }
                }
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                if event.state == ElementState::Pressed {
                    if code == KeyCode::Escape {
                        event_loop.exit();
                    }
                    if !state.gui.wants_keyboard() && !event.repeat {
                        state.key_pressed(code);
                    }
                    state.keys.insert(code);
                } else {
                    state.key_released(code);
                    state.keys.remove(&code);
                }
            }
            WindowEvent::MouseInput {
                button,
                state: button_state,
                ..
            } => {
                let pressed = button_state == ElementState::Pressed;
                if pressed && (consumed || state.gui.wants_pointer()) {
                    return;
                }
                if let Some(hud) = &mut state.hud_input {
                    hud.button(button, pressed);
                }
                match button {
                    MouseButton::Left => {
                        if state.panel.mode == Mode::Fly {
                            state.looking = pressed;
                        }
                    }
                    MouseButton::Right if state.panel.mode == Mode::Walk => {
                        state.walk.camera_action(actions::MOUSE_LOOK_HOLD, pressed);
                    }
                    _ => {}
                }
            }
            // The wheel scrolls the HUD panel under the pointer, else zooms the camera
            // (`HudInput::frame`).
            WindowEvent::MouseWheel { delta, phase, .. } if !state.gui.wants_pointer() => {
                let scale = state.window.scale_factor();
                if let Some(hud) = &mut state.hud_input {
                    hud.wheel(delta, phase, scale);
                }
            }
            // A frame whose HUD confirmed the quit question closes the viewer.
            WindowEvent::RedrawRequested if state.frame() => event_loop.exit(),
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let (Some(state), DeviceEvent::MouseMotion { delta }) = (self.state.as_mut(), event) else {
            return;
        };
        match state.panel.mode {
            Mode::Fly if state.looking => state.fly.turn(delta.0 as f32, delta.1 as f32),
            Mode::Walk => state.walk.mouse_look(delta.0 as f32, delta.1 as f32),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

impl State {
    fn key_pressed(&mut self, code: KeyCode) {
        let step = DAY_SECONDS / 24.0;
        match code {
            KeyCode::BracketLeft => self.time_of_day -= step,
            KeyCode::BracketRight => self.time_of_day += step,
            KeyCode::F1 => self.panel.visible = !self.panel.visible,
            KeyCode::PageUp | KeyCode::PageDown => {
                if let Some(showcase) = self.ac1.as_mut() {
                    showcase
                        .weapons
                        .cycle(if code == KeyCode::PageUp { 1 } else { -1 });
                }
            }
            KeyCode::KeyM => self.map_open = !self.map_open,
            KeyCode::KeyG => self.view.toggle_guide(),
            KeyCode::KeyT => self.view.toggle_tracker(),
            KeyCode::Tab => {
                let mode = match self.panel.mode {
                    Mode::Walk => Mode::Fly,
                    Mode::Fly => Mode::Walk,
                };
                self.set_mode(mode);
            }
            KeyCode::KeyV => self.walk.camera_action(actions::TOGGLE_FIRST_PERSON, true),
            KeyCode::Home => self.walk.camera_action(actions::RESET, true),
            KeyCode::ArrowLeft => self.walk.camera_action(actions::ROTATE_LEFT, true),
            KeyCode::ArrowRight => self.walk.camera_action(actions::ROTATE_RIGHT, true),
            _ => {
                if let Some(action) = numpad_camera_action(code) {
                    self.walk.camera_action(action, true);
                }
            }
        }
        self.time_of_day = self.time_of_day.rem_euclid(DAY_SECONDS);
    }

    fn key_released(&mut self, code: KeyCode) {
        match code {
            KeyCode::ArrowLeft => self.walk.camera_action(actions::ROTATE_LEFT, false),
            KeyCode::ArrowRight => self.walk.camera_action(actions::ROTATE_RIGHT, false),
            _ => {
                if let Some(action) = numpad_camera_action(code) {
                    self.walk.camera_action(action, false);
                }
            }
        }
    }

    /// The model the panel has selected.
    fn model_visual(&self) -> Option<DataId> {
        self.panel.models.get(self.panel.model).map(|m| m.visual)
    }

    /// Switches between the fly camera and the player, carrying the position across.
    fn set_mode(&mut self, mode: Mode) {
        if mode == self.panel.mode {
            return;
        }
        match mode {
            Mode::Walk => {
                let landing = self.landing_at(self.fly.position.x, self.fly.position.y);
                self.panel.set_landing(landing);
                self.walk.teleport(landing);
            }
            Mode::Fly => {
                if let Some(frame) = self.walk.camera_frame() {
                    let size = self.walk.block_size();
                    let world = |cell: u32, origin: [f32; 3]| {
                        ac2_math::Vec3::new(
                            f32::from((cell >> 24) as u8) * size + origin[0],
                            f32::from((cell >> 16) as u8) * size + origin[1],
                            origin[2],
                        )
                    };
                    let eye = world(frame.cell, frame.origin);
                    let target = world(frame.cell, frame.target);
                    self.fly = FlyCamera::looking_at(eye, target);
                }
            }
        }
        self.panel.mode = mode;
    }

    /// The landing under a world position.
    fn landing_at(&self, x: f32, y: f32) -> Landing {
        let size = self.walk.block_size();
        let block = |axis: f32| (axis / size).floor().clamp(0.0, 254.0) as u8;
        let block = (block(x), block(y));
        Landing {
            block,
            x: x - f32::from(block.0) * size,
            y: y - f32::from(block.1) * size,
        }
    }

    fn walk_input(&self) -> WalkInput {
        if self.gui.wants_keyboard() {
            return WalkInput::default();
        }
        let held = |code| f32::from(u8::from(self.keys.contains(&code)));
        WalkInput {
            forward: held(KeyCode::KeyW) - held(KeyCode::KeyS),
            strafe: held(KeyCode::KeyE) - held(KeyCode::KeyQ),
            turn: held(KeyCode::KeyA) - held(KeyCode::KeyD),
            jump: self.keys.contains(&KeyCode::Space),
        }
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::SetMode(mode) => self.set_mode(mode),
            Command::SetModel(index) => {
                if let Some(model) = self.panel.models.get(index) {
                    self.walk.set_model(model.visual);
                }
            }
            Command::Teleport(landing) => {
                self.walk.teleport(landing);
                if self.panel.mode == Mode::Fly {
                    self.panel.mode = Mode::Walk;
                }
            }
        }
    }

    /// One frame; returns whether the HUD's quit question was confirmed.
    fn frame(&mut self) -> bool {
        let now = Instant::now();
        let seconds = (now - self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;
        if seconds > 0.0 {
            self.fps += (1.0 / seconds - self.fps) * 0.1;
        }
        let time = self.started.elapsed().as_secs_f64();
        let input = self.walk_input();
        if self.panel.mode == Mode::Fly {
            let held = |code| f32::from(u8::from(self.keys.contains(&code)));
            let typing = self.gui.wants_keyboard();
            let axis = |plus, minus| {
                if typing {
                    0.0
                } else {
                    held(plus) - held(minus)
                }
            };
            self.fly.advance(
                FlyInput {
                    forward: axis(KeyCode::KeyW, KeyCode::KeyS),
                    right: axis(KeyCode::KeyD, KeyCode::KeyA),
                    up: axis(KeyCode::Space, KeyCode::KeyC),
                    fast: self.keys.contains(&KeyCode::ShiftLeft),
                },
                seconds,
            );
        }
        self.walk.speed = self.panel.speed;
        self.walk.update(seconds, time, input);

        let mut frame = client_frame(
            self.panel.mode,
            &self.walk,
            &self.fly,
            self.model_visual(),
            self.time_of_day,
        );
        if let Some(showcase) = self.ac1.as_mut() {
            let body = self
                .panel
                .models
                .get(self.panel.model)
                .map(|m| (m.race, m.sex));
            let on = self.panel.ac1_motion.get_or_insert(false);
            if let (true, Some((race, sex)), Some(player)) = (*on, body, frame.player.as_mut()) {
                let applied = showcase.motion.apply(player, race, sex, &input, time);
                self.panel.ac1_motion_note =
                    (!applied).then(|| "no AC1 body for this race".to_string());
            }
        }
        if let Some(hud) = &mut self.hud_input {
            hud.frame(&mut self.view, &mut self.walk);
        }
        let quit = match self.session.as_mut() {
            Some(session) => session.frame(&mut self.view, &mut frame, f64::from(seconds)),
            None => {
                self.view.offline_frameworks(&frame);
                false
            }
        };
        if let Some(showcase) = self.ac1.as_mut() {
            let walk = &self.walk;
            let creatures = showcase.objects(frame.player.as_ref(), time, |cell, origin| {
                walk.ground_height(cell, origin)
            });
            frame.objects.extend(creatures);
            showcase.weapons.hold(&mut frame);
            if let Some(model) = self.panel.models.get(self.panel.model) {
                showcase.wear(&mut frame, model.race, model.sex);
            }
            let block_size = self.walk.block_size();
            showcase
                .fx
                .frame(&mut self.view, &mut frame, time, block_size);
        }
        self.view.update(&self.gpu, &frame, LOAD_BUDGET);
        let Some(output) = acquire_frame(&self.surface, &self.gpu.device, &self.config) else {
            return quit;
        };
        let target = output.texture.create_view(&Default::default());
        let aspect = self.config.width as f32 / self.config.height as f32;
        self.view.prepare(&self.gpu, &frame, aspect, time);
        self.view.render(&self.gpu, &target, &self.depth, true);

        let mut commands = Vec::new();
        let message = self
            .walk
            .message
            .clone()
            .or_else(|| self.models_error.clone());
        let status = self.walk.status();
        let fps = self.fps;
        let mut day = self.time_of_day;
        let panel = &mut self.panel;
        let (map, map_open, dat_dir) = (&mut self.map, &mut self.map_open, &self.dat_dir);
        let player = status
            .as_ref()
            .map(|s| ac2hero_map::player_at(s.cell, s.origin, s.heading_degrees));
        let mut travel = None;
        self.gui.frame(
            &self.window,
            &self.gpu,
            &target,
            [self.config.width, self.config.height],
            |ui| {
                commands = panel.show(
                    ui,
                    PanelView {
                        status,
                        message: message.as_deref(),
                        fps,
                        time_of_day: &mut day,
                        particle_count: 0,
                    },
                );
                if *map_open {
                    let map = map.get_or_insert_with(|| {
                        let world = ac2hero_map::open_world(dat_dir).ok().flatten();
                        let mut map =
                            ac2hero_map::WorldMap::new(ui.ctx(), ac2_guide::baked(), world);
                        map.set_travel(true);
                        map
                    });
                    let size = ui.ctx().content_rect().size();
                    egui::Window::new("World map")
                        .open(map_open)
                        .default_size(size * 0.8)
                        .default_pos(egui::pos2(size.x * 0.1, size.y * 0.08))
                        .show(ui.ctx(), |ui| {
                            if let Some(ac2hero_map::MapAction::Travel(at)) = map.ui(ui, player) {
                                travel = Some(at);
                            }
                        });
                }
            },
        );
        if let Some(at) = travel {
            let (x, y) = ac2hero_map::world_metres(at);
            commands.push(Command::Teleport(self.landing_at(x, y)));
        }
        self.time_of_day = day;
        self.gpu.queue.present(output);
        for command in commands {
            self.apply(command);
        }
        quit
    }
}

fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("viewer depth"),
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

/// The camera keys of the retail main key map (`0x41000001`, the numpad): 8 and 2 tilt, 4 and 6
/// turn, 5 resets, 7 looks at the ground and 9 at the player, 1 and 3 swing out, 0 is the
/// shoulder view, `.` first person, Enter the map view, and + and - zoom.
fn numpad_camera_action(code: KeyCode) -> Option<u32> {
    Some(match code {
        KeyCode::Numpad8 => actions::PITCH_UP,
        KeyCode::Numpad2 => actions::PITCH_DOWN,
        KeyCode::Numpad4 => actions::ROTATE_LEFT,
        KeyCode::Numpad6 => actions::ROTATE_RIGHT,
        KeyCode::Numpad5 => actions::RESET,
        KeyCode::Numpad7 => actions::LOOK_AT_GROUND,
        KeyCode::Numpad9 => actions::LOOK_AT_SELF,
        KeyCode::Numpad1 => actions::FAKE_TRANSITION,
        KeyCode::Numpad3 => actions::SWING_OUT_RIGHT,
        KeyCode::Numpad0 => actions::SHOULDER_VIEW,
        KeyCode::NumpadDecimal => actions::TOGGLE_FIRST_PERSON,
        KeyCode::NumpadEnter => actions::MAP_VIEW,
        KeyCode::NumpadAdd => actions::ZOOM_SCALE_IN,
        KeyCode::NumpadSubtract => actions::ZOOM_SCALE_OUT,
        _ => return None,
    })
}
