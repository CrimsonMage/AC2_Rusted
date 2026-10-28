//! The paper doll's 3D scene (`window` feature) over [`UiScenes`]: the player's avatar in
//! `PaperDollViewport`'s scene, remade when the appearance changes and framed by its race
//! ([`ac2_ui::paper_doll_scene`] builds the figures and places them). A pick answers the mesh
//! type of the avatar's part under the pointer (`FinishPaperDollClick`).

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_gpu::Gpu;
use ac2_ui::paper_doll_scene::{avatar_key, figures, position};
use ac2_ui::SceneViewport;
use ac2_world::Ac2WorldSource;

use crate::frame::PlayerFrame;
use crate::ui_scene::{SceneData, UiScenes, ViewSetup};

/// The paper doll's scene.
pub(crate) struct PaperDollScene {
    scenes: UiScenes,
}

impl PaperDollScene {
    pub(crate) fn new(store: Arc<ResourceStore>, world: Arc<Ac2WorldSource>) -> Self {
        Self {
            scenes: UiScenes::new(store, world),
        }
    }

    pub(crate) fn scenes(&self) -> &UiScenes {
        &self.scenes
    }

    /// Follows the doll's viewport (when shown) at `now` seconds with the player's appearance
    /// and race.
    pub(crate) fn update(
        &mut self,
        gpu: &Gpu,
        viewports: Vec<SceneViewport>,
        player: Option<&PlayerFrame>,
        race: Option<i32>,
        now: f64,
        wait: bool,
    ) {
        let avatar = player.and_then(|p| {
            let visual = p.visual?;
            Some((visual, ac2_physics::local_visual_desc(&p.appearance)))
        });
        let mut figures = |viewport: &SceneViewport, scene: &SceneData| {
            figures(
                viewport.scene,
                scene,
                avatar.as_ref().map(|(visual, local)| (*visual, local)),
            )
        };
        let mut direct = |setup: &mut ViewSetup<'_>| position(setup, race);
        self.scenes
            .update(gpu, viewports, now, wait, &mut figures, &mut direct);
    }

    /// `UIScene::StartSelection` / `FinishSelection` at `point` in the doll's viewport: the mesh
    /// type of the avatar's part hit, if the avatar was hit.
    pub(crate) fn pick(&self, point: (i32, i32), viewport: &SceneViewport) -> Option<u32> {
        let hit = self.scenes.pick(point, Some(viewport.root))?;
        (hit.key == avatar_key(viewport.scene)).then_some(hit.mesh_type)
    }

    pub(crate) fn busy(&self) -> bool {
        self.scenes.busy()
    }

    pub(crate) fn prepare(&mut self, gpu: &Gpu, time: f64) {
        self.scenes.prepare(gpu, time);
    }
}
