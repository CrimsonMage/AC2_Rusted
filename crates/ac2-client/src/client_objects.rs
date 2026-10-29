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

    /// The physics body of the weenie `object` a script acted on: the player's, or a remote
    /// object's (`CWPhysicsObject::m_phys_obj` is any weenie's body).
    fn script_body_mut(&mut self, object: u64) -> Option<&mut ac2_physics::PhysicsBody> {
        match self.player.as_mut().filter(|p| p.id == object) {
            Some(player) => player.body.as_mut().map(|b| &mut b.physics),
            None => self.remotes.body_mut(object),
        }
    }

    /// A body state change a script made (`CPhysicsObj::set_movement_frozen`,
    /// `set_animation_frozen`, `set_edge_slide`): the weenie's body takes it, the player's or a
    /// remote object's.
    pub(crate) fn apply_body_state(
        &mut self,
        object: u64,
        state: ac2_script::host::body_state::BodyState,
    ) {
        use ac2_script::host::body_state::BodyState;
        let Some(physics) = self.script_body_mut(object) else {
            return;
        };
        match state {
            BodyState::MovementFrozen(frozen) => physics.set_movement_frozen(frozen),
            BodyState::AnimationFrozen(frozen) => physics.set_animation_frozen(frozen),
            BodyState::EdgeSlide(slide) => physics.set_edge_slide(slide),
        }
    }

    /// `CCollisionManager::SetCollisionState` for a script's `CollisionParameters`: the weenie's
    /// body (the player's or a remote object's) takes the non-zero impact minimums.
    pub(crate) fn apply_collision_minimums(&mut self, object: u64, xy: f32, z: f32) {
        if let Some(physics) = self.script_body_mut(object) {
            physics.set_collision_minimums(xy, z);
        }
    }

    /// `CCollisionManager::SetCollisionState` for a script's collision sets: the weenie's body
    /// (the player's or a remote object's) merges the script's placement and movement ethereal
    /// and physics-type sets (only the sets its bits name).
    pub(crate) fn apply_collision_sets(
        &mut self,
        object: u64,
        sets: &ac2_physics::CollisionParameters,
    ) {
        if self.player.as_ref().is_some_and(|p| p.id == object) {
            self.apply_player_collision_sets(object, sets);
            return;
        }
        if !self.remotes.tracks(object) {
            return;
        }
        // The other bodies `GetObjectA` can answer for, taken before the subject is borrowed.
        let mut peers: std::collections::HashMap<u64, ac2_physics::ConflictPeer> = self
            .remotes
            .iter()
            .filter(|(id, _)| *id != object)
            .filter_map(|(id, remote)| {
                let body = remote.body()?;
                Some((
                    id,
                    ac2_physics::ConflictPeer {
                        id,
                        ..body.conflict_peer()
                    },
                ))
            })
            .collect();
        if let Some((id, body)) = self
            .player
            .as_ref()
            .and_then(|p| Some((p.id, p.body.as_ref()?)))
        {
            peers.insert(
                id,
                ac2_physics::ConflictPeer {
                    id,
                    ..body.physics.conflict_peer()
                },
            );
        }
        let peer = |id: u64| peers.get(&id).cloned();
        let Some(physics) = self.remotes.body_mut(object) else {
            return;
        };
        physics.set_instance_id(object);
        let effects = physics.set_collision_state(sets, &peer);
        for effect in effects {
            if let Some(other) = self.remotes.body_mut(effect.object) {
                other.add_to_temp_ethereal_list(effect.add);
            } else if let Some(player) = self
                .player
                .as_mut()
                .filter(|p| p.id == effect.object)
                .and_then(|p| p.body.as_mut())
            {
                player.physics.add_to_temp_ethereal_list(effect.add);
            }
        }
    }

    fn apply_player_collision_sets(
        &mut self,
        object: u64,
        sets: &ac2_physics::CollisionParameters,
    ) {
        let remotes = &self.remotes;
        let peer = |id: u64| {
            remotes
                .get(id)
                .and_then(|r| r.body())
                .map(|body| ac2_physics::ConflictPeer {
                    id,
                    ..body.conflict_peer()
                })
        };
        let Some(body) = self
            .player
            .as_mut()
            .filter(|p| p.id == object)
            .and_then(|p| p.body.as_mut())
        else {
            return;
        };
        // `CCollisionManager::SetCollisionState`, then `UpdateConflictList`: the objects that
        // stopped being passable are ignored by both bodies from now on.
        body.physics.set_instance_id(object);
        let effects = body.physics.set_collision_state(sets, &peer);
        for effect in effects {
            if let Some(other) = self.remotes.body_mut(effect.object) {
                other.add_to_temp_ethereal_list(effect.add);
            }
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
        // `CPhysicsObj::RecvEvt_InitializeMissile`.
        if let SessionEvent::ObjectAction {
            object,
            action:
                ObjectAction::InitializeMissile {
                    velocity, target, ..
                },
        } = event
        {
            let target = crate::motion::target_position(target);
            if let Some(body) = self.remotes.body_mut(*object) {
                body.recv_initialize_missile(&self.terrain, *velocity, &target);
            }
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
        let tables = self.session.player_script().string_tables();
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
                        // A bone-index child (`StickySetParent`) rides the bone alone.
                        point: match info.kind {
                            ac2_physics::ChildType::BoneIndex => Some(HoldPoint {
                                group: 0,
                                bone: info.location as usize,
                                transform: ac2_anim::Ac2Transform::IDENTITY,
                            }),
                            _ => self.holding_point(id, info.location),
                        },
                        offset: info.offset,
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
                    stack_name: ac2_weenie::ItemStats::stack_name(&o.qualities, tables),
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
    pub(crate) fn holding_point(&self, holder: u64, location: u32) -> Option<HoldPoint> {
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
                    stack_name: None,
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
                    // A story missile hangs from its attacker until it fires (`SetParent`), and
                    // from the bone it sticks to (`StickySetParent`).
                    holder: self
                        .stories
                        .missile_holder(id)
                        .map(|(holder, info)| Holder {
                            id: holder,
                            location: info.location,
                            orientation: None,
                            point: match info.kind {
                                ac2_physics::ChildType::BoneIndex => Some(HoldPoint {
                                    group: 0,
                                    bone: info.location as usize,
                                    transform: ac2_anim::Ac2Transform::IDENTITY,
                                }),
                                _ => self.holding_point(holder, info.location),
                            },
                            offset: info.offset,
                        }),
                    fx_follow: remote.fx_follow(),
                    container: None,
                    wielder: None,
                    // A cloned story missile looks like its weapon.
                    appearance: self.stories.missile_appearance(id).unwrap_or_default(),
                    pose: ac2_physics::part_poses(body, self.pose_blend_time()),
                    anim_scale: body.draw_scale(),
                    // Client-made objects have temporary ids (`IIDUtils::VerifyTemporary`).
                    adaptive_degrade: false,
                })
            })
            .collect()
    }
}

