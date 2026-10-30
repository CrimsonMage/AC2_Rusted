//! `gmPetUI`: the pet panel, the `PetPanel` root of the gameplay layout (`0x41000009`). Its
//! `PetPanelListBox` lists one `PetTemplate` row per pet: the pet's name (`Name`, state `safe` or
//! `unsafe`), its mode (`Mode`, one state per mode name, `Unique` for a mode not in the table) and
//! the `NextMode` / `PreviousMode` arrows. The four buttons under the panel (`AttackButton`,
//! `ReturnButton`, `HealButton`, `BuffButton`) act on the selection or the pet.
//!
//! The pet notices reach the panel as `CM_Character::SendNotice_AddPet`, `RemovePet`,
//! `UpdatePetMode`, `UpdatePetName`, `PetIsUnderAttack` and the selection (`SelectionChanged`).
//! Legions 7160 has every handler in the notice vtable of `gmPetUI` (0x9ca2e8: AddPet 0x419810,
//! PetIsUnderAttack 0x419570, ReloadGameplayOptions 0x418c00, RemovePet 0x418f70,
//! SelectionChanged 0x419010, UpdatePetMode 0x4199b0, UpdatePetName 0x41a1e0); the Legions
//! decompile names only `UpdatePetMode`, the rest were read from the disassembly beside admin
//! 7148's named bodies and match them. The mode table and the request timer follow the admin
//! 7148 build (`ModeTable` from its static initializer, `gmPetUI::Refresh`'s five-second cycle).
//!
//! A right press (`0x40000005`) on a row makes its `Name` editable and active with the text
//! selected; when it deactivates, a name that differs (case-insensitively) from the stored one is
//! requested (`RequestSetPetName`) and the row shows the stored name again until the server's
//! `UpdatePetName` arrives. The server's name has its first `(...)` removed
//! (`AddPet` and `UpdatePetName`).
//!
//! Not modelled yet: the list's height for its rows, and the row's `petid` attribute on the name
//! (nothing reads it back here).

use std::sync::Arc;

use ac2_net::protocol::StringInfo;
use ac2_weenie::GameplayNotice;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};

/// `ModeTable` (admin 7148, `_E554`): the pet modes in cycling order, each with its state name.
/// The first mode's id is not written by the initializer; 0x20 is its value in the image.
pub const PET_MODES: [(u32, &str); 9] = [
    (0x20, "Aggressive"),
    (0x1000, "Opportunistic"),
    (0x2000, "Courageous"),
    (0x4000_0001, "Protective"),
    (0x4000_0003, "Assisting"),
    (0x4000_0004, "Defensive"),
    (0x4000_0002, "Supporting"),
    (0x4000_0006, "Cooperative"),
    (0x4000_0005, "Idle"),
];

/// The state a mode without a table entry shows (`Tokenize("Unique")`).
const UNIQUE: &str = "Unique";
/// `gmPetUI::Refresh`'s wait between two passes, and the time a pet stays unsafe.
const REFRESH_INTERVAL: f64 = 5.0;

/// `RIGHT_MOUSE_BUTTON` (`0x40000005`): the press that starts a name edit.
const RIGHT_BUTTON: i32 = 0x4000_0005;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `GetNextMode`: the mode after `mode` in the table, wrapping; the first mode when `mode` is not
/// in the table.
pub fn next_mode(mode: u32) -> u32 {
    match PET_MODES.iter().position(|(m, _)| *m == mode) {
        Some(k) => PET_MODES[(k + 1) % PET_MODES.len()].0,
        None => PET_MODES[0].0,
    }
}

/// `GetPreviousMode`: the mode before `mode` in the table, wrapping; the last mode when `mode` is
/// not in the table.
pub fn previous_mode(mode: u32) -> u32 {
    match PET_MODES.iter().position(|(m, _)| *m == mode) {
        Some(k) => PET_MODES[(k + PET_MODES.len() - 1) % PET_MODES.len()].0,
        None => PET_MODES[PET_MODES.len() - 1].0,
    }
}

/// The state name a mode's text shows (`RequestSetPetMode`).
fn mode_state(mode: u32) -> &'static str {
    PET_MODES
        .iter()
        .find(|(m, _)| *m == mode)
        .map_or(UNIQUE, |(_, name)| name)
}

