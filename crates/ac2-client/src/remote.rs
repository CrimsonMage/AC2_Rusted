//! The physics objects of everything but the player: `CObjectMaint` (`FinishEntityCreation`,
//! the lost-cell table, `ReleaseObjCell` / `InitObjCell`), `CBookKeep` and `CPhysicsObj`'s
//! visibility (`SyncPositionSimple`, `EnterWorldSimple`, `LeaveVisibility`,
//! `ReEnterVisibility`, `LeaveWorld`), and the non-player branch of
//! `CPhysicsObj::RecvEvt_Position` (Legions follows admin 0x004ED470).
//!
//! An object whose cell is not resident leaves the world into a "lost cell" with its position
//! stored and is queued for destruction 25 s on (`OnlineSession::add_object_to_be_destroyed`);
//! when that cell becomes resident it re-enters (`EnterWorldSimple`, sliding) and the
//! destruction is cancelled.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use ac2_net::protocol::ServerPosition;
use ac2_physics::{
    CellSource, Frame, Heading, LookTarget, PhysicsBody, Position, Quaternion, SetPositionError,
    Terrain, Vector,
};
use ac2_weenie::{OnlineSession, RemoteObject};

/// A body's look-at snapshot: where it is and how big (`GetPosition`, `GetHeight`, `is_mobile`).
pub(crate) fn look_snapshot(body: &ac2_physics::PhysicsBody) -> LookTarget {
    LookTarget {
        position: body.position(),
        height: body.height(),
        mobile: body.is_mobile(),
    }
}

use crate::client::{ENTER_WORLD_FLAGS, SYNC_POSITION_FLAGS};
use crate::world_data::BodyData;

/// `CPhysicsObj::RecvEvt_Position`: beyond this distance from the player (metres) an object is
/// out of physics range (`HandleOutOfPhysicsRange`) and snaps to its positions.
const PHYSICS_RANGE: f32 = 96.0;

/// One object other than the player.
#[derive(Debug)]
pub(crate) struct Remote {
    /// The visual its Setup comes from.
    visual: Option<u32>,
    /// Built once its Setup has loaded (`CObjectMaint::FinishEntityCreation`).
    body: Option<PhysicsBody>,
    /// The Setup failed to load: the object is shown at its server position only.
    failed: bool,
    /// `get_position` until the body exists (`CPhysicsObj::StorePosition`).
    stored: Option<Position>,
    /// `CPhysicsObj::IsInWorld`.
    in_world: bool,
    /// `CBookKeep::m_lostCellID`.
    lost_cell: Option<u32>,
    /// `!(m_transient_state & 0x2000)`: within physics range of the player.
    active: bool,
    /// `CPhysicsObj::m_fPlayerDistance`.
    player_distance: f32,
}

impl Remote {
    pub(crate) fn position(&self) -> Option<Position> {
        self.body
            .as_ref()
            .map(PhysicsBody::position)
            .or(self.stored)
    }

    pub(crate) fn in_world(&self) -> bool {
        self.in_world
    }

    pub(crate) fn body(&self) -> Option<&PhysicsBody> {
        self.body.as_ref()
    }

    pub(crate) fn active(&self) -> bool {
        self.active
    }

    pub(crate) fn lost_cell(&self) -> Option<u32> {
        self.lost_cell
    }
}

/// The received position as a `Position`: the pack's heading on an upright frame
/// (`PositionOffset::ToPosition`, `Frame::set_heading`).
pub(crate) fn received_position(update: &ServerPosition) -> Position {
    let o = update.position.origin;
    let mut frame = Frame::new(
        Vector::new(o[0], o[1], o[2]),
        Quaternion::new(1.0, 0.0, 0.0, 0.0),
    );
    frame.set_heading(Heading::from_degrees(update.heading_degrees));
    Position::new(update.position.cell, frame)
}

fn table_position(object: &RemoteObject) -> Option<Position> {
    let p = object.position?;
    let [w, x, y, z] = p.rotation;
    Some(Position::new(
        p.cell,
        Frame::new(
            Vector::new(p.origin[0], p.origin[1], p.origin[2]),
            Quaternion::new(w, x, y, z),
        ),
    ))
}

/// `CObjCell::GetVisible(cell, 1)`: the cell is resident.
fn visible(terrain: &Terrain, cell: u32) -> bool {
    terrain.land_cell(cell).is_some() || terrain.env_cell(cell).is_some()
}

