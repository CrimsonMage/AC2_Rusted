//! `AMLook` (`DBAnimator` type 0x40000005): turns the neck and head bones by the look sliders.
//! It adds no pose of its own: `AMLook::Animate` rotates the bones the layers below it already
//! posed, `AMLook::GetPeriod` is one second and `AMLook::GetTriggers` adds nothing.

use super::{Ac2AnimatorContext, Ac2BoneTransform};

/// The look sliders `AMLook::Animate` reads: the turn about each axis.
const SLIDER_LOOK_X: u32 = 0x4000_0007;
const SLIDER_LOOK_Y: u32 = 0x4000_0008;
const SLIDER_LOOK_Z: u32 = 0x4000_0009;
/// The bone types that turn by the larger share of the look (`AnimSetup::IsBoneType`)...
const BONES_NECK: [u32; 2] = [0x4000_0003, 0x4000_0005];
/// ...and those that turn by the smaller.
const BONES_HEAD: [u32; 2] = [0x4000_0004, 0x4000_0006];
/// `F_EPSILON`.
const F_EPSILON: f32 = 0.0002;

/// `AMLook::GetPeriod`.
pub(super) const LOOK_PERIOD: f32 = 1.0;

fn sum_squares(q: [f32; 4]) -> f32 {
    let q = q.map(f64::from);
    (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]) as f32
}

fn any_nan(values: &[f32]) -> bool {
    values.iter().any(|v| v.is_nan())
}

/// `D3DXVec3Normalize`: the vector over its float length.
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let [x, y, z] = v.map(f64::from);
    let norm = f64::from((x * x + y * y + z * z).sqrt() as f32);
    if norm == 0.0 {
        return [0.0; 3];
    }
    [(x / norm) as f32, (y / norm) as f32, (z / norm) as f32]
}

/// `TransformBase::NormalizeIfInvalid`: a transform with a NaN offset, scale or rotation, or a
/// rotation whose squared norm is not within `F_EPSILON` of one, has its rotation renormalised
/// (`NormalizeEvenInvalid`: identity when the squared norm is under 0.0002).
fn normalize_if_invalid(transform: &mut Ac2BoneTransform) {
    let valid = !any_nan(&transform.offset)
        && !any_nan(&transform.scale)
        && !any_nan(&transform.rotation)
        && {
            let norm = sum_squares([
                transform.rotation[0],
                transform.rotation[1],
                transform.rotation[2],
                transform.rotation[3],
            ]);
            !norm.is_nan() && (f64::from(norm) - 1.0).abs() < f64::from(F_EPSILON)
        };
    if valid {
        return;
    }
    let [w, x, y, z] = transform.rotation;
    let z2 = f64::from(z) * f64::from(z);
    let norm = (z2
        + f64::from(y) * f64::from(y)
        + f64::from(x) * f64::from(x)
        + f64::from(w) * f64::from(w)) as f32;
    if norm < 0.0002 {
        transform.rotation = [1.0, 0.0, 0.0, 0.0];
        return;
    }
    let inverse = (1.0 / f64::from(norm).sqrt()) as f32;
    transform.rotation = [w, x, y, z].map(|c| (f64::from(inverse) * f64::from(c)) as f32);
}

/// `Quaternion::multiply` (`a * b`), term order as the x87 code accumulates it.
fn multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [aw, ax, ay, az] = a.map(f64::from);
    let [bw, bx, by, bz] = b.map(f64::from);
    [
        (((aw * bw - bx * ax) - by * ay) - bz * az) as f32,
        (((ax * bw + bx * aw) + bz * ay) - by * az) as f32,
        (((by * aw - bz * ax) + ay * bw) + bx * az) as f32,
        (((bz * aw + by * ax) - bx * ay) + az * bw) as f32,
    ]
}

/// `Quaternion::rotate`: turn `q` by the rotation vector `v` (axis times angle in radians);
/// `None` (unchanged) under `F_EPSILON`.
fn rotate(q: [f32; 4], v: [f32; 3]) -> Option<[f32; 4]> {
    let [x, y, z] = v.map(f64::from);
    let norm = z * z + y * y + x * x;
    let epsilon = f64::from(F_EPSILON);
    if norm < epsilon * epsilon {
        return None;
    }
    let angle = norm.sqrt();
    let inverse = 1.0 / angle;
    let half = angle * 0.5;
    let sine = f64::from(half.sin() as f32);
    let turn = [
        half.cos() as f32,
        (sine * x * inverse) as f32,
        (sine * inverse * y) as f32,
        (sine * inverse * z) as f32,
    ];
    Some(multiply(turn, q))
}

/// `AMLook::Animate`: the look sliders (at `weight`, kept within the unit sphere) turn the neck
/// bones by 0.35 of the x look and 0.35 of the y and z looks over four, and the head bones by
/// 0.15345 of the x look and 0.93 of a quarter of y and z (the neck's y and z a further 1.4);
/// a bone keeps the turn only when its rotation stays a valid unit quaternion.
pub(super) fn animate_look(
    ctx: &dyn Ac2AnimatorContext,
    weight: f32,
    out: &mut [Ac2BoneTransform],
) {
    let w = f64::from(weight);
    let mut look = [SLIDER_LOOK_X, SLIDER_LOOK_Y, SLIDER_LOOK_Z]
        .map(|slider| (f64::from(ctx.slider(slider)) * w) as f32);
    let [x, y, z] = look.map(f64::from);
    if 1.0 < z * z + y * y + x * x {
        look = normalize(look);
    }
    let [x, y, z] = look.map(f64::from);
    let neck = [
        (x * f64::from(0.35_f32)) as f32,
        (f64::from(1.4_f32) * (y * 0.25)) as f32,
        (f64::from(1.4_f32) * (z * 0.25)) as f32,
    ];
    let head = [
        (x * f64::from(0.153_450_01_f32)) as f32,
        ((y * 0.25) * f64::from(0.93_f32)) as f32,
        ((z * 0.25) * f64::from(0.93_f32)) as f32,
    ];
    if any_nan(&neck) {
        return;
    }
    for (bone, transform) in out.iter_mut().enumerate() {
        let turn = if BONES_NECK.iter().any(|&ty| ctx.is_bone_type(bone, ty)) {
            neck
        } else if BONES_HEAD.iter().any(|&ty| ctx.is_bone_type(bone, ty)) {
            head
        } else {
            continue;
        };
        normalize_if_invalid(transform);
        let mut q = transform.rotation;
        if let Some(turned) = rotate(q, turn) {
            q = turned;
        }
        let norm = sum_squares(q);
        if !any_nan(&q) && !norm.is_nan() && (f64::from(norm) - 1.0).abs() < f64::from(F_EPSILON) {
            transform.rotation = q;
        }
    }
}
