//! The client's `TargetManager` of every object (`CPhysicsObj::m_pTargetManager`): the player's
//! move-to registers its target the way `MoveToManager::MoveToEvent` does, missiles register theirs
//! (`CMissileManager`), each quantum's `HandleTargetting` sends the voyeur updates, and
//! `CPhysicsObj::HandleTeleportStart` / `HandleLeaveWorld` let go of targets and tell the voyeurs.
//! The managers and their routing are [`TargetHub`]; here the object table is the player plus
//! [`Remotes`](crate::remote::Remotes).

use ac2_physics::TargetOwner;
use ac2_smartbox::remotes::TargetLog;
use ac2_smartbox::target_hub::TargetNotice;

use crate::client::Client;

/// `MoveToManager::MoveToEvent`'s `SetTarget(0, id, 0.5, 1.0)`: the radius a target must move
/// before the mover hears of it, and the seconds the mover looks ahead.
pub(crate) const MOVE_TO_TARGET_RADIUS: f32 = 0.5;
pub(crate) const MOVE_TO_TARGET_QUANTUM: f64 = 1.0;

impl Client {
    /// What `TargetManager` reads of the player's body (`None` until it exists).
    fn player_target_owner(&self) -> Option<TargetOwner> {
        let player = self.player.as_ref()?;
        let body = player.body.as_ref()?;
        let v = body.physics.movement_state().velocity;
        let scale = body.scale;
        Some(TargetOwner {
            id: player.id,
            position: body.physics.position(),
            velocity: ac2_physics::Vector::new(v[0], v[1], v[2]),
            radius: body.radius as f32,
            height: body.height as f32,
            scale: ac2_physics::Vector::new(scale, scale, scale),
        })
    }

    /// Runs `call` on the target hub with the object table (`CPhysicsObj::GetObjectA`: the player
    /// and the remotes in the world; `extra` is a body that just left it), then hands the notices
    /// about the player's own target to its move-to
    /// (`NoticeHandler::RecvNotice_CharacterSet` -> `CMotionInterp::UpdateTargetInfo`).
    fn with_targets(
        &mut self,
        extra: Option<TargetOwner>,
        call: impl FnOnce(
            &mut ac2_smartbox::TargetHub,
            ac2_smartbox::TargetOwners<'_>,
        ) -> Vec<TargetNotice>,
    ) {
        let player = self.player_target_owner();
        let remotes = &self.remotes;
        let owners = |id: u64| {
            if let Some(owner) = extra.filter(|o| o.id == id) {
                return Some(owner);
            }
            match player {
                Some(owner) if owner.id == id => Some(owner),
                _ => remotes.target_owner(id),
            }
        };
        let notices = call(&mut self.targets, &owners);
        let Some(player) = self.player.as_mut() else {
            return;
        };
        for notice in notices {
            if notice.object == player.id {
                if let Some(body) = player.body.as_mut() {
                    body.physics.update_target_info(&notice.info);
                }
            }
        }
    }

    /// `MoveToManager::MoveToEvent`'s `SetTarget(0, target, 0.5, 1.0)` for a move-to the server
    /// started on the player (an object target; a position target lets go of the old one, as
    /// `SetTarget` with `INVALID_IID` does).
    pub(crate) fn start_move_to_target(&mut self, target: Option<u64>) {
        let Some(player) = self.player.as_ref().map(|p| p.id) else {
            return;
        };
        let now = self.session.cur_time();
        match target {
            Some(id) => self.with_targets(None, |hub, owners| {
                hub.set_target(
                    player,
                    0,
                    id,
                    MOVE_TO_TARGET_RADIUS,
                    MOVE_TO_TARGET_QUANTUM,
                    now,
                    owners,
                )
            }),
            None => self.with_targets(None, |hub, owners| hub.clear_target(player, now, owners)),
        }
    }

