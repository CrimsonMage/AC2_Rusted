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
    CellSource, FxFollow, LookTarget, ObjectShape, PhysicsBody, Position, StaticObject, Terrain,
    Vector,
};
use ac2_world::Ac2MotionInterpDesc;

use crate::BodyData;

/// `FXManager::DoCreateObjectFX`'s `CollisionParameters` bit for an FX object.
pub const FX_OBJECT_COLLISION_BIT: u32 = 0x10;

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
fn sync_shadow(remote: &mut Remote, terrain: &mut Terrain) {
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
            let mut object =
                StaticObject::dynamic(position, scale, shape, mobile).with_blocking_mask(blocking);
            object.set_ethereal(remote.ethereal);
            let id = terrain.insert_object(object);
            remote.shadow = Some(id);
            if let Some(body) = remote.body.as_mut() {
                body.set_world_id(Some(id));
            }
        }
        (Some((position, _, _, _, blocking)), Some(id)) => {
            terrain.move_object(id, position);
            terrain.set_object_blocking(id, blocking);
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
        for remote in self.objects.values_mut() {
            sync_shadow(remote, terrain);
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
    /// runs the quantum. `cur_time` is the server clock the positions were stamped on.
    pub fn step_quantum(&mut self, terrain: &mut Terrain, cur_time: f64) {
        for remote in self.objects.values_mut() {
            if remote.in_world && remote.active {
                if let Some(body) = &mut remote.body {
                    body.set_time(cur_time);
                    body.step_quantum(terrain);
                }
            }
            sync_shadow(remote, terrain);
        }
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

#[cfg(test)]
#[path = "remotes_tests.rs"]
mod tests;
