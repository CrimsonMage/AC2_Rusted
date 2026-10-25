//! `gmCharacterUI` on the real toolbar and gameplay layouts (`AC2_DAT_DIR`): the tree, the
//! refresh from a profile, the buttons, Apply, Recall, Default and the expanders.

use ac2_db::ResourceStore;

use crate::character_options::{
    OptionSetting, OptionsSettings, Setter, PROFILE_ACCEPT_ALLEGIANCE, PROFILE_ACCEPT_FELLOWSHIP,
    PROFILE_ACCEPT_TRADE, PROFILE_AUTO_TARGET, PROFILE_BYPASS_GOLD_MELTING,
    PROFILE_HIDE_FELLOWS_ON_MAP, PROFILE_HIDE_PET_UI, PROFILE_LATENCY_INDICATOR_GRAPH,
    PROFILE_SHORTCUT_INSTA_SCROLL, PROFILE_SHORTCUT_LOCKED, PROFILE_SHOW_DAMAGE_TEXT,
    PROFILE_SHOW_PK_DAMAGE, PROFILE_SHOW_RADAR_COORDS, PROFILE_SHOW_SHORTCUT_NUMBERS,
};
use crate::dialog::{DialogOutcome, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::token::{
    tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON,
    MSG_HIGHLIGHT_STATE_CHANGE, MSG_MOUSE_PRESS,
};
use crate::{build_gameplay_hud, CharacterRequest, CharacterUi, DialogFactory, GameplayHud};

/// The radar mask with monsters, NPCs, PCs, portals and lifestones (not items).
const RADAR_MASK: u32 = 0x1F;

/// The player's profile: auto target and the coordinates on, the damage text off, the radar
/// mask as [`RADAR_MASK`], the shortcut bar 2 rows high, the third-party range a quarter.
fn profile() -> OptionsSettings {
    OptionsSettings {
        bits: PROFILE_AUTO_TARGET | PROFILE_SHOW_RADAR_COORDS,
        radar_mask: RADAR_MASK,
        shortcut_height: 2,
        damage_text_range_other: 0.25,
    }
}

/// The profile the defaults give (every checkbox at its `Setup` default).
fn defaults() -> OptionsSettings {
    OptionsSettings {
        bits: PROFILE_AUTO_TARGET
            | PROFILE_SHOW_DAMAGE_TEXT
            | PROFILE_SHOW_PK_DAMAGE
            | PROFILE_SHOW_RADAR_COORDS
            | PROFILE_ACCEPT_ALLEGIANCE
            | PROFILE_ACCEPT_FELLOWSHIP
            | PROFILE_ACCEPT_TRADE
            | PROFILE_SHOW_SHORTCUT_NUMBERS
            | PROFILE_SHORTCUT_LOCKED
            | PROFILE_SHORTCUT_INSTA_SCROLL
            | PROFILE_BYPASS_GOLD_MELTING
            | PROFILE_LATENCY_INDICATOR_GRAPH,
        radar_mask: RADAR_MASK,
        shortcut_height: 1,
        damage_text_range_other: 1.0,
    }
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

fn message(element_id: &str, element: Option<usize>, message: u32) -> UiMessage {
    UiMessage {
        element_id: tokenize(element_id),
        element,
        message,
        data: 0,
        item: None,
    }
}

/// The checkbox of `name` shows its highlight.
fn checked(tree: &UiTree, page: &CharacterUi, name: &str) -> bool {
    let control = page.control(name).expect(name);
    matches!(
        tree.get(control)
            .and_then(|e| e.attribute(BUTTON_HIGHLIGHTED)),
        Some(AttributeValue::Bool(true))
    )
}

/// The user ticks or unticks `name` (the checkbox's highlight changes and its message follows).
fn tick(
    tree: &mut UiTree,
    page: &mut CharacterUi,
    dialogs: &mut DialogFactory,
    name: &str,
    on: bool,
) {
    let control = page.control(name).expect(name);
    tree.set_bool(control, BUTTON_HIGHLIGHTED, on);
    page.message(
        tree,
        dialogs,
        &message("CheckboxButton", Some(control), MSG_HIGHLIGHT_STATE_CHANGE),
    );
}

/// The user presses `element_id`'s button.
fn press(tree: &mut UiTree, page: &mut CharacterUi, dialogs: &mut DialogFactory, element_id: &str) {
    let element = tree.child_recursive(page.page(), tokenize(element_id));
    page.message(
        tree,
        dialogs,
        &message(element_id, element, MSG_BUTTON_RELEASE_ON),
    );
}

fn ghosted(tree: &UiTree, page: &CharacterUi, element_id: &str) -> bool {
    let button = tree
        .child_recursive(page.page(), tokenize(element_id))
        .expect(element_id);
    matches!(
        tree.get(button).and_then(|e| e.attribute(BUTTON_GHOSTED)),
        Some(AttributeValue::Bool(true))
    )
}

/// The tab opens (the panel's `opened` message).
fn open(tree: &mut UiTree, page: &mut CharacterUi, dialogs: &mut DialogFactory) {
    page.message(
        tree,
        dialogs,
        &UiMessage {
            element_id: tokenize("CharacterTab"),
            element: None,
            message: MSG_OPENED,
            data: 0,
            item: None,
        },
    );
}

/// The user answers the shown confirmation: its `YesButton` or `NoButton` is pressed, the dialog
/// closes and its outcomes are the page's to take.
fn answer(tree: &mut UiTree, dialogs: &mut DialogFactory, yes: bool) -> Vec<DialogOutcome> {
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ConfirmationField"))
        })
        .expect("the confirmation dialog");
    let name = if yes { "YesButton" } else { "NoButton" };
    let button = tree.child_recursive(root, tokenize(name));
    dialogs.message(tree, &message(name, button, MSG_BUTTON_RELEASE_ON));
    dialogs.take_outcomes()
}

