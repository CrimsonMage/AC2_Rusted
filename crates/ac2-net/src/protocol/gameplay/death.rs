//! The player's death and respawn (`WM_Death`, package 0x207).
//!
//! The server's death flag is `WM_Death` 0x33 (one int): `CPlayer` 0x107 -> `CDeathSystem` 0x3
//! sets the bool quality 0x121 (`Death_CanResurrect`, through `Agent` 0x3F) and raises
//! `SendNotice_UpdateDeathState` with the flag. The client's respawn request is the `WM_Death` 0x36
//! stub (no arguments), which `CPlayer` 0x125 sends (it is the call the player's script makes on
//! `gmCWInterface::RequestResurrect`, IFun 0xB4006B). The respawn's answer (the lifestone's teleport
//! and the flag down again) is the server's; the codec only names the two messages.

use super::calls::ScriptCall;
use super::Args;
use crate::protocol::pack::PackValue;
use crate::protocol::ProtocolError;

/// S->C, `WM_Death` 0x33: the death flag (`UpdateDeathState`).
pub const PLAYER_DEATH: u32 = 0x8207_0033;
/// C->S, `WM_Death` 0x36: the respawn request (no arguments).
pub const RESURRECT_REQUEST: u32 = 0x8207_000E;

/// The server's death answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathMessage {
    /// [`PLAYER_DEATH`]: the player is dead (`true`) or alive again (`false`).
    PlayerDeath(bool),
}

impl DeathMessage {
    /// The function id the message is sent to.
    pub fn function(&self) -> u32 {
        PLAYER_DEATH
    }

    /// The packed values in wire order: the flag.
    pub fn arguments(&self) -> Vec<PackValue> {
        match *self {
            Self::PlayerDeath(dead) => vec![PackValue::Int(u32::from(dead))],
        }
    }

    /// The answer from its function id and packed arguments, or `None` for another id.
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        if function != PLAYER_DEATH {
            return None;
        }
        Some(Args::new(arguments).and_then(|mut a| {
            let dead = a.int()? != 0;
            a.finish()?;
            Ok(Self::PlayerDeath(dead))
        }))
    }

    /// The respawn request as the client's call: `WM_Death` 0x36, no arguments.
    pub fn resurrect_request() -> ScriptCall {
        ScriptCall::new(RESURRECT_REQUEST, Vec::new()).expect("WM_Death 0x36's layout")
    }

    /// Whether `call` is the respawn request.
    pub fn is_resurrect_request(call: &ScriptCall) -> bool {
        call.function == RESURRECT_REQUEST && call.arguments.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_death_flag_round_trips_and_the_request_has_no_values() {
        for dead in [true, false] {
            let message = DeathMessage::PlayerDeath(dead);
            let bytes = crate::protocol::pack::pack_values(&message.arguments());
            assert_eq!(
                DeathMessage::decode(PLAYER_DEATH, &bytes),
                Some(Ok(message))
            );
        }
        let (queue, bytes) = DeathMessage::resurrect_request().to_event().encode();
        assert_eq!(queue, 3);
        assert_eq!(bytes.len(), 12);
        assert!(DeathMessage::is_resurrect_request(
            &DeathMessage::resurrect_request()
        ));
        assert!(DeathMessage::decode(0x8204_0019, &[]).is_none());
    }
}
