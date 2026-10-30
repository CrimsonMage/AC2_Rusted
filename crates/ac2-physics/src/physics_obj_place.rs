//! `CPhysicsObj::SetPosition`: putting a body at a position through a placement transition
//! (`SetPositionInternal(SetPositionStruct*)`, `SetScatterPositionInternal`, `AdjustPosition`,
//! `CheckPositionInternal`, `ForceIntoCell`, `SetPlacementPosition`), and the position helpers
//! around it (`StorePosition`, `find_inside_position`, `local_to_global`).

use ac2_core::Random;

use crate::cells::{CellSource, ObjCell};
use crate::fpu::{ld, st};
use crate::frame::Frame;
use crate::landdefs::{adjust_to_outside, is_outside};
use crate::physics_obj::PhysicsObj;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::transition::{Transition, TransitionObject};
use crate::vector::Vector;

/// `SetPositionFlag`: after the body is placed, `SetPlacementPosition` makes the requested
/// position its previous and animation position and restarts its mode.
pub const PLACEMENT_SPF: u32 = 0x1;
/// `SetPositionFlag`: slide to a free spot nearby when the position itself is blocked.
pub const SLIDE_SPF: u32 = 0x10;
/// `SetPositionFlag`: put the body in the cell without a placement check.
pub const FORCE_SPF: u32 = 0x40;
/// `SetPositionFlag`: when the position itself fails, try up to `num_tries` positions
/// scattered around it (`SetScatterPositionInternal`).
pub const SCATTER_SPF: u32 = 0x100;
/// `SetPositionFlag`: skip the exact position and only try scattered ones.
pub const RANDOMSCATTER_SPF: u32 = 0x200;

/// `SetPositionStruct`: where to place a body and how (the flags above), with the scatter
/// radii and try count. (`line`, the `LINE_SPF` direction, is not used by any placement here.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetPositionStruct {
    pub position: Position,
    pub flags: u32,
    pub x_radius: f32,
    pub y_radius: f32,
    pub num_tries: u32,
}

impl SetPositionStruct {
    /// `SetPositionStruct::SetPositionStruct` with `SetPosition` and `SetFlags`: no scatter.
    pub fn new(position: Position, flags: u32) -> Self {
        Self {
            position,
            flags,
            x_radius: 0.0,
            y_radius: 0.0,
            num_tries: 0,
        }
    }

    /// `SetPositionStruct::SetScatterRadius`: a negative `y_radius` repeats `x_radius`.
    pub fn set_scatter_radius(&mut self, x_radius: f32, y_radius: f32) {
        self.x_radius = x_radius;
        self.y_radius = if y_radius < 0.0 { x_radius } else { y_radius };
    }

