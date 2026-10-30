//! AC1's joints. An AC1 Setup has no skeleton: every animation frame places each part in object
//! space, and a part's origin need not be where it turns (the human forearm's is mid-forearm).
//! The joint between a part and its child is the point the two carry together in every frame,
//! found here by least squares over the motion table's cycles, and drawn gently towards where
//! their bare shapes meet, which fixes the point along a hinge's axis (a hinge alone leaves it
//! free there).

use ac2_anim::Ac2Transform;
use ac2_math::{Quat, Vec3};
use ac2_physics::PartPose;

use crate::dats::Ac1Dats;
use crate::formats::Ac1Result;
use crate::retarget::BonePair;

/// The cycles whose frames place the joints: ready, run, walk back, side step and turn.
const MOTIONS: [u32; 5] = [
    0x4100_0003,
    0x4400_0007,
    0x4500_0006,
    0x6500_000F,
    0x6500_000D,
];
/// How strongly the joints are drawn to the shapes' meeting point, against each frame's fit.
const SHAPE_WEIGHT: f64 = 0.01;

impl Ac1Dats {
    /// The joints of `pairs`' limbs (each part and its child) on Setup `setup` animated by
    /// `motion_table`: (part, child part, joint in the reference pose `reference`, object
    /// space).
    pub fn limb_joints(
        &self,
        setup: u32,
        motion_table: u32,
        pairs: &[BonePair],
        reference: &[Ac2Transform],
    ) -> Ac1Result<Vec<(usize, usize, Vec3)>> {
        let shapes = self.part_shapes(setup)?;
        let mut frames: Vec<Vec<Ac2Transform>> = vec![reference.to_vec()];
        for motion in MOTIONS {
            if let Some(clip) = self.motion_cycle(motion_table, motion, reference.len())? {
                frames.extend(
                    clip.poses()
                        .map(|pose| pose.iter().map(PartPose::transform).collect()),
                );
            }
        }
        let mut joints = Vec::new();
        for pair in pairs {
            let Some(child) = pair.ac1_child else {
                continue;
            };
            let (Some(a), Some(b)) = (reference.get(pair.ac1), reference.get(child)) else {
                continue;
            };
            let guess = shape_meeting(a, b, shapes.get(pair.ac1), shapes.get(child));
            joints.push((pair.ac1, child, pivot(&frames, pair.ac1, child, guess)));
        }
        Ok(joints)
    }
}

/// Where part `a`'s bare shape ends and its child `b`'s begins, along the line between their
/// origins (the midpoint of the two ends); `b`'s origin without both shapes.
fn shape_meeting(
    a: &Ac2Transform,
    b: &Ac2Transform,
    a_shape: Option<&Vec<Vec3>>,
    b_shape: Option<&Vec<Vec3>>,
) -> Vec3 {
    let Some(dir) = (b.offset - a.offset).try_normalize() else {
        return b.offset;
    };
    let reach = |part: &Ac2Transform, shape: &[Vec3]| {
        shape
            .iter()
            .map(|v| (part.rotation * *v).dot(dir))
            .fold((f32::MAX, f32::MIN), |(lo, hi), d| (lo.min(d), hi.max(d)))
    };
    match (
        a_shape.filter(|s| !s.is_empty()),
        b_shape.filter(|s| !s.is_empty()),
    ) {
        (Some(sa), Some(sb)) => {
            let (_, a_end) = reach(a, sa);
            let (b_start, _) = reach(b, sb);
            ((a.offset + dir * a_end) + (b.offset + dir * b_start)) * 0.5
        }
        _ => b.offset,
    }
}

