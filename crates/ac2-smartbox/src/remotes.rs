//! The objects other than the player that the world keeps resident: `CObjectMaint`'s object
//! table, its lost-cell table (`CBookKeep::m_lostCellID`), the Setup bodies the objects share, and
//! each object's entry in the world's object lists (`CObjCell::AddObject`).
//!
//! This is the session-free half of the residency. The client feeds it plain data (positions,
//! cell ids, flags, loaded bodies) and applies the session's destroy queue itself; the session
//! reads (`RemoteObject`) and the motion conversion stay with the client.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use ac2_anim::{Ac2AnimEvent, FxScripts};
use ac2_core::Random;
use ac2_physics::{
    CellSource, ConflictPeer, FxFollow, LookTarget, ObjectShape, PhysicsBody, Position,
    SetPositionError, StaticObject, TargetOwner, Terrain, Vector,
};
use ac2_world::Ac2MotionInterpDesc;

use crate::collision_traffic::Touch;
use crate::membership::CellMembership;
use crate::remote_membership::sync_membership;
use crate::BodyData;

/// `FXManager::DoCreateObjectFX`'s `CollisionParameters` bit for an FX object.
pub const FX_OBJECT_COLLISION_BIT: u32 = 0x10;

/// `CPhysicsObj::RecvEvt_Position`: beyond this distance from the player (metres) an object is
/// out of physics range (`HandleOutOfPhysicsRange`) and snaps to its positions.
const PHYSICS_RANGE: f32 = 96.0;

/// `CPhysicsObj::SyncPositionSimple`'s placement flags.
pub const SYNC_POSITION_FLAGS: u32 = 0x1010;

/// `CPhysicsObj::EnterWorldSimple`'s placement flags.
pub const ENTER_WORLD_FLAGS: u32 = 0x11;

/// The session facts a residency call reads about one object (`RemoteObject`).
#[derive(Clone, Copy, Debug)]
pub struct ObjectRecord {
    /// The object's position in the server's table; `None` when it has none.
    pub position: Option<Position>,
    /// The object has a parent: it is held or contained.
    pub held: bool,
    /// The visual its appearance comes from (`RemoteObject::visual`).
    pub visual: Option<u32>,
    /// The `EntityDesc` DID from its create (`RemoteObject::entity`).
    pub entity: Option<u32>,
}

/// What a received position did (`CPhysicsObj::RecvEvt_Position` for an object other than the
/// player): the residency placed it, or the body interpolates to it.
#[derive(Debug, PartialEq, Eq)]
pub enum MoveOutcome {
    /// The body walks to the position (`InterpolateTo`); the client makes that call.
    Interpolate,
    /// The residency placed the object (or only stored its position); these session changes
    /// follow.
    Placed(Vec<ResidencyEffect>),
}

/// A client-made object (`CObjectMaint::CreateTemporaryObject`), such as an FX's, with no
/// session entry.
#[derive(Clone, Copy, Debug)]
pub struct LocalObject {
    pub visual: u32,
    pub position: Position,
    pub fx_follow: Option<FxFollow>,
    /// The alpha moves an FX object makes once its body exists (`DoCreateObjectFX`'s sliders).
    pub alpha_moves: [(f32, f32); 2],
}

/// The session's view of the objects, by id: `None` for an object the session does not have.
pub type Records<'a> = &'a dyn Fn(u64) -> Option<ObjectRecord>;

/// A session change a residency call asks for. The client applies them, in order, once the call
/// returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidencyEffect {
    /// `CBookKeep::AddObjectToBeDestroyed` (`CPhysicsObj::LeaveWorld`).
    QueueDestroy(u64),
    /// `CBookKeep::RemoveObjectToBeDestroyed` (`CBookKeep::HandleEnterWorld`).
    CancelDestroy(u64),
}

