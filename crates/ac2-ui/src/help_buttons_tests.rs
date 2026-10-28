//! The help buttons of the panels that open a help page, on the real gameplay layout
//! (`AC2_DAT_DIR`). Each button's element name is the one its retail class tokenizes
//! (`Tokenize("<Name>")` in the class's init), and each maps to the topic its
//! `ListenToMessage` opens with `KeyStone::OpenHelp`.

use ac2_db::ResourceStore;

use crate::help::{help_button_topic, HELP_BUTTONS};
use crate::hud::{load_layout_by_enum, GAMEPLAY_ROOTS, LAYOUT_GAMEPLAY};
use crate::token::tokenize;

/// The retail callers of `KeyStone::OpenHelp` that a panel's own button opens: (class, button
/// element, topic). `gmBookUI`'s and `gmSocialUI`'s buttons are answered by their panels
/// ([`crate::help_window`] callers in `hud_book` and `social_page`), the rest by [`HELP_BUTTONS`].
const PANEL_BUTTONS: &[(&str, &str, &str)] = &[
    ("gmShopUI", "ShopHelpButton", "ID_Help_Shop"),
    (
        "gmShopCatalogUI",
        "CatalogHelpButton",
        "ID_Help_ShopCatalog",
    ),
    (
        "gmShopConsignmentUI",
        "ConsignmentHelpButton",
        "ID_Help_ShopConsignment",
    ),
    (
        "gmShopNewSaleUI",
        "NewSaleHelpButton",
        "ID_Help_ShopConsignment",
    ),
    ("gmBookUI", "BookHelpButton", "ID_Help_Book"),
    ("gmSkillUI", "SkillHelpButton", "ID_Help_Skill"),
    (
        "gmNavigationUI",
        "NavigationHelpButton",
        "ID_Help_Navigation",
    ),
    ("gmOptionUI", "OptionsHelpButton", "ID_Help_Option"),
    (
        "gmOverviewMapUI",
        "OverviewMapHelpButton",
        "ID_Help_OverviewMap",
    ),
    ("gmInventoryUI", "InventoryHelpButton", "ID_Help_Inventory"),
    ("gmQuestsUI", "QuestHelpButton", "ID_Help_Quest"),
    ("gmSocialUI", "SocialHelpButton", "ID_Help_Social"),
    (
        "gmExaminationUI",
        "ExaminationHelpButton",
        "ID_Help_Examine",
    ),
];

#[test]
fn every_panel_help_button_is_in_the_gameplay_layout_and_maps_to_its_topic() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    // The gameplay layout and the layouts of its roots (the toolbar's panels are in those).
    let mut layouts = Vec::new();
    for layout_enum in std::iter::once(LAYOUT_GAMEPLAY)
        .chain(GAMEPLAY_ROOTS.iter().map(|(layout_enum, _)| *layout_enum))
    {
        if let Some(layout) = load_layout_by_enum(&store, layout_enum).unwrap() {
            layouts.push(layout);
        }
    }
    let in_layouts = |id: u32| {
        layouts
            .iter()
            .any(|layout| layout.find_element(id).is_some())
    };
    let mut missing = Vec::new();
    let mut unmapped = Vec::new();
    for (class, button, topic) in PANEL_BUTTONS {
        if !in_layouts(tokenize(button)) {
            missing.push(format!("{class}: {button}"));
        }
        // The panels that answer their own button are not in the shared table.
        if matches!(*class, "gmBookUI" | "gmSocialUI") {
            continue;
        }
        let mapped = help_button_topic(tokenize(button)).map(|t| t.id);
        if mapped != Some(*topic) {
            unmapped.push(format!("{class}: {button} maps to {mapped:?}, not {topic}"));
        }
    }
    assert!(
        missing.is_empty(),
        "help buttons not in the layout: {missing:?}"
    );
    assert!(
        unmapped.is_empty(),
        "help buttons mapped wrongly: {unmapped:?}"
    );
    // Every shared-table entry is one of the panels' buttons.
    for (button, _) in HELP_BUTTONS {
        assert!(
            PANEL_BUTTONS.iter().any(|(_, name, _)| name == button),
            "{button} is a panel button"
        );
    }
}
