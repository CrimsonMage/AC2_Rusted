//! Hosting weenie scripts: each object's script process (`CWDistributor::Construct`), script
//! calls from the server (`CWDistributor::DispatchInterpMsg` for `InterpCEvent`), the calls
//! scripts send (`gCWInterface::SendServerEvt` -> `InterpSEvent`) and the UI notices they raise
//! (`CM_*::SendNotice_*`).
//!
//! The session owns the network; it feeds [`ScriptHost::dispatch`] and sends what
//! [`ScriptHost::take_server_calls`] returns. Arguments use ac2-net's [`PackValue`].

pub mod chargen;
pub mod craft_queries;
mod glue;
mod gun;
mod ifun;
pub mod ifun_ids;
mod interface;
mod inventory_globals;
pub mod inventory_queries;
pub mod item_queries;
pub mod latent;
mod notices;
mod notices_typed;
pub mod outcome;
mod popup;
pub mod port;
pub mod queries;
pub mod radar;
mod skill_panel;
pub mod skill_requests;
pub mod social_queries;
pub mod ui_calls;

use std::collections::HashMap;
use std::sync::Arc;

use ac2_net::protocol::PackValue;
use ac2_weenie::GameplayNotice;

use crate::native::NativeObject;
use crate::vm::{
    tag, Context, EFunHost, EFuns, Fault, FunctionId, InitState, InterpStatus, Program, ScriptData,
    Vm,
};
use crate::wire;
use crate::wstate::WSTATE_BASE_DID;

pub use crate::vm::DataRequest;
pub use gun::{ChatRequest, ChatServices};
pub use ifun::{IFunArg, IFunResult, IFunReturn, OutValue};
pub use latent::LatentPhysics;
pub use notices::{notice_names, NoticeArg, ScriptNotice};
pub use outcome::ScriptState;
pub use skill_panel::SkillPanelData;

/// A script call for the server (`ECM_Interp::SendEvt_InterpSEvent[Encrypt]`).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerCall {
    pub function: u32,
    pub args: Vec<PackValue>,
    pub encrypt: bool,
}

/// What the scripts produced for the host since it last looked.
#[derive(Debug, Default)]
pub struct Outbox {
    pub server_calls: Vec<ServerCall>,
    pub notices: Vec<GameplayNotice>,
    /// Notices ac2-weenie has no type for, in the order raised.
    pub script_notices: Vec<ScriptNotice>,
    /// Chat-service requests (`gmCWInterface::RequestGUN_*`).
    pub chat_requests: Vec<ChatRequest>,
    /// Calls and notices the boundary could not convert (a native the wire cannot carry yet).
    pub dropped: Vec<String>,
}

/// One weenie object's script process (`CWDistributor::m_process`).
#[derive(Debug)]
struct Process {
    instance: u32,
    /// Contexts that returned `Interrupted` and wait to be resumed.
    waiting: Vec<Context>,
}

/// How a dispatched call ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchOutcome {
    pub status: InterpStatus,
    /// The context id (`exec_cid`).
    pub context: u32,
    /// Why it crashed, when it did.
    pub fault: Option<Fault>,
    /// The context stopped for a DAT read (cache-only data), not for a UI answer.
    pub data_wait: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// No script process for this object.
    UnknownObject(u64),
    /// `PackBuffer::UnPackBuffer` refused the arguments (a reference that does not unpack).
    BadArguments,
    /// A host query's script call ended other than `Terminated` (and why, when it crashed).
    Stopped(InterpStatus, Option<Fault>),
}

/// The pure glue and the host glue.
fn host_efuns() -> EFuns {
    let mut efuns = EFuns::new();
    crate::glue::register_pure(&mut efuns);
    glue::register(&mut efuns);
    notices::register(&mut efuns);
    gun::register(&mut efuns);
    interface::register(&mut efuns);
    popup::register(&mut efuns);
    efuns
}