/// One object other than the player.
#[derive(Debug)]
pub struct Remote {
    /// The visual its Setup comes from.
    pub visual: Option<u32>,
    /// Built once its Setup has loaded (`CObjectMaint::FinishEntityCreation`).
    pub body: Option<PhysicsBody>,
    /// The Setup failed to load: the object is shown at its server position only.
    pub failed: bool,
    /// `get_position` until the body exists (`CPhysicsObj::StorePosition`).
    pub stored: Option<Position>,
    /// `CPhysicsObj::IsInWorld`.
    pub in_world: bool,
    /// `CBookKeep::m_lostCellID`.
    pub lost_cell: Option<u32>,
    /// `!(m_transient_state & 0x2000)`: within physics range of the player.
    pub active: bool,
    /// `CPhysicsObj::m_fPlayerDistance`.
    pub player_distance: f32,
    /// The Setup's collision shape and total scale, once the body exists (`CPartArray` of the
    /// visual); `None` when the Setup has no physics shape.
    pub object: Option<(Arc<ObjectShape>, Vector)>,
    /// The body's entry in the world's object lists (`CObjCell::AddObject`), while it is in the
    /// world with a shape.
    pub shadow: Option<u32>,
    /// The cell whose object list holds the object (`CObjCell::m_objectList`), while listed.
    pub listed_cell: Option<u32>,
    /// An FX object: ethereal to every mover (`FXManager::DoCreateObjectFX`).
    pub ethereal: bool,
    /// The alpha moves an FX object makes once its body exists (`DoCreateObjectFX`'s sliders).
    pub alpha_moves: Option<[(f32, f32); 2]>,
    /// The emitter and attach point of an FX object that hangs from one (`FXSetParent`).
    pub fx_follow: Option<FxFollow>,
}

/// `CPhysicsObj::add_to_cells` / `remove_from_cells` and `CObjCell::AddObject` /
/// `RemoveObject` for one remote: while the object is in the world with a body and a shape, its
/// entry in the world's object lists is at its body's position; otherwise it is not listed.
fn sync_shadow(
    guid: u64,
    remote: &mut Remote,
    terrain: &mut Terrain,
    membership: &mut CellMembership,
) {
    sync_membership(guid, remote, membership);
    // The entry's blocking mask is the body's (`CPhysicsObj::IsBlocking` reads m_blocking_mask),
    // so it is re-read on every sync: a body's bits can change between syncs.
    // The Setup's root scale times the body's own (`CPhysicsObj::SetScale`, which a script's
    // `CWRepository::SetScale` changes at run time): a new scale lists the object again.
    let wanted = match (&remote.body, &remote.object, remote.in_world) {
        (Some(body), Some((shape, scale)), true) => {
            let own = body.effects_scale();
            let times = |a: f32, b: f32| (f64::from(a) * f64::from(b)) as f32;
            Some((
                body.position(),
                Arc::clone(shape),
                Vector::new(
                    times(scale.x, own.x),
                    times(scale.y, own.y),
                    times(scale.z, own.z),
                ),
                body.is_mobile(),
                body.blocking_bits(),
            ))
        }
        _ => None,
    };
    if let (Some((_, _, scale, _, _)), Some(id)) = (&wanted, remote.shadow) {
        if terrain
            .object_scale(id)
            .is_some_and(|listed| listed != *scale)
        {
            terrain.remove_object(id);
            remote.shadow = None;
        }
    }
    match (wanted, remote.shadow) {
        (Some((position, shape, scale, mobile, blocking)), None) => {
            // The object is named by its guid, so a collision report lists it.
            let mut object = StaticObject::dynamic(position, scale, shape, mobile)
                .with_blocking_mask(blocking)
                .with_instance_id(guid);
            object.set_ethereal(remote.ethereal);
            let id = terrain.insert_object(object);
            if let Some(body) = &remote.body {
                terrain.set_object_ignores_collisions(id, body.ignores_collisions());
                terrain.set_object_collision(
                    id,
                    body.collision_state_ptr(),
                    body.current_properties(),
                );
            }
            remote.shadow = Some(id);
            if let Some(body) = remote.body.as_mut() {
                body.set_world_id(Some(id));
            }
        }
        (Some((position, _, _, _, blocking)), Some(id)) => {
            terrain.move_object(id, position);
            terrain.set_object_blocking(id, blocking);
            if let Some(body) = &remote.body {
                terrain.set_object_ignores_collisions(id, body.ignores_collisions());
                terrain.set_object_collision(
                    id,
                    body.collision_state_ptr(),
                    body.current_properties(),
                );
            }
        }
        (None, Some(id)) => {
            terrain.remove_object(id);
            remote.shadow = None;
            if let Some(body) = remote.body.as_mut() {
                body.set_world_id(None);
            }
        }
        (None, None) => {}
    }
}

