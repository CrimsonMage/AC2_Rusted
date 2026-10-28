//! The client's glue for the objects other than the player: the session's object events and
//! the resident landblocks drive [`Remotes`], and each frame lists them for the renderer.

use std::sync::Arc;

use ac2_net::protocol::{MoveTarget, ObjectAction};
use ac2_weenie::SessionEvent;

use crate::cells::BlockChange;
use crate::client::Client;
use crate::frame::{HoldPoint, Holder, ObjectFrame};
use crate::loader::Request;
use crate::remote::{Context, RemoteCalls};
use crate::world_data::BodyData;

impl Client {
    /// The remote objects' view of the client this call.
    fn remote_context(&mut self) -> (&mut crate::remote::Remotes, Context<'_>) {
        let player = self
            .player
            .as_ref()
            .filter(|p| p.body.is_some())
            .map(|p| p.position());
        (
            &mut self.remotes,
            Context {
                session: &mut self.session,
                terrain: &mut self.terrain,
                player,
            },
        )
    }

    /// Asks the loader threads for the Setups new objects need.
    pub(crate) fn request_remote_bodies(&mut self) {
        for visual in self.remotes.take_requests() {
            self.loader.request(Request::Body(visual));
        }
        for (object, did) in self.remotes.take_entity_requests() {
            self.loader.request(Request::EntityMask(object, did));
        }
        self.request_player_entity_mask();
    }

    /// The player's create names an `EntityDesc` (once, as the player's body is made from it):
    /// its ethereal bits load on a loader thread.
    fn request_player_entity_mask(&mut self) {
        let Some(player) = self.player.as_ref().filter(|p| !p.entity_mask_requested) else {
            return;
        };
        let id = player.id;
        let Some(did) = self
            .session
            .objects()
            .get(id)
            .and_then(|object| object.entity)
            .filter(|&did| crate::remote::is_entity_desc(did))
        else {
            return;
        };
        if let Some(player) = self.player.as_mut() {
            player.entity_mask_requested = true;
        }
        self.loader.request(Request::EntityMask(id, did));
    }

    /// `CPhysicsObj::InitVisualDesc`'s `ApplyPropertyCollection` landed for an object: the
    /// player's body or a remote's takes the ethereal bits. A failed read sets none.
    pub(crate) fn entity_mask_loaded(&mut self, object: u64, mask: Result<u64, String>) {
        let bits = mask.unwrap_or(0);
        if let Some(body) = self
            .player
            .as_mut()
            .filter(|p| p.id == object)
            .and_then(|p| p.body.as_mut())
        {
            body.physics.apply_blocking_bits(bits);
            return;
        }
        self.remotes.entity_mask_loaded(object, bits);
    }

    /// The session's object events (`CObjectMaint`, `CPhysicsObj::RecvEvt_*`).
    /// A physics change a script asked of a weenie's physics object (`gCWPhysicsObject`'s scale
    /// and FX efuns, the `CWPhysicsObject` calls behind them): the server event's effect, on the
    /// player's body or a remote's.
    pub(crate) fn apply_script_physics(
        &mut self,
        object: u64,
        action: &ac2_net::protocol::ObjectAction,
    ) {
        if let Some(body) = self
            .player
            .as_mut()
            .filter(|p| p.id == object)
            .and_then(|p| p.body.as_mut())
        {
            crate::motion::apply_owned(&mut body.physics, object, action, None);
            return;
        }
        self.remotes.motion(object, action, None);
    }

    /// `CCollisionManager::SetCollisionState` for a script's `CollisionParameters`: the player's
    /// body takes the non-zero impact minimums. Remote bodies keep theirs (their scripts are not
    /// run here; see the catalog).
    pub(crate) fn apply_collision_minimums(&mut self, object: u64, xy: f32, z: f32) {
        if let Some(body) = self
            .player
            .as_mut()
            .filter(|p| p.id == object)
            .and_then(|p| p.body.as_mut())
        {
            body.physics.set_collision_minimums(xy, z);
        }
    }

    /// `CCollisionManager::SetCollisionState` for a script's collision sets: the player's body
    /// merges the script's placement and movement ethereal and physics-type sets (only the sets
    /// its bits name). Remote bodies keep theirs, as with the minimums.
    pub(crate) fn apply_collision_sets(
        &mut self,
        object: u64,
        sets: &ac2_physics::CollisionParameters,
    ) {
        if let Some(body) = self
            .player
            .as_mut()
            .filter(|p| p.id == object)
            .and_then(|p| p.body.as_mut())
        {
            body.physics.collision_mut().merge_set_state(sets);
        }
    }

    /// The position a targeted impulse on `object` pushes from: the target's, when the target
    /// is another object in the world (`CPhysics::GetObjectA(target)` with `pCVar7 != this`).
    fn impulse_source(&self, object: u64, target: u64) -> Option<ac2_physics::Position> {
        if target == object {
            return None;
        }
        match self.player.as_ref().filter(|p| p.id == target) {
            Some(player) => Some(player.position()),
            None => self
                .remotes
                .target_info(target)
                .map(|(position, _, _)| position),
        }
    }

