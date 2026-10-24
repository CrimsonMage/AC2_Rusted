//! `CTransition` placement: putting a body down at a position (`find_placement_position`) --
//! out of any interior geometry first (`placement_insert`, initial placement), then clear of
//! everything (`find_placement_pos`, which may search outwards in rings), then stepped down
//! onto the ground.

use crate::cells::{find_collisions, CellSource, ObjCell};
use crate::fpu::{ld, st, F_EPSILON, LANDING_Z};
use crate::frame::Frame;
use crate::heading::Heading;
use crate::transition::sphere_path::InsertType;
use crate::transition::{Transition, TransitionState};
use crate::vector::Vector;

impl Transition {
    /// `CTransition::placement_insert`: up to three passes of the check cell and the cells it
    /// touches; adjustments go round again.
    pub(crate) fn placement_insert(&mut self, cells: &dyn CellSource) -> TransitionState {
        if self.sphere_path.check_cell.is_none() {
            return TransitionState::Collided;
        }
        let mut ts = TransitionState::Collided;
        for _ in 0..3 {
            self.physics_timestamp += 1;
            let cell = self
                .sphere_path
                .check_cell
                .and_then(|id| ObjCell::get(cells, id));
            ts = match cell {
                Some(cell) => find_collisions(self, cell, cells),
                None => TransitionState::Collided,
            };
            match ts {
                TransitionState::Ok => {
                    ts = self.check_other_cells(self.sphere_path.check_cell, cells);
                    if ts == TransitionState::Ok || ts == TransitionState::Collided {
                        return ts;
                    }
                }
                TransitionState::Collided => return ts,
                TransitionState::Slid => self.collision_info.clear_contact_plane(),
                TransitionState::Adjusted | TransitionState::Invalid => {}
            }
        }
        ts
    }

    /// `CTransition::validate_placement`: accept the check position, or (once, with `adjust`)
    /// insert again after an adjustment or slide.
    fn validate_placement(
        &mut self,
        ts: TransitionState,
        adjust: bool,
        cells: &dyn CellSource,
    ) -> TransitionState {
        if self.sphere_path.check_cell.is_none() {
            return TransitionState::Collided;
        }
        match ts {
            TransitionState::Ok => {
                self.accept_check_position();
                ts
            }
            TransitionState::Adjusted | TransitionState::Slid if adjust => {
                let ts = self.placement_insert(cells);
                self.validate_placement(ts, false, cells)
            }
            _ => ts,
        }
    }

    /// `CTransition::validate_placement_transition`: accept the check position; a collided,
    /// adjusted or slid insert that may slide starts its collision record over.
    pub(crate) fn validate_placement_transition(&mut self, ts: TransitionState) -> TransitionState {
        if self.sphere_path.check_cell.is_none() {
            return TransitionState::Collided;
        }
        if ts == TransitionState::Ok {
            self.accept_check_position();
        } else if ts != TransitionState::Invalid && self.sphere_path.placement_allows_sliding {
            self.collision_info.init();
        }
        ts
    }

    /// The check position becomes the current one.
    fn accept_check_position(&mut self) {
        let path = &mut self.sphere_path;
        path.curr_pos = path.check_pos;
        path.curr_cell = path.check_cell;
        path.cache_global_curr_center();
    }

    /// A placement insert at the check position as `find_placement_pos` runs each try.
    fn try_placement(&mut self, cells: &dyn CellSource) -> bool {
        self.collision_info.sliding_normal_valid = false;
        self.collision_info.clear_contact_plane();
        let ts = self.transitional_insert(3, cells);
        self.validate_placement_transition(ts) == TransitionState::Ok
    }

