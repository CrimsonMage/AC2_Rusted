//! Buffs and debuffs against a running Pyrealium server (see `tests/live/mod.rs`): a skill that puts a
//! timed effect on the player adds it to the status bar's effect strip (`gmEffectsUI`) with its
//! timer running down; when the time is up the server takes it off (`WM_Effect` 0x16) and the
//! strip empties; a recast replaces the effect and the strip restarts its timer; a creature's
//! effects are listed in its examination; death clears the player's effects.
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_effects scripts/pyrealium-live-test.sh`: that starts the server
//! with the development kit, experience to train the buff chains with, and
//! `PYREALIUM_DEV_EFFECT_SCALE` (every timed effect lasts that fraction of its record's duration, so
//! the tests watch an effect run out in seconds).

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_net::protocol::{ExaminationRequest, ObjectAction, ScriptMessage, ShortcutInfo};
use ac2_ui::{tokenize, AttributeValue, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent, SkillRequest};
use live::LiveRig;

/// `ShortcutInfo::_type` of a skill, and the right hand.
const SKILL_SHORTCUT: u32 = 0x4000_0001;
const PRIMARY_HAND: u32 = 0x40;

/// The vigor regeneration buff and what it needs (`skills.tsv`: 0x67 after 0x64 after 0x55 after
/// 0x59): self, 180 s in its record, 5 s to recast.
const FLAME_BOLT: u32 = 0x59;
const VIGOR_BUFF: u32 = 0x67;
const VIGOR_BUFF_CHAIN: [u32; 4] = [0x59, 0x55, 0x64, 0x67];

/// The rig, when the server was started with the effects settings.
fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("PYREALIUM_DEV_EFFECT_SCALE").is_none() {
        eprintln!("PYREALIUM_DEV_EFFECT_SCALE is not set: run through scripts/pyrealium-live-test.sh with PYREALIUM_LIVE_TEST=pyrealium_live_effects");
        return None;
    }
    LiveRig::new(name)
}

/// The fraction of a record's duration the server runs effects at.
fn scale() -> f64 {
    std::env::var("PYREALIUM_DEV_EFFECT_SCALE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1.0)
}

/// The server's chat lines so far (its `DisplayStringInfo` literals).
fn lines(rig: &LiveRig) -> Vec<String> {
    rig.seen
        .iter()
        .filter_map(|e| match e {
            SessionEvent::Interp {
                function,
                arguments,
                ..
            } => match ScriptMessage::decode(*function, arguments) {
                Some(Ok(ScriptMessage::DisplayStringInfo { text, .. })) => text.literal,
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Frames until `ready` holds (it may ask the script queries, which need `&mut`), at most 60 s.
fn wait(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}; the server said: {:?}", lines(rig));
}

/// The player's script answers and experience is there.
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
    panic!("no experience or skills");
}

/// Trains `chain` in order.
fn train(rig: &mut LiveRig, chain: &[u32]) {
    for &skill in chain {
        rig.command(Command::Skill(SkillRequest::Train(skill)));
        wait(rig, "the skill trained", |r| {
            r.client.is_skill_trained(skill) == Ok(true)
        });
    }
}

/// Raises a trained skill with experience (the server refuses a raise soon after other
/// advancement, so it is asked again).
fn raise(rig: &mut LiveRig, skill: u32) {
    for _ in 0..10 {
        rig.run(0.5);
        rig.command(Command::Skill(SkillRequest::Raise(skill)));
        if rig.until(3.0, |r| r.client.frame().progress.skills.xp(skill) > 0) {
            return;
        }
    }
    panic!("the skill never raised");
}

/// Puts `skill` on the first empty shortcut slot and wields the starter weapon (the skills want
/// an item in the right hand); returns the slot.
fn arm(rig: &mut LiveRig, skill: u32) -> u32 {
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
                value: skill,
            },
        )
        .unwrap();
    wait(rig, "the shortcut", |r| {
        r.client.shortcut_array().is_ok_and(
            |bar| matches!(bar[slot as usize], ShortcutInfo::Enum { value, .. } if value == skill),
        )
    });
    if !rig
        .client
        .frame()
        .inventory
        .equipped
        .iter()
        .any(|&(location, _)| location == PRIMARY_HAND)
    {
        let weapon = rig
            .client
            .frame()
            .inventory
            .items
            .iter()
            .find(|i| i.stats.preferred_location & PRIMARY_HAND != 0)
            .map(|i| i.id)
            .expect("a weapon in the starter gear");
        rig.command(Command::EquipItem {
            item: weapon,
            location: PRIMARY_HAND,
        });
        wait(rig, "the weapon wielded", |r| {
            r.client
                .frame()
                .inventory
                .equipped
                .iter()
                .any(|&(location, _)| location == PRIMARY_HAND)
        });
    }
    slot
}

/// One effect the status bar's strip shows: its id, tooltip, the element's state and the
/// position (1 full, 0 empty) of its time meter.
#[derive(Debug, Clone, PartialEq)]
struct Shown {
    eid: u32,
    tooltip: Option<String>,
    state: u32,
    meter: Option<f32>,
}

fn float(tree: &UiTree, element: usize, name: &str) -> Option<f32> {
    match tree.get(element)?.attribute(tokenize(name)) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

/// The effects `gmEffectsUI` shows on the HUD, in strip order.
fn strip(rig: &LiveRig) -> Vec<Shown> {
    let Some(tree) = rig.view.ui_tree() else {
        return Vec::new();
    };
    let Some(list) = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("EffectListBox")))
    else {
        return Vec::new();
    };
    tree.list_items(list)
        .iter()
        .filter_map(|&item| {
            let eid = match tree.get(item)?.attribute(tokenize("EffectID")) {
                Some(AttributeValue::Id(eid)) => *eid,
                _ => return None,
            };
            let meter = tree
                .child_recursive(item, tokenize("EffectTimeMeter"))
                .and_then(|m| float(tree, m, "position"));
            Some(Shown {
                eid,
                tooltip: tree.tooltip_text(item),
                state: tree.get(item)?.state(),
                meter,
            })
        })
        .collect()
}

/// Uses the shortcut and waits until the player's list holds `count` effects.
fn cast(rig: &mut LiveRig, slot: u32, count: usize) {
    assert_eq!(rig.client.use_shortcut(slot), Ok(1));
    wait(rig, "the effect on the player", |r| {
        r.client.effect_list().is_ok_and(|l| l.len() == count)
    });
}

#[test]
fn a_self_buff_joins_the_strip_counts_down_and_expires() {
    let Some(mut rig) = rig_for("Effectbuff") else {
        return;
    };
    if std::env::var_os("PYREALIUM_LIVE_TRAINING_KIT").is_none() {
        eprintln!("the server has no skill rules (--training-kit true); skipped");
        return;
    }
    ready(&mut rig);
    train(&mut rig, &VIGOR_BUFF_CHAIN);
    let slot = arm(&mut rig, VIGOR_BUFF);
    assert_eq!(rig.client.effect_list(), Ok(Vec::new()));
    assert!(strip(&rig).is_empty());

    cast(&mut rig, slot, 1);
    let node = rig.client.effect_list().unwrap().remove(0);
    let now = rig.client.frame().server_time;
    eprintln!("effect {node:?} at {now}");
    // The record's 180 s at the server's development scale; cast just now.
    let want = 180.0 * scale();
    assert!(
        (node.duration - want).abs() < 0.5,
        "{} {want}",
        node.duration
    );
    assert!(node.time_cast <= now && now - node.time_cast < 5.0);
    assert!(!node.name.is_empty() && node.icon != 0, "{node:?}");
    assert_eq!(node.flags, 1, "enabled, not pulsed");
    assert!(!node.is_pulsed());

    // The strip shows it with its name, running, its meter counting down.
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    let first = strip(&rig).remove(0);
    assert_eq!(first.eid, node.eid);
    assert_eq!(first.tooltip.as_deref(), Some(node.name.as_str()));
    assert_eq!(first.state, tokenize("active"));
    rig.run(3.0);
    let later = strip(&rig).remove(0);
    eprintln!("meter {:?} -> {:?}", first.meter, later.meter);
    assert!(
        later.meter.unwrap() < first.meter.unwrap(),
        "the meter runs down"
    );

    // It runs out: the player's list and the strip empty.
    wait(&mut rig, "the effect expired", |r| {
        r.client.effect_list().is_ok_and(|l| l.is_empty())
    });
    wait(&mut rig, "the strip emptied", |r| strip(r).is_empty());
    eprintln!("chat {:?}", lines(&rig));
}

/// The seconds a countdown row of the examination panel shows (`m:ss`).
fn countdown(rig: &LiveRig) -> Option<u32> {
    rig.view.examination_texts().iter().find_map(|t| {
        let (m, s) = t.split_once(':')?;
        Some(m.parse::<u32>().ok()? * 60 + s.parse::<u32>().ok()?)
    })
}

/// A press on an effect of the strip examines it (`gmEffectsUI::ListenToMessage`): the panel
/// shows its name and description, and its end time as a countdown that runs down.
#[test]
fn pressing_an_effect_shows_its_description_and_the_time_left_counting_down() {
    let Some(mut rig) = rig_for("Effectexam") else {
        return;
    };
    ready(&mut rig);
    train(&mut rig, &VIGOR_BUFF_CHAIN);
    let slot = arm(&mut rig, VIGOR_BUFF);
    // Raised, the skill has a level; the effect carries it as its spellcraft.
    raise(&mut rig, VIGOR_BUFF);
    let level = rig.client.skill_level(VIGOR_BUFF).unwrap().raw;
    cast(&mut rig, slot, 1);
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    let node = rig.client.effect_list().unwrap().remove(0);
    let at = rig
        .view
        .ui_element_center("EffectTemplate")
        .expect("the effect's icon");
    rig.click(at);
    wait(&mut rig, "the examination", |r| {
        r.view.examination_texts().contains(&node.desc)
    });
    let texts = rig.view.examination_texts();
    eprintln!("examination {texts:?}");
    assert!(texts.iter().any(|t| t.starts_with(&node.name)), "{texts:?}");
    eprintln!("node {node:?} now {}", rig.client.frame().server_time);
    // The spellcraft row: the label, then the skill's level the effect was cast at.
    let at = texts
        .iter()
        .position(|t| t.starts_with("Spellcraft"))
        .expect("a spellcraft row");
    assert!(
        texts.iter().any(|t| *t == level.to_string()) && level > 0,
        "{level} not in {texts:?} (spellcraft at {at})"
    );
    let first = countdown(&rig).expect("a countdown row");
    let total = (180.0 * scale()) as u32;
    assert!((1..=total).contains(&first), "{first} of {total}");
    // The row counts down on the global loop (`TimeDisplay::ListenToMessage`).
    rig.run(4.0);
    let later = countdown(&rig).expect("a countdown row");
    eprintln!("countdown {first} -> {later}");
    assert!(
        later + 2 <= first,
        "the countdown runs down: {first} -> {later}"
    );
}

/// Seconds of the server's clock since `since`.
fn since(rig: &LiveRig, since: f64) -> f64 {
    rig.client.frame().server_time - since
}

/// A recast of an effect that is still on replaces its record (a new effect id, cast again), and
/// the strip restarts its timer.
#[test]
fn recasting_an_active_buff_replaces_it_and_restarts_the_timer() {
    let Some(mut rig) = rig_for("Effectagain") else {
        return;
    };
    ready(&mut rig);
    train(&mut rig, &VIGOR_BUFF_CHAIN);
    let slot = arm(&mut rig, VIGOR_BUFF);
    cast(&mut rig, slot, 1);
    let first = rig.client.effect_list().unwrap().remove(0);
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    // The skill's own recovery time (5 s in its record) passes, the effect still on.
    wait(&mut rig, "the skill ready again", |r| {
        since(r, first.time_cast) > 6.0
    });
    let before = strip(&rig).remove(0);
    assert_eq!(before.eid, first.eid);
    assert!(before.meter.unwrap() < 0.75, "{before:?}");
    assert_eq!(rig.client.use_shortcut(slot), Ok(1));
    wait(&mut rig, "the effect replaced", |r| {
        r.client
            .effect_list()
            .is_ok_and(|l| l.len() == 1 && l[0].eid != first.eid)
    });
    let second = rig.client.effect_list().unwrap().remove(0);
    assert!(
        second.time_cast > first.time_cast + 5.0,
        "{first:?} {second:?}"
    );
    assert_eq!(second.duration, first.duration);
    wait(&mut rig, "the strip shows the new record", |r| {
        strip(r).iter().map(|s| s.eid).eq([second.eid])
    });
    let after = strip(&rig).remove(0);
    eprintln!("meter {:?} -> {:?}", before.meter, after.meter);
    assert!(after.meter.unwrap() > 0.9, "the timer restarted: {after:?}");
    // It runs out once, as one effect.
    wait(&mut rig, "the effect expired", |r| {
        r.client.effect_list().is_ok_and(|l| l.is_empty())
    });
    assert!(since(&rig, second.time_cast) >= second.duration - 0.5);
    wait(&mut rig, "the strip emptied", |r| strip(r).is_empty());
}

/// The defence buff (Defensive Stance's chain) beside the vigor buff.
const DEFENCE_BUFF: u32 = 0xA8;
const DEFENCE_BUFF_CHAIN: [u32; 4] = [0xB2, 0xA1, 0xA4, 0xA8];

/// Effects of different skills stack as separate records, listed by the time they run out, and
/// each leaves when its own time is up.
#[test]
fn buffs_of_different_skills_stack_and_expire_one_by_one() {
    let Some(mut rig) = rig_for("Effectstack") else {
        return;
    };
    ready(&mut rig);
    train(&mut rig, &VIGOR_BUFF_CHAIN);
    train(&mut rig, &DEFENCE_BUFF_CHAIN);
    let vigor_slot = arm(&mut rig, VIGOR_BUFF);
    let defence_slot = arm(&mut rig, DEFENCE_BUFF);
    cast(&mut rig, vigor_slot, 1);
    // The global recovery between two skills (1 s).
    rig.run(1.5);
    cast(&mut rig, defence_slot, 2);
    let list = rig.client.effect_list().unwrap();
    let vigor = list.iter().find(|n| n.name == "Tenaciousness").unwrap();
    let defence = list.iter().find(|n| n.name != "Tenaciousness").unwrap();
    eprintln!("effects {list:?}");
    assert_ne!(vigor.eid, defence.eid);
    assert!((vigor.duration - 180.0 * scale()).abs() < 0.5);
    assert!((defence.duration - 120.0 * scale()).abs() < 0.5);
    // The strip lists them by the time they run out: the defence buff (12 s) first.
    wait(&mut rig, "the strip", |r| strip(r).len() == 2);
    let eids: Vec<u32> = strip(&rig).iter().map(|s| s.eid).collect();
    assert_eq!(eids, [defence.eid, vigor.eid]);
    // The shorter one runs out first and leaves the longer one.
    wait(&mut rig, "the defence buff expired", |r| {
        r.client.effect_list().is_ok_and(|l| l.len() == 1)
    });
    assert_eq!(rig.client.effect_list().unwrap()[0].eid, vigor.eid);
    wait(&mut rig, "the strip follows", |r| {
        strip(r).iter().map(|s| s.eid).eq([vigor.eid])
    });
    assert!(since(&rig, defence.time_cast) >= defence.duration - 0.5);
    wait(&mut rig, "the vigor buff expired", |r| {
        r.client.effect_list().is_ok_and(|l| l.is_empty())
    });
    assert!(since(&rig, vigor.time_cast) >= vigor.duration - 0.5);
    wait(&mut rig, "the strip emptied", |r| strip(r).is_empty());
}

/// The self heal over time (the chain 0x94, 0x92, 0x9D, then 0x91, level 30): 1 health every
/// second for 180 s in its record at rank 1.
const HEAL_OVER_TIME: u32 = 0x91;
const HEAL_OVER_TIME_CHAIN: [u32; 4] = [0x94, 0x92, 0x9D, 0x91];

/// How many green `+N` damage texts (`Do3DDamage`) reached the client since event
/// `from` of `rig.seen`.
fn heal_texts(rig: &LiveRig, from: usize) -> usize {
    rig.seen[from..]
        .iter()
        .filter(|e| {
            matches!(e, SessionEvent::Notice(GameplayNotice::Do3DDamage { text, .. })
                if text.text.starts_with('+'))
        })
        .count()
}

/// A heal over time ticks at its period, each tick raising the health and showing its green `+1`
/// (`Do3DDamage`), and stops when the effect leaves the strip.
#[test]
fn a_heal_over_time_ticks_every_second_while_its_effect_lasts() {
    let Some(mut rig) = rig_for("Effecthot") else {
        return;
    };
    ready(&mut rig);
    if rig.client.frame().progress.experience.level < 30 {
        eprintln!("the character is under level 30 (PYREALIUM_DEV_START_XP); skipped");
        return;
    }
    train(&mut rig, &HEAL_OVER_TIME_CHAIN);
    let slot = arm(&mut rig, HEAL_OVER_TIME);
    let hurt = rig.client.frame().player_vitals.health.unwrap();
    assert!(hurt.current < hurt.max, "the character starts hurt");
    let from = rig.seen.len();
    assert_eq!(rig.client.use_shortcut(slot), Ok(1));
    wait(&mut rig, "the effect", |r| {
        r.client.effect_list().is_ok_and(|l| l.len() == 1)
    });
    let node = rig.client.effect_list().unwrap().remove(0);
    assert!((node.duration - 180.0 * scale()).abs() < 0.5, "{node:?}");
    // A health change over time is a pulsed effect: the strip shows it in its pulsed state.
    assert!(node.is_pulsed(), "{node:?}");
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    assert_eq!(strip(&rig)[0].state, tokenize("pulsed"));
    // The ticks, with the server time each reached the client at.
    let mut ticks: Vec<f64> = Vec::new();
    let mut counted = heal_texts(&rig, from);
    let mut left = false;
    for _ in 0..60 * 60 {
        rig.frame(&[]);
        let total = heal_texts(&rig, from);
        for _ in counted..total {
            ticks.push(rig.client.frame().server_time);
        }
        counted = total;
        if rig.client.effect_list().is_ok_and(|l| l.is_empty()) {
            left = true;
            break;
        }
    }
    assert!(left, "the effect never ran out");
    let gone = rig.client.frame().server_time;
    eprintln!("ticks {ticks:.2?} cast {} gone {gone}", node.time_cast);
    // One tick a second for the effect's time.
    let expected = node.duration as usize;
    assert!(
        (expected - 2..=expected + 1).contains(&ticks.len()),
        "{} ticks for {expected} s",
        ticks.len()
    );
    for pair in ticks.windows(2) {
        assert!((0.8..1.3).contains(&(pair[1] - pair[0])), "{ticks:?}");
    }
    assert!(ticks[0] - node.time_cast < 2.0 && gone - ticks[ticks.len() - 1] < 2.0);
    // The ticks stop with the effect.
    rig.run(3.0);
    assert_eq!(
        heal_texts(&rig, from),
        counted,
        "a tick after the effect left"
    );
    // The health rose by the ticks (creatures' blows aside, one point each).
    let healed = rig.client.frame().player_vitals.health.unwrap();
    assert!(healed.current > hurt.current, "{hurt:?} -> {healed:?}");
}

/// The first creature that has attacked the player (the monsters beside the start come to it).
fn attacker(rig: &mut LiveRig) -> u64 {
    loop {
        let found = rig.seen.iter().find_map(|e| match e {
            SessionEvent::ObjectAction {
                object,
                action: ObjectAction::DoBehavior(_),
            } if object >> 48 == 0x8140 => Some(*object),
            _ => None,
        });
        if let Some(creature) = found {
            return creature;
        }
        rig.frame(&[]);
    }
}

/// Examines `target` afresh (the same request is not asked twice, so the panel is cleared first)
/// and returns the panel's lines once the new profile shows.
fn examine(rig: &mut LiveRig, target: u64) -> Vec<String> {
    rig.command(Command::Examine(ExaminationRequest::default()));
    rig.run(0.3);
    rig.command(Command::Examine(ExaminationRequest::object(target)));
    rig.view.open_examination();
    // The profile the weenie held shows at once; the server's follows.
    rig.run(1.0);
    wait(rig, "the examination", |r| {
        r.view
            .examination_texts()
            .iter()
            .any(|t| t.starts_with("Health:"))
    });
    rig.view.examination_texts()
}

/// The effect lines of an examination (Pyrealium's own text for a creature's effects).
fn effect_lines(texts: &[String]) -> Vec<String> {
    texts
        .iter()
        .filter(|t| t.contains(" s left"))
        .cloned()
        .collect()
}

/// Flame Bolt (0x59, the first skill of the magic tree; a stave in the right hand) hits a creature
/// and burns it: the burn is one of the creature's effects, listed in its examination with the
/// time left, and it leaves when its time is up.
#[test]
fn a_burn_on_a_creature_is_listed_in_its_examination_and_runs_out() {
    let Some(mut rig) = rig_for("Effectburn") else {
        return;
    };
    ready(&mut rig);
    train(&mut rig, &[FLAME_BOLT]);
    let slot = arm(&mut rig, FLAME_BOLT);
    let target = attacker(&mut rig);
    rig.command(Command::Select(Some(target)));
    rig.run(0.5);
    assert!(effect_lines(&examine(&mut rig, target)).is_empty());
    // The starter weapons: the stave is the one the skill accepts.
    let weapons: Vec<u64> = rig
        .client
        .frame()
        .inventory
        .items
        .iter()
        .filter(|i| i.stats.preferred_location & PRIMARY_HAND != 0)
        .map(|i| i.id)
        .collect();
    let mut burning = false;
    for weapon in weapons {
        rig.command(Command::EquipItem {
            item: weapon,
            location: PRIMARY_HAND,
        });
        if !rig.until(10.0, |r| {
            r.client
                .frame()
                .inventory
                .equipped
                .iter()
                .any(|&(location, id)| location == PRIMARY_HAND && id == weapon)
        }) {
            continue;
        }
        let before = lines(&rig).len();
        assert_eq!(rig.client.use_shortcut(slot), Ok(1));
        if rig.until(4.0, |r| {
            lines(r)[before..].iter().any(|l| l.starts_with("You hit"))
        }) {
            burning = true;
            break;
        }
    }
    assert!(burning, "no weapon let Flame Bolt hit");
    let struck = lines(&rig).len();
    // The burn is on the creature (its record's 30 s at the development scale).
    let texts = examine(&mut rig, target);
    eprintln!("examination {texts:?}");
    let on = effect_lines(&texts);
    assert_eq!(on.len(), 1, "{texts:?}");
    assert!(on[0].starts_with("Burning"), "{on:?}");
    // The creature takes the burn's ticks while it lasts.
    wait(&mut rig, "a burn tick", |r| {
        lines(r)[struck..]
            .iter()
            .any(|l| l.starts_with("You hit for 1"))
    });
    // Past its time the burn is off the creature.
    rig.run(30.0 * scale() + 1.5);
    let later = examine(&mut rig, target);
    assert!(effect_lines(&later).is_empty(), "{later:?}");
}

/// Death takes the player's effects off (Pyrealium's policy; the retail server's is not known). The
/// creatures beside the start must be able to kill the character before the effect runs out, so
/// this one runs on a server whose new characters start nearly dead and whose effects last their
/// record's time: `PYREALIUM_DEV_EFFECT_SCALE=1 PYREALIUM_DEV_START_DAMAGE=760
/// PYREALIUM_LIVE_TEST=pyrealium_live_effects scripts/pyrealium-live-test.sh death_takes`; on the ordinary run it
/// is skipped.
#[test]
fn death_takes_the_players_effects_off() {
    let Some(mut rig) = rig_for("Effectdeath") else {
        return;
    };
    ready(&mut rig);
    let health = rig.client.frame().player_vitals.health.unwrap();
    if health.current > 100 {
        eprintln!(
            "the character starts with {health:?}; run with PYREALIUM_DEV_START_DAMAGE=760: skipped"
        );
        return;
    }
    train(&mut rig, &VIGOR_BUFF_CHAIN);
    let slot = arm(&mut rig, VIGOR_BUFF);
    // The effect lasts the 180 s of its record at this run's scale; the creatures kill the player
    // well before.
    cast(&mut rig, slot, 1);
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    let cast_at = rig.client.effect_list().unwrap()[0].time_cast;
    let dead = rig.until(150.0, |r| {
        r.seen.iter().any(|e| {
            matches!(
                e,
                SessionEvent::Notice(GameplayNotice::UpdateDeathState(true))
            )
        })
    });
    assert!(
        dead,
        "the player never died: {:?}",
        rig.client.frame().player_vitals
    );
    assert!(
        since(&rig, cast_at) < 180.0 * scale(),
        "the effect would have run out first"
    );
    wait(&mut rig, "the effect taken off", |r| {
        r.client.effect_list().is_ok_and(|l| l.is_empty())
    });
    wait(&mut rig, "the strip emptied", |r| strip(r).is_empty());
}