/// The VM with every weenie object's script process.
pub struct ScriptHost {
    vm: Vm,
    processes: HashMap<u64, Process>,
    next_context: u32,
}

impl ScriptHost {
    /// A VM over `program` with the pure glue and the host glue bound.
    pub fn new(program: Arc<Program>, data: Box<dyn ScriptData>) -> Self {
        let efuns = host_efuns();
        Self {
            vm: Vm::new(program, data, efuns),
            processes: HashMap::new(),
            next_context: 1,
        }
    }

    /// Engine glue served elsewhere (UI, physics, ...).
    pub fn set_engine(&mut self, host: Box<dyn EFunHost>) {
        self.vm.efuns.set_host(host);
    }

    pub fn vm(&self) -> &Vm {
        &self.vm
    }

    pub fn vm_mut(&mut self) -> &mut Vm {
        &mut self.vm
    }

    /// `CWDistributor::Construct`: the object's process from its WState (`did == 0`: the class
    /// default `0x70000000 | pid & 0xFFFF`; a `0x40000049`-type did: `0x71000000 | did &
    /// 0xFFFFFF`), else a fresh instance of `pid` with its initializer run. False when neither
    /// exists, or when cache-only data does not hold the state yet (it is then among
    /// [`ScriptHost::take_data_requests`]; load it and call again).
    pub fn create_weenie(&mut self, id: u64, pid: u32, did: u32) -> bool {
        let state_did = if did == 0 {
            Some((WSTATE_BASE_DID + 0x0100_0000) | pid & 0xFFFF)
        } else if self.vm.data().db_type(did) == Some(0x4000_0049) {
            Some((WSTATE_BASE_DID + 0x0200_0000) | did & 0x00FF_FFFF)
        } else {
            None
        };
        // A state whose references do not unpack leaves no process, as in retail, and the
        // fresh instance follows.
        let state = state_did.and_then(|d| self.vm.data().wstate(d));
        if self.vm.data().take_missed() {
            return false;
        }
        let from_state = state.and_then(|s| self.vm.allocate_from_state(&s, true));
        let instance =
            from_state.or_else(|| self.vm.allocate_package(pid, InitState::NoWState, true));
        match instance {
            Some(instance) => {
                self.vm.owners.insert(instance, id);
                self.vm.init_data.insert(id, (pid, did));
                self.processes.insert(
                    id,
                    Process {
                        instance,
                        waiting: Vec::new(),
                    },
                );
                true
            }
            None => false,
        }
    }

    /// The player's object id (`CObjectMaint::GetPlayerID`); its process is
    /// `GetPlayerWeenieObjectReference`.
    pub fn set_player(&mut self, id: u64) {
        self.vm.player = id;
    }

    /// `CWRepository::SetQualities` / `SetWeenieDesc`: the object's qualities (from the player
    /// description or `WeenieQualities::from_weenie_desc`), which the scripts read and change
    /// through `gCWRepository`.
    pub fn set_qualities(&mut self, id: u64, qualities: ac2_weenie::WeenieQualities) {
        self.vm.repositories.insert(id, qualities);
    }

    /// The object's local visual description as the client holds it (the server's
    /// `CWPhysicsObject::RecvEvt_UpdateVisualDesc` delta, applied); the scripts copy it with
    /// `gCWPhysicsObject::GetLocalVisualDesc`.
    pub fn set_appearance(&mut self, id: u64, appearance: ac2_weenie::Appearance) {
        self.vm.appearances.insert(id, appearance);
    }

    /// The object's qualities as the scripts left them; quality updates from the server go
    /// through [`ac2_weenie::WeenieQualities::set`] on this.
    pub fn qualities_mut(&mut self, id: u64) -> Option<&mut ac2_weenie::WeenieQualities> {
        self.vm.repositories.get_mut(&id)
    }

    pub fn qualities(&self, id: u64) -> Option<&ac2_weenie::WeenieQualities> {
        self.vm.repositories.get(&id)
    }

