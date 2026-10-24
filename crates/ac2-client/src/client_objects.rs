//! The client's glue for the objects other than the player: the session's object events and
//! the resident landblocks drive [`Remotes`], and each frame lists them for the renderer.

use ac2_weenie::SessionEvent;

use crate::cells::BlockChange;
use crate::client::Client;
use crate::frame::ObjectFrame;
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
    pub(crate) fn apply_remote(&mut self, event: &SessionEvent) {
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
            }
            SessionEvent::ObjectRemoved(id) => remotes.removed(*id),
            _ => {}
        }
        self.request_remote_bodies();
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
                let (cell, origin, rotation, velocity, contact) = match body {
                    Some(body) => {
                        let state = body.movement_state();
                        (
                            state.cell_id,
                            state.origin,
                            state.rotation,
                            state.velocity,
                            state.contact,
                        )
                    }
                    None => {
                        let p = o.position?;
                        (p.cell, p.origin, p.rotation, [0.0; 3], false)
                    }
                };
                if o.position.is_none() && !remote.is_some_and(|r| r.in_world()) {
                    return None;
                }
                Some(ObjectFrame {
                    id: o.id,
                    visual: o.visual,
                    setup: o.setup,
                    name: o.name.clone(),
                    cell,
                    origin,
                    rotation,
                    heading_degrees: o.heading_degrees,
                    in_world: remote.is_some_and(|r| r.in_world()),
                    lost_cell: remote.and_then(|r| r.lost_cell()),
                    active: remote.is_none_or(|r| r.active()),
                    velocity,
                    contact,
                    mode: o.mode,
                    motion: o.motion,
                    parent: o.parent,
                })
            })
            .collect()
    }
}