    /// `SetPositionStruct::SetNumTries`.
    pub fn set_num_tries(&mut self, num_tries: u32) {
        self.num_tries = num_tries;
    }
}

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
pub(crate) fn adjust_position(
    cells: &dyn CellSource,
    p: &mut Position,
    sphere_center: Vector,
) -> Option<u32> {
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
    /// `CPhysicsObj::SetPosition(SetPositionStruct*)`: `SetPositionInternal` for the struct, then
    /// on success with `PLACEMENT_SPF` `SetPlacementPosition` of the requested position.
    /// `random` is the client's shared generator (`Random::RollDice`) the scatter flags roll on.
    pub(crate) fn set_position_struct(
        &mut self,
        sps: &SetPositionStruct,
        terrain: &Terrain,
        random: &mut Random,
    ) -> SetPositionError {
        let result = self.set_position_internal_struct(sps, terrain, random);
        if result == SetPositionError::Ok && sps.flags & PLACEMENT_SPF != 0 {
            self.set_placement_position(&sps.position);
        }
        result
    }

    /// `CPhysicsObj::SetPositionInternal(SetPositionStruct*)`: `RANDOMSCATTER_SPF` scatters
    /// at once; otherwise the exact position, then (with `SCATTER_SPF`, when it failed) the
    /// scattered ones.
    fn set_position_internal_struct(
        &mut self,
        sps: &SetPositionStruct,
        terrain: &Terrain,
        random: &mut Random,
    ) -> SetPositionError {
        if sps.flags & RANDOMSCATTER_SPF != 0 {
            return self.set_scatter_position_internal(sps, terrain, random);
        }
        let result = self.set_position_checked(sps.position, sps.flags, terrain);
        if result != SetPositionError::Ok && sps.flags & SCATTER_SPF != 0 {
            return self.set_scatter_position_internal(sps, terrain, random);
        }
        result
    }

    /// `CPhysicsObj::SetScatterPositionInternal`: up to `num_tries` positions, each the
    /// requested one moved by `RollDice(-1, 1)` times the x radius and again (a second roll)
    /// times the y radius; the first that places stands, else the last error (general failure
    /// with no tries).
    fn set_scatter_position_internal(
        &mut self,
        sps: &SetPositionStruct,
        terrain: &Terrain,
        random: &mut Random,
    ) -> SetPositionError {
        let mut result = SetPositionError::GeneralFailure;
        for _ in 0..sps.num_tries {
            let mut p = sps.position;
            let x = random.roll_dice_f32(-1.0, 1.0);
            let y = random.roll_dice_f32(-1.0, 1.0);
            p.frame.origin.x = st(ld(x) * ld(sps.x_radius) + ld(p.frame.origin.x));
            p.frame.origin.y = st(ld(y) * ld(sps.y_radius) + ld(p.frame.origin.y));
            result = self.set_position_checked(p, sps.flags, terrain);
            if result == SetPositionError::Ok {
                return result;
            }
        }
        result
    }

    /// `CPhysicsObj::SetPlacementPosition`: the previous and animation positions become `pos`
    /// (no render interpolation across the placement), the motion interpreter turns to its
    /// heading at once (`CMotionInterp::TurnTo`), and the animation's mode is set again.
    pub(crate) fn set_placement_position(&mut self, pos: &Position) {
        // Retail takes the requested position as is; when `AdjustPosition` put the body in
        // another cell, the previous position names that cell instead (AGENTS.md exception), so
        // a drawn position never reports a cell the body is not in.
        let at = if pos.cell_id == self.position.cell_id {
            *pos
        } else {
            self.position
        };
        self.prev_position = at;
        self.anim_position = at;
        self.motion.turn_to(pos.frame.heading());
        // Retail sets `GetMode()`, the mode reached so far; here the desired mode, so a mode
        // requested before the object entered the world is not dropped (AGENTS.md exception).
        let mode = self.anim.modes.desired_mode();
        self.anim
            .with_modes(|modes, source| modes.set_mode(mode, source));
        self.sync_mode();
    }

    /// `CPhysicsObj::StorePosition`: the body's position without any cell residency; an outdoor
    /// position is first moved into the land cell under it (`LandDefs::adjust_to_outside`, which
    /// leaves cell 0 off the map).
    pub(crate) fn store_position(&mut self, position: Position) {
        let mut p = position;
        if is_outside(p.cell_id) {
            adjust_to_outside(&mut p.cell_id, &mut p.frame.origin);
        }
        self.position = p;
    }

    /// `CPhysicsObj::find_inside_position`: the interior cell whose BSP holds the body's first
    /// sphere centre placed at `p` (or `p`'s origin without a shape), taking it as `p`'s cell.
    /// False when no resident interior cell holds it.
    pub(crate) fn find_inside_position(&self, p: &mut Position, terrain: &Terrain) -> bool {
        let point = match self.shape.spheres.first() {
            Some(sphere) => p.local_to_global(p, sphere.center),
            None => p.frame.origin,
        };
        match terrain.find_in_visible_cells(point) {
            Some(cell) => {
                p.cell_id = cell;
                true
            }
            None => false,
        }
    }

    /// `CPhysicsObj::local_to_global`: `local_frame` (relative to the body) as a position in
    /// the body's cell, moved into the cell that holds it (`AdjustPosition` without creating
    /// cells). `None` when no resident cell does.
    pub(crate) fn local_to_global(
        &self,
        local_frame: &Frame,
        terrain: &Terrain,
    ) -> Option<Position> {
        let combined = Frame::combine(&self.position.frame, local_frame.origin, local_frame.q);
        let mut p = Position::new(self.position.cell_id, combined);
        adjust_position(terrain, &mut p, Vector::ZERO)?;
        Some(p)
    }

    /// `CPhysicsObj::SetPosition` (no scatter): a placement transition with the body's spheres
    /// from no begin position. `FORCE_SPF` (or an immobile body) skips the check; without
    /// `SLIDE_SPF` the placement may not move the body sideways. On success the body takes the
    /// placed position and its contact. The previous and animation positions are left alone
    /// (only `PLACEMENT_SPF` sets them, in [`Self::set_position_struct`]).
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
            collision: &self.collision,
            instance_id: self.instance_id,
            blocking: self.blocking,
            identity: self.world_id,
            missile_proximity: self.missile_proximity(),
        };
        let mut p = position;
        let probe = Transition::new(&object, None, None, p);
        let center = probe.sphere_path.local_sphere[0].center;
        let Some(cell) = adjust_position(terrain, &mut p, center) else {
            return SetPositionError::NoCell;
        };
        if flags & FORCE_SPF != 0 || !self.is_mobile() {
            // `ForceIntoCell`: the frame is set, the cell membership follows (the world's
            // object lists are synced to it, `PhysicsBody::world_id`), then the cell material.
            self.position = p;
            self.update_cell_material(terrain);
            return SetPositionError::Ok;
        }
        let mut transition = Transition::new(&object, Some(cell), None, p);
        if flags & SLIDE_SPF == 0 {
            transition.sphere_path.placement_allows_sliding = false;
        }
        if !self.check_position(&mut transition, &mut p, cell, flags, terrain) {
            // `SetPositionInternal`: `handle_all_collisions`' result picks `Collided`.
            return if self.handle_all_collisions(&transition.collision_info, false) {
                SetPositionError::Collided
            } else {
                SetPositionError::NoValidPosition
            };
        }
        if transition.sphere_path.curr_cell.is_none() {
            return SetPositionError::NoCell;
        }
        self.set_position_internal(&transition);
        self.update_cell_material(terrain);
        SetPositionError::Ok
    }

    /// `CPhysicsObj::UpdateCellMaterial` (`SetPositionInternal` into a new cell, `ForceIntoCell`): the material
    /// of the cell the body stands in, an interior cell's or the landscape vertex's, sets the
    /// body's gravity (`UpdateGravity`). The collision manager's environmental properties are
    /// the report's (`PhysicsBody::contact_materials`).
    pub(crate) fn update_cell_material(&mut self, terrain: &Terrain) {
        let position = self.position;
        let material = if is_outside(position.cell_id) {
            terrain.land_physics_material(&position)
        } else {
            terrain.env_physics_material(position.cell_id)
        };
        if let Some(material) = material {
            self.update_gravity(Some(&material));
            self.cell_material = Some(material);
        }
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
