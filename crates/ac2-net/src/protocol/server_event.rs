//! Events the server sends, decoded as their retail handlers unpack them.

use crate::protocol::character::{CharacterIdentity, CharacterSet, MinimalCharacterSet};
use crate::protocol::dat_event::DatEvent;
use crate::protocol::object::CreateObject;
use crate::protocol::opcode::Opcode;
use crate::protocol::physics_event::ObjectEvent;
use crate::protocol::position::ServerPosition;
use crate::protocol::property::PropertyTypes;
use crate::protocol::qualities::Qualities;
use crate::protocol::quality_event::QualityUpdate;
use crate::protocol::reader::Reader;
use crate::protocol::string_info::StringInfo;
use crate::protocol::types::InstanceIdWithStamp;
use crate::protocol::{Build, ProtocolError};

/// An event received from the server.
#[derive(Debug, Clone, PartialEq)]
pub enum ServerEvent {
    /// Queue 5 data-download events (`CLCache::UseTime`).
    Dat(DatEvent),
    CharacterSet(CharacterSet),
    MinimalCharacterSet(MinimalCharacterSet),
    /// `ECM_Login::RecvEvt_CharacterError`: the `charError` reason.
    CharacterError(u32),
    /// `ECM_Login::RecvEvt_CharGenVerification`: the response, the identity (always unpacked)
    /// and the weenie's character-generation result.
    CharGenVerification {
        response: u32,
        identity: Box<CharacterIdentity>,
        weenie_result: u32,
    },
    /// `ECM_Physics::RecvEvt_CreatePlayer`: the player's object id and region.
    CreatePlayer {
        id: u64,
        region: u32,
    },
    CreateObject(Box<CreateObject>),
    /// `ECM_Login::RecvEvt_PlayerDesc`: the player's qualities (`CBaseQualities::UnPack`).
    PlayerDescription(Box<Qualities>),
    /// `0x10086`.
    UpdatePosition(ServerPosition),
    /// `0x100C1`: a position with cell semantics, same layout.
    PositionCell(ServerPosition),
    /// `ECM_Physics::RecvEvt_DestroyObject`.
    DestroyObject(InstanceIdWithStamp),
    /// `ECM_Physics::RecvEvt_Contain`: the child with its position stamp.
    Contain(InstanceIdWithStamp),
    /// The other per-object physics events.
    Object(ObjectEvent),
    /// `ECM_Qualities::RecvEvt_Update*`: one quality of the player or of an object.
    Quality(QualityUpdate),
    /// `ECM_Physics::RecvEvt_DoFX_Private`: FX ids played on the player (no sender;
    /// `PSmartArray<unsigned long>::StreamPack`, a u32 count then the ids).
    PrivateFx(Vec<u32>),
    /// `ECM_Interp::RecvEvt_InterpCEvent_Broadcast` (`0x10066`, `0x100CA`; for the player) and
    /// `_Cell` (`0x10067`, `0x100C4`; for `object`): a script function id and its packed
    /// arguments (`PackBuffer::UnPack`: a u32 size, then the bytes), handed to
    /// `CWDistributor::RecvEvt_InterpCEvent_Private`.
    Interp {
        object: Option<InstanceIdWithStamp>,
        function: u32,
        arguments: Vec<u8>,
    },
    /// `ECM_Admin::RecvEvt_AccountBooted`: the reason.
    AccountBooted(Box<StringInfo>),
    /// `ECM_Admin::RecvEvt_DisplayStringInfo`: the display type and the text.
    DisplayStringInfo {
        kind: u32,
        text: Box<StringInfo>,
    },
    /// `ECM_Admin::RecvEvt_WorldName`.
    WorldName(Box<StringInfo>),
    /// Queue 4 `0x100D3` (`Turbine::Client::ProcessLogonEventQueue`): the chat client's data
    /// (`IChatClient` receives the length and the bytes after it).
    Chat(Vec<u8>),
    /// `ECM_Login::RecvEvt_CharExitGame`.
    CharExitGame,
    /// `ECM_Login::RecvEvt_CharacterDeletion`.
    CharacterDeleted(u64),
    /// Any other event, by opcode, with its whole body.
    Other {
        opcode: u32,
        body: Vec<u8>,
    },
}

/// `PackBuffer::UnPack`: with fewer than 4 bytes left the buffer stays empty and nothing is
/// read (the rest is left as retail leaves it); a size of `0x10000000` or more, or past the
/// end, fails.
fn pack_buffer(r: &mut Reader<'_>) -> Result<Vec<u8>, ProtocolError> {
    if r.remaining() < 4 {
        r.rest();
        return Ok(Vec::new());
    }
    let size = r.u32()? as usize;
    if size >= 0x1000_0000 {
        return Err(ProtocolError::Invalid("PackBuffer size"));
    }
    Ok(r.take(size)?.to_vec())
}

impl ServerEvent {
    /// Decodes a blob. Login layouts that differ between builds follow `build`.
    pub fn decode(bytes: &[u8], build: Build) -> Result<Self, ProtocolError> {
        Self::decode_from(Reader::new(bytes), bytes, build)
    }

