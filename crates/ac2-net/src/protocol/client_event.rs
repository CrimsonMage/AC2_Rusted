use crate::protocol::opcode::Opcode;
use crate::protocol::position::ClientPosition;
use crate::protocol::reader::Writer;
use crate::protocol::{QUEUE_CONTROL, QUEUE_ORDERED};

/// An event the client sends, and the queue `Proto_UI` sends it on.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientEvent {
    /// `Proto_UI::SendEnterWorld`: the character id and the account.
    EnterWorld {
        character: u64,
        account: String,
    },
    /// The client finished loading the scene around its player.
    SceneRenderingComplete,
    Position(ClientPosition),
    /// Head look: yaw and pitch in degrees.
    Look {
        yaw: f32,
        pitch: f32,
    },
    /// `ECM_Physics::SendEvt_CLookAt`: the target object, or 0 to clear.
    LookAt {
        target: u64,
    },
    /// `Proto_UI::LogOffCharacter`.
    LogOffCharacter {
        character: u64,
    },
    /// `Proto_UI::SendDeleteCharacter`.
    DeleteCharacter {
        account: String,
        character: u64,
    },
}

impl ClientEvent {
    /// The event's bytes and its queue.
    pub fn encode(&self) -> (u16, Vec<u8>) {
        match self {
            Self::EnterWorld { character, account } => {
                let mut w = Writer::event(Opcode::ENTER_WORLD);
                w.u64(*character).string(account.as_bytes());
                (QUEUE_CONTROL, w.bytes)
            }
            Self::SceneRenderingComplete => (
                QUEUE_CONTROL,
                Opcode::SCENE_RENDERING_COMPLETE.to_le_bytes().to_vec(),
            ),
            Self::Position(position) => (QUEUE_ORDERED, position.encode()),
            Self::Look { yaw, pitch } => {
                let mut w = Writer::event(Opcode::CLIENT_LOOK);
                w.f32(*yaw).f32(*pitch);
                (QUEUE_ORDERED, w.bytes)
            }
            Self::LookAt { target } => {
                let mut w = Writer::event(Opcode::CLIENT_LOOK_AT);
                w.u64(*target);
                (QUEUE_ORDERED, w.bytes)
            }
            Self::LogOffCharacter { character } => {
                let mut w = Writer::event(Opcode::CHAR_EXIT_GAME);
                w.u64(*character);
                (QUEUE_CONTROL, w.bytes)
            }
            Self::DeleteCharacter { account, character } => {
                let mut w = Writer::event(Opcode::CHARACTER_DELETION);
                w.string(account.as_bytes()).u64(*character);
                (QUEUE_CONTROL, w.bytes)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientEvent;

    #[test]
    fn enter_world_matches_send_enter_world() {
        // "abcd" nibble swapped 16 26 36 46, XORed with 38 92 84 E5.
        let (queue, bytes) = ClientEvent::EnterWorld {
            character: 0x0102_0304_0506_0708,
            account: "abcd".into(),
        }
        .encode();
        assert_eq!(queue, 4);
        assert_eq!(
            bytes,
            [0x2B, 0, 1, 0, 8, 7, 6, 5, 4, 3, 2, 1, 4, 0, 0x2E, 0xB4, 0xB2, 0xA3, 0, 0]
        );
    }

    #[test]
    fn control_and_movement_events_use_their_queues() {
        assert_eq!(
            ClientEvent::SceneRenderingComplete.encode(),
            (4, vec![0xD1, 0, 1, 0])
        );
        assert_eq!(
            ClientEvent::Look {
                yaw: 0.0,
                pitch: 0.0
            }
            .encode()
            .0,
            3
        );
        let (queue, bytes) = ClientEvent::LogOffCharacter { character: 1 }.encode();
        assert_eq!((queue, bytes.len()), (4, 12));
    }
}
