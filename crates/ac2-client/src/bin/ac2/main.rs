//! The Asheron's Call 2 client in a window: log on to a server, enter the world and play with
//! the retail controls (the DATs' key maps and action map: W/S/A/D or the arrows to move, Q/E to
//! strafe, the wheel to zoom, the middle button for mouse look, ...), drawn through the shared
//! scene and seen from the retail camera.
//!
//! ```text
//! ac2 --dat-dir DIR [--server 127.0.0.1:9000] [--account local] [--version 7160]
//!     [--character NAME] [--client-exe PATH] [--radius 1]
//! ```

mod keys;

use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_client::view::ClientView;
use ac2_client::{frame_input, Client, DatWorld, FrameInput};
use ac2_gpu::{render_to, Gpu, DEPTH_FORMAT};
use ac2_net::Credential;
use ac2_ui::{Controls, InputEvent};
use ac2_weenie::{CharacterChoice, OnlineConfig, OnlineSession};
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
    character: CharacterChoice,
    client_exe: Option<String>,
    radius: i32,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        dat_dir: std::env::var("AC2_DAT_DIR").unwrap_or_default(),
        server: "127.0.0.1:9000".into(),
        account: "local".into(),
        version: ac2_net::LEGIONS_VERSION.into(),
        character: CharacterChoice::First,
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
            "--character" => args.character = CharacterChoice::Name(value),
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
    view: ClientView,
    controls: Controls,
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
        let input = frame_input(state.controls.frame());
        if let Err(error) = self.client.use_time(&input) {
            eprintln!("ac2: {error}");
            event_loop.exit();
            return;
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
        state.view.update(&state.gpu, &frame, LOAD_BUDGET);
        let Ok(output) = state.surface.get_current_texture() else {
            state.surface.configure(&state.gpu.device, &state.config);
            return;
        };
        let target = output.texture.create_view(&Default::default());
        let aspect = state.config.width as f32 / state.config.height as f32;
        let (_, time) = state.now();
        let drawn = state.view.prepare(&state.gpu, &frame, aspect, time);
        let view = &state.view;
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
        let view = ClientView::new(Arc::clone(&self.world), &gpu, format, self.radius);
        self.state = Some(State {
            window,
            gpu,
            surface,
            config,
            depth,
            view,
            controls: self.controls.take().expect("controls"),
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
            }
            WindowEvent::Focused(focused) => {
                let (_, now) = state.now();
                state.controls.focus(focused, now);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat {
                    return;
                }
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(scan) = keys::dik(code) {
                        state.input(InputEvent::Key {
                            scan,
                            pressed: event.state == ElementState::Pressed,
                        });
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
    let session = OnlineSession::connect(OnlineConfig {
        server: args.server.parse().map_err(|e| format!("--server: {e}"))?,
        account: args.account,
        credential: Credential::Account,
        version: args.version,
        character: args.character,
        server_key,
        language: 0,
        dat_iterations: ac2_weenie::dat_iterations(&dataset)?,
    })
    .map_err(|e| e.to_string())?;
    let mut client = Client::new(session, Arc::new(dat_world));
    // Enter the world before opening the window.
    let deadline = Instant::now() + Duration::from_secs(60);
    while !client.frame().in_world {
        if Instant::now() > deadline {
            return Err("no world entry".into());
        }
        client.use_time(&FrameInput::default())?;
        std::thread::sleep(Duration::from_millis(16));
    }
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let mut app = App {
        client,
        world,
        controls: Some(controls),
        radius: args.radius,
        state: None,
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
