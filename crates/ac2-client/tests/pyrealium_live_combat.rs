//! Combat against a running Pyrealium server (`PYREALIUM_LIVE`, see `tests/live/mod.rs`): the new
//! character starts in an interior cell next to the hostile creatures Pyrealium's encounter rules
//! spawn, which attack at once. Each test plays one part of that fight as the window would: a
//! creature selected by a click and by the selection keys, attacked with the bar's attack
//! (default attack), hit and killed; the player's vitals falling under the creatures' blows; and
//! the player's death, the resurrect dialog and the respawn.
//!
//! Run with `PYREALIUM_LIVE_TEST=pyrealium_live_combat scripts/pyrealium-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_net::protocol::ObjectAction;
use ac2_ui::tokenize;
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::{LiveRig, SIZE};

/// The weenie type bits of a monster (`gmSelectionUI::TargetTypeVsKey`'s `0x4F`).
const MONSTER_BITS: i32 = 0x4F;
/// `gmSelectionUI`'s keys: the closest monster, and the closest foe (`CanAutoAttackTarget`).
const CLOSEST_MONSTER_KEY: u32 = 0x4100_001B;
const CLOSEST_FOE_KEY: u32 = 0x4100_0024;
/// The `death` behaviour (`gmbehaviors.emp`) and the `dead` mode (`gmmodes.emp`).
const DEATH_BEHAVIOR: u32 = 0x4100_001F;
const DEAD_MODE: u32 = 0x4100_0005;
/// `Health` (the vital code of `GameplayNotice::UpdateVital`).
const HEALTH: u32 = 0x4000_0001;

/// The monsters in the player's world.
fn monsters(rig: &LiveRig) -> Vec<u64> {
    let player = rig.client.session().player();
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| Some(o.id) != player)
        .filter(|o| {
            o.qualities
                .inq_int(0x14)
                .is_some_and(|t| t & MONSTER_BITS == MONSTER_BITS)
        })
        .map(|o| o.id)
        .collect()
}

/// Frames until the creatures' qualities (type, selectability: the class defaults their entity
/// DIDs name) are known.
fn wait_for_monsters(rig: &mut LiveRig) -> Vec<u64> {
    rig.wait("no creature came into the world", |r| {
        !monsters(r).is_empty()
    });
    monsters(rig)
}

fn selected(rig: &LiveRig) -> Option<u64> {
    rig.client.frame().selection.map(|s| s.id)
}

/// A screen point where the pick finds `id` first (`SmartBox`'s pick list), if there is one.
fn point_on(rig: &LiveRig, id: u64) -> Option<(i32, i32)> {
    let frame = rig.client.frame();
    (0..SIZE.1 as i32).step_by(8).find_map(|y| {
        (0..SIZE.0 as i32).step_by(8).find_map(|x| {
            (rig.view.pick_list(&frame, SIZE, (x, y)).first() == Some(&id)).then_some((x, y))
        })
    })
}

/// The player's health, as the vitals panel shows it.
fn health(rig: &LiveRig) -> Option<(u64, u64)> {
    rig.client
        .frame()
        .player_vitals
        .health
        .map(|v| (v.current, v.max))
}

fn notices(rig: &LiveRig) -> impl Iterator<Item = &GameplayNotice> {
    rig.seen.iter().filter_map(|e| match e {
        SessionEvent::Notice(n) => Some(n),
        _ => None,
    })
}

fn said(rig: &LiveRig, part: &str) -> bool {
    notices(rig).any(|n| {
        matches!(n, GameplayNotice::DisplayStringInfo { text, .. }
            if text.literal.as_deref().is_some_and(|t| t.contains(part)))
    })
}

/// A click on a creature in the world selects it: `gmSelectionUI::Finish3DSelection` picks the
/// first of the pick list, which `CWRepository::GetBool(0x270)` allows, and the selection panel
/// then has the creature's health.
#[test]
fn a_click_selects_a_creature() {
    let Some(mut rig) = LiveRig::new("Clickselect") else {
        return;
    };
    let creatures = wait_for_monsters(&mut rig);
    rig.settle_view();
    // Some of the monsters in the vault (the guards) have no health to show, and which creature
    // the pick finds first depends on where the training creatures settled around the player, so
    // each one picked is tried until a selected creature shows its health.
    let mut tried = Vec::new();
    let mut clicked = None;
    'search: for _ in 0..40 {
        let untried: Vec<u64> = creatures
            .iter()
            .copied()
            .filter(|id| !tried.contains(id))
            .collect();
        for id in untried {
            let Some(at) = point_on(&rig, id) else {
                continue;
            };
            rig.click(at);
            if selected(&rig) != Some(id) {
                continue;
            }
            tried.push(id);
            if rig.until(3.0, |r| {
                r.client
                    .frame()
                    .selection
                    .and_then(|s| s.info)
                    .is_some_and(|i| i.max_health > 0)
            }) {
                clicked = Some(id);
                break 'search;
            }
        }
        rig.run(0.25);
        rig.settle_view();
    }
    let id = clicked.expect("no click selected a creature that shows its health");
    assert_eq!(selected(&rig), Some(id));
}