impl Remote {
    pub fn position(&self) -> Option<Position> {
        self.body
            .as_ref()
            .map(PhysicsBody::position)
            .or(self.stored)
    }

    /// The visual the object was created with.
    pub fn visual(&self) -> Option<u32> {
        self.visual
    }

    pub fn in_world(&self) -> bool {
        self.in_world
    }

    pub fn body(&self) -> Option<&PhysicsBody> {
        self.body.as_ref()
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn lost_cell(&self) -> Option<u32> {
        self.lost_cell
    }

    pub fn fx_follow(&self) -> Option<FxFollow> {
        self.fx_follow
    }
}

/// `CObjCell::GetVisible(cell, 1)`: the cell is resident.
pub fn visible(terrain: &Terrain, cell: u32) -> bool {
    terrain.land_cell(cell).is_some() || terrain.env_cell(cell).is_some()
}

/// An `EntityDesc` DID (`DivineType` `0x40000049`, the `0x47` prefix of its DIDs).
pub fn is_entity_desc(did: u32) -> bool {
    did & 0xFF00_0000 == 0x4700_0000
}

/// A body's look-at snapshot: where it is and how big (`GetPosition`, `GetHeight`, `is_mobile`).
pub fn look_snapshot(body: &PhysicsBody) -> LookTarget {
    LookTarget {
        position: body.position(),
        height: body.height(),
        mobile: body.is_mobile(),
    }
}

/// The residency state of the objects other than the player. `B` is the loaded body payload of
/// a visual; the residency reads only its [`BodyData`] (`AsRef`), so the client's richer payload
/// (FX and holding data) rides along.
pub struct Remotes<B> {
    objects: BTreeMap<u64, Remote>,
    /// The objects the client made itself (an FX's objects), with no session entry.
    local: BTreeSet<u64>,
    /// `CObjectMaint::lost_cell_table`: each lost cell's objects, in arrival order.
    lost_cells: BTreeMap<u32, Vec<u64>>,
    /// Loaded Setups by visual, and the visuals asked for.
    bodies: HashMap<Option<u32>, Result<B, String>>,
    requested: BTreeSet<Option<u32>>,
    /// Each object's `EntityDesc` DID from its create (the `CObjectMaint` entity), and whether
    /// its ethereal bits were asked for.
    entities: BTreeMap<u64, u32>,
    entity_requested: BTreeSet<u64>,
    /// The ethereal bits each object's `EntityDesc` set, applied when its body is made.
    masks: HashMap<u64, u64>,
    /// The cells' object lists and watchers (`CObjCell::AddObject`, `AddVoyeur`).
    membership: CellMembership,
    /// The client-made objects' ids (`CObjectMaint::m_used_temporary_ids`).
    temporary_ids: crate::temporary_ids::TemporaryIds,
    /// What the remotes' collisions left for the client to apply (see `collision_traffic`).
    traffic: crate::collision_traffic::CollisionTraffic,
}

impl<B> Default for Remotes<B> {
    fn default() -> Self {
        Self {
            objects: BTreeMap::new(),
            local: BTreeSet::new(),
            lost_cells: BTreeMap::new(),
            bodies: HashMap::new(),
            requested: BTreeSet::new(),
            entities: BTreeMap::new(),
            entity_requested: BTreeSet::new(),
            masks: HashMap::new(),
            membership: CellMembership::new(),
            temporary_ids: Default::default(),
            traffic: Default::default(),
        }
    }
}

impl<B> Remotes<B> {
    pub fn get(&self, id: u64) -> Option<&Remote> {
        self.objects.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Remote> {
        self.objects.get_mut(&id)
    }

    /// The body of object `id`, when it has one.
    pub fn body_mut(&mut self, id: u64) -> Option<&mut PhysicsBody> {
        self.objects.get_mut(&id).and_then(|r| r.body.as_mut())
    }

    /// Whether `id` is a remote object the client still tracks.
    pub fn tracks(&self, id: u64) -> bool {
        self.objects.contains_key(&id)
    }

    /// Every tracked object, in id order.
    pub fn iter(&self) -> impl Iterator<Item = (u64, &Remote)> + '_ {
        self.objects.iter().map(|(&id, remote)| (id, remote))
    }

    /// Starts tracking `id` (`CObjectMaint`'s new entry).
    pub fn insert(&mut self, id: u64, remote: Remote) {
        self.objects.insert(id, remote);
    }