/// What the panel asks of the player's script (`gmCWInterface` calls made from `gmPetUI`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PetAction {
    /// `gmCWInterface::SetSelection(pet)`: the row was pressed.
    Select(u64),
    /// `gmCWInterface::RequestPetAttack(selection)`.
    Attack(u64),
    /// `RequestPetBuff`, `RequestPetHeal`, `RequestPetReturn`.
    Buff,
    Heal,
    Return,
    /// `gmCWInterface::RequestPetMode(pet, mode)`: `Refresh` sends a row's wanted mode.
    Mode {
        pet: u64,
        mode: u32,
    },
    /// `gmCWInterface::RequestSetPetName(pet, name)`: a name edit that changed the name.
    Rename {
        pet: u64,
        name: String,
    },
}

/// One pet's row (`PetInfo` in the `_pets` table).
#[derive(Debug, Clone)]
struct PetRow {
    id: u64,
    root: usize,
    /// The mode the server last reported (`PetInfo::mode`).
    mode: u32,
    /// The mode the player wants (`PetInfo::nextMode`), sent by `Refresh`.
    next_mode: u32,
    /// When the pet was last attacked (`PetInfo::lastAttacked`), 0 for never.
    last_attacked: f64,
    /// `PetInfo::name`: the name as the panel shows it (the server's, `(...)` removed).
    name: String,
}

/// `gmPetUI`'s state.
#[derive(Debug, Clone)]
pub struct PetUi {
    layout: Arc<LayoutDesc>,
    /// `_petPanelField`: the `PetPanel` root.
    panel: Option<usize>,
    /// `_petUIList`: the `PetPanelListBox`.
    list: Option<usize>,
    /// `_pets`, in the order added.
    pets: Vec<PetRow>,
    /// The framework's shown state (`UIFramework::Show`).
    shown: bool,
    /// `GetShowPetUI` on the player.
    show_pet_ui: bool,
    /// `_lastUpdate`: when `Refresh` last ran.
    last_update: f64,
    /// `_iidCurrentSelection` (0 for `INVALID_IID`): the selected pet's row.
    selection: u64,
    /// Whether the panel listens for the edited name's `deactivated` (`RegisterForMessage` in
    /// `ListenToMessage`'s right press).
    editing: bool,
}