    /// [`ServerEvent::decode`] with the `MasterProperty` types, so property collections (in
    /// light FX) decode too.
    pub fn decode_with_properties(
        bytes: &[u8],
        build: Build,
        types: &dyn PropertyTypes,
    ) -> Result<Self, ProtocolError> {
        Self::decode_from(Reader::with_property_types(bytes, types), bytes, build)
    }

    fn decode_from(mut r: Reader<'_>, bytes: &[u8], build: Build) -> Result<Self, ProtocolError> {
        let opcode = r.u32()?;
        let event = match opcode {
            Opcode::DAT_DATA
            | Opcode::DAT_REQUEST_FAILED
            | Opcode::DAT_INTERROGATION
            | Opcode::DAT_PATCHTIME_PENDING
            | Opcode::BEGIN_DDD
            | Opcode::END_DDD => Self::Dat(DatEvent::read(opcode, &mut r)?),
            Opcode::CHARACTER_SET => Self::CharacterSet(CharacterSet::read(&mut r, build)?),
            Opcode::MIN_CHARACTER_SET => {
                Self::MinimalCharacterSet(MinimalCharacterSet::read(&mut r)?)
            }
            Opcode::CHARACTER_ERROR => {
                let reason = r.u32()?;
                r.align()?;
                Self::CharacterError(reason)
            }
            Opcode::CHAR_GEN_VERIFICATION => {
                let response = r.u32()?;
                r.align()?;
                let identity = Box::new(CharacterIdentity::read(&mut r)?);
                let weenie_result = r.u32()?;
                r.align()?;
                Self::CharGenVerification {
                    response,
                    identity,
                    weenie_result,
                }
            }
            Opcode::CREATE_PLAYER => {
                let id = r.u64()?;
                r.align()?;
                let region = r.u32()?;
                r.align()?;
                Self::CreatePlayer { id, region }
            }
            Opcode::CREATE_OBJECT => Self::CreateObject(Box::new(CreateObject::read(&mut r)?)),
            Opcode::PLAYER_DESCRIPTION => {
                Self::PlayerDescription(Box::new(Qualities::read(&mut r)?))
            }
            Opcode::UPDATE_POSITION => Self::UpdatePosition(ServerPosition::read(&mut r)?),
            Opcode::POSITION_CELL => Self::PositionCell(ServerPosition::read(&mut r)?),
            Opcode::DESTROY_OBJECT => Self::DestroyObject(InstanceIdWithStamp::read(&mut r)?),
            Opcode::CONTAIN => Self::Contain(InstanceIdWithStamp::read(&mut r)?),
            Opcode::DO_FX_PRIVATE => {
                Self::PrivateFx((0..r.count()?).map(|_| r.u32()).collect::<Result<_, _>>()?)
            }
            Opcode::INTERP_C_EVENT_BROADCAST
            | Opcode::INTERP_C_EVENT_BROADCAST_2
            | Opcode::INTERP_C_EVENT_CELL
            | Opcode::INTERP_C_EVENT_CELL_2 => {
                let cell = matches!(
                    opcode,
                    Opcode::INTERP_C_EVENT_CELL | Opcode::INTERP_C_EVENT_CELL_2
                );
                let object = cell
                    .then(|| InstanceIdWithStamp::read(&mut r))
                    .transpose()?;
                let function = r.u32()?;
                r.align()?;
                let arguments = pack_buffer(&mut r)?;
                Self::Interp {
                    object,
                    function,
                    arguments,
                }
            }
            Opcode::ACCOUNT_BOOTED => Self::AccountBooted(Box::new(StringInfo::read(&mut r)?)),
            Opcode::DISPLAY_STRING_INFO => {
                let kind = r.u32()?;
                r.align()?;
                Self::DisplayStringInfo {
                    kind,
                    text: Box::new(StringInfo::read(&mut r)?),
                }
            }
            Opcode::WORLD_NAME => Self::WorldName(Box::new(StringInfo::read(&mut r)?)),
            Opcode::CHAT => {
                let len = r.u32()? as usize;
                let data = r.take(len)?.to_vec();
                r.rest();
                Self::Chat(data)
            }
            Opcode::CHAR_EXIT_GAME => Self::CharExitGame,
            Opcode::CHARACTER_DELETION => {
                let id = r.u64()?;
                r.align()?;
                Self::CharacterDeleted(id)
            }
            opcode => {
                if let Some(event) = ObjectEvent::read(opcode, &mut r) {
                    Self::Object(event?)
                } else if let Some(update) = QualityUpdate::read(opcode, &mut r) {
                    Self::Quality(update?)
                } else {
                    let mut r = Reader::new(bytes);
                    r.u32()?;
                    return Ok(Self::Other {
                        opcode,
                        body: r.rest().to_vec(),
                    });
                }
            }
        };
        r.finish()?;
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::ServerEvent;
    use crate::protocol::types::InstanceIdWithStamp;
    use crate::protocol::Build;
    use crate::{pack_string, pack_wide_string};

    fn character_set(legions: bool) -> Vec<u8> {
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
        if legions {
            b.extend([1u32, 2].iter().flat_map(|v| v.to_le_bytes()));
        }
        b
    }

    #[test]
    fn character_set_follows_the_build_layout() {
        let b = character_set(true);
        let ServerEvent::CharacterSet(set) = ServerEvent::decode(&b, Build::Legions7160).unwrap()
        else {
            panic!("character set");
        };
        assert_eq!(set.characters[0].id, 7);
        assert_eq!(set.characters[0].name, "Explorer");
        assert_eq!(set.characters[0].visual.parent, Some(0x1F00_0023));
        assert_eq!((set.allowed, set.account.as_str()), (5, "local"));
        assert_eq!(set.legions, Some([1, 2]));
        assert!(ServerEvent::decode(&b, Build::Admin7148).is_err());
        let admin = character_set(false);
        assert!(ServerEvent::decode(&admin, Build::Admin7148).is_ok());
        assert!(ServerEvent::decode(&admin, Build::Legions7160).is_err());
        for end in 4..b.len() {
            assert!(ServerEvent::decode(&b[..end], Build::Legions7160).is_err());
        }
    }

    #[test]
    fn small_events_decode_exactly() {
        let mut create = 0x0001_0060u32.to_le_bytes().to_vec();
        create.extend(1u64.to_le_bytes());
        create.extend(1u32.to_le_bytes());
        assert_eq!(
            ServerEvent::decode(&create, Build::Legions7160).unwrap(),
            ServerEvent::CreatePlayer { id: 1, region: 1 }
        );
        let mut destroy = 0x0001_00C2u32.to_le_bytes().to_vec();
        destroy.extend(2u64.to_le_bytes());
        destroy.extend([3, 0, 4, 0]);
        assert_eq!(
            ServerEvent::decode(&destroy, Build::Legions7160).unwrap(),
            ServerEvent::DestroyObject(InstanceIdWithStamp {
                id: 2,
                instance: 3,
                other: 4
            })
        );
        let mut extra = create.clone();
        extra.push(0);
        assert!(ServerEvent::decode(&extra, Build::Legions7160).is_err());
        assert!(matches!(
            ServerEvent::decode(&[1, 2, 3, 4, 5], Build::Legions7160).unwrap(),
            ServerEvent::Other {
                opcode: 0x0403_0201,
                ..
            }
        ));
    }

    fn words(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn interp_events_carry_the_function_and_its_pack_buffer() {
        let mut b = words(&[0x0001_0066, 0x0300_0021, 3]);
        b.extend([7, 8, 9]);
        assert_eq!(
            ServerEvent::decode(&b, Build::Legions7160).unwrap(),
            ServerEvent::Interp {
                object: None,
                function: 0x0300_0021,
                arguments: vec![7, 8, 9]
            }
        );
        for end in 4..b.len() {
            // Fewer than four bytes after the function id leave an empty buffer, as
            // `PackBuffer::UnPack` does; anything shorter than the declared size fails.
            let decoded = ServerEvent::decode(&b[..end], Build::Legions7160);
            if end >= 8 && end < 12 {
                assert!(
                    matches!(decoded, Ok(ServerEvent::Interp { ref arguments, .. }) if arguments.is_empty())
                );
            } else {
                assert!(decoded.is_err(), "{end}");
            }
        }
        let mut cell = 0x0001_00C4u32.to_le_bytes().to_vec();
        cell.extend(0x55u64.to_le_bytes());
        cell.extend([2, 0, 3, 0]);
        cell.extend(words(&[9, 0]));
        let ServerEvent::Interp {
            object,
            function,
            arguments,
        } = ServerEvent::decode(&cell, Build::Legions7160).unwrap()
        else {
            panic!("interp");
        };
        assert_eq!(
            (object.map(|o| o.id), function, arguments.len()),
            (Some(0x55), 9, 0)
        );
    }

    #[test]
    fn admin_messages_and_chat_decode_exactly() {
        let mut b = words(&[0x0001_00C6, 2]);
        b.extend(words(&[0x10, 0x2300_0001, 0]));
        let ServerEvent::DisplayStringInfo { kind, text } =
            ServerEvent::decode(&b, Build::Legions7160).unwrap()
        else {
            panic!("display string");
        };
        assert_eq!(
            (kind, text.string_id, text.table_id),
            (2, 0x10, 0x2300_0001)
        );
        for end in 4..b.len() {
            assert!(ServerEvent::decode(&b[..end], Build::Legions7160).is_err());
        }
        let mut chat = words(&[0x0001_00D3, 2]);
        chat.extend([0xAA, 0xBB, 0xCC]);
        assert_eq!(
            ServerEvent::decode(&chat, Build::Legions7160).unwrap(),
            ServerEvent::Chat(vec![0xAA, 0xBB])
        );
        assert!(ServerEvent::decode(&chat[..9], Build::Legions7160).is_err());
    }
}