const ORDER: [&str; 27] = [
    "ID_Options_Combat",
    "ID_Option_AutoTargetText",
    "ID_Option_DamageNumbersText",
    "ID_Option_ThirdPartyDamageText",
    "ID_Option_ShowPKDamage",
    "ID_Option_ThirdPartyDamageRange",
    "ID_Options_RadarFilter",
    "ID_Option_RadarShowMonsters",
    "ID_Option_RadarShowNPCs",
    "ID_Option_RadarShowPCs",
    "ID_Option_RadarShowPortals",
    "ID_Option_RadarShowLifestones",
    "ID_Option_RadarShowItems",
    "ID_Option_RadarShowCoordinates",
    "ID_Options_Social",
    "ID_Option_AcceptAllegianceText",
    "ID_Option_AcceptFellowshipText",
    "ID_Option_AcceptTradeText",
    "ID_Options_MiscGameplay",
    "ID_Option_ShortcutNumbersText",
    "ID_Option_ShortcutLocked",
    "ID_Option_ShortcutInstaScroll",
    "ID_Option_GoldMeltingConfirmation",
    "ID_Option_LatencyIndicatorGraph",
    "ID_Option_ShowFellowsOnMap",
    "ID_Option_ShowPetUI",
    "ID_Option_ShortcutHeight",
];

#[test]
fn the_page_lists_the_expanders_and_options_in_setup_order() {
    let Some(mut hud) = hud() else {
        return;
    };
    let page = hud.character.take().expect("CharacterPage");
    assert_eq!(page.order(), ORDER);
    // The top level: the four expanders, dismantled (their options not listed).
    assert_eq!(
        page.listed(&hud.tree),
        [
            "ID_Options_Combat",
            "ID_Options_RadarFilter",
            "ID_Options_Social",
            "ID_Options_MiscGameplay"
        ]
    );
    assert_eq!(page.rows(&hud.tree, "ID_Options_Combat"), Some(0));
    assert_eq!(page.is_open("ID_Options_Combat"), Some(false));
    // Each option is labelled and has its help as the tooltip (`OptionBase::Describe`).
    for name in ORDER.iter().filter(|n| n.starts_with("ID_Option_")) {
        let element = page.element(name).expect(name);
        let label = hud
            .tree
            .child_recursive(element, tokenize("OptionText"))
            .and_then(|t| hud.tree.get(t)?.text());
        assert!(label.is_some_and(|l| !l.is_empty()), "{name}: no label");
        assert!(
            hud.tree
                .tooltip_text(element)
                .is_some_and(|t| !t.is_empty()),
            "{name}: no tooltip"
        );
    }
}

