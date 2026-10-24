//! The sky as `LScapeSky` draws it: each `CelestialPosition` from `CDayDesc::GetSky` becomes an
//! object centred on the viewer (`SetViewerPosition`), turned to its heading and then pitched about
//! the world Y axis (`CalcFrame`: `Frame::set_heading`, `Frame::grotate(0, -pitch, 0)`), and tinted
//! by its colour (`UpdateTimeOfDay`). `Draw(0)` draws the objects before the world and then the
//! sky fog quad; `Draw(1)` draws the rest after it.

use ac2_core::DataId;
use ac2_math::{Frame, Mat4, Vec3};
use ac2_world::CelestialPosition;

/// `Draw` skips the before-world objects when the sky is fogged out (`|fog_sky - 1| < 0.0002`).
const FOG_SKY_EPSILON: f32 = 0.0002;

/// One sky object to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyPlacement {
    pub gfx: DataId,
    pub world: Mat4,
    pub tint: [f32; 4],
}

/// The frame's sky objects, split into those drawn before the world and those drawn after it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkyFrame {
    pub before: Vec<SkyPlacement>,
    pub after: Vec<SkyPlacement>,
    /// `DrawScreenQuad` colour over the before-world objects (fog colour, alpha `fog_sky`),
    /// or `None` when `fog_sky` is 0.
    pub fog_quad: Option<[f32; 4]>,
}

/// `LScapeSky::SetViewerPosition` and `Draw` for `objects` seen from `viewer`.
pub fn sky_frame(
    objects: &[CelestialPosition],
    viewer: Vec3,
    in_water: bool,
    fog_color: [f32; 3],
    fog_sky: f32,
) -> SkyFrame {
    let mut frame = SkyFrame {
        fog_quad: (fog_sky > 0.0).then_some([fog_color[0], fog_color[1], fog_color[2], fog_sky]),
        ..SkyFrame::default()
    };
    for object in objects {
        let Some(gfx) = object.gfx else {
            continue;
        };
        if object.water != 0 && object.water != u32::from(in_water) {
            continue;
        }
        let placement = SkyPlacement {
            gfx,
            world: celestial_frame(object.heading, object.pitch, viewer).to_mat4(),
            tint: object.color,
        };
        if object.after {
            frame.after.push(placement);
        } else if (fog_sky - 1.0).abs() >= FOG_SKY_EPSILON {
            frame.before.push(placement);
        }
    }
    frame
}

/// `LScapeSky::CalcFrame` on an identity frame, then the viewer's origin.
fn celestial_frame(heading: f32, pitch: f32, viewer: Vec3) -> Frame {
    let mut frame = Frame::IDENTITY;
    if heading != 0.0 {
        frame.set_heading(heading);
    }
    if pitch != 0.0 {
        frame.rotate_global(Vec3::new(0.0, -pitch * 0.017_453_292, 0.0));
    }
    frame.origin = viewer;
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(after: bool, heading: f32, pitch: f32) -> CelestialPosition {
        CelestialPosition {
            gfx: Some(DataId::new(0x0400_0001)),
            heading,
            pitch,
            color: [1.0, 0.5, 0.25, 0.75],
            after,
            water: 0,
            height_mod: 0.0,
            fixed_pos: false,
            fixed_x: 0,
            fixed_y: 0,
        }
    }

    #[test]
    fn objects_ride_the_viewer_and_split_by_draw_order() {
        let viewer = Vec3::new(10.0, 20.0, 30.0);
        let sky = sky_frame(
            &[object(false, 0.0, 0.0), object(true, 90.0, 0.0)],
            viewer,
            false,
            [1.0; 3],
            0.5,
        );
        assert_eq!(sky.before.len(), 1);
        assert_eq!(sky.after.len(), 1);
        assert_eq!(sky.fog_quad, Some([1.0, 1.0, 1.0, 0.5]));
        assert_eq!(sky.before[0].world.w_axis.truncate(), viewer);
        // Heading 90 turns local north (+y) to east (+x).
        let east = sky.after[0].world.transform_vector3(Vec3::Y);
        assert!((east - Vec3::X).length() < 1e-5, "{east}");
        // A fully fogged sky drops the before-world objects.
        let fogged = sky_frame(&[object(false, 0.0, 0.0)], viewer, false, [1.0; 3], 1.0);
        assert!(fogged.before.is_empty());
    }

    #[test]
    fn pitch_rotates_about_world_y() {
        let frame = celestial_frame(0.0, 90.0, Vec3::ZERO);
        // grotate(0, -pi/2, 0) takes +x to +z.
        let v = frame.local_to_global_vector(Vec3::X);
        assert!((v - Vec3::Z).length() < 1e-5, "{v}");
    }
}
