//! `gmPetUI` on the real gameplay layout (`AC2_DAT_DIR`), and the mode table's cycling.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::hud::{load_layout_by_enum, LAYOUT_GAMEPLAY};
use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};
use crate::{next_mode, previous_mode, PetAction, PetUi, UiTree, PET_MODES};

const IDLE: u32 = 0x4000_0005;
const DEFENSIVE: u32 = 0x4000_0004;
const AGGRESSIVE: u32 = 0x20;
const PET: u64 = 0x8000_0000_0000_0A01;
const OTHER: u64 = 0x8000_0000_0000_0A02;

/// The cycling follows `GetNextMode` / `GetPreviousMode`: the table's order, wrapping, and the
/// first (next) or last (previous) mode for an id the table does not list.
#[test]
fn modes_cycle_in_table_order_and_wrap() {
    assert_eq!(next_mode(AGGRESSIVE), 0x1000);
    assert_eq!(next_mode(IDLE), AGGRESSIVE);
    assert_eq!(previous_mode(AGGRESSIVE), IDLE);
    assert_eq!(previous_mode(0x1000), AGGRESSIVE);
    assert_eq!(next_mode(0x1234), AGGRESSIVE);
    assert_eq!(previous_mode(0x1234), IDLE);
    assert_eq!(PET_MODES.len(), 9);
}

fn gameplay() -> Option<Arc<crate::layout::LayoutDesc>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ResourceStore::new(dataset);
    load_layout_by_enum(&store, LAYOUT_GAMEPLAY).unwrap()
}

fn element(tree: &UiTree, panel: usize, name: &str) -> usize {
    tree.child_recursive(panel, tokenize(name))
        .unwrap_or_else(|| panic!("{name} in the pet panel"))
}

fn release(tree: &UiTree, panel: usize, name: &str) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: Some(element(tree, panel, name)),
        message: MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    }
}

fn panel_root(tree: &UiTree) -> usize {
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).is_some_and(|e| e.id() == tokenize("PetPanel")))
        .expect("the PetPanel root")
}

#[test]
fn the_panel_lists_pets_and_ghosts_its_buttons_until_one_is_listed() {
    let Some(layout) = gameplay() else {
        return;
    };
    let mut tree = UiTree::new(1024, 768);
    let mut ui = PetUi::create(&mut tree, &layout);
    let panel = panel_root(&tree);
    assert!(!ui.is_shown());
    // Drawn only while shown and a pet is listed.
    assert!(!tree.get(panel).is_some_and(|e| e.visible()));
    assert_eq!(tree.get(panel).map(|e| e.state()), Some(tokenize("nopets")));
    assert_eq!(
        tree.get(element(&tree, panel, "AttackButton"))
            .and_then(|e| e.attribute(BUTTON_GHOSTED).cloned()),
        Some(crate::layout::AttributeValue::Bool(true)),
        "no pet: the buttons are ghosted"
    );
    ui.show(&mut tree, true);
    assert!(!tree.get(panel).is_some_and(|e| e.visible()), "no pet yet");
    ui.add_pet(&mut tree, PET, "Rex", 0, IDLE);
    assert_eq!(ui.pets(), vec![PET]);
    assert!(tree.get(panel).is_some_and(|e| e.visible()));
    assert_eq!(tree.get(panel).map(|e| e.state()), Some(tokenize("pets")));
    assert_eq!(
        tree.get(element(&tree, panel, "AttackButton"))
            .and_then(|e| e.attribute(BUTTON_GHOSTED).cloned()),
        Some(crate::layout::AttributeValue::Bool(false))
    );
    // A pet already listed is not listed twice.
    ui.add_pet(&mut tree, PET, "Rex", 0, IDLE);
    assert_eq!(ui.pets().len(), 1);
    // The UI option off ghosts the buttons again.
    ui.set_show_pet_ui(&mut tree, false);
    assert_eq!(tree.get(panel).map(|e| e.state()), Some(tokenize("nopets")));
    assert!(!tree.get(panel).is_some_and(|e| e.visible()));
}

