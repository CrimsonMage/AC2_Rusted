//! Skills and crafting against a running AC2e server (see `tests/live/mod.rs`): the skill panel
//! lists the player's skills top down, and training, raising and untraining a skill from the
//! panel reach the server and come back as the server's rules set them.
//! `scripts/ac2e-live-test.sh` starts the server with `AC2E_DEV_START_XP` (a new character starts
//! with that experience, which levels it and gives it credits and experience to spend).

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_net::protocol::{ObjectAction, ScriptMessage, ShortcutInfo};
use ac2_ui::ToolbarChild;
use ac2_weenie::{SessionEvent, SkillRequest};
use live::LiveRig;

/// A melee skill of level 2 that costs one credit and needs the starter skill 0xB0.
const SKILL: u32 = 0xB2;
const ROOT: u32 = 0xB0;

/// The rig, when the server was started with the skill tests' settings (`scripts/ac2e-live-test.sh`
/// with `AC2E_LIVE_TEST=ac2e_live_skills`: experience and lost health to start with, the
/// development kit's skill rules).
fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("AC2E_DEV_START_XP").is_none() {
        eprintln!("AC2E_DEV_START_XP is not set: run through scripts/ac2e-live-test.sh with AC2E_LIVE_TEST=ac2e_live_skills");
        return None;
    }
    LiveRig::new(name)
}

/// Frames until `ready` holds (it may ask the script queries, which need `&mut`), at most 60 s.
fn wait(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}");
}

/// The player's script process answers (the skill queries need it), and experience is there.
fn ready(rig: &mut LiveRig) {
    wait(rig, "the player's script", |r| {
        r.client.skill_credits().is_ok()
    });
    for _ in 0..60 * 60 {
        let progress = rig.client.frame().progress;
        if progress.experience.available_xp > 0 && !progress.skills.skills.is_empty() {
            return;
        }
        rig.frame(&[]);
    }
    panic!(
        "no experience or skills: {:?} {:?}",
        rig.client.frame().progress,
        rig.client.take_script_problems()
    );
}

/// A click on the centre of the shown HUD element `name`, once it shows.
fn press(rig: &mut LiveRig, name: &str) {
    let mut center = None;
    wait(rig, name, |r| {
        center = r.view.ui_element_center(name);
        center.is_some()
    });
    rig.click(center.unwrap());
}

/// Opens the melee page and selects `SKILL`'s node.
fn select_skill(rig: &mut LiveRig) {
    rig.panel(ToolbarChild::Skill, true);
    assert!(
        rig.until(20.0, |r| r.view.skill_node_center(SKILL).is_some()),
        "no node for the skill"
    );
    let at = rig.view.skill_node_center(SKILL).unwrap();
    rig.click(at);
}

/// Presses a skill panel button, then answers the confirmation if it asks.
fn press_and_confirm(rig: &mut LiveRig, button: &str) {
    press(rig, button);
    rig.run(0.5);
    if let Some(yes) = rig.view.ui_element_center("YesButton") {
        rig.click(yes);
    }
}

#[test]
fn the_skill_panel_lists_the_players_skills_top_down() {
    let Some(mut rig) = rig_for("Skilllist") else {
        return;
    };
    ready(&mut rig);
    rig.panel(ToolbarChild::Skill, true);
    // Shown with nothing loaded, the modern panel opens on the melee page by itself.
    assert!(
        rig.until(20.0, |r| !r.view.skill_nodes().is_empty()),
        "the skill panel lists no skills"
    );
    // Top down: a skill sits below the skill it needs.
    let (_, root_y) = rig.view.skill_node_center(ROOT).expect("a root skill");
    let (_, child_y) = rig
        .view
        .skill_node_center(SKILL)
        .expect("a skill that needs the root");
    assert!(root_y < child_y, "{root_y} {child_y}");
}

#[test]
fn a_skill_is_trained_raised_and_untrained_from_the_panel() {
    let Some(mut rig) = rig_for("Skilltrain") else {
        return;
    };
    ready(&mut rig);
    let credits = rig.client.skill_credits().unwrap();
    let available = rig.client.frame().progress.experience.available_xp;
    assert!(
        credits > 0,
        "no credits to train with: {:?}",
        rig.client.frame().progress
    );
    assert_eq!(rig.client.is_skill_trained(SKILL), Ok(false));
    select_skill(&mut rig);

    // Train: one credit.
    press_and_confirm(&mut rig, "TrainButton");
    wait(&mut rig, "the skill trained", |r| {
        r.client.is_skill_trained(SKILL) == Ok(true)
    });
    rig.run(0.5);
    assert_eq!(rig.client.skill_credits(), Ok(credits - 1));

    // Raise: experience moves from the available pool into the skill.
    press_and_confirm(&mut rig, "RaiseButton");
    wait(&mut rig, "the skill raised", |r| {
        r.client.skill_xp(SKILL).is_ok_and(|x| x > 0)
    });
    let spent = rig.client.skill_xp(SKILL).unwrap();
    rig.run(0.5);
    assert_eq!(
        rig.client.frame().progress.experience.available_xp,
        available - spent
    );

    // Untrain: a raised skill is untrained over time (its experience paid back), not at once.
    press_and_confirm(&mut rig, "UntrainButton");
    wait(&mut rig, "the skill being untrained", |r| {
        r.client.is_skill_being_untrained(SKILL) == Ok(true)
    });
    // Cancelling keeps the skill.
    press_and_confirm(&mut rig, "CancelUntrainButton");
    wait(&mut rig, "the untraining cancelled", |r| {
        r.client.is_skill_being_untrained(SKILL) == Ok(false)
    });
    assert_eq!(rig.client.is_skill_trained(SKILL), Ok(true));
}

