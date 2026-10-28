//! The weenies' script processes over the session's objects: each object the session reports
//! gets its script process (`CWDistributor::Construct`, from the WState the loader reads first),
//! the server's script calls are dispatched to it (`CWDistributor::DispatchInterpMsg`), the
//! objects' qualities and appearance are shared with the VM as the session keeps them, and what
//! the scripts produce goes out through [`outcome`]. Written against [`ScriptPort`]; the client
//! answers the port's hooks (the session's objects, the loader).

use ac2_net::protocol::{unpack_values, ObjectAction, PackValue};
use ac2_weenie::SessionEvent;

use super::ifun::IFunArg;
use super::outcome::outcome;
use super::port::ScriptPort;

/// The weenie interface's lifecycle IFuns (`gCWInterface::CallIFun_Handle*`, Legions 7160): the
/// object's script hears that it entered or left the world, or was destroyed.
pub mod lifecycle_ifun {
    /// `CWInterface::HandleEnterWorld` -> `gCWInterface::CallIFun_HandleEnterWorld`.
    pub const HANDLE_ENTER_WORLD: u32 = 0x007F_0017;
    /// `CWInterface::HandleLeaveWorld` -> `gCWInterface::CallIFun_HandleLeaveWorld`.
    pub const HANDLE_LEAVE_WORLD: u32 = 0x007F_0018;
    /// `CWInterface::HandlePostLeaveWorld` -> `gCWInterface::CallIFun_HandlePostLeaveWorld`.
    pub const HANDLE_POST_LEAVE_WORLD: u32 = 0x007F_0019;
    /// `CWInterface::HandleDestruction` -> `gCWInterface::CallIFun_HandleDestruction`.
    pub const HANDLE_DESTRUCTION: u32 = 0x007F_0016;
    /// `CWInterface::HandleCharacterSessionStart` ->
    /// `gCWInterface::CallIFun_HandleCharacterSessionStart`.
    pub const HANDLE_CHARACTER_SESSION_START: u32 = 0x007F_001C;
    /// `CWInterface::HandleCharacterSessionEnd` ->
    /// `gCWInterface::CallIFun_HandleCharacterSessionEnd`.
    pub const HANDLE_CHARACTER_SESSION_END: u32 = 0x007F_001D;
}

/// The script lifecycle, on any [`ScriptPort`] (the client implements the port).
pub trait ScriptLifecycle: ScriptPort {
    /// Asks the loader for `id`'s script state (its weenie's package and entity DID). Nothing when
    /// the object has no package or is already running or waiting.
    fn request_script(&mut self, id: u64) {
        let Some(scripts) = self.script_state() else {
            return;
        };
        if scripts.running.contains(&id) || scripts.waiting.contains_key(&id) {
            return;
        }
        let Some(identity) = self.script_identity(id) else {
            return;
        };
        let Some(package) = identity.package else {
            return;
        };
        let did = identity.entity.unwrap_or(0);
        if let Some(scripts) = self.script_state() {
            scripts.waiting.insert(id, Vec::new());
        }
        self.request_script_state(id, package, did);
    }

    /// The loader read `id`'s script state: its process starts, then the calls it missed run.
    fn script_state_loaded(&mut self, id: u64, pid: u32, did: u32) {
        let player = self.player_id();
        let Some(scripts) = self.script_state() else {
            return;
        };
        let Some(missed) = scripts.waiting.remove(&id) else {
            return;
        };
        if !scripts.host.create_weenie(id, pid, did) {
            let reads = scripts.host.take_data_requests();
            if reads.is_empty() {
                scripts.problems.push(format!(
                    "{id:#018x}: no script process for package {pid:#x}"
                ));
                return;
            }
            // The WState is not resident yet: the loader reads what create_weenie missed, and
            // the process is tried again once those answer (absent records answer too, and the
            // retry then starts the process from the fresh instance, as retail's null does).
            scripts.waiting.insert(id, missed);
            scripts.state_retries.insert(id, (pid, did));
            scripts.data_pending += reads.len();
            for read in reads {
                self.request_script_data(read);
            }
            return;
        }
        scripts.running.insert(id);
        if Some(id) == player {
            scripts.host.set_player(id);
        }
        self.share_qualities(id);
        self.share_appearance(id);
        let mut events = Vec::new();
        for (function, arguments) in missed {
            self.dispatch_script(id, function, &arguments, &mut events);
        }
        // `CWInterface::HandleEnterWorld`: the object is in the world as its process starts.
        self.lifecycle_ifun(id, lifecycle_ifun::HANDLE_ENTER_WORLD, &mut events);
        self.push_produced(events);
    }

    /// A script data read landed (`answered`: the world resolved it, see `Loaded::ScriptData`);
    /// once none is outstanding, the calls waiting on data run on and the objects whose process
    /// waited for a WState start again. A batch with an unanswered read stops here: those waits
    /// stay until the world answers, and nothing is asked again in a loop.
    fn script_data_loaded(&mut self, answered: bool) {
        let Some(scripts) = self.script_state() else {
            return;
        };
        scripts.data_pending = scripts.data_pending.saturating_sub(1);
        scripts.data_unanswered |= !answered;
        if scripts.data_pending > 0 {
            return;
        }
        if std::mem::take(&mut scripts.data_unanswered) {
            return;
        }
        let retries = std::mem::take(&mut scripts.state_retries);
        let waits = std::mem::take(&mut scripts.data_waits);
        for (id, (pid, did)) in retries {
            self.request_script_state(id, pid, did);
        }
        let mut events = Vec::new();
        for (context, (id, function)) in waits {
            let Some(scripts) = self.script_state() else {
                return;
            };
            let Some(result) = scripts.host.resume(id, context) else {
                continue;
            };
            outcome(self, id, function, Ok(result), &mut events);
        }
        self.push_produced(events);
    }