#[test]
fn a_pet_under_attack_is_unsafe_until_five_quiet_seconds_pass() {
    let Some(layout) = gameplay() else {
        return;
    };
    let mut tree = UiTree::new(1024, 768);
    let mut ui = PetUi::create(&mut tree, &layout);
    ui.add_pet(&mut tree, PET, "Rex", 0, IDLE);
    let panel = panel_root(&tree);
    // The row's name is the first `Name` below the list.
    let row = tree.list_items(element(&tree, panel, "PetPanelListBox"))[0];
    let name = tree.child_recursive(row, tokenize("Name")).unwrap();
    assert_eq!(tree.get(name).map(|e| e.state()), Some(tokenize("safe")));
    ui.pet_under_attack(&mut tree, PET, 10.0);
    assert_eq!(tree.get(name).map(|e| e.state()), Some(tokenize("unsafe")));
    // Refresh runs once more than five seconds have passed since its last pass (the first pass
    // is at 14 s, four seconds after the attack: still unsafe). The next pass is at 20 s.
    assert!(ui.tick(&mut tree, 14.0).is_empty());
    assert_eq!(tree.get(name).map(|e| e.state()), Some(tokenize("unsafe")));
    assert!(ui.tick(&mut tree, 16.0).is_empty());
    assert_eq!(tree.get(name).map(|e| e.state()), Some(tokenize("unsafe")));
    assert!(ui.tick(&mut tree, 20.0).is_empty());
    assert_eq!(tree.get(name).map(|e| e.state()), Some(tokenize("safe")));
}

#[test]
fn the_buttons_ask_the_script_and_the_arrows_want_a_mode_sent_by_refresh() {
    let Some(layout) = gameplay() else {
        return;
    };
    let mut tree = UiTree::new(1024, 768);
    let mut ui = PetUi::create(&mut tree, &layout);
    ui.show(&mut tree, true);
    ui.add_pet(&mut tree, PET, "Rex", 0, IDLE);
    ui.add_pet(&mut tree, OTHER, "Mo", 0, DEFENSIVE);
    let panel = panel_root(&tree);
    // The buttons ask: attack on the selection (none gives 0), return, heal and buff.
    let attack = release(&tree, panel, "AttackButton");
    let ret = release(&tree, panel, "ReturnButton");
    let heal = release(&tree, panel, "HealButton");
    let buff = release(&tree, panel, "BuffButton");
    assert_eq!(
        ui.message(&mut tree, &attack, Some(0x77)),
        vec![PetAction::Attack(0x77)]
    );
    assert_eq!(
        ui.message(&mut tree, &attack, None),
        vec![PetAction::Attack(0)]
    );
    assert_eq!(ui.message(&mut tree, &ret, None), vec![PetAction::Return]);
    assert_eq!(ui.message(&mut tree, &heal, None), vec![PetAction::Heal]);
    assert_eq!(ui.message(&mut tree, &buff, None), vec![PetAction::Buff]);
    // The first row's next-mode arrow steps its wanted mode (Idle is last: the wrap gives
    // Aggressive) and sends nothing yet.
    let row = tree.list_items(element(&tree, panel, "PetPanelListBox"))[0];
    let arrow = tree.child_recursive(row, tokenize("NextMode")).unwrap();
    let next = UiMessage {
        element_id: tokenize("NextMode"),
        element: Some(arrow),
        message: MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    };
    assert!(ui.message(&mut tree, &next, None).is_empty());
    let mode = tree.child_recursive(row, tokenize("Mode")).unwrap();
    assert_eq!(
        tree.get(mode).map(|e| e.state()),
        Some(tokenize("Aggressive"))
    );
    // The next refresh sends the wanted mode for that row only; the second row is unchanged.
    let sent = ui.tick(&mut tree, 6.0);
    assert_eq!(
        sent,
        vec![PetAction::Mode {
            pet: PET,
            mode: AGGRESSIVE
        }]
    );
    // A press on a row selects its pet.
    let press = UiMessage {
        element_id: 0,
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data: 0,
        item: None,
    };
    assert_eq!(
        ui.message(&mut tree, &press, None),
        vec![PetAction::Select(PET)]
    );
}

/// `gmPetUI::RecvNotice_UpdatePetMode`: a row that wanted its old mode follows the server (its
/// wanted mode and text move with it); a row that wanted another mode keeps that wanted mode,
/// so `tick` sends it.
#[test]
fn a_pet_mode_notice_moves_a_row_that_was_following_the_server() {
    let Some(layout) = gameplay() else {
        return;
    };
    let mut tree = UiTree::new(1024, 768);
    let mut ui = PetUi::create(&mut tree, &layout);
    let panel = panel_root(&tree);
    ui.show(&mut tree, true);
    ui.add_pet(&mut tree, PET, "Rex", 0, AGGRESSIVE);
    let mode_state = |tree: &UiTree| {
        let row = tree.list_items(element(tree, panel, "PetPanelListBox"))[0];
        let mode = tree.child_recursive(row, tokenize("Mode")).unwrap();
        tree.get(mode).map(|e| e.state())
    };
    // The row wants Aggressive, its mode: the server's Defensive is followed, text included.
    ui.update_pet_mode(&mut tree, PET, DEFENSIVE);
    assert_eq!(mode_state(&tree), Some(tokenize("Defensive")));
    assert!(
        ui.tick(&mut tree, 10.0).is_empty(),
        "the row follows the server"
    );
    // The player steps the row's wanted mode on to Supporting; then the server says
    // Aggressive. The row does not follow it, so `tick` sends the wanted mode.
    let next = release(&tree, panel, "NextMode");
    ui.message(&mut tree, &next, None);
    ui.update_pet_mode(&mut tree, PET, AGGRESSIVE);
    assert_eq!(
        ui.tick(&mut tree, 20.0),
        [PetAction::Mode {
            pet: PET,
            mode: 0x4000_0002
        }]
    );
}