#[test]
fn an_unraised_skill_is_untrained_at_once_and_its_credit_comes_back() {
    let Some(mut rig) = rig_for("Skilluntrain") else {
        return;
    };
    ready(&mut rig);
    let credits = rig.client.skill_credits().unwrap();
    select_skill(&mut rig);
    press_and_confirm(&mut rig, "TrainButton");
    wait(&mut rig, "the skill trained", |r| {
        r.client.is_skill_trained(SKILL) == Ok(true)
    });
    press_and_confirm(&mut rig, "UntrainButton");
    wait(&mut rig, "the skill untrained", |r| {
        r.client.is_skill_trained(SKILL) == Ok(false)
    });
    rig.run(0.5);
    assert_eq!(rig.client.skill_credits(), Ok(credits));
}

/// Ease Suffering: a healing skill (level 3, after skill 0x59) that costs vigor.
const HEAL: u32 = 0x55;
const HEAL_PREREQUISITE: u32 = 0x59;
/// `ShortcutInfo::_type` of a skill.
const SKILL_SHORTCUT: u32 = 0x4000_0001;

#[test]
fn a_skill_is_used_and_its_vigor_is_spent_and_comes_back() {
    let Some(mut rig) = rig_for("Skilluse") else {
        return;
    };
    if std::env::var_os("AC2E_LIVE_TRAINING_KIT").is_none() {
        eprintln!("the server has no skill rules (--training-kit true); skipped");
        return;
    }
    ready(&mut rig);
    // The server regenerates health at rest too: the hurt character heals a little by itself.
    for skill in [HEAL_PREREQUISITE, HEAL] {
        rig.command(Command::Skill(SkillRequest::Train(skill)));
        wait(&mut rig, "the skill trained", |r| {
            r.client.is_skill_trained(skill) == Ok(true)
        });
    }
    // Put the skill on the shortcut bar and use it from there, as the player does.
    // The server's own bar holds the basic attacks; take the first slot it leaves empty.
    let bar = rig.client.shortcut_array().unwrap();
    let slot = bar
        .iter()
        .position(|s| matches!(s, ShortcutInfo::Empty(_)))
        .expect("an empty shortcut slot") as u32;
    rig.client
        .set_shortcut(
            slot,
            ShortcutInfo::Enum {
                kind: SKILL_SHORTCUT,
                value: HEAL,
            },
        )
        .unwrap();
    wait(&mut rig, "the shortcut", |r| {
        r.client
            .shortcut_array()
            .is_ok_and(|bar| matches!(bar[slot as usize], ShortcutInfo::Enum { value: HEAL, .. }))
    });
    let vitals = |rig: &LiveRig| rig.client.frame().player_vitals;
    let before = vitals(&rig);
    let (health, vigor) = (before.health.unwrap(), before.vigor.unwrap());
    assert!(health.current < health.max, "the character starts hurt");
    assert_eq!(
        vigor.current, vigor.max,
        "the character starts with its vigor"
    );

    assert_eq!(rig.client.use_shortcut(slot), Ok(1));
    wait(&mut rig, "the vigor spent", |r| {
        r.client
            .frame()
            .player_vitals
            .vigor
            .is_some_and(|v| v.current < vigor.max)
    });
    let after = vitals(&rig);
    let spent = vigor.current - after.vigor.unwrap().current;
    assert_eq!(spent, 10, "the skill's vigor cost");
    // The skill healed.
    wait(&mut rig, "the health healed", |r| {
        r.client
            .frame()
            .player_vitals
            .health
            .is_some_and(|h| h.current > health.current)
    });

    // The server gives vigor back only to a player nothing fights (AC2e's regeneration skips a
    // player a creature targets). The start in the Menhir Vault has creatures that come and hit
    // the player, so the test kills each one that attacks (as the player would) until the vigor
    // rises above where the skill left it; it fails when the vigor never comes back.
    let low = vitals(&rig).vigor.unwrap().current;
    let mut fought = Vec::new();
    let mut back = false;
    for _ in 0..8 {
        back = rig.until(20.0, |r| {
            r.client
                .frame()
                .player_vitals
                .vigor
                .is_some_and(|v| v.current > low)
        });
        if back {
            break;
        }
        // Not yet: a creature is still on the player.
        if let Some(target) = attacker(&rig, &fought) {
            fought.push(target);
            kill(&mut rig, target);
        }
    }
    let vigor = vitals(&rig).vigor.unwrap();
    eprintln!(
        "vigor {} -> {}/{} after {} creatures were fought",
        low,
        vigor.current,
        vigor.max,
        fought.len()
    );
    assert!(
        back,
        "the vigor never came back: {}/{} after {} creatures were fought",
        vigor.current,
        vigor.max,
        fought.len()
    );
}

