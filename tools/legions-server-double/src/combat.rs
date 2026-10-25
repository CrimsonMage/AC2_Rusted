//! The special attack against the double (`WM_Combat`). A skill used from the shortcut bar with a
//! selected creature sends `WM_Combat` 0x17 (the attack, the maneuver, the target); the double
//! answers with the attack state going up (`WM_Combat` 0x1B) and the attack's end (`WM_Combat`
//! 0x19) when the target is one of the double's creatures, so the bar's maneuver goes pending and back. A start (`StartSpecialAttack`) has no
//! forwarder the double can send (see `ac2_net::protocol::CombatMessage`), and no damage or
//! death is applied (the damage text and the monster's health are not answered yet).

use ac2_net::protocol::{CombatMessage, ScriptCall, SpecialAttackRequest};

use crate::{State, CREATURE_ONE, CREATURE_TWO};

impl State {
    /// The server's answer to a combat call the client sends (a request for a special attack).
    pub(crate) fn answer_combat(&mut self, call: &ScriptCall) {
        let Some(request) = SpecialAttackRequest::from_call(call) else {
            return;
        };
        // Only a creature the double knows is attacked; any other target is refused silently.
        if request.target != CREATURE_ONE && request.target != CREATURE_TWO {
            return;
        }
        self.send_combat(&CombatMessage::UpdateAttackState(true));
        self.send_combat(&CombatMessage::EndSpecialAttack(request.attack));
    }

    /// Queues a combat answer (a test can send one directly).
    pub fn send_combat(&mut self, message: &CombatMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }
}
