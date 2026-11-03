//! The script host's core on the client's side of the VM: the state a running script set keeps
//! ([`ScriptState`]), and what becomes of each call's output ([`outcome`]): the physics it asked
//! for, the latent starts, the problems it noted, the DAT reads it stopped for, and, when the call
//! is the VM's, the server calls, chat lines and notices it produced. Everything the host needs
//! from the client goes through [`ScriptPort`].

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ac2_net::protocol::{pack_values, HeapObject, PackValue, PackedEntry};
use ac2_weenie::SessionEvent;

use super::latent;
use super::notices::ScriptNotice;
use super::port::ScriptPort;
use super::{DispatchOutcome, ScriptHost};

/// The script VM and the objects waiting for their process.
pub struct ScriptState {
    pub host: ScriptHost,
    /// Objects whose WState is being read, and the script calls that arrived meanwhile
    /// (`CObjectMaint::GetObjectOrQueueEvent` keeps an object's events until it exists).
    pub waiting: BTreeMap<u64, Vec<(u32, Vec<u8>)>>,
    /// Objects with a process.
    pub running: BTreeSet<u64>,
    /// Server calls to a running process that name an object whose process is still being read
    /// (target, function, packed arguments), in arrival order: they run once those processes
    /// start. Retail made an object's weenie object as its `CreateObject` was handled, so a call
    /// after it found it (`EquipItem_Done`'s `GetInternalWeenieObject` of the item the server
    /// just made anew).
    pub held: VecDeque<HeldCall>,
    /// What the boundary dropped and dispatches that failed, for diagnostics.
    pub problems: Vec<String>,
    /// The server calls and UI IFuns handed to the VM.
    pub owned: BTreeSet<u32>,
    /// Events produced between frames (a process started on a loader result), for the next
    /// frame's events.
    pub produced: Vec<SessionEvent>,
    /// Contexts stopped for data the loader is reading: context id -> (object, the call it
    /// runs; `None` for a UI IFun, always the VM's).
    pub data_waits: BTreeMap<u32, (u64, Option<u32>)>,
    /// Script data reads handed to the loader and not back yet.
    pub data_pending: usize,
    /// Objects whose process could not start for a WState the loader is reading, with the
    /// package and entity DID to ask again with once those reads are answered.
    pub state_retries: BTreeMap<u64, (u32, u32)>,
    /// A read of the current batch the world did not answer (`load_script_data` gave `false`):
    /// nothing waiting on the batch resumes, since the reads would only be asked again.
    pub data_unanswered: bool,
    /// The UI queries that stopped for data the loader was reading (their values were not
    /// read): when the reads land, a craft query among them makes the craft panel ask again.
    pub waited_queries: BTreeSet<u32>,
    /// The `CM_*` notices ac2-weenie has no type for, since the last take.
    pub script_notices: Vec<ScriptNotice>,
    /// The objects whose physics objects the scripts set no-draw (`SetNoDrawPhysics`).
    pub no_draw: BTreeSet<u64>,
}

impl ScriptState {
    /// Whether the packed `arguments` name an object whose process is still being read.
    pub fn names_waiting(&self, arguments: &[u8]) -> bool {
        if self.waiting.is_empty() {
            return false;
        }
        let Ok(values) = ac2_net::protocol::unpack_values(arguments) else {
            return false;
        };
        values.iter().any(|value| match value {
            PackValue::LongInt(id) => self.waiting.contains_key(id),
            PackValue::Reference(graph) => graph.entries.iter().any(|entry| match entry {
                PackedEntry::Heap { object, .. } => {
                    instance_ids(object).any(|id| self.waiting.contains_key(&id))
                }
                _ => false,
            }),
            _ => false,
        })
    }

    /// Whether a call to `object` arriving now waits behind [`Self::held`] (an earlier call to
    /// it is held, or it names an object still loading).
    pub fn holds(&self, object: u64, arguments: &[u8]) -> bool {
        self.held.iter().any(|call| call.target == object) || self.names_waiting(arguments)
    }

    /// A fresh state over `host`, with the server calls and UI IFuns in `functions` handed to it.
    pub fn new(host: ScriptHost, functions: &[u32]) -> Self {
        Self {
            host,
            waiting: BTreeMap::new(),
            running: BTreeSet::new(),
            held: VecDeque::new(),
            problems: Vec::new(),
            owned: functions.iter().copied().collect(),
            produced: Vec::new(),
            data_waits: BTreeMap::new(),
            data_pending: 0,
            state_retries: BTreeMap::new(),
            data_unanswered: false,
            waited_queries: BTreeSet::new(),
            script_notices: Vec::new(),
            no_draw: BTreeSet::new(),
        }
    }
}

/// A server call held for a loading object ([`ScriptState::held`]).
#[derive(Debug, Clone)]
pub struct HeldCall {
    pub target: u64,
    pub function: u32,
    pub arguments: Vec<u8>,
    /// The container and inventory refreshes the session's inventory stand-in raised for the
    /// call as it arrived (`UpdateContainer` / `UpdateInventory`, just before the call's
    /// `SessionEvent::Interp`): the panels read the contents from the script
    /// (`gmCWInterface::GetContentsInfo`), which changes only when the call runs, so they are
    /// raised again after it.
    pub refreshes: Vec<SessionEvent>,
}