    /// The lines the scripts entered for the chat shell since the last call, in order.
    pub fn take_shell_lines(&mut self) -> Vec<String> {
        std::mem::take(&mut self.vm.shell_lines)
    }

    /// The no-draw changes the scripts asked for since the last call (`SetNoDrawPhysics`).
    pub fn take_no_draw(&mut self) -> Vec<(u64, bool)> {
        std::mem::take(&mut self.vm.no_draw)
    }

    /// The physics object changes the scripts asked for, in order (see `Vm::physics`).
    pub fn take_physics(&mut self) -> Vec<(u64, ac2_net::protocol::ObjectAction)> {
        std::mem::take(&mut self.vm.physics)
    }

    /// The collision minimums the scripts set on weenies' collision managers (see
    /// [`Vm::collision_minimums`]), in order.
    pub fn take_collision_minimums(&mut self) -> Vec<(u64, f32, f32)> {
        std::mem::take(&mut self.vm.collision_minimums)
    }

    /// The collision sets the scripts set since the last take, with their objects.
    pub fn take_collision_sets(&mut self) -> Vec<(u64, ac2_physics::CollisionParameters)> {
        std::mem::take(&mut self.vm.collision_sets)
    }

    /// The object went away (`VM::DeleteProcess`).
    pub fn remove_weenie(&mut self, id: u64) {
        if let Some(p) = self.processes.remove(&id) {
            self.vm.owners.remove(&p.instance);
        }
        self.vm.init_data.remove(&id);
        self.vm.repositories.remove(&id);
        self.vm.appearances.remove(&id);
    }

    /// The object's script instance (`CWDistributor::GetInstanceReference`).
    pub fn instance(&self, id: u64) -> Option<u32> {
        self.processes.get(&id).map(|p| p.instance)
    }

    /// `CWDistributor::DispatchInterpMsg`: a new context in the object's process gets a 0
    /// return slot, the arguments (`PackBuffer::UnPackBuffer`) and the instance, then
    /// `VM::CallIFun(instance package, function)`. An interrupted context waits for
    /// [`ScriptHost::resume`]; a finished or crashed one is dropped.
    pub fn dispatch(
        &mut self,
        id: u64,
        function: u32,
        args: &[PackValue],
    ) -> Result<DispatchOutcome, HostError> {
        let instance = self
            .processes
            .get(&id)
            .ok_or(HostError::UnknownObject(id))?
            .instance;
        let cid = self.next_context;
        self.next_context += 1;
        let mut ctx = Context::new(cid, Some(instance));
        ctx.can_yield = true;
        let pushed = ctx.push(0, tag::INT).is_ok() && self.unpack_buffer(&mut ctx, args);
        if !pushed {
            return Err(HostError::BadArguments);
        }
        if ctx.push(instance, tag::REF).is_err() {
            return Err(HostError::BadArguments);
        }
        self.vm.last_fault = None;
        let pkg = self.vm.package_id(instance).unwrap_or(u32::MAX);
        let status = self.vm.call_ifun(&mut ctx, pkg, FunctionId(function));
        Ok(self.finish(id, ctx, status))
    }

    /// `CWDistributor::ResumeContext`: runs a waiting context on.
    pub fn resume(&mut self, id: u64, context: u32) -> Option<DispatchOutcome> {
        let process = self.processes.get_mut(&id)?;
        let at = process.waiting.iter().position(|c| c.cid == context)?;
        let mut ctx = process.waiting.remove(at);
        self.vm.last_fault = None;
        let status = self.vm.execute(&mut ctx);
        Some(self.finish(id, ctx, status))
    }

    /// The DAT reads the scripts waited on since the last call (cache-only data): run each
    /// through `DatScriptData::load` on a worker, then [`ScriptHost::resume_data_waits`].
    pub fn take_data_requests(&mut self) -> Vec<DataRequest> {
        self.vm.data().take_requests()
    }

