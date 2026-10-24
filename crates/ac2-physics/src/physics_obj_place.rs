//! `CPhysicsObj::SetPosition`: putting a body at a position through a placement transition
//! (`SetPositionInternal(SetPositionStruct*)`, `AdjustPosition`, `CheckPositionInternal`,
//! `ForceIntoCell`).

use crate::cells::{CellSource, ObjCell};
use crate::fpu::{ld, st};
use crate::landdefs::{adjust_to_outside, is_outside};
use crate::physics_obj::PhysicsObj;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::transition::{Transition, TransitionObject};
use crate::vector::Vector;

/// `SetPositionFlag`: slide to a free spot nearby when the position itself is blocked.
pub const SLIDE_SPF: u32 = 0x10;
/// `SetPositionFlag`: put the body in the cell without a placement check.
pub const FORCE_SPF: u32 = 0x40;

/// `SetPositionError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPositionError {
    Ok,
    GeneralFailure,
    NoValidPosition,
    NoCell,
    Collided,
}

/// `CEnvCell::find_visible_child_cell`: the interior cell, or one of its stab list's resident
/// cells, holding `point`.
fn find_visible_child_cell(cells: &dyn CellSource, cell_id: u32, point: Vector) -> Option<u32> {
    let cell = cells.env_cell(cell_id)?;
    if cell.point_in_cell(point) {
        return Some(cell_id);
    }
    cell.stab_list.iter().copied().find(|&id| {
        cells
            .env_cell(id)
            .is_some_and(|other| other.point_in_cell(point))
    })
}

/// `CPhysicsObj::AdjustPosition`: the resident cell a position belongs in -- the land cell
/// under an outdoor point, the interior cell (or a visible neighbour) holding the first sphere,
/// or the land cell outside an interior seen from outside. The position is rebased onto it.
fn adjust_position(cells: &dyn CellSource, p: &mut Position, sphere_center: Vector) -> Option<u32> {
    if !is_outside(p.cell_id) {
        let cell = cells.env_cell(p.cell_id)?;
        let point = p.local_to_global(p, sphere_center);
        if let Some(found) = find_visible_child_cell(cells, p.cell_id, point) {
            p.cell_id = found;
            return Some(found);
        }
        if !cell.seen_outside {
            return None;
        }
    }
    if !adjust_to_outside(&mut p.cell_id, &mut p.frame.origin) {
        return None;
    }
    ObjCell::get(cells, p.cell_id).map(|_| p.cell_id)
}

impl PhysicsObj {
    /// `CPhysicsObj::SetPosition` (no scatter): a placement transition with the body's spheres
    /// from no begin position. `FORCE_SPF` (or an immobile body) skips the check; without
    /// `SLIDE_SPF` the placement may not move the body sideways. On success the body takes the
    /// placed position and its contact.
    pub(crate) fn set_position_checked(
        &mut self,
        position: Position,
        flags: u32,
        terrain: &Terrain,
    ) -> SetPositionError {
        let spheres = self.shape.spheres.clone();
        let object = TransitionObject {
            state: 0,
            spheres: &spheres,
            scale: st(ld(self.shape.scale) * ld(self.scale.z)),
            step_up_height: self.shape.step_up_height,
            step_down_height: self.step_down_height(),
            affected_by_gravity: self.state & crate::physics_obj::GRAVITY_PS != 0,
            contact: None,
            sliding_normal: None,
            frames_stationary_fall: 0,
            containment_radius: self.containment_radius(),
        };
        let mut p = position;
        let probe = Transition::new(&object, None, None, p);
        let center = probe.sphere_path.local_sphere[0].center;
        let Some(cell) = adjust_position(terrain, &mut p, center) else {
            return SetPositionError::NoCell;
        };
        if flags & FORCE_SPF != 0 || !self.is_mobile() {
            // `ForceIntoCell`.
            self.position = p;
            self.prev_position = p;
            self.anim_position = p;
            return SetPositionError::Ok;
        }
        let mut transition = Transition::new(&object, Some(cell), None, p);
        if flags & SLIDE_SPF == 0 {
            transition.sphere_path.placement_allows_sliding = false;
        }
        if !self.check_position(&mut transition, &mut p, cell, flags, terrain) {
            self.handle_all_collisions(&transition.collision_info, false);
            return if transition.collision_info.collide_objects.is_empty() {
                SetPositionError::NoValidPosition
            } else {
                SetPositionError::Collided
            };
        }
        if transition.sphere_path.curr_cell.is_none() {
            return SetPositionError::NoCell;
        }
        self.set_position_internal(&transition);
        self.prev_position = self.position;
        self.anim_position = self.position;
        SetPositionError::Ok
    }

    /// `CPhysicsObj::CheckPositionInternal`: a placement there; without sliding the result must
    /// be the requested spot (less than 0.05 m further in x and y, same cell), whose origin then
    /// takes the placed one.
    fn check_position(
        &self,
        transition: &mut Transition,
        p: &mut Position,
        cell: u32,
        flags: u32,
        terrain: &Terrain,
    ) -> bool {
        if !transition.find_valid_position(terrain) {
            return false;
        }
        if flags & SLIDE_SPF != 0 {
            return true;
        }
        let curr = transition.sphere_path.curr_pos;
        let near = ld(curr.frame.origin.x) - ld(p.frame.origin.x) < ld(0.05_f32)
            && ld(curr.frame.origin.y) - ld(p.frame.origin.y) < ld(0.05_f32);
        if !near || curr.cell_id != cell {
            return false;
        }
        p.frame.origin = curr.frame.origin;
        true
    }
}
