use crate::protocol::character::{CharacterIdentity, CharacterSet};
use crate::protocol::object::CreateObject;
use crate::protocol::opcode::Opcode;
use crate::protocol::position::ServerPosition;
use crate::protocol::reader::Reader;
use crate::protocol::ProtocolError;

/// An event received from the server.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    /// The data download phase ended; character selection may begin.
    EndDdd,
    CharacterSet(CharacterSet),
    /// The world refused the selected character (`CharacterError` reason).
    CharacterError(u32),
    /// Character creation outcome: the reason code (1 is success) and the new identity.
    CharGenVerification {
        result: u32,
        identity: Option<CharacterIdentity>,
    },
    /// `ECM_Physics::RecvEvt_CreatePlayer`: the player's object id and region.
    CreatePlayer {
        id: u64,
        region: u32,
    },
    CreateObject(Box<CreateObject>),
    /// The player's qualities (`CBaseQualities`), not unpacked here.
    PlayerDescription(Vec<u8>),
    UpdatePosition(ServerPosition),
    DestroyObject {
        id: u64,
        instance: u16,
    },
    Look {
        id: u64,
        instance: u16,
        yaw: f32,
        pitch: f32,
    },
    LookAt {
        id: u64,
        instance: u16,
        target: u64,
    },
    /// The server ended the character's world session.
    CharExitGame,
    CharacterDeleted(u64),
    /// Any other event, by opcode, with its whole body.
    Other {
        opcode: u32,
        body: Vec<u8>,
    },
}

impl ServerEvent {
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut r = Reader::new(bytes);
        let opcode = r.u32()?;
        let event = match opcode {
            Opcode::END_DDD => Self::EndDdd,
            Opcode::CHARACTER_SET => Self::CharacterSet(CharacterSet::read(&mut r)?),
            Opcode::CHARACTER_ERROR => Self::CharacterError(r.u32()?),
            Opcode::CHAR_GEN_VERIFICATION => {
                let result = r.u32()?;
                let identity = if result == 1 {
                    Some(CharacterIdentity::read(&mut r)?)
                } else {
                    r.rest();
                    None
                };
                r.rest();
                Self::CharGenVerification { result, identity }
            }
            Opcode::CREATE_PLAYER => Self::CreatePlayer {
                id: r.u64()?,
                region: r.u32()?,
            },
            Opcode::CREATE_OBJECT => Self::CreateObject(Box::new(CreateObject::read(&mut r)?)),
            Opcode::PLAYER_DESCRIPTION => Self::PlayerDescription(r.rest().to_vec()),
            Opcode::UPDATE_POSITION => Self::UpdatePosition(ServerPosition::read(&mut r)?),
            Opcode::DESTROY_OBJECT => {
                let id = r.u64()?;
                let instance = r.u16()?;
                r.align()?;
                Self::DestroyObject { id, instance }
            }
            Opcode::LOOK => {
                let id = r.u64()?;
                let instance = r.u16()?;
                r.align()?;
                Self::Look {
                    id,
                    instance,
                    yaw: r.f32()?,
                    pitch: r.f32()?,
                }
            }
            Opcode::LOOK_AT => {
                let id = r.u64()?;
                let instance = r.u16()?;
                r.align()?;
                Self::LookAt {
                    id,
                    instance,
                    target: r.u64()?,
                }
            }
            Opcode::CHAR_EXIT_GAME => Self::CharExitGame,
            Opcode::CHARACTER_DELETION => Self::CharacterDeleted(r.u64()?),
            opcode => Self::Other {
                opcode,
                body: r.rest().to_vec(),
            },
        };
        r.finish()?;
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::ServerEvent;
    use crate::{pack_string, pack_wide_string};

    #[test]
    fn character_set_decodes_identities_and_account() {
        let mut b = 0x0001_0055u32.to_le_bytes().to_vec();
        b.extend(1u32.to_le_bytes());
        b.extend(7u64.to_le_bytes());
        pack_wide_string(&mut b, 0, "Explorer");
        b.extend(0u32.to_le_bytes());
        b.extend(2u32.to_le_bytes());
        b.extend(0x1F00_0023u32.to_le_bytes());
        b.extend([0u32, 0, 5].iter().flat_map(|v| v.to_le_bytes()));
        pack_string(&mut b, 0, b"local");
        b.extend(0u32.to_le_bytes());
        let ServerEvent::CharacterSet(set) = ServerEvent::decode(&b).unwrap() else {
            panic!("character set");
        };
        assert_eq!(set.characters[0].id, 7);
        assert_eq!(set.characters[0].name, "Explorer");
        assert_eq!(set.characters[0].visual.parent, Some(0x1F00_0023));
        assert_eq!((set.allowed, set.account.as_str()), (5, "local"));
        for end in 4..b.len() {
            assert!(ServerEvent::decode(&b[..end]).is_err());
        }
    }

    #[test]
    fn small_events_decode_exactly() {
        let mut create = 0x0001_0060u32.to_le_bytes().to_vec();
        create.extend(1u64.to_le_bytes());
        create.extend(1u32.to_le_bytes());
        assert_eq!(
            ServerEvent::decode(&create).unwrap(),
            ServerEvent::CreatePlayer { id: 1, region: 1 }
        );
        let mut destroy = 0x0001_00C2u32.to_le_bytes().to_vec();
        destroy.extend(2u64.to_le_bytes());
        destroy.extend([3, 0, 0, 0]);
        assert_eq!(
            ServerEvent::decode(&destroy).unwrap(),
            ServerEvent::DestroyObject { id: 2, instance: 3 }
        );
        assert_eq!(
            ServerEvent::decode(&0x0001_003Bu32.to_le_bytes()).unwrap(),
            ServerEvent::EndDdd
        );
        let mut extra = create.clone();
        extra.push(0);
        assert!(ServerEvent::decode(&extra).is_err());
        assert!(matches!(
            ServerEvent::decode(&[1, 2, 3, 4, 5]).unwrap(),
            ServerEvent::Other {
                opcode: 0x0403_0201,
                ..
            }
        ));
    }
}
