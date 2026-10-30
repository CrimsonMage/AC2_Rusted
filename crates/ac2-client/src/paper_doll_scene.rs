//! The paper doll's 3D scene (`window` feature) over [`UiScenes`]: the player's avatar in
//! `PaperDollViewport`'s scene and in the consignment's `Portrait` (`InitPortrait`), remade when
//! the appearance changes and framed by its race ([`ac2_ui::paper_doll_scene`] builds the figures
//! and places them; the portrait head and shoulders). A pick answers the mesh type of the avatar's
//! part under the pointer (`FinishPaperDollClick`).

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_gpu::Gpu;
use ac2_ui::paper_doll_scene::{avatar_key, figures, portrait_position, position};
use ac2_ui::SceneViewport;
use ac2_world::Ac2WorldSource;

use crate::frame::PlayerFrame;
use crate::ui_scene::{SceneData, UiScenes, ViewSetup};

/// The paper doll's scene.
pub(crate) struct PaperDollScene {
    scenes: UiScenes,
    /// The avatar last drawn: kept while the doll ignores appearance changes.
    avatar: Option<(u32, ac2_assets::Ac2VisualDesc)>,
}

impl PaperDollScene {
    pub(crate) fn new(store: Arc<ResourceStore>, world: Arc<Ac2WorldSource>) -> Self {
        Self {
            scenes: UiScenes::new(store, world),
            avatar: None,
        }
    }

    /// `DynamicObjectShadows` for the scenes' shadows.
    pub(crate) fn set_object_shadows(&mut self, level: ac2_render::prefs::ObjectShadows) {
        self.scenes.set_object_shadows(level);
    }

    pub(crate) fn scenes(&self) -> &UiScenes {
        &self.scenes
    }

    /// Follows the doll's viewport and the consignment's `portrait` (when shown) at `now`
    /// seconds with the player's appearance and race. While `hold` (`m_ignoreVDescChanges`, the player on a mount) the avatar keeps
    /// the appearance it had (`RecvNotice_LocalVisualDescUpdate` skips `UpdatePaperDoll`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update(
        &mut self,
        gpu: &Gpu,
        (viewports, portrait): (Vec<SceneViewport>, Option<usize>),
        player: Option<&PlayerFrame>,
        race: Option<i32>,
        now: f64,
        wait: bool,
        hold: bool,
    ) {
        if !hold || self.avatar.is_none() {
            self.avatar = player.and_then(|p| {
                let visual = p.visual?;
                Some((visual, ac2_physics::local_visual_desc(&p.appearance)))
            });
        }
        let avatar = &self.avatar;
        let mut figures = |viewport: &SceneViewport, scene: &SceneData| {
            figures(
                viewport.scene,
                scene,
                avatar.as_ref().map(|(visual, local)| (*visual, local)),
            )
        };
        // The consignment's portrait is framed head and shoulders (`UpdatePortraitPosition`).
        let mut direct = |setup: &mut ViewSetup<'_>| {
            if Some(setup.viewport.root) == portrait {
                portrait_position(setup, race)
            } else {
                position(setup, race)
            }
        };
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
