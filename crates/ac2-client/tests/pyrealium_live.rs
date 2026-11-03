//! The gameplay suite against a running Pyrealium server (`PYREALIUM_LIVE=127.0.0.1:9000`, see
//! `tests/live/mod.rs`): each test makes its own account and character and checks what the
//! player can do with it. Skipped unless `PYREALIUM_LIVE` is set.

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use live::LiveRig;

/// The skill panel opens on the player's skills: its tree lists nodes.
///
/// This failed because Pyrealium's player `CreateObject` named no script package, so the player had no
/// script process to answer the panel's queries (fixed in Pyrealium: `PLAYER_PACKAGE`).
#[test]
fn the_skill_panel_lists_the_players_skills() {
    let Some(mut rig) = LiveRig::new("Skilltest") else {
        return;
    };
    rig.panel(ToolbarChild::Skill, true);
    let listed = rig.until(20.0, |r| !r.view.skill_nodes().is_empty());
    let kinds: std::collections::BTreeSet<String> = rig
        .seen
        .iter()
        .map(|e| {
            format!("{e:?}")
                .split(['(', ' ', '{'])
                .next()
                .unwrap_or("")
                .to_owned()
        })
        .collect();
    eprintln!("events seen: {kinds:?}");
    eprintln!("skill nodes: {:?}", rig.view.skill_nodes());
    assert!(listed, "the skill panel lists no skills");
}

/// The quest panel's act list (`gmCWInterface::InqActInfoList`, IFun 0xB4008F) runs the player's
/// script over the scenes its WState unpacked: no script crashes after the login and the panel
/// shown.
#[test]
fn the_quest_panel_runs_the_players_script_without_a_crash() {
    let Some(mut rig) = LiveRig::new("Questlist") else {
        return;
    };
    rig.panel(ToolbarChild::Quest, true);
    rig.run(3.0);
    let crashes: Vec<String> = rig
        .client
        .take_script_problems()
        .into_iter()
        .filter(|p| p.contains("Crash"))
        .collect();
    assert!(crashes.is_empty(), "script crashes: {crashes:?}");
}