/// The client's objects other than the player.
#[derive(Default)]
pub(crate) struct Remotes {
    objects: BTreeMap<u64, Remote>,
    /// `CObjectMaint::lost_cell_table`: each lost cell's objects, in arrival order.
    lost_cells: BTreeMap<u32, Vec<u64>>,
    /// Loaded Setups by visual, and the visuals asked for.
    bodies: HashMap<Option<u32>, Result<BodyData, String>>,
    requested: BTreeSet<Option<u32>>,
}

/// What the remote objects need from the client on each call.
pub(crate) struct Context<'a> {
    pub session: &'a mut OnlineSession,
    pub terrain: &'a Terrain,
    /// The player's position (`CPhysics::GetObjectA(player_id)->GetPosition`).
    pub player: Option<Position>,
}

impl Remotes {
    pub(crate) fn get(&self, id: u64) -> Option<&Remote> {
        self.objects.get(&id)
    }

    /// The visuals to load (each once).
    pub(crate) fn take_requests(&mut self) -> Vec<Option<u32>> {
        let wanted: Vec<_> = self
            .objects
            .values()
            .filter(|r| r.body.is_none() && !r.failed)
            .map(|r| r.visual)
            .filter(|v| !self.requested.contains(v) && !self.bodies.contains_key(v))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        self.requested.extend(wanted.iter().copied());
        wanted
    }

    /// `CObjectMaint::RecvEvt_CreateObject` for an object other than the player. A new object
    /// waits for its Setup; the same instance again (`CObjectMaint::UpdateObject`) moves to the
    /// create's position (`SyncPositionSimple(pos, 1)`).
    pub(crate) fn added(&mut self, id: u64, cx: &mut Context<'_>) {
        let Some(object) = cx.session.objects().get(id) else {
            return;
        };
        let position = table_position(object);
        let held = object.parent.is_some();
        let visual = object.visual;
        if self.objects.contains_key(&id) {
            if let (Some(position), false) = (position, held) {
                if position.cell_id != 0 {
                    self.sync_position_simple(id, position, cx);
                }
            }
            return;
        }
        self.objects.insert(
            id,
            Remote {
                visual,
                body: None,
                failed: false,
                stored: position,
                in_world: false,
                lost_cell: None,
                active: true,
                player_distance: 0.0,
            },
        );
        self.try_create(id, cx);
    }

    /// A Setup arrived from a loader thread.
    pub(crate) fn body_loaded(
        &mut self,
        visual: Option<u32>,
        body: Result<BodyData, String>,
        cx: &mut Context<'_>,
    ) {
        self.bodies.insert(visual, body);
        let waiting: Vec<u64> = self
            .objects
            .iter()
            .filter(|(_, r)| r.visual == visual && r.body.is_none() && !r.failed)
            .map(|(&id, _)| id)
            .collect();
        for id in waiting {
            self.try_create(id, cx);
        }
    }

    /// `CObjectMaint::FinishEntityCreation` once the Setup is in: the physics object, placed at
    /// the create's position when it has a cell and no parent (`SyncPositionSimple(pos, 1)`).
    fn try_create(&mut self, id: u64, cx: &mut Context<'_>) {
        let Some(remote) = self.objects.get_mut(&id) else {
            return;
        };
        let Some(loaded) = self.bodies.get(&remote.visual) else {
            return;
        };
        let data = match loaded {
            Ok(data) => data.clone(),
            Err(_) => {
                remote.failed = true;
                return;
            }
        };
        let start = remote.stored.unwrap_or_default();
        let mut body = PhysicsBody::new(start, data.shape, data.motion);
        body.set_radius(data.radius);
        if let Some(animation) = data.animation {
            body.set_animation(animation);
        }
        // The `PhysicsDesc`'s mode, behaviours and sliders (`CPhysicsObj::set_description`);
        // the mode is the latest `SetMode` when one came before the body.
        if let Some(object) = cx.session.objects().get(id) {
            let mode = object.mode.unwrap_or(0);
            crate::motion::describe(&mut body, mode, &object.behaviors, &object.sliders);
        }
        remote.body = Some(body);
        let held = cx
            .session
            .objects()
            .get(id)
            .is_some_and(|o| o.parent.is_some());
        if let (Some(position), false) = (remote.stored, held) {
            if position.cell_id != 0 {
                self.sync_position_simple(id, position, cx);
            }
        }
    }