/// The point parts `a` and `b` carry together over `frames` (the first the reference pose), in
/// the reference pose: the local points `x_a`, `x_b` minimising `|R_a x_a + p_a - R_b x_b -
/// p_b|²` over the frames, plus `SHAPE_WEIGHT` times their reference places' distance to
/// `guess`.
fn pivot(frames: &[Vec<Ac2Transform>], a: usize, b: usize, guess: Vec3) -> Vec3 {
    let mut normal = [[0.0f64; 6]; 6];
    let mut rhs = [0.0f64; 6];
    let mut add_row = |row: [f64; 6], value: f64, weight: f64| {
        for i in 0..6 {
            for j in 0..6 {
                normal[i][j] += weight * row[i] * row[j];
            }
            rhs[i] += weight * row[i] * value;
        }
    };
    // The rotation's columns: the turned axes.
    let matrix = |q: Quat| [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| (q * axis).as_dvec3());
    for frame in frames {
        let (Some(fa), Some(fb)) = (frame.get(a), frame.get(b)) else {
            continue;
        };
        let (ra, rb) = (matrix(fa.rotation), matrix(fb.rotation));
        let gap = (fb.offset - fa.offset).as_dvec3();
        for axis in 0..3 {
            let mut row = [0.0; 6];
            for c in 0..3 {
                row[c] = ra[c][axis];
                row[3 + c] = -rb[c][axis];
            }
            add_row(row, gap[axis], 1.0);
        }
    }
    let (Some(ra0), Some(rb0)) = (frames[0].get(a), frames[0].get(b)) else {
        return guess;
    };
    for (offset, part) in [(0, ra0), (3, rb0)] {
        let r = matrix(part.rotation);
        let target = (guess - part.offset).as_dvec3();
        for axis in 0..3 {
            let mut row = [0.0; 6];
            for c in 0..3 {
                row[offset + c] = r[c][axis];
            }
            add_row(row, target[axis], SHAPE_WEIGHT);
        }
    }
    let Some(x) = solve(normal, rhs) else {
        return guess;
    };
    let local = |i: usize| Vec3::new(x[i] as f32, x[i + 1] as f32, x[i + 2] as f32);
    let at_a = ra0.offset + ra0.rotation * local(0);
    let at_b = rb0.offset + rb0.rotation * local(3);
    (at_a + at_b) * 0.5
}

/// `m x = v` by Gaussian elimination with partial pivoting; `None` when singular.
fn solve(mut m: [[f64; 6]; 6], mut v: [f64; 6]) -> Option<[f64; 6]> {
    for col in 0..6 {
        let pivot = (col..6).max_by(|&i, &j| m[i][col].abs().total_cmp(&m[j][col].abs()))?;
        if m[pivot][col].abs() < 1e-12 {
            return None;
        }
        m.swap(col, pivot);
        v.swap(col, pivot);
        let top = m[col];
        for row in col + 1..6 {
            let f = m[row][col] / top[col];
            for (cell, above) in m[row][col..].iter_mut().zip(&top[col..]) {
                *cell -= f * above;
            }
            v[row] -= f * v[col];
        }
    }
    let mut x = [0.0; 6];
    for row in (0..6).rev() {
        let sum: f64 = (row + 1..6).map(|k| m[row][k] * x[k]).sum();
        x[row] = (v[row] - sum) / m[row][row];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(offset: Vec3, rotation: Quat) -> Ac2Transform {
        Ac2Transform {
            offset,
            scale: Vec3::ONE,
            rotation,
        }
    }

    #[test]
    fn a_forearm_turning_about_its_elbow_finds_the_elbow_not_its_origin() {
        // An upper arm at the origin, hanging down; its forearm's origin 0.2 below the elbow
        // at (0, 0, -0.3), swinging about x through the elbow.
        let elbow = Vec3::new(0.0, 0.0, -0.3);
        let frames: Vec<Vec<Ac2Transform>> = [0.0f32, 0.4, 0.9, 1.3]
            .iter()
            .map(|&angle| {
                let q = Quat::from_rotation_x(angle);
                vec![
                    at(Vec3::ZERO, Quat::IDENTITY),
                    at(elbow + q * Vec3::new(0.0, 0.0, -0.2), q),
                ]
            })
            .collect();
        // A guess 0.15 off below: the frames fix y and z, the guess x (and pulls z a little).
        let found = pivot(&frames, 0, 1, Vec3::new(0.0, 0.0, -0.45));
        assert!((found - elbow).length() < 5e-3, "{found:?}");
        let along_hinge = pivot(&frames, 0, 1, Vec3::new(0.1, 0.0, -0.3));
        assert!(
            (along_hinge - Vec3::new(0.1, 0.0, -0.3)).length() < 1e-3,
            "{along_hinge:?}"
        );
    }

    #[test]
    fn solve_inverts_a_diagonal_system() {
        let mut m = [[0.0; 6]; 6];
        for (i, row) in m.iter_mut().enumerate() {
            row[i] = (i + 1) as f64;
        }
        let x = solve(m, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).expect("solves");
        assert!(x.iter().all(|v| (v - 1.0).abs() < 1e-12));
    }
}
