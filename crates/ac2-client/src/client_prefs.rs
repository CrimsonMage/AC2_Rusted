//! The client's preferences: `Preferences.ini` read at startup and written on a save, the
//! subsystems' preferences registered in the bank, and the names read from the DAT's enum tables.
//!
//! The frame thread never touches the file: [`ClientPrefs::load`] runs before the frame loop, and
//! [`ClientPrefs::save`] builds the bytes on the caller and writes them on a worker thread.
//!
//! The registered sets are the sound manager's, the render's (graphics and avatar, and the
//! display), and the camera's. The client network's keys and FX's are not registered yet (see
//! `docs/retail-catalog`).

use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread::JoinHandle;

use ac2_db::ResourceStore;
use ac2_prefs::ini::text_of;
use ac2_prefs::{paths, NameTable, PreferenceBank, Preferences};
use ac2_script::data::{DatScriptData, EMAPPER_GROUP};
use ac2_script::vm::ScriptData;

/// The section names' mapper (`EnumMapper::GetString(0x4000003D, section)`).
const SECTION_MAP: u32 = 0x4000_003D;
/// The key names' mapper (`EnumMapper::GetString(0x4000003E, key)`).
const KEY_MAP: u32 = 0x4000_003E;

/// Deepest base chain an `EnumMapper` is followed up (`FileArray::GetDIDFromEnum` walks it).
const BASE_CHAIN: usize = 64;

/// The DAT's enum tables: the names of sections, keys and enum labels are game data, read on
/// demand and never stored in the port. A mapper is found the way retail finds it: the mapper's
/// value is looked up in the enum master map (`GetDIDFromEnum(value, EMAPPER_GROUP)`), then the
/// names are read from that mapper and then from its base mappers.
pub struct DatNames {
    data: DatScriptData,
}

impl DatNames {
    pub fn new(store: &Arc<ResourceStore>) -> Self {
        let data = DatScriptData::new(Arc::clone(store.dataset()), (**store).clone());
        Self { data }
    }

    /// `EnumMapper::GetString(map, value)`: the name of `value` in mapper `map`.
    fn name(&self, map: u32, value: u32) -> Option<String> {
        let mut did = self.data.enum_did(map, EMAPPER_GROUP)?;
        for _ in 0..BASE_CHAIN {
            if self.data.db_type(did) != Some(ac2_script::data::DB_TYPE_ENUM_MAPPER) {
                return None;
            }
            let mapper = self.data.enum_mapper(did)?;
            if let Some(name) = mapper.name(value) {
                return Some(text_of(name));
            }
            if mapper.base == 0 {
                return None;
            }
            did = mapper.base;
        }
        None
    }

    /// `EnumMapper::GetEnum(map, name)`: the value named `name` in mapper `map`.
    fn value(&self, map: u32, name: &str) -> Option<u32> {
        let bytes: Vec<u8> = name
            .chars()
            .map(|c| u8::try_from(u32::from(c)).unwrap_or(b'?'))
            .collect();
        let mut did = self.data.enum_did(map, EMAPPER_GROUP)?;
        for _ in 0..BASE_CHAIN {
            if self.data.db_type(did) != Some(ac2_script::data::DB_TYPE_ENUM_MAPPER) {
                return None;
            }
            let mapper = self.data.enum_mapper(did)?;
            if let Some(value) = mapper.value(&bytes) {
                return Some(value);
            }
            if mapper.base == 0 {
                return None;
            }
            did = mapper.base;
        }
        None
    }
}

impl NameTable for DatNames {
    fn section_name(&self, id: u32) -> Option<String> {
        self.name(SECTION_MAP, id)
    }

    fn key_name(&self, id: u32) -> Option<String> {
        self.name(KEY_MAP, id)
    }

    fn enum_name(&self, map: u32, label: u32) -> Option<String> {
        self.name(map, label)
    }

    fn enum_label(&self, map: u32, name: &str) -> Option<u32> {
        self.value(map, name)
    }
}

