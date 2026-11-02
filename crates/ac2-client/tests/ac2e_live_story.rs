//! Story scenes against a running AC2e server (`AC2E_LIVE`, see `tests/live/mod.rs`): a new
//! character is granted the Prologue's scenes (AC2e policy, `emulator/src/story.rs`: the retail
//! server's trigger is not in the client or the data), the Quest panel's Story page lists them
//! playable (`WM_Quest` 0x23 -> `CQuestSystem::UpdateStoryQuest_Done`), and a relog keeps them
//! playable (the login's registry is empty; the scenes are granted again). A double click on a
//! scene row of the Story page (its tab clicked first, as a player does) plays the story, the skip
//! key ends it, and AC2e logs the played list it received (`gmStoryQuestsUI` -> `gmStoryUI::Play`
//! -> `HandlePlayScenesDone` -> the server's function 0x82130013).
//!
//! Run with `AC2E_LIVE_TEST=ac2e_live_story scripts/ac2e-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_ui::{tokenize, ToolbarChild};
use live::LiveRig;

/// The Prologue act (`ActPrologue`) and its scenes.
const PROLOGUE: u32 = 0x4000_0008;
const SCENES: [u32; 3] = [0x4000_0021, 0x4000_0067, 0x4000_0068];

/// The Prologue's scenes the Story page lists as playable, in list order.
fn playable(rig: &LiveRig) -> Vec<u32> {
    rig.view
        .story_acts()
        .iter()
        .find(|a| a.id == PROLOGUE)
        .map(|a| {
            a.scenes
                .iter()
                .filter(|s| s.is_playable != 0)
                .map(|s| s.id)
                .collect()
        })
        .unwrap_or_default()
}

/// The UI action of a double click (`0x40000007`), and the skip key (`gmStoryUI::ListenToMessage`).
const DOUBLE_CLICK: u32 = 0x4000_0007;
const SKIP_KEY: u32 = 0x4000_001E;
/// The first Prologue scene and the size of the played list for it: the 980 bytes of
/// `story_scenes_7160`'s `PLAYED_LIST` (the double test passes on the 7160 and the 7230 DATs).
const FIRST_SCENE: u32 = 0x4000_0021;
const PLAYED_LIST_BYTES: usize = 980;

