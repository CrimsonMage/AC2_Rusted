//! The latent script physics: `gCWPhysicsObject::CallEFun_DoMode` and `CallEFun_DoBehavior` start
//! the mode or behaviour on the weenie's body and stop the context until it is done. The body
//! reports the end with `CPhysicsObj::HandleModeDone` / `HandleBehaviorDone` (the context and an
//! error, 0 when done; `CWInterface::HandleModeDone` and `CWInterface::HandleBehaviorDone`
//! forward to `gCWPhysicsObject::Resume_DoMode` / `Resume_DoBehavior`), and
//! [`ScriptHost::resume_latent`] runs `ggmCWInterface::Resume_DoMode` /
//! `Resume_DoBehavior`: the out parameter takes the error, the efun's frame closes and the context
//! runs on.

use ac2_net::protocol::ObjectAction;

use super::port::ScriptPort;
use super::{DispatchOutcome, ScriptHost};
use crate::vm::{tag, Fault};
use ac2_anim::Ac2AnimEvent;
use ac2_weenie::SessionEvent;

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

/// Starts the modes and behaviours the scripts made latent. The body takes each with its context
/// through [`ScriptPort::start_latent`]; one it refuses (an error comes back at once), and one on
/// an object without a body, finishes at once and resumes the script ([`resume`]).
pub fn start_all<P: ScriptPort + ?Sized>(
    port: &mut P,
    starts: Vec<LatentPhysics>,
    out: &mut Vec<SessionEvent>,
) {
    for start in starts {
        if let Some(error) = port.start_latent(&start) {
            resume(port, start.object, start.context, error, out);
        }
    }
}

/// `Resume_DoMode` / `Resume_DoBehavior`: the script's context takes `error` and runs on, its
/// output routed as a call of the VM (`outcome`).
pub fn resume<P: ScriptPort + ?Sized>(
    port: &mut P,
    object: u64,
    context: u32,
    error: u32,
    out: &mut Vec<SessionEvent>,
) {
    let Some(scripts) = port.script_state() else {
        return;
    };
    let Some(outcome) = scripts.host.resume_latent(object, context, error) else {
        return;
    };
    super::outcome::outcome(port, object, None, Ok(outcome), out);
}

/// The (context, error) of each script-owned mode or behaviour a body reports done this step
/// (`CPhysicsObj::HandleModeDone` / `HandleBehaviorDone`; context 0 is no script's).
pub fn completions(events: &[Ac2AnimEvent]) -> Vec<(u32, u32)> {
    events
        .iter()
        .filter_map(|event| match *event {
            Ac2AnimEvent::ModeDone { context, error }
            | Ac2AnimEvent::BehaviorDone { context, error }
                if context != 0 =>
            {
                Some((context, error))
            }
            _ => None,
        })
        .collect()
}

/// The frame's completions on object `object`'s body: each waiting context resumes
/// ([`resume`]), and what they produce goes to the story flow (`push_outbox`).
pub fn finish<P: ScriptPort + ?Sized>(port: &mut P, object: u64, completions: Vec<(u32, u32)>) {
    if completions.is_empty() {
        return;
    }
    let mut out = Vec::new();
    for (context, error) in completions {
        resume(port, object, context, error, &mut out);
    }
    port.push_outbox(out);
}