/// The selection keys take the closest creature: they pick among the radar's blips, which the
/// player's radar options (`GameplayOptionsProfile::m_radarMask`) let the creatures have.
#[test]
fn the_selection_keys_pick_a_creature() {
    let Some(mut rig) = LiveRig::new("Keyselect") else {
        return;
    };
    wait_for_monsters(&mut rig);
    assert!(
        rig.until(20.0, |r| !r.client.frame().radar.blips.is_empty()),
        "the radar shows no blip"
    );
    for key in [CLOSEST_MONSTER_KEY, CLOSEST_FOE_KEY] {
        rig.command(Command::Select(None));
        rig.run(0.5);
        assert_eq!(selected(&rig), None);
        rig.frame(&[(key, 1.0, true)]);
        rig.frame(&[(key, 1.0, false)]);
        assert!(
            rig.until(5.0, |r| selected(r).is_some()),
            "key {key:#x} selected nothing"
        );
        let id = selected(&rig).unwrap();
        assert!(monsters(&rig).contains(&id), "{id:#x} is no creature");
    }
}

/// The default attack (the bar's attack button, `gmShortcutUI::RequestAttackStateChange`) on a
/// selected creature: the creature's health falls with each blow, the selection panel follows,
/// and the creature dies and leaves the world.
#[test]
fn attacking_a_creature_kills_it() {
    let Some(mut rig) = LiveRig::new("Attackkill") else {
        return;
    };
    let creatures = wait_for_monsters(&mut rig);
    let target = creatures[0];
    rig.command(Command::Select(Some(target)));
    rig.run(1.0);
    assert_eq!(selected(&rig), Some(target));
    rig.seen.clear();
    rig.command(Command::SetAttacking(true));
    rig.wait("the attack never started", |r| {
        notices(r).any(|n| matches!(n, GameplayNotice::UpdateAttackState(true)))
    });
    let hurt = rig.until(30.0, |r| {
        r.client
            .session()
            .objects()
            .get(target)
            .and_then(|o| o.qualities.inq_int(0x102))
            .is_some_and(|health| health < 60)
    });
    assert!(hurt, "the creature's health never fell");
    // Dead: its health is gone and it lies down; Pyrealium keeps the body in the world (a training
    // kill leaves it standing for good), or removes it once it despawns.
    let gone = rig.until(60.0, |r| {
        r.seen.contains(&SessionEvent::ObjectRemoved(target))
            || r.client
                .session()
                .objects()
                .get(target)
                .is_none_or(|o| o.qualities.inq_int(0x102) == Some(0))
    });
    assert!(gone, "the creature never died");
    // It lay down first: the `death` behaviour into the `dead` mode (`gmbehaviors.emp`,
    // `gmmodes.emp`), at zero health. Pyrealium keeps the body there as the corpse until it despawns.
    let died = rig.seen.iter().any(|e| {
        matches!(e, SessionEvent::ObjectAction { object, action: ObjectAction::DoBehavior(b) }
            if *object == target && b.behavior == DEATH_BEHAVIOR && b.mode == DEAD_MODE)
    });
    assert!(died, "the creature never played its death");
    // It was struck more than once on the way.
    let blows = rig
        .seen
        .iter()
        .filter(|e| {
            matches!(e, SessionEvent::QualityChanged { object, quality: 0x102, .. } if *object == target)
        })
        .count();
    assert!(blows >= 2, "{blows} blows");
    assert!(said(&rig, "defeated"), "no kill message");
}

/// The creatures' blows take the player's health down, in the vitals panel and as damage text.
#[test]
fn the_players_health_falls_under_the_creatures_blows() {
    let Some(mut rig) = LiveRig::new("Takedamage") else {
        return;
    };
    wait_for_monsters(&mut rig);
    rig.wait("no vitals", |r| health(r).is_some());
    let (_, max) = health(&rig).unwrap();
    assert!(
        rig.until(30.0, |r| health(r).is_some_and(|(now, _)| now < max)),
        "the vitals never fell from {max}"
    );
    assert!(said(&rig, "hits you"), "no damage line");
    assert!(notices(&rig).any(|n| matches!(n, GameplayNotice::UpdateVital { vital: HEALTH, .. })));
}

