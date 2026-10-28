//! The portal space's camera (Legions `portal/client/src/smartbox/worldrender/portalspace.cpp`,
//! `PortalSpace::Effect::Render`): the render position the tunnel is seen from. It interpolates
//! the two portals around the tunnel's current position: the translation of the matrices, the
//! rotation (`Matrix4::QuaternionFromRotation` of each, then `D3DXQuaternionSlerp`), and, while the
//! tunnel is fading in or out, a step back along the normalised interpolated row-1 vector by
//! `(1 - visibility) * 100`.

use ac2_math::{Quat, Vec3};

use super::tunnel::{make_safe_index, Portal, Tunnel};

/// `SceneCamera::SceneCamera`'s defaults, which `Render`'s `SceneCamera` keeps: the near and far
/// distances and the vertical field of view in degrees (`mNearDistance` 0.1, `mFarDistance`
/// 2000, `mFOV` 65). `Render` calls `SetFOV(mFOV * m_fCameraFOVScale)`; the decompile labels the
/// operand `camera.mFarDistance`, but the recovered export names it `local_f8.mFOV`.
pub const CAMERA_NEAR: f32 = 0.1;
pub const CAMERA_FAR: f32 = 2000.0;
pub const CAMERA_FOV_DEGREES: f32 = 65.0;

/// `SetFOV(mFOV * m_fCameraFOVScale)` in radians: the product in extended precision, stored as a
/// float in degrees (`SceneCamera::mFOV`), then the projection's `fov * pi / 360` half angle
/// doubled.
pub fn camera_fov_y(scale: f32) -> f32 {
    let degrees = (f64::from(CAMERA_FOV_DEGREES) * f64::from(scale)) as f32;
    f64::from(degrees).to_radians() as f32
}

/// The fade-in distance: the camera backs away `(1 - visibility) * 100` units.
const FADE_DISTANCE: f32 = 100.0;

/// The render position: where the camera is and its rotation (the frame's local axes; the view
/// looks along the rotated +y, the frame's forward, with +z up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortalCamera {
    pub eye: Vec3,
    pub rotation: Quat,
}

impl PortalCamera {
    /// The point the camera looks at: `eye` along the rotated forward.
    pub fn target(&self) -> Vec3 {
        self.eye + self.rotation * Vec3::Y
    }

    /// The render camera for this position: the frame's up is the world's +z (AC2's up).
    pub fn to_render_camera(&self, fov_y: f32, near: f32, far: f32) -> crate::Camera {
        crate::Camera {
            eye: self.eye,
            target: self.target(),
            fov_y,
            near,
            far,
        }
    }
}

/// `Render`'s camera: the portals at `last` and `last + 1` around `current`, blended by the
/// fraction `(current - p_last) / (p_next - p_last)`, stepped back for `visibility`.
pub fn portal_camera(tunnel: &Tunnel, last: usize, current: f32, visibility: f32) -> PortalCamera {
    let a = tunnel.portals[make_safe_index(last as i64)];
    let b = tunnel.portals[make_safe_index(last as i64 + 1)];
    let f = (current - a.position) / (b.position - a.position);
    let lerp = |i: usize| a.world.m[i] + (b.world.m[i] - a.world.m[i]) * f;
    let translation = Vec3::new(lerp(12), lerp(13), lerp(14));
    let forward = Vec3::new(lerp(4), lerp(5), lerp(6)).normalize_or_zero();
    let rotation = slerp(quaternion_of(&a), quaternion_of(&b), f);
    let fade = (1.0 - visibility) * FADE_DISTANCE;
    let eye = if visibility < 1.0 {
        translation - forward * fade
    } else {
        translation
    };
    PortalCamera { eye, rotation }
}

/// `Matrix4::QuaternionFromRotation`: the rotation of the matrix's three rows as the right,
/// forward and up axes (`Quaternion::SetRotationAxesAndNormalize`).
pub fn quaternion_of(portal: &Portal) -> Quat {
    let m = &portal.world.m;
    quaternion_from_axes(
        [f64::from(m[0]), f64::from(m[1]), f64::from(m[2])],
        [f64::from(m[4]), f64::from(m[5]), f64::from(m[6])],
        [f64::from(m[8]), f64::from(m[9]), f64::from(m[10])],
    )
}

