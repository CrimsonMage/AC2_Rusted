//! The session's gameplay entry points: the IFuns the native UI calls on the player's weenie
//! (`gmCWInterface::SetSelection`, `StartExamination`, ...), run by the
//! [`PlayerScript`] stand-in, whose messages go out as `InterpSEvent`s.

use ac2_net::protocol::{ExaminationRequest, ScriptMessage};

use super::{OnlineSession, SessionEvent};
use crate::player_script::{PlayerScript, ScriptOutput};

/// `CWInterface::IsCreature`: the weenie type (int quality 0x14) has all of 0x4F.
const CREATURE_TYPE: i32 = 0x4F;

impl OnlineSession {
    /// The player's script state (selection, examination).
    pub fn player_script(&self) -> &PlayerScript {
        &self.script
    }

    /// `gmCWInterface::SetSelection` (IFun `SetSelection`): select an object, or none with
    /// [`crate::NO_SELECTION`]. Returns the notices the script raised.
    pub fn select(&mut self, id: u64) -> Vec<SessionEvent> {
        let combat_target = self.is_combat_target(id);
        let out = self.script.set_selection(id, combat_target);
        self.apply_script(out)
    }

    /// `gmCWInterface::SetExaminationRequest`.
    pub fn examine(&mut self, request: ExaminationRequest) -> Vec<SessionEvent> {
        let out = self.script.examine(request);
        self.apply_script(out)
    }

    /// `gmCWInterface::StartExamination`: the examination panel opened (`context` is the
    /// panel's word, `gmExaminationUI` +0xB8 in Legions).
    pub fn start_examination(&mut self, context: u32) -> Vec<SessionEvent> {
        let out = self.script.start_examination(context);
        self.apply_script(out)
    }

    /// `gmCWInterface::StopExamination`: the examination panel closed.
    pub fn stop_examination(&mut self) -> Vec<SessionEvent> {
        let out = self.script.stop_examination();
        self.apply_script(out)
    }

    /// `CCombatSystem` 0x1 as far as the client knows it: another object whose weenie is a
    /// creature (`CWInterface::IsCreature`). Retail also asks `CCombatSystem` 0x5 and the
    /// weenie type system.
    fn is_combat_target(&self, id: u64) -> bool {
        Some(id) != self.world.player()
            && self
                .world
                .objects
                .get(id)
                .and_then(|o| o.qualities.ints.get(&0x14))
                .is_some_and(|t| t & CREATURE_TYPE == CREATURE_TYPE)
    }

    /// A known `InterpCEvent` for the player's scripts; `None` when the function is not one
    /// the stand-in handles (the caller reports it as a raw script event).
    pub(super) fn receive_script(
        &mut self,
        function: u32,
        arguments: &[u8],
    ) -> Option<Vec<SessionEvent>> {
        match ScriptMessage::decode(function, arguments)? {
            Ok(message) => {
                let out = self.script.receive(message);
                Some(self.apply_script(out))
            }
            Err(error) => Some(vec![SessionEvent::Rejected(format!(
                "script function {function:#010x}: {error}"
            ))]),
        }
    }

    fn apply_script(&mut self, out: ScriptOutput) -> Vec<SessionEvent> {
        for request in out.requests {
            self.send(request.to_event());
        }
        out.notices.into_iter().map(SessionEvent::Notice).collect()
    }
}
