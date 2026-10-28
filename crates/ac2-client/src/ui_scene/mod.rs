//! The UI's 3D scenes (`window` feature): each shown viewport's `UIScene`
//! (`UIViewportObject::Render` / `UIScene::Render`) with the figures its owner stands in it,
//! seen through its camera and drawn into the viewport's rectangle among the UI's surfaces. The
//! state (which scene each view shows, its figures, bodies and camera) is `ac2_render`'s
//! [`UiViews`]; `ac2_gpu`'s [`UiSceneRenderers`] draw it. The pregame screens
//! ([`crate::pregame_scene`]) own one each.
//!
//! This module binds them: the scenes, objects and models load on the scene loader thread
//! ([`data`], which takes the DATs' reads and each object's body), and the frame thread takes what
//! finished and lays out frames.
//!
//! Picks are `UIScene::StartSelection` / `FinishSelection` (`UiViews::pick`).

pub(crate) mod data;
mod render;

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_gpu::{Gpu, UiSceneRenderers};
use ac2_physics::PhysicsBody;
use ac2_render::ui_scene::SceneLoads;
use ac2_ui::SceneViewport;
use ac2_world::Ac2WorldSource;

pub use ac2_render::ui_scene::data::{CameraData, EntityData, LightData, ObjectData, SceneData};
pub use ac2_render::ui_scene::figure::{figure_key, scene_figures, At, Drawn, Figure, Placed};
pub use ac2_render::ui_scene::{ScenePick, UiViews, ViewSetup};
use data::SceneLoader;
pub use render::render_with_scenes;

/// The UI's viewport scenes.
pub struct UiScenes {
    loader: SceneLoader,
    views: UiViews,
    renderers: UiSceneRenderers,
}

impl UiScenes {
    /// Scenes over `store`'s DATs; each viewport's models load through a `WorldScene` over
    /// `world`.
    pub fn new(store: Arc<ResourceStore>, world: Arc<Ac2WorldSource>) -> Self {
        Self {
            loader: SceneLoader::new(store),
            views: UiViews::new(world),
            renderers: UiSceneRenderers::new(),
        }
    }

    /// Follows `viewports` at `now` seconds: their scenes, the figures `figures` stands in each,
    /// the loads that finished (all of them when `wait`), the animation and the frame's layout,
    /// then `direct` adjusts each frame.
    pub fn update(
        &mut self,
        gpu: &Gpu,
        viewports: Vec<SceneViewport>,
        now: f64,
        wait: bool,
        figures: &mut dyn FnMut(&SceneViewport, &SceneData) -> Vec<Figure>,
        direct: &mut dyn FnMut(&mut ViewSetup<'_>),
    ) {
        let updates = self
            .views
            .update(&mut self.loader, viewports, now, wait, figures, direct);
        for (root, update) in updates {
            self.renderers.apply(gpu, root, update);
        }
        let roots = self.views.roots();
        self.renderers.retain(|root| roots.contains(&root));
    }

    /// `UIScene::StartSelection` / `FinishSelection` at screen `point` in the viewport of root
    /// `root` (any viewport under the point when `None`): the nearest figure under it.
    pub fn pick(&self, point: (i32, i32), root: Option<usize>) -> Option<ScenePick> {
        self.views.pick(point, root)
    }

    /// A figure's body, once made.
    pub fn body_mut(&mut self, key: u64) -> Option<&mut PhysicsBody> {
        self.views.body_mut(key)
    }

    /// Whether `object`'s data is in (loaded or missing).
    pub fn object_known(&self, object: u32) -> bool {
        self.loader.object_known(object)
    }

    /// The scene `did`, once loaded.
    pub(crate) fn scene(&self, did: u32) -> Option<&SceneData> {
        self.loader.scene(did)
    }

    /// The figures of the viewport of root `root` as last laid out.
    pub fn placed(&self, root: usize) -> Option<&[Placed]> {
        self.views.placed(root)
    }

    /// Whether loads are under way (headless snapshots wait them out).
    pub fn busy(&self) -> bool {
        self.loader.busy()
    }

    /// Writes each view's frame (`time` is client time for animated materials).
    pub fn prepare(&mut self, gpu: &Gpu, time: f64) {
        for view in self.views.take_frames() {
            self.renderers.prepare(gpu, view, time);
        }
    }

    /// The prepared viewports.
    pub fn drawn(&self) -> Vec<SceneViewport> {
        self.views.drawn()
    }

    /// Records the scene of the viewport of root `root` (its viewport is the caller's).
    pub fn draw(&self, root: usize, pass: &mut wgpu::RenderPass<'_>) {
        self.renderers.draw(root, pass);
    }

    /// How many figures of the views have their models loaded.
    pub fn drawn_objects(&self) -> usize {
        self.views.drawn_objects()
    }
}
