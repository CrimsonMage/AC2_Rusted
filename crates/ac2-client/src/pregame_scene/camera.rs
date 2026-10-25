//! The pregame scenes' cameras and lights.
//!
//! A viewport renders through its camera entity (`UIScene::Render`, `SceneTool::SetupCamera`):
//! the view from the entity's frame looking along its +Y axis (`SceneTool::SetupView`'s forward
//! row), the vertical field of view in degrees and the viewport's aspect
//! (`SetupPerspectiveProjection`). Avatar modification moves its camera to frame the avatar
//! (`gmAvatarModificationUI::InitCamera`) and turns it about the avatar by the rotation slider
//! (`UpdateCamera`), and makes `Light_1` a light shining from the camera.
//!
//! The scene's light entities are linked as the lights (`SceneTool::LinkLight`) and the sun is
//! disabled; a scene without lights gets ambient 0.1 and the sun along (0.3, -0.9, -1) at
//! half white (`SetAmbientLight`, `SetSunlight`).

use ac2_math::Vec3;
use ac2_render::object::{object_point_lights, PointLight, SceneLight};
use ac2_render::{Camera, SceneLighting};
use ac2_ui::pregame::ModificationCamera;
use ac2_world::{Ac2LightKind, Ac2StaticLight};

use super::data::{CameraData, LightData, SceneData, LIGHT_DIRECTIONAL};

/// `UIScene::Render`'s ambient light when the scene has no lights, which stays set for the
/// scenes that do (`DisableSunlight` leaves the ambient).
const AMBIENT: f32 = 0.1;
/// `UIScene::Render`'s sun: direction, white at this intensity.
const SUN_DIRECTION: [f64; 3] = [0.3, -0.9, -1.0];
const SUN_INTENSITY: f32 = 0.5;
/// No fog (`SetFogProperties` off): past any scene.
const NO_FOG: (f32, f32) = (1.0e6, 2.0e6);
/// `gmAvatarModificationUI::UpdateCamera`'s light (`Light_1`): diffuse 0.6 grey, ambient 0.1.
const CAMERA_LIGHT: &str = "Light_1";
const CAMERA_LIGHT_DIFFUSE: f32 = 0.6;
const CAMERA_LIGHT_AMBIENT: f32 = 0.1;

/// The camera entity a viewport names (`UIElement_Viewport::SetCameraID`), else the scene's
/// first.
pub(crate) fn camera_entity<'a>(
    scene: &'a SceneData,
    name: Option<&str>,
) -> Option<&'a CameraData> {
    scene
        .cameras
        .iter()
        .find(|c| name.is_some() && c.name.as_deref() == name)
        .or_else(|| scene.cameras.first())
}

