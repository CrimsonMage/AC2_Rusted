//! Blending one bone's transform between two animation quanta (the drawn pose between the last
//! two refreshes). Not retail: the retail client draws the bones of the last refresh. The
//! blend is evaluated in `f64` and rounded to `f32` at the store.

use ac2_math::{Quat, Vec3};

use crate::Ac2Transform;

/// `from` at `t = 0`, `to` at `t = 1`, `t` clamped to `[0, 1]`: the offset and scale lerp, the
/// rotation slerps the shorter way round. The endpoints are returned exactly.
pub fn blend_transform(from: &Ac2Transform, to: &Ac2Transform, t: f32) -> Ac2Transform {
    if !(t > 0.0) {
        return *from;
    }
    if t >= 1.0 {
        return *to;
    }
    let t = f64::from(t);
    Ac2Transform {
        offset: lerp3(from.offset, to.offset, t),
        scale: lerp3(from.scale, to.scale, t),
        rotation: slerp(from.rotation, to.rotation, t),
    }
}

fn lerp3(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    let (a, b) = (a.to_array(), b.to_array());
    let mut out = [0.0f32; 3];
    for i in 0..3 {
        out[i] = (f64::from(a[i]) + (f64::from(b[i]) - f64::from(a[i])) * t) as f32;
    }
    Vec3::from(out)
}

/// The shortest-arc slerp of two rotations, in `f64`. `q` and `-q` are the same rotation, so
/// the second is negated when it lies more than 90 degrees from the first. Nearly parallel
/// rotations are normalised-lerped to avoid dividing by a vanishing `sin`.
fn slerp(a: Quat, b: Quat, t: f64) -> Quat {
    let a = a.to_array().map(f64::from);
    let mut b = b.to_array().map(f64::from);
    let mut dot: f64 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
    if dot < 0.0 {
        b = b.map(|v| -v);
        dot = -dot;
    }
    let (wa, wb) = if dot > 0.9995 {
        (1.0 - t, t)
    } else {
        let theta = dot.clamp(-1.0, 1.0).acos();
        let sin = theta.sin();
        (((1.0 - t) * theta).sin() / sin, (t * theta).sin() / sin)
    };
    let mut q = [0.0f64; 4];
    for i in 0..4 {
        q[i] = wa * a[i] + wb * b[i];
    }
    let len = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    if len > 0.0 {
        q = q.map(|v| v / len);
    }
    Quat::from_array(q.map(|v| v as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(offset: [f32; 3], rotation: Quat) -> Ac2Transform {
        Ac2Transform {
            offset: Vec3::from(offset),
            scale: Vec3::new(1.0, 2.0, 3.0),
            rotation,
        }
    }

    fn about_z(degrees: f32) -> Quat {
        Quat::from_rotation_z(degrees.to_radians())
    }

    fn close(a: Quat, b: Quat) -> bool {
        // `q` and `-q` are one rotation.
        a.dot(b).abs() > 1.0 - 1e-5
    }

    #[test]
    fn fraction_zero_is_the_previous_quantum_and_one_the_current() {
        let previous = at([0.0, 0.0, 0.0], about_z(0.0));
        let current = at([4.0, -2.0, 1.0], about_z(90.0));
        assert_eq!(blend_transform(&previous, &current, 0.0), previous);
        assert_eq!(blend_transform(&previous, &current, 1.0), current);
        // Out-of-range fractions clamp to the endpoints.
        assert_eq!(blend_transform(&previous, &current, -0.5), previous);
        assert_eq!(blend_transform(&previous, &current, 2.0), current);
    }

    #[test]
    fn half_way_is_the_midpoint_of_offset_scale_and_rotation() {
        let previous = at([0.0, 0.0, 0.0], about_z(0.0));
        let current = at([4.0, -2.0, 1.0], about_z(90.0));
        let mid = blend_transform(&previous, &current, 0.5);
        assert!(
            (mid.offset - Vec3::new(2.0, -1.0, 0.5)).length() < 1e-6,
            "{mid:?}"
        );
        assert!((mid.scale - Vec3::new(1.0, 2.0, 3.0)).length() < 1e-6);
        assert!(close(mid.rotation, about_z(45.0)), "{:?}", mid.rotation);
    }

    #[test]
    fn rotation_slerps_the_shorter_way_round() {
        // `-q` is the same rotation as `q`, 90 degrees about z. A slerp that ignores the sign
        // turns the long way (through 180 degrees) and lands on 225 degrees at the half way.
        let from = about_z(0.0);
        let to = -about_z(90.0);
        let mid = blend_transform(&at([0.0; 3], from), &at([0.0; 3], to), 0.5);
        assert!(close(mid.rotation, about_z(45.0)), "{:?}", mid.rotation);
    }

    #[test]
    fn a_fraction_past_the_turn_is_still_a_rotation_between_the_ends() {
        let from = about_z(-30.0);
        let to = about_z(60.0);
        for step in 1..10 {
            let t = step as f32 / 10.0;
            let mid = blend_transform(&at([0.0; 3], from), &at([0.0; 3], to), t);
            assert!(close(mid.rotation, about_z(-30.0 + 90.0 * t)), "t={t}");
        }
    }
}
