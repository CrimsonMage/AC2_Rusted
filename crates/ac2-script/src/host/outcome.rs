//! The script host's core on the client's side of the VM: the state a running script set keeps
//! ([`ScriptState`]), and what becomes of each call's output ([`outcome`]): the physics it asked
//! for, the latent starts, the problems it noted, the DAT reads it stopped for, and, when the call
//! is the VM's, the server calls, chat lines and notices it produced. Everything the host needs
//! from the client goes through [`ScriptPort`].

use std::collections::{BTreeMap, BTreeSet};

use ac2_net::protocol::pack_values;
use ac2_weenie::SessionEvent;

use super::gun::ChatRequest;
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
    /// The `CM_*` notices ac2-weenie has no type for, since the last take.
    pub script_notices: Vec<ScriptNotice>,
    /// GUN requests for the GUN service (`ScriptPort::set_gun_service`).
    pub gun_requests: Vec<ChatRequest>,
    /// Whether the client has a GUN service; the UI's requests are queued only with one.
    pub gun_service: bool,
    /// The objects whose physics objects the scripts set no-draw (`SetNoDrawPhysics`).
    pub no_draw: BTreeSet<u64>,
}

impl ScriptState {
    /// A fresh state over `host`, with the server calls and UI IFuns in `functions` handed to it.
    pub fn new(host: ScriptHost, functions: &[u32]) -> Self {
        Self {
            host,
            waiting: BTreeMap::new(),
            running: BTreeSet::new(),
            problems: Vec::new(),
            owned: functions.iter().copied().collect(),
            produced: Vec::new(),
            data_waits: BTreeMap::new(),
            data_pending: 0,
            state_retries: BTreeMap::new(),
            data_unanswered: false,
            script_notices: Vec::new(),
            gun_requests: Vec::new(),
            gun_service: false,
            no_draw: BTreeSet::new(),
        }
    }
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
    let chat = scripts.host.take_chat_requests();
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
    for call in calls {
        port.send_interp(call.function, pack_values(&call.args), call.encrypt);
    }
    for request in chat {
        match request {
            ChatRequest::SendToRoom { room, text, flags } => {
                port.send_to_room(room, &text, flags);
            }
            // The GUN service's (friends, ignore list, presence): only queued once
            // the client says it has a GUN service.
            other => {
                if let Some(scripts) = port.script_state() {
                    scripts.gun_requests.push(other);
                }
            }
        }
    }
    out.extend(notices.into_iter().map(SessionEvent::Notice));
    if let Some(scripts) = port.script_state() {
        scripts.script_notices.extend(script_notices);
    }
}
