//! The special attack and the attack state (`WM_Combat`, package 0x204).
//!
//! Using a skill from the shortcut bar (`gmCWInterface::UseShortcut`, a `ShortcutInfo` enum)
//! raises `CM_Combat::SendNotice_RequestSpecialAttack` and sends `WM_Combat` 0x17 (the stub
//! `CCombatSystem` 0x4 packs): the attack id (1 for the auto attack), the maneuver (the skill's
//! id), and the target (the selection, a long int), in that pack order.
//!
//! The server's answers are forwarders the player's script reaches: `WM_Combat` 0x19 ->
//! `CPlayer` 0xB2 -> `CCombatSystem` 0xF -> `SendNotice_EndSpecialAttack` (one int, the attack);
//! `WM_Combat` 0x28 -> `CPlayer` 0x1C9 -> `SendNotice_ToggleSpecialAttack` (the toggled flag, then
//! the maneuver, packed in that order); `WM_Combat` 0x1B -> `CPlayer` 0x2 ->
//! `SendNotice_UpdateAttackState` (one int). A special attack's start (`CCombatSystem` 0x2,
//! `SendNotice_StartSpecialAttack`) is reached from `CPlayer` 0xD only, and no `WM_*` forwarder
//! calls that function, so the codec has no start message.

use super::calls::ScriptCall;
use super::Args;
use crate::protocol::pack::PackValue;
use crate::protocol::ProtocolError;

/// C->S, `WM_Combat` 0x17: the special attack request (attack, maneuver, target).
pub const REQUEST_SPECIAL_ATTACK: u32 = 0x8204_000B;
/// S->C, `WM_Combat` 0x19 -> `CPlayer` 0xB2: the attack ended.
pub const END_SPECIAL_ATTACK: u32 = 0x8204_0019;
/// S->C, `WM_Combat` 0x28 -> `CPlayer` 0x1C9: a maneuver toggled.
pub const TOGGLE_SPECIAL_ATTACK: u32 = 0x8204_0028;
/// S->C, `WM_Combat` 0x1B -> `CPlayer` 0x2: the attack state.
pub const UPDATE_ATTACK_STATE: u32 = 0x8204_001B;

/// The request a shortcut's skill makes: `attack` (the attack id), `maneuver` (the skill), and the
/// `target` the selection holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialAttackRequest {
    pub attack: u32,
    pub maneuver: u32,
    pub target: u64,
}

impl SpecialAttackRequest {
    /// The client's `WM_Combat` 0x17 call, its values in the stub's pack order.
    pub fn to_call(self) -> ScriptCall {
        ScriptCall::new(
            REQUEST_SPECIAL_ATTACK,
            vec![
                PackValue::Int(self.attack),
                PackValue::Int(self.maneuver),
                PackValue::LongInt(self.target),
            ],
        )
        .expect("WM_Combat 0x17's layout")
    }

    /// The request a client call carries, or `None` for another call.
    pub fn from_call(call: &ScriptCall) -> Option<Self> {
        match call.arguments[..] {
            [PackValue::Int(attack), PackValue::Int(maneuver), PackValue::LongInt(target)]
                if call.function == REQUEST_SPECIAL_ATTACK =>
            {
                Some(Self {
                    attack,
                    maneuver,
                    target,
                })
            }
            _ => None,
        }
    }
}

/// A server answer about the attack state or a special attack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatMessage {
    /// [`END_SPECIAL_ATTACK`]: the attack `attack` ended (`SendNotice_EndSpecialAttack`).
    EndSpecialAttack(u32),
    /// [`TOGGLE_SPECIAL_ATTACK`]: `maneuver` toggled on (`toggled`) or off
    /// (`SendNotice_ToggleSpecialAttack`).
    ToggleSpecialAttack { maneuver: u32, toggled: bool },
    /// [`UPDATE_ATTACK_STATE`]: the player is attacking or not (`SendNotice_UpdateAttackState`).
    UpdateAttackState(bool),
}

