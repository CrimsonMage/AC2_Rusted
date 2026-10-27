//! The collision impact minimums (`CCollisionManager::SetMinReportedImpactXY` and `Z`, set by
//! `CPhysicsObj::SetVelocityScale`, `CPhysicsObj::SetJumpScale` and `CPhysicsObj::InitObjectEnd`)
//! and the gate `CCollisionManager::SetCollisionNormal` puts on an impact: a contact only records
//! its impact speed when the vertical fall is above the minimum. The collision manager's sounds and
//! reports are not ported; the body keeps the impact it would report.

use crate::fpu::{ld, st};
use crate::vector::Vector;

/// The floor under the vertical minimum in `SetCollisionNormal` (`0.0002`).
const MIN_IMPACT_FLOOR: f32 = 0.0002;
/// Added to the scaled minimums (`+ 0.1`).
const MIN_IMPACT_OFFSET: f32 = 0.1;

/// `m_min_reported_impactXY` and `m_min_reported_impactZ` of the collision manager's current info.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ImpactMinimums {
    pub xy: f32,
    pub z: f32,
}

impl ImpactMinimums {
    /// `CPhysicsObj::SetVelocityScale`: the XY minimum is the scale times the standard values'
    /// `mMaxForwardVel`, plus 0.1 (evaluated in f64, rounded on store).
    pub(crate) fn set_velocity_scale(&mut self, scale: f32, max_forward_velocity: f32) {
        self.xy = st(ld(scale) * ld(max_forward_velocity) + ld(MIN_IMPACT_OFFSET));
    }

    /// `CPhysicsObj::SetJumpScale`: the Z minimum is the jump scale times the standard values'
    /// `mJumpImpulse`, plus 0.1. Retail reads that field through `CPartArray::GetOpacity` (Ghidra
    /// typed the object as a part array; the code at +0x50 is `MotionValues::mJumpImpulse`).
    pub(crate) fn set_jump_scale(&mut self, jump_scale: f32, jump_impulse: f32) {
        self.z = st(ld(jump_scale) * ld(jump_impulse) + ld(MIN_IMPACT_OFFSET));
    }
}

/// `CCollisionManager::SetCollisionNormal`: the impact speed of a contact with `normal` at
/// `velocity`, when the vertical speed is above the Z minimum (floored at 0.0002). The speed is
/// `-(velocity.z * normal.z)`. `None` when the contact is too slow to record.
pub(crate) fn impact_speed(minimum_z: f32, normal: Vector, velocity: Vector) -> Option<f32> {
    let floor = if minimum_z < MIN_IMPACT_FLOOR {
        MIN_IMPACT_FLOOR
    } else {
        minimum_z
    };
    (floor < velocity.z.abs()).then(|| st(-(ld(velocity.z) * ld(normal.z))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f32, y: f32, z: f32) -> Vector {
        Vector::new(x, y, z)
    }

    #[test]
    fn the_xy_minimum_scales_the_standard_forward_speed_plus_a_tenth() {
        let mut m = ImpactMinimums::default();
        m.set_velocity_scale(0.5, 4.0);
        assert!((m.xy - 2.1).abs() < 1e-6, "{}", m.xy);
    }

    #[test]
    fn the_z_minimum_scales_the_jump_impulse_plus_a_tenth() {
        let mut m = ImpactMinimums::default();
        m.set_jump_scale(1.5, 2.0);
        assert!((m.z - 3.1).abs() < 1e-6, "{}", m.z);
    }

    #[test]
    fn a_fall_above_the_minimum_records_its_speed_and_a_slower_one_does_not() {
        let up = v(0.0, 0.0, 1.0);
        // Falling at 2 against a minimum of 1: recorded, as -(vz * nz) = 2.
        assert_eq!(impact_speed(1.0, up, v(0.0, 0.0, -2.0)), Some(2.0));
        // Falling at 0.5: below the minimum of 1, not recorded.
        assert_eq!(impact_speed(1.0, up, v(0.0, 0.0, -0.5)), None);
    }

    #[test]
    fn the_floor_keeps_a_zero_minimum_above_0_0002() {
        let up = v(0.0, 0.0, 1.0);
        assert_eq!(impact_speed(0.0, up, v(0.0, 0.0, -0.0001)), None);
        assert_eq!(impact_speed(0.0, up, v(0.0, 0.0, -0.0003)), Some(0.0003));
    }

    #[test]
    fn a_sloped_contact_records_the_normal_component() {
        let slope = v(0.6, 0.0, 0.8);
        // -(vz * nz) = -(-5 * 0.8) = 4.
        let speed = impact_speed(1.0, slope, v(0.0, 0.0, -5.0)).expect("above the minimum");
        assert!((speed - 4.0).abs() < 1e-6, "{speed}");
    }
}
