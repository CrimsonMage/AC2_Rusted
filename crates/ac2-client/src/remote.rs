//! The physics objects of everything but the player: the session-facing half of the residency.
//! The residency state (object table, lost cells, bodies, shadows) is `ac2_smartbox::remotes`;
//! this module reads the session (`RemoteObject`'s position, parent, mode and destroy queue),
//! and turns the server's positions and motion into the calls the residency takes.
//!
//! An object whose cell is not resident leaves the world into a "lost cell" with its position
//! stored and is queued for destruction 25 s on (`OnlineSession::add_object_to_be_destroyed`);
//! when that cell becomes resident it re-enters (`EnterWorldSimple`, sliding) and the
//! destruction is cancelled.

use ac2_net::protocol::ServerPosition;
use ac2_physics::{Frame, Heading, Position, Quaternion, Vector};
use ac2_smartbox::remotes::{ObjectRecord, Remote, ResidencyEffect, FX_OBJECT_COLLISION_BIT};
use ac2_weenie::{OnlineSession, RemoteObject};

pub(crate) use ac2_smartbox::remotes::{is_entity_desc, look_snapshot, visible};

use crate::frame::FxFollow;
use crate::world_data::BodyData;

/// `CPhysicsObj::RecvEvt_Position`: beyond this distance from the player (metres) an object is
/// out of physics range (`HandleOutOfPhysicsRange`) and snaps to its positions.
const PHYSICS_RANGE: f32 = 96.0;

/// The client's residency of the objects other than the player: the smartbox table over the
/// client's body payload.
pub(crate) type Remotes = ac2_smartbox::remotes::Remotes<BodyData>;

/// What the remote objects need from the client on each call.
pub(crate) struct Context<'a> {
    pub session: &'a mut ac2_weenie::OnlineSession,
    pub terrain: &'a mut ac2_physics::Terrain,
    /// The player's position (`CPhysics::GetObjectA(player_id)->GetPosition`).
    pub player: Option<Position>,
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

/// The session facts of object `id` that the residency's calls read.
fn record(session: &OnlineSession, id: u64) -> Option<ObjectRecord> {
    let object = session.objects().get(id)?;
    Some(ObjectRecord {
        position: table_position(object),
        held: object.parent.is_some(),
    })
}

