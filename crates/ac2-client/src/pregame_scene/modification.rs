//! Avatar modification's camera: `gmAvatarModificationUI::InitCamera` frames the avatar and
//! `UpdateCamera` turns the camera about it by the rotation slider and makes `Light_1` a light
//! shining from the camera.

use ac2_math::Vec3;
use ac2_render::Camera;
use ac2_ui::pregame::ModificationCamera;

use ac2_ui::pregame::PregameUi;

use crate::ui_scene::{CameraData, ViewSetup};

/// The frame of avatar modification's viewport: its camera framing the avatar and `Light_1`
/// from the camera.
pub(crate) fn direct(ui: &PregameUi, setup: &mut ViewSetup<'_>) {
    let Some(modification) = ui
        .avatar_modification()
        .filter(|m| m.viewport() == Some(setup.viewport.root))
    else {
        return;
    };
    let Some(race) = ui.data().current().map(|c| c.race) else {
        return;
    };
    let bounds = setup.object(modification.phys_obj()).and_then(|o| o.bounds);
    let avatar = setup.figures.last().and_then(|p| p.drawn.as_ref());
    if let (Some(bounds), Some(avatar)) = (bounds, avatar) {
        let scale_z = avatar.rig_scale.z;
        let (camera, light) =
            modification_camera(setup.base, race, bounds, scale_z, modification.camera());
        setup.camera = camera;
        setup.eye_offset = 0.0;
        setup.camera_light = Some(light);
    }
}

/// `InitCamera`'s framing for a race: the height factor, the camera's height and target
/// fractions and its distance fraction, (un)zoomed. Legions adds Drudges (8) and Empyreans
/// (0x10) with the human factors; other races leave them unset (the human ones here).
fn framing(race: u32, zoomed: bool) -> (f64, f64, f64, f64) {
    let human = if zoomed {
        (0.35, 0.1, 0.75)
    } else {
        (0.85, 0.6, 0.75)
    };
    let ((height, target, distance), scale) = match race {
        1 => (human, 0.95),
        2 if zoomed => ((0.25, 0.1, 0.65), 1.0),
        2 => ((0.75, 0.55, 0.4), 1.0),
        4 if zoomed => ((0.25, 0.05, 1.0), 0.84),
        4 => ((0.85, 0.6, 0.8), 0.84),
        _ => (human, 1.0),
    };
    (scale, height, target, distance)
}

/// `InitCamera` then `UpdateCamera`: the eye at the race's height and distance fractions of the
/// avatar's height (its box, scaled by the avatar's height when zoomed, the lower three
/// quarters then skipped), looking at the target height on the avatar's axis, turned about it
/// so the camera's heading is the rotation slider's position times 360 degrees.
pub(crate) fn modification_camera(
    base: &CameraData,
    race: u32,
    bounds: (Vec3, Vec3),
    scale_z: f32,
    state: ModificationCamera,
) -> (Camera, Vec3) {
    let (factor, height_f, target_f, distance_f) = framing(race, state.zoomed);
    let mut height = (f64::from(bounds.1.z) - f64::from(bounds.0.z)) * factor;
    let mut floor = 0.0;
    if state.zoomed {
        height *= f64::from(scale_z);
        floor = height * 0.75;
    }
    let span = height - floor;
    let eye_z = span * height_f + floor;
    let target_z = span * target_f + floor;
    let distance = span * distance_f;
    // `Heading::set_degrees(pos * 360)`; the camera looks along `(sin a, cos a)`.
    let angle = (f64::from(state.rotation) * 360.0).to_radians();
    let eye = Vec3::new(
        (-distance * angle.sin()) as f32,
        (-distance * angle.cos()) as f32,
        eye_z as f32,
    );
    let target = Vec3::new(0.0, 0.0, target_z as f32);
    let camera = Camera {
        eye,
        target,
        fov_y: base.fov.to_radians(),
        near: base.near,
        far: base.far,
    };
    let forward = (target - eye).normalize_or_zero();
    // The light along the view, tilted down by half a metre per metre.
    let light = Vec3::new(forward.x, forward.y, forward.z - 0.5).normalize_or_zero();
    (camera, light)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_math::Frame;

    fn base() -> CameraData {
        CameraData {
            name: Some("Camera_1".into()),
            frame: Frame::IDENTITY,
            orthographic: false,
            fov: 90.0,
            vertical_scale: 1.0,
            near: 0.1,
            far: 2500.0,
        }
    }

    /// `UpdateCamera`: the slider's position times 360 degrees is the camera's heading, at the
    /// distance and heights `InitCamera` gives a human (height factor 0.95).
    #[test]
    fn the_rotation_slider_turns_the_camera_about_the_avatar() {
        let bounds = (Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0));
        for rotation in [0.0_f32, 0.25, 0.5, 0.8] {
            let state = ModificationCamera {
                zoomed: false,
                rotation,
            };
            let (camera, light) = modification_camera(&base(), 1, bounds, 1.0, state);
            let f = camera.target - camera.eye;
            let heading = (450.0 - f64::from(f.y).atan2(f64::from(f.x)).to_degrees()) % 360.0;
            let want = f64::from(rotation) * 360.0 % 360.0;
            let diff = (heading - want).rem_euclid(360.0);
            assert!(
                !(1e-3..=360.0 - 1e-3).contains(&diff),
                "{rotation}: {heading}"
            );
            let height = 2.0 * 0.95;
            assert!((camera.eye.z - (height * 0.85) as f32).abs() < 1e-5);
            assert!((camera.target.z - (height * 0.6) as f32).abs() < 1e-5);
            let flat = camera.eye.truncate().length();
            assert!((flat - (height * 0.75) as f32).abs() < 1e-5);
            assert!(light.z < 0.0, "the camera light shines down");
        }
    }

    /// Zoomed, the lower three quarters of the (scaled) height are skipped.
    #[test]
    fn zooming_frames_the_head() {
        let bounds = (Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0));
        let state = ModificationCamera {
            zoomed: true,
            rotation: 0.5,
        };
        let (camera, _) = modification_camera(&base(), 2, bounds, 1.5, state);
        let (height, floor) = (2.0 * 1.5, 2.0 * 1.5 * 0.75);
        let span = height - floor;
        assert!((camera.eye.z - (span * 0.25 + floor) as f32).abs() < 1e-5);
        assert!((camera.target.z - (span * 0.1 + floor) as f32).abs() < 1e-5);
        assert!((camera.eye.y - (span * 0.65) as f32).abs() < 1e-5);
    }
}