    /// `CPhysicsObj::SyncPositionSimple(pos, sliding = 1)`: in the world, `SetPositionSimple`;
    /// out of it, `ReEnterVisibility(pos)` (`EnterWorldSimple`, sliding).
    fn sync_position_simple(&mut self, id: u64, position: Position, cx: &mut Context<'_>) {
        let Some(remote) = self.objects.get_mut(&id) else {
            return;
        };
        let Some(body) = &mut remote.body else {
            remote.stored = Some(position);
            return;
        };
        if remote.in_world {
            if body.place(position, SYNC_POSITION_FLAGS, cx.terrain) == SetPositionError::NoCell {
                self.leave_visibility(id, position, cx);
            }
        } else {
            self.enter_world(id, position, cx);
        }
    }

    /// `CPhysicsObj::EnterWorldSimple(pos, 1)` -> `EnterWorld(sps)`: placed, the object is in
    /// the world (`HandleEnterWorld`: out of its lost cell, no longer to be destroyed); with no
    /// resident cell it leaves visibility there instead.
    fn enter_world(&mut self, id: u64, position: Position, cx: &mut Context<'_>) -> bool {
        let Some(remote) = self.objects.get_mut(&id) else {
            return false;
        };
        let Some(body) = &mut remote.body else {
            return false;
        };
        if remote.in_world || position.cell_id == 0 {
            return false;
        }
        match body.place(position, ENTER_WORLD_FLAGS, cx.terrain) {
            SetPositionError::Ok => {
                remote.in_world = true;
                self.remove_from_lost_cell(id);
                cx.session.remove_object_to_be_destroyed(id);
                true
            }
            SetPositionError::NoCell => {
                self.leave_visibility(id, position, cx);
                false
            }
            _ => false,
        }
    }

    /// `CPhysicsObj::LeaveVisibility(pos)`: `LeaveWorld` (queued for destruction), the position
    /// stored, and into the lost cell of `pos` (`CBookKeep::GotoLostCell`).
    fn leave_visibility(&mut self, id: u64, position: Position, cx: &mut Context<'_>) {
        self.leave_world(id, cx);
        let Some(remote) = self.objects.get_mut(&id) else {
            return;
        };
        match &mut remote.body {
            Some(body) => body.set_position(position),
            None => remote.stored = Some(position),
        }
        let held = cx
            .session
            .objects()
            .get(id)
            .is_some_and(|o| o.parent.is_some());
        if position.cell_id != 0 && !held {
            self.remove_from_lost_cell(id);
            if let Some(remote) = self.objects.get_mut(&id) {
                remote.lost_cell = Some(position.cell_id);
            }
            self.lost_cells
                .entry(position.cell_id)
                .or_default()
                .push(id);
        }
    }

    /// `CPhysicsObj::LeaveWorld`: out of the world (`HandleLeaveWorld`), or, already out, out of
    /// its lost cell; either way queued for destruction (`CBookKeep::AddObjectToBeDestroyed`).
    fn leave_world(&mut self, id: u64, cx: &mut Context<'_>) {
        let Some(remote) = self.objects.get_mut(&id) else {
            return;
        };
        if remote.in_world {
            remote.in_world = false;
        } else {
            self.remove_from_lost_cell(id);
        }
        cx.session.add_object_to_be_destroyed(id);
    }

    /// `CObjectMaint::RemoveFromLostCell`.
    fn remove_from_lost_cell(&mut self, id: u64) {
        let Some(cell) = self.objects.get_mut(&id).and_then(|r| r.lost_cell.take()) else {
            return;
        };
        if let Some(list) = self.lost_cells.get_mut(&cell) {
            list.retain(|&other| other != id);
            if list.is_empty() {
                self.lost_cells.remove(&cell);
            }
        }
    }