/// Applies the destroy queue changes a residency call made, in order.
fn apply_effects(session: &mut OnlineSession, effects: Vec<ResidencyEffect>) {
    for effect in effects {
        match effect {
            ResidencyEffect::QueueDestroy(id) => session.add_object_to_be_destroyed(id),
            ResidencyEffect::CancelDestroy(id) => session.remove_object_to_be_destroyed(id),
        }
    }
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

/// The session calls of the residency: each takes the client's context and the object's session
/// entry where it reads one.
pub(crate) trait RemoteCalls {
    /// `CObjectMaint::RecvEvt_CreateObject` for an object other than the player.
    fn added(&mut self, id: u64, cx: &mut Context<'_>);
    /// A Setup arrived from a loader thread.
    fn body_loaded(
        &mut self,
        visual: Option<u32>,
        body: Result<BodyData, String>,
        cx: &mut Context<'_>,
    );
    /// `CPhysicsObj::RecvEvt_Position` for an object other than the player.
    fn moved(&mut self, id: u64, update: &ServerPosition, teleported: bool, cx: &mut Context<'_>);
    /// The server's motion events for the object's body.
    fn motion(
        &mut self,
        id: u64,
        action: &ac2_net::protocol::ObjectAction,
        impulse_source: Option<Position>,
    );
    /// The table's bookkeeping took the object out of the world or made it a held child.
    fn action(&mut self, id: u64, cx: &mut Context<'_>);
    /// `CObjectMaint::CreateTemporaryObject`: an object the client makes (an FX's).
    fn add_local(
        &mut self,
        id: u64,
        visual: u32,
        position: Position,
        fx_follow: Option<FxFollow>,
        alpha_moves: [(f32, f32); 2],
        cx: &mut Context<'_>,
    );
    /// `CObjectMaint::ReleaseObjCell` for every cell of a released block.
    fn block_released(&mut self, block: u32, cx: &mut Context<'_>);
    /// `CObjectMaint::InitObjCell` for every cell of a resident block.
    fn block_resident(&mut self, block: u32, cx: &mut Context<'_>);
}

impl RemoteCalls for Remotes {
    fn added(&mut self, id: u64, cx: &mut Context<'_>) {
        let Some(object) = cx.session.objects().get(id) else {
            return;
        };
        let position = table_position(object);
        let held = object.parent.is_some();
        let visual = object.visual;
        if let Some(did) = object.entity.filter(|&did| is_entity_desc(did)) {
            self.record_entity(id, did);
        }
        if self.tracks(id) {
            if let (Some(position), false) = (position, held) {
                if position.cell_id != 0 {
                    let effects = self.sync_position_simple(
                        id,
                        position,
                        &|id| record(cx.session, id),
                        cx.terrain,
                    );
                    apply_effects(cx.session, effects);
                }
            }
            return;
        }
        self.insert(
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
                object: None,
                shadow: None,
                ethereal: false,
                alpha_moves: None,
                fx_follow: None,
            },
        );
        try_create(self, id, cx);
    }

    fn body_loaded(
        &mut self,
        visual: Option<u32>,
        body: Result<BodyData, String>,
        cx: &mut Context<'_>,
    ) {
        for id in self.store_body(visual, body) {
            try_create(self, id, cx);
        }
    }

    fn moved(&mut self, id: u64, update: &ServerPosition, teleported: bool, cx: &mut Context<'_>) {
        let received = received_position(update);
        let Some(remote) = self.get_mut(id) else {
            return;
        };
        if remote.body.is_none() {
            // Still being created: the newest position is where it will appear.
            remote.stored = Some(received);
            return;
        }
        if !visible(cx.terrain, received.cell_id) {
            let mut effects = Vec::new();
            self.leave_visibility(id, received, &|id| record(cx.session, id), &mut effects);
            apply_effects(cx.session, effects);
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
            let effects =
                self.sync_position_simple(id, received, &|id| record(cx.session, id), cx.terrain);
            apply_effects(cx.session, effects);
        } else {
            interpolate_to(self, id, update, received, cx);
        }
    }

    fn motion(
        &mut self,
        id: u64,
        action: &ac2_net::protocol::ObjectAction,
        impulse_source: Option<Position>,
    ) {
        if let Some(body) = self.body_mut(id) {
            crate::motion::apply_owned(body, id, action, impulse_source);
        }
    }

    fn action(&mut self, id: u64, cx: &mut Context<'_>) {
        self.object_action(id, &|id| record(cx.session, id));
    }

    fn add_local(
        &mut self,
        id: u64,
        visual: u32,
        position: Position,
        fx_follow: Option<FxFollow>,
        alpha_moves: [(f32, f32); 2],
        cx: &mut Context<'_>,
    ) {
        self.insert_local(
            id,
            Remote {
                visual: Some(visual),
                body: None,
                failed: false,
                stored: Some(position),
                in_world: false,
                lost_cell: None,
                active: true,
                player_distance: 0.0,
                object: None,
                shadow: None,
                ethereal: true,
                alpha_moves: Some(alpha_moves),
                fx_follow,
            },
        );
        try_create(self, id, cx);
    }

    fn block_released(&mut self, block: u32, cx: &mut Context<'_>) {
        let effects = self.release_block(block, &|id| record(cx.session, id));
        apply_effects(cx.session, effects);
    }

    fn block_resident(&mut self, block: u32, cx: &mut Context<'_>) {
        let effects = self.resident_block(block, &|id| record(cx.session, id), cx.terrain);
        apply_effects(cx.session, effects);
    }
}

/// `CObjectMaint::FinishEntityCreation` once the Setup is in: the physics object, placed at
/// the create's position when it has a cell and no parent (`SyncPositionSimple(pos, 1)`).
fn try_create(remotes: &mut Remotes, id: u64, cx: &mut Context<'_>) {
    let Some(visual) = remotes.get(id).map(|r| r.visual()) else {
        return;
    };
    let data = match remotes.body_result(visual) {
        None => return,
        Some(Ok(data)) => Some(data.clone()),
        Some(Err(_)) => None,
    };
    let Some(data) = data else {
        if let Some(remote) = remotes.get_mut(id) {
            remote.failed = true;
        }
        return;
    };
    let Some(remote) = remotes.get_mut(id) else {
        return;
    };
    let start = remote.stored.unwrap_or_default();
    let ethereal = remote.ethereal;
    let stored = remote.stored;
    let alpha_moves = remote.alpha_moves.take();
    remote.object = data.body.object.clone();
    let mut body = ac2_physics::PhysicsBody::new(start, data.body.shape, data.body.motion);
    body.set_radius(data.body.radius);
    if ethereal {
        // `FXManager::DoCreateObjectFX`: `CollisionParameters::SetBit(0x10)` and the ETHEREAL
        // physics type, which the object's list entry carries (`sync_shadow`).
        body.collision_mut().set_bit(FX_OBJECT_COLLISION_BIT);
    }
    if let Some(bits) = remotes.entity_mask(id) {
        body.apply_blocking_bits(bits);
    }
    if let Some(animation) = data.body.animation {
        body.set_animation(animation);
    }
    // The `PhysicsDesc`'s mode, behaviours and sliders (`CPhysicsObj::set_description`);
    // the mode is the latest `SetMode` when one came before the body.
    if let Some(object) = cx.session.objects().get(id) {
        let mode = object.mode.unwrap_or(0);
        crate::motion::describe(&mut body, mode, &object.behaviors, &object.sliders);
    }
    if let Some(moves) = alpha_moves {
        for (pos, time) in moves {
            body.do_slider(0, ac2_smartbox::fx_objects::ALPHA_SLIDER, pos, time);
        }
    }
    if let Some(remote) = remotes.get_mut(id) {
        remote.body = Some(body);
    }
    let held = cx
        .session
        .objects()
        .get(id)
        .is_some_and(|o| o.parent.is_some());
    if let (Some(position), false) = (stored, held) {
        if position.cell_id != 0 {
            let effects = remotes.sync_position_simple(
                id,
                position,
                &|id| record(cx.session, id),
                cx.terrain,
            );
            apply_effects(cx.session, effects);
        }
    }
}

/// `CPhysicsObj::RecvEvt_Position` for an object other than the player, after the stamps
/// (`ObjectTable::apply_position`) accepted it: `InterpolateTo`, or `InterpolateAndImpulse`
/// when the pack carries a jump (or an impulse): the position queues on the object's
/// `InterpolationManager`, which walks the body there and blips it when it falls behind.
fn interpolate_to(
    remotes: &mut Remotes,
    id: u64,
    update: &ServerPosition,
    received: Position,
    cx: &mut Context<'_>,
) {
    let Some(body) = remotes.body_mut(id) else {
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
