//! `Preferences`: the client's INI cache for the values that are not `PreferenceBank` preferences
//! (`Preferences::InqBool`, `InqString`, `SetBool` and `SetUInt32`). A read looks in the values
//! set this session first, then in the file. `SetBool` and `SetUInt32` only store the value for the
//! session (they add to the change table, and `Shutdown` frees it), so the file never gets them.
//! The one value retail writes to the file on the spot is the graphics quality, by
//! `Render::SaveOverallGraphicsQuality` (see [`Preferences::write_file_value`]).

use std::collections::BTreeMap;

use crate::cnum;
use crate::ini::IniFile;
use crate::names::NameTable;

/// `LoadValue_Internal` reads into a 0x400-byte buffer.
const READ_LIMIT: usize = 1023;

/// The session's values over the file image.
#[derive(Debug, Clone, Default)]
pub struct Preferences {
    file: IniFile,
    /// `m_changeTable`: (section, key) to the value's text, as set this session.
    changes: BTreeMap<(u32, u32), String>,
}

impl Preferences {
    /// The cache over the file image `bytes`.
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            file: IniFile::parse(bytes),
            changes: BTreeMap::new(),
        }
    }

    /// `Preferences::InqBool`: the value as an unsigned integer (`is_uint32`, base 0), or `None`
    /// when it is missing or is not one.
    pub fn inq_bool(&self, names: &dyn NameTable, section: u32, key: u32) -> Option<u32> {
        let text = self.inq_string(names, section, key)?;
        cnum::is_uint32(&text).then(|| cnum::strtoul(text.as_bytes()).value)
    }

    /// `Preferences::InqString`: the session's text, or the file's (cut to [`READ_LIMIT`]).
    pub fn inq_string(&self, names: &dyn NameTable, section: u32, key: u32) -> Option<String> {
        if let Some(text) = self.changes.get(&(section, key)) {
            return Some(text.clone());
        }
        let (section_name, key_name) = (names.section_name(section)?, names.key_name(key)?);
        let text = self.file.get(&section_name, &key_name)?;
        Some(text.chars().take(READ_LIMIT).collect())
    }

    /// `WritePrivateProfileStringA` on the file image: the value is in the file from now on (the
    /// graphics quality is the one retail writes this way).
    pub fn write_file_value(&mut self, section: &str, key: &str, value: &str) {
        self.file.set(section, key, value);
    }

    /// The file's bytes as they stand.
    pub fn file_bytes(&self) -> Vec<u8> {
        self.file.to_bytes()
    }

    /// `Preferences::SetBool`: `1` or `0` for the session.
    pub fn set_bool(&mut self, section: u32, key: u32, value: bool) {
        self.changes
            .insert((section, key), if value { "1" } else { "0" }.to_owned());
    }

    /// `Preferences::SetUInt32`: the decimal text of `value` for the session.
    pub fn set_uint32(&mut self, section: u32, key: u32, value: u32) {
        self.changes.insert((section, key), value.to_string());
    }
}
