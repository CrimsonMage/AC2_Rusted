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

use ac2_assets::Ac2VisualDesc;
use ac2_math::{Frame, Vec3};
use ac2_render::ui_scene::camera;
use ac2_render::ui_scene::data::SceneData;
use ac2_render::ui_scene::figure::{figure_key, scene_figures, At, Figure};
use ac2_render::ui_scene::ViewSetup;

/// The avatar's figure key in the doll's scene.
pub fn avatar_key(scene: u32) -> u64 {
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

/// The doll's figures: the scene's own, then the player's avatar (its visual and local
/// appearance) at the scene's origin.
pub fn figures(scene: u32, data: &SceneData, avatar: Option<(u32, &Ac2VisualDesc)>) -> Vec<Figure> {
    let mut out = scene_figures(scene, data);
    if let Some((visual, local)) = avatar {
        out.push(Figure {
            key: avatar_key(scene),
            object: visual,
            local: Some(local.clone()),
            at: At::Scene(Frame::IDENTITY, Vec3::ONE),
            pick: Some(0),
            mode: None,
        });
    }
    out
}

/// `gmPaperDollUI::UpdatePaperDollPosition`, the admin 7148 framing: the avatar half its
/// height below the origin and the orthographic camera that height tall, so the whole figure
/// shows. Legions 7160 sets the avatar at -0.875 of its height and the camera to a quarter of
/// it, a head-and-shoulders portrait; a deliberate deviation (AGENTS.md) keeps the full figure.
/// The race factors are Legions'.
pub fn position(setup: &mut ViewSetup<'_>, race: Option<i32>) {
    frame_avatar(setup, race, -0.5, 1.0);
}

/// `gmShopConsignmentUI::UpdatePortraitPosition` (Legions 0x41af10): the consignment's portrait
/// of the player, head and shoulders: the avatar at -0.875 of its height (the race's factor
/// applied) and the camera a quarter of that height tall.
pub fn portrait_position(setup: &mut ViewSetup<'_>, race: Option<i32>) {
    frame_avatar(setup, race, -0.875, 0.25);
}

/// The avatar `offset` of its (race-scaled) height below the scene's origin, and the camera's
/// vertical scale `scale` of that height.
fn frame_avatar(setup: &mut ViewSetup<'_>, race: Option<i32>, offset: f64, scale: f64) {
    let key = avatar_key(setup.viewport.scene);
    let Some(avatar) = setup.figures.iter_mut().find(|p| p.key == key) else {
        return;
    };
    let Some(height) = avatar.height() else {
        return;
    };
    let (z, vertical_scale) = avatar_framing(height, race, offset, scale);
    let mut frame = Frame::IDENTITY;
    frame.origin.z = z;
    avatar.move_to(frame);
    let mut base = setup.base.clone();
    base.vertical_scale = vertical_scale;
    let (view, offset) = camera::scene_camera(&base);
    setup.camera = view;
    setup.eye_offset = offset;
}

/// The avatar's place below the origin and the camera's vertical scale for a box `height` tall:
/// `float` stores of the race-scaled height, then its `offset` and `scale` products.
pub(crate) fn avatar_framing(
    height: f32,
    race: Option<i32>,
    offset: f64,
    scale: f64,
) -> (f32, f32) {
    let height = (f64::from(height) * race_factor(race)) as f32;
    (
        (f64::from(height) * offset) as f32,
        (f64::from(height) * scale) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::avatar_framing;

    #[test]
    fn the_doll_shows_the_whole_figure_and_the_portrait_head_and_shoulders() {
        // The doll (admin's framing): half the height below, the camera the height tall.
        assert_eq!(avatar_framing(2.0, Some(3), -0.5, 1.0), (-1.0, 2.0));
        // The consignment's portrait (Legions 0x41af10): -0.875 and a quarter, with the race's
        // factor (0.9 for races 1, 2, 8 and 16; 0.75 for race 4).
        assert_eq!(avatar_framing(2.0, Some(3), -0.875, 0.25), (-1.75, 0.5));
        let scaled = (2.0f64 * 0.9) as f32;
        assert_eq!(
            avatar_framing(2.0, Some(8), -0.875, 0.25),
            (
                (f64::from(scaled) * -0.875) as f32,
                (f64::from(scaled) * 0.25) as f32
            )
        );
        assert_eq!(avatar_framing(2.0, Some(4), -0.875, 0.25), (-1.3125, 0.375));
    }
}
