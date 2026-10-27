//! The latent script physics: `gCWPhysicsObject::CallEFun_DoMode` and `CallEFun_DoBehavior` start
//! the mode or behaviour on the weenie's body and stop the context until it is done. The body
//! reports the end with `CPhysicsObj::HandleModeDone` / `HandleBehaviorDone` (the context and an
//! error, 0 when done), and [`ScriptHost::resume_latent`] runs `ggmCWInterface::Resume_DoMode` /
//! `Resume_DoBehavior`: the out parameter takes the error, the efun's frame closes and the context
//! runs on.

use ac2_net::protocol::ObjectAction;

use super::{DispatchOutcome, ScriptHost};
use crate::vm::{tag, Fault};

/// `gCWPhysicsObject::CallEFun_DoMode` (the efun the context waits in, `m_efun` 0x800015).
pub(super) const DO_MODE: u32 = 0x0080_0015;
/// `gCWPhysicsObject::CallEFun_DoBehavior` (`m_efun` 0x80001B).
pub(super) const DO_BEHAVIOR: u32 = 0x0080_001B;

/// A mode or behaviour the script started on an object's body, with the context that waits for
/// it to finish.
#[derive(Debug, Clone, PartialEq)]
pub struct LatentPhysics {
    pub object: u64,
    pub context: u32,
    /// `ObjectAction::DoMode` or `DoBehavior`.
    pub action: ObjectAction,
}

impl ScriptHost {
    /// The latent starts the scripts made since the last call, in order.
    pub fn take_latent_physics(&mut self) -> Vec<LatentPhysics> {
        std::mem::take(&mut self.vm.latent_physics)
    }

    /// `Resume_DoMode(obj, cid, error)` / `Resume_DoBehavior(obj, cid, error)`: the context
    /// `context` of object `id` waits in a DoMode or DoBehavior efun. Its out parameter takes
    /// `error` (0 when the mode or behaviour is done), the efun's frame closes and the context runs
    /// on. `None` when no such context waits (retail returns 0 and changes nothing).
    pub fn resume_latent(&mut self, id: u64, context: u32, error: u32) -> Option<DispatchOutcome> {
        let process = self.processes.get_mut(&id)?;
        let at = process
            .waiting
            .iter()
            .position(|c| c.cid == context && (c.efun == DO_MODE || c.efun == DO_BEHAVIOR))?;
        let mut ctx = process.waiting.remove(at);
        let out = ctx.arg(3)?;
        ctx.efun = 0;
        self.vm.last_fault = None;
        self.vm.write_through(&mut ctx, out, error, tag::INT);
        let status = match ctx.leave() {
            Ok(()) => self.vm.execute(&mut ctx),
            Err(_) => self.vm.fault(Fault::Stack),
        };
        Some(self.finish(id, ctx, status))
    }
}
