use crate::protocol::opcode::Opcode;
use crate::protocol::position::ClientPosition;
use crate::protocol::reader::Writer;
use crate::protocol::{QUEUE_CONTROL, QUEUE_ORDERED};

/// An event the client sends through `Proto_UI::SendBuff`, with the queue its sender uses.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientEvent {
    /// `Proto_UI::SendEnterWorld`: the character id and the account, queue 4.
    EnterWorld { character: u64, account: String },
    /// `ECM_Login::SendEvt_ClientSceneRenderingComplete`, queue 3.
    SceneRenderingComplete,
    /// `ECM_Physics::SendEvt_CPosition`, queue 3.
    Position(ClientPosition),
    /// `ECM_Physics::SendEvt_CLookAtDir`: head direction, queue 3.
    LookAtDir { x: f32, y: f32 },
    /// `ECM_Physics::SendEvt_CLookAt`: the target object, queue 3.
    LookAt { target: u64 },
    /// `ECM_Physics::SendEvt_CForceCreate`: ask for an object's create, queue 3.
    ForceCreate { object: u64 },
    /// `Proto_UI::LogOffCharacter`, queue 4.
    LogOffCharacter { character: u64 },
    /// `Proto_UI::SendDeleteCharacter`, queue 4.
    DeleteCharacter { account: String, character: u64 },
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
                QUEUE_ORDERED,
                Opcode::SCENE_RENDERING_COMPLETE.to_le_bytes().to_vec(),
            ),
            Self::Position(position) => (QUEUE_ORDERED, position.encode()),
            Self::LookAtDir { x, y } => {
                let mut w = Writer::event(Opcode::CLIENT_LOOK);
                w.f32(*x).align().f32(*y).align();
                (QUEUE_ORDERED, w.bytes)
            }
            Self::LookAt { target } => {
                let mut w = Writer::event(Opcode::CLIENT_LOOK_AT);
                w.u64(*target).align();
                (QUEUE_ORDERED, w.bytes)
            }
            Self::ForceCreate { object } => {
                let mut w = Writer::event(Opcode::CLIENT_FORCE_CREATE);
                w.u64(*object).align();
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
    fn senders_use_their_retail_queues_and_layouts() {
        assert_eq!(
            ClientEvent::SceneRenderingComplete.encode(),
            (3, vec![0xD1, 0, 1, 0])
        );
        let (queue, bytes) = ClientEvent::LookAtDir { x: 1.0, y: -1.0 }.encode();
        assert_eq!(queue, 3);
        assert_eq!(bytes, [0xC8, 0, 1, 0, 0, 0, 0x80, 0x3F, 0, 0, 0x80, 0xBF]);
        assert_eq!(
            ClientEvent::LookAt { target: 2 }.encode(),
            (3, vec![0xC7, 0, 1, 0, 2, 0, 0, 0, 0, 0, 0, 0])
        );
        assert_eq!(
            ClientEvent::ForceCreate { object: 3 }.encode(),
            (3, vec![0x8F, 0, 1, 0, 3, 0, 0, 0, 0, 0, 0, 0])
        );
        assert_eq!(
            ClientEvent::LogOffCharacter { character: 1 }.encode(),
            (4, vec![0x27, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0])
        );
        let (queue, bytes) = ClientEvent::DeleteCharacter {
            account: "abcd".into(),
            character: 9,
        }
        .encode();
        assert_eq!(queue, 4);
        assert_eq!(
            &bytes[..12],
            &[0x29, 0, 1, 0, 4, 0, 0x2E, 0xB4, 0xB2, 0xA3, 0, 0]
        );
        assert_eq!(&bytes[12..], &9u64.to_le_bytes());
    }
}
