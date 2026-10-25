//! The names of sections, keys and enum labels. Retail reads them from the DAT's enum tables
//! (`EnumMapper::GetString(0x4000003D, section)` for a section, `0x4000003E` for a key, and the
//! enum's own mapper for a label), so the names are game data: this crate takes them through the
//! trait, and the client implements it over the DAT.

/// The name tables the preferences read.
pub trait NameTable {
    /// The section named `id` (`EnumMapper` 0x4000003D).
    fn section_name(&self, id: u32) -> Option<String>;
    /// The key named `id` (`EnumMapper` 0x4000003E).
    fn key_name(&self, id: u32) -> Option<String>;
    /// The name of enum label `label` in mapper `map` (`EnumMapper::GetString`).
    fn enum_name(&self, map: u32, label: u32) -> Option<String>;
    /// The label named `name` in mapper `map` (`EnumMapper::GetEnum`, case-insensitive).
    fn enum_label(&self, map: u32, name: &str) -> Option<u32>;
}

/// A table with no names: nothing has a name, so nothing is read from or written to the file.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoNames;

impl NameTable for NoNames {
    fn section_name(&self, _id: u32) -> Option<String> {
        None
    }
    fn key_name(&self, _id: u32) -> Option<String> {
        None
    }
    fn enum_name(&self, _map: u32, _label: u32) -> Option<String> {
        None
    }
    fn enum_label(&self, _map: u32, _name: &str) -> Option<u32> {
        None
    }
}
