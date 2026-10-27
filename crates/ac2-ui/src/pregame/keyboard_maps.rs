//! The key maps the controls screen edits, as the input manager holds them: the main map and the
//! shift and control maps (pushed under left shift and left control). Restore switches the three
//! to a preset's maps (`gmKeyboardUI::RecvNotice_CloseConfirmationDialog`'s restore branches);
//! save builds three maps from the list (`SaveKeyboardList`) and the input manager takes them.
//!
//! Not modelled yet: the keymap files `SaveKeyboardList` writes (their paths come from the
//! preference bank, entries `0x40000030` to `0x40000032`, and are not known to be a user config
//! location), and the action names each saved mapping carries.

use std::collections::HashMap;
use std::sync::Arc;

use crate::maps::{ActionMap, ActionMapping, KeyMap};
use crate::pregame::keyboard_list::{KeyboardList, META_NONE};

/// Left shift and left control: the meta keys the shift and control maps push under.
pub const LEFT_SHIFT: u32 = 0x8100_042A;
pub const RIGHT_SHIFT: u32 = 0x8100_0436;
pub const LEFT_CONTROL: u32 = 0x8100_041D;
pub const RIGHT_CONTROL: u32 = 0x8100_049D;

/// The three maps the input manager pushes for editing: main, control and shift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyMapSet {
    pub main: KeyMap,
    pub control: KeyMap,
    pub shift: KeyMap,
}

/// The restore presets (`restore_button`, `restore_eq_button`, `restore_daoc_button`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Default,
    Eq,
    Daoc,
}

impl Preset {
    /// The main, control and shift maps' enums.
    pub fn maps(self) -> [u32; 3] {
        match self {
            Self::Default => [0x4100_0001, 0x4100_0002, 0x4100_0003],
            Self::Eq => [0x4100_0004, 0x4100_0005, 0x4100_0006],
            Self::Daoc => [0x4100_0007, 0x4100_0008, 0x4100_0009],
        }
    }
}

/// The key maps and the action map the DATs hold, loaded for the controls screen.
#[derive(Debug, Clone, Default)]
pub struct KeyMapLibrary {
    pub action_map: ActionMap,
    /// The key maps by enum.
    pub maps: HashMap<u32, KeyMap>,
}

impl KeyMapLibrary {
    /// Each action's name as the DATs' maps give it (`EnumMapper::GetString(0x4000001d, action)`
    /// is the saved name in retail; the maps' bindings carry it here). An action no map names
    /// has none.
    pub fn action_names(&self) -> HashMap<u32, String> {
        let mut names = HashMap::new();
        for map in self.maps.values() {
            for mapping in &map.mappings {
                if !mapping.name.is_empty() {
                    names
                        .entry(mapping.action)
                        .or_insert_with(|| mapping.name.clone());
                }
            }
        }
        names
    }

    /// The maps of a preset, when the DATs hold all three.
    pub fn preset(&self, preset: Preset) -> Option<KeyMapSet> {
        let [main, control, shift] = preset.maps();
        Some(KeyMapSet {
            main: self.maps.get(&main)?.clone(),
            control: self.maps.get(&control)?.clone(),
            shift: self.maps.get(&shift)?.clone(),
        })
    }
}

/// Which saved map a binding goes to: none to the main map, a shift key to the shift map, a
/// control key to the control map (`SaveKeyboardList`). Other meta keys are not saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapKind {
    Main,
    Shift,
    Control,
}

/// The map a binding's meta key files it under.
pub fn map_kind(meta: u32) -> Option<MapKind> {
    match meta {
        META_NONE => Some(MapKind::Main),
        LEFT_SHIFT | RIGHT_SHIFT => Some(MapKind::Shift),
        LEFT_CONTROL | RIGHT_CONTROL => Some(MapKind::Control),
        _ => None,
    }
}

/// What the controls screen shares with the client: the library, and the maps the input manager
/// holds now (the defaults until a save or a restore changes them).
#[derive(Debug, Clone, Default)]
pub(crate) struct ActiveKeyMaps {
    pub library: Option<Arc<KeyMapLibrary>>,
    pub active: Option<KeyMapSet>,
}

impl ActiveKeyMaps {
    /// The defaults, as the input manager starts with them.
    pub(crate) fn new(library: Option<Arc<KeyMapLibrary>>) -> Self {
        let active = library.as_ref().and_then(|l| l.preset(Preset::Default));
        Self { library, active }
    }

    /// The list the active maps make (`InitializeKeyboardList`'s bindings), when both exist.
    pub(crate) fn list(&self) -> KeyboardList {
        match (&self.library, &self.active) {
            (Some(library), Some(active)) => list_of(&library.action_map, active),
            _ => KeyboardList::default(),
        }
    }
}

/// The list the maps make: the main map's bindings, then the shift's and the control's, under
/// their meta keys.
pub(crate) fn list_of(actions: &ActionMap, set: &KeyMapSet) -> KeyboardList {
    KeyboardList::from_maps(
        actions,
        &[
            (&set.main, META_NONE),
            (&set.shift, LEFT_SHIFT),
            (&set.control, LEFT_CONTROL),
        ],
    )
}

/// The meta key a meta mode stands for: the input manager's control bit (4) is left control, its
/// shift bit (1) is left shift.
pub fn meta_key_of(meta_mode: u32) -> u32 {
    if meta_mode & 4 != 0 {
        LEFT_CONTROL
    } else if meta_mode & 1 != 0 {
        LEFT_SHIFT
    } else {
        META_NONE
    }
}

/// `SaveKeyboardList`: the three maps the list's bindings make, named `mydefault`, `mydefaultc`
/// and `mydefaults`. A binding is named by its action (`names`, the action's name); an action no
/// map names is written by its enum value (`0x........`), which the reader takes back.
pub fn saved_set(list: &KeyboardList, names: &HashMap<u32, String>) -> KeyMapSet {
    let mut set = KeyMapSet {
        main: named("mydefault"),
        control: named("mydefaultc"),
        shift: named("mydefaults"),
    };
    for (action, binding) in list.bindings() {
        let name = names
            .get(&action)
            .cloned()
            .unwrap_or_else(|| format!("{action:#010x}"));
        let mapping = ActionMapping {
            action,
            semantic: binding.semantic,
            name,
        };
        match map_kind(binding.meta) {
            Some(MapKind::Main) => set.main.mappings.push(mapping),
            Some(MapKind::Shift) => set.shift.mappings.push(mapping),
            Some(MapKind::Control) => set.control.mappings.push(mapping),
            None => {}
        }
    }
    set
}

fn named(name: &str) -> KeyMap {
    KeyMap {
        name: name.to_owned(),
        guid: [0; 16],
        mappings: Vec::new(),
    }
}
