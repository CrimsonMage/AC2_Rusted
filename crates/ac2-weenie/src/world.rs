//! `CEventQueue` and the event-routing part of `CObjectMaint`: which server events reach the
//! game, in which order, and which wait for an object that does not exist yet.
//!
//! Queue-1 blobs pass `CEventQueue::ConsiderQueueingEvent` (out-of-world blobs always; nothing
//! before world entry; while entering only `CreatePlayer` and `CharacterError`, the rest held
//! until `FinishEnteringWorld`; in the world, everything but `CreateObject` held until the
//! player exists), then go by ordering type: ordered blobs through the player's `TSRecv`, visual
//! ordered blobs through their object's `VisualOrderingQueue`, unordered blobs straight to
//! `CEventQueue::DispatchEvent`.

use std::collections::{BTreeMap, VecDeque};

use ac2_net::protocol::{
    Build, CreateObject, InstanceIdWithStamp, Opcode, PropertyTypes, ServerEvent,
};
use ac2_net::{is_out_of_world, ORDERING_VISUAL};
use ac2_net::{ordering_stamp, ordering_type, Blob, ORDERING_ORDERED, ORDERING_UNORDERED};

use crate::objects::{ObjectTable, PositionApplied, RemoteObject};
use crate::ordering::{TsRecv, TsResult, VisualOrderingQueue};

/// `CEventQueue::m_worldState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldState {
    NotInWorld,
    EnteringWorld,
    InWorld,
}

/// What dispatching produced, for the session.
#[derive(Debug, Clone, PartialEq)]
pub enum WorldEvent {
    /// A decoded event reached `DispatchEvent`.
    Dispatched(ServerEvent),
    /// A blob that failed to decode: its opcode and the reason.
    Undecoded { opcode: u32, error: String },
    /// `CPhysicsObj::RecvEvt_Position` accepted an update for `id` (before its `Dispatched`).
    PositionApplied { id: u64, applied: PositionApplied },
    /// `VisualOrderingQueue::IsHeavilyBlocked`: ask the server to re-create the object
    /// (`ECM_Physics::SendEvt_CForceCreate`).
    ForceCreate(u64),
}

/// `CBookKeep`: an object's ordering state.
#[derive(Debug, Default)]
struct Bookie {
    instance_stamp: u16,
    ordered: TsRecv,
    visual: VisualOrderingQueue,
}

/// The client's event routing and object bookkeeping.
#[derive(Debug)]
pub struct World {
    build: Build,
    state: WorldState,
    /// `CEventQueue::queuedEvents`.
    queued: Vec<Blob>,
    player: Option<u64>,
    bookies: BTreeMap<u64, Bookie>,
    /// `CObjectMaint::m_event_queue_table`.
    waiting: BTreeMap<u64, VecDeque<Blob>>,
    pub objects: ObjectTable,
    out: Vec<WorldEvent>,
    cur_time: f64,
    /// `MasterProperty` types for property collections, when the client has the DATs.
    property_types: Option<Resolver>,
}

/// The installed `MasterProperty` resolver.
struct Resolver(Box<dyn PropertyTypes + Send + Sync>);

impl std::fmt::Debug for Resolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Resolver(..)")
    }
}

impl World {
    pub fn new(build: Build) -> Self {
        Self {
            build,
            state: WorldState::NotInWorld,
            queued: Vec::new(),
            player: None,
            bookies: BTreeMap::new(),
            waiting: BTreeMap::new(),
            objects: ObjectTable::default(),
            out: Vec::new(),
            cur_time: 0.0,
            property_types: None,
        }
    }

    /// Lets events carrying property collections (light FX) decode
    /// (`MasterProperty::InqPropertyType`).
    pub fn set_property_types(&mut self, types: Box<dyn PropertyTypes + Send + Sync>) {
        self.property_types = Some(Resolver(types));
    }

    pub fn state(&self) -> WorldState {
        self.state
    }

    pub fn player(&self) -> Option<u64> {
        self.player
    }

    /// `CEventQueue::StartEnteringWorld` (`Proto_UI::SendEnterWorld`).
    pub fn start_entering_world(&mut self) {
        self.state = WorldState::EnteringWorld;
    }

    /// `CEventQueue::DestroyEventQueues`: back out of the world.
    pub fn leave_world(&mut self) {
        self.queued.clear();
        self.state = WorldState::NotInWorld;
        self.player = None;
        self.bookies.clear();
        self.waiting.clear();
        self.objects.clear();
    }

    /// `CEventQueue::UseTime` for one queue-1 blob; returns what reached the game.
    pub fn receive(&mut self, blob: Blob, cur_time: f64) -> Vec<WorldEvent> {
        self.cur_time = cur_time;
        self.process_net_blob(blob);
        std::mem::take(&mut self.out)
    }

