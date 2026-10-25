//! `gmKeyboardUI`'s binding list: the rows `InitializeKeyboardList` makes (an action's header,
//! then its two key slots, three listbox items per action), the set logic (`StartSet`,
//! `FinishSet`, `KeyHitHandler`), clear and clear all (`ListenToMessage`'s clear button and
//! `DoClearAll`), and the already-mapped question's answer (`RecvNotice_CloseConfirmationDialog`).
//!
//! The model is pure: the page (`keyboard.rs`) renders its rows and handles its messages. The
//! list shows the bindings of the maps the input manager holds, two per action. Element creation,
//! key names and the saved maps are in `keyboard.rs`, `key_names.rs` and `keyboard_maps.rs`.
//!
//! Not modelled yet: a binding's over-mapped flag (`MappedKey::overMapped`), and `StartSet`'s
//! second pass over actions without a row.

use std::collections::BTreeMap;

use crate::maps::{ActionMap, KeyMap};

/// A binding's meta key when it has none (`MappedKey::metakey` 0).
pub const META_NONE: u32 = 0;
/// The meta key of the shift key map: left shift (`ID_HitMeta` holds the key's semantic).
pub const META_SHIFT: u32 = 0x8100_042A;
/// The meta key of the control key map: left control.
pub const META_CONTROL: u32 = 0x8100_041D;
/// `gmKeyboardUI::KeyHitHandler`'s cancel key (`0x8200040C`): it leaves the set.
pub const KEY_CANCEL: u32 = 0x8200_040C;
/// The two mouse semantics `KeyHitHandler` ignores while a binding waits for a key.
pub const MOUSE_IGNORED: [u32; 2] = [0x8200_0300, 0x8200_0304];

/// Key slots per action (`InitializeKeyboardList`'s two `MappedKey`s).
pub const SLOTS: usize = 2;

/// A key a slot holds: its input semantic and meta mode (`MappedKey`'s `semantic` and `metakey`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub semantic: u32,
    pub meta: u32,
}

/// Where a key slot sits: the row, and the slot within it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotRef {
    pub row: usize,
    pub slot: usize,
}

/// An action's row: its header (the name and description string ids `GetDescripValues` reads)
/// and its key slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingRow {
    pub action: u32,
    pub name: u32,
    pub description: u32,
    pub slots: [Option<KeyBinding>; SLOTS],
}

/// What `gmKeyboardUI::StartSet` does with a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOutcome {
    /// The key is on another slot already: `m_occupied` is that slot and the already-mapped
    /// question opens (`m_already_mapped`); the answer is [`KeyboardList::answer_already_mapped`].
    AlreadyMapped { occupied: SlotRef },
    /// The key is free (or is the selected slot's own): `FinishSet` wrote it to the selection.
    Set,
}

/// `gmKeyboardUI`'s list state: the rows, the selected slot (`m_selected`), the slot a conflict
/// names (`m_occupied`) and whether anything changed (`m_modified`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyboardList {
    rows: Vec<BindingRow>,
    selected: Option<SlotRef>,
    occupied: Option<SlotRef>,
    /// `m_sem`, `m_meta`: the key a set is pending for, while the already-mapped question is up.
    pending: Option<KeyBinding>,
    modified: bool,
}

impl KeyboardList {
    /// `gmKeyboardUI::InitializeKeyboardList`: a row per action with a description, in action
    /// order (the action enum's order), its slots filled from `maps` in order. Each map is a key
    /// map with its meta mode (`META_NONE`, `META_SHIFT`, `META_CONTROL`); bindings past the
    /// second slot are not listed.
    pub fn from_maps(actions: &ActionMap, maps: &[(&KeyMap, u32)]) -> Self {
        let mut rows: BTreeMap<u32, BindingRow> = actions
            .descriptions
            .iter()
            .map(|(&action, &(name, description))| {
                (
                    action,
                    BindingRow {
                        action,
                        name,
                        description,
                        slots: [None; SLOTS],
                    },
                )
            })
            .collect();
        for (map, meta) in maps {
            for mapping in &map.mappings {
                let Some(row) = rows.get_mut(&mapping.action) else {
                    continue;
                };
                let binding = KeyBinding {
                    semantic: mapping.semantic,
                    meta: *meta,
                };
                if let Some(slot) = row.slots.iter_mut().find(|s| s.is_none()) {
                    *slot = Some(binding);
                }
            }
        }
        Self {
            rows: rows.into_values().collect(),
            ..Self::default()
        }
    }

    /// The rows, in list order.
    pub fn rows(&self) -> &[BindingRow] {
        &self.rows
    }

    /// `m_selected`: the slot a key set would write.
    pub fn selected(&self) -> Option<SlotRef> {
        self.selected
    }

