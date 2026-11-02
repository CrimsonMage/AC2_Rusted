//! A trained skill dragged from the skill panel onto the shortcut bar against a running AC2e
//! (`scripts/ac2e-live-test.sh` with `AC2E_LIVE_TEST=ac2e_live_skills_drag`, the skills profile:
//! experience and credits to train with): the classic HUD at 2560 x 1440, as the player plays.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_net::protocol::ShortcutInfo;
use ac2_weenie::SkillRequest;
use live::LiveRig;

const HEAL: u32 = 0x55;
const HEAL_PREREQUISITE: u32 = 0x59;
const SKILL_SHORTCUT: u32 = 0x4000_0001;

fn wait(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}");
}

#[test]
fn a_trained_skill_drags_from_the_skill_panel_onto_the_bar() {
    if std::env::var_os("AC2E_DEV_START_XP").is_none() {
        eprintln!(
            "run through scripts/ac2e-live-test.sh with AC2E_LIVE_TEST=ac2e_live_skills_drag"
        );
        return;
    }
    let Some(mut rig) = LiveRig::new("Skilldrag") else {
        return;
    };
    rig.view
        .set_hud_style(&rig.gpu, ac2_gpu::COLOR_FORMAT, ac2_ui::HudStyle::Classic);
    wait(&mut rig, "the classic HUD", |r| r.view.ui_tree().is_some());
    rig.view.resize(2560, 1440);
    rig.view.set_ui_scale(ac2_ui::UiScaleSetting::Retail);
    rig.run(1.0);
    wait(&mut rig, "the script and experience", |r| {
        r.client.skill_credits().is_ok() && r.client.frame().progress.experience.available_xp > 0
    });
    for skill in [HEAL_PREREQUISITE, HEAL] {
        rig.command(Command::Skill(SkillRequest::Train(skill)));
        wait(&mut rig, "trained", |r| {
            r.client.is_skill_trained(skill) == Ok(true)
        });
    }
    wait(&mut rig, "the bar", |r| r.view.shortcut_slots() > 0);
    rig.panel(ac2_ui::ToolbarChild::Skill, true);
    let mut node = None;
    for root in [
        ac2_ui::ROOT_MELEE_PANEL,
        ac2_ui::ROOT_MISSILE_PANEL,
        ac2_ui::ROOT_MAGIC_PANEL,
        ac2_ui::ROOT_NONCOMBAT_PANEL,
    ] {
        let Ok(Some(panel)) = rig.client.root_panel_from_race(root) else {
            continue;
        };
        rig.command(Command::Examine(ac2_net::protocol::ExaminationRequest {
            kind: ac2_ui::EXAMINE_PANEL,
            data_enum: panel,
            ..Default::default()
        }));
        if rig.until(10.0, |r| r.view.skill_node_center(HEAL).is_some()) {
            node = rig.view.skill_node_center(HEAL);
            eprintln!("heal is on root panel {root:#x}");
            break;
        }
    }
    let from = node.expect("the heal skill's node on a skill page");
    let bar = rig.client.shortcut_array().unwrap();
    let slot = bar
        .iter()
        .position(|s| matches!(s, ShortcutInfo::Empty(_)))
        .expect("an empty slot");
    let to = rig.view.shortcut_slot_center(slot).unwrap();
    eprintln!("node {from:?} -> slot {slot} at {to:?}");
    rig.point(from);
    rig.run(1.0);
    rig.button(true);
    for step in 1..=8 {
        let t = step as f32 / 8.0;
        let x = from.0 as f32 + (to.0 - from.0) as f32 * t;
        let y = from.1 as f32 + (to.1 - from.1) as f32 * t;
        rig.point((x.round() as i32, y.round() as i32));
    }
    let dragging = rig.view.is_dragging();
    rig.button(false);
    rig.run(1.0);
    eprintln!(
        "dragging {dragging}; slot now {:?}",
        rig.client.shortcut_array().unwrap()[slot]
    );
    assert!(dragging, "the skill follows the pointer");
    assert_eq!(
        rig.client.shortcut_array().unwrap()[slot],
        ShortcutInfo::Enum {
            kind: SKILL_SHORTCUT,
            value: HEAL
        }
    );
}