/// The indices of the HUD elements called `name`.
fn elements(rig: &LiveRig, name: &str) -> Vec<usize> {
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

/// Whether an element called `name` is shown with all its parents.
fn shown(rig: &LiveRig, name: &str) -> bool {
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

/// The scene row of `FIRST_SCENE` on the Story page.
fn scene_row(rig: &LiveRig) -> Option<usize> {
    let tree = rig.view.ui_tree()?;
    elements(rig, "SceneTemplate").into_iter().find(|&i| {
        matches!(
            tree.get(i).and_then(|e| e.attribute(tokenize("sceneid"))),
            Some(ac2_ui::AttributeValue::Id(id)) if *id == FIRST_SCENE
        )
    })
}

/// The lines of the server's log (`AC2E_STATE` is the live script's state folder).
fn server_log() -> String {
    let state = std::env::var("AC2E_STATE").expect("AC2E_STATE (scripts/ac2e-live-test.sh)");
    std::fs::read_to_string(format!("{state}/server.log")).unwrap_or_default()
}

fn sorted(mut ids: Vec<u32>) -> Vec<u32> {
    ids.sort_unstable();
    ids
}

#[test]
fn a_new_characters_prologue_scenes_are_playable_and_survive_a_relog() {
    let Some(mut rig) = LiveRig::new("Storyline") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    rig.wait("the Prologue's scenes playable", |r| {
        playable(r).len() == SCENES.len()
    });
    assert_eq!(sorted(playable(&rig)), SCENES);

    // A relog: the scenes are granted again after the login's empty registry.
    rig.relog();
    rig.panel(ToolbarChild::Quest, true);
    rig.wait("the scenes playable after the relog", |r| {
        playable(r).len() == SCENES.len()
    });
    assert_eq!(sorted(playable(&rig)), SCENES);
}

#[test]
fn a_double_click_on_a_prologue_scene_plays_the_story_and_the_server_takes_the_played_list() {
    let Some(mut rig) = LiveRig::new("Playstory") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    rig.wait("the Prologue's scenes playable", |r| {
        playable(r).len() == SCENES.len()
    });

    // The Story tab, as a player clicks it; the page lists the Prologue's scenes.
    let tab = rig
        .view
        .ui_element_center("StoryTab")
        .expect("the Story tab");
    rig.click(tab);
    // The Prologue's row may be closed (an act opens when its own notice was the last): the
    // player clicks its label to open it.
    if !rig.until(3.0, |r| scene_row(r).is_some()) {
        let tree = rig.view.ui_tree().unwrap();
        let act = elements(&rig, "ActTemplate")
            .into_iter()
            .find(|&i| {
                matches!(
                    tree.get(i).and_then(|e| e.attribute(tokenize("actid"))),
                    Some(ac2_ui::AttributeValue::Id(id)) if *id == PROLOGUE
                )
            })
            .expect("the Prologue's row");
        let label = elements(&rig, "ActName")
            .into_iter()
            .find(|&i| tree.get(i).and_then(|e| e.parent_index()) == Some(act))
            .expect("the act's label");
        let at = rig.view.ui_index_center(label).expect("the label's place");
        rig.click(at);
    }
    rig.wait("the scene's row", |r| scene_row(r).is_some());
    assert!(!shown(&rig, "BackgroundField"), "no story yet");
    let keyframe = rig
        .view
        .story_acts()
        .iter()
        .find(|a| a.id == PROLOGUE)
        .and_then(|a| a.scenes.iter().find(|s| s.id == FIRST_SCENE))
        .and_then(|s| s.keyframes.first().cloned())
        .expect("the scene's first keyframe");

    // The double click on the row's label: the press, then the double-click action.
    let at = {
        let row = scene_row(&rig).unwrap();
        let tree = rig.view.ui_tree().unwrap();
        let name = elements(&rig, "SceneName")
            .into_iter()
            .find(|&i| tree.get(i).and_then(|e| e.parent_index()) == Some(row))
            .expect("the row's label");
        rig.view.ui_index_center(name).expect("the label's place")
    };
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);

    // The story view: the background, then the first keyframe's image and subtitle.
    rig.wait("the story background", |r| shown(r, "BackgroundField"));
    let content = |r: &LiveRig| {
        let tree = r.view.ui_tree().expect("the HUD");
        let image = elements(r, "ImageField")
            .into_iter()
            .filter_map(|i| tree.get(i).filter(|e| e.visible()).and_then(|e| e.image()))
            .next();
        let text = elements(r, "TextField")
            .into_iter()
            .filter_map(|i| tree.get(i).filter(|e| e.visible()).and_then(|e| e.text()))
            .find(|t| !t.is_empty());
        (image, text)
    };
    rig.wait("the first keyframe", |r| content(r).0.is_some());
    let (image, _) = content(&rig);
    assert_eq!(image.map(|i| i.file), Some(keyframe.video_fragment));
    // The subtitle renders on a worker ("Loading ..." until its string table is read) and is an
    // entry of string table 0x250014BE that the DAT holds empty. The keyframe may have advanced
    // by then (each keyframe has its own token), so only the table is checked.
    rig.wait("the subtitle", |r| {
        content(r).1.is_some_and(|t| t != "Loading ...")
    });
    let (_, text) = content(&rig);
    let text = text.expect("the subtitle");
    assert!(
        text.starts_with("<string table error; token: 0x")
            && text.ends_with(" tableDID: 0x250014BE>"),
        "{text}"
    );

    // The player skips it: the story hides and the played list goes to AC2e.
    rig.frame(&[(SKIP_KEY, 1.0, true)]);
    rig.frame(&[(SKIP_KEY, 1.0, false)]);
    rig.wait("the story hidden", |r| !shown(r, "BackgroundField"));
    let line = "played-scene list accepted for character";
    let tail = format!("({PLAYED_LIST_BYTES} bytes)");
    rig.wait("AC2e took the played list", |_| {
        server_log()
            .lines()
            .any(|l| l.contains(line) && l.ends_with(&tail))
    });
    // The connection stays up and the scenes stay playable.
    rig.run(1.0);
    assert!(rig.client.frame().in_world, "still in the world");
    assert_eq!(sorted(playable(&rig)), SCENES);
}