/// A buff cast before a log off is on the strip after the player enters again, with the time it
/// had left, and runs out on time: the server adds it again after the login (`WM_Effect` 0x17).
#[test]
fn effects_survive_a_relog_with_their_time_left() {
    let Some(mut rig) = rig_for("Effectrelog") else {
        return;
    };
    ready(&mut rig);
    train(&mut rig, &VIGOR_BUFF_CHAIN);
    let slot = arm(&mut rig, VIGOR_BUFF);
    cast(&mut rig, slot, 1);
    let before = rig.client.effect_list().unwrap().remove(0);
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    rig.run(2.0);
    let shown = strip(&rig).remove(0);
    rig.relog();
    wait(&mut rig, "the effect on the strip again", |r| {
        strip(r).len() == 1
    });
    let after = rig.client.effect_list().unwrap();
    assert_eq!(after.len(), 1, "{after:?}");
    let (kept, again) = (&before, &after[0]);
    eprintln!("before {kept:?} after {again:?}");
    assert_eq!(
        (again.eid, &again.name, again.icon, again.duration),
        (kept.eid, &kept.name, kept.icon, kept.duration)
    );
    assert!(
        (again.time_cast - kept.time_cast).abs() < 0.05,
        "{kept:?} {again:?}"
    );
    // The meter went on from where the time had taken it.
    let back = strip(&rig).remove(0);
    let now = rig.client.frame().server_time;
    let left = (again.time_cast + again.duration - now) / again.duration;
    eprintln!(
        "meter {:?} -> {:?}, time left {left:.3}",
        shown.meter, back.meter
    );
    assert!(back.meter.unwrap() < shown.meter.unwrap());
    assert!(
        (f64::from(back.meter.unwrap()) - left).abs() < 0.15,
        "{back:?} {left}"
    );
    assert_eq!(back.state, tokenize("active"));
    // It runs out at its own time.
    wait(&mut rig, "the effect expired", |r| {
        r.client.effect_list().is_ok_and(|l| l.is_empty())
    });
    assert!((since(&rig, again.time_cast) - again.duration).abs() < 2.0);
    wait(&mut rig, "the strip emptied", |r| strip(r).is_empty());
}

