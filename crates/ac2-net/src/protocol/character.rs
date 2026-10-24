use crate::protocol::object::VisualDesc;
use crate::protocol::reader::Reader;
use crate::protocol::ProtocolError;

/// Characters an account may list.
const MAX_CHARACTERS: u32 = 64;

/// `CharacterIdentity::StreamPack`: id, UTF-16 name, seconds greyed out, appearance.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterIdentity {
    pub id: u64,
    pub name: String,
    pub seconds_greyed_out: u32,
    pub visual: VisualDesc,
}

impl CharacterIdentity {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            id: reader.u64()?,
            name: reader.wide_string()?,
            seconds_greyed_out: reader.u32()?,
            visual: VisualDesc::read(reader)?,
        })
    }
}

/// `CharacterSet::StreamPack`: active and deleted identities, status, allowed character count,
/// account and the Turbine chat flag.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSet {
    pub characters: Vec<CharacterIdentity>,
    pub deleted: Vec<CharacterIdentity>,
    pub status: u32,
    pub allowed: u32,
    pub account: String,
    pub turbine_chat: bool,
}

fn identities(reader: &mut Reader<'_>) -> Result<Vec<CharacterIdentity>, ProtocolError> {
    let count = reader.u32()?;
    if count > MAX_CHARACTERS {
        return Err(ProtocolError::Invalid("character count"));
    }
    (0..count)
        .map(|_| CharacterIdentity::read(reader))
        .collect()
}

impl CharacterSet {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            characters: identities(reader)?,
            deleted: identities(reader)?,
            status: reader.u32()?,
            allowed: reader.u32()?,
            account: String::from_utf8_lossy(&reader.string()?).into_owned(),
            turbine_chat: reader.u32()? != 0,
        })
    }
}
