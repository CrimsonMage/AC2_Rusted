//! The paper doll's 3D scene (`window` feature): `gmPaperDollUI::InitPaperDoll` makes the
//! player's avatar from its total appearance (`VisualDesc::GetTotalVisualDesc`,
//! `UIScenePhysObjEntity::InitFromVDesc`) in `PaperDollViewport`'s scene; `UpdatePaperDoll`
//! remakes it when the appearance changes (`RecvNotice_LocalVisualDescUpdate`), and
//! `UpdatePaperDollPosition` stands it below the scene's origin and sets the camera's vertical
//! scale from the avatar's height (its box's height, times 0.9 for races 1, 2, 8 and 16 and 0.75
//! for race 4). Legions 7160's constants: the avatar sits at -0.875 of that height and the scale
//! is a quarter of it (admin 7148 has -0.5 and the full height). The camera is orthographic in
//! the scene's data. A pick answers the mesh type of the avatar's part under the pointer
//! (`FinishPaperDollClick`).

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_gpu::Gpu;
use ac2_math::{Frame, Vec3};
use ac2_ui::SceneViewport;
use ac2_world::Ac2WorldSource;

use crate::frame::PlayerFrame;
use crate::ui_scene::{
    camera, figure_key, scene_figures, At, Figure, SceneData, UiScenes, ViewSetup,
};

/// The avatar's figure key in the doll's scene.
fn avatar_key(scene: u32) -> u64 {
    figure_key(scene, "paper doll", 0)
}

/// `UpdatePaperDollPosition`'s height factor for the player's race (int 290), Legions 7160
/// (admin 7148 has no factor for 8 and 16).
fn race_factor(race: Option<i32>) -> f64 {
    match race {
        Some(1 | 2 | 8 | 16) => 0.9,
        Some(4) => 0.75,
        _ => 1.0,
    }
}

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
            Some((visual, crate::appearance::local_visual_desc(&p.appearance)))
        });
        let mut figures = |viewport: &SceneViewport, scene: &SceneData| {
            let mut out = scene_figures(viewport.scene, scene);
            if let Some((visual, local)) = &avatar {
                out.push(Figure {
                    key: avatar_key(viewport.scene),
                    object: *visual,
                    local: Some(local.clone()),
                    at: At::Scene(Frame::IDENTITY, Vec3::ONE),
                    pick: Some(0),
                    mode: None,
                });
            }
            out
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

/// `gmPaperDollUI::UpdatePaperDollPosition`, the admin 7148 framing: the avatar half its
/// height below the origin and the orthographic camera that height tall, so the whole figure
/// shows. Legions 7160 sets the avatar at -0.875 of its height and the camera to a quarter of
/// it, a head-and-shoulders portrait; a deliberate deviation (AGENTS.md) keeps the full figure.
/// The race factors are Legions'.
fn position(setup: &mut ViewSetup<'_>, race: Option<i32>) {
    let key = avatar_key(setup.viewport.scene);
    let Some(avatar) = setup.figures.iter_mut().find(|p| p.key == key) else {
        return;
    };
    let Some(height) = avatar.height() else {
        return;
    };
    // `float` stores: the scaled height, then its offset below the origin.
    let height = (f64::from(height) * race_factor(race)) as f32;
    let mut frame = Frame::IDENTITY;
    frame.origin.z = (f64::from(height) * -0.5) as f32;
    avatar.move_to(frame);
    let mut base = setup.base.clone();
    base.vertical_scale = height;
    let (view, offset) = camera::scene_camera(&base);
    setup.camera = view;
    setup.eye_offset = offset;
}