/// The player at zero health: the death state raises the resurrect dialog
/// (`ResurrectField`), its button asks the respawn, and the player is alive again.
#[test]
fn the_player_dies_is_offered_resurrection_and_comes_back() {
    let Some(mut rig) = LiveRig::new("Diethenlive") else {
        return;
    };
    wait_for_monsters(&mut rig);
    let dead = rig.until(180.0, |r| {
        notices(r).any(|n| matches!(n, GameplayNotice::UpdateDeathState(true)))
    });
    assert!(dead, "the player never died; health {:?}", health(&rig));
    assert_eq!(health(&rig).map(|(now, _)| now), Some(0));
    // The dialog's button, once the dialog shows.
    let dialog = |rig: &LiveRig| {
        let tree = rig.view.hud_tree()?;
        let root = *tree.roots().iter().find(|&&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ResurrectField"))
        })?;
        let button = tree.child_recursive(root, tokenize("ResurrectButton"))?;
        let b = tree.screen_box(button);
        tree.get(root)
            .is_some_and(|e| e.visible())
            .then_some(((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2))
    };
    assert!(
        rig.until(5.0, |r| dialog(r).is_some()),
        "no resurrect dialog"
    );
    let before = rig.client.frame().player.map(|p| (p.cell, p.origin));
    rig.seen.clear();
    let at = dialog(&rig).unwrap();
    rig.click(at);
    rig.wait("the player never came back", |r| {
        notices(r).any(|n| matches!(n, GameplayNotice::UpdateDeathState(false)))
    });
    assert!(
        rig.until(10.0, |r| health(r).is_some_and(|(now, _)| now > 0)),
        "alive again but health {:?}",
        health(&rig)
    );
    assert!(rig.until(5.0, |r| dialog(r).is_none()), "the dialog stayed");
    let after = rig.client.frame().player.map(|p| (p.cell, p.origin));
    eprintln!("respawned: {before:?} -> {after:?}");
    assert!(after.is_some());
}

/// Examining a creature as the player does (a right click on it in the world: the selection, then
/// `CM_UI::SendNotice_OpenExaminationPanel`) shows its profile in the examination panel: the
/// creature's name and the rows the server answers with (its health).
#[test]
fn examining_a_creature_shows_its_profile() {
    let Some(mut rig) = LiveRig::new("Creatureexamine") else {
        return;
    };
    let creatures = wait_for_monsters(&mut rig);
    rig.settle_view();
    let mut tried = Vec::new();
    let mut shown = None;
    'search: for _ in 0..3 {
        let untried: Vec<u64> = creatures
            .iter()
            .copied()
            .filter(|id| !tried.contains(id))
            .collect();
        for id in untried {
            let Some(at) = point_on(&rig, id) else {
                continue;
            };
            rig.right_click(at);
            if selected(&rig) != Some(id) {
                continue;
            }
            tried.push(id);
            if rig.until(5.0, |r| {
                r.view
                    .examination_texts()
                    .iter()
                    .any(|t| t.starts_with("Health:"))
            }) {
                shown = Some(id);
                break 'search;
            }
        }
        rig.run(0.25);
        rig.settle_view();
    }
    let Some(id) = shown else {
        panic!(
            "no creature's examination showed a profile: tried {tried:x?}, panel {:?}, server said {:?}",
            rig.view.examination_texts(),
            notices(&rig)
                .filter_map(|n| match n {
                    GameplayNotice::DisplayStringInfo { text, .. } => text.literal.clone(),
                    _ => None,
                })
                .collect::<Vec<_>>()
        );
    };
    let texts = rig.view.examination_texts();
    eprintln!("creature {id:#x} examination {texts:?}");
    // The rows the creature's qualities give (Pyrealium's profile: the name, then its health), and
    // the name is a name, not the server's fallback for an unnamed object.
    let object = rig.client.session().objects().get(id).unwrap();
    let (current, max) = (
        object.qualities.inq_int(0x102).unwrap(),
        object.qualities.inq_int(0x107).unwrap(),
    );
    assert!(
        texts.contains(&format!("Health: {current} / {max}")),
        "{texts:?}"
    );
    let name = texts.iter().find(|t| !t.contains(':')).expect("a name");
    assert!(!matches!(name.as_str(), "Object" | "Creature"), "{texts:?}");
}
