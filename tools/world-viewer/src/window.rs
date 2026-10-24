//! The interactive window: a wgpu surface, the fly camera and block residency around it.

use std::sync::Arc;
use std::time::Instant;

use ac2_gpu::{render_to, Gpu, DEPTH_FORMAT};
use ac2_world::Ac2WorldSource;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::fly::{FlyCamera, FlyInput};
use crate::scene::{Scene, DAY_SECONDS};

pub fn run(world: Arc<Ac2WorldSource>, block: (u8, u8), radius: i32) {
    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App {
        world,
        block,
        radius,
        state: None,
    };
    event_loop.run_app(&mut app).expect("event loop runs");
}

struct App {
    world: Arc<Ac2WorldSource>,
    block: (u8, u8),
    radius: i32,
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    gpu: Gpu,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    scene: Scene,
    fly: FlyCamera,
    keys: std::collections::HashSet<KeyCode>,
    looking: bool,
    last_frame: Instant,
    /// Client time origin (`Timer::local_time`) for animated materials.
    started: Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
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
        let mut scene = Scene::new(self.world.clone(), &gpu, format, self.radius);
        scene.set_daytime();
        scene.update_residency(&gpu, self.block);
        let fly = crate::start_camera(&scene, self.block);
        self.state = Some(State {
            window,
            gpu,
            surface,
            config,
            depth,
            scene,
            fly,
            keys: Default::default(),
            looking: false,
            last_frame: Instant::now(),
            started: Instant::now(),
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                state.config.width = size.width.max(1);
                state.config.height = size.height.max(1);
                state.surface.configure(&state.gpu.device, &state.config);
                state.depth = depth_view(&state.gpu.device, &state.config);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                let pressed = event.state == ElementState::Pressed;
                if pressed {
                    let step = DAY_SECONDS / 24.0;
                    match code {
                        KeyCode::Escape => event_loop.exit(),
                        KeyCode::BracketLeft => state.scene.time_of_day -= step,
                        KeyCode::BracketRight => state.scene.time_of_day += step,
                        _ => {}
                    }
                    state.scene.time_of_day = state.scene.time_of_day.rem_euclid(DAY_SECONDS);
                    state.keys.insert(code);
                } else {
                    state.keys.remove(&code);
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state: button,
                ..
            } => state.looking = button == ElementState::Pressed,
            WindowEvent::RedrawRequested => state.frame(),
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let (Some(state), DeviceEvent::MouseMotion { delta }) = (self.state.as_mut(), event) {
            if state.looking {
                state.fly.turn(delta.0 as f32, delta.1 as f32);
            }
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.window.request_redraw();
        }
    }
}

impl State {
    fn frame(&mut self) {
        let now = Instant::now();
        let seconds = (now - self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;
        let held = |code| f32::from(u8::from(self.keys.contains(&code)));
        let input = FlyInput {
            forward: held(KeyCode::KeyW) - held(KeyCode::KeyS),
            right: held(KeyCode::KeyD) - held(KeyCode::KeyA),
            up: held(KeyCode::Space) - held(KeyCode::KeyC),
            fast: self.keys.contains(&KeyCode::ShiftLeft),
        };
        self.fly.advance(input, seconds);
        let block = self.scene.block_at(self.fly.position);
        self.scene.update_residency(&self.gpu, block);

        let Ok(output) = self.surface.get_current_texture() else {
            self.surface.configure(&self.gpu.device, &self.config);
            return;
        };
        let view = output.texture.create_view(&Default::default());
        let aspect = self.config.width as f32 / self.config.height as f32;
        let time = self.started.elapsed().as_secs_f64();
        let lighting = self
            .scene
            .prepare(&self.gpu, &self.fly.camera(), aspect, time);
        let scene = &self.scene;
        render_to(
            &self.gpu.device,
            &self.gpu.queue,
            &view,
            &self.depth,
            lighting.fog_color,
            |pass| scene.draw(pass),
        );
        output.present();
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
