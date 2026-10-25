//! The UI scenes' cameras and lights.
//!
//! A viewport renders through its camera entity (`UIScene::Render`, `SceneTool::SetupCamera`):
//! the view from the entity's frame looking along its +Y axis (`SceneTool::SetupView`'s forward
//! row), the vertical field of view in degrees and the viewport's aspect
//! (`SetupPerspectiveProjection`), or an orthographic view `mVerticalScale` metres high
//! (`SetupOrthographicProjection`). The renderer's camera is perspective, so an orthographic
//! one looks from [`ORTHO_DISTANCE`] behind the camera through the field of view that spans the
//! vertical scale there (sizes differ from a true orthographic view by about distance / 10 km).
//! A frame may make `Light_1` a light shining from the camera (avatar modification's
//! `UpdateCamera`).
//!
//! The scene's light entities are linked as the lights (`SceneTool::LinkLight`) and the sun is
//! disabled; a scene without lights gets ambient 0.1 and the sun along (0.3, -0.9, -1) at
//! half white (`SetAmbientLight`, `SetSunlight`).

use ac2_math::Vec3;
use ac2_render::object::{object_point_lights, PointLight, SceneLight};
use ac2_render::{Camera, SceneLighting};
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
/// How far behind an orthographic camera its view is taken from.
pub(crate) const ORTHO_DISTANCE: f32 = 10_000.0;
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

/// `SceneTool::SetupCamera`: the camera at its frame; an orthographic one (`vertical_scale`
/// metres high) from [`ORTHO_DISTANCE`] behind. Also the distance the eye stands behind the
/// camera's origin (for the fog).
pub(crate) fn scene_camera(camera: &CameraData) -> (Camera, f32) {
    let forward = camera.frame.orientation * Vec3::Y;
    let origin = camera.frame.origin;
    if !camera.orthographic {
        let view = Camera {
            eye: origin,
            target: origin + forward,
            fov_y: camera.fov.to_radians(),
            near: camera.near,
            far: camera.far,
        };
        return (view, 0.0);
    }
    let distance = f64::from(ORTHO_DISTANCE);
    let fov = 2.0 * (f64::from(camera.vertical_scale) * 0.5 / distance).atan();
    let view = Camera {
        eye: origin - forward * ORTHO_DISTANCE,
        target: origin,
        fov_y: fov as f32,
        // An orthographic camera's near plane may be behind it (the paper doll's is -20).
        near: (ORTHO_DISTANCE + camera.near).max(1.0),
        far: ORTHO_DISTANCE + camera.far,
    };
    (view, ORTHO_DISTANCE)
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

    /// `SetupOrthographicProjection`: `vertical_scale` metres high at the camera's plane,
    /// looking along the frame's +Y; the near plane may lie behind the camera.
    #[test]
    fn an_orthographic_camera_spans_its_vertical_scale() {
        let base = CameraData {
            name: None,
            frame: ac2_math::Frame::IDENTITY,
            orthographic: true,
            fov: 90.0,
            vertical_scale: 2.0,
            near: -20.0,
            far: 20.0,
        };
        let (view, offset) = scene_camera(&base);
        assert_eq!(offset, ORTHO_DISTANCE);
        assert!((view.target - Vec3::ZERO).length() < 1e-6);
        assert!(view.eye.y < -9_999.0, "behind the camera along -Y");
        let vp = view.view_projection(1.0);
        let top = vp.project_point3(Vec3::new(0.0, 0.0, 1.0));
        assert!((top.y - 1.0).abs() < 1e-3, "{top:?}");
        let behind = vp.project_point3(Vec3::new(0.0, -10.0, 0.0));
        assert!((0.0..=1.0).contains(&behind.z), "{behind:?}");
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
