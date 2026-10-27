//! The client's glue for the objects other than the player: the session's object events and
//! the resident landblocks drive [`Remotes`], and each frame lists them for the renderer.

use ac2_net::protocol::{MoveTarget, ObjectAction};
use ac2_weenie::SessionEvent;

use crate::cells::BlockChange;
use crate::client::Client;
use crate::frame::{Holder, ObjectFrame};
use crate::loader::Request;
use crate::remote::Context;
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
                terrain: &self.terrain,
                player,
            },
        )
    }

    /// Asks the loader threads for the Setups new objects need.
    fn request_remote_bodies(&mut self) {
        for visual in self.remotes.take_requests() {
            self.loader.request(Request::Body(visual));
        }
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

    pub(crate) fn apply_remote(&mut self, event: &SessionEvent) {
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
            SessionEvent::ObjectRemoved(id) => remotes.removed(*id),
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
        self.session
            .objects()
            .iter()
            .filter(|o| Some(o.id) != player_id)
            .filter_map(|o| {
                let remote = self.remotes.get(o.id);
                let body = remote.and_then(|r| r.body());
                let held = o.parent.map(|(id, location)| Holder {
                    id,
                    location,
                    orientation: o.orientation,
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
                            crate::motion::drawn_place(body, self.render_interpolation);
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
                    container: weenie.and_then(|w| w.container),
                    wielder: weenie.and_then(|w| w.wielder),
                    appearance: o.appearance.clone(),
                    pose: body
                        .map(|b| crate::motion::pose(b, self.pose_blend_time()))
                        .unwrap_or_default(),
                })
            })
            .collect()
    }
}
