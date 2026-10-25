//! `InterpolationManager`: steering a remote object along the positions the server sends.
//! Each quantum it finds the closest point of the queued path, seeks a point a quantum's
//! travel further along it, takes queued impulses (jumps) where the path reaches them,
//! blips (teleports) to the newest position when stuck, and rails the object's movement
//! toward the target so it does not overshoot.

use crate::fpu::{ld, st, F_EPSILON};
use crate::heading::Heading;
use crate::motion_interp::ELEMENT_AIR;
use crate::physics_obj::{PhysicsObj, CONTACT_TS, SYNC_TRANSIT_PS};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::Vector;

use super::{InterpolationPath, QueuedPosition};

/// `SetPositionSimple` flags for a blip: placement with sliding.
const BLIP_FLAGS: u32 = 0x1000 | crate::physics_obj_place::SLIDE_SPF;

#[derive(Debug, Clone, Default)]
pub(crate) struct InterpolationManager {
    pub active: bool,
    waiting_for_contact: bool,
    path: InterpolationPath,
    last_do_motion: Vector,
    progress_to_next_node: f32,
    time_waiting_for_contact: f32,
}

/// What a quantum of interpolation steers the motion interpreter with.
pub(crate) struct Steering {
    pub motion: Vector,
    pub heading: Heading,
}

/// `AddEvent`'s clamp of each do-motion component into `[-1, 1]`.
fn clamp_motion(v: Vector) -> Vector {
    let clamp = |c: f32| {
        let c = if 1.0 <= c { 1.0 } else { c };
        if c <= -1.0 {
            -1.0
        } else {
            c
        }
    };
    Vector::new(clamp(v.x), clamp(v.y), clamp(v.z))
}

impl InterpolationManager {
    /// The `!m_bActive` reset `AddEvent` and `AddImpulseEvent` start a new path with.
    fn activate(&mut self) {
        if !self.active {
            self.path.clear();
            self.time_waiting_for_contact = 0.0;
            self.progress_to_next_node = 0.0;
            self.active = true;
        }
    }

    /// `InterpolationManager::StopInterpolating`.
    pub(crate) fn stop(&mut self) {
        self.active = false;
        self.last_do_motion = Vector::ZERO;
    }

    /// `InterpolationManager::AddEvent`.
    pub(crate) fn add_event(
        &mut self,
        time_sent: f64,
        position: &Position,
        heading: Heading,
        motion: Vector,
        has_contact: bool,
    ) {
        let motion = clamp_motion(motion);
        self.activate();
        let mut pos = *position;
        pos.frame.set_heading(heading);
        self.path.add(&QueuedPosition {
            pos,
            time_sent,
            local_motion: motion,
            impulse: false,
            impulse_stamp: 0,
            impulse_is_jump: false,
            has_contact,
            velocity_at_impulse: Vector::ZERO,
        });
    }

    /// `InterpolationManager::AddImpulseEvent`: a position where the sender jumped or was
    /// pushed with `velocity`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_impulse_event(
        &mut self,
        time_sent: f64,
        position: &Position,
        heading: Heading,
        motion: Vector,
        velocity: Vector,
        jump: bool,
    ) {
        let motion = clamp_motion(motion);
        self.activate();
        let mut pos = *position;
        pos.frame.set_heading(heading);
        let stamp = self.path.next_impulse_stamp.wrapping_add(1);
        self.path.next_impulse_stamp = stamp;
        self.path.add(&QueuedPosition {
            pos,
            time_sent,
            local_motion: motion,
            impulse: true,
            impulse_stamp: stamp,
            impulse_is_jump: jump,
            has_contact: true,
            velocity_at_impulse: velocity,
        });
    }
}

impl PhysicsObj {
    /// `CPhysicsObj::InterpolateTo`: steer toward a server position within 15 m
    /// (`CMotionInterp::InterpolateTo` -> `AddEvent`), else jump there (`SyncPositionSimple`).
    pub(crate) fn interpolate_to(
        &mut self,
        time_sent: f64,
        position: &Position,
        motion: Vector,
        heading: Heading,
        has_contact: bool,
        terrain: &Terrain,
    ) {
        if position.distance_squared(&self.position) <= 225.0 {
            self.interp
                .get_or_insert_with(InterpolationManager::default)
                .add_event(time_sent, position, heading, motion, has_contact);
        } else {
            self.set_position_checked(*position, BLIP_FLAGS, terrain);
        }
    }

