//! The GPU side of `ac2_render::ui_scene::UiViews`: one [`SceneRenderer`] per shown viewport
//! root, uploading each view's finished loads and drawing the frame the view laid out.

use std::collections::HashMap;

use ac2_render::prefs::ObjectShadows;
use ac2_render::scene::SceneUpdate;
use ac2_render::ui_scene::ViewFrame;

use crate::{Gpu, SceneRenderer};

/// The viewport scenes' renderers, keyed by viewport root.
pub struct UiSceneRenderers {
    renderers: HashMap<usize, SceneRenderer>,
    /// `DynamicObjectShadows`, which sizes the scenes' sun shadows.
    object_shadows: ObjectShadows,
}

impl Default for UiSceneRenderers {
    fn default() -> Self {
        Self {
            renderers: HashMap::new(),
            object_shadows: ac2_render::prefs::SceneOptions::default().object_shadows,
        }
    }
}

impl UiSceneRenderers {
    /// No renderers yet; each is made on its view's first load or frame.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sizes the scenes' sun shadows for the `DynamicObjectShadows` level: `UIScene::Render`
    /// drew its objects' shadow volumes from level 2 up.
    pub fn set_object_shadows(&mut self, level: ObjectShadows) {
        self.object_shadows = level;
        for renderer in self.renderers.values_mut() {
            renderer.set_object_shadows(level);
        }
    }

    fn renderer(&mut self, gpu: &Gpu, root: usize) -> &mut SceneRenderer {
        let level = self.object_shadows;
        self.renderers.entry(root).or_insert_with(|| {
            let mut renderer = SceneRenderer::new(gpu);
            renderer.set_viewport_scene();
            renderer.set_object_shadows(level);
            renderer
        })
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

#[cfg(test)]
#[path = "ui_scene_shadow_tests.rs"]
mod shadow_tests;
