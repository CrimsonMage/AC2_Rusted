//! The gameplay suite against a running AC2e server (`AC2E_LIVE=127.0.0.1:9000`, see
//! `tests/live/mod.rs`): each test makes its own account and character and checks what the
//! player can do with it. Skipped unless `AC2E_LIVE` is set.

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use live::LiveRig;

/// The skill panel opens on the player's skills: its tree lists nodes.
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
