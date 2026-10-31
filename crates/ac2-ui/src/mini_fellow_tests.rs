//! `gmMiniFellowUI` on the real gameplay layout (`AC2_DAT_DIR`).

use std::cell::RefCell;
use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::fellowship::FellowshipSource;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};
use crate::{
    build_gameplay_hud, Fellow, FellowNotice, FellowStrings, FellowshipData, GameplayHud, SkillVar,
    UiTree,
};

/// Strings as `id[vars]`; the player is 1 and has no experience shares to show.
struct Player {
    rendered: RefCell<usize>,
}

impl FellowStrings for Player {
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        *self.rendered.borrow_mut() += 1;
        let vars: Vec<String> = vars
            .iter()
            .map(|(_, v)| match v {
                SkillVar::UInt(n) => n.to_string(),
                SkillVar::Text(t) => t.clone(),
                other => format!("{other:?}"),
            })
            .collect();
        format!("{string_id:08x}[{}]", vars.join(","))
    }
}

impl FellowshipSource for Player {
    fn player(&self) -> u64 {
        1
    }

    fn fellow_xp(&self) -> Option<Vec<(u64, f32)>> {
        None
    }
}

fn player() -> Player {
    Player {
        rendered: RefCell::new(0),
    }
}

fn fellow(id: u64, name: &str, health: u32) -> Fellow {
    Fellow {
        id,
        name: name.into(),
        level: 7,
        health,
        max_health: 50,
        vigor: 20,
        max_vigor: 30,
        ..Fellow::default()
    }
}

/// The text of the row's part; empty when the template has no such part.
fn text(tree: &UiTree, root: usize, name: &str) -> String {
    tree.child_recursive(root, tokenize(name))
        .and_then(|e| tree.get(e))
        .and_then(|e| e.text())
        .unwrap_or_default()
}

/// `FellowshipMiniButton` in the tree: the toolbar's `FellowshipPage` holds it in the state the
/// page takes with a fellowship (`gmMiniFellowUI::ListenToMessage`'s element).
fn button(tree: &UiTree) -> Option<usize> {
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("FellowshipMiniButton")))
}

#[test]
fn the_mini_panel_starts_hidden_and_its_button_toggles_it() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let GameplayHud {
        tree,
        mini_fellow,
        fellowship,
        ..
    } = &mut hud;
    let panel = mini_fellow.as_mut().expect("the mini panel");
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("FellowMiniPanel"))
        })
        .expect("the FellowMiniPanel root");
    assert!(!tree.get(root).unwrap().visible());
    // No fellowship: the page is in `nofellowship` and its button is not in the tree.
    assert!(button(tree).is_none());
    let strings = player();
    let data = FellowshipData {
        name: "Wanderers".into(),
        leader: 1,
        social: true,
        looting: 2,
        fellows: vec![fellow(1, "Explorer", 50)],
    };
    fellowship
        .as_mut()
        .expect("the fellowship page")
        .update_fellowship(tree, &strings, &data);
    // With a fellowship the button is in the tree: the element a release comes from.
    let button = button(tree).expect("FellowshipMiniButton");
    let press = UiMessage {
        element_id: tokenize("FellowshipMiniButton"),
        element: Some(button),
        message: MSG_BUTTON_RELEASE_ON,
        data: 0,
        item: None,
    };
    panel.message(tree, &press);
    assert!(panel.is_shown());
    assert!(tree.get(root).unwrap().visible());
    // The location is applied again: the shown state holds.
    panel.location_applied(tree);
    assert!(tree.get(root).unwrap().visible());
    panel.message(tree, &press);
    assert!(!panel.is_shown());
    assert!(!tree.get(root).unwrap().visible());
}

#[test]
fn the_mini_list_shows_names_and_vitals_leader_first() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.mini_fellow.as_mut().expect("the mini panel");
    let tree = &mut hud.tree;
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("FellowMiniPanel"))
        })
        .expect("the FellowMiniPanel root");
    let strings = player();

    // The table lists Birch before the leader; the leader's row moves to the top.
    let data = FellowshipData {
        name: "Wanderers".into(),
        leader: 1,
        social: true,
        looting: 2,
        fellows: vec![fellow(2, "Birch", 40), fellow(1, "Explorer", 50)],
    };
    panel.update_fellowship(tree, &strings, &data);
    let rows = panel.rows(tree);
    assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2]);
    let birch = rows[1].1;
    assert_eq!(text(tree, birch, "FellowTemplateNameText"), "Birch");
    let health = format!("{:08x}[40,50]", tokenize("ID_Vital_DisplayText"));
    assert_eq!(text(tree, birch, "FellowTemplateHealthText"), health);
    // No level and no share on the mini panel (`FellowInfo::FellowInfo` with `miniPanel`).
    let level = format!("{:08x}[7]", tokenize("ID_Fellowship_Level"));
    assert_ne!(text(tree, birch, "FellowTemplateLevelText"), level);
    let share = format!("{:08x}[50]", tokenize("ID_Fellowship_Percentage"));
    assert_ne!(text(tree, birch, "FellowTemplatePercentText"), share);

    // Birch's health moves; the leader moves to the top after a leader change.
    panel.fellow_notice(
        tree,
        &strings,
        &FellowNotice::Health {
            fellow: 2,
            value: 10,
            pk: 0,
        },
    );
    let health = format!("{:08x}[10,50]", tokenize("ID_Vital_DisplayText"));
    assert_eq!(text(tree, birch, "FellowTemplateHealthText"), health);
    panel.fellow_notice(tree, &strings, &FellowNotice::Leader(2));
    assert_eq!(panel.rows(tree)[0].0, 2);

    // Birch leaves; the fellowship ends: the list is empty and the panel hides.
    panel.fellow_notice(tree, &strings, &FellowNotice::Clear(2));
    assert_eq!(panel.rows(tree).len(), 1);
    panel.show(tree, true);
    panel.clear_fellowship(tree);
    assert!(panel.rows(tree).is_empty());
    assert!(!panel.is_shown());
    assert!(!tree.get(root).unwrap().visible());
}