    /// Runs on every context that stopped for data (each stops again if what it needs is
    /// still missing): the object, and how the context ended this time.
    pub fn resume_data_waits(&mut self) -> Vec<(u64, DispatchOutcome)> {
        let waiting: Vec<(u64, u32)> = self
            .processes
            .iter()
            .flat_map(|(&id, p)| {
                p.waiting
                    .iter()
                    .filter(|c| c.data_wait)
                    .map(move |c| (id, c.cid))
            })
            .collect();
        waiting
            .into_iter()
            .filter_map(|(id, cid)| Some((id, self.resume(id, cid)?)))
            .collect()
    }

    fn finish(&mut self, id: u64, ctx: Context, status: InterpStatus) -> DispatchOutcome {
        let outcome = DispatchOutcome {
            status,
            context: ctx.cid,
            fault: (status == InterpStatus::Crash)
                .then(|| self.vm.last_fault.clone())
                .flatten(),
            data_wait: ctx.data_wait,
        };
        if !matches!(status, InterpStatus::Terminated | InterpStatus::Crash) {
            if let Some(p) = self.processes.get_mut(&id) {
                p.waiting.push(ctx);
            }
        }
        outcome
    }

    /// `PackBuffer::UnPackBuffer`: ints tag 2, 64-bit ints 0x82 and doubles 0x83 (low word
    /// first), floats 3, packages unpacked as new references (tag 1).
    fn unpack_buffer(&mut self, ctx: &mut Context, args: &[PackValue]) -> bool {
        for arg in args {
            let ok = match arg {
                PackValue::Int(v) => ctx.push(*v, tag::INT).is_ok(),
                PackValue::LongInt(v) => ctx.push64(*v, tag::LONG_INT).is_ok(),
                PackValue::Float(v) => ctx.push(v.to_bits(), tag::FLOAT).is_ok(),
                PackValue::LongFloat(v) => ctx.push64(v.to_bits(), tag::DOUBLE).is_ok(),
                PackValue::Reference(r) => {
                    let packed = wire::from_net(r);
                    match self.vm.refs.unpack(&packed, None) {
                        Some(reference) => ctx.push(reference, tag::REF).is_ok(),
                        None => false,
                    }
                }
            };
            if !ok {
                return false;
            }
        }
        true
    }

    /// The calls the scripts sent since the last take, in order.
    pub fn take_server_calls(&mut self) -> Vec<ServerCall> {
        std::mem::take(&mut self.vm.outbox.server_calls)
    }

    /// The UI notices the scripts raised since the last take, in order.
    pub fn take_notices(&mut self) -> Vec<GameplayNotice> {
        std::mem::take(&mut self.vm.outbox.notices)
    }

    /// What the boundary had to drop since the last take.
    /// The `CM_*` notices without a [`GameplayNotice`] type since the last call.
    pub fn take_script_notices(&mut self) -> Vec<ScriptNotice> {
        std::mem::take(&mut self.vm.outbox.script_notices)
    }

    /// The chat-service requests scripts made since the last call.
    pub fn take_chat_requests(&mut self) -> Vec<ChatRequest> {
        std::mem::take(&mut self.vm.outbox.chat_requests)
    }

    /// Which chat services requests can go to (none until the host says).
    pub fn set_chat_services(&mut self, services: ChatServices) {
        self.vm.chat = services;
    }

    /// The world's name for `gTurbineGunUtils::CallEFun_GetWorldName` (the client's world name).
    pub fn set_world_name(&mut self, name: &str) {
        self.vm.world_name = name.to_owned();
    }

    pub fn take_dropped(&mut self) -> Vec<String> {
        std::mem::take(&mut self.vm.outbox.dropped)
    }
}

/// The wire form of a native the VM holds (for hosts that read script objects).
pub fn native_for_wire(
    native_type: u32,
    native: &NativeObject,
) -> Option<ac2_net::protocol::NativeObject> {
    wire::native_to_net(native_type, native).ok()
}

#[cfg(test)]
mod tests;