impl PetUi {
    /// `gmPetUI::gmPetUI`: the `PetPanel` root (`CreateAndAddRootElement(0x41000009, PetPanel)`),
    /// then `UpdateState`. It starts hidden.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Self {
        let panel = tree.create_root(Arc::clone(gameplay), id("PetPanel"));
        let list = panel.and_then(|p| tree.child_recursive(p, id("PetPanelListBox")));
        let ui = Self {
            layout: Arc::clone(gameplay),
            panel,
            list,
            pets: Vec::new(),
            shown: false,
            show_pet_ui: true,
            last_update: 0.0,
            selection: 0,
            editing: false,
        };
        ui.update_state(tree);
        ui
    }

    /// Whether the panel is shown (`UIFramework::IsShown`).
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// `gmPetUI::Show`: the framework's shown state; the root is drawn when it is shown and a pet
    /// is listed with the UI on (`update_state`).
    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.shown = shown;
        self.update_state(tree);
    }

    /// The player's `GetShowPetUI` changed: `UpdateState` shows the pets or none.
    pub fn set_show_pet_ui(&mut self, tree: &mut UiTree, show: bool) {
        self.show_pet_ui = show;
        self.update_state(tree);
    }

    /// The pet's name as the panel keeps it (`PetInfo::name`).
    pub fn name_of(&self, pet: u64) -> Option<&str> {
        self.pets
            .iter()
            .find(|p| p.id == pet)
            .map(|p| p.name.as_str())
    }

    /// The pets' ids, in the order added.
    pub fn pets(&self) -> Vec<u64> {
        self.pets.iter().map(|p| p.id).collect()
    }

    /// `gmPetUI::UpdateState`: the root shows `state_pets` with pets listed and the UI on, else
    /// `state_nopets`; the four buttons are ghosted unless a pet is listed and the UI is on. The
    /// root is drawn only while the panel is shown and a pet is listed (the `nopets` state's art
    /// is not checked, so it is not drawn).
    fn update_state(&self, tree: &mut UiTree) {
        let Some(panel) = self.panel else {
            return;
        };
        let active = !self.pets.is_empty() && self.show_pet_ui;
        tree.set_state(panel, id(if active { "pets" } else { "nopets" }));
        tree.set_visible(panel, self.shown && active);
        for button in ["AttackButton", "ReturnButton", "HealButton", "BuffButton"] {
            if let Some(index) = tree.child_recursive(panel, id(button)) {
                tree.set_bool(index, BUTTON_GHOSTED, !active);
            }
        }
    }

    /// `gmPetUI::RecvNotice_AddPet`: a `PetTemplate` row for the pet, its mode shown and its
    /// wanted mode the same as the server's, then `RecvNotice_UpdatePetName` for the name, the
    /// `Icon` image (`AddImageToState(icon, 0, stateless)`) when there is one, and `UpdateState`.
    /// A pet already listed is not listed twice.
    pub fn add_pet(&mut self, tree: &mut UiTree, pet: u64, name: &str, icon: u32, mode: u32) {
        let (Some(list), Some(desc)) = (self.list, self.layout.find_element(id("PetTemplate")))
        else {
            return;
        };
        if self.pets.iter().any(|p| p.id == pet) {
            return;
        }
        let Some(root) = tree.create_element(Arc::clone(&self.layout), desc, false) else {
            return;
        };
        tree.list_add_item(list, root);
        self.pets.push(PetRow {
            id: pet,
            root,
            mode,
            next_mode: mode,
            last_attacked: 0.0,
            name: String::new(),
        });
        if let Some(mode_text) = tree.child_recursive(root, id("Mode")) {
            tree.set_state(mode_text, id(mode_state(mode)));
        }
        if let Some(name_text) = tree.child_recursive(root, id("Name")) {
            tree.set_state(name_text, id("safe"));
        }
        self.update_pet_name(tree, pet, name);
        if icon != 0 {
            if let Some(image) = tree.child_recursive(root, id("Icon")) {
                tree.add_image_to_state(image, icon, 0, crate::token::STATELESS);
            }
        }
        self.update_state(tree);
    }

    /// `gmPetUI::RecvNotice_RemovePet`: the pet leaves the table and its row leaves the list;
    /// `UpdateState` runs whether or not the pet was listed.
    pub fn remove_pet(&mut self, tree: &mut UiTree, pet: u64) {
        if let Some(at) = self.pets.iter().position(|p| p.id == pet) {
            let row = self.pets.remove(at);
            if let Some(list) = self.list {
                tree.list_delete_item(list, row.root);
            }
        }
        self.update_state(tree);
    }

    /// `gmPetUI::RecvNotice_UpdatePetName`: the server's name with its first `(...)` removed
    /// (only when the `(` is not the first character and a `)` follows it) is the pet's name and
    /// the row's `Name` text. A pet not listed is ignored.
    pub fn update_pet_name(&mut self, tree: &mut UiTree, pet: u64, name: &str) {
        let Some(row) = self.pets.iter_mut().find(|p| p.id == pet) else {
            return;
        };
        row.name = strip_parenthetical(name);
        let (root, shown) = (row.root, row.name.clone());
        if let Some(text) = tree.child_recursive(root, id("Name")) {
            tree.set_text(text, &shown);
        }
    }

    /// `gmPetUI::RecvNotice_SelectionChanged`: the old selection's row goes `unselected` (and its
    /// name stops being editable when the selection moved elsewhere), then the new selection's
    /// row, when it is a listed pet, goes `selected`.
    pub fn selection_changed(&mut self, tree: &mut UiTree, new: u64) {
        if self.selection != 0 {
            if let Some(root) = self
                .pets
                .iter()
                .find(|p| p.id == self.selection)
                .map(|p| p.root)
            {
                tree.set_state(root, id("unselected"));
                if new != self.selection {
                    // The row has no `Name` element: retail returns without moving on.
                    let Some(name) = tree.child_recursive(root, id("Name")) else {
                        return;
                    };
                    if tree.is_text_editable(name) {
                        tree.set_bool(name, id("editable"), false);
                        self.editing = false;
                    }
                }
            }
            self.selection = 0;
        }
        if let Some(root) = self.pets.iter().find(|p| p.id == new).map(|p| p.root) {
            tree.set_state(root, id("selected"));
            self.selection = new;
        }
    }

    /// `gmPetUI::RecvNotice_PetIsUnderAttack`: the pet's name turns `unsafe`, and it stays so
    /// until `Refresh` finds it unattacked for the interval. A pet not listed is ignored.
    pub fn pet_under_attack(&mut self, tree: &mut UiTree, pet: u64, now: f64) {
        let Some(row) = self.pets.iter_mut().find(|p| p.id == pet) else {
            return;
        };
        row.last_attacked = now;
        if let Some(name) = tree.child_recursive(row.root, id("Name")) {
            tree.set_state(name, id("unsafe"));
        }
    }

    /// `gmPetUI::RecvNotice_UpdatePetMode`: the pet's mode is the server's. A row whose wanted
    /// mode was its old mode follows the server (`RequestSetPetMode` with the new mode: the
    /// wanted mode and its text move to it); any other row keeps its wanted mode, and `tick`
    /// sends the difference. A pet not listed is ignored.
    pub fn update_pet_mode(&mut self, tree: &mut UiTree, pet: u64, mode: u32) {
        let Some(row) = self.pets.iter_mut().find(|p| p.id == pet) else {
            return;
        };
        let follows = row.mode == row.next_mode;
        row.mode = mode;
        if follows {
            row.next_mode = mode;
            if let Some(text) = tree.child_recursive(row.root, id("Mode")) {
                tree.set_state(text, id(mode_state(mode)));
            }
        }
    }

    /// The panel's messages (`gmPetUI::ListenToMessage`): a button release asks the script, an
    /// arrow steps the row's wanted mode (sent later by `Refresh`), a press on a row selects it.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        message: &UiMessage,
        selection: Option<u64>,
    ) -> Vec<PetAction> {
        let mut actions = Vec::new();
        if message.message == MSG_BUTTON_RELEASE_ON {
            let target = selection.unwrap_or(0);
            match message.element_id {
                x if x == id("AttackButton") => actions.push(PetAction::Attack(target)),
                x if x == id("ReturnButton") => actions.push(PetAction::Return),
                x if x == id("HealButton") => actions.push(PetAction::Heal),
                x if x == id("BuffButton") => actions.push(PetAction::Buff),
                x if x == id("NextMode") || x == id("PreviousMode") => {
                    if let Some(element) = message.element {
                        self.step_mode(tree, element, message.element_id == id("NextMode"));
                    }
                }
                _ => {}
            }
        } else if message.message == MSG_MOUSE_PRESS {
            if let Some(row) = message.element.and_then(|e| self.row_of(tree, e)) {
                actions.push(PetAction::Select(row.id));
                if message.data == RIGHT_BUTTON {
                    self.begin_name_edit(tree, row.root);
                }
            }
        } else if self.editing
            && message.element_id == id("Name")
            && message.message == id("deactivated")
        {
            if let Some(element) = message.element {
                actions.extend(self.end_name_edit(tree, element));
            }
        }
        actions
    }

    /// `ListenToMessage`'s right press on a row (after `SetSelection`): the name is editable, its
    /// text selected, and it becomes the active element; the panel then listens for its
    /// `deactivated`.
    fn begin_name_edit(&mut self, tree: &mut UiTree, root: usize) {
        let Some(name) = tree.child_recursive(root, id("Name")) else {
            return;
        };
        tree.set_bool(name, id("editable"), true);
        tree.select_all(name);
        tree.activate(name);
        self.editing = true;
    }

    /// `ListenToMessage` on the name's `deactivated`: no longer editable, no longer listened to;
    /// a text that differs from the selected pet's stored name (ignoring case) is requested, and
    /// the row shows the stored name again.
    fn end_name_edit(&mut self, tree: &mut UiTree, name: usize) -> Option<PetAction> {
        tree.set_bool(name, id("editable"), false);
        self.editing = false;
        let text = tree.get(name).and_then(|e| e.text()).unwrap_or_default();
        let stored = self
            .pets
            .iter()
            .find(|p| p.id == self.selection)?
            .name
            .clone();
        let renamed = name_differs(&stored, &text).then_some(PetAction::Rename {
            pet: self.selection,
            name: text,
        });
        tree.clear_text(name);
        tree.set_text(name, &stored);
        renamed
    }

    /// The row whose element `element` is (or is inside).
    fn row_of(&self, tree: &UiTree, element: usize) -> Option<PetRow> {
        let mut at = Some(element);
        while let Some(index) = at {
            if let Some(row) = self.pets.iter().find(|p| p.root == index) {
                return Some(row.clone());
            }
            at = tree.get(index).and_then(|e| e.parent_index());
        }
        None
    }

    /// `gmPetUI::RequestSetPetMode` on the arrow's row: the wanted mode steps, and its text
    /// shows the new mode. Nothing is sent until `Refresh`.
    fn step_mode(&mut self, tree: &mut UiTree, arrow: usize, forward: bool) {
        let Some(row) = self.row_of(tree, arrow).map(|r| r.id) else {
            return;
        };
        let Some(row) = self.pets.iter_mut().find(|p| p.id == row) else {
            return;
        };
        row.next_mode = if forward {
            next_mode(row.next_mode)
        } else {
            previous_mode(row.next_mode)
        };
        let root = row.root;
        if let Some(mode) = tree.child_recursive(root, id("Mode")) {
            tree.set_state(mode, id(mode_state(row.next_mode)));
        }
    }

    /// `gmPetUI::ListenToMessage` on the refresh message: once five seconds have passed, each
    /// row whose wanted mode differs from its mode asks for it, and a pet unattacked for five
    /// seconds turns `safe` again.
    pub fn tick(&mut self, tree: &mut UiTree, now: f64) -> Vec<PetAction> {
        if now - self.last_update <= REFRESH_INTERVAL {
            return Vec::new();
        }
        let mut actions = Vec::new();
        for row in &self.pets {
            if row.mode != row.next_mode {
                actions.push(PetAction::Mode {
                    pet: row.id,
                    mode: row.next_mode,
                });
            }
            if row.last_attacked > 0.0 && now - row.last_attacked > REFRESH_INTERVAL {
                if let Some(name) = tree.child_recursive(row.root, id("Name")) {
                    tree.set_state(name, id("safe"));
                }
            }
        }
        self.last_update = now;
        actions
    }
}

