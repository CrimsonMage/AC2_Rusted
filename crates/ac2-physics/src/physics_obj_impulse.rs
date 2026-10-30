//! `CPhysicsObj`'s impulses: the server's knockback (`ECM_Physics::RecvEvt_AddImpulse`,
//! `RecvEvt_AddTargetedImpulse`) through `CPhysicsObj::AddImpulse` and `AddTargetedImpulse`.

use super::{PhysicsObj, CONTACT_TS};
use crate::fpu::{ld, st, F_EPSILON};
use crate::position::Position;
use crate::vector::Vector;

impl PhysicsObj {
    /// `CPhysicsObj::AddImpulse(direction, contact)`: the world-axis direction is added to the
    /// velocity (`fadds` on the body's `m_vVelocity`, each component rounded to `float`) and the
    /// body leaves contact (`m_transient_state &= ~CONTACT_TS`), so the next quantum's transition
    /// reports the ground again. The retail `contact` argument is passed on to the velocity
    /// setter, which this model does not separate.
    pub(crate) fn add_impulse(&mut self, direction: Vector) {
        self.transient_state &= !CONTACT_TS;
        let v = self.velocity;
        self.velocity = Vector::new(
            st(ld(direction.x) + ld(v.x)),
            st(ld(direction.y) + ld(v.y)),
            st(ld(direction.z) + ld(v.z)),
        );
    }

    /// `CPhysicsObj::AddImpulseLocal(impulse, contact)`: the impulse in the body's own axes,
    /// turned by its frame (`Frame::localtoglobalvec`) and added by `AddImpulse`.
    pub(crate) fn add_impulse_local(&mut self, impulse: Vector) {
        let global = self.position.frame.local_to_global_vec(impulse);
        self.add_impulse(global);
    }

    /// `CPhysicsObj::AddTargetedImpulse(target, magnitude, falloff, vertical)` with the target's
    /// position as `source`: the impulse points from the source to this body (`GetOffset` from
    /// this body to the source, negated), normalised to `magnitude`. With `falloff` the
    /// magnitude is divided by the distance a second time, so it falls off with distance. An
    /// impulse within `F_EPSILON` of the source has no horizontal part. `vertical` is added to
    /// the z component. The result goes through `AddImpulse` with contact 0.
    pub(crate) fn add_targeted_impulse(
        &mut self,
        source: &Position,
        magnitude: f32,
        falloff: bool,
        vertical: f32,
    ) {
        let off = self.position.offset_to(source);
        // `fVar1` (z), `fVar4` (y) and `VStack_c.x`: the offset negated, from the source.
        let (fz, fy, mut x) = (-off.z, -off.y, -off.x);
        let len2 = st((ld(fz) * ld(fz) + ld(fy) * ld(fy)) + ld(x) * ld(x));
        let len = st(ld(len2).sqrt());
        let (mut ox, mut oy, mut oz) = (0.0_f32, 0.0_f32, 0.0_f32);
        if ld(len) >= ld(F_EPSILON) {
            let inv = st(1.0 / ld(len));
            x = st(ld(x) * ld(inv));
            let y = st(ld(fy) * ld(inv));
            let mut z = st(ld(inv) * ld(fz) * ld(magnitude));
            ox = st(ld(x) * ld(magnitude));
            oy = st(ld(y) * ld(magnitude));
            if falloff {
                let inv_far = st(1.0 / ld(len2).sqrt());
                ox = st(ld(inv_far) * ld(x) * ld(magnitude));
                oy = st(ld(y) * ld(magnitude) * ld(inv_far));
                z = st(ld(inv_far) * ld(z));
            }
            oz = z;
        }
        oz = st(ld(oz) + ld(vertical));
        self.add_impulse(Vector::new(ox, oy, oz));
    }
}
