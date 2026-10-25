//! The weenies' scripts on the script VM (`ac2_script::host::ScriptHost`): each object the
//! session reports gets its script process (`CWDistributor::Construct`, from its WState, read
//! into the cache on a loader thread first), the server's script calls are dispatched to it
//! (`CWDistributor::DispatchInterpMsg`), and what the scripts produce goes out: their server
//! calls as `InterpSEvent`s (`gCWInterface::SendServerEvt`), their notices among the frame's
//! events (`CM_*::SendNotice_*`).
//!
//! The session's native stand-ins (`PlayerScript`, the inventory, skills) still answer the
//! calls they know; a call is the VM's once it is handed over ([`Client::attach_scripts`]'s
//! `functions`), or when no stand-in knows it. For a call a stand-in still answers, the VM runs
//! too (its state follows the server's) but what it produces is set aside, so nothing is sent
//! or shown twice. UI commands still go to the stand-ins.

use std::collections::{BTreeMap, BTreeSet};

use ac2_net::protocol::{pack_values, unpack_values, PackValue};
use ac2_script::host::ScriptHost;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::loader::Request;

/// The server calls the script VM answers as the stand-ins did (verified against them), and so
/// takes over: `WM_Communication` 0x11, 0x23 (`DisplayStringInfo`) and 0x12 (`DisplayText`).
pub const SCRIPT_VM_FUNCTIONS: &[u32] = &[
    ac2_net::protocol::script_function::DISPLAY_STRING_INFO,
    ac2_net::protocol::script_function::DISPLAY_STRING_INFO_2,
    ac2_net::protocol::script_function::DISPLAY_TEXT,
];

/// `gmCWInterface::SetSelection`'s IFun (`ggmCWInterface::CallIFun_SetSelection`: the
/// object, then `CPlayer` 0x6A).
pub const IFUN_SET_SELECTION: u32 = 0x00B4_0002;

/// The script VM and the objects waiting for their process.
pub(crate) struct Scripts {
    host: ScriptHost,
    /// Objects whose WState is being read, and the script calls that arrived meanwhile
    /// (`CObjectMaint::GetObjectOrQueueEvent` keeps an object's events until it exists).
    waiting: BTreeMap<u64, Vec<(u32, Vec<u8>)>>,
    /// Objects with a process.
    running: BTreeSet<u64>,
    /// What the boundary dropped and dispatches that failed, for diagnostics.
    problems: Vec<String>,
    /// The server calls and UI IFuns handed to the VM.
    owned: BTreeSet<u32>,
    /// Events produced between frames (a process started on a loader result), for the next
    /// frame's events.
    produced: Vec<SessionEvent>,
}

impl Client {
    /// Runs the weenies' scripts on `host` from now on, the server calls in `functions`
    /// handed over from the session's stand-ins (`OnlineSession::set_script_functions`). Build
    /// the host off the frame thread (`ScriptHost::new` over the WLib and a `DatScriptData`
    /// sharing the client's resource store, so the loader's WState reads land in its cache).
    pub fn attach_scripts(&mut self, host: ScriptHost, functions: &[u32]) {
        self.session.set_script_functions(functions);
        self.scripts = Some(Scripts {
            host,
            waiting: BTreeMap::new(),
            running: BTreeSet::new(),
            problems: Vec::new(),
            owned: functions.iter().copied().collect(),
            produced: Vec::new(),
        });
        // Objects already known get their processes too.
        let known: Vec<u64> = self.session.objects().iter().map(|o| o.id).collect();
        for id in known {
            self.request_script(id);
        }
    }

    /// The script VM, when attached.
    pub fn script_host(&self) -> Option<&ScriptHost> {
        self.scripts.as_ref().map(|s| &s.host)
    }

    /// What the script boundary dropped and the dispatches that failed since the last call.
    pub fn take_script_problems(&mut self) -> Vec<String> {
        self.scripts
            .as_mut()
            .map(|s| std::mem::take(&mut s.problems))
            .unwrap_or_default()
    }

