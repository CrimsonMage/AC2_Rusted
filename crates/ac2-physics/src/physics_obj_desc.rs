//! The physics half of `CPhysicsObj::set_description`: what a `PhysicsDesc` says about the body's
//! velocity, spin, external acceleration, speed and jump scales and head direction. The animation
//! half (mode, behaviours, sliders) is `PhysicsObj::set_description_animation`.

use crate::body::PhysicsBody;
use crate::position::Position;
use crate::vector::Vector;

/// The missile part of a `PhysicsDesc` (`bitfield & 0xC00`: a target id or a target position): a
/// missile that reaches the client already in flight. `PhysicsDesc::target_height` is never
/// streamed, so retail's receiving description holds its constructor's 0.
#[derive(Debug, Clone, PartialEq)]
pub struct DescMissile {
    /// `PhysicsDesc::target_pos` (an invalid position when the description has none).
    pub target_pos: Position,
    /// `m_missile_is_activated` (bit `0x100000`).
    pub activated: bool,
    /// `m_missile_is_moving` (bit `0x80000`).
    pub moving: bool,
    /// `target_id` (bit `0x400`) with `target_offset` and `use_target_scale` (bit `0x4000`).
    pub target: Option<(u64, Vector, bool)>,
}

/// The motion fields of a `PhysicsDesc`, with the constructor's defaults (`PhysicsDesc::PhysicsDesc`:
/// zero vectors, scales 1, head direction 1).
#[derive(Debug, Clone, PartialEq)]
pub struct DescPhysics {
    pub velocity: [f32; 3],
    pub omega: [f32; 3],
    pub external_acceleration: [f32; 3],
    pub velocity_scale: f32,
    pub jump_scale: f32,
    pub head_x: f32,
    pub head_z: f32,
    /// `PhysicsDesc::m_fx`: the effects running on the object, with their scalars.
    pub fx: Vec<(u32, f32)>,
    /// The missile branch of `set_description`, when the description carries one.
    pub missile: Option<DescMissile>,
}

impl Default for DescPhysics {
    fn default() -> Self {
        Self {
            velocity: [0.0; 3],
            omega: [0.0; 3],
            external_acceleration: [0.0; 3],
            velocity_scale: 1.0,
            jump_scale: 1.0,
            head_x: 1.0,
            head_z: 1.0,
            fx: Vec::new(),
            missile: None,
        }
    }
}

impl PhysicsBody {
    /// The head of `CPhysicsObj::set_description`: a body with a motion table takes the
    /// described velocity (`set_velocity`, culled and with its movement active), every body the
    /// angular velocity and the external acceleration field, then the speed and jump scales
    /// (`SetVelocityScale`, `SetJumpScale`) and the head direction (`m_fHeadX`, `m_fHeadZ`).
    /// The look-at id is only stored by retail (the `LookAt` event starts the look). A
    /// description with a target (`bitfield & 0xC00`) makes the body's missile manager when it has
    /// none and sets its target position, activation, moving state and target id, then the
    /// velocity as a missile's (the launch event is `recv_initialize_missile`).
    pub fn set_description_physics(&mut self, desc: &DescPhysics) {
        if self.object.motion.has_desc() {
            let [x, y, z] = desc.velocity;
            self.object.set_velocity_culled(Vector::new(x, y, z));
        }
        let [x, y, z] = desc.omega;
        self.set_omega_velocity(Vector::new(x, y, z));
        let [x, y, z] = desc.external_acceleration;
        self.object.acceleration_field_vel = Vector::new(x, y, z);
        self.set_velocity_scale(desc.velocity_scale);
        self.set_jump_scale(desc.jump_scale);
        self.object.head_x = desc.head_x;
        self.object.head_z = desc.head_z;
        // The effects already running are stopped and the described ones started
        // (`FXManager::GetCurrentFX`, `StopFX`, `DoTargettedFX`); a new body has none running,
        // and the FX target only steers the visual part.
        for &(fx, scalar) in &desc.fx {
            self.do_fx(fx, scalar);
        }
        if let Some(missile) = &desc.missile {
            self.object.describe_missile(missile, desc.velocity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_the_constructors() {
        let d = DescPhysics::default();
        assert_eq!((d.velocity_scale, d.jump_scale, d.head_x), (1.0, 1.0, 1.0));
    }

    #[test]
    fn a_description_sets_scales_spin_head_and_the_external_field() {
        let mut body = crate::test_support::body_at(70.0, 70.0, 50.0);
        body.set_description_physics(&DescPhysics {
            velocity: [0.0, 3.0, 0.0],
            omega: [0.0, 0.0, 1.5],
            external_acceleration: [0.0, 0.0, 2.0],
            velocity_scale: 2.0,
            jump_scale: 0.5,
            head_x: 0.25,
            head_z: -0.5,
            fx: vec![(0x40, 1.0)],
            missile: None,
        });
        let events = body.take_animation_events();
        assert!(events.iter().any(|e| matches!(
            e,
            ac2_anim::Ac2AnimEvent::StartFx { fx: 0x40, intensity, .. } if *intensity == 1.0
        )));
        assert_eq!(body.velocity_scale(), 2.0);
        assert_eq!(body.jump_scale(), 0.5);
        assert_eq!(body.omega_velocity().z, 1.5);
        assert_eq!((body.look_at_x(), body.look_at_z()), (0.25, -0.5));
        // The test body has a motion table, so it takes the velocity.
        assert_eq!(body.velocity().y, 3.0);
        body.with_object(|o| assert_eq!(o.acceleration_field_vel.z, 2.0));
    }

    #[test]
    fn a_description_with_a_target_makes_a_missile_in_flight() {
        let mut body = crate::test_support::body_at(70.0, 70.0, 50.0);
        assert!(!body.has_missile_manager());
        body.set_description_physics(&DescPhysics {
            velocity: [0.0, 12.0, 0.0],
            missile: Some(DescMissile {
                target_pos: Position::default(),
                activated: true,
                moving: true,
                target: Some((0x77, Vector::new(0.0, 0.0, 0.5), true)),
            }),
            ..DescPhysics::default()
        });
        assert!(body.has_missile_manager());
        body.with_object(|o| {
            let m = o.missile.as_ref().expect("manager");
            assert!(m.is_activated && m.is_moving);
            let t = &m.target_info;
            assert_eq!((t.target_id, t.use_target_scale), (0x77, true));
            assert_eq!(t.target_height_adjustment, 0.0);
            assert_eq!(t.target_offset.z, 0.5);
            assert_eq!(t.bitfield & 3, 3);
            assert_eq!(o.velocity.y, 12.0);
        });
    }
}
