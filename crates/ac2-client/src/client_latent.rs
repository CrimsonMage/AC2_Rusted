//! The latent script physics: `gCWPhysicsObject::CallEFun_DoMode` and `CallEFun_DoBehavior` start a
//! mode or behaviour on the weenie's body with the script's context, and the context resumes when
//! the body reports it done (`CPhysicsObj::HandleModeDone`, `HandleBehaviorDone`, which reach the
//! script as `Resume_DoMode` / `Resume_DoBehavior`).

use ac2_net::protocol::ObjectAction;
use ac2_script::host::LatentPhysics;

use crate::client::Client;
use crate::motion::behavior;

/// The error `CPhysicsObj::DoBehavior` reports (`HandleBehaviorDone(context, 0x5d)`) when the
/// behaviour cannot start.
const BEHAVIOR_REFUSED: u32 = 0x5d;

impl Client {
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
}