    /// Starts tracking `id` as a client-made object (`CObjectMaint::CreateTemporaryObject`).
    pub fn insert_local(&mut self, id: u64, remote: Remote) {
        self.objects.insert(id, remote);
        self.local.insert(id);
    }

    /// The client-made objects, in id order.
    pub fn local_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.local.iter().copied()
    }

    /// Whether `id` is a client-made object.
    pub fn is_local(&self, id: u64) -> bool {
        self.local.contains(&id)
    }

    /// The `EntityDesc`s of new objects to read their ethereal bits from (each once).
    pub fn take_entity_requests(&mut self) -> Vec<(u64, u32)> {
        let wanted: Vec<_> = self
            .entities
            .iter()
            .filter(|(id, _)| !self.entity_requested.contains(id))
            .map(|(&id, &did)| (id, did))
            .collect();
        self.entity_requested
            .extend(wanted.iter().map(|&(id, _)| id));
        wanted
    }

    /// The object's `EntityDesc` DID from its create.
    pub fn record_entity(&mut self, id: u64, did: u32) {
        self.entities.insert(id, did);
    }

    /// The object's `EntityDesc` ethereal bits arrived: its body takes them now, or when made.
    pub fn entity_mask_loaded(&mut self, object: u64, bits: u64) {
        self.masks.insert(object, bits);
        if let Some(body) = self.body_mut(object) {
            body.apply_blocking_bits(bits);
        }
    }

    /// The ethereal bits an `EntityDesc` set for `object`, when they arrived.
    pub fn entity_mask(&self, object: u64) -> Option<u64> {
        self.masks.get(&object).copied()
    }

    /// The visuals to load (each once).
    pub fn take_requests(&mut self) -> Vec<Option<u32>> {
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

    /// A Setup arrived from a loader thread: stored for `visual`. Returns the objects waiting on
    /// it (no body yet, not failed), in id order, to create now.
    pub fn store_body(&mut self, visual: Option<u32>, body: Result<B, String>) -> Vec<u64> {
        self.bodies.insert(visual, body);
        self.objects
            .iter()
            .filter(|(_, r)| r.visual == visual && r.body.is_none() && !r.failed)
            .map(|(&id, _)| id)
            .collect()
    }

    /// The loaded data of `visual`.
    pub fn body_data(&self, visual: Option<u32>) -> Option<&B> {
        self.bodies.get(&visual)?.as_ref().ok()
    }

    /// The loaded Setup of `visual`, failed or not, when its load finished.
    pub fn body_result(&self, visual: Option<u32>) -> Option<&Result<B, String>> {
        self.bodies.get(&visual)
    }

    /// The remote objects whose body exists, with their visual (`CPhysicsObj` created).
    pub fn bodied(&self) -> impl Iterator<Item = (u64, Option<u32>)> + '_ {
        self.objects
            .iter()
            .filter(|(_, remote)| remote.body.is_some())
            .map(|(id, remote)| (*id, remote.visual))
    }