    /// `CPhysicsObj::RecvEvt_Position` for an object other than the player, after the stamps
    /// (`ObjectTable::apply_position`) accepted it.
    pub(crate) fn moved(
        &mut self,
        id: u64,
        update: &ServerPosition,
        teleported: bool,
        cx: &mut Context<'_>,
    ) {
        let received = received_position(update);
        let Some(remote) = self.objects.get_mut(&id) else {
            return;
        };
        if remote.body.is_none() {
            // Still being created: the newest position is where it will appear.
            remote.stored = Some(received);
            return;
        }
        if !visible(cx.terrain, received.cell_id) {
            self.leave_visibility(id, received, cx);
            return;
        }
        let mut out_of_range = false;
        if let Some(player) = cx.player {
            let d = player.offset_to(&received);
            let (x, y, z) = (f64::from(d.x), f64::from(d.y), f64::from(d.z));
            remote.player_distance = (z * z + y * y + x * x).sqrt() as f32;
            // `HandleInPhysicsRange` / `HandleOutOfPhysicsRange`.
            remote.active = remote.player_distance <= PHYSICS_RANGE;
            out_of_range = !remote.active;
        }
        // A teleport, or an object out of range or out of the world, snaps there
        // (`StopInterpolating`, `SyncPositionSimple`); otherwise it interpolates there.
        if teleported || out_of_range || !remote.in_world {
            if let Some(body) = &mut remote.body {
                body.stop_interpolating();
            }
            self.sync_position_simple(id, received, cx);
        } else {
            self.interpolate_to(id, update, received, cx);
        }
    }

    /// `CPhysicsObj::InterpolateTo`, or `InterpolateAndImpulse` when the pack carries a jump
    /// (or an impulse): the position queues on the object's `InterpolationManager`, which walks
    /// the body there through its motion interpreter and blips it when it falls behind.
    fn interpolate_to(
        &mut self,
        id: u64,
        update: &ServerPosition,
        received: Position,
        cx: &mut Context<'_>,
    ) {
        let Some(body) = self.objects.get_mut(&id).and_then(|r| r.body.as_mut()) else {
            return;
        };
        match update.velocity.filter(|_| update.jump || update.impulse) {
            Some(velocity) => body.interpolate_and_impulse(
                update.time,
                received,
                update.heading_degrees,
                update.motion,
                velocity,
                update.jump,
            ),
            None => body.interpolate_to(
                update.time,
                received,
                update.heading_degrees,
                update.motion,
                update.contact,
                cx.terrain,
            ),
        }
    }

    /// The server's motion events (`RecvEvt_DoMode`, `RecvEvt_SetMode`, the behaviour, layer
    /// and slider events) for the object's body.
    /// `CMotionInterp::SetMIDESC` for a remote object whose description the loader read.
    pub(crate) fn set_motion_desc(&mut self, id: u64, desc: Arc<ac2_world::Ac2MotionInterpDesc>) {
        if let Some(body) = self.objects.get_mut(&id).and_then(|r| r.body.as_mut()) {
            body.set_motion_desc(Some(desc));
        }
    }

    pub(crate) fn motion(
        &mut self,
        id: u64,
        action: &ac2_net::protocol::ObjectAction,
        impulse_source: Option<ac2_physics::Position>,
    ) {
        if let Some(body) = self.objects.get_mut(&id).and_then(|r| r.body.as_mut()) {
            crate::motion::apply_owned(body, id, action, impulse_source);
        }
    }

    /// A move-to target as `TargetManager` reports it: an object in the world with a body, its
    /// position, `GetRadius` and `GetHeight` (the Setup's, scaled).
    pub(crate) fn target_info(&self, id: u64) -> Option<(Position, f32, f32)> {
        let remote = self.objects.get(&id).filter(|r| r.in_world)?;
        let body = remote.body.as_ref()?;
        let data = self.bodies.get(&remote.visual)?.as_ref().ok()?;
        let scale = f64::from(data.shape.scale);
        let radius = (f64::from(data.radius) * scale) as f32;
        let height = (f64::from(data.shape.height) * scale) as f32;
        Some((body.position(), radius, height))
    }

    /// The table's bookkeeping took the object out of the world (`RecvEvt_LeaveWorld`,
    /// `RecvEvt_DeParent`) or made it a held child (`RecvEvt_Parent`): its body leaves the world.
    pub(crate) fn action(&mut self, id: u64, cx: &mut Context<'_>) {
        let placed = cx
            .session
            .objects()
            .get(id)
            .is_some_and(|o| o.position.is_some());
        let Some(remote) = self.objects.get_mut(&id) else {
            return;
        };
        if !placed && remote.in_world {
            remote.in_world = false;
        }
        if !placed {
            self.remove_from_lost_cell(id);
        }
    }