/// The Banderling Thrasher beside the start (`PYREALIUM_LIVE_THRASHER=1`, see the script): its entity
/// record gives it skill 0x74b at level 43, a health loss over time on a player for 10 s.
const THRASHER_SKILL_LEVEL: u32 = 43;

/// A creature's harmful skill lands on the player as the data defines it: the effect is on the
/// strip with its time running down, the health falls by the same amount every second for its
/// 10 s, the effect's examination names its level, and it leaves when the time is up.
#[test]
fn a_creature_ability_puts_a_damage_over_time_on_the_player() {
    if std::env::var_os("PYREALIUM_LIVE_THRASHER").is_none() {
        eprintln!("no Thrasher beside the start (PYREALIUM_LIVE_THRASHER=1, see the script); skipped");
        return;
    }
    let Some(mut rig) = rig_for("Effectdebuff") else {
        return;
    };
    ready(&mut rig);
    let from = rig.seen.len();
    wait(&mut rig, "the creature's effect on the player", |r| {
        r.client.effect_list().is_ok_and(|l| !l.is_empty())
    });
    let node = rig.client.effect_list().unwrap().remove(0);
    eprintln!("debuff {node:?}");
    assert!((node.duration - 10.0).abs() < 0.5, "{node:?}");
    assert!(!node.name.is_empty() && node.icon != 0);
    wait(&mut rig, "the strip", |r| strip(r).len() == 1);
    let shown = strip(&rig).remove(0);
    assert_eq!(shown.eid, node.eid);
    assert_eq!(shown.tooltip.as_deref(), Some(node.name.as_str()));
    assert!(node.is_pulsed());
    assert_eq!(shown.state, tokenize("pulsed"));
    // Pressing it examines it: the name, the end time counting down, the creature's skill level
    // (the spellcraft the effect was cast at).
    let at = rig
        .view
        .ui_element_center("EffectTemplate")
        .expect("the icon");
    rig.click(at);
    wait(&mut rig, "the examination", |r| {
        r.view
            .examination_texts()
            .iter()
            .any(|t| t.starts_with(&node.name))
    });
    let texts = rig.view.examination_texts();
    eprintln!("examination {texts:?}");
    assert!(
        texts.contains(&THRASHER_SKILL_LEVEL.to_string()),
        "{texts:?}"
    );
    // A pulsed effect gets no countdown row from the script.
    assert_eq!(countdown(&rig), None, "{texts:?}");
    // The health falls at the effect's period, by the same amount, until it is over.
    let mut hits: Vec<(f64, String)> = Vec::new();
    let mut counted = 0;
    let mut gone = false;
    for _ in 0..40 * 60 {
        rig.frame(&[]);
        let now = rig.client.frame().server_time;
        let texts: Vec<String> = rig.seen[from..]
            .iter()
            .filter_map(|e| match e {
                SessionEvent::Notice(GameplayNotice::Do3DDamage { text, .. })
                    if text.text.starts_with('-') =>
                {
                    Some(text.text.clone())
                }
                _ => None,
            })
            .collect();
        for text in &texts[counted..] {
            hits.push((now, text.clone()));
        }
        counted = texts.len();
        if rig
            .client
            .effect_list()
            .is_ok_and(|l| l.iter().all(|n| n.eid != node.eid))
        {
            gone = true;
            break;
        }
    }
    assert!(gone, "the effect never ran out");
    // The longest run of the same loss one second apart is the damage over time.
    let mut best = Vec::new();
    for start in 0..hits.len() {
        let mut run = vec![hits[start].clone()];
        for hit in &hits[start + 1..] {
            let last = run.last().unwrap();
            let gap = hit.0 - last.0;
            if hit.1 == last.1 && (0.8..1.3).contains(&gap) {
                run.push(hit.clone());
            }
        }
        if run.len() > best.len() {
            best = run;
        }
    }
    eprintln!("ticks {best:?}");
    let amount: i32 = best[0].1.trim_start_matches('-').parse().unwrap();
    assert!((8..=11).contains(&best.len()) && amount >= 10, "{best:?}");
    assert!(best[0].0 - node.time_cast < 2.0);
}

