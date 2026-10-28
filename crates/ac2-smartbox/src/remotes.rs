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
    CellSource, FxFollow, LookTarget, ObjectShape, PhysicsBody, Position, SetPositionError,
    StaticObject, TargetOwner, Terrain, Vector,
};
use ac2_world::Ac2MotionInterpDesc;

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
fn sync_shadow(guid: u64, remote: &mut Remote, terrain: &mut Terrain) {
    // The entry's blocking mask is the body's (`CPhysicsObj::IsBlocking` reads m_blocking_mask),
    // so it is re-read on every sync: a body's bits can change between syncs.
    let wanted = match (&remote.body, &remote.object, remote.in_world) {
        (Some(body), Some((shape, scale)), true) => Some((
            body.position(),
            Arc::clone(shape),
            *scale,
            body.is_mobile(),
            body.blocking_bits(),
        )),
        _ => None,
    };
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

    /// `CObjectMaint::DestroyObjectInternal` removed the object: its entry leaves the world's
    /// object lists.
    pub fn removed(&mut self, id: u64, terrain: &mut Terrain) {
        self.remove_from_lost_cell(id);
        if let Some(shadow) = self.objects.remove(&id).and_then(|r| r.shadow) {
            terrain.remove_object(shadow);
        }
        self.local.remove(&id);
    }

    /// Every remote's entry in the world's object lists is brought up to date (after the
    /// session's events and the FX objects' changes).
    pub fn sync_shadows(&mut self, terrain: &mut Terrain) {
        for (&guid, remote) in self.objects.iter_mut() {
            sync_shadow(guid, remote, terrain);
        }
    }

    /// Leaving the world: every object goes, and its entry with it.
    pub fn clear(&mut self, terrain: &mut Terrain) {
        for remote in self.objects.values() {
            if let Some(shadow) = remote.shadow {
                terrain.remove_object(shadow);
            }
        }
        self.objects.clear();
        self.lost_cells.clear();
    }

    /// `CPhysics::UpdateAllPhysics` for the objects: each one in the world and in physics range
    /// runs the quantum. `cur_time` is the server clock the positions were stamped on. A missile
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
        for (&guid, remote) in self.objects.iter_mut() {
            if remote.in_world && remote.active {
                if let Some(body) = &mut remote.body {
                    body.set_time(cur_time);
                    body.step_quantum_in(terrain, &mut env);
                    if body.has_missile_manager() {
                        // `CPhysicsObj::ReportCollisions` -> `CMissileManager::HandleCollision`.
                        let materials = body.contact_materials(terrain);
                        if let Some(report) = body.report_collisions(&materials, false) {
                            body.handle_missile_collision(terrain, &report, &impacts);
                        }
                    }
                    events.extend(body.take_missile_events().into_iter().map(|e| (guid, e)));
                }
            }
            sync_shadow(guid, remote, terrain);
        }
        events
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

#[cfg(test)]
#[path = "remotes_tests.rs"]
mod tests;
