//! The GPU side of `ac2_render::ui_scene::UiViews`: one [`SceneRenderer`] per shown viewport
//! root, uploading each view's finished loads and drawing the frame the view laid out.

use std::collections::HashMap;

use ac2_render::scene::SceneUpdate;
use ac2_render::ui_scene::ViewFrame;

use crate::{Gpu, SceneRenderer};

/// The viewport scenes' renderers, keyed by viewport root.
#[derive(Default)]
pub struct UiSceneRenderers {
    renderers: HashMap<usize, SceneRenderer>,
}

impl UiSceneRenderers {
    /// No renderers yet; each is made on its view's first load or frame.
    pub fn new() -> Self {
        Self::default()
    }

    fn renderer(&mut self, gpu: &Gpu, root: usize) -> &mut SceneRenderer {
        self.renderers
            .entry(root)
            .or_insert_with(|| SceneRenderer::new(gpu))
    }

    /// Uploads the loads that finished for the view of root `root`.
    pub fn apply(&mut self, gpu: &Gpu, root: usize, update: SceneUpdate) {
        self.renderer(gpu, root).apply(gpu, update);
    }

    /// Writes a view's frame (`time` is client time for animated materials).
    pub fn prepare(&mut self, gpu: &Gpu, view: ViewFrame, time: f64) {
        let ViewFrame {
            root,
            camera,
            aspect,
            frame,
            objects,
        } = view;
        self.renderer(gpu, root)
            .prepare(gpu, &camera, aspect, time, frame, objects);
    }

    /// Records the scene of the view of root `root`.
    pub fn draw(&self, root: usize, pass: &mut wgpu::RenderPass<'_>) {
        if let Some(renderer) = self.renderers.get(&root) {
            renderer.draw(pass);
        }
    }

    /// Drops the renderers of the views `keep` rejects (no longer shown).
    pub fn retain(&mut self, mut keep: impl FnMut(usize) -> bool) {
        self.renderers.retain(|root, _| keep(*root));
    }
}