    /// `CObjectMaint::RemoveFromLostCell`.
    pub fn remove_from_lost_cell(&mut self, id: u64) {
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

    /// `CBookKeep::GotoLostCell`'s table entry: `id` waits in lost cell `cell`, in arrival order.
    pub fn push_lost_cell(&mut self, cell: u32, id: u64) {
        self.lost_cells.entry(cell).or_default().push(id);
    }

    /// The lost cells in `block`, in cell id order (`CObjectMaint::InitObjCell`'s walk).
    pub fn lost_cells_in(&self, block: u32) -> Vec<u32> {
        self.lost_cells
            .keys()
            .copied()
            .filter(|cell| cell & 0xFFFF_0000 == block)
            .collect()
    }

    /// Removes lost cell `cell` and returns its objects, in arrival order.
    pub fn take_lost_cell(&mut self, cell: u32) -> Option<Vec<u64>> {
        self.lost_cells.remove(&cell)
    }

    /// The cells' object lists and watchers.
    pub fn membership(&mut self) -> &mut CellMembership {
        &mut self.membership
    }

    /// `CObjectMaint::DestroyObjectInternal` removed the object: its entry leaves the world's
    /// object lists.
    pub fn removed(&mut self, id: u64, terrain: &mut Terrain) {
        self.remove_from_lost_cell(id);
        if let Some(remote) = self.objects.remove(&id) {
            if let Some(cell) = remote.listed_cell {
                self.membership.remove_object(cell, id);
            }
            self.membership.forget_object(id);
            if let Some(shadow) = remote.shadow {
                terrain.remove_object(shadow);
            }
        }
        if self.local.remove(&id) {
            // `CObjectMaint::DeallocateTemporaryID`.
            self.temporary_ids.deallocate(id);
        }
    }

    /// The tail of `CPhysicsObj::DoMode`: each child body takes the mode (`DoMode(0, mode)`).
    pub fn children_do_mode(&mut self, children: &[u64], mode: u32) {
        for &child in children {
            if let Some(body) = self.body_mut(child) {
                body.follow_holder_mode(mode);
            }
        }
    }

    /// `CPhysicsObj::DoBehavior`'s `mPropagate`: each child body runs the behaviour at the same
    /// time scale, without locking actions and with its own children fading.
    pub fn children_do_behavior(&mut self, children: &[u64], behavior: u32, time_scale: f32) {
        for &child in children {
            if let Some(body) = self.body_mut(child) {
                body.follow_holder_behavior(behavior, time_scale);
            }
        }
    }

    /// `CObjectMaint::AllocateTemporaryID`: an id for an object the client makes; `None` when
    /// the temporary range is used up.
    pub fn allocate_temporary_id(&mut self) -> Option<u64> {
        let objects = &self.objects;
        self.temporary_ids.allocate(|id| objects.contains_key(&id))
    }

    /// `CObjectMaint::DeallocateTemporaryID` for an id whose object was never made.
    pub fn deallocate_temporary_id(&mut self, id: u64) {
        self.temporary_ids.deallocate(id);
    }

    /// Every remote's entry in the world's object lists is brought up to date (after the
    /// session's events and the FX objects' changes).
    pub fn sync_shadows(&mut self, terrain: &mut Terrain) {
        for (&guid, remote) in self.objects.iter_mut() {
            sync_shadow(guid, remote, terrain, &mut self.membership);
        }
    }

    /// Leaving the world: every object goes, and its entry with it.
    pub fn clear(&mut self, terrain: &mut Terrain) {
        for remote in self.objects.values() {
            if let Some(shadow) = remote.shadow {
                terrain.remove_object(shadow);
            }
        }
        self.membership = CellMembership::new();
        self.objects.clear();
        self.lost_cells.clear();
    }

    /// `CPhysics::UpdateAllPhysics` for the objects: each one in the world and in physics range
    /// runs the quantum, its range from the player (`others`' first entry, the player's id and
    /// body from `ICPhysicsObj::player_id`, `SetPlayerID`) checked first as
    /// `CPhysicsObj::UpdateMovement` does. `cur_time` is the server clock the positions were
    /// stamped on. A missile
    /// reads the other objects (`ICPhysicsObj::GetObjectA`: these remotes and `others`, the
    /// player) and rolls on `random`; what the missiles told the weenie and the streaks comes
    /// back with the object's id.
    pub fn step_quantum(
        &mut self,
        terrain: &mut Terrain,
        cur_time: f64,
        random: &mut Random,
        others: &[(u64, ac2_physics::ObjectView)],
    ) -> Vec<(u64, ac2_physics::MissileEvent)> {
        struct Table(HashMap<u64, ac2_physics::ObjectView>);
        impl ac2_physics::ObjectLookup for Table {
            fn object(&self, id: u64) -> Option<ac2_physics::ObjectView> {
                self.0.get(&id).copied()
            }
        }
        struct Impacts(HashMap<u64, ac2_physics::ImpactTarget>);
        impl ac2_physics::ImpactLookup for Impacts {
            fn target(&self, id: u64) -> Option<ac2_physics::ImpactTarget> {
                self.0.get(&id).copied()
            }
        }
        let flying = self.objects.values().any(|r| {
            r.in_world
                && r.active
                && r.body
                    .as_ref()
                    .is_some_and(PhysicsBody::has_missile_manager)
        });
        let table = Table(if flying {
            let mut views: HashMap<u64, ac2_physics::ObjectView> = others.iter().copied().collect();
            for (&id, remote) in &self.objects {
                if let (true, Some(body)) = (remote.in_world, remote.body.as_ref()) {
                    views.insert(id, body.object_view());
                }
            }
            views
        } else {
            HashMap::new()
        });
        let impacts = Impacts(if flying {
            let mut found: HashMap<u64, ac2_physics::ImpactTarget> = others
                .iter()
                .map(|&(id, _)| {
                    let target = ac2_physics::ImpactTarget {
                        id,
                        dynamic: true,
                        has_missile_manager: false,
                        weenie: None,
                    };
                    (id, target)
                })
                .collect();
            for (&id, remote) in &self.objects {
                if let (true, Some(body)) = (remote.in_world, remote.body.as_ref()) {
                    found.insert(id, body.impact_target(id));
                }
            }
            found
        } else {
            HashMap::new()
        });
        let mut coin = |chance: f32| random.check(chance);
        let mut env = ac2_physics::MissileEnv {
            objects: &table,
            coin: &mut coin,
        };
        let mut events = Vec::new();
        let mut touches = Vec::new();
        let mut out_of_range_temporaries = Vec::new();
        // `CPhysicsObj::UpdateMovement`: the range check against the player runs each quantum.
        let player = others.first().map(|(_, view)| view.position);
        for (&guid, remote) in self.objects.iter_mut() {
            if let (true, Some(player)) = (remote.in_world, player) {
                if let Some(position) = remote.body.as_ref().map(PhysicsBody::position) {
                    remote.update_physics_range(&player, &position);
                    // `HandleOutOfPhysicsRange`: an object the client made goes
                    // (`IIDUtils::VerifyTemporary`, `CObjectMaint::DestroyObject`).
                    if !remote.active && crate::temporary_ids::is_temporary(guid) {
                        out_of_range_temporaries.push(guid);
                        continue;
                    }
                }
            }
            if remote.in_world && remote.active {
                if let Some(body) = &mut remote.body {
                    body.set_time(cur_time);
                    body.step_quantum_in(terrain, &mut env);
                    // `CPhysicsObj::handle_all_collisions`: each object the body hit hears of it.
                    let mover = ConflictPeer {
                        id: guid,
                        ..body.conflict_peer()
                    };
                    for hit in body.take_collided() {
                        touches.push(Touch {
                            target: hit.id,
                            mover: mover.clone(),
                            ignores: body.ignore_collisions(),
                            impact: hit.impact,
                        });
                    }
                    let manager = body.has_collision_manager();
                    let missile = body.has_missile_manager();
                    if manager || missile {
                        // `CPhysicsObj::ReportCollisions`: the weenie hears the report
                        // (`HandleCollision`), then `CMissileManager::HandleCollision`.
                        let materials = body.contact_materials(terrain);
                        if let Some(report) = body.report_collisions(&materials, false) {
                            if manager {
                                self.traffic.reports.push((guid, report.clone()));
                            }
                            if missile {
                                body.handle_missile_collision(terrain, &report, &impacts);
                            }
                        }
                    }
                    self.traffic
                        .drops
                        .extend(body.take_dropped().into_iter().map(|other| (guid, other)));
                    events.extend(body.take_missile_events().into_iter().map(|e| (guid, e)));
                }
            }
            sync_shadow(guid, remote, terrain, &mut self.membership);
        }
        for id in out_of_range_temporaries {
            self.removed(id, terrain);
        }
        // The other remotes hear of the movers; the player's body is the client's.
        for touch in touches {
            match self.body_mut(touch.target) {
                Some(target) => target.touched_by(&touch.mover, touch.ignores, touch.impact),
                None => self.traffic.touches.push(touch),
            }
        }
        // A remote's erased entry drops the reporting remote too (`CollisionRemoveObject`).
        let drops = std::mem::take(&mut self.traffic.drops);
        for (owner, other) in drops {
            match self.body_mut(other) {
                Some(body) => {
                    // The properties of `owner` end at once, for `other`'s script.
                    if let Some(report) = body.collision_remove_object(owner) {
                        self.traffic.reports.push((other, report));
                    }
                }
                None => self.traffic.drops.push((owner, other)),
            }
        }
        events
    }

    /// The collision reports the remotes with a collision manager made since the last call, by
    /// object, for each one's script (`CPhysicsObj::ReportCollisions` -> `HandleCollision`).
    pub fn take_collision_reports(&mut self) -> Vec<(u64, ac2_physics::CollisionReport)> {
        std::mem::take(&mut self.traffic.reports)
    }

    /// The touches of remote movers on bodies the remotes do not hold (the player's).
    pub fn take_collision_touches(&mut self) -> Vec<Touch> {
        std::mem::take(&mut self.traffic.touches)
    }

    /// `(owner, other)` pairs where `owner`'s report erased `other`, which is not a remote: the
    /// client calls `collision_remove_object(owner)` on that body.
    pub fn take_collision_drops(&mut self) -> Vec<(u64, u64)> {
        std::mem::take(&mut self.traffic.drops)
    }

    /// `CPhysics::UpdateAllAnimation` for the objects.
    pub fn update_animation(
        &mut self,
        step: f32,
        local_time: f64,
        wind: [f32; 3],
        random: &mut Random,
        scripts: &FxScripts,
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
    pub fn look_targets(&self, into: &mut HashMap<u64, LookTarget>) {
        for (id, remote) in &self.objects {
            if let (true, Some(body)) = (remote.in_world, remote.body.as_ref()) {
                into.insert(*id, look_snapshot(body));
            }
        }
    }

    /// The animation events each object's body raised since the last call, with its visual.
    pub fn take_animation_events(&mut self) -> Vec<(u64, Option<u32>, Vec<Ac2AnimEvent>)> {
        self.objects
            .iter_mut()
            .filter_map(|(id, remote)| {
                let events = remote.body.as_mut()?.take_animation_events();
                Some((*id, remote.visual, events))
            })
            .collect()
    }

    /// `CMotionInterp::SetMIDESC` for a remote object whose description the loader read.
    pub fn set_motion_desc(&mut self, id: u64, desc: Arc<Ac2MotionInterpDesc>) {
        if let Some(body) = self.body_mut(id) {
            body.set_motion_desc(Some(desc));
        }
    }
}

impl<B: AsRef<BodyData>> Remotes<B> {
    /// `CObjectMaint::RecvEvt_CreateObject` for an object other than the player: its entry is
    /// made (or, when it is already tracked, placed at its position), and its body is created
    /// once its Setup is in. `describe` applies the session's `PhysicsDesc` to the new body.
    pub fn create_object(
        &mut self,
        id: u64,
        records: Records<'_>,
        terrain: &mut Terrain,
        describe: &dyn Fn(&mut PhysicsBody, u64),
    ) -> Vec<ResidencyEffect> {
        let mut effects = Vec::new();
        let Some(object) = records(id) else {
            return effects;
        };
        if let Some(did) = object.entity.filter(|&did| is_entity_desc(did)) {
            self.record_entity(id, did);
        }
        if self.tracks(id) {
            if let (Some(position), false) = (object.position, object.held) {
                if position.cell_id != 0 {
                    effects.extend(self.sync_position_simple(id, position, records, terrain));
                }
            }
            return effects;
        }
        self.insert(
            id,
            Remote {
                visual: object.visual,
                body: None,
                failed: false,
                stored: object.position,
                in_world: false,
                lost_cell: None,
                active: true,
                player_distance: 0.0,
                object: None,
                shadow: None,
                listed_cell: None,
                ethereal: false,
                alpha_moves: None,
                fx_follow: None,
            },
        );
        self.try_create_into(id, records, terrain, describe, &mut effects);
        effects
    }

    /// `CObjectMaint::CreateTemporaryObject`: an object the client makes (an FX's), created like
    /// any other once its Setup is in.
    pub fn create_local(
        &mut self,
        id: u64,
        local: LocalObject,
        records: Records<'_>,
        terrain: &mut Terrain,
        describe: &dyn Fn(&mut PhysicsBody, u64),
    ) -> Vec<ResidencyEffect> {
        self.insert_local(
            id,
            Remote {
                visual: Some(local.visual),
                body: None,
                failed: false,
                stored: Some(local.position),
                in_world: false,
                lost_cell: None,
                active: true,
                player_distance: 0.0,
                object: None,
                shadow: None,
                listed_cell: None,
                ethereal: true,
                alpha_moves: Some(local.alpha_moves),
                fx_follow: local.fx_follow,
            },
        );
        let mut effects = Vec::new();
        self.try_create_into(id, records, terrain, describe, &mut effects);
        effects
    }

    /// `CObjectMaint::FinishEntityCreation` once the Setup is in: the physics object, placed at
    /// the create's position when it has a cell and no parent (`SyncPositionSimple(pos, 1)`).
    pub fn try_create(
        &mut self,
        id: u64,
        records: Records<'_>,
        terrain: &mut Terrain,
        describe: &dyn Fn(&mut PhysicsBody, u64),
    ) -> Vec<ResidencyEffect> {
        let mut effects = Vec::new();
        self.try_create_into(id, records, terrain, describe, &mut effects);
        effects
    }

    fn try_create_into(
        &mut self,
        id: u64,
        records: Records<'_>,
        terrain: &mut Terrain,
        describe: &dyn Fn(&mut PhysicsBody, u64),
        effects: &mut Vec<ResidencyEffect>,
    ) {
        let Some(visual) = self.get(id).map(|r| r.visual()) else {
            return;
        };
        let data = match self.body_result(visual) {
            None => return,
            Some(Ok(data)) => Some(data.as_ref().clone()),
            Some(Err(_)) => None,
        };
        let Some(data) = data else {
            if let Some(remote) = self.get_mut(id) {
                remote.failed = true;
            }
            return;
        };
        let Some(remote) = self.get_mut(id) else {
            return;
        };
        let start = remote.stored.unwrap_or_default();
        let ethereal = remote.ethereal;
        let stored = remote.stored;
        let alpha_moves = remote.alpha_moves.take();
        remote.object = data.object;
        let mut body = PhysicsBody::new(start, data.shape, data.motion);
        body.set_radius(data.radius);
        if let Some(parts) = data.parts {
            body.set_part_group(parts);
            // `CPhysicsObj::InitObjectEnd`: `PHYSICSOBJREP::CacheHasPhysicsBSP` sets the state bit.
            body.cache_has_physics_bsp();
        }
        if ethereal {
            // `FXManager::DoCreateObjectFX`: `CollisionParameters::SetBit(0x10)` and the ETHEREAL
            // physics type, which the object's list entry carries (`sync_shadow`).
            body.collision_mut().set_bit(FX_OBJECT_COLLISION_BIT);
        }
        if let Some(bits) = self.entity_mask(id) {
            body.apply_blocking_bits(bits);
        }
        if let Some(animation) = data.animation {
            body.set_animation(animation);
        }
        // The `PhysicsDesc`'s mode, behaviours and sliders (`CPhysicsObj::set_description`),
        // applied by the client.
        describe(&mut body, id);
        if let Some(moves) = alpha_moves {
            for (pos, time) in moves {
                body.do_slider(0, crate::fx_objects::ALPHA_SLIDER, pos, time);
            }
        }
        if let Some(remote) = self.get_mut(id) {
            remote.body = Some(body);
        }
        let held = records(id).is_some_and(|r| r.held);
        if let (Some(position), false) = (stored, held) {
            if position.cell_id != 0 {
                effects.extend(self.sync_position_simple(id, position, records, terrain));
            }
        }
    }

    /// What `TargetManager` reads of the object's body (`TargetOwner`): its id, position,
    /// velocity, `GetRadius`, `GetHeight` and scale. `None` unless it is in the world with a body.
    pub fn target_owner(&self, id: u64) -> Option<TargetOwner> {
        let (position, radius, height) = self.target_info(id)?;
        let remote = self.objects.get(&id)?;
        let body = remote.body.as_ref()?;
        let v = body.movement_state().velocity;
        let scale = remote
            .object
            .as_ref()
            .map_or(Vector::new(1.0, 1.0, 1.0), |(_, s)| *s);
        Some(TargetOwner {
            id,
            position,
            velocity: Vector::new(v[0], v[1], v[2]),
            radius,
            height,
            scale,
        })
    }

    /// A move-to target as `TargetManager` reports it: an object in the world with a body, its
    /// position, `GetRadius` and `GetHeight` (the Setup's, scaled).
    pub fn target_info(&self, id: u64) -> Option<(Position, f32, f32)> {
        let remote = self.objects.get(&id).filter(|r| r.in_world)?;
        let body = remote.body.as_ref()?;
        let data = self.bodies.get(&remote.visual)?.as_ref().ok()?.as_ref();
        let scale = f64::from(data.shape.scale);
        let radius = (f64::from(data.radius) * scale) as f32;
        let height = (f64::from(data.shape.height) * scale) as f32;
        Some((body.position(), radius, height))
    }
}

#[path = "remote_residency.rs"]
mod residency;

#[cfg(test)]
#[path = "remotes_tests.rs"]
mod tests;
