//! The window: winit events into the client's input stack (`ac2_ui::Controls`, as the world
//! viewer's `hud_input.rs` and the client's window take them), the wiki's frame, and its
//! surfaces drawn by the client's UI overlay (`ac2_gpu::UiOverlay`).

use std::sync::Arc;
use std::time::Instant;

use ac2_client::keys;
use ac2_db::ResourceStore;
use ac2_gpu::{render_to, Gpu, UiLayer, UiOverlay, DEPTH_FORMAT};
use ac2_ui::{wheel_event_delta, Controls, InputEvent, WheelAccumulator};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::wiki::{coord_text, Outcome, Wiki};
use crate::Args;

pub struct App {
    store: Arc<ResourceStore>,
    args: Args,
    state: Option<State>,
}

struct State {
    window: Arc<Window>,
    gpu: Gpu,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth: wgpu::TextureView,
    overlay: UiOverlay,
    wiki: Wiki,
    controls: Controls,
    wheel: WheelAccumulator,
    super_held: bool,
    started: Instant,
}

impl App {
    pub fn new(store: Arc<ResourceStore>, args: Args) -> Self {
        Self {
            store,
            args,
            state: None,
        }
    }
}

fn title(style: ac2_ui::HudStyle) -> String {
    let look = match style {
        ac2_ui::HudStyle::Modern => "custom UI",
        ac2_ui::HudStyle::Classic => "default UI",
    };
    format!("AC2 Vault wiki ({look}; F7 switches)")
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let (w, h) = self.args.size;
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(title(self.args.style))
                        .with_inner_size(winit::dpi::PhysicalSize::new(w, h))
                        .with_min_inner_size(winit::dpi::PhysicalSize::new(640, 480)),
                )
                .expect("window"),
        );
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).expect("surface");
        let gpu = Gpu::new(Some(&surface), instance).expect("GPU adapter");
        let caps = surface.get_capabilities(&gpu.adapter);
        // Gamma-space output, as the client's back buffer.
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
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
        let overlay = UiOverlay::new(&gpu, format);
        eprintln!(
            "vault-wiki: building the {} UI from the DATs...",
            match self.args.style {
                ac2_ui::HudStyle::Modern => "custom",
                ac2_ui::HudStyle::Classic => "default",
            }
        );
        let wiki = Wiki::build(
            &self.store,
            self.args.style,
            config.width,
            config.height,
            self.args.page.as_deref(),
        )
        .unwrap_or_else(|e| {
            eprintln!("vault-wiki: {e}");
            std::process::exit(1);
        });
        let files = ac2_ui::keymap_file::KeymapFiles::user();
        let mut controls = Controls::load_with(&self.store, Some(&files))
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                eprintln!("vault-wiki: the DATs hold no action map");
                std::process::exit(1);
            });
        // The tree takes the UI actions (the pointer, presses, the wheel), as the client's HUD.
        controls.ui_attached = true;
        eprintln!(
            "vault-wiki: ready ({} x {}, {:?})",
            config.width,
            config.height,
            gpu.adapter.get_info().backend
        );
        event_loop.set_control_flow(ControlFlow::Poll);
        self.state = Some(State {
            window,
            gpu,
            surface,
            config,
            depth,
            overlay,
            wiki,
            controls,
            wheel: WheelAccumulator::default(),
            super_held: false,
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
                state.wiki.resize(state.config.width, state.config.height);
            }
            WindowEvent::CursorMoved { position, .. } => state.input(InputEvent::Cursor {
                x: position.x as i32,
                y: position.y as i32,
            }),
            WindowEvent::Focused(focused) => {
                let now = state.now().1;
                state.controls.focus(focused, now);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                state.controls.set_alt(modifiers.state().alt_key());
                state.super_held = modifiers.state().super_key();
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
            WindowEvent::MouseWheel { delta, phase, .. } => {
                if phase == TouchPhase::Started {
                    state.wheel.start_gesture();
                }
                let notches = match delta {
                    MouseScrollDelta::LineDelta(_, y) => state.wheel.line_delta(y),
                    MouseScrollDelta::PixelDelta(p) => {
                        state.wheel.pixel_delta(p.y / state.window.scale_factor())
                    }
                };
                for _ in 0..notches.unsigned_abs() {
                    state.input(InputEvent::Wheel {
                        delta: wheel_event_delta(notches.signum()),
                    });
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                let typing = state.wiki.text_mode();
                if pressed && !event.repeat && !typing {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => return event_loop.exit(),
                        PhysicalKey::Code(KeyCode::F7) => return state.switch_style(&self.store),
                        _ => {}
                    }
                }
                state.key(&event);
            }
            WindowEvent::RedrawRequested if state.frame() => event_loop.exit(),
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
    /// `Timer::cur_time` for the input manager: milliseconds and seconds since the start.
    fn now(&self) -> (u32, f64) {
        let elapsed = self.started.elapsed();
        (elapsed.as_millis() as u32, elapsed.as_secs_f64())
    }

    fn input(&mut self, event: InputEvent) {
        let (stamp, now) = self.now();
        self.controls.event(event, stamp, now);
    }

    /// A key: the clipboard shortcut it is (macOS), else its `DIK` key and, pressed, its text.
    /// Outside text mode a held key is one press.
    fn key(&mut self, event: &winit::event::KeyEvent) {
        if event.repeat && !self.wiki.text_mode() {
            return;
        }
        let pressed = event.state == ElementState::Pressed;
        let shortcut = keys::clipboard_shortcut(
            keys::ShortcutPlatform::current(),
            self.super_held,
            &event.logical_key,
        );
        if let Some(code) = shortcut {
            if pressed {
                self.input(InputEvent::Text { ch: code });
            }
            return;
        }
        if let PhysicalKey::Code(code) = event.physical_key {
            if let Some(scan) = keys::dik(code) {
                self.input(InputEvent::Key { scan, pressed });
            }
        }
        if pressed {
            if let Some(text) = &event.text {
                for typed in ac2_ui::typed_text(text) {
                    self.input(typed);
                }
            }
        }
    }

    /// F7: the window again in the other look, at the page it showed.
    fn switch_style(&mut self, store: &Arc<ResourceStore>) {
        let style = self.wiki.style.toggled();
        let slug = self.wiki.slug();
        match Wiki::build(
            store,
            style,
            self.config.width,
            self.config.height,
            slug.as_deref(),
        ) {
            Ok(wiki) => {
                self.wiki = wiki;
                self.window.set_title(&title(style));
            }
            Err(e) => eprintln!("vault-wiki: {e}"),
        }
    }

    /// One frame; true when the window's close button closed the guide.
    fn frame(&mut self) -> bool {
        let (stamp, now) = self.now();
        let modifiers = self.controls.modifiers();
        let actions = self.controls.take_ui_input();
        let (text_actions, characters, _) = self.controls.take_text_input();
        let (messages, outcomes) =
            self.wiki
                .frame(&actions, &text_actions, &characters, modifiers, now);
        self.controls
            .set_text_mode(self.wiki.text_mode(), stamp, now);
        self.controls.set_copy_text(self.wiki.text_copy_focus());
        self.controls.ui_messages(&messages, now);
        self.controls.frame();
        for outcome in outcomes {
            let clipboard = ac2_client::shared_clipboard();
            match outcome {
                Outcome::Share(text) => {
                    clipboard.set_text(&text);
                    println!("share {text}");
                }
                Outcome::PointAt(at) => {
                    let text = coord_text(at);
                    clipboard.set_text(&text);
                    println!("point-at {text}");
                }
            }
        }
        if self.wiki.closed() {
            return true;
        }
        if let Some(surfaces) = self.wiki.compose() {
            self.overlay.set_layers(
                &self.gpu,
                surfaces.iter().map(|s| UiLayer {
                    x: s.x,
                    y: s.y,
                    width: s.width,
                    height: s.height,
                    rotation: s.rotation,
                    shown: s.shown,
                    opacity: s.opacity,
                    rgba: &s.rgba,
                }),
            );
        }
        let Some(output) = ac2_gpu::acquire_frame(&self.surface, &self.gpu.device, &self.config)
        else {
            self.gpu.flush_writes();
            return false;
        };
        let target = output.texture.create_view(&Default::default());
        self.overlay
            .prepare(&self.gpu.queue, self.config.width, self.config.height);
        render_to(
            &self.gpu.device,
            &self.gpu.queue,
            &target,
            &self.depth,
            [0.02, 0.03, 0.04],
            |pass| self.overlay.draw(pass),
        );
        self.gpu.queue.present(output);
        false
    }
}

fn depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("vault-wiki depth"),
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
