//! The latent script physics: `gCWPhysicsObject::CallEFun_DoMode` and `CallEFun_DoBehavior` start a
//! mode or behaviour on the weenie's body with the script's context, and the context resumes when
//! the body reports it done (`CPhysicsObj::HandleModeDone`, `HandleBehaviorDone`, which reach the
//! script as `Resume_DoMode` / `Resume_DoBehavior`).

use ac2_anim::Ac2AnimEvent;
use ac2_net::protocol::ObjectAction;
use ac2_script::host::LatentPhysics;
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::motion::behavior;

/// The error `CPhysicsObj::DoBehavior` reports (`HandleBehaviorDone(context, 0x5d)`) when the
/// behaviour cannot start.
const BEHAVIOR_REFUSED: u32 = 0x5d;

/// The (context, error) of each script-owned mode or behaviour a body reports done this step.
pub(crate) fn completions(events: &[Ac2AnimEvent]) -> Vec<(u32, u32)> {
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

impl Client {
    /// `Resume_DoMode` / `Resume_DoBehavior` for `object`'s waiting context.
    pub(crate) fn resume_latent(
        &mut self,
        object: u64,
        context: u32,
        error: u32,
        out: &mut Vec<SessionEvent>,
    ) {
        ac2_script::host::latent::resume(self, object, context, error, out);
    }

    /// The body's start of a latent mode or behaviour (`CPhysicsObj::DoMode` / `DoBehavior`):
    /// `None` while the body runs it, `Some(error)` when it ends at once. An object without a
    /// player body is not modelled, so it finishes at once.
    pub(crate) fn start_latent_on_body(&mut self, start: &LatentPhysics) -> Option<u32> {
        let body = self
            .player
            .as_mut()
            .filter(|p| p.id == start.object)
            .and_then(|p| p.body.as_mut());
        let Some(body) = body else {
            return Some(0);
        };
        match &start.action {
            ObjectAction::DoMode(mode) => {
                (!body.physics.do_mode(start.context, *mode)).then_some(0)
            }
            ObjectAction::DoBehavior(params) => {
                let mut params = behavior(params);
                params.context = start.context;
                (!body.physics.do_behavior(&params)).then_some(BEHAVIOR_REFUSED)
            }
            _ => None,
        }
    }

    /// The frame's completions on an object's body: the waiting contexts resume, and what they
    /// produce joins the next frame's events (`script_outbox`).
    pub(crate) fn finish_latent(&mut self, object: u64, completions: Vec<(u32, u32)>) {
        if completions.is_empty() {
            return;
        }
        let mut out = Vec::new();
        for (context, error) in completions {
            self.resume_latent(object, context, error, &mut out);
        }
        self.script_outbox.append(&mut out);
    }
}