    /// `m_modified`: a binding changed since the list was made.
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// Sets `m_modified` (a restore changes the bindings without a list edit).
    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    /// The slot a conflict named and the key it is asked about, while the question is up.
    pub fn question(&self) -> Option<(SlotRef, KeyBinding)> {
        Some((self.occupied?, self.pending?))
    }

    /// The slot `index` names in the listbox (three items per row: the header, then the two
    /// slots). A header click selects its first slot (`ListenToMessage`).
    pub fn click(&mut self, index: usize) -> Option<SlotRef> {
        let row = index / (SLOTS + 1);
        let item = index % (SLOTS + 1);
        if row >= self.rows.len() {
            return None;
        }
        if item == 0 {
            return self.select(SlotRef { row, slot: 0 });
        }
        self.select(SlotRef {
            row,
            slot: item - 1,
        })
    }

    /// Selects a slot for a key set (`m_selected`, `m_waitforClick`).
    pub fn select(&mut self, slot: SlotRef) -> Option<SlotRef> {
        if slot.row < self.rows.len() && slot.slot < SLOTS {
            self.selected = Some(slot);
        }
        self.selected
    }

    /// `gmKeyboardUI::StartSet(semantic, meta)`: a key on another slot is already mapped (the
    /// conflict is named, the key is kept pending for the question); otherwise `FinishSet` writes
    /// it to the selection. Without a selection there is nothing to set.
    pub fn start_set(&mut self, binding: KeyBinding) -> Option<SetOutcome> {
        self.selected?;
        for (row_index, row) in self.rows.iter().enumerate() {
            for (slot_index, slot) in row.slots.iter().enumerate() {
                let here = SlotRef {
                    row: row_index,
                    slot: slot_index,
                };
                if Some(here) != self.selected && *slot == Some(binding) {
                    self.occupied = Some(here);
                    self.pending = Some(binding);
                    return Some(SetOutcome::AlreadyMapped { occupied: here });
                }
            }
        }
        self.finish_set(binding);
        Some(SetOutcome::Set)
    }

    /// The already-mapped question's answer (`RecvNotice_CloseConfirmationDialog`): yes finishes
    /// the set, which clears the occupied slot; no drops it.
    pub fn answer_already_mapped(&mut self, yes: bool) {
        let Some(binding) = self.pending.take() else {
            return;
        };
        if yes {
            self.finish_set(binding);
        } else {
            self.occupied = None;
        }
    }

    /// `gmKeyboardUI::FinishSet`: the occupied slot (if a conflict named one) is cleared, the
    /// selection takes the key, and the list's selection clears.
    pub fn finish_set(&mut self, binding: KeyBinding) {
        let Some(slot) = self.selected else {
            return;
        };
        self.modified = true;
        if let Some(occupied) = self.occupied.take() {
            self.write(occupied, None);
        }
        self.pending = None;
        self.write(slot, Some(binding));
        self.selected = None;
    }

    /// `KeyHitHandler`: while a slot is selected, a key starts a set. The cancel key leaves the
    /// set; an open dialog, a mouse semantic in [`MOUSE_IGNORED`] or a `reserved` key (the
    /// input manager's check) is ignored. `None` when the key does nothing.
    pub fn key_hit(
        &mut self,
        semantic: u32,
        meta: u32,
        dialog_open: bool,
        reserved: bool,
    ) -> Option<SetOutcome> {
        self.selected?;
        if semantic == KEY_CANCEL {
            self.selected = None;
            return None;
        }
        if dialog_open || reserved || MOUSE_IGNORED.contains(&semantic) {
            return None;
        }
        self.start_set(KeyBinding { semantic, meta })
    }

    /// The clear button (`ListenToMessage`): the selected slot loses its key.
    pub fn clear_selected(&mut self) {
        let Some(slot) = self.selected else {
            return;
        };
        self.write(slot, None);
        self.modified = true;
        self.selected = None;
    }

    /// `gmKeyboardUI::DoClearAll`: every slot loses its key (the modified flag is set).
    pub fn clear_all(&mut self) {
        self.modified = true;
        for row in &mut self.rows {
            row.slots = [None; SLOTS];
        }
        self.selected = None;
        self.occupied = None;
        self.pending = None;
    }

    /// The bindings `SaveKeyboardList` writes, in list order: the action and the key of each
    /// filled slot.
    pub fn bindings(&self) -> Vec<(u32, KeyBinding)> {
        self.rows
            .iter()
            .flat_map(|row| {
                row.slots
                    .iter()
                    .filter_map(move |slot| slot.map(|binding| (row.action, binding)))
            })
            .collect()
    }

    fn write(&mut self, slot: SlotRef, binding: Option<KeyBinding>) {
        if let Some(row) = self.rows.get_mut(slot.row) {
            if let Some(cell) = row.slots.get_mut(slot.slot) {
                *cell = binding;
            }
        }
    }
}