    /// `CPhysicsObj::InterpolateAndImpulse` -> `CMotionInterp::ImpulseAt` -> `AddImpulseEvent`.
    pub(crate) fn interpolate_and_impulse(
        &mut self,
        time_sent: f64,
        position: &Position,
        motion: Vector,
        heading: Heading,
        velocity: Vector,
        jump: bool,
    ) {
        self.interp
            .get_or_insert_with(InterpolationManager::default)
            .add_impulse_event(time_sent, position, heading, motion, velocity, jump);
    }

    /// `CMotionInterp::ConvertDoMotionToLocalDesired(motion, 0, out, 1)` at this body's
    /// motion values and scale.
    fn interp_desired_velocity(&self, motion: Vector) -> Vector {
        let values = self.motion.motion_values(self.desired_element());
        self.motion
            .local_desired_for(motion, &values, self.scale, 1.0)
    }

    /// `InterpolationManager::Update`: steer while a path is active, else lift the 2D distance
    /// cap.
    pub(crate) fn interp_update(&mut self, quantum: f32, terrain: &Terrain) -> Option<Steering> {
        // `CMotionInterp::UpdateMovement` only calls `Update` while the manager is active, so
        // the inactive branch that lifts the 2D distance cap never runs from here.
        let active = self.interp.as_ref().is_some_and(|interp| interp.active);
        if !active {
            return None;
        }
        Some(self.perform_interpolation(quantum, terrain))
    }

    /// `InterpolationManager::PerformInterpolation`.
    fn perform_interpolation(&mut self, quantum: f32, terrain: &Terrain) -> Steering {
        let Some(mut interp) = self.interp.take() else {
            return Steering {
                motion: Vector::ZERO,
                heading: self.position.frame.heading(),
            };
        };
        let current = self.position;
        let now = self.cur_time;
        let (mapped, along, predicting) = {
            let desired = |motion: Vector| self.interp_desired_velocity(motion);
            interp.path.closest_point(&current, now, &desired)
        };
        let forward = self.interp_desired_velocity(Vector::new(0.0, 1.0, 0.0));
        let q = ld(quantum);
        let sx = ld(st(ld(forward.x) * q));
        let sy = ld(st(ld(forward.y) * q));
        let sz_ext = ld(forward.z) * q;
        let sz = ld(st(sz_ext));
        let step = (sz_ext * sz + sy * sy) + sx * sx;
        let target_distance = st(step.sqrt() + ld(along));
        let target = interp.path.point_from_distance(target_distance);
        let off_path = st(current.distance(&mapped));
        let mut steering = Steering {
            motion: Vector::ZERO,
            heading: Heading::from_radians(0.0),
        };
        let arrived = self.seek(
            &mut interp,
            &current,
            along,
            off_path,
            &target,
            &mut steering,
        );
        if self.has_contact() {
            interp.waiting_for_contact = false;
        }
        self.do_impulse_logic(
            &mut interp,
            quantum,
            &current,
            &target,
            along,
            &mut steering,
            terrain,
        );
        if interp.waiting_for_contact && self.desired_element() == ELEMENT_AIR {
            steering.motion = Vector::ZERO;
        } else if !arrived && !self.check_progress(&mut interp, quantum, &current, target_distance)
        {
            self.blip(&mut interp, &mut steering, terrain);
        }
        interp.last_do_motion = steering.motion;
        let contact = interp.path.has_contact(along);
        self.do_constraint(&interp, &current, &target, predicting, off_path, contact);
        let v = self.velocity;
        let (vx, vy, vz) = (ld(v.x), ld(v.y), ld(v.z));
        if arrived && (vz * vz + vy * vy) + vx * vx < ld(F_EPSILON) {
            interp.active = false;
        }
        self.interp = Some(interp);
        steering
    }