/// `_wcsicmp(stored, edited) != 0`.
pub(crate) fn name_differs(stored: &str, edited: &str) -> bool {
    stored.to_lowercase() != edited.to_lowercase()
}

/// `RecvNotice_UpdatePetName`'s name: the first `(` (not at index 0) and the first `)` after it
/// bound a `(...)` that is removed wherever it occurs in the name (`PStringBase::replace`).
fn strip_parenthetical(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let open = chars.iter().position(|&c| c == '(');
    let close = chars.iter().position(|&c| c == ')');
    match (open, close) {
        (Some(open), Some(close)) if open > 0 && close > open => {
            let part: String = chars[open..=close].iter().collect();
            name.replace(&part, "")
        }
        _ => name.to_owned(),
    }
}

/// What the pet panel reads from the player: the server's strings, rendered (a pet's name).
pub trait PetSource {
    /// A string the server sent, rendered.
    fn render_wire(&self, info: &StringInfo) -> String;
}

/// `gmPetUI`'s notices: `AddPet` lists the pet, `RemovePet` unlists it, `PetIsUnderAttack` marks
/// it unsafe, `UpdatePetMode` takes the server's mode, `UpdatePetName` its name and
/// `SelectionChanged` marks the selected pet's row.
pub fn pet_notice<S: PetSource>(
    panel: &mut PetUi,
    tree: &mut UiTree,
    source: &S,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::AddPet {
            pet,
            name,
            icon,
            mode,
        } => panel.add_pet(tree, *pet, &source.render_wire(name), *icon, *mode),
        GameplayNotice::RemovePet(pet) => panel.remove_pet(tree, *pet),
        GameplayNotice::UpdatePetName { pet, name } => {
            panel.update_pet_name(tree, *pet, &source.render_wire(name));
        }
        GameplayNotice::SelectionChanged(new) => panel.selection_changed(tree, *new),
        GameplayNotice::PetIsUnderAttack(pet) => panel.pet_under_attack(tree, *pet, now),
        GameplayNotice::UpdatePetMode { pet, mode } => panel.update_pet_mode(tree, *pet, *mode),
        _ => {}
    }
}
