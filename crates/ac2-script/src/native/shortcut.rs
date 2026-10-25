//! `ShortcutInfo` (0xAC) as the scripts hold it: every field (`ShortcutInfo::Clear` values by
//! default), of which `StreamPack` packs the type and the one datum the type names.

use ac2_net::protocol::ShortcutInfo as Wire;

/// `ShortcutInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    /// `_type`.
    pub kind: u32,
    /// `_data_iid` (`INVALID_IID` when cleared).
    pub iid: u64,
    /// `_data_enum`.
    pub data_enum: u32,
    /// `_data_dataid`.
    pub data_id: u32,
    /// `_data_str`.
    pub text: Vec<u16>,
}

impl Default for Shortcut {
    /// `ShortcutInfo::ShortcutInfo` / `Clear`.
    fn default() -> Self {
        Self {
            kind: 0,
            iid: u64::MAX,
            data_enum: 0,
            data_id: 0,
            text: Vec::new(),
        }
    }
}

impl Shortcut {
    /// The unpacked form (`ShortcutInfo::StreamPack` on a cleared object).
    pub fn from_net(w: &Wire) -> Self {
        let mut s = Self::default();
        match w {
            Wire::Enum { kind, value } => (s.kind, s.data_enum) = (*kind, *value),
            Wire::Instance(iid) => (s.kind, s.iid) = (0x4000_0002, *iid),
            Wire::Text(text) => (s.kind, s.text) = (0x4000_0003, text.encode_utf16().collect()),
            Wire::DataId(did) => (s.kind, s.data_id) = (0x4000_0006, *did),
            Wire::Empty(kind) => s.kind = *kind,
        }
        s
    }

    /// The packed form (`ShortcutInfo::StreamPack`).
    pub fn to_net(&self) -> Wire {
        match self.kind {
            0x4000_0001 | 0x4000_0004 => Wire::Enum {
                kind: self.kind,
                value: self.data_enum,
            },
            0x4000_0002 => Wire::Instance(self.iid),
            0x4000_0003 => Wire::Text(String::from_utf16_lossy(&self.text)),
            0x4000_0006 => Wire::DataId(self.data_id),
            kind => Wire::Empty(kind),
        }
    }
}