    /// `InterpolationManager::Seek`: aim at `target` (the do-motion toward it in local axes,
    /// the heading toward it). Near the end of a short, still path the remainder is thrown
    /// away and the body stops facing the last heading (returns true); otherwise the speed
    /// follows the path's motion when close, or catches up at 1.2 times. A target straight up
    /// or down for a body that cannot move vertically walks forward.
    fn seek(
        &self,
        interp: &mut InterpolationManager,
        current: &Position,
        along: f32,
        off_path: f32,
        target: &Position,
        steering: &mut Steering,
    ) -> bool {
        let mut arrived = false;
        let off = current.offset_to(target);
        steering.heading = current.heading_to(target);
        let m = current.frame.m.map(ld);
        let (x, y, z) = (ld(off.x), ld(off.y), ld(off.z));
        let lx = (z * m[2] + y * m[1]) + x * m[0];
        let ly = ld(st((z * m[5] + y * m[4]) + x * m[3]));
        let lz = (z * m[8] + y * m[7]) + x * m[6];
        let length = ((ly * ly + lz * lz) + lx * lx).sqrt();
        steering.motion = if length < ld(F_EPSILON) {
            Vector::ZERO
        } else {
            let inverse = 1.0 / length;
            Vector::new(st(lx * inverse), st(ly * inverse), st(lz * inverse))
        };
        let Some(last) = interp.path.last().copied() else {
            return false;
        };
        let to_current = last.pos.offset_to(current);
        let (cx, cy, cz) = (ld(to_current.x), ld(to_current.y), ld(to_current.z));
        let remaining2 = ld(st((cz * cz + cy * cy) + cx * cx));
        let lm = last.local_motion;
        let (mx, my, mz) = (ld(lm.x), ld(lm.y), ld(lm.z));
        let still = (mx * mx + my * my) + mz * mz < ld(0.01_f32);
        if ld(interp.path.total_distance()) < 4.0 && still {
            if remaining2 < ld(0.0025_f32) {
                let total = interp.path.total_distance();
                interp.path.throw_away(total);
                arrived = true;
            }
            if remaining2 < 0.5 {
                steering.heading = last.pos.frame.heading();
            }
            if arrived {
                steering.motion = Vector::ZERO;
                return true;
            }
        } else {
            let interpolated = self.interpolated_distance(interp);
            let close = ld(interpolated) < 3.0 && ld(off_path) < 1.0;
            match interp.path.interpolated_values(along).filter(|_| close) {
                Some((velocity, heading)) => {
                    steering.heading = heading;
                    let (vx, vy, vz) = (ld(velocity.x), ld(velocity.y), ld(velocity.z));
                    let speed = st(((vz * vz + vy * vy) + vx * vx).sqrt());
                    let scale = if 1.0 <= speed { 1.0 } else { speed };
                    let scale = ld(if 0.5 < scale { scale } else { 0.5 });
                    let m = steering.motion;
                    steering.motion = Vector::new(
                        st(scale * ld(m.x)),
                        st(scale * ld(m.y)),
                        st(scale * ld(m.z)),
                    );
                }
                None => {
                    let k = ld(1.2_f32);
                    let m = steering.motion;
                    steering.motion =
                        Vector::new(st(k * ld(m.x)), st(k * ld(m.y)), st(k * ld(m.z)));
                }
            }
        }
        let vertical_only =
            0.1 > ld(off.x).abs() && 0.1 > ld(off.y).abs() && ld(F_EPSILON) <= ld(off.z).abs();
        if vertical_only
            && self
                .motion
                .motion_values(self.desired_element())
                .max_vertical_velocity
                == 0.0
        {
            steering.motion = Vector::new(0.0, 1.0, 0.0);
        }
        arrived
    }

    /// `InterpolationPath::GetInterpolatedDistance`: the path length plus how far the last
    /// motion carries in the time since it was sent.
    fn interpolated_distance(&self, interp: &InterpolationManager) -> f32 {
        let Some(last) = interp.path.last() else {
            return 0.0;
        };
        let dt = ld(st((self.cur_time - last.time_sent).abs()));
        let v = self.interp_desired_velocity(last.local_motion);
        let (x, y, z) = (ld(v.x), ld(v.y), ld(v.z));
        let speed = ld(st(((z * z + y * y) + x * x).sqrt()));
        st(speed * dt + ld(interp.path.total_distance()))
    }

    /// `InterpolationManager::CheckProgress`: false (blip) after two seconds without passing a
    /// node, or when the newest position is more than 15 m away.
    fn check_progress(
        &self,
        interp: &mut InterpolationManager,
        quantum: f32,
        current: &Position,
        target_distance: f32,
    ) -> bool {
        if interp.path.throw_away(target_distance) == 0 {
            interp.progress_to_next_node = st(ld(quantum) + ld(interp.progress_to_next_node));
            if !(ld(interp.progress_to_next_node) < 2.0) {
                return false;
            }
        } else {
            interp.progress_to_next_node = 0.0;
        }
        interp
            .path
            .last()
            .is_some_and(|last| current.distance_squared(&last.pos) <= 225.0)
    }

    /// `InterpolationManager::Blip`: give up interpolating and jump to the newest position
    /// (`SyncPositionSimple` with sliding), moving at its motion.
    fn blip(
        &mut self,
        interp: &mut InterpolationManager,
        steering: &mut Steering,
        terrain: &Terrain,
    ) {
        if !interp.active {
            return;
        }
        interp.active = false;
        let Some(last) = interp.path.last().copied() else {
            return;
        };
        self.set_position_checked(last.pos, BLIP_FLAGS, terrain);
        steering.motion = last.local_motion;
        steering.heading = last.pos.frame.heading();
        let velocity = self.interp_desired_velocity(last.local_motion);
        self.set_velocity_checked(velocity);
    }