    /// `CTransition::find_placement_pos`: the position itself, 0.1 m higher, then (when sliding
    /// is allowed) rings of growing radius around it, each tried at evenly spaced headings.
    fn find_placement_pos(&mut self, cells: &dyn CellSource) -> bool {
        let (curr, cell) = (self.sphere_path.curr_pos, self.sphere_path.curr_cell);
        self.sphere_path.check_pos = curr;
        self.sphere_path.check_cell = cell;
        self.sphere_path.cell_array_valid = false;
        self.sphere_path.cache_global_sphere(None);
        if self.try_placement(cells) {
            return true;
        }
        if !self.sphere_path.placement_allows_sliding {
            return false;
        }
        self.sphere_path.set_check_pos(curr, cell);
        self.sphere_path
            .add_offset_to_check_pos(Vector::new(0.0, 0.0, 0.1));
        if self.try_placement(cells) {
            return true;
        }
        let mut radius = self.sphere_path.local_sphere[0].radius;
        let (small, max) = if radius < 0.125 {
            (true, 2.0_f32)
        } else {
            if radius < 0.5 {
                radius = 0.5;
            }
            (false, 4.0_f32)
        };
        let mut steps = ld(4.0_f32) / (ld(0.9_f32) * ld(radius));
        if small {
            steps *= 0.5;
        }
        if !(steps > 1.0) && !steps.is_nan() {
            return false;
        }
        let rings = steps.ceil();
        let step = st(ld(max) / rings);
        let rings = rings as i64 as u32;
        let angle_step = st(ld(step) * std::f64::consts::PI / ld(radius));
        let (mut ring_radius, mut angle) = (0.0_f32, 0.0_f32);
        for _ in 0..rings {
            ring_radius = st(ld(ring_radius) + ld(step));
            let sum = ld(angle_step) + ld(angle);
            angle = st(sum);
            let count = (sum.ceil() as i64 as u32).wrapping_mul(2);
            let per_step = st(360.0 / f64::from(count));
            let mut frame = Frame::IDENTITY;
            for index in 0..count {
                self.sphere_path.check_pos = curr;
                self.sphere_path.check_cell = cell;
                self.sphere_path.cell_array_valid = false;
                self.sphere_path.cache_global_sphere(None);
                frame.set_heading(Heading::from_degrees(st(f64::from(index) * ld(per_step))));
                let v = frame.vector_heading();
                let r = ld(ring_radius);
                let offset = Vector::new(st(r * ld(v.x)), st(r * ld(v.y)), st(r * ld(v.z)));
                let adjusted = self.adjust_offset(offset);
                self.sphere_path.global_offset = adjusted;
                let (x, y, z) = (ld(adjusted.x), ld(adjusted.y), ld(adjusted.z));
                if (x * x + y * y) + z * z < ld(F_EPSILON) * ld(F_EPSILON) {
                    continue;
                }
                let path = &mut self.sphere_path;
                path.cell_array_valid = false;
                let origin = &mut path.check_pos.frame.origin;
                origin.x = st(ld(origin.x) + x);
                origin.y = st(ld(origin.y) + y);
                origin.z = st(ld(origin.z) + z);
                path.cache_global_sphere(Some(adjusted));
                if self.try_placement(cells) {
                    return true;
                }
            }
        }
        false
    }

    /// `CTransition::find_placement_position`: out of interiors (initial placement), clear of
    /// everything, then down onto the ground when the object steps down.
    pub(crate) fn find_placement_position(&mut self, cells: &dyn CellSource) -> bool {
        let (curr, cell) = (self.sphere_path.curr_pos, self.sphere_path.curr_cell);
        self.sphere_path.check_pos = curr;
        self.sphere_path.check_cell = cell;
        self.sphere_path.cell_array_valid = false;
        self.sphere_path.cache_global_sphere(None);
        self.sphere_path.insert_type = InsertType::InitialPlacement;
        let ts = self.placement_insert(cells);
        if self.validate_placement(ts, true, cells) != TransitionState::Ok {
            return false;
        }
        self.sphere_path.insert_type = InsertType::Placement;
        if !self.find_placement_pos(cells) {
            return false;
        }
        if self.object_info.step_down {
            let mut height = self.object_info.step_down_height;
            self.sphere_path.walkable_allowance = LANDING_Z;
            self.sphere_path.save_check_pos();
            self.sphere_path.backup = self.sphere_path.insert_type;
            self.sphere_path.insert_type = InsertType::Transition;
            let radius = ld(self.sphere_path.global_sphere[0].radius);
            if self.sphere_path.num_sphere < 2 && radius + radius < ld(height) {
                height = st(radius * 0.5);
            }
            let radius = ld(self.sphere_path.global_sphere[0].radius);
            let stepped = if ld(height) <= radius + radius {
                self.step_down(height, LANDING_Z, cells)
            } else {
                height = st(ld(height) * 0.5);
                self.step_down(height, LANDING_Z, cells) || self.step_down(height, LANDING_Z, cells)
            };
            if !stepped {
                self.sphere_path.restore_check_pos();
                self.collision_info.clear_contact_plane();
            }
            self.sphere_path.insert_type = self.sphere_path.backup;
            self.sphere_path.walkable = None;
        }
        self.validate_placement(TransitionState::Ok, true, cells) == TransitionState::Ok
    }
}