impl CombatMessage {
    /// The function id the message is sent to.
    pub fn function(&self) -> u32 {
        match self {
            Self::EndSpecialAttack(_) => END_SPECIAL_ATTACK,
            Self::ToggleSpecialAttack { .. } => TOGGLE_SPECIAL_ATTACK,
            Self::UpdateAttackState(_) => UPDATE_ATTACK_STATE,
        }
    }

    /// The packed values in wire order (see the module docs for each message's order).
    pub fn arguments(&self) -> Vec<PackValue> {
        match *self {
            Self::EndSpecialAttack(attack) => vec![PackValue::Int(attack)],
            Self::ToggleSpecialAttack { maneuver, toggled } => {
                vec![PackValue::Int(u32::from(toggled)), PackValue::Int(maneuver)]
            }
            Self::UpdateAttackState(attacking) => vec![PackValue::Int(u32::from(attacking))],
        }
    }

    /// The answer from its function id and packed arguments, or `None` for another id.
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        let read = |f: fn(&mut Args) -> Result<Self, ProtocolError>| {
            Args::new(arguments).and_then(|mut a| {
                let message = f(&mut a)?;
                a.finish()?;
                Ok(message)
            })
        };
        Some(match function {
            END_SPECIAL_ATTACK => read(|a| Ok(Self::EndSpecialAttack(a.int()?))),
            TOGGLE_SPECIAL_ATTACK => read(|a| {
                let toggled = a.int()? != 0;
                Ok(Self::ToggleSpecialAttack {
                    maneuver: a.int()?,
                    toggled,
                })
            }),
            UPDATE_ATTACK_STATE => read(|a| Ok(Self::UpdateAttackState(a.int()? != 0))),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn the_request_packs_attack_maneuver_and_target_in_stub_order() {
        let request = SpecialAttackRequest {
            attack: 1,
            maneuver: 6,
            target: 0x8000_0000_0000_0A01,
        };
        let call = request.to_call();
        let (queue, bytes) = call.to_event().encode();
        assert_eq!(queue, 3);
        // The skill's use against `CREATURE_ONE` (the VM's bytes): int 1, int 6, long int.
        assert_eq!(
            bytes,
            words(&[
                0x0001_0053,
                0x8204_000B,
                28,
                1,
                1,
                1,
                6,
                2,
                0x0A01,
                0x8000_0000
            ])
        );
        assert_eq!(SpecialAttackRequest::from_call(&call), Some(request));
    }

    #[test]
    fn the_answers_decode_from_their_packed_values() {
        let end = words(&[1, 6]);
        assert_eq!(
            CombatMessage::decode(END_SPECIAL_ATTACK, &end),
            Some(Ok(CombatMessage::EndSpecialAttack(6)))
        );
        // The toggle packs the flag first, then the maneuver.
        let toggle = words(&[1, 1, 1, 6]);
        assert_eq!(
            CombatMessage::decode(TOGGLE_SPECIAL_ATTACK, &toggle),
            Some(Ok(CombatMessage::ToggleSpecialAttack {
                maneuver: 6,
                toggled: true
            }))
        );
        let state = words(&[1, 1]);
        assert_eq!(
            CombatMessage::decode(UPDATE_ATTACK_STATE, &state),
            Some(Ok(CombatMessage::UpdateAttackState(true)))
        );
        assert!(CombatMessage::decode(0x8204_0008, &[]).is_none());
    }

    #[test]
    fn the_answers_encode_to_the_forwarders_values() {
        for message in [
            CombatMessage::EndSpecialAttack(6),
            CombatMessage::ToggleSpecialAttack {
                maneuver: 6,
                toggled: false,
            },
            CombatMessage::UpdateAttackState(true),
        ] {
            let bytes = crate::protocol::pack::pack_values(&message.arguments());
            assert_eq!(
                CombatMessage::decode(message.function(), &bytes),
                Some(Ok(message))
            );
        }
    }
}