/// The instance ids a packed package object holds: a slot tagged as an id's low word (0x82, or
/// 0x84 as `CallIFun_*` pushes it) followed by its high word (tag 0xCD).
fn instance_ids(object: &HeapObject) -> impl Iterator<Item = u64> + '_ {
    object
        .tags
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| matches!(pair[0], 0x82 | 0x84) && pair[1] == 0xCD)
        .map(|(i, _)| u64::from(object.slots[i + 1]) << 32 | u64::from(object.slots[i]))
}

/// The end of one dispatch, resume or IFun on object `id`: problems noted, a call that stopped for
/// data remembered, the output routed (`out`) when the call is the VM's (set aside when a stand-in
/// already answered it), and the reads it needs handed to the loader.
///
/// `function` is `None` for a UI IFun. Ported from `CWDistributor::DispatchInterpMsg`'s tail
/// (`gCWInterface::SendServerEvt`, `CM_*::SendNotice_*`).
pub fn outcome<P: ScriptPort + ?Sized>(
    port: &mut P,
    id: u64,
    function: Option<u32>,
    result: Result<DispatchOutcome, String>,
    out: &mut Vec<SessionEvent>,
) {
    // The physics changes the scripts made (`gCWPhysicsObject` scale and FX efuns) reach the
    // bodies whatever became of the call.
    let physics = port
        .script_state()
        .map(|s| s.host.take_physics())
        .unwrap_or_default();
    for (object, action) in physics {
        port.apply_script_physics(object, &action);
    }
    // The freezes and the edge-slide flag a script set on a body.
    let states = port
        .script_state()
        .map(|s| s.host.take_body_states())
        .unwrap_or_default();
    for (object, state) in states {
        port.apply_body_state(object, state);
    }
    // `gCWPhysicsObject::CallEFun_ApplyAdditiveVisualDesc` and the other visual-description edits.
    let edits = port
        .script_state()
        .map(|s| s.host.take_appearance_edits())
        .unwrap_or_default();
    for (object, appearance) in edits {
        port.set_object_appearance(object, appearance);
    }
    // `gCWPhysicsObject::CallEFun_StopAllCameraFX`.
    let stops = port
        .script_state()
        .map(|s| s.host.take_camera_fx_stops())
        .unwrap_or_default();
    for object in stops {
        port.stop_all_camera_fx(object);
    }
    // `gCWPhysicsObject::CallEFun_SetCollisionState` (the impact minimums).
    let minimums = port
        .script_state()
        .map(|s| s.host.take_collision_minimums())
        .unwrap_or_default();
    for (object, xy, z) in minimums {
        port.apply_collision_minimums(object, xy, z);
    }
    // The same call's ethereal and physics-type sets (`CCollisionManager::SetCollisionState`).
    let sets = port
        .script_state()
        .map(|s| s.host.take_collision_sets())
        .unwrap_or_default();
    for (object, sets) in sets {
        port.apply_collision_sets(object, &sets);
    }
    let latent = port
        .script_state()
        .map(|s| s.host.take_latent_physics())
        .unwrap_or_default();
    latent::start_all(port, latent, out);
    let owned = function.is_none_or(|f| !port.stand_in_handles(f));
    let Some(scripts) = port.script_state() else {
        return;
    };
    let call = function.map_or("IFun".to_owned(), |f| format!("{f:#010x}"));
    match result {
        Err(error) => scripts.problems.push(format!("{id:#018x} {call}: {error}")),
        Ok(outcome) if outcome.fault.is_some() => scripts.problems.push(format!(
            "{id:#018x} {call}: {:?} {:?}",
            outcome.status, outcome.fault
        )),
        // A context stopped for a DAT read runs again when the read lands; one stopped for
        // a question (`PopupYesNoDialog`) waits for its answer.
        Ok(outcome) if outcome.data_wait => {
            scripts.data_waits.insert(outcome.context, (id, function));
        }
        Ok(_) => {}
    }
    let lines = scripts.host.take_shell_lines();
    let help = scripts.host.take_help_requests();
    for (object, flag) in scripts.host.take_no_draw() {
        if flag {
            scripts.no_draw.insert(object);
        } else {
            scripts.no_draw.remove(&object);
        }
    }
    let calls = scripts.host.take_server_calls();
    let notices = scripts.host.take_notices();
    let script_notices = scripts.host.take_script_notices();
    let dropped = scripts.host.take_dropped();
    scripts.problems.extend(dropped);
    let reads = scripts.host.take_data_requests();
    scripts.data_pending += reads.len();
    for read in reads {
        port.request_script_data(read);
    }
    if !owned {
        return;
    }
    // The lines the scripts entered for the chat shell (`CWInterface::ProcessText`), in order.
    for line in lines {
        out.extend(port.process_text(&line));
    }
    for cmd in help {
        out.extend(port.display_help(&cmd));
    }
    for call in calls {
        port.send_interp(call.function, pack_values(&call.args), call.encrypt);
    }
    // `RequestGUN_SendToRoom` with Turbine chat: `CWBookKeeper::CSendToTurbineRoomByID` (the chat
    // client is the session's) with the packed `GunChatExtraInfo`.
    let gun = ac2_weenie::gun::shared();
    for send in gun.take_turbine_sends() {
        if let Some(context) = port.send_to_room(send.room, &send.text, send.extra.clone()) {
            gun.register_turbine_context(context, &send);
        }
    }
    out.extend(notices.into_iter().map(SessionEvent::Notice));
    if let Some(scripts) = port.script_state() {
        scripts.script_notices.extend(script_notices);
    }
}
