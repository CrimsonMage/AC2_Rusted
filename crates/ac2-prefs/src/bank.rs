//! `PreferenceBank`: the preferences the client holds, in the order they were registered, over
//! the INI file they are read from and written to.
//!
//! Retail's bank keeps a list (`mPreferences`), a monitor per section (`mSectionTable`) and a
//! handler per key (`mKeyTable`, `Link`). The section operations (`Submit`, `Apply`, `Recall`,
//! `Reload`) call the handler of each preference they touch, grouped by handler. Here they return
//! the keys they touched, in list order, and the caller dispatches them; the owner of each key
//! (the subsystem that reads it) is recorded by [`PreferenceBank::link`].
//!
//! Loading happens when a preference is registered (`PreferenceBank::Set` clones it and calls
//! `Preference::Load`); saving writes every preference in list order (`PreferenceBank::Save`).

use std::collections::BTreeMap;

use crate::ini::IniFile;
use crate::names::NameTable;
use crate::pref::Pref;

/// Text longer than this is cut when read (`Preference::Load` reads into a 100-byte buffer).
pub const LOAD_LIMIT: usize = 99;

/// `PreferenceBank`: the preferences, the file image they load from, and their owners.
#[derive(Debug, Clone, Default)]
pub struct PreferenceBank {
    prefs: Vec<Pref>,
    file: IniFile,
    owners: BTreeMap<u32, Vec<&'static str>>,
}

impl PreferenceBank {
    /// A bank over the file image `bytes` (the INI file as read; empty when there is none).
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            prefs: Vec::new(),
            file: IniFile::parse(bytes),
            owners: BTreeMap::new(),
        }
    }

    /// The file's bytes as they stand (what [`PreferenceBank::save`] last wrote).
    pub fn file_bytes(&self) -> Vec<u8> {
        self.file.to_bytes()
    }

    /// `WritePrivateProfileStringA` on the file image, for a value that is not a preference.
    pub fn write_file_value(&mut self, section: &str, key: &str, value: &str) {
        self.file.set(section, key, value);
    }

    /// `PreferenceBank::Set`: a new preference is loaded from the file and added; one already
    /// registered takes the new one's defaults, range and choices (`Copy`) and keeps its place.
    /// `None` when the preference has no key (or its section or key has no name).
    pub fn register(&mut self, def: Pref, names: &dyn NameTable) -> Option<usize> {
        if names.section_name(def.section).is_none() || names.key_name(def.key).is_none() {
            return None;
        }
        if let Some(index) = self.index_of(def.key) {
            self.prefs[index].copy_from(&def);
            return Some(index);
        }
        let mut pref = def;
        self.load_into(&mut pref, names);
        self.prefs.push(pref);
        Some(self.prefs.len() - 1)
    }

    /// `Preference::Load`: the preference's text from the file (its default when the file has
    /// none), cut to [`LOAD_LIMIT`], then applied: the loaded value is the saved one.
    fn load_into(&self, pref: &mut Pref, names: &dyn NameTable) {
        let (Some(section), Some(key)) =
            (names.section_name(pref.section), names.key_name(pref.key))
        else {
            return;
        };
        let default = pref.default_text(names);
        let text = self.file.get(&section, &key).unwrap_or(default);
        let cut = truncate_bytes(&text, LOAD_LIMIT);
        pref.set_text(&cut, names);
        pref.apply();
    }

    /// `PreferenceBank::Save`: every preference is written to the file image in list order, as
    /// `Preference::Save` writes it (`WritePrivateProfileStringA`).
    pub fn save(&mut self, names: &dyn NameTable) {
        for pref in &self.prefs {
            let (Some(section), Some(key)) =
                (names.section_name(pref.section), names.key_name(pref.key))
            else {
                continue;
            };
            self.file.set(&section, &key, &pref.value_text(names));
        }
    }

    /// `PreferenceBank::Find`: the preference with `key`.
    pub fn find(&self, key: u32) -> Option<&Pref> {
        self.index_of(key).map(|i| &self.prefs[i])
    }

    /// The preference with `key`, to change its value.
    pub fn find_mut(&mut self, key: u32) -> Option<&mut Pref> {
        self.index_of(key).map(|i| &mut self.prefs[i])
    }

    /// Every preference, in list order.
    pub fn all(&self) -> &[Pref] {
        &self.prefs
    }

    /// `PreferenceBank::List`: the keys of `section`, in list order.
    pub fn list(&self, section: u32) -> Vec<u32> {
        self.prefs
            .iter()
            .filter(|p| p.section == section)
            .map(|p| p.key)
            .collect()
    }

    /// `PreferenceBank::Submit`: the keys of `section` whose value changed (the handlers are told,
    /// nothing is applied).
    pub fn submit(&self, section: u32) -> Vec<u32> {
        self.prefs
            .iter()
            .filter(|p| p.section == section && p.changed())
            .map(|p| p.key)
            .collect()
    }

    /// `PreferenceBank::Apply(section)`: each changed preference of `section` is applied; the keys
    /// applied are returned.
    pub fn apply(&mut self, section: u32) -> Vec<u32> {
        let mut keys = Vec::new();
        for pref in self.prefs.iter_mut().filter(|p| p.section == section) {
            if pref.changed() {
                pref.apply();
                keys.push(pref.key);
            }
        }
        keys
    }

    /// `PreferenceBank::Apply(preference)`: one preference is applied.
    pub fn apply_key(&mut self, key: u32) -> bool {
        match self.find_mut(key) {
            Some(pref) => {
                pref.apply();
                true
            }
            None => false,
        }
    }

    /// `PreferenceBank::Recall`: every preference of `section` goes back to its applied value;
    /// the keys are returned.
    pub fn recall(&mut self, section: u32) -> Vec<u32> {
        let mut keys = Vec::new();
        for pref in self.prefs.iter_mut().filter(|p| p.section == section) {
            pref.recall();
            keys.push(pref.key);
        }
        keys
    }

    /// The section's defaults become its values (`Reset` of each preference, which the expander's
    /// `Default` calls); the applied values are kept, so the section then needs an `apply`. The keys
    /// are returned.
    pub fn reset_section(&mut self, section: u32) -> Vec<u32> {
        let mut keys = Vec::new();
        for pref in self.prefs.iter_mut().filter(|p| p.section == section) {
            pref.reset();
            keys.push(pref.key);
        }
        keys
    }

    /// `PreferenceBank::Reload`: the monitors of `section` read its preferences again (the keys).
    pub fn reload(&self, section: u32) -> Vec<u32> {
        self.list(section)
    }

    /// `PreferenceBank::Link`: `owner` reads the preference `key`. An owner is recorded once.
    pub fn link(&mut self, key: u32, owner: &'static str) {
        let owners = self.owners.entry(key).or_default();
        if !owners.contains(&owner) {
            owners.push(owner);
        }
    }

    /// The subsystems that read the preference `key`, in the order they linked it.
    pub fn owners(&self, key: u32) -> &[&'static str] {
        self.owners.get(&key).map_or(&[], Vec::as_slice)
    }

    fn index_of(&self, key: u32) -> Option<usize> {
        self.prefs.iter().position(|p| p.key == key)
    }
}

/// `text` cut to `limit` bytes of ANSI text (one `char` is one byte).
fn truncate_bytes(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}
