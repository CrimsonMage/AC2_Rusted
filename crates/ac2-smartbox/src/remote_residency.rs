//! The residency transitions of [`Remotes`]: an object's actions and positions from the server,
//! a landblock released or made resident, and entering and leaving the world
//! (`CPhysicsObj::RecvEvt_Action` / `RecvEvt_Position`, `CObjectMaint`'s block handling,
//! `EnterWorld` / `LeaveWorld`).

use super::*;

impl<B> Remotes<B> {
    /// `CPhysicsObj::RecvEvt_Action`'s table part: an object the session no longer places leaves
    /// the world and its lost cell.
    pub fn object_action(&mut self, id: u64, records: Records<'_>) {
        let placed = records(id).is_some_and(|r| r.position.is_some());
        let Some(remote) = self.get_mut(id) else {
            return;
        };
        if !placed && remote.in_world {
            remote.in_world = false;
        }
        if !placed {
            self.remove_from_lost_cell(id);
        }
    }

    /// `CPhysicsObj::RecvEvt_Position` for an object other than the player, up to the walk: a
    /// position for an object still being created is stored; one outside a resident cell leaves
    /// visibility there; otherwise the physics range is updated, and a teleport, an object out of
    /// range or out of the world snaps there (`SyncPositionSimple`). The rest interpolates.
    pub fn receive_position(
        &mut self,
        id: u64,
        received: Position,
        teleported: bool,
        player: Option<Position>,
        records: Records<'_>,
        terrain: &mut Terrain,
    ) -> MoveOutcome {
        let Some(remote) = self.get_mut(id) else {
            return MoveOutcome::Placed(Vec::new());
        };
        if remote.body.is_none() {
            // Still being created: the newest position is where it will appear.
            remote.stored = Some(received);
            return MoveOutcome::Placed(Vec::new());
        }
        if !visible(terrain, received.cell_id) {
            let mut effects = Vec::new();
            self.leave_visibility(id, received, records, &mut effects);
            return MoveOutcome::Placed(effects);
        }
        let mut out_of_range = false;
        if let Some(player) = player {
            let d = player.offset_to(&received);
            let (x, y, z) = (f64::from(d.x), f64::from(d.y), f64::from(d.z));
            remote.player_distance = (z * z + y * y + x * x).sqrt() as f32;
            // `HandleInPhysicsRange` / `HandleOutOfPhysicsRange`.
            remote.active = remote.player_distance <= PHYSICS_RANGE;
            out_of_range = !remote.active;
        }
        if teleported || out_of_range || !remote.in_world {
            if let Some(body) = &mut remote.body {
                body.stop_interpolating();
            }
            return MoveOutcome::Placed(self.sync_position_simple(id, received, records, terrain));
        }
        MoveOutcome::Interpolate
    }

    /// `CObjectMaint::ReleaseObjCell` for a released block: each object of the block in the
    /// world leaves visibility, unless it is held.
    pub fn release_block(&mut self, block: u32, records: Records<'_>) -> Vec<ResidencyEffect> {
        let leaving: Vec<(u64, Position)> = self
            .iter()
            .filter(|(_, r)| r.in_world())
            .filter_map(|(id, r)| Some((id, r.position()?)))
            .filter(|(_, p)| p.cell_id & 0xFFFF_0000 == block)
            .collect();
        let mut effects = Vec::new();
        for (id, position) in leaving {
            let held = records(id).is_some_and(|r| r.held);
            if !held {
                self.leave_visibility(id, position, records, &mut effects);
            }
        }
        effects
    }

    /// `CObjectMaint::InitObjCell` for a resident block: the objects waiting in its lost cells
    /// enter the world at their stored positions.
    pub fn resident_block(
        &mut self,
        block: u32,
        records: Records<'_>,
        terrain: &mut Terrain,
    ) -> Vec<ResidencyEffect> {
        let mut effects = Vec::new();
        for cell in self.lost_cells_in(block) {
            let Some(objects) = self.take_lost_cell(cell) else {
                continue;
            };
            for id in objects {
                let Some(remote) = self.get_mut(id) else {
                    continue;
                };
                remote.lost_cell = None;
                if let Some(position) = remote.position() {
                    self.enter_world(id, position, records, terrain, &mut effects);
                }
            }
        }
        effects
    }