    /// `CPhysicsObj::RecvEvt_SetMIDESC`: the description is read on a loader thread first; the
    /// body's table changes when it arrives (`motion_desc_loaded`).
    fn request_motion_desc(&mut self, event: &SessionEvent) -> bool {
        match event {
            SessionEvent::ObjectAction {
                object,
                action: ObjectAction::SetMotionInterp(did),
            } => {
                self.loader.request(Request::MotionDesc(*object, *did));
                true
            }
            _ => false,
        }
    }

    /// A loaded `MotionInterpDesc` for `object` (`CMotionInterp::SetMIDESC`): the player's body
    /// or a remote's. A description that failed to load keeps the table, as retail's SetMIDESC
    /// returns 0 when `DBObj::Get` fails.
    pub(crate) fn motion_desc_loaded(
        &mut self,
        object: u64,
        desc: Result<Option<Arc<ac2_world::Ac2MotionInterpDesc>>, String>,
    ) {
        let Ok(Some(desc)) = desc else {
            return;
        };
        if let Some(body) = self
            .player
            .as_mut()
            .filter(|p| p.id == object)
            .and_then(|p| p.body.as_mut())
        {
            body.physics.set_motion_desc(Some(desc));
            return;
        }
        self.remotes.set_motion_desc(object, desc);
    }

    pub(crate) fn apply_remote(&mut self, event: &SessionEvent) {
        if self.request_motion_desc(event) {
            return;
        }
        let impulse_source = match event {
            SessionEvent::ObjectAction {
                object,
                action: ObjectAction::AddTargetedImpulse { target, .. },
            } => self.impulse_source(*object, *target),
            _ => None,
        };
        // `CPhysicsObj::RecvEvt_DoBehavior`: a behaviour that names a weenie emote is asked of the
        // object's weenie first, and a no leaves the body alone.
        let emote_refused = match event {
            SessionEvent::ObjectAction {
                object,
                action: ObjectAction::DoBehavior(params),
            } if params.weenie_emote != 0 => {
                !self.answer_weenie_emote(*object, params.weenie_emote)
            }
            _ => false,
        };
        let (remotes, mut cx) = self.remote_context();
        match event {
            SessionEvent::ObjectAdded(id) => remotes.added(*id, &mut cx),
            SessionEvent::ObjectMoved {
                object,
                update,
                teleported,
            } => remotes.moved(*object, update, *teleported, &mut cx),
            SessionEvent::ObjectAction { object, action } => {
                let target = match action {
                    ac2_net::protocol::ObjectAction::DeParent(child) => child.id,
                    _ => *object,
                };
                remotes.action(target, &mut cx);
                if !emote_refused {
                    remotes.motion(*object, action, impulse_source);
                }
            }
            // `CPhysicsObj::RecvEvt_Contain`: out of the world, into the inventory.
            SessionEvent::ObjectContained(id) => remotes.action(*id, &mut cx),
            SessionEvent::ObjectRemoved(id) => remotes.removed(*id, cx.terrain),
            _ => {}
        }
        // The player's body takes its motion events too, and only it moves to (`RecvEvt_MoveTo`;
        // the session keeps the move-to stamp).
        if let SessionEvent::ObjectAction { object, action } = event {
            if let Some(player) = self
                .player
                .as_mut()
                .filter(|p| p.id == *object && !emote_refused)
            {
                let started = player.body.as_mut().is_some_and(|body| {
                    crate::motion::apply_owned(&mut body.physics, *object, action, impulse_source)
                });
                if let (true, ObjectAction::MoveTo { params, .. }) = (started, action) {
                    player.move_to_target = match params.target {
                        MoveTarget::Object(id) => Some(id),
                        MoveTarget::Position(_) => None,
                    };
                }
            }
        }
        self.request_remote_bodies();
    }

    /// `TargetManager` for the player's move-to an object: each physics quantum the target's
    /// position and size (`MoveToManager::HandleUpdateTarget`); a target no longer in the world
    /// is lost. Ended move-tos are let go: the weenie's move-to callback is not modelled.
    pub(crate) fn track_move_to_target(&mut self) {
        let Some(player) = self.player.as_mut() else {
            return;
        };
        let Some(body) = player.body.as_mut() else {
            return;
        };
        body.physics.take_move_to_results();
        let Some(id) = player.move_to_target else {
            return;
        };
        if !body.physics.is_moving_to() {
            player.move_to_target = None;
            return;
        }
        match self.remotes.target_info(id) {
            Some((position, radius, height)) => {
                body.physics.update_move_to_target(position, radius, height)
            }
            None => {
                body.physics.lose_move_to_target();
                player.move_to_target = None;
            }
        }
    }

    /// A Setup for other objects arrived.
    pub(crate) fn remote_body_loaded(
        &mut self,
        visual: Option<u32>,
        body: Result<BodyData, String>,
    ) {
        let (remotes, mut cx) = self.remote_context();
        remotes.body_loaded(visual, body, &mut cx);
    }