    /// `MoveToManager::UpdateMoveTo`'s read of its object target for one quantum: the target's
    /// top-level object (`MoveToManager::GetTargetObj`) gives its position and size while it is
    /// in the world; without it the move-to keeps what the target manager last told it, and a
    /// status other than `Ok` (the target left the world or teleported, or said nothing for ten
    /// seconds) ends it. An ended move-to lets go of its target (`MoveToManager::Stop`'s
    /// `ClearTarget`). Then `TargetManager::HandleTargetting` for every object.
    pub(crate) fn track_move_to_target(&mut self) {
        let now = self.session.cur_time();
        let mut clear = None;
        if let Some(player) = self.player.as_mut() {
            let id = player.id;
            if let Some(body) = player.body.as_mut() {
                body.physics.take_move_to_results();
                if let Some(target) = player.move_to_target {
                    if !body.physics.is_moving_to() {
                        player.move_to_target = None;
                        clear = Some(id);
                    } else {
                        let top = self
                            .session
                            .objects()
                            .links()
                            .top_parent(target)
                            .unwrap_or(target);
                        if let Some((position, radius, height)) = self.remotes.target_info(top) {
                            body.physics.update_move_to_target(position, radius, height);
                        }
                    }
                }
            }
        }
        if let Some(id) = clear {
            self.with_targets(None, |hub, owners| hub.clear_target(id, now, owners));
        }
        self.with_targets(None, |hub, owners| hub.handle_targetting(now, owners));
    }

    /// `CPhysicsObj::HandleTeleportStart`'s `TargetManager` calls on the player: `ClearTarget`,
    /// then every voyeur is told `Teleported`.
    pub(crate) fn player_teleport_start_targets(&mut self) {
        let Some(player) = self.player.as_ref().map(|p| p.id) else {
            return;
        };
        let now = self.session.cur_time();
        self.with_targets(None, |hub, owners| hub.teleport_start(player, now, owners));
    }

    /// `CPhysicsObj::HandleLeaveWorld` on the player: the body's motion part and transient
    /// states, the `TargetManager` (`ClearTarget`, voyeurs told `ExitWorld`) and
    /// `EndAllCollisions` (its report goes to the player's script with the next quantum's).
    pub(crate) fn player_handle_leave_world(&mut self) {
        let now = self.session.cur_time();
        let Some(owner) = self.player_target_owner() else {
            return;
        };
        let report = self
            .player
            .as_mut()
            .and_then(|p| p.body.as_mut())
            .and_then(|body| {
                body.physics.leave_world();
                let materials = body.physics.contact_materials(&self.terrain);
                body.physics.end_all_collisions(&materials)
            });
        if let Some(report) = report {
            self.pending_collision_reports.push((owner.id, report));
        }
        self.with_targets(Some(owner), |hub, owners| {
            hub.leave_world(owner.id, now, owners)
        });
    }

    /// What the remotes asked of the target managers since the last call, in order: an object
    /// that left the world (`HandleLeaveWorld`: its manager's `HandleLeaveWorld` and, while the
    /// object exists, `EndAllCollisions`), one that was destroyed (its manager goes), and a
    /// missile's `SetTarget`.
    pub(crate) fn apply_target_log(&mut self) {
        let now = self.session.cur_time();
        for entry in self.remotes.take_target_log() {
            match entry {
                TargetLog::Left(left) => {
                    let owner = self.remotes.left_owner(&left);
                    self.with_targets(Some(owner), |hub, owners| {
                        hub.leave_world(owner.id, now, owners)
                    });
                    self.remotes.end_all_collisions(owner.id, &self.terrain);
                }
                TargetLog::Gone(id) => self.targets.remove_object(id),
                TargetLog::SetTarget {
                    owner,
                    object_id,
                    radius,
                    quantum,
                } => self.with_targets(None, |hub, owners| {
                    hub.set_target(owner, 0, object_id, radius, quantum, now, owners)
                }),
            }
        }
    }
}

#[cfg(feature = "test-support")]
impl Client {
    /// The object `object`'s target manager has as its target (`CPhysicsObj::GetTargetID`; `None`
    /// without one). For tests of the target wiring.
    pub fn target_of(&self, object: u64) -> Option<u64> {
        let id = self.targets.target_id(object);
        (id != 0).then_some(id)
    }

    /// The objects watching `object` (`TargetManager::m_voyeur_table`), in id order.
    pub fn voyeurs_of(&self, object: u64) -> Vec<u64> {
        self.targets
            .manager(object)
            .map(|m| m.voyeurs().map(|v| v.object_id).collect())
            .unwrap_or_default()
    }

    /// `CMotionInterp::IsMovingTo` on the player's body.
    pub fn player_moving_to(&self) -> bool {
        self.player
            .as_ref()
            .and_then(|p| p.body.as_ref())
            .is_some_and(|b| b.physics.is_moving_to())
    }
}