    /// `InterpolationManager::DoImpulseLogic`: on reaching a queued impulse, teleport to it
    /// and jump (the jump velocity clamped to the body's limits) or apply the impulse; three
    /// seconds without managing it blips.
    #[allow(clippy::too_many_arguments)]
    fn do_impulse_logic(
        &mut self,
        interp: &mut InterpolationManager,
        quantum: f32,
        current: &Position,
        target: &Position,
        along: f32,
        steering: &mut Steering,
        terrain: &Terrain,
    ) {
        if !(current.distance(target) < 0.1) {
            return;
        }
        let Some((pos, velocity, jump)) = interp.path.should_impulse_here(along) else {
            return;
        };
        self.set_position_checked(pos, BLIP_FLAGS, terrain);
        let took = if jump {
            let local = pos.frame.global_to_local_vec(velocity);
            let values = self.motion.motion_values(self.desired_element());
            let clamped = self.motion.clamp_velocity(local, &values, self.scale);
            let global = pos.local_to_global_vec(clamped);
            self.set_velocity_checked(global);
            self.jump_logic()
        } else {
            self.apply_impulse(velocity);
            true
        };
        if took {
            interp.path.last_impulse_stamp = interp.path.last_impulse_stamp.wrapping_add(1);
            interp.time_waiting_for_contact = 0.0;
            interp.waiting_for_contact = true;
        } else {
            interp.time_waiting_for_contact = st(ld(quantum) + ld(interp.time_waiting_for_contact));
            if ld(interp.time_waiting_for_contact) > 3.0 {
                self.blip(interp, steering, terrain);
                interp.time_waiting_for_contact = 0.0;
                interp.path.last_impulse_stamp = interp.path.last_impulse_stamp.wrapping_add(1);
            }
        }
        steering.motion = Vector::ZERO;
        steering.heading = pos.frame.heading();
    }

    /// `InterpolationManager::DoConstraint`: enable the constraint manager only when the path's
    /// last node lies at path distance 0; cap the 2D travel at the distance to the target
    /// and rail the velocity toward it; synchronise the transition (taking the path's contact)
    /// once on the path short of its end.
    fn do_constraint(
        &mut self,
        interp: &InterpolationManager,
        current: &Position,
        target: &Position,
        predicting: bool,
        off_path: f32,
        contact: bool,
    ) {
        // The constraint holds only once the path's last node is at distance 0.
        if let (Some(constraint), Some((_, tail))) =
            (self.constraint.as_mut(), interp.path.queue.back())
        {
            constraint.enabled = ld(*tail) <= ld(F_EPSILON);
        }
        let off = current.offset_to(target);
        let (x, y) = (ld(off.x), ld(off.y));
        self.distance_cap_2d = st((y * y + x * x).sqrt());
        let length = ((y * y + x * x) + 0.0).sqrt();
        let rail = if length < ld(F_EPSILON) {
            Vector::ZERO
        } else {
            let inverse = 1.0 / length;
            Vector::new(st(x * inverse), st(y * inverse), st(inverse * 0.0))
        };
        self.velocity_rail = Some(rail);
        if ld(off_path) < ld(F_EPSILON) && !predicting && !interp.waiting_for_contact {
            self.state |= SYNC_TRANSIT_PS;
            if contact {
                self.transient_state |= CONTACT_TS;
            } else {
                self.transient_state &= !CONTACT_TS;
            }
        } else {
            self.state &= !SYNC_TRANSIT_PS;
        }
    }

    /// `CPhysicsObj::set_velocity`: replaced only when some axis differs by `F_EPSILON`, then
    /// held to the terminal velocity (`CullVelocity`).
    pub(crate) fn set_velocity_checked(&mut self, v: Vector) {
        let current = self.velocity;
        let moved = |a: f32, b: f32| (ld(a) - ld(b)).abs() >= ld(F_EPSILON);
        if moved(v.x, current.x) || moved(v.y, current.y) || moved(v.z, current.z) {
            self.velocity = v;
            self.cull_velocity();
        }
    }

    /// `CPhysicsObj::ApplyImpulse`: leave the ground into the air at `velocity`.
    pub(crate) fn apply_impulse(&mut self, velocity: Vector) {
        self.transient_state &= !CONTACT_TS;
        self.do_element(ELEMENT_AIR);
        self.set_velocity_checked(velocity);
    }
}