    fn opcode(blob: &Blob) -> u32 {
        blob.data
            .get(..4)
            .map_or(0, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    /// `CEventQueue::ConsiderQueueingEvent`: `true` when the blob is held or dropped.
    fn consider_queueing(&mut self, blob: &Blob) -> bool {
        if is_out_of_world(blob.id) {
            return false;
        }
        let opcode = Self::opcode(blob);
        match self.state {
            WorldState::NotInWorld => true,
            WorldState::EnteringWorld => {
                if opcode != Opcode::CREATE_PLAYER && opcode != Opcode::CHARACTER_ERROR {
                    self.queued.push(blob.clone());
                    return true;
                }
                false
            }
            WorldState::InWorld => {
                let player_known = self.player.is_some_and(|p| self.bookies.contains_key(&p));
                if !player_known && opcode != Opcode::CREATE_OBJECT {
                    let id = self.player.unwrap_or(u64::MAX);
                    self.waiting.entry(id).or_default().push_back(blob.clone());
                    return true;
                }
                false
            }
        }
    }

    /// `CEventQueue::ProcessNetBlob`.
    fn process_net_blob(&mut self, blob: Blob) {
        if self.consider_queueing(&blob) {
            return;
        }
        match ordering_type(blob.id) {
            ORDERING_ORDERED => self.process_ordered(blob),
            ORDERING_VISUAL => self.process_visual(blob),
            ORDERING_UNORDERED => self.dispatch(blob),
            _ => {}
        }
    }

    /// `CEventQueue::ProcessOrderedNetBlob`: through the player's `TSRecv`, then every entry it
    /// releases (`CBookKeep::FlushOrderedEvents`).
    fn process_ordered(&mut self, blob: Blob) {
        let Some(player) = self.player else {
            return;
        };
        let cur_time = self.cur_time;
        let stamp = u32::from(ordering_stamp(blob.id));
        let Some(bookie) = self.bookies.get_mut(&player) else {
            return;
        };
        if let TsResult::Ready(ready) = bookie.ordered.add_and_check(stamp, blob, cur_time) {
            self.dispatch(ready);
            while let Some(next) = self
                .bookies
                .get_mut(&player)
                .and_then(|b| b.ordered.next_ready(cur_time))
            {
                self.dispatch(next);
            }
        }
    }

    /// `CEventQueue::ProcessVisualOrderedEvent`: the sender's stamp orders it within its object.
    fn process_visual(&mut self, blob: Blob) {
        let Some(sender) = blob.data.get(4..16).map(read_sender) else {
            return;
        };
        if !self.get_object_or_queue(&sender, &blob) {
            return;
        }
        if let Some(bookie) = self.bookies.get_mut(&sender.id) {
            bookie.visual.push(sender.other, blob);
        }
        self.flush_visual(sender.id);
    }

    /// `CBookKeep::FlushVisualOrderedEvents`.
    fn flush_visual(&mut self, id: u64) {
        let cur_time = self.cur_time;
        while let Some(next) = self
            .bookies
            .get_mut(&id)
            .and_then(|b| b.visual.extract(cur_time))
        {
            self.dispatch(next);
        }
        if let Some(bookie) = self.bookies.get_mut(&id) {
            if bookie.visual.heavily_blocked(cur_time) {
                bookie.visual.discard_all();
                self.out.push(WorldEvent::ForceCreate(id));
            }
        }
    }

    /// `CObjectMaint::GetObjectOrQueueEvent`: `true` when the object exists at this instance;
    /// an unknown object or a newer instance holds the blob for it, an older instance drops it.
    fn get_object_or_queue(&mut self, sender: &InstanceIdWithStamp, blob: &Blob) -> bool {
        let Some(bookie) = self.bookies.get(&sender.id) else {
            self.waiting
                .entry(sender.id)
                .or_default()
                .push_back(blob.clone());
            return false;
        };
        let current = bookie.instance_stamp;
        if sender.instance != current {
            if crate::ordering::newer16(sender.instance, current) {
                self.waiting
                    .entry(sender.id)
                    .or_default()
                    .push_back(blob.clone());
            }
            return false;
        }
        true
    }

    /// `CEventQueue::PostEvent` -> `DispatchEvent`, and the object bookkeeping each handler does.
    fn dispatch(&mut self, blob: Blob) {
        let decoded = match &self.property_types {
            Some(types) => ServerEvent::decode_with_properties(&blob.data, self.build, &*types.0),
            None => ServerEvent::decode(&blob.data, self.build),
        };
        let event = match decoded {
            Ok(event) => event,
            Err(error) => {
                self.out.push(WorldEvent::Undecoded {
                    opcode: Self::opcode(&blob),
                    error: error.to_string(),
                });
                return;
            }
        };
        match &event {
            ServerEvent::CreatePlayer { id, .. } => {
                // `GlobalEventHandler::RecvEvt_CreatePlayer`: set the player, then
                // `CEventQueue::FinishEnteringWorld` replays everything held.
                self.player = Some(*id);
                self.out.push(WorldEvent::Dispatched(event));
                self.finish_entering_world();
                return;
            }
            ServerEvent::CreateObject(object) => {
                if !self.create_object(object, &blob) {
                    return;
                }
                self.out.push(WorldEvent::Dispatched(event.clone()));
                // `CObjectMaint::FinishEntityCreation`: release its visual queue from the
                // create's stamp, then replay what waited for it.
                if let Some(bookie) = self.bookies.get_mut(&object.id) {
                    bookie
                        .visual
                        .discard_to(object.physics.visual_ordering_stamp);
                }
                self.flush_visual(object.id);
                self.flush_waiting(object.id);
                return;
            }
            ServerEvent::DestroyObject(sender) => {
                if Some(sender.id) != self.player {
                    if !self.get_object_or_queue(sender, &blob) {
                        return;
                    }
                    self.bookies.remove(&sender.id);
                    self.objects.remove(sender.id);
                }
            }
            ServerEvent::UpdatePosition(update) | ServerEvent::PositionCell(update) => {
                if !self.get_object_or_queue(&update.sender, &blob) {
                    return;
                }
                if let Some(applied) = self.objects.apply_position(update) {
                    self.out.push(WorldEvent::PositionApplied {
                        id: update.sender.id,
                        applied,
                    });
                }
            }
            // `ECM_Interp::RecvEvt_InterpCEvent_Cell`: through `GetObjectOrQueueEvent`.
            ServerEvent::Interp {
                object: Some(sender),
                ..
            } => {
                if !self.get_object_or_queue(sender, &blob) {
                    return;
                }
            }
            ServerEvent::Object(object) => {
                if !self.get_object_or_queue(&object.sender, &blob) {
                    return;
                }
                self.objects.apply_action(object.sender.id, &object.action);
            }
            // `ECM_Login::RecvEvt_PlayerDesc` -> `CWInterface::RecvEvt_PlayerDesc`.
            ServerEvent::PlayerDescription(qualities) => {
                if let Some(player) = self.player {
                    self.objects.set_qualities(player, qualities);
                }
            }
            // `ECM_Qualities::RecvEvt_Update*`: private updates go to the player, visual ones
            // through `GetObjectOrQueueEvent` to their object.
            ServerEvent::Quality(update) => match &update.object {
                Some(sender) => {
                    if !self.get_object_or_queue(sender, &blob) {
                        return;
                    }
                    self.objects.apply_quality(sender.id, update);
                }
                None => {
                    if let Some(player) = self.player {
                        self.objects.apply_quality(player, update);
                    }
                }
            },
            _ => {}
        }
        self.out.push(WorldEvent::Dispatched(event));
    }

    /// `CEventQueue::FinishEnteringWorld` and `FlushQueue`.
    fn finish_entering_world(&mut self) {
        self.state = WorldState::InWorld;
        for blob in std::mem::take(&mut self.queued) {
            self.process_net_blob(blob);
        }
    }

    /// `CObjectMaint::RecvEvt_CreateObject`: `false` when the create waits or is stale.
    fn create_object(&mut self, object: &CreateObject, blob: &Blob) -> bool {
        if let Some((parent, _, stamp)) = object.physics.parent {
            let sender = InstanceIdWithStamp {
                id: parent,
                instance: stamp,
                other: 0,
            };
            if !self.get_object_or_queue(&sender, blob) {
                return false;
            }
        }
        let stamp = object.physics.instance_stamp;
        if let Some(existing) = self.bookies.get(&object.id) {
            if existing.instance_stamp == stamp {
                // `CObjectMaint::UpdateObject`: the same instance keeps its ordering state.
                self.objects.insert(RemoteObject::from_create(object));
                return true;
            }
            if !crate::ordering::newer16(stamp, existing.instance_stamp) {
                return false;
            }
        }
        // `StartEntityCreation` / `FinishEntityCreation`: a new instance replaces the old.
        self.bookies.insert(
            object.id,
            Bookie {
                instance_stamp: stamp,
                ..Bookie::default()
            },
        );
        self.objects.insert(RemoteObject::from_create(object));
        true
    }

    /// `CObjectMaint::FlushEventQueueForBookie`.
    fn flush_waiting(&mut self, id: u64) {
        if let Some(queue) = self.waiting.remove(&id) {
            for blob in queue {
                self.process_net_blob(blob);
            }
        }
    }
}

fn read_sender(bytes: &[u8]) -> InstanceIdWithStamp {
    InstanceIdWithStamp {
        id: u64::from_le_bytes(bytes[..8].try_into().expect("8 bytes")),
        instance: u16::from_le_bytes([bytes[8], bytes[9]]),
        other: u16::from_le_bytes([bytes[10], bytes[11]]),
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
