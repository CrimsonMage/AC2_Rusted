//! `gmStoryQuestsUI` on the real toolbar layout (`AC2_DAT_DIR`): the acts laid out from the
//! script's list, the rows opened and closed, a scene row's press asking for its scene, and a
//! notice while the panel is hidden flashing the quest button and opening the Story tab.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_net::protocol::{GmActInfo, GmSceneInfo, StringInfo};

use crate::journal_quests::{find_named, JournalSource};
use crate::mouse::UiMessage;
use crate::story_quests::{StoryQuestAction, STORY_CODE_LIST};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};
use crate::{build_gameplay_hud, UiTree};

struct Names;

impl crate::ui_string::UiStrings for Names {
    fn render_string(&self, string: &crate::ui_string::UiString) -> String {
        format!("{:08x}", string.string_id)
    }
}

impl JournalSource for Names {
    fn render_info(&self, _info: &StringInfo) -> String {
        "name".into()
    }
}

fn scene(id: u32, num: u32, playable: bool, hidden: bool) -> GmSceneInfo {
    GmSceneInfo {
        id,
        name: StringInfo::default(),
        act_id: 7,
        scene_num: num,
        is_hidden: u32::from(hidden),
        is_playable: u32::from(playable),
        keyframes: Vec::new(),
    }
}

/// Act 7 with three scenes: 70 playable, 71 not, 72 hidden (left out of the rows).
fn act(id: u32, num: u32) -> GmActInfo {
    GmActInfo {
        id,
        name: StringInfo::default(),
        act_num: num,
        visible: 1,
        playable: 1,
        scenes: vec![
            scene(70, 1, true, false),
            scene(71, 2, false, false),
            scene(72, 3, true, true),
        ],
    }
}

fn state_is(tree: &UiTree, name: &str, state: &str) -> bool {
    find_named(tree, name)
        .and_then(|e| tree.get(e))
        .is_some_and(|e| e.state() == tokenize(state))
}

/// The Story page's act list.
fn story_list(tree: &UiTree) -> usize {
    let page = find_named(tree, "StoryPage").expect("the Story page");
    tree.child_recursive(page, tokenize("ActListBox"))
        .expect("the act list")
}

/// The press message of a row: `element` named `name`, with the message and its data.
fn press(element: usize, name: &str, message: u32, data: i32) -> UiMessage {
    UiMessage {
        element_id: tokenize(name),
        element: Some(element),
        message,
        data,
        item: None,
    }
}

#[test]
fn the_act_list_lays_out_the_acts_and_opens_one_at_a_time() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");
    let list = story_list(tree);
    assert!(tree.list_items(list).is_empty());

    // The script's list, reloaded by the global loop after a notice for act 7.
    quests.recv_story_notice(tree, STORY_CODE_LIST, 0);
    assert!(quests.story().reload_pending());
    quests.story_global_loop(tree, &Names, || vec![act(7, 1), act(8, 2)]);
    assert_eq!(quests.story().acts().len(), 2);
    assert_eq!(tree.list_items(list).len(), 2, "two act rows, both closed");

    // Pressing an act row opens it: its visible scenes follow it (70 and 71; 72 is hidden).
    let first = tree.list_items(list)[0];
    let release = press(first, "ActTemplate", MSG_BUTTON_RELEASE_ON, 0);
    assert_eq!(quests.story_message(tree, &release), None);
    quests.story_global_loop(tree, &Names, || vec![act(7, 1), act(8, 2)]);
    assert_eq!(
        tree.list_items(list).len(),
        4,
        "act 7's two scene rows added"
    );
    let first = tree.list_items(list)[0];
    assert_eq!(
        tree.get(first).map(|e| e.state()),
        Some(tokenize("highlight"))
    );

    // Pressing it again closes it.
    let first = tree.list_items(list)[0];
    quests.story_message(tree, &press(first, "ActTemplate", MSG_BUTTON_RELEASE_ON, 0));
    quests.story_global_loop(tree, &Names, || vec![act(7, 1), act(8, 2)]);
    assert_eq!(tree.list_items(list).len(), 2);
}

/// `ReloadActs` lists only the visible acts.
#[test]
fn an_act_that_is_not_visible_has_no_row() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");
    let list = story_list(tree);
    quests.recv_story_notice(tree, STORY_CODE_LIST, 0);
    quests.story_global_loop(tree, &Names, || {
        let mut hidden = act(8, 2);
        hidden.visible = 0;
        vec![act(7, 1), hidden, act(9, 3)]
    });
    assert_eq!(tree.list_items(list).len(), 2, "acts 7 and 9");
}

#[test]
fn a_playable_scene_row_asks_for_its_scene_and_others_do_not() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");
    let list = story_list(tree);
    quests.recv_story_notice(tree, STORY_CODE_LIST, 0);
    quests.story_global_loop(tree, &Names, || vec![act(7, 1)]);
    let first = tree.list_items(list)[0];
    quests.story_message(tree, &press(first, "ActTemplate", MSG_BUTTON_RELEASE_ON, 0));
    quests.story_global_loop(tree, &Names, || vec![act(7, 1)]);

    // Rows: act 7, scene 70 (playable), scene 71 (not playable).
    let rows = tree.list_items(list).to_vec();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        tree.get(rows[1]).map(|e| e.state()),
        Some(tokenize("completed"))
    );
    assert_eq!(
        tree.get(rows[2]).map(|e| e.state()),
        Some(tokenize("notcompleted"))
    );
    let scene_press =
        |element: usize, data: i32| press(element, "SceneTemplate", MSG_MOUSE_PRESS, data);
    assert_eq!(
        quests.story_message(tree, &scene_press(rows[1], 0x4000_0007)),
        Some(StoryQuestAction::PlayScene { act: 7, scene: 70 })
    );
    assert_eq!(
        quests.story_message(tree, &scene_press(rows[2], 0x4000_0007)),
        None
    );
    // Another press code on a scene row does nothing.
    assert_eq!(
        quests.story_message(tree, &scene_press(rows[1], 0x4000_0001)),
        None
    );
}

#[test]
fn a_notice_while_the_panel_is_hidden_flashes_the_button_and_opens_the_story_tab() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let quests = hud.quests.as_mut().expect("the quest panel");
    assert!(!quests.shown());

    quests.recv_story_notice(tree, 0x4000_0001, 7);
    assert!(state_is(tree, "QuestButton", "changed"));
    assert!(quests.story().reload_pending());
    assert!(!quests.journal().shown());

    // Once the panel is open, the notice flashes nothing; the list notice never does.
    quests.sync(tree, true);
    quests.recv_story_notice(tree, STORY_CODE_LIST, 0);
    assert!(state_is(tree, "QuestButton", "unchanged"));
}