/// `Render::SaveOverallGraphicsQuality`'s section and key (literal in the client).
const GRAPHICS_SECTION: &str = "Graphics";
const GRAPHICS_QUALITY_KEY: &str = "OverallGraphicsQuality";
/// `Preferences::InqBool(0x40000004, 0x40000073)`: the section and key the quality is read from.
const GRAPHICS_SECTION_ID: u32 = 0x4000_0004;
const GRAPHICS_QUALITY_ID: u32 = 0x4000_0073;

/// The client's preferences: the bank, the session cache of the other values, the names they
/// are written with, and the file they live in.
pub struct ClientPrefs {
    bank: PreferenceBank,
    cache: Preferences,
    names: DatNames,
    path: PathBuf,
}

impl ClientPrefs {
    /// Reads `path` (an empty file when there is none) and registers the sound preferences. Runs
    /// before the frame loop.
    pub fn load(path: PathBuf, store: Arc<ResourceStore>) -> io::Result<Self> {
        let bytes = paths::read_or_empty(&path)?;
        let names = DatNames::new(&store);
        let mut bank = PreferenceBank::new(&bytes);
        ac2_audio::prefs::register(&mut bank, &names);
        ac2_render::prefs::register_graphics(&mut bank, &names);
        ac2_render::prefs::register_device(&mut bank, &names);
        ac2_smartbox::prefs::register(&mut bank, &names);
        let cache = Preferences::new(&bytes);
        Ok(Self {
            bank,
            cache,
            names,
            path,
        })
    }

    /// `Preferences::InqBool(0x40000004, 0x40000073)`: the saved graphics quality (the render
    /// applies it when it is 1 to 5).
    pub fn overall_graphics_quality(&self) -> Option<u32> {
        self.cache
            .inq_bool(&self.names, GRAPHICS_SECTION_ID, GRAPHICS_QUALITY_ID)
    }

    /// `Render::SetOverallGraphicsQuality`'s save: `Preferences::SetUInt32` for the session, then
    /// `[Graphics] OverallGraphicsQuality` written into the file at once. The file is written on
    /// a worker thread.
    pub fn save_overall_graphics_quality(&mut self, level: u32) -> JoinHandle<io::Result<()>> {
        self.cache
            .set_uint32(GRAPHICS_SECTION_ID, GRAPHICS_QUALITY_ID, level);
        let text = level.to_string();
        self.bank
            .write_file_value(GRAPHICS_SECTION, GRAPHICS_QUALITY_KEY, &text);
        self.cache
            .write_file_value(GRAPHICS_SECTION, GRAPHICS_QUALITY_KEY, &text);
        self.write_file()
    }

    /// `Render::SetOverallGraphicsQuality(level)`: the preset's values and defaults go into the
    /// bank and the graphics section is applied (the file is written by
    /// [`ClientPrefs::save_overall_graphics_quality`]). `false` for a level without a preset.
    pub fn set_overall_graphics_quality(&mut self, level: u32) -> bool {
        ac2_render::prefs::apply_quality(&mut self.bank, level)
    }

    pub fn bank(&self) -> &PreferenceBank {
        &self.bank
    }

    pub fn bank_mut(&mut self) -> &mut PreferenceBank {
        &mut self.bank
    }

    pub fn names(&self) -> &DatNames {
        &self.names
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `PreferenceBank::Save`: every preference is written into the file image, and the image is
    /// written to the file on a worker thread (the caller does not wait for the disk).
    pub fn save(&mut self) -> JoinHandle<io::Result<()>> {
        self.bank.save(&self.names);
        self.write_file()
    }

    /// The file image, written to the path on a worker thread.
    fn write_file(&self) -> JoinHandle<io::Result<()>> {
        let bytes = self.bank.file_bytes();
        let path = self.path.clone();
        std::thread::spawn(move || paths::write_atomic(&path, &bytes))
    }
}