/// `Quaternion::SetRotationAxesInternal` (the branch on the trace, then the largest diagonal),
/// in double precision; normalised as `SetRotationAxesAndNormalize` does.
fn quaternion_from_axes(right: [f64; 3], forward: [f64; 3], up: [f64; 3]) -> Quat {
    let (m00, m01, m02) = (right[0], right[1], right[2]);
    let (m10, m11, m12) = (forward[0], forward[1], forward[2]);
    let (m20, m21, m22) = (up[0], up[1], up[2]);
    let trace = m00 + m11 + m22;
    let [w, x, y, z] = if trace >= 0.0 {
        let s = (trace + 1.0).sqrt();
        [
            s * 0.5,
            (m12 - m21) * 0.5 / s,
            (m20 - m02) * 0.5 / s,
            (m01 - m10) * 0.5 / s,
        ]
    } else if m00 >= m11 && m00 >= m22 {
        let s = (m00 - m11 - m22 + 1.0).sqrt();
        let k = 0.5 / s;
        [(m12 - m21) * k, s * 0.5, (m01 + m10) * k, (m02 + m20) * k]
    } else if m11 >= m22 {
        let s = (m11 - m00 - m22 + 1.0).sqrt();
        let k = 0.5 / s;
        [(m20 - m02) * k, (m01 + m10) * k, s * 0.5, (m12 + m21) * k]
    } else {
        let s = (m22 - m00 - m11 + 1.0).sqrt();
        let k = 0.5 / s;
        [(m01 - m10) * k, (m02 + m20) * k, (m12 + m21) * k, s * 0.5]
    };
    let length_squared = w * w + x * x + y * y + z * z;
    let q = if length_squared > 3.999_999_6e-8 {
        let inverse = 1.0 / length_squared.sqrt();
        [w * inverse, x * inverse, y * inverse, z * inverse]
    } else {
        [w, x, y, z]
    };
    Quat::from_xyzw(q[1] as f32, q[2] as f32, q[3] as f32, q[0] as f32)
}

/// `D3DXQuaternionSlerp(out, a, b, t)`: the shorter arc, in double precision.
fn slerp(a: Quat, b: Quat, t: f32) -> Quat {
    let (a, b) = ([a.w, a.x, a.y, a.z], [b.w, b.x, b.y, b.z]);
    let mut cos = a
        .iter()
        .zip(&b)
        .map(|(x, y)| f64::from(*x) * f64::from(*y))
        .sum::<f64>();
    let mut b = b.map(f64::from);
    if cos < 0.0 {
        cos = -cos;
        b = b.map(|v| -v);
    }
    let a = a.map(f64::from);
    let t = f64::from(t);
    let (wa, wb) = if cos > 0.999_999 {
        (1.0 - t, t)
    } else {
        let theta = cos.acos();
        let sin = theta.sin();
        (((1.0 - t) * theta).sin() / sin, (t * theta).sin() / sin)
    };
    let q: [f64; 4] = std::array::from_fn(|i| wa * a[i] + wb * b[i]);
    Quat::from_xyzw(q[1] as f32, q[2] as f32, q[3] as f32, q[0] as f32)
}

#[cfg(test)]
mod tests {
    use ac2_math::matrix::Matrix4;
    use ac2_math::{Quat, Vec3};

    use super::{portal_camera, quaternion_of, slerp, Portal, Tunnel};

    fn portal(position: f32, translation: [f32; 3], world: Matrix4) -> Portal {
        let mut world = world;
        world.m[12..15].copy_from_slice(&translation);
        Portal {
            position,
            radius: 1.0,
            world,
            has_particle: false,
        }
    }

    #[test]
    fn an_identity_ring_is_the_identity_rotation() {
        let q = quaternion_of(&portal(0.0, [0.0; 3], Matrix4::identity()));
        assert!((q.w - 1.0).abs() < 1e-6 && q.length() - 1.0 < 1e-6);
        assert!(q.x.abs() < 1e-6 && q.y.abs() < 1e-6 && q.z.abs() < 1e-6);
    }

    #[test]
    fn a_quarter_turn_about_z_round_trips_through_the_quaternion() {
        let turn = Matrix4::rotate_z(std::f32::consts::FRAC_PI_2);
        let q = quaternion_of(&portal(0.0, [0.0; 3], turn));
        // Half the angle: w = cos(pi/4), z = sin(pi/4).
        let half = std::f32::consts::FRAC_PI_4;
        assert!(
            (q.w - half.cos()).abs() < 1e-5 && (q.z - half.sin()).abs() < 1e-5,
            "{q:?}"
        );
    }

    #[test]
    fn slerp_takes_the_halfway_arc() {
        let turn = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let halfway = slerp(Quat::IDENTITY, turn, 0.5);
        let expected = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
        assert!(halfway.dot(expected).abs() > 0.999_999, "{halfway:?}");
    }

    #[test]
    fn the_camera_is_at_the_interpolated_translation_when_fully_visible() {
        let mut tunnel = Tunnel::new();
        tunnel.portals[0] = portal(10.0, [0.0, 0.0, 0.0], Matrix4::identity());
        tunnel.portals[1] = portal(20.0, [10.0, 0.0, 4.0], Matrix4::identity());
        // Halfway between the portals: the translation is halfway.
        let camera = portal_camera(&tunnel, 0, 15.0, 1.0);
        assert!(
            (camera.eye - Vec3::new(5.0, 0.0, 2.0)).length() < 1e-5,
            "{:?}",
            camera.eye
        );
        assert_eq!(camera.target(), camera.eye + Vec3::Y);
    }

    #[test]
    fn the_camera_backs_away_along_the_forward_while_fading_in() {
        let mut tunnel = Tunnel::new();
        tunnel.portals[0] = portal(10.0, [0.0; 3], Matrix4::identity());
        tunnel.portals[1] = portal(20.0, [0.0; 3], Matrix4::identity());
        // Forward is the row-1 vector (0, 1, 0); visibility 0.75 backs away 25 units.
        let camera = portal_camera(&tunnel, 0, 10.0, 0.75);
        assert!(
            (camera.eye - Vec3::new(0.0, -25.0, 0.0)).length() < 1e-4,
            "{:?}",
            camera.eye
        );
    }
}
