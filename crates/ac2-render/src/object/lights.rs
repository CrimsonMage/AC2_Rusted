//! The lights an outdoor object is drawn with. `WorldRender::SetupEnvironment` hands the frame's
//! lighting to `SceneTool::SetSunlight` (direction, colour, brightness) and `SetAmbientLight`;
//! `SceneTool::ApplyLights` sends the sun to the device for objects in outdoor cells through
//! `LightSource::FillRenderLight`, and `RenderDeviceD3D::SetupVertexShaderConstants` derives the
//! vertex program's light constants from it.

use ac2_math::Vec3;

use crate::SceneLighting;

/// `SetSunlight` treats a direction with every component under this as unset.
const DIRECTION_EPSILON: f32 = 0.0002;

/// The device's directional light 0 for outdoor objects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SunLight {
    /// Unit direction the light travels, in world AC axes.
    pub direction: Vec3,
    /// `m_cDiffuse * m_wDiffuseIntensity * m_fOpacity`.
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

/// Lights for one frame of outdoor objects.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectLights {
    /// `RenderDeviceD3D::SetAmbient`: `m_finalAmbColor * m_finalAmbBright` outdoors. The vertex
    /// program's `c_AmbientColor` is this plus the lights' ambient (black for the sun), times the
    /// material ambient, capped at 1.
    pub ambient: [f32; 3],
    pub sun: Option<SunLight>,
}

impl ObjectLights {
    /// `SetSunlight(m_finalLightDir, m_finalDirColor, m_finalDirBright, m_finalDirColor)` and
    /// `SetAmbientLight(m_finalAmbColor * m_finalAmbBright)`.
    pub fn outdoor(lighting: &SceneLighting) -> Self {
        let dir = lighting.light_dir;
        let direction = if dir.abs().max_element() >= DIRECTION_EPSILON {
            dir.normalize()
        } else {
            Vec3::new(0.0, 0.0, -1.0)
        };
        let diffuse = lighting.dir_color.map(|c| c * lighting.dir_brightness);
        Self {
            ambient: lighting.ambient,
            sun: Some(SunLight {
                direction,
                diffuse,
                specular: lighting.dir_color,
            }),
        }
    }
}

/// The environment objects are drawn in (`WorldRender::SetupEnvironment`): the device lights
/// and the linear fog, if any.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectEnvironment {
    pub lights: ObjectLights,
    /// Fog colour and (start, end) metres; `None` is `FOGMODE_NONE`.
    pub fog: Option<([f32; 3], (f32, f32))>,
}

impl ObjectEnvironment {
    /// `OutsideEnvironment`: the sun, outdoor ambient and the region's linear fog.
    pub fn outside(lighting: &SceneLighting) -> Self {
        Self {
            lights: ObjectLights::outdoor(lighting),
            fog: Some((lighting.fog_color, lighting.fog_range)),
        }
    }

    /// `SkyEnvironment` (`WorldRender::SetupEnvironment`): the frame's sun with a black ambient
    /// (only `OutsideEnvironment` takes the region's ambient colour and brightness). The sky's
    /// fog is `LScapeSky::Draw`'s `fog_sky` screen quad, so the objects draw unfogged.
    pub fn sky(lighting: &SceneLighting) -> Self {
        let mut lights = ObjectLights::outdoor(lighting);
        lights.ambient = [0.0; 3];
        Self { lights, fog: None }
    }

    /// `InsideEnvironment`: the landblock's interior ambient, no sun (`ApplyLights` adds it only
    /// in outdoor cells) and the interior fog when it is linear (`0x40000002`).
    pub fn inside(ambient: [f32; 3], fog: Option<([f32; 3], (f32, f32))>) -> Self {
        Self {
            lights: ObjectLights { ambient, sun: None },
            fog,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ObjectLights;
    use crate::SceneLighting;
    use ac2_math::Vec3;

    #[test]
    fn sun_is_normalised_and_scaled_by_direct_brightness() {
        let lighting = SceneLighting {
            light_dir: Vec3::new(0.0, 3.0, -4.0),
            dir_color: [1.0, 0.5, 0.25],
            dir_brightness: 0.8,
            ..SceneLighting::default()
        };
        let sun = ObjectLights::outdoor(&lighting).sun.unwrap();
        assert!((sun.direction - Vec3::new(0.0, 0.6, -0.8)).length() < 1e-6);
        assert_eq!(sun.diffuse, [0.8, 0.4, 0.2]);
        assert_eq!(sun.specular, [1.0, 0.5, 0.25]);
        let dark = SceneLighting {
            light_dir: Vec3::ZERO,
            ..lighting
        };
        assert_eq!(
            ObjectLights::outdoor(&dark).sun.unwrap().direction,
            Vec3::new(0.0, 0.0, -1.0)
        );
    }
}
