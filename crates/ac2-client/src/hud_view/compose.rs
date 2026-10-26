//! The compose path: the tree composed (`UIRegion::DrawHere`, software) when it changed, the
//! radar field alone each frame otherwise, and the layers the overlay draws.

use ac2_gpu::{Gpu, UiLayer, UiOverlay};

use super::HudView;

impl HudView {
    /// The frame's HUD composed again, or its surfaces moved, after `gm3DTextUI::Update`.
    pub(crate) fn finish_frame(&mut self, gpu: &Gpu) {
        if let Some((tree, _)) = &mut self.hud {
            if tree.take_changed() {
                self.dirty = true;
            }
        }
        self.compose(gpu);
    }

    pub(super) fn compose(&mut self, gpu: &Gpu) {
        let Some((tree, images)) = &mut self.hud else {
            return;
        };
        if !self.dirty {
            if tree.take_moved() {
                let placements = tree.placements();
                self.overlay.set_placements(
                    placements
                        .into_iter()
                        .map(|p| (p.x, p.y, p.rotation, p.shown, p.opacity)),
                );
            }
            // `gmRadarUI::ListenToMessage`'s `ForceDraw`: the radar field alone, each frame.
            let field = self.radar.as_ref().and_then(|r| r.field());
            let layer = field.and_then(|f| self.layer_roots.iter().position(|&r| r == f));
            if let (Some(field), Some(layer), Some(radar)) = (field, layer, &self.radar) {
                let mut surface = tree.compose_root(field, images);
                radar.draw(&mut surface);
                self.overlay.replace_pixels(gpu, layer, &surface.rgba);
            }
            return;
        }
        tree.take_moved();
        self.dirty = false;
        self.images.request_missing(tree, images);
        tree.take_changed();
        let mut surfaces = tree.compose(images);
        if let Some(radar) = &self.radar {
            if let Some(surface) = surfaces.iter_mut().find(|s| Some(s.root) == radar.field()) {
                radar.draw(surface);
            }
        }
        self.layer_roots = surfaces.iter().map(|surface| surface.root).collect();
        self.overlay.set_layers(
            gpu,
            surfaces.iter().map(|surface| UiLayer {
                x: surface.x,
                y: surface.y,
                width: surface.width,
                height: surface.height,
                rotation: surface.rotation,
                shown: surface.shown,
                opacity: surface.opacity,
                rgba: &surface.rgba,
            }),
        );
    }

    pub(crate) fn prepare(&self, queue: &wgpu::Queue) {
        let scale = &self.scale;
        self.overlay.prepare_scaled(
            queue,
            self.size,
            scale.scale as f32,
            (scale.origin.0 as f32, scale.origin.1 as f32),
        );
    }

    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.overlay.draw(pass);
    }

    /// The drawn layers and their roots, in drawing order.
    pub(crate) fn layers_for_scenes(&self) -> (&UiOverlay, &[usize]) {
        (&self.overlay, &self.layer_roots)
    }
}
