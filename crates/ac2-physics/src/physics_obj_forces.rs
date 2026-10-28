//! `CPhysicsObj`'s force terms: `CalcAcceleration`, `CalcFriction`, `CullVelocity`,
//! `UpdateRotation` and `CalcExternalForces`.

use super::{PhysicsObj, CONTACT_TS, GRAVITY_PS, MISC_ACTIVE_TS, ON_WALKABLE_TS, STATIC_PS};
use crate::fpu::{epsilon_squared, ld, st, F_EPSILON};
use crate::frame::Frame;
use crate::heading::Heading;
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_SEA_FLOOR, ELEMENT_WATER};
use crate::terrain::Terrain;
use crate::vector::Vector;

/// `1/180` and `pi` as the `float` constants `UpdateRotation` multiplies degrees by.
const INVERSE_180: f32 = 0.005_555_555_7;
const PI: f32 = std::f32::consts::PI;
const TWO_PI: f32 = 6.283_185_5;

impl PhysicsObj {
    /// `CPhysicsObj::CalcAcceleration`: gravity off walkable ground, water lift and hover lift by
    /// element, plus the motion interpreter's local acceleration turned into world axes.
    pub(super) fn calc_acceleration(&mut self, terrain: &Terrain) {
        self.acceleration = Vector::ZERO;
        let values = self
            .motion
            .desired_motion_values(self.desired_element(), self.motion.mode());
        let affected = self.state & GRAVITY_PS != 0;
        if affected && values.gravity && (!self.has_contact() || !self.on_walkable()) {
            let g = self.local_gravity;
            self.acceleration = Vector::new(
                st(ld(self.acceleration.x) + ld(g.x)),
                st(ld(g.y) + ld(self.acceleration.y)),
                st(ld(g.z) + ld(self.acceleration.z)),
            );
        }
        let desired = self.desired_element();
        let lift = |acceleration: &mut Vector, amount: f64| {
            acceleration.z = st(amount + ld(acceleration.z));
        };
        if (desired == ELEMENT_WATER || desired == ELEMENT_SEA_FLOOR)
            && !self.has_contact()
            && self.has_water
        {
            let depth = ld(self.depth);
            if -1.0 < depth && depth < 1.0 {
                lift(&mut self.acceleration, ld(values.lift) * depth);
            } else if depth > 1.0 {
                lift(&mut self.acceleration, ld(values.lift));
            } else {
                let g = self.local_gravity;
                let a = &mut self.acceleration;
                a.x = st(ld(g.x) + ld(a.x));
                a.y = st(ld(g.y) + ld(a.y));
                lift(a, ld(g.z));
            }
        } else if desired == ELEMENT_AIR {
            if values.gravity {
                lift(&mut self.acceleration, ld(values.lift));
            } else {
                let mut height = self.height_above_walkable(terrain);
                if self.has_water {
                    let above = -ld(self.depth);
                    if above < ld(height) {
                        height = st(above);
                    }
                }
                let hover = ld(self.motion.hover_height(self.desired_element()));
                if (ld(height) - hover).abs() >= 0.100_000_001_490_116_12 {
                    lift(&mut self.acceleration, ld(values.lift));
                }
            }
        }
        let local = self.motion.acceleration();
        let m = self.position.frame.m.map(ld);
        let (x, y, z) = (ld(local.x), ld(local.y), ld(local.z));
        let gx = (y * m[3] + z * m[6]) + x * m[0];
        let gy = (y * m[4] + z * m[7]) + x * m[1];
        let gz = ld(st((y * m[5] + z * m[8]) + x * m[2]));
        let a = &mut self.acceleration;
        a.x = st(gx + ld(a.x));
        a.y = st(gy + ld(a.y));
        a.z = st(gz + ld(a.z));
    }

