//! Story scenes over the Legions server double (`AC2_DAT_DIR`): the server's grant
//! (`WM_Quest` 0x23 -> `CPlayer` 0x2A -> `CQuestSystem::UpdateStoryQuest_Done(errorCode, bAdd,
//! sceneId)`) adds a scene to the player's `ActRegistry` (the login's empty one, argument 2 of
//! `WM_Player` 0x27), the Quest panel's Story page lists it playable
//! (`QuestSystem::InqActInfoList` -> `ggmSceneInfo::SetPlayable`), and a removal takes it off
//! again. A double click on a scene row (the Story tab selected as a player does) plays the story
//! and its end sends the played list. Needs a GPU adapter.
#![cfg(feature = "window")]

mod hud;

use ac2_net::protocol::{pack_values, PackValue};
use ac2_ui::{tokenize, ToolbarChild};
use hud::Rig;
use legions_server_double::UPDATE_STORY_QUEST;

/// The Prologue act (`ActPrologue`) and its first scene.
const PROLOGUE: u32 = 0x4000_0008;
const FIRST_SCENE: u32 = 0x4000_0021;
const OTHER_SCENE: u32 = 0x4000_0067;

/// The element called `name` whose attribute `attribute` is the id `want`.
fn row_with(rig: &Rig, name: &str, attribute: &str, want: u32) -> Option<usize> {
    let tree = rig.view.ui_tree()?;
    elements(rig, name).into_iter().find(|&i| {
        matches!(
            tree.get(i).and_then(|e| e.attribute(tokenize(attribute))),
            Some(ac2_ui::AttributeValue::Id(id)) if *id == want
        )
    })
}

/// Clicks the Quest panel's Story tab and waits for the Story page to show.
fn open_story_page(rig: &mut Rig) {
    let tab = rig
        .view
        .ui_element_center("StoryTab")
        .expect("the Story tab");
    rig.click(tab);
    rig.wait("the Story page shown", |r| {
        r.view.ui_element_center("ActListBox").is_some()
    });
}

/// The window pixel at the centre of row `row`'s child `name` (a click on the row's label; the
/// scene icons beside it are a list box that takes the press).
fn child_center(rig: &Rig, row: usize, name: &str) -> (i32, i32) {
    let tree = rig.view.ui_tree().expect("the HUD");
    let child = elements(rig, name)
        .into_iter()
        .find(|&i| tree.get(i).and_then(|e| e.parent_index()) == Some(row))
        .expect("the row's label");
    rig.view.ui_index_center(child).expect("the label's place")
}

fn grant(rig: &mut Rig, scene: u32, add: bool) {
    rig.server.borrow_mut().send_script_call(
        UPDATE_STORY_QUEST,
        &[
            PackValue::Int(0),
            PackValue::Int(u32::from(add)),
            PackValue::Int(scene),
        ],
    );
}

/// Whether the Story page lists `scene` of the Prologue as playable.
fn playable(rig: &Rig, scene: u32) -> Option<bool> {
    rig.view
        .story_acts()
        .iter()
        .find(|a| a.id == PROLOGUE)?
        .scenes
        .iter()
        .find(|s| s.id == scene)
        .map(|s| s.is_playable != 0)
}

fn elements(rig: &Rig, name: &str) -> Vec<usize> {
    let Some(tree) = rig.view.ui_tree() else {
        return Vec::new();
    };
    let id = tokenize(name);
    let mut found = Vec::new();
    let mut stack: Vec<usize> = tree.roots().to_vec();
    while let Some(index) = stack.pop() {
        if tree.get(index).is_some_and(|e| e.id() == id) {
            found.push(index);
        }
        stack.extend_from_slice(tree.children(index));
    }
    found.sort_unstable();
    found
}

/// The grant's arguments are the three ints `(errorCode, bAdd, sceneId)` AC2e writes with
/// `script::integer` (a count of 1, then the value).
#[test]
fn the_grant_arguments_are_three_ints() {
    let bytes = pack_values(&[
        PackValue::Int(0),
        PackValue::Int(1),
        PackValue::Int(FIRST_SCENE),
    ]);
    assert_eq!(
        bytes,
        [1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0x21, 0, 0, 0x40]
    );
}