/// A creature that has attacked the player and has not been fought yet (the monsters beside the
/// start come to it).
fn attacker(rig: &LiveRig, fought: &[u64]) -> Option<u64> {
    rig.seen.iter().find_map(|e| match e {
        SessionEvent::ObjectAction {
            object,
            action: ObjectAction::DoBehavior(_),
        } if object >> 48 == 0x8140 && !fought.contains(object) => Some(*object),
        _ => None,
    })
}

/// Whether a chat line containing `text` arrived since event `from`.
fn said(rig: &LiveRig, from: usize, text: &str) -> bool {
    rig.seen[from..].iter().any(|e| match e {
        SessionEvent::Interp {
            function,
            arguments,
            ..
        } => matches!(
            ScriptMessage::decode(*function, arguments),
            Some(Ok(ScriptMessage::DisplayStringInfo { text: t, .. }))
                if t.literal.as_deref().is_some_and(|l| l.contains(text))
        ),
        _ => false,
    })
}

/// Selects `target`, attacks until the server says it is defeated, and stops attacking.
fn kill(rig: &mut LiveRig, target: u64) {
    let before = rig.seen.len();
    rig.command(Command::Select(Some(target)));
    rig.run(0.5);
    rig.command(Command::SetAttacking(true));
    assert!(
        rig.until(90.0, |r| said(r, before, "defeated")),
        "the creature dies"
    );
    rig.command(Command::SetAttacking(false));
    rig.run(0.5);
}

/// The development kit's craft skill and its two recipes (health and vigor potions), the items it
/// takes (a Mucor Mushroom, a bundle of Tallow) and the vigor potion it gives.
const CRAFT_SKILL: u32 = 0x6F00_28D7;
const HEALTH_RECIPE: u32 = 0x6F00_1F17;
const VIGOR_RECIPE: u32 = 0x6F00_1F18;

/// The server's craft registry (a craft skill and the health and vigor recipes) reaches the
/// player's script and its queries. Not covered: the craft panel's rows and executing a craft.
#[test]
fn the_players_craft_skill_and_recipes_reach_the_client() {
    let Some(mut rig) = rig_for("Skillcraft") else {
        return;
    };
    if std::env::var_os("AC2E_LIVE_TRAINING_KIT").is_none() {
        eprintln!("the server has no craft recipes (--training-kit true); skipped");
        return;
    }
    ready(&mut rig);
    // The registry the server sends (its recipe records read from the DATs on first ask).
    wait(&mut rig, "the craft recipes", |r| {
        r.client
            .recipes(CRAFT_SKILL)
            .is_some_and(|recipes| recipes.len() == 2)
    });
    let recipes = rig.client.recipes(CRAFT_SKILL).unwrap();
    assert!(recipes.iter().any(|r| r.did == HEALTH_RECIPE));
    assert!(recipes.iter().any(|r| r.did == VIGOR_RECIPE));
}

/// Opening the skill panel (its first page asks for the melee panel and opens the examination
/// panel) must not stop an object's examination from showing its profile afterwards.
#[test]
fn examining_an_object_after_opening_the_skill_panel_shows_its_profile() {
    let Some(mut rig) = rig_for("Skillexamine") else {
        return;
    };
    ready(&mut rig);
    rig.panel(ToolbarChild::Skill, true);
    assert!(
        rig.until(20.0, |r| !r.view.skill_nodes().is_empty()),
        "the skill panel lists no skills"
    );
    let me = rig.client.session().player().unwrap();
    let (target, name) = rig
        .client
        .session()
        .objects()
        .iter()
        .find(|o| o.id != me && o.name.is_some())
        .map(|o| (o.id, o.name.as_ref().unwrap().to_string()))
        .expect("a named object near the start");
    rig.command(Command::Select(Some(target)));
    rig.command(Command::Examine(
        ac2_net::protocol::ExaminationRequest::object(target),
    ));
    rig.view.open_examination();
    let shown = rig.until(20.0, |r| r.view.examination_texts().contains(&name));
    assert!(
        shown,
        "the examination shows no profile: {:?}",
        rig.view.examination_texts()
    );
}