/// `UIFramework::LocationApplied`: the panel's saved location goes with the layout's
/// `SAVE_LOC` window, and a load applies it again (`UIMainFramework::ApplyLocations`).
#[test]
fn the_mini_panel_saves_its_location_and_takes_it_back_on_load() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel_id = tokenize("FellowMiniPanel");
    let root = |tree: &UiTree| {
        tree.roots()
            .iter()
            .copied()
            .find(|&r| tree.get(r).is_some_and(|e| e.id() == panel_id))
            .expect("the FellowMiniPanel root")
    };
    let saved_entry = |saved: &[crate::SavedWindow]| {
        saved
            .iter()
            .find(|w| w.element == panel_id)
            .map(|w| w.location)
    };
    let moved_to = {
        let root = root(&hud.tree);
        hud.tree.move_to(root, 120, 150);
        hud.tree.inq_locations()
    };
    let saved = saved_entry(&moved_to).expect("the mini panel saves its location");

    // A fresh HUD takes the saved location on load.
    let mut reloaded = build_gameplay_hud(&store, 1024, 768).unwrap();
    assert!(reloaded.tree.apply_locations(&moved_to));
    let again = saved_entry(&reloaded.tree.inq_locations()).expect("saved again");
    assert!((again.x0 - saved.x0).abs() < 1e-6 && (again.y0 - saved.y0).abs() < 1e-6);
    assert_eq!(again.shown, saved.shown);
}

/// `FellowshipHelper::ListenToMessage` on the mini panel's list: a press on a member's row
/// selects that member, and the assisted-tell press also starts a tell to them.
#[test]
fn a_press_on_a_mini_panel_row_selects_the_member() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let GameplayHud {
        tree, mini_fellow, ..
    } = &mut hud;
    let panel = mini_fellow.as_mut().expect("the mini panel");
    let data = FellowshipData {
        name: "Wanderers".into(),
        leader: 1,
        social: true,
        looting: 2,
        fellows: vec![fellow(1, "Explorer", 50), fellow(2, "Mira", 40)],
    };
    panel.update_fellowship(tree, &player(), &data);
    panel.show(tree, true);
    let rows = panel.rows(tree);
    let (id, row) = *rows.iter().find(|(id, _)| *id == 2).expect("Mira's row");
    let at = tree.screen_box(row);
    tree.pointer = ((at.x0 + at.x1) / 2, (at.y0 + at.y1) / 2);
    let list = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("FellowMiniPanelListBox")))
        .expect("the list");
    let press = |data: i32| UiMessage {
        element_id: tokenize("FellowMiniPanelListBox"),
        element: Some(list),
        message: tokenize("mouse_press"),
        data,
        item: None,
    };
    panel.message(tree, &press(0x4000_0001));
    assert_eq!(
        panel.take_requests(),
        vec![crate::FellowshipRequest::Select(id)]
    );
    panel.message(tree, &press(0x4000_0007));
    assert_eq!(
        panel.take_requests(),
        vec![
            crate::FellowshipRequest::Select(id),
            crate::FellowshipRequest::AssistedTell("Mira".into())
        ]
    );
}

/// The modern mini panel (`modern_fellow_rows`): rows 24 tall with a rule at their foot and the
/// health and vigor numbers shown; the classic rows keep retail's 15 and start the numbers hidden.
#[test]
fn the_modern_mini_rows_show_their_numbers_and_a_rule() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    for style in [crate::HudStyle::Modern, crate::HudStyle::Classic] {
        let mut hud = crate::build_styled_hud(&store, 1365, 768, style).unwrap();
        let GameplayHud {
            tree, mini_fellow, ..
        } = &mut hud;
        let panel = mini_fellow.as_mut().expect("the mini panel");
        let data = FellowshipData {
            name: "Wanderers".into(),
            leader: 1,
            social: true,
            looting: 2,
            fellows: vec![fellow(1, "Explorer", 50), fellow(2, "Mira", 40)],
        };
        panel.update_fellowship(tree, &player(), &data);
        let modern = style == crate::HudStyle::Modern;
        for (_, row) in panel.rows(tree) {
            assert_eq!(tree.el(row).rect.height(), if modern { 24 } else { 15 });
            assert_eq!(
                tree.child_recursive(row, tokenize("ModernFellowRule"))
                    .is_some(),
                modern
            );
            // The numbers: the visible texts other than the name.
            let mut shown = 0;
            let mut stack = tree.children(row).to_vec();
            while let Some(e) = stack.pop() {
                stack.extend(tree.children(e).iter().copied());
                let el = tree.get(e).unwrap();
                if el.visible() && el.text().is_some_and(|t| t.contains('[')) {
                    shown += 1;
                }
            }
            assert_eq!(shown > 0, modern, "{style:?}");
        }
    }
}