    /// The frame's session events through the scripts (after the session dispatched them).
    fn run_scripts(&mut self, events: &mut Vec<SessionEvent>) {
        if self.script_state().is_none() {
            return;
        }
        let mut produced = std::mem::take(&mut self.script_state().expect("attached").produced);
        for event in events.iter() {
            match event {
                SessionEvent::ObjectAdded(id) => self.request_script(*id),
                SessionEvent::QualityChanged {
                    object,
                    table,
                    quality,
                } => {
                    self.share_quality(*object, *table, *quality);
                    self.stat_changed(*object, *table, *quality, &mut produced);
                }
                SessionEvent::ObjectAction {
                    object,
                    action: ObjectAction::UpdateVisualDesc(_),
                } => self.share_appearance(*object),
                SessionEvent::PlayerQualities(_) => {
                    if let Some(player) = self.player_id() {
                        self.share_qualities(player);
                    }
                }
                SessionEvent::EnteredWorld(entry) => self.request_script(entry.player),
                SessionEvent::ObjectRemoved(id) => {
                    // `CWInterface::HandleDestruction`, before the process goes.
                    self.lifecycle_ifun(*id, lifecycle_ifun::HANDLE_DESTRUCTION, &mut produced);
                    if let Some(scripts) = self.script_state() {
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
                    let scripts = self.script_state().expect("attached");
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

    /// `CWInterface::Handle*` for object `id`: the lifecycle IFun `function` (no arguments) on its
    /// process, run only when the VM owns the call and the process runs (as the stand-ins keep
    /// the calls they answer). The script's output joins `out`.
    fn lifecycle_ifun(&mut self, id: u64, function: u32, out: &mut Vec<SessionEvent>) {
        let Some(scripts) = self.script_state() else {
            return;
        };
        if !scripts.owned.contains(&function) || !scripts.running.contains(&id) {
            return;
        }
        let result = scripts
            .host
            .call_ifun(id, function, &[])
            .map(|r| r.outcome)
            .map_err(|e| format!("{e:?}"));
        outcome(self, id, None, result, out);
    }

    /// Whether object `id` has a running process (the scripts' copies of it are kept).
    fn script_running(&mut self, id: u64) -> bool {
        self.script_state().is_some_and(|s| s.running.contains(&id))
    }

    /// `CWPhysicsObject::SetLocalVisualDesc` for a running process: the scripts copy the object's
    /// local description (`gCWPhysicsObject::GetLocalVisualDesc`) as the session keeps it from the
    /// server (`RecvEvt_UpdateVisualDesc`, the object's create).
    fn share_appearance(&mut self, id: u64) {
        if !self.script_running(id) {
            return;
        }
        if let Some(appearance) = self.object_appearance(id) {
            if let Some(scripts) = self.script_state() {
                scripts.host.set_appearance(id, appearance);
            }
        }
    }

    /// `CWRepository::SetQualities` / `SetWeenieDesc` for a running process: the scripts read
    /// the object's qualities (`gCWRepository`) as the session keeps them from the server
    /// (the player description, the weenie description, the quality updates).
    fn share_qualities(&mut self, id: u64) {
        if !self.script_running(id) {
            return;
        }
        if let Some(qualities) = self.object_qualities(id) {
            if let Some(scripts) = self.script_state() {
                scripts.host.set_qualities(id, qualities);
            }
        }
    }

    /// One quality update for a running process: that quality alone is copied into the
    /// scripts' repository, so what the scripts wrote since (`Agent` 0x3F's dead flag, bool 289)
    /// survives it, as it does in retail's single `CWRepository`.
    fn share_quality(&mut self, id: u64, table: u32, quality: u32) {
        if !self.script_running(id) {
            return;
        }
        let Some(qualities) = self.object_qualities(id) else {
            return;
        };
        let Some(scripts) = self.script_state() else {
            return;
        };
        match scripts.host.qualities_mut(id) {
            Some(mine) => mine.copy_quality(&qualities, table, quality),
            None => scripts.host.set_qualities(id, qualities),
        }
    }

    /// `CWRepository::RecvEvt_Update*_Private` -> `CWInterface::OnStatChanged`: once the VM
    /// has the call, the object's script hears which quality changed (`CPlayer` 0x85: the
    /// vitals, the XP bar, the skill panel; `CItem` 0x3: the item update).
    fn stat_changed(&mut self, id: u64, table: u32, quality: u32, out: &mut Vec<SessionEvent>) {
        let function = ac2_weenie::IFUN_ON_STAT_CHANGED;
        let Some(scripts) = self.script_state() else {
            return;
        };
        if !scripts.owned.contains(&function) || !scripts.running.contains(&id) {
            return;
        }
        let args = [IFunArg::Int(table), IFunArg::Int(quality)];
        let result = scripts
            .host
            .call_ifun(id, function, &args)
            .map(|r| r.outcome)
            .map_err(|e| format!("{e:?}"));
        outcome(self, id, None, result, out);
    }

    /// `CWDistributor::DispatchInterpMsg`, then the scripts' output ([`outcome`]).
    fn dispatch_script(
        &mut self,
        id: u64,
        function: u32,
        arguments: &[u8],
        out: &mut Vec<SessionEvent>,
    ) {
        let Some(scripts) = self.script_state() else {
            return;
        };
        let result = unpack_values(arguments)
            .map_err(|e| e.to_string())
            .and_then(|values: Vec<PackValue>| {
                scripts
                    .host
                    .dispatch(id, function, &values)
                    .map_err(|e| format!("{e:?}"))
            });
        outcome(self, id, Some(function), result, out);
    }
}

impl<P: ScriptPort + ?Sized> ScriptLifecycle for P {}