    /// `CPhysicsObj::CullVelocity`: speeds above the terminal velocity are scaled back to it.
    pub(crate) fn cull_velocity(&mut self) {
        let terminal = self
            .motion
            .motion_values(self.desired_element())
            .terminal_velocity;
        let v = self.velocity;
        let speed2 = st((ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y)) + ld(v.z) * ld(v.z));
        let limit = ld(terminal) * ld(terminal);
        if ld(speed2) > limit {
            let unit = v.d3dx_normalize();
            self.velocity = Vector::new(
                st(ld(terminal) * ld(unit.x)),
                st(ld(terminal) * ld(unit.y)),
                st(ld(terminal) * ld(unit.z)),
            );
        }
    }

    /// `CPhysicsObj::CalcFriction`: drag `(1 - drag)^quantum` on the velocity across the
    /// direction the body accelerates in (or all of it when not accelerating), then on walkable
    /// ground the part into the contact plane is dropped when small.
    pub(super) fn calc_friction(&mut self, quantum: f32, _velocity_mag2: f32) {
        let values = self
            .motion
            .desired_motion_values(self.desired_element(), self.motion.mode());
        if ld(values.drag) == 0.0 {
            return;
        }
        let local = self.motion.acceleration();
        let accel2 =
            (ld(local.x) * ld(local.x) + ld(local.y) * ld(local.y)) + ld(local.z) * ld(local.z);
        let damping = (1.0 - ld(values.drag)).powf(ld(quantum));
        if epsilon_squared() >= accel2 {
            let v = self.velocity;
            self.velocity = Vector::new(
                st(damping * ld(v.x)),
                st(damping * ld(v.y)),
                st(damping * ld(v.z)),
            );
        } else {
            let direction = self
                .position
                .frame
                .local_to_global_vec(local)
                .d3dx_normalize();
            let v = self.velocity;
            let along =
                (ld(direction.z) * ld(v.z) + ld(direction.y) * ld(v.y)) + ld(direction.x) * ld(v.x);
            let forward = if along > 0.0 {
                Vector::new(
                    st(ld(direction.x) * along),
                    st(ld(direction.y) * along),
                    st(ld(direction.z) * along),
                )
            } else {
                Vector::ZERO
            };
            let across = Vector::new(
                st(ld(v.x) - ld(forward.x)),
                st(ld(v.y) - ld(forward.y)),
                st(ld(v.z) - ld(forward.z)),
            );
            let ax = ld(st(ld(across.x) * damping));
            let ay = ld(st(ld(across.y) * damping));
            self.velocity = Vector::new(
                st(ax + ld(forward.x)),
                st(ay + ld(forward.y)),
                st(damping * ld(across.z) + ld(forward.z)),
            );
        }
        if self.transient_state & ON_WALKABLE_TS != 0 {
            let n = self.contact_plane.n;
            let v = self.velocity;
            let into = (ld(v.z) * ld(n.z) + ld(v.y) * ld(n.y)) + ld(v.x) * ld(n.x);
            if into.abs() < ld(0.25_f32) {
                let pz = ld(st(into * ld(n.z)));
                self.velocity = Vector::new(
                    st(ld(v.x) - into * ld(n.x)),
                    st(ld(v.y) - into * ld(n.y)),
                    st(ld(v.z) - pz),
                );
            }
        }
    }

    /// `CPhysicsObj::UpdateRotation`: turn the frame offset by `omega q + omega_acl q^2 / 2`
    /// (degrees to radians, with the retail float stores), stop a clamped heading at its target
    /// and integrate the yaw rate toward the omega clamp.
    pub(super) fn update_rotation(&mut self, quantum: f32, offset: &mut Frame) {
        let q = ld(quantum);
        let (inv180, pi) = (ld(INVERSE_180), ld(PI));
        let wa = self.omega_acceleration;
        let half_x = ld(wa.x) * 0.5;
        let half_y = ld(wa.y) * 0.5;
        let half_z = ld(st(ld(wa.z) * 0.5));
        let ax = ld(st(ld(st(half_x * inv180)) * pi)) * q * q;
        let ay = ld(st(ld(st(half_y * inv180 * pi)) * q)) * q;
        let az = ld(st(ld(st(half_z * inv180 * pi * q)) * q));
        let w = self.omega;
        let wx = ld(st(ld(st(ld(w.x) * inv180 * pi)) * q));
        let wy = ld(st(ld(w.y) * inv180 * pi * q));
        let wz = ld(st(ld(st(ld(w.z) * inv180)) * pi * q));
        let mut rotation = Vector::new(st(wx + ax), st(wy + ay), st(wz + az));
        if let Some(target) = self.clamp_heading {
            let current = ld(st(self.position.frame.heading().radians()));
            let mut goal = target.radians();
            let past = if ld(rotation.z) < 0.0 {
                if goal < current {
                    goal += ld(TWO_PI);
                }
                !(current - ld(rotation.z) <= goal)
            } else {
                if goal > current {
                    goal -= ld(TWO_PI);
                }
                current - ld(rotation.z) < goal
            };
            if past {
                // `CPhysicsObj::set_heading(clamp, 0)`: face the clamp heading (byte steps).
                let heading = Heading::from_u8(target.to_u8());
                self.position.frame.set_heading(heading);
                self.omega.z = 0.0;
                self.omega_acceleration.z = 0.0;
                rotation.z = 0.0;
            }
        }
        offset.grotate(rotation);
        let wa = self.omega_acceleration;
        let tz = ld(st(q * ld(wa.z)));
        let x = st(q * ld(wa.x) + ld(self.omega.x));
        let y = st(q * ld(wa.y) + ld(self.omega.y));
        let mut z = tz + ld(self.omega.z);
        if let Some(clamp) = self.clamp_omega {
            let (before, clamp) = (ld(self.omega.z), ld(clamp));
            if before < clamp && z > clamp {
                z = clamp;
            }
            if before > clamp && z < clamp {
                z = clamp;
            }
        }
        self.omega = Vector::new(x, y, st(z));
    }

    /// `CPhysicsObj::CalcExternalForces`: the pushers' velocity and acceleration fields for the
    /// desired element (`CCollisionManager::UpdatePusherValues`), the residual field velocity's
    /// decay (x0.8 in contact or in water), and the offset the fields move the body this
    /// quantum (`m_vExternalField`). A strong upward push takes the body off the ground and out
    /// of gravity; otherwise gravity applies again, and an upward residual gains gravity.
    /// The intermediate roundings follow the x87 spills of the retail code.
    pub(super) fn calc_external_forces(&mut self, quantum: f32) {
        let sum = self
            .collision_manager
            .pushers
            .update_values(self.desired_element());
        self.walk_on_air = sum.walk_on_air;
        let vf = sum.velocity;
        let af = sum.acceleration;
        self.external_field = Vector::ZERO;
        if self.has_contact() || self.desired_element() == ELEMENT_WATER {
            let f = self.acceleration_field_vel;
            let decay = ld(0.8_f32);
            self.acceleration_field_vel = Vector::new(
                st(ld(f.x) * decay),
                st(ld(f.y) * decay),
                st(ld(f.z) * decay),
            );
        }
        let eps = ld(F_EPSILON);
        let mag2 = |v: Vector| ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y) + ld(v.z) * ld(v.z);
        if mag2(self.external_field) < eps {
            self.external_field = Vector::ZERO;
        }
        if mag2(self.acceleration_field_vel) < eps {
            self.acceleration_field_vel = Vector::ZERO;
        }
        if ld(vf.z) > eps || ld(af.z) > eps {
            self.transient_state &= !CONTACT_TS;
            if self.state & STATIC_PS == 0 {
                self.transient_state |= MISC_ACTIVE_TS;
            }
            self.state &= !GRAVITY_PS;
        } else {
            self.state |= GRAVITY_PS;
            if ld(self.acceleration_field_vel.z) > 0.0 {
                let g = self.local_gravity;
                let f = self.acceleration_field_vel;
                self.acceleration_field_vel = Vector::new(
                    st(ld(g.x) + ld(f.x)),
                    st(ld(g.y) + ld(f.y)),
                    st(ld(g.z) + ld(f.z)),
                );
            }
        }
        let q = ld(quantum);
        let half = ld(0.5_f32);
        let f = self.acceleration_field_vel;
        // `a * 0.5 * q * q` per axis: x spills `a * 0.5 * q` to a float before the last product.
        let term_x = st(ld(st(ld(af.x) * half * q)) * q);
        let term_y = st(ld(af.y) * half * q * q);
        let term_z = ld(af.z) * half * q * q;
        let part_x = st(q * ld(vf.x) + q * ld(f.x));
        let part_y = st(ld(st(q * ld(vf.y))) + q * ld(f.y));
        let part_z = st(ld(st(q * ld(vf.z))) + ld(st(q * ld(f.z))));
        self.external_field = Vector::new(
            st(ld(part_x) + ld(term_x)),
            st(ld(part_y) + ld(term_y)),
            st(ld(part_z) + term_z),
        );
        self.acceleration_field_vel = Vector::new(
            st(ld(af.x) + ld(f.x)),
            st(ld(af.y) + ld(f.y)),
            st(ld(af.z) + ld(f.z)),
        );
    }
}