#[test]
fn refresh_from_a_profile_sets_each_checkbox_and_value() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    assert!(checked(tree, &page, "ID_Option_AutoTargetText"));
    assert!(!checked(tree, &page, "ID_Option_DamageNumbersText"));
    assert!(checked(tree, &page, "ID_Option_RadarShowMonsters"));
    assert!(!checked(tree, &page, "ID_Option_RadarShowItems"));
    assert!(checked(tree, &page, "ID_Option_RadarShowCoordinates"));
    assert!(!checked(tree, &page, "ID_Option_AcceptTradeText"));
    // The gold-melting confirmation is ticked while the bypass is clear.
    assert!(checked(tree, &page, "ID_Option_GoldMeltingConfirmation"));
    // The hidden bits show the opposite: fellows and pets are shown.
    assert!(checked(tree, &page, "ID_Option_ShowFellowsOnMap"));
    assert!(checked(tree, &page, "ID_Option_ShowPetUI"));
    assert_eq!(page.value("ID_Option_ThirdPartyDamageRange"), Some(0.25));
    assert_eq!(page.value("ID_Option_ShortcutHeight"), Some(2.0));
    assert!(!page.changed());
    // The hidden flags, set in the profile, hide them.
    let hidden = OptionsSettings {
        bits: profile().bits | PROFILE_HIDE_FELLOWS_ON_MAP | PROFILE_HIDE_PET_UI,
        ..profile()
    };
    page.refresh(tree, &hidden);
    assert!(!checked(tree, &page, "ID_Option_ShowFellowsOnMap"));
    assert!(!checked(tree, &page, "ID_Option_ShowPetUI"));
    assert!(!page.changed());
}

#[test]
fn toggling_marks_changed_and_unghosts_apply_and_recall() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    open(tree, &mut page, &mut dialogs);
    assert!(ghosted(tree, &page, "ApplyButton"));
    assert!(ghosted(tree, &page, "RecallButton"));
    tick(
        tree,
        &mut page,
        &mut dialogs,
        "ID_Option_DamageNumbersText",
        true,
    );
    assert!(page.changed());
    assert!(!ghosted(tree, &page, "ApplyButton"));
    assert!(!ghosted(tree, &page, "RecallButton"));
    tick(
        tree,
        &mut page,
        &mut dialogs,
        "ID_Option_DamageNumbersText",
        false,
    );
    assert!(!page.changed());
    assert!(ghosted(tree, &page, "ApplyButton"));
}

#[test]
fn apply_yields_the_setter_requests_for_the_changed_options_only() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    open(tree, &mut page, &mut dialogs);
    tick(
        tree,
        &mut page,
        &mut dialogs,
        "ID_Option_AutoTargetText",
        false,
    );
    assert!(page.set_value(tree, "ID_Option_ThirdPartyDamageRange", 0.5));
    assert!(page.set_value(tree, "ID_Option_ShortcutHeight", 4.0));
    tick(
        tree,
        &mut page,
        &mut dialogs,
        "ID_Option_RadarShowItems",
        true,
    );
    press(tree, &mut page, &mut dialogs, "ApplyButton");
    // Each changed flag's setter, then the radar mask (`ApplyFlag`); a radar flag sets only the
    // mask; the range and the height go to their setters; then the upload.
    assert_eq!(
        page.take_requests(),
        [
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::AutoTarget,
                value: 0
            }),
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::RadarMask,
                value: RADAR_MASK
            }),
            CharacterRequest::Set(OptionSetting::Float {
                setter: Setter::DamageTextRangeOther,
                value: 0.5
            }),
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::RadarMask,
                value: RADAR_MASK | 0x20
            }),
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::ShortcutHeight,
                value: 4
            }),
            CharacterRequest::Upload,
        ]
    );
    // Applied: nothing differs now, so Apply ghosts again, and a second apply only uploads.
    assert!(!page.changed());
    assert!(ghosted(tree, &page, "ApplyButton"));
    press(tree, &mut page, &mut dialogs, "ApplyButton");
    assert_eq!(page.take_requests(), [CharacterRequest::Upload]);
}

#[test]
fn recall_asks_then_restores_the_saved_values() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    open(tree, &mut page, &mut dialogs);
    tick(
        tree,
        &mut page,
        &mut dialogs,
        "ID_Option_AutoTargetText",
        false,
    );
    assert!(page.set_value(tree, "ID_Option_ThirdPartyDamageRange", 0.9));
    press(tree, &mut page, &mut dialogs, "RecallButton");
    // The question is up and nothing is restored yet.
    let (context, kind) = dialogs.shown().expect("the recall question");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert!(!checked(tree, &page, "ID_Option_AutoTargetText"));
    assert!(page.changed());
    // The answer yes: restored, nothing sent.
    let outcomes = answer(tree, &mut dialogs, true);
    assert_eq!(
        outcomes,
        [DialogOutcome::Confirmation { context, yes: true }]
    );
    page.dialog_outcomes(tree, &mut dialogs, &outcomes);
    assert!(checked(tree, &page, "ID_Option_AutoTargetText"));
    assert_eq!(page.value("ID_Option_ThirdPartyDamageRange"), Some(0.25));
    assert!(!page.changed());
    assert!(page.take_requests().is_empty());
    assert!(dialogs.shown().is_none());
}