impl Client {
    /// What the missiles' bodies asked of the world in this quantum. A streak starts or stops on
    /// the object (`CPhysicsObj::StartStreak` / `StopStreak`). `CPhysicsObj::DeleteSelf` fades the
    /// body and has the object maintenance delete it (`CObjectMaint::FadeAndDestroyObject`).
    /// The weenie's callbacks (slots `0x80`, `0x84`, `0x8c`) are `CWeenieObject`'s defaults in the
    /// client image, which return at once, so there is nothing to run. A missile that sticks
    /// (`StickySetParent`) hangs from the bone it hit ([`Client::stick_missile`]).
    pub(crate) fn missile_events_taken(&mut self, events: &[(u64, ac2_physics::MissileEvent)]) {
        let follow = self.remotes.follow_missiles(events);
        for (missile, object) in follow.stick {
            self.stick_missile(missile, object);
        }
        self.pending_streaks.extend(follow.streaks);
        for (id, seconds) in follow.destroy {
            self.session.fade_and_destroy_object(id, seconds);
        }
    }

    /// `CPhysicsObj::StickySetParent(object)` on `missile`: it hangs from the nearest bone of
    /// the object's root part array (`CalculateBoneIndexAndOffset`) as a bone-index child, leaves
    /// the world, and is no longer queued for destruction. The view then places it on that bone
    /// of the object's posed skeleton every frame (`CPhysicsObj::UpdateChild`).
    fn stick_missile(&mut self, missile: u64, object: u64) {
        let Some(at) = self
            .remotes
            .get(missile)
            .and_then(|r| r.body())
            .map(|b| b.position())
        else {
            return;
        };
        let holder = match self.player.as_ref().filter(|p| p.id == object) {
            Some(player) => player.body.as_ref().map(|b| &b.physics),
            None => self.remotes.get(object).and_then(|r| r.body()),
        };
        let Some((bone, offset)) = holder.and_then(|body| body.sticky_attachment(&at)) else {
            return;
        };
        // A story missile is the client's own and has no session entry: the client keeps its
        // parent.
        if self.stories.stick(missile, object, bone, offset)
            || self.session.stick_object(missile, object, bone, offset)
        {
            self.remotes.leave_world_for_parent(missile);
        }
    }