    /// `CObjCell::ReleaseDynamicObjects` / `InitDynamicObjects` for the blocks that left or
    /// joined the resident world: `CObjectMaint::ReleaseObjCell` / `InitObjCell`.
    pub(crate) fn object_cells(&mut self) {
        let changes = self.cells.take_changes();
        let (remotes, mut cx) = self.remote_context();
        for change in changes {
            match change {
                BlockChange::Released(block) => remotes.block_released(block, &mut cx),
                BlockChange::Resident(block) => remotes.block_resident(block, &mut cx),
            }
        }
    }

    /// Every object other than the player: the physics object's pose when it has one, else
    /// the server's last position (held and contained objects have neither and are left out).
    pub(crate) fn object_frames(&self) -> Vec<ObjectFrame> {
        let player_id = self.player.as_ref().map(|p| p.id);
        let links = self.session.objects().links();
        self.session
            .objects()
            .iter()
            .filter(|o| Some(o.id) != player_id)
            .filter_map(|o| {
                let remote = self.remotes.get(o.id);
                let body = remote.and_then(|r| r.body());
                // The holder and location are the physics child list's
                // (`CPhysicsObj::m_pParent`, `CHILDINFO::m_locationID`).
                let held = links
                    .parent_of(o.id)
                    .zip(links.info_of(o.id))
                    .map(|(id, info)| Holder {
                        id,
                        location: info.location,
                        orientation: o.orientation,
                        point: self.holding_point(id, info.location),
                    });
                let in_world = remote.is_some_and(|r| r.in_world());
                // Contained objects are in an inventory; only placed, held or in-world ones
                // are listed.
                if o.position.is_none() && !in_world && held.is_none() {
                    return None;
                }
                let (cell, origin, rotation, velocity, contact) = match (body, o.position) {
                    (Some(body), _) => {
                        let state = body.movement_state();
                        let (cell, origin, rotation) =
                            ac2_physics::drawn_place(body, self.render_interpolation);
                        (cell, origin, rotation, state.velocity, state.contact)
                    }
                    (None, Some(p)) => (p.cell, p.origin, p.rotation, [0.0; 3], false),
                    (None, None) => (0, [0.0; 3], [1.0, 0.0, 0.0, 0.0], [0.0; 3], false),
                };
                let weenie = o.qualities.weenie.as_ref();
                Some(ObjectFrame {
                    id: o.id,
                    visual: o.visual,
                    setup: o.setup,
                    name: o.name.clone(),
                    cell,
                    origin,
                    rotation,
                    heading_degrees: o.heading_degrees,
                    in_world,
                    lost_cell: remote.and_then(|r| r.lost_cell()),
                    active: remote.is_none_or(|r| r.active()),
                    velocity,
                    contact,
                    mode: o.mode,
                    animated_mode: body.map(|b| b.mode()),
                    motion: o.motion,
                    holder: held,
                    fx_follow: None,
                    container: weenie.and_then(|w| w.container),
                    wielder: weenie.and_then(|w| w.wielder),
                    appearance: o.appearance.clone(),
                    pose: body
                        .map(|b| ac2_physics::part_poses(b, self.pose_blend_time()))
                        .unwrap_or_default(),
                    anim_scale: body.map_or([1.0; 4], |b| b.draw_scale()),
                    adaptive_degrade: body.is_some_and(|b| b.is_mobile() && !b.is_static()),
                })
            })
            .chain(self.local_object_frames())
            .collect()
    }

    /// `PHYSICSOBJREP::GetHoldingLocation`: holding location `location` of the object `holder`,
    /// from the holder's loaded Setup. `None` while its body is not in, or the Setup lacks it.
    fn holding_point(&self, holder: u64, location: u32) -> Option<HoldPoint> {
        let visual = self.session.objects().get(holder)?.visual;
        let body = self.remotes.body_data(visual)?;
        ac2_physics::find_holding_point(&body.holding, location)
    }

    /// The objects the client made for FX (`CObjectMaint::CreateTemporaryObject`), once their body
    /// is in: they have no session entry, so they are drawn where their body stands.
    fn local_object_frames(&self) -> Vec<ObjectFrame> {
        self.remotes
            .local_ids()
            .filter_map(|id| {
                let remote = self.remotes.get(id)?;
                let body = remote.body()?;
                let (cell, origin, rotation) =
                    ac2_physics::drawn_place(body, self.render_interpolation);
                let state = body.movement_state();
                Some(ObjectFrame {
                    id,
                    visual: remote.visual(),
                    setup: None,
                    name: None,
                    cell,
                    origin,
                    rotation,
                    heading_degrees: None,
                    in_world: remote.in_world(),
                    lost_cell: None,
                    active: true,
                    velocity: state.velocity,
                    contact: state.contact,
                    mode: None,
                    animated_mode: Some(body.mode()),
                    motion: [0.0; 3],
                    holder: None,
                    fx_follow: remote.fx_follow(),
                    container: None,
                    wielder: None,
                    appearance: Default::default(),
                    pose: ac2_physics::part_poses(body, self.pose_blend_time()),
                    anim_scale: body.draw_scale(),
                    // Client-made objects have temporary ids (`IIDUtils::VerifyTemporary`).
                    adaptive_degrade: false,
                })
            })
            .collect()
    }
}
