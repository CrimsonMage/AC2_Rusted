//! egui over the viewer's frame: `egui-winit` turns window events into egui input and
//! `egui-wgpu` paints the result in a pass of its own over the scene (colour kept, no depth).

use ac2_gpu::Gpu;
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct Gui {
    ctx: egui::Context,
    /// The window's input state; `None` for a headless snapshot.
    state: Option<egui_winit::State>,
    renderer: Renderer,
}

impl Gui {
    pub fn new(window: &Window, gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        let ctx = egui::Context::default();
        let state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(gpu.device.limits().max_texture_dimension_2d as usize),
        );
        let renderer = Renderer::new(&gpu.device, format, RendererOptions::default());
        Self {
            ctx,
            state: Some(state),
            renderer,
        }
    }

    /// egui with no window, for painting one frame into an offscreen target.
    pub fn headless(gpu: &Gpu, format: wgpu::TextureFormat) -> Self {
        Self {
            ctx: egui::Context::default(),
            state: None,
            renderer: Renderer::new(&gpu.device, format, RendererOptions::default()),
        }
    }

    /// Feeds a window event to egui; whether egui took it.
    pub fn event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state
            .as_mut()
            .is_some_and(|state| state.on_window_event(window, event).consumed)
    }

    /// Whether egui is using the keyboard (a text field has focus).
    pub fn wants_keyboard(&self) -> bool {
        self.ctx.egui_wants_keyboard_input()
    }

    /// Whether the pointer is over, or being used by, egui.
    pub fn wants_pointer(&self) -> bool {
        self.ctx.egui_wants_pointer_input() || self.ctx.is_pointer_over_egui()
    }

    /// Runs one frame of `ui` and paints it over `view` (`size` in physical pixels).
    pub fn frame(
        &mut self,
        window: &Window,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        size: [u32; 2],
        ui: impl FnMut(&mut egui::Ui),
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let input = state.take_egui_input(window);
        let platform = self.paint(gpu, view, size, input, ui);
        if let Some(state) = self.state.as_mut() {
            state.handle_platform_output(window, platform);
        }
    }

    /// One frame of `ui` over an offscreen `view` of `size` pixels, at scale 1.
    pub fn headless_frame(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        size: [u32; 2],
        mut ui: impl FnMut(&mut egui::Ui),
    ) {
        // egui sizes a window and loads its fonts on the first passes and shows it from the
        // fourth, once its fade-in has run.
        for pass in 0..4 {
            let input = egui::RawInput {
                time: Some(f64::from(pass) * 0.5),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(size[0] as f32, size[1] as f32),
                )),
                ..Default::default()
            };
            self.paint(gpu, view, size, input, &mut ui);
        }
    }

    fn paint(
        &mut self,
        gpu: &Gpu,
        view: &wgpu::TextureView,
        size: [u32; 2],
        input: egui::RawInput,
        ui: impl FnMut(&mut egui::Ui),
    ) -> egui::PlatformOutput {
        let mut output = self.ctx.run_ui(input, ui);
        let jobs = self.ctx.tessellate(output.shapes, output.pixels_per_point);
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.renderer
                    .update_texture(&gpu.device, &gpu.queue, *id, delta);
            }
        }
        let screen = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: output.pixels_per_point,
        };
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        let extra =
            self.renderer
                .update_buffers(&gpu.device, &gpu.queue, &mut encoder, &jobs, &screen);
        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.renderer
                .render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        gpu.queue
            .submit(extra.into_iter().chain([encoder.finish()]));
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        output.textures_delta.clear();
        output.platform_output
    }
}