    /// Asks a loader thread for `id`'s script state (its weenie's package and entity DID).
    fn request_script(&mut self, id: u64) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if scripts.running.contains(&id) || scripts.waiting.contains_key(&id) {
            return;
        }
        let object = self.session.objects().get(id);
        let Some(pid) = object.and_then(|o| o.script_package) else {
            return;
        };
        let did = object.and_then(|o| o.entity).unwrap_or(0);
        scripts.waiting.insert(id, Vec::new());
        self.loader.request(Request::ScriptState(id, pid, did));
    }

    /// The loader read `id`'s script state: its process starts, then the calls it missed run.
    pub(crate) fn script_state_loaded(&mut self, id: u64, pid: u32, did: u32) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        let Some(missed) = scripts.waiting.remove(&id) else {
            return;
        };
        if !scripts.host.create_weenie(id, pid, did) {
            scripts.problems.push(format!(
                "{id:#018x}: no script process for package {pid:#x}"
            ));
            return;
        }
        scripts.running.insert(id);
        if Some(id) == self.session.player() {
            scripts.host.set_player(id);
        }
        let mut events = Vec::new();
        for (function, arguments) in missed {
            self.dispatch_script(id, function, &arguments, &mut events);
        }
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(events);
        }
    }

    /// The frame's session events through the scripts (after the session dispatched them).
    pub(crate) fn run_scripts(&mut self, events: &mut Vec<SessionEvent>) {
        if self.scripts.is_none() {
            return;
        }
        let mut produced = std::mem::take(&mut self.scripts.as_mut().expect("attached").produced);
        for event in events.iter() {
            match event {
                SessionEvent::ObjectAdded(id) => self.request_script(*id),
                SessionEvent::EnteredWorld(entry) => self.request_script(entry.player),
                SessionEvent::ObjectRemoved(id) => {
                    if let Some(scripts) = self.scripts.as_mut() {
                        scripts.host.remove_weenie(*id);
                        scripts.running.remove(id);
                        scripts.waiting.remove(id);
                    }
                }
                SessionEvent::Interp {
                    object,
                    function,
                    arguments,
                } => {
                    let scripts = self.scripts.as_mut().expect("attached");
                    if let Some(queue) = scripts.waiting.get_mut(object) {
                        queue.push((*function, arguments.clone()));
                    } else if scripts.running.contains(object) {
                        self.dispatch_script(*object, *function, arguments, &mut produced);
                    }
                }
                _ => {}
            }
        }
        events.extend(produced);
    }

    /// Whether the UI's IFun (or server call) `function` is the script VM's.
    pub(crate) fn scripts_own(&self, function: u32) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|s| s.owned.contains(&function))
            && self.session.player().is_some_and(|p| {
                self.scripts
                    .as_ref()
                    .is_some_and(|s| s.running.contains(&p))
            })
    }

    /// `ggmCWInterface::CallIFun_*`: the IFun on the player's process (a 0 return slot, the
    /// arguments, the instance, `VM::CallIFun`), its output sent and returned.
    pub(crate) fn script_ifun(&mut self, function: u32, args: &[PackValue]) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        if let Some(player) = self.session.player() {
            let arguments = pack_values(args);
            self.dispatch_script(player, function, &arguments, &mut out);
        }
        out
    }

    /// `CWDistributor::DispatchInterpMsg`, then the scripts' output: sent and shown when the
    /// call is the VM's, set aside when a stand-in already answered it.
    fn dispatch_script(
        &mut self,
        id: u64,
        function: u32,
        arguments: &[u8],
        out: &mut Vec<SessionEvent>,
    ) {
        let owned = !self.session.stand_in_handles(function);
        let scripts = self.scripts.as_mut().expect("attached");
        let result = unpack_values(arguments)
            .map_err(|e| e.to_string())
            .and_then(|values| {
                scripts
                    .host
                    .dispatch(id, function, &values)
                    .map_err(|e| format!("{e:?}"))
            });
        match result {
            Err(error) => scripts
                .problems
                .push(format!("{id:#018x} {function:#010x}: {error}")),
            Ok(outcome) if outcome.fault.is_some() => scripts.problems.push(format!(
                "{id:#018x} {function:#010x}: {:?} {:?}",
                outcome.status, outcome.fault
            )),
            Ok(_) => {}
        }
        let calls = scripts.host.take_server_calls();
        let notices = scripts.host.take_notices();
        let dropped = scripts.host.take_dropped();
        scripts.problems.extend(dropped);
        if !owned {
            return;
        }
        for call in calls {
            self.session
                .send_interp(call.function, pack_values(&call.args), call.encrypt);
        }
        out.extend(notices.into_iter().map(SessionEvent::Notice));
    }
}
