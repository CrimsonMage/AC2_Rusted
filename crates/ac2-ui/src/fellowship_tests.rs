//! `gmFellowshipUI` on the real toolbar layout (`AC2_DAT_DIR`).

use std::cell::RefCell;

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON};
use crate::{
    build_gameplay_hud, Fellow, FellowNotice, FellowStrings, FellowshipData, FellowshipRequest,
    FellowshipSource, SkillVar, UiTree,
};

/// Strings as `id[vars]`; the player is 1; Birch has half the experience.
struct Player {
    rendered: RefCell<Vec<String>>,
}

impl FellowStrings for Player {
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        let vars: Vec<String> = vars
            .iter()
            .map(|(_, v)| match v {
                SkillVar::UInt(n) => n.to_string(),
                SkillVar::Int(n) => n.to_string(),
                SkillVar::Text(t) => t.clone(),
                SkillVar::Float { value, .. } => format!("{value}"),
                other => format!("{other:?}"),
            })
            .collect();
        let text = format!("{string_id:08x}[{}]", vars.join(","));
        self.rendered.borrow_mut().push(text.clone());
        text
    }
}

impl FellowshipSource for Player {
    fn player(&self) -> u64 {
        1
    }

    fn fellow_xp(&self) -> Option<Vec<(u64, f32)>> {
        Some(vec![(2, 0.5), (1, 0.5)])
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

fn text(tree: &UiTree, root: usize, name: &str) -> String {
    let element = tree.child_recursive(root, tokenize(name)).unwrap();
    tree.get(element).unwrap().text().unwrap_or_default()
}

fn ghosted(tree: &UiTree, page: usize, name: &str) -> bool {
    let button = tree.child_recursive(page, tokenize(name)).unwrap();
    tree.bool_attribute(button, BUTTON_GHOSTED)
}

#[test]
fn the_fellowship_page_lists_the_members_leader_first() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.fellowship.as_mut().expect("the fellowship page");
    let tree = &mut hud.tree;
    let page = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("FellowshipPage")))
        .unwrap();
    // No fellowship: no list yet.
    assert_eq!(tree.get(page).unwrap().state(), tokenize("nofellowship"));
    assert!(tree
        .child_recursive(page, tokenize("FellowshipListBox"))
        .is_none());

    let player = Player {
        rendered: RefCell::new(Vec::new()),
    };
    // The table lists Birch before the leader; the leader's row moves to the top.
    let data = FellowshipData {
        name: "Wanderers".into(),
        leader: 1,
        social: true,
        looting: 2,
        fellows: vec![fellow(2, "Birch", 40), fellow(1, "Explorer", 50)],
    };
    panel.update_fellowship(tree, &player, &data);
    assert_eq!(tree.get(page).unwrap().state(), tokenize("fellowship"));
    assert_eq!(text(tree, page, "FellowshipNameText"), "Wanderers");
    let rows = panel.rows(tree);
    assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2]);
    let birch = rows[1].1;
    assert_eq!(text(tree, birch, "FellowTemplateNameText"), "Birch");
    let level = format!("{:08x}[7]", tokenize("ID_Fellowship_Level"));
    assert_eq!(text(tree, birch, "FellowTemplateLevelText"), level);
    let health = format!("{:08x}[40,50]", tokenize("ID_Vital_DisplayText"));
    assert_eq!(text(tree, birch, "FellowTemplateHealthText"), health);
    let share = format!("{:08x}[50]", tokenize("ID_Fellowship_Percentage"));
    assert_eq!(text(tree, birch, "FellowTemplatePercentText"), share);
    // The player leads: the leader's buttons are live.
    assert!(!ghosted(tree, page, "FellowshipDisbandButton"));

    // Birch takes the lead: the buttons ghost and Birch's row goes to the top.
    panel.fellow_notice(tree, &player, &FellowNotice::Leader(2));
    assert!(ghosted(tree, page, "FellowshipRecruitButton"));
    assert_eq!(panel.rows(tree)[0].0, 2);

    // A hit: Birch's health text follows.
    panel.fellow_notice(
        tree,
        &player,
        &FellowNotice::Health {
            fellow: 2,
            value: 10,
            pk: 0,
        },
    );
    let health = format!("{:08x}[10,50]", tokenize("ID_Vital_DisplayText"));
    assert_eq!(text(tree, birch, "FellowTemplateHealthText"), health);

    // Leave sends its request.
    let leave = tree
        .child_recursive(page, tokenize("FellowshipLeaveButton"))
        .unwrap();
    panel.message(
        tree,
        &UiMessage {
            element_id: tokenize("FellowshipLeaveButton"),
            element: Some(leave),
            message: MSG_BUTTON_RELEASE_ON,
            data: 0,
            item: None,
        },
    );
    assert_eq!(panel.take_requests(), vec![FellowshipRequest::Leave]);

    // A member leaves, then the fellowship ends.
    panel.fellow_notice(tree, &player, &FellowNotice::Clear(2));
    assert_eq!(panel.rows(tree).len(), 1);
    panel.clear_fellowship(tree);
    assert_eq!(tree.get(page).unwrap().state(), tokenize("nofellowship"));
    assert!(panel.rows(tree).is_empty());
}

#[test]
fn the_looting_drop_down_holds_five_methods_and_create_sends_the_chosen_one() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let panel = hud.fellowship.as_mut().expect("the fellowship page");
    let tree = &mut hud.tree;
    let page = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("FellowshipPage")))
        .unwrap();
    // `InitLootingMenu`: five methods, method 1 selected.
    let menu = tree
        .child_recursive(page, tokenize("LootingMethodMenu"))
        .unwrap();
    assert_eq!(tree.menu_items(menu).len(), 5);
    assert_eq!(tree.menu_selected_index(menu), Some(1));
    // Each item has its tooltip, switched on.
    for item in tree.menu_items(menu) {
        assert!(tree.tooltip_text(*item).is_some_and(|t| !t.is_empty()));
    }

    // The drop-down's choice is not a request of its own: the page's Create carries it.
    tree.menu_set_selected_index(menu, Some(3));
    for message in tree.take_messages() {
        panel.message(tree, &message);
    }
    assert!(panel.take_requests().is_empty());

    let name = tree
        .child_recursive(page, tokenize("NoFellowshipNameEntryText"))
        .unwrap();
    tree.set_text(name, "Wanderers");
    let create = tokenize("FellowshipCreateButton");
    let release = UiMessage {
        element_id: create,
        element: tree.child_recursive(page, create),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    };
    panel.message(tree, &release);
    let requests = panel.take_requests();
    assert!(
        matches!(
            requests.as_slice(),
            [FellowshipRequest::Create { name, looting: 3, .. }] if name == "Wanderers"
        ),
        "{requests:?}"
    );
}
