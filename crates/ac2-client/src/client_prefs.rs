//! The client's preferences: the DAT's name table, and the registrations of the subsystems
//! (sound, render and smartbox) over the file the [`ac2_prefs::PrefsFile`] reads and writes.
//!
//! The file IO is `ac2_prefs::PrefsFile` (read at startup, written on a worker thread on a save);
//! the graphics quality's read and save are `ac2_render::prefs`. The client network's keys are
//! not registered yet (see `docs/retail-catalog`); the FX channel preferences are
//! (`ac2_smartbox::fx_prefs`).

use std::io;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;

use ac2_db::ResourceStore;
use ac2_prefs::ini::text_of;
use ac2_prefs::{NameTable, PrefsFile};
use ac2_render::prefs::SceneOptions;
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
        // The client-only keys (the HUD's style) have no name in the DAT's mapper.
        self.name(KEY_MAP, id)
            .or_else(|| ac2_ui::hud_style::client_key_name(id).map(str::to_owned))
    }

    fn enum_name(&self, map: u32, label: u32) -> Option<String> {
        self.name(map, label)
    }

    fn enum_label(&self, map: u32, name: &str) -> Option<u32> {
        self.value(map, name)
    }
}

/// `StringStateInfo::GetTopGoreLevel` and `GetTopBloodLevel` of the language's `StringState`
/// (the first of the local DAT), `None` without one.
fn top_levels(store: &ResourceStore) -> Option<(u32, u32)> {
    let state = ac2_db::load_string_state(store, ac2_core::DataId::new(0x5800_0000))
        .ok()
        .flatten()?;
    Some((state.settings[0], state.settings[1]))
}

/// The client's preferences: the file with the subsystems' preferences registered, over the DAT's
/// names. The file's methods are reached through `Deref`.
pub struct ClientPrefs {
    file: PrefsFile<DatNames>,
}

impl ClientPrefs {
    /// Reads `path` (an empty file when there is none) and registers the sound, render (graphics
    /// and display), camera and tooltip preferences. Runs before the frame loop.
    pub fn load(path: PathBuf, store: Arc<ResourceStore>) -> io::Result<Self> {
        let mut file = PrefsFile::load(path, DatNames::new(&store))?;
        let (bank, names) = file.parts_mut();
        ac2_audio::prefs::register(bank, names);
        ac2_render::prefs::register_graphics(bank, names);
        ac2_render::prefs::register_device(bank, names);
        ac2_smartbox::prefs::register(bank, names);
        ac2_smartbox::fx_prefs::register(bank, names, top_levels(&store));
        ac2_ui::tooltip_prefs::register(bank, names);
        ac2_ui::hud_style::register(bank, names);
        Ok(Self { file })
    }

    /// `Preferences::InqBool(0x40000004, 0x40000073)`: the saved graphics quality.
    pub fn overall_graphics_quality(&self) -> Option<u32> {
        ac2_render::prefs::overall_graphics_quality(&self.file)
    }

    /// `Render::SaveOverallGraphicsQuality`'s save, on a worker thread.
    pub fn save_overall_graphics_quality(&mut self, level: u32) -> JoinHandle<io::Result<()>> {
        ac2_render::prefs::save_overall_graphics_quality(&mut self.file, level)
    }

    /// `Render::InitRenderingPreferences`' quality step: a saved level from 1 to 5 is applied
    /// (`SetOverallGraphicsQuality`). Without one, retail kept the registered defaults; the client
    /// applies `first_run_level` instead (`ac2_gpu::suggested_quality_level`, from the GPU), unsaved,
    /// so the player's own choice in the options still decides. Returns the level applied.
    pub fn apply_startup_quality(&mut self, first_run_level: u32) -> u32 {
        let level = self
            .overall_graphics_quality()
            .filter(|level| (1..=5).contains(level))
            .unwrap_or(first_run_level);
        self.set_overall_graphics_quality(level);
        level
    }

    /// `Render::SetOverallGraphicsQuality(level)`: the preset's values go into the bank. `false`
    /// for a level without a preset.
    pub fn set_overall_graphics_quality(&mut self, level: u32) -> bool {
        ac2_render::prefs::apply_quality(self.file.bank_mut(), level)
    }

    /// The scene's graphics options as the bank holds them now (`ac2_render::prefs::scene_options`).
    pub fn scene_options(&self) -> SceneOptions {
        ac2_render::prefs::scene_options(self.file.bank())
    }

    /// The file the preferences are read from and written to.
    pub fn file(&self) -> &PrefsFile<DatNames> {
        &self.file
    }

    /// The file, for the options page's actions (`ac2_ui::OptionsPrefs`).
    pub fn file_mut(&mut self) -> &mut PrefsFile<DatNames> {
        &mut self.file
    }
}

impl Deref for ClientPrefs {
    type Target = PrefsFile<DatNames>;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl DerefMut for ClientPrefs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}

impl crate::Client {
    /// Gives the client its preferences, read before the frame loop; the options pages write them.
    pub fn attach_prefs(&mut self, prefs: ClientPrefs) {
        self.prefs = Some(prefs);
    }

    pub fn prefs(&self) -> Option<&ClientPrefs> {
        self.prefs.as_ref()
    }

    pub fn prefs_mut(&mut self) -> Option<&mut ClientPrefs> {
        self.prefs.as_mut()
    }

    /// The applied preferences as the live client reads them: the sound manager's volumes
    /// (`SoundManager::HandleApply`) and the camera's values (`CameraManager::HandleApply`). Returns
    /// the scene's graphics options for the view (`WorldScene::set_options`).
    pub fn apply_live_preferences(&mut self) -> Option<ac2_render::prefs::SceneOptions> {
        let prefs = self.prefs.take()?;
        self.apply_sound_preferences(prefs.bank());
        self.apply_light_preferences(prefs.bank());
        self.apply_streak_preferences(prefs.bank());
        self.apply_decal_preferences(prefs.bank());
        ac2_smartbox::fx_prefs::apply(prefs.bank());
        self.camera
            .apply_prefs(&ac2_smartbox::prefs::values(prefs.bank()));
        let scene = prefs.scene_options();
        self.prefs = Some(prefs);
        Some(scene)
    }
}