    /// `CObjectMaint::DestroyObjectInternal` removed the object.
    pub(crate) fn removed(&mut self, id: u64) {
        self.remove_from_lost_cell(id);
        self.objects.remove(&id);
    }

    /// `CObjectMaint::ReleaseObjCell` for every cell of a released block: its objects in the
    /// world, unless held, leave visibility where they are.
    pub(crate) fn block_released(&mut self, block: u32, cx: &mut Context<'_>) {
        let leaving: Vec<(u64, Position)> = self
            .objects
            .iter()
            .filter(|(_, r)| r.in_world)
            .filter_map(|(&id, r)| Some((id, r.position()?)))
            .filter(|(_, p)| p.cell_id & 0xFFFF_0000 == block)
            .collect();
        for (id, position) in leaving {
            let held = cx
                .session
                .objects()
                .get(id)
                .is_some_and(|o| o.parent.is_some());
            if !held {
                self.leave_visibility(id, position, cx);
            }
        }
    }

    /// `CObjectMaint::InitObjCell` for every cell of a resident block: each lost cell there
    /// gives its objects back to the world (`CBookKeep::ReEnterVisibility`).
    pub(crate) fn block_resident(&mut self, block: u32, cx: &mut Context<'_>) {
        let cells: Vec<u32> = self
            .lost_cells
            .keys()
            .copied()
            .filter(|cell| cell & 0xFFFF_0000 == block)
            .collect();
        for cell in cells {
            let Some(objects) = self.lost_cells.remove(&cell) else {
                continue;
            };
            for id in objects {
                let Some(remote) = self.objects.get_mut(&id) else {
                    continue;
                };
                remote.lost_cell = None;
                if let Some(position) = remote.position() {
                    self.enter_world(id, position, cx);
                }
            }
        }
    }

    /// `CPhysics::UpdateAllPhysics` for the objects: each one in the world and in physics range
    /// runs the quantum.
    /// `cur_time` is the server clock the positions were stamped on.
    pub(crate) fn step_quantum(&mut self, terrain: &Terrain, cur_time: f64) {
        for remote in self.objects.values_mut() {
            if !remote.in_world || !remote.active {
                continue;
            }
            if let Some(body) = &mut remote.body {
                body.set_time(cur_time);
                body.step_quantum(terrain);
            }
        }
    }

    /// `CPhysics::UpdateAllAnimation` for the objects.
    pub(crate) fn update_animation(
        &mut self,
        step: f32,
        local_time: f64,
        wind: [f32; 3],
        random: &mut ac2_core::Random,
        scripts: &crate::FxScripts,
        looks: &HashMap<u64, LookTarget>,
    ) {
        for remote in self.objects.values_mut() {
            if let Some(body) = &mut remote.body {
                body.resolve_fx_scripts(|id| scripts.get(id));
                body.set_wind(wind);
                body.set_look_snapshot(body.look_target().and_then(|t| looks.get(&t).copied()));
                body.update_animation(step, local_time, random);
            }
        }
    }

    /// Every in-world remote's look-at snapshot (position, height, mobility), by id.
    pub(crate) fn look_targets(&self, into: &mut HashMap<u64, LookTarget>) {
        for (id, remote) in &self.objects {
            if let (true, Some(body)) = (remote.in_world, remote.body.as_ref()) {
                into.insert(*id, look_snapshot(body));
            }
        }
    }

    /// The animation events each object's body raised since the last call, with its visual.
    pub(crate) fn take_animation_events(
        &mut self,
    ) -> Vec<(u64, Option<u32>, Vec<ac2_anim::Ac2AnimEvent>)> {
        self.objects
            .iter_mut()
            .filter_map(|(id, remote)| {
                let events = remote.body.as_mut()?.take_animation_events();
                Some((*id, remote.visual, events))
            })
            .collect()
    }

    /// The loaded data of `visual`.
    pub(crate) fn body_data(&self, visual: Option<u32>) -> Option<&BodyData> {
        self.bodies.get(&visual)?.as_ref().ok()
    }

    /// Leaving the world: every object goes.
    pub(crate) fn clear(&mut self) {
        self.objects.clear();
        self.lost_cells.clear();
    }
}
