//! Where a body is drawn between physics quanta: retail's render interpolation, and the
//! player's led heading (`AGENTS.md`).

use crate::body::{PhysicsBody, PHYSICS_QUANTUM};
use crate::frame::Frame;
use crate::position::Position;
use crate::vector::{Quaternion, Vector};

impl PhysicsBody {
    /// The drawn position (`CPhysicsObj::PrepareObjectForDrawing`, and
    /// `TargetingModule::GetPosition` for the camera's target): `Position::slerp` from the
    /// previous to the current position by `t`, the clock's render interpolation
    /// (`CPhysics::sRenderInterpolation`, `PhysicsClock::render_interpolation`). A static body
    /// draws where it is.
    pub fn drawing_position(&self, t: f32) -> Position {
        let o = &self.object;
        if o.state & crate::objects::STATIC_PS != 0 {
            return o.position;
        }
        let mut drawn = o.position;
        drawn.slerp(&o.prev_position, &o.position, t);
        drawn
    }

    /// The player's drawn place `t` of the way through the current quantum: the origin as
    /// [`Self::drawing_position`] interpolates it, but the rotation is the latest quantum's turned
    /// on by what the next quantum's `UpdateRotation` will turn over that part of it (the yaw
    /// rate and the acceleration the held turn input picks). A turn shows from the frame its key
    /// goes down, and the next quantum lands where it was drawn, instead of the drawn heading
    /// trailing the physics one by up to a quantum (not retail: `CPhysics::sRenderInterpolation`
    /// slerps from the previous quantum).
    pub fn led_drawing_position(&self, t: f32) -> Position {
        let o = &self.object;
        let mut drawn = self.drawing_position(t);
        if o.state & crate::objects::STATIC_PS != 0 {
            return drawn;
        }
        let dt = f64::from(t) * f64::from(PHYSICS_QUANTUM);
        let lead = o
            .motion
            .turn_lead(o.desired_element(), o.omega.z, self.turn, dt);
        let mut offset = Frame::IDENTITY;
        offset.grotate(Vector::new(0.0, 0.0, lead.to_radians() as f32));
        drawn
            .frame
            .set_rotate(Quaternion::multiply(o.position.frame.q, offset.q));
        drawn
    }
}