    /// `CPhysicsObj::m_posDrawing` of each missile that `HasParent`: where the holder's holding
    /// location puts it, which `CMissileManager::InitPosition` starts the flight from. The child
    /// graph is the session's object table; a missile without a parent has none. The story
    /// missiles the client made follow ([`Client::story_missiles_quantum`]).
    pub(crate) fn missile_parent_positions(&mut self) {
        let launched: Vec<u64> = self
            .remotes
            .iter()
            .filter(|(_, r)| r.body().is_some_and(|b| b.has_missile_manager()))
            .map(|(id, _)| id)
            .collect();
        for id in launched {
            let links = self.session.objects().links();
            let placed = links
                .parent_of(id)
                .zip(links.info_of(id))
                .and_then(|(holder, info)| self.held_drawing_position(holder, info));
            if let Some(body) = self.remotes.body_mut(id) {
                body.set_parent_drawing_position(placed);
            }
        }
        self.story_missiles_quantum();
    }

    /// Where `holder`'s child `info` is drawn (`CPhysicsObj::UpdateChild` on the holder's drawing
    /// position): `None` while the holder has no body.
    pub(crate) fn held_drawing_position(
        &self,
        holder: u64,
        info: &ac2_physics::ChildInfo,
    ) -> Option<ac2_physics::Position> {
        let holder_body = match self.player.as_ref().filter(|p| p.id == holder) {
            Some(player) => &player.body.as_ref()?.physics,
            None => self.remotes.get(holder)?.body()?,
        };
        let position = holder_body.drawing_position(self.render_interpolation);
        let point = self.holding_point(holder, info.location);
        // The bone the child rides, posed: a bone-index child's own bone, a holding location's
        // bone in the root group (`CPartArray::GetHoldingLocation` composes it with the bone's
        // matrix). Identity while the holder's parts are not in.
        let bone = match info.kind {
            ac2_physics::ChildType::BoneIndex => holder_body.bone_transform(info.location),
            _ => point
                .as_ref()
                .filter(|p| p.group == 0)
                .and_then(|p| holder_body.bone_transform(p.bone as u32)),
        }
        .unwrap_or(ac2_anim::Ac2Transform::IDENTITY);
        ac2_physics::parent_drawing_position(
            &position,
            holder_body.object_view().scale,
            info,
            bone,
            point.as_ref(),
        )
    }
}