#[test]
fn a_granted_scene_is_playable_on_the_story_page_until_it_is_removed() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.server.borrow_mut().send_init_player();
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.panel(ToolbarChild::Quest, true);
    rig.idle(8);

    grant(&mut rig, FIRST_SCENE, true);
    rig.wait("the first scene playable", |r| {
        playable(r, FIRST_SCENE) == Some(true)
    });
    assert_eq!(
        playable(&rig, OTHER_SCENE),
        Some(false),
        "only the granted scene"
    );
    assert!(rig.client.take_script_problems().is_empty());

    // The Story page is a hidden page of the Quest panel until its tab is clicked
    // (`gmQuestsUI` page buttons); a click before that lands on the visible page.
    open_story_page(&mut rig);

    // The granted scene's act is the updated one, so the Prologue is open and lists the scene's
    // row (`gmStoryQuestsUI`'s `m_updatedActID`). A click on the act's label closes it and the next
    // opens it again.
    assert!(row_with(&rig, "SceneTemplate", "sceneid", FIRST_SCENE).is_some());
    for open in [false, true] {
        let act_row = row_with(&rig, "ActTemplate", "actid", PROLOGUE).expect("the Prologue's row");
        let at = child_center(&rig, act_row, "ActName");
        rig.click(at);
        rig.wait("the Prologue toggled", |r| {
            row_with(r, "SceneTemplate", "sceneid", FIRST_SCENE).is_some() == open
        });
    }

    // A removal (bAdd 0) takes it off the page again.
    grant(&mut rig, FIRST_SCENE, false);
    rig.wait("the scene not playable", |r| {
        playable(r, FIRST_SCENE) == Some(false)
    });
    assert!(rig.client.take_script_problems().is_empty());
}

/// The UI action of a double click (`0x40000007`), and the skip key (`gmStoryUI::ListenToMessage`).
const DOUBLE_CLICK: u32 = 0x4000_0007;
const SKIP_KEY: u32 = 0x4000_001E;
/// The played-list call (`HandlePlayScenesDone` -> the server's function 0x82130013).
const PLAYED_FUNCTION: u32 = 0x8213_0013;

/// The packed arguments of the played-list call for the Prologue's first scene (0x40000021) as the
/// DAT holds it (`PList<gmSceneInfo>`, one scene with its 19 keyframes): the layout of
/// `story_script_7160.rs`'s `PLAYED_LIST`, with this scene's id, name, act and keyframes. The scene
/// id is the first word after the list count.
const PLAYED_LIST: &[u8] = &[
    5, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 68, 0, 0, 0, 0, 164, 0, 255, 255, 1, 0, 0, 0, 33,
    0, 0, 64, 230, 133, 19, 212, 190, 20, 0, 37, 0, 0, 0, 0, 8, 0, 0, 64, 1, 0, 0, 0, 0, 0, 0, 0,
    1, 0, 0, 0, 19, 0, 0, 0, 33, 0, 0, 64, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 125, 73, 0, 65, 0,
    0, 0, 0, 17, 3, 0, 44, 55, 139, 109, 201, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 21, 0, 0, 0,
    33, 0, 0, 64, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 250, 100, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 55,
    139, 110, 201, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 33, 0, 0, 64, 3, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 20, 64, 251, 100, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33, 0, 0, 64, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 252, 100,
    0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33,
    0, 0, 64, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 55, 139,
    105, 201, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 33, 0, 0, 64, 6, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 64, 125, 73, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 0, 0, 0, 33, 0, 0, 64, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 64, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 55, 139, 107, 201, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 33,
    0, 0, 64, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 64, 253, 100, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 55,
    139, 100, 201, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 33, 0, 0, 64, 9, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 64, 254, 100, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33, 0, 0, 64, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 26, 200, 188, 64, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0,
    33, 0, 0, 64, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 64, 255, 100, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33, 0, 0, 64, 12, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 64, 0, 101, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 26, 204, 188, 64, 190, 20, 0, 37, 0, 0,
    0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 33, 0, 0, 64, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 1, 101, 0,
    65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33, 0,
    0, 64, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 2, 101, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33, 0, 0, 64, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8,
    64, 3, 101, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 26, 194, 188, 64, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0,
    0, 0, 17, 0, 0, 0, 33, 0, 0, 64, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 64, 4, 101, 0, 65, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 33, 0, 0, 64, 17, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 26, 198, 188, 64, 190, 20,
    0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 33, 0, 0, 64, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24,
    64, 5, 101, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    0, 0, 0, 33, 0, 0, 64, 19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 40, 64, 125, 73, 0, 65, 0, 0, 0, 0, 0, 0,
    0, 0, 26, 218, 188, 64, 190, 20, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0,
];