/// `SceneTool::SetupCamera` for a perspective camera (an orthographic one is not drawn).
pub(crate) fn scene_camera(camera: &CameraData) -> Option<Camera> {
    if camera.orthographic {
        return None;
    }
    let forward = camera.frame.orientation * Vec3::Y;
    Some(Camera {
        eye: camera.frame.origin,
        target: camera.frame.origin + forward,
        fov_y: camera.fov.to_radians(),
        near: camera.near,
        far: camera.far,
    })
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

/// The frame's lighting and the point lights each object gets: the scene's lights linked
/// (`LinkLight`) with the sun off, or the default sun and ambient. `camera_light` is avatar
/// modification's direction for `Light_1`.
pub(crate) fn scene_lighting(scene: &SceneData, camera_light: Option<Vec3>) -> SceneLighting {
    let fog = scene.fog;
    let mut lighting = SceneLighting {
        light_dir: Vec3::ZERO,
        dir_color: [0.0; 3],
        dir_brightness: 0.0,
        ambient: [AMBIENT; 3],
        fog_color: fog.map_or([0.0; 3], |f| f.0),
        fog_range: fog.map_or(NO_FOG, |f| f.1),
        fog_sky: 0.0,
    };
    if scene.lights.is_empty() {
        let [x, y, z] = SUN_DIRECTION;
        let length = (x * x + y * y + z * z).sqrt();
        let direction = Vec3::new(
            (x / length) as f32,
            (y / length) as f32,
            (z / length) as f32,
        );
        lighting.light_dir = direction * SUN_INTENSITY;
        lighting.dir_color = [1.0; 3];
        lighting.dir_brightness = SUN_INTENSITY;
        return lighting;
    }
    for light in &scene.lights {
        let camera = camera_light.filter(|_| light.name.as_deref() == Some(CAMERA_LIGHT));
        if let Some(direction) = camera {
            let ambient = CAMERA_LIGHT_AMBIENT;
            lighting.ambient = lighting.ambient.map(|a| a + ambient);
            set_directional(
                &mut lighting,
                direction,
                [CAMERA_LIGHT_DIFFUSE; 3],
                light.intensity,
            );
        } else if light.kind == LIGHT_DIRECTIONAL {
            set_directional(
                &mut lighting,
                light.direction,
                light.diffuse,
                light.intensity,
            );
        }
    }
    lighting
}

fn set_directional(lighting: &mut SceneLighting, direction: Vec3, color: [f32; 3], intensity: f32) {
    lighting.light_dir = direction.normalize_or_zero() * intensity;
    lighting.dir_color = color;
    lighting.dir_brightness = intensity;
}

/// The scene's point lights as static lights (`LightSource` at the entity's origin); with a
/// camera light, `Light_1` is directional instead.
pub(crate) fn point_lights(scene: &SceneData, camera_light: bool) -> Vec<SceneLight> {
    scene
        .lights
        .iter()
        .filter(|l| l.kind != LIGHT_DIRECTIONAL)
        .filter(|l| !(camera_light && l.name.as_deref() == Some(CAMERA_LIGHT)))
        .map(|l: &LightData| SceneLight {
            light: Ac2StaticLight {
                cell: ac2_core::CellId::new(0),
                kind: Ac2LightKind::Point,
                origin: l.origin,
                direction: l.direction,
                range: l.range,
                intensity: l.intensity,
                color: l.diffuse,
                lit: Box::default(),
            },
            origin: l.origin,
            indoor: false,
        })
        .collect()
}

/// `SceneTool::CacheStaticLights` for an object at `position` with radius `radius`.
pub(crate) fn object_lights(lights: &[SceneLight], position: Vec3, radius: f32) -> Vec<PointLight> {
    object_point_lights(lights, position, radius, false)
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
            assert!(diff < 1e-3 || diff > 360.0 - 1e-3, "{rotation}: {heading}");
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

    /// `UIScene::Render` without lights: ambient 0.1 and the half-white sun; with lights the
    /// sun is off and the point lights light the objects.
    #[test]
    fn scenes_without_lights_get_the_default_sun() {
        let scene = SceneData::default();
        let lighting = scene_lighting(&scene, None);
        assert_eq!(lighting.ambient, [0.1; 3]);
        assert_eq!(lighting.dir_brightness, 0.5);
        let dir = lighting.light_dir.normalize();
        assert!((dir.y - (-0.9 / 1.3784_f32)).abs() < 1e-3);
        let lit = SceneData {
            lights: vec![LightData {
                name: Some("Light".into()),
                origin: Vec3::new(0.0, 0.0, 2.0),
                kind: super::super::data::LIGHT_POINT,
                direction: Vec3::ZERO,
                range: 15.0,
                intensity: 0.5,
                diffuse: [1.0, 0.5, 0.25],
                specular: [1.0; 3],
            }],
            ..SceneData::default()
        };
        let lighting = scene_lighting(&lit, None);
        assert_eq!(lighting.dir_brightness, 0.0);
        let points = object_lights(&point_lights(&lit, false), Vec3::ZERO, 1.0);
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].diffuse, [0.5, 0.25, 0.125]);
    }
}
