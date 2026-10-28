//! `gmPetUI`: the pet panel, the `PetPanel` root of the gameplay layout (`0x41000009`). Its
//! `PetPanelListBox` lists one `PetTemplate` row per pet: the pet's name (`Name`, state `safe` or
//! `unsafe`), its mode (`Mode`, one state per mode name, `Unique` for a mode not in the table) and
//! the `NextMode` / `PreviousMode` arrows. The four buttons under the panel (`AttackButton`,
//! `ReturnButton`, `HealButton`, `BuffButton`) act on the selection or the pet.
//!
//! The pet notices reach the panel as `CM_Character::SendNotice_AddPet`, `PetIsUnderAttack` and
//! `UpdatePetMode` (Legions 7160's `gmPetUI::RecvNotice_UpdatePetMode`, which is the only pet
//! notice handler the Legions decompile names). `RemovePet` and `UpdatePetName` have no Legions
//! handler identified, so the panel does not change for them. The mode table and the request
//! timer follow the admin 7148 build (`ModeTable` from its static initializer,
//! `gmPetUI::Refresh`'s five-second cycle).
//!
//! Not modelled yet: the pet's icon image (`AddImageToState`), the list's height for its rows, the
//! name edit (`RequestSetPetName` is a command, see `ac2-client`), and removing a pet's row.

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

    /// `gmPetUI::RecvNotice_AddPet`: a `PetTemplate` row for the pet, its name and mode shown and
    /// its wanted mode the same as the server's. A pet already listed is not listed twice.
    pub fn add_pet(&mut self, tree: &mut UiTree, pet: u64, name: &str, mode: u32) {
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
        if let Some(name_text) = tree.child_recursive(root, id("Name")) {
            tree.set_text(name_text, name);
            tree.set_state(name_text, id("safe"));
        }
        if let Some(mode_text) = tree.child_recursive(root, id("Mode")) {
            tree.set_state(mode_text, id(mode_state(mode)));
        }
        tree.list_add_item(list, root);
        self.pets.push(PetRow {
            id: pet,
            root,
            mode,
            next_mode: mode,
            last_attacked: 0.0,
        });
        self.update_state(tree);
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
            }
        }
        actions
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

/// What the pet panel reads from the player: the server's strings, rendered (a pet's name).
pub trait PetSource {
    /// A string the server sent, rendered.
    fn render_wire(&self, info: &StringInfo) -> String;
}

/// `gmPetUI`'s notices: `AddPet` lists the pet, `PetIsUnderAttack` marks it unsafe and
/// `UpdatePetMode` takes the server's mode. `RemovePet` and `UpdatePetName` change nothing in
/// the panel (see the module notes).
pub fn pet_notice<S: PetSource>(
    panel: &mut PetUi,
    tree: &mut UiTree,
    source: &S,
    now: f64,
    notice: &GameplayNotice,
) {
    match notice {
        GameplayNotice::AddPet {
            pet, name, mode, ..
        } => panel.add_pet(tree, *pet, &source.render_wire(name), *mode),
        GameplayNotice::PetIsUnderAttack(pet) => panel.pet_under_attack(tree, *pet, now),
        GameplayNotice::UpdatePetMode { pet, mode } => panel.update_pet_mode(tree, *pet, *mode),
        _ => {}
    }
}