/// Whether the first element called `name` is shown, with the parents' shown states.
fn shown(rig: &Rig, name: &str) -> bool {
    let Some(tree) = rig.view.ui_tree() else {
        return false;
    };
    elements(rig, name).into_iter().any(|mut i| loop {
        if !tree.get(i).is_some_and(|e| e.visible()) {
            return false;
        }
        match tree.get(i).and_then(|e| e.parent_index()) {
            Some(parent) => i = parent,
            None => return true,
        }
    })
}

/// The shown field's image and text: the `ImageField` and `TextField` under a shown story field.
fn field_content(rig: &Rig) -> (Option<ac2_ui::UiImage>, Option<String>) {
    let tree = rig.view.ui_tree().expect("the HUD");
    let mut image = None;
    let mut text = None;
    for i in elements(rig, "ImageField") {
        let e = tree.get(i).unwrap();
        if e.visible() && e.image().is_some() {
            image = e.image();
        }
    }
    for i in elements(rig, "TextField") {
        let e = tree.get(i).unwrap();
        if e.visible() && e.text().is_some_and(|t| !t.is_empty()) {
            text = e.text();
        }
    }
    (image, text)
}

/// A real double click on the Story page's scene row plays the story (`gmStoryQuestsUI` plays a
/// scene row on `MSG_MOUSE_PRESS` with data 0x40000007): the story view shows the scene's first
/// keyframe, the skip key ends it, and the played list goes to the server.
#[test]
fn a_double_click_on_a_scene_row_plays_the_story_and_reports_it_played() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    rig.server.borrow_mut().send_init_player();
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.panel(ToolbarChild::Quest, true);
    rig.idle(8);
    grant(&mut rig, FIRST_SCENE, true);
    rig.wait("the first scene playable", |r| {
        playable(r, FIRST_SCENE) == Some(true)
    });
    open_story_page(&mut rig);
    rig.wait("the scene's row", |r| {
        row_with(r, "SceneTemplate", "sceneid", FIRST_SCENE).is_some()
    });
    assert!(!shown(&rig, "BackgroundField"), "no story yet");
    let keyframe = rig
        .view
        .story_acts()
        .iter()
        .find(|a| a.id == PROLOGUE)
        .and_then(|a| a.scenes.iter().find(|s| s.id == FIRST_SCENE))
        .and_then(|s| s.keyframes.first().cloned())
        .expect("the scene's first keyframe");

    let row = row_with(&rig, "SceneTemplate", "sceneid", FIRST_SCENE).unwrap();
    let at = child_center(&rig, row, "SceneName");
    let before = rig.server.borrow().script_bytes.len();
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);

    // The story view: the background and the first keyframe's image and subtitle.
    rig.wait("the story background", |r| shown(r, "BackgroundField"));
    rig.wait("the first keyframe", |r| field_content(r).0.is_some());
    let (image, text) = field_content(&rig);
    assert_eq!(image.map(|i| i.file), Some(keyframe.video_fragment));
    // The keyframe's subtitle is the entry 0xC96D8B37 of string table 0x250014BE, which the DAT
    // holds empty (`dat_index`: 7160 and 7230); the shared `StringInfo` render prints the
    // table error for it.
    assert_eq!(
        text.as_deref(),
        Some("<string table error; token: 0xC96D8B37 tableDID: 0x250014BE>")
    );

    // The player skips it; the story hides and the played list goes out.
    rig.frame(&[(SKIP_KEY, 1.0, true)]);
    rig.frame(&[(SKIP_KEY, 1.0, false)]);
    rig.wait("the story hidden", |r| !shown(r, "BackgroundField"));
    rig.wait("the played list", |r| {
        r.server.borrow().script_bytes[before..]
            .iter()
            .any(|(f, _)| *f == PLAYED_FUNCTION)
    });
    let sent: Vec<_> = rig.server.borrow().script_bytes[before..]
        .iter()
        .filter(|(f, _)| *f == PLAYED_FUNCTION)
        .map(|(_, b)| b.clone())
        .collect();
    assert_eq!(sent.len(), 1, "one played list");
    // The list holds the one scene played (count 1, id 0x40000021), byte for byte.
    assert_eq!(sent[0][24..32], [1, 0, 0, 0, 0x21, 0, 0, 0x40]);
    assert_eq!(sent[0], PLAYED_LIST);
    assert!(rig.client.take_script_problems().is_empty());
}
