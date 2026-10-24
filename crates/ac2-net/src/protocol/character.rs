//! Login events: `CharacterSet` (`0x10055`), `MinimalCharacterSet` (`0x100B8`) and the
//! `CharacterIdentity` they and `CharGenVerification` (`0x10057`) carry.

use crate::protocol::reader::Reader;
use crate::protocol::visual::VisualDesc;
use crate::protocol::{Build, ProtocolError};

/// Characters an account may list.
const MAX_CHARACTERS: usize = 64;

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
/// account and the Turbine chat flag. The Legions client (0x00664C80) reads two more 32-bit
/// fields after the chat flag, which the PDB does not name; admin 7148 ends at the chat flag.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSet {
    pub characters: Vec<CharacterIdentity>,
    pub deleted: Vec<CharacterIdentity>,
    pub status: u32,
    pub allowed: u32,
    pub account: String,
    pub turbine_chat: bool,
    /// The two Legions-only words at `CharacterSet + 0x34` and `+ 0x38`.
    pub legions: Option<[u32; 2]>,
}

/// `PSmartArray<CharacterIdentity>::StreamPack`: a u32 count, then each identity.
fn identities(reader: &mut Reader<'_>) -> Result<Vec<CharacterIdentity>, ProtocolError> {
    let count = reader.count()?;
    if count > MAX_CHARACTERS {
        return Err(ProtocolError::Invalid("character count"));
    }
    (0..count)
        .map(|_| CharacterIdentity::read(reader))
        .collect()
}

impl CharacterSet {
    pub(crate) fn read(reader: &mut Reader<'_>, build: Build) -> Result<Self, ProtocolError> {
        Ok(Self {
            characters: identities(reader)?,
            deleted: identities(reader)?,
            status: reader.u32()?,
            allowed: reader.u32()?,
            account: String::from_utf8_lossy(&reader.string()?).into_owned(),
            turbine_chat: reader.u32()? != 0,
            legions: match build {
                Build::Admin7148 => None,
                Build::Legions7160 => Some([reader.u32()?, reader.u32()?]),
            },
        })
    }
}

/// `MinimalCharacterSet::StreamPack`: allowed count, account, names, ids.
#[derive(Debug, Clone, PartialEq)]
pub struct MinimalCharacterSet {
    pub allowed: u32,
    pub account: String,
    pub names: Vec<String>,
    pub ids: Vec<u64>,
}

impl MinimalCharacterSet {
    pub(crate) fn read(reader: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let allowed = reader.u32()?;
        let account = String::from_utf8_lossy(&reader.string()?).into_owned();
        let names = (0..reader.count()?)
            .map(|_| reader.wide_string())
            .collect::<Result<_, _>>()?;
        let ids = (0..reader.count()?)
            .map(|_| reader.u64())
            .collect::<Result<_, _>>()?;
        Ok(Self {
            allowed,
            account,
            names,
            ids,
        })
    }
}