    /// `CPhysicsObj::SyncPositionSimple(pos, sliding = 1)`: in the world, `SetPositionSimple`;
    /// out of it, `ReEnterVisibility(pos)` (`EnterWorldSimple`, sliding).
    pub fn sync_position_simple(
        &mut self,
        id: u64,
        position: Position,
        records: Records<'_>,
        terrain: &mut Terrain,
    ) -> Vec<ResidencyEffect> {
        let mut effects = Vec::new();
        let Some(remote) = self.get_mut(id) else {
            return effects;
        };
        let Some(body) = &mut remote.body else {
            remote.stored = Some(position);
            return effects;
        };
        if remote.in_world {
            if body.place(position, SYNC_POSITION_FLAGS, terrain) == SetPositionError::NoCell {
                self.leave_visibility(id, position, records, &mut effects);
            }
        } else {
            self.enter_world(id, position, records, terrain, &mut effects);
        }
        effects
    }

    /// `CPhysicsObj::EnterWorldSimple(pos, 1)` -> `EnterWorld(sps)`: placed, the object is in
    /// the world (`HandleEnterWorld`: out of its lost cell, no longer to be destroyed); with no
    /// resident cell it leaves visibility there instead.
    fn enter_world(
        &mut self,
        id: u64,
        position: Position,
        records: Records<'_>,
        terrain: &mut Terrain,
        effects: &mut Vec<ResidencyEffect>,
    ) -> bool {
        let Some(remote) = self.get_mut(id) else {
            return false;
        };
        let Some(body) = &mut remote.body else {
            return false;
        };
        if remote.in_world || position.cell_id == 0 {
            return false;
        }
        match body.place(position, ENTER_WORLD_FLAGS, terrain) {
            SetPositionError::Ok => {
                remote.in_world = true;
                self.remove_from_lost_cell(id);
                effects.push(ResidencyEffect::CancelDestroy(id));
                true
            }
            SetPositionError::NoCell => {
                self.leave_visibility(id, position, records, effects);
                false
            }
            _ => false,
        }
    }

    /// `CPhysicsObj::LeaveVisibility(pos)`: `LeaveWorld` (queued for destruction), the position
    /// stored, and into the lost cell of `pos` (`CBookKeep::GotoLostCell`).
    pub fn leave_visibility(
        &mut self,
        id: u64,
        position: Position,
        records: Records<'_>,
        effects: &mut Vec<ResidencyEffect>,
    ) {
        self.leave_world(id, effects);
        let Some(remote) = self.get_mut(id) else {
            return;
        };
        match &mut remote.body {
            Some(body) => body.set_position(position),
            None => remote.stored = Some(position),
        }
        let held = records(id).is_some_and(|r| r.held);
        if position.cell_id != 0 && !held {
            self.remove_from_lost_cell(id);
            if let Some(remote) = self.get_mut(id) {
                remote.lost_cell = Some(position.cell_id);
            }
            self.push_lost_cell(position.cell_id, id);
        }
    }

    /// `CPhysicsObj::StickySetParent`'s `LeaveWorld`: the object is out of the world and, as
    /// `CBookKeep::RemoveObjectToBeDestroyed` follows at once, not queued for destruction.
    pub fn leave_world_for_parent(&mut self, id: u64) {
        let mut queued = Vec::new();
        self.leave_world(id, &mut queued);
    }

    /// `CPhysicsObj::LeaveWorld`: out of the world (`HandleLeaveWorld`), or, already out, out of
    /// its lost cell; either way queued for destruction (`CBookKeep::AddObjectToBeDestroyed`).
    fn leave_world(&mut self, id: u64, effects: &mut Vec<ResidencyEffect>) {
        let Some(remote) = self.get_mut(id) else {
            return;
        };
        if remote.in_world {
            remote.in_world = false;
        } else {
            self.remove_from_lost_cell(id);
        }
        effects.push(ResidencyEffect::QueueDestroy(id));
    }
}