/// The skill levels as the client reads them (`InqSkillLevel`: the raw level and the level with the
/// player's effects on it, which the skill panel shows): the melee defense skill.
const MELEE_DEFENSE: u32 = 8;

/// A buff that changes a stat shows it where the client shows stats: the skill's level with the
/// effect on it (`SkillLevel::enchanted`). Defensive Stance (0xA8) raises the melee defense rating
/// by its record's table read at the skill's level (`800008=1:10,50:100`); the level goes back
/// when the effect runs out.
#[test]
fn a_stat_buff_changes_the_skill_level_while_it_lasts_and_reverts() {
    let Some(mut rig) = rig_for("Effectstat") else {
        return;
    };
    ready(&mut rig);
    train(&mut rig, &DEFENCE_BUFF_CHAIN);
    let slot = arm(&mut rig, DEFENCE_BUFF);
    raise(&mut rig, DEFENCE_BUFF);
    let rank = rig.client.skill_level(DEFENCE_BUFF).unwrap().raw;
    assert!(rank > 0);
    let level = |rig: &mut LiveRig| rig.client.skill_level(MELEE_DEFENSE).unwrap();
    let before = level(&mut rig);
    assert_eq!(before.raw, before.enchanted, "{before:?}");
    cast(&mut rig, slot, 1);
    rig.run(1.0);
    let during = level(&mut rig);
    // The table's two points (level 1: 10, level 50: 100), read at the skill's level.
    let expected = 10.0 + 90.0 * f64::from(rank - 1) / 49.0;
    eprintln!("before {before:?} during {during:?} expected {expected:.2}");
    assert_eq!(during.raw, before.raw);
    assert_eq!(f64::from(during.enchanted - during.raw), expected.floor());
    wait(&mut rig, "the effect expired", |r| {
        r.client.effect_list().is_ok_and(|l| l.is_empty())
    });
    rig.run(1.0);
    let after = level(&mut rig);
    assert_eq!(after.raw, after.enchanted, "{after:?}");
}