fn row_name(tree: &UiTree, panel: usize) -> (usize, usize) {
    let row = tree.list_items(element(tree, panel, "PetPanelListBox"))[0];
    (row, tree.child_recursive(row, tokenize("Name")).unwrap())
}

/// `gmPetUI::RecvNotice_UpdatePetName` (and `AddPet`, which calls it): the first `(...)` of the
/// server's name is removed when the `(` is not first; `RemovePet` takes the row out. (The
/// tests have no font, so the name is read from the panel's table.)
#[test]
fn names_lose_their_parenthetical_and_a_removed_pet_leaves_the_list() {
    let Some(layout) = gameplay() else {
        return;
    };
    let mut tree = UiTree::new(1024, 768);
    let mut ui = PetUi::create(&mut tree, &layout);
    let panel = panel_root(&tree);
    ui.show(&mut tree, true);
    ui.add_pet(&mut tree, PET, "Rex (Lvl 3)", 0, IDLE);
    assert_eq!(ui.name_of(PET), Some("Rex "));
    ui.update_pet_name(&mut tree, PET, "(odd) Rex");
    assert_eq!(ui.name_of(PET), Some("(odd) Rex"), "a `(` first is kept");
    ui.update_pet_name(&mut tree, PET, "Max (a) (b)");
    assert_eq!(ui.name_of(PET), Some("Max  (b)"));
    // A pet not listed is ignored.
    ui.update_pet_name(&mut tree, OTHER, "Zed");
    assert_eq!(ui.name_of(OTHER), None);
    ui.remove_pet(&mut tree, PET);
    let list = element(&tree, panel, "PetPanelListBox");
    assert!(tree.list_items(list).is_empty());
    assert!(ui.pets().is_empty());
    assert_eq!(tree.get(panel).map(|e| e.state()), Some(tokenize("nopets")));
}

/// `gmPetUI::RecvNotice_SelectionChanged` marks the selected pet's row; a right press makes the
/// name editable and a changed name is requested when it deactivates.
#[test]
fn selection_marks_a_row_and_a_right_press_edits_the_name() {
    let Some(layout) = gameplay() else {
        return;
    };
    let mut tree = UiTree::new(1024, 768);
    let mut ui = PetUi::create(&mut tree, &layout);
    let panel = panel_root(&tree);
    ui.show(&mut tree, true);
    ui.add_pet(&mut tree, PET, "Rex", 0, IDLE);
    let (row, name) = row_name(&tree, panel);
    ui.selection_changed(&mut tree, PET);
    assert_eq!(tree.get(row).map(|e| e.state()), Some(tokenize("selected")));
    let press = UiMessage {
        element_id: 0,
        element: Some(row),
        message: MSG_MOUSE_PRESS,
        data: 0x4000_0005,
        item: None,
    };
    assert_eq!(
        ui.message(&mut tree, &press, None),
        vec![PetAction::Select(PET)]
    );
    assert!(tree.is_text_editable(name));
    assert_eq!(tree.active(), Some(name));
    // The field is empty here (no font), so leaving it differs from the stored name: requested.
    let deactivated = UiMessage {
        element_id: tokenize("Name"),
        element: Some(name),
        message: tokenize("deactivated"),
        data: 0,
        item: None,
    };
    assert_eq!(
        ui.message(&mut tree, &deactivated, None),
        vec![PetAction::Rename {
            pet: PET,
            name: String::new()
        }]
    );
    assert!(!tree.is_text_editable(name));
    // A second `deactivated` is not listened to any more.
    assert!(ui.message(&mut tree, &deactivated, None).is_empty());
    // Another selection ends the row's selected look.
    ui.selection_changed(&mut tree, OTHER);
    assert_eq!(
        tree.get(row).map(|e| e.state()),
        Some(tokenize("unselected"))
    );
}

#[test]
fn an_edited_name_is_compared_ignoring_case() {
    assert!(!super::pets::name_differs("Rex", "REX"));
    assert!(super::pets::name_differs("Rex", "Max"));
}