#[test]
fn default_asks_then_applies_the_defaults() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    // Auto target is off and the shortcut bar is 3 rows; the defaults are on and 1.
    let mut start = defaults();
    start.bits &= !PROFILE_AUTO_TARGET;
    start.shortcut_height = 3;
    page.refresh(tree, &start);
    open(tree, &mut page, &mut dialogs);
    press(tree, &mut page, &mut dialogs, "DefaultButton");
    let (context, kind) = dialogs.shown().expect("the default question");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert!(page.take_requests().is_empty());
    assert_eq!(page.value("ID_Option_ShortcutHeight"), Some(3.0));
    let outcomes = answer(tree, &mut dialogs, true);
    assert_eq!(
        outcomes,
        [DialogOutcome::Confirmation { context, yes: true }]
    );
    page.dialog_outcomes(tree, &mut dialogs, &outcomes);
    assert!(checked(tree, &page, "ID_Option_AutoTargetText"));
    assert_eq!(page.value("ID_Option_ShortcutHeight"), Some(1.0));
    assert_eq!(
        page.take_requests(),
        [
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::AutoTarget,
                value: 1
            }),
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::RadarMask,
                value: RADAR_MASK
            }),
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::ShortcutHeight,
                value: 1
            }),
            CharacterRequest::Upload,
        ]
    );
}

#[test]
fn a_no_keeps_the_values() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    open(tree, &mut page, &mut dialogs);
    tick(
        tree,
        &mut page,
        &mut dialogs,
        "ID_Option_AutoTargetText",
        false,
    );
    press(tree, &mut page, &mut dialogs, "RecallButton");
    let outcomes = answer(tree, &mut dialogs, false);
    page.dialog_outcomes(tree, &mut dialogs, &outcomes);
    assert!(!checked(tree, &page, "ID_Option_AutoTargetText"));
    assert!(page.changed());
    assert!(dialogs.shown().is_none());
}

#[test]
fn the_expanders_collapse_and_expand_their_options() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    let header = page.header("ID_Options_Combat").expect("the combat header");
    // A press opens it: its five options listed; a second press takes them out.
    page.message(
        tree,
        &mut dialogs,
        &message("TreeToggle", Some(header), MSG_BUTTON_RELEASE_ON),
    );
    assert_eq!(page.is_open("ID_Options_Combat"), Some(true));
    assert_eq!(page.rows(tree, "ID_Options_Combat"), Some(5));
    page.message(
        tree,
        &mut dialogs,
        &message("TreeToggle", Some(header), MSG_BUTTON_RELEASE_ON),
    );
    assert_eq!(page.is_open("ID_Options_Combat"), Some(false));
    assert_eq!(page.rows(tree, "ID_Options_Combat"), Some(0));
    // The other expanders are untouched.
    assert_eq!(page.is_open("ID_Options_Social"), Some(false));
}

#[test]
fn the_slider_arrows_step_a_tenth_of_the_range() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let mut dialogs = hud.dialogs;
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    let right = tree
        .child_recursive(
            page.element("ID_Option_ThirdPartyDamageRange").unwrap(),
            tokenize("SliderRight"),
        )
        .expect("SliderRight");
    page.message(
        tree,
        &mut dialogs,
        &message("SliderRight", Some(right), MSG_MOUSE_PRESS),
    );
    let value = page.value("ID_Option_ThirdPartyDamageRange").unwrap();
    assert!((value - 0.35).abs() < 1e-9, "{value}");
    assert!(page.changed());
}

#[test]
fn the_shortcut_bar_height_is_taken_and_uploaded() {
    let Some(mut hud) = hud() else {
        return;
    };
    let mut page = hud.character.take().expect("CharacterPage");
    let tree = &mut hud.tree;
    page.refresh(tree, &profile());
    page.recv_shortcut_height(tree, 5);
    assert_eq!(page.value("ID_Option_ShortcutHeight"), Some(5.0));
    assert!(!page.changed());
    assert_eq!(
        page.take_requests(),
        [
            CharacterRequest::Set(OptionSetting::Int {
                setter: Setter::ShortcutHeight,
                value: 5
            }),
            CharacterRequest::Upload,
        ]
    );
}
