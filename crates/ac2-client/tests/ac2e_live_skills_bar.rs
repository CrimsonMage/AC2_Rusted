//! The shortcut bar's skill slots and the combat toggle against a running AC2e (the skills run of
//! `scripts/ac2e-live-test.sh`, `AC2E_LIVE_TEST=ac2e_live_skills_bar`): a skill that wants an item
//! in the right hand is usable (`IsSkillUsableBasedOnInventoryByEnum`) once a weapon is wielded,
//! pressing its slot uses it, the slot's `TimeLeftMeter` counts the recharge and clears; and the
//! attack key toggles combat on a selected creature, the bar's attack buttons swapping with the
//! server's attack state.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_net::protocol::ShortcutInfo;
use ac2_ui::{tokenize, AttributeValue};
use ac2_weenie::{GameplayNotice, SessionEvent, SkillRequest};
use live::LiveRig;

const HEAL: u32 = 0x55;
const HEAL_PREREQUISITE: u32 = 0x59;
const SKILL_SHORTCUT: u32 = 0x4000_0001;
const PRIMARY_HAND: u32 = 0x40;
/// `gmShortcutUI::ListenToMessage`'s attack key (`AUTOATTACK_KEY`).
const ATTACK_KEY: u32 = 0x4100_0013;

fn rig_for(name: &str) -> Option<LiveRig> {
    if std::env::var_os("AC2E_DEV_START_XP").is_none() {
        eprintln!("run through scripts/ac2e-live-test.sh with AC2E_LIVE_TEST=ac2e_live_skills_bar");
        return None;
    }
    LiveRig::new(name)
}

fn wait(rig: &mut LiveRig, what: &str, mut ready: impl FnMut(&mut LiveRig) -> bool) {
    for _ in 0..60 * 60 {
        if ready(rig) {
            return;
        }
        rig.frame(&[]);
    }
    panic!("{what}");
}

/// The slot's recharge meter position (0 just used, 1 ready), if it shows one.
fn meter(rig: &LiveRig, slot: usize) -> Option<f32> {
    let tree = rig.view.ui_tree()?;
    let item = rig.view.shortcut_slot_element(slot)?;
    let m = tree.child_recursive(item, tokenize("TimeLeftMeter"))?;
    match tree.get(m)?.attribute(tokenize("position")) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

#[test]
fn a_wielded_weapon_unlocks_the_skill_and_its_slot_counts_the_recharge() {
    let Some(mut rig) = rig_for("Skillbar") else {
        return;
    };
    wait(&mut rig, "the script and experience", |r| {
        r.client.skill_credits().is_ok() && r.client.frame().progress.experience.available_xp > 0
    });
    for skill in [HEAL_PREREQUISITE, HEAL] {
        rig.command(Command::Skill(SkillRequest::Train(skill)));
        wait(&mut rig, "trained", |r| {
            r.client.is_skill_trained(skill) == Ok(true)
        });
    }
    let bar = rig.client.shortcut_array().unwrap();
    let slot = bar
        .iter()
        .position(|s| matches!(s, ShortcutInfo::Empty(_)))
        .expect("an empty slot");
    rig.client
        .set_shortcut(
            slot as u32,
            ShortcutInfo::Enum {
                kind: SKILL_SHORTCUT,
                value: HEAL,
            },
        )
        .unwrap();
    wait(&mut rig, "the slot", |r| {
        r.client
            .shortcut_array()
            .is_ok_and(|bar| matches!(bar[slot], ShortcutInfo::Enum { value, .. } if value == HEAL))
    });
    rig.run(1.0);
    let hand = |r: &LiveRig| {
        r.client
            .frame()
            .inventory
            .equipped
            .iter()
            .any(|&(l, _)| l == PRIMARY_HAND)
    };
    // The VM's answer with nothing wielded, then with the weapon in the hand.
    let before = rig.client.is_skill_usable_by_inventory(HEAL);
    let before_meter = meter(&rig, slot);
    eprintln!("nothing wielded: usable {before:?}, slot meter {before_meter:?}");
    assert!(!hand(&rig));
    // The skill's tree wants a particular weapon (the skill panel's text: "You must be wielding
    // a Staff ..."), so each hand item of the starter gear is wielded in turn and the VM asked.
    let candidates: Vec<u64> = rig
        .client
        .frame()
        .inventory
        .items
        .iter()
        .filter(|i| i.stats.preferred_location & PRIMARY_HAND != 0)
        .map(|i| i.id)
        .collect();
    assert!(!candidates.is_empty(), "hand items in the starter gear");
    let mut after = Ok(false);
    let mut after_meter = None;
    for weapon in candidates {
        rig.command(Command::EquipItem {
            item: weapon,
            location: PRIMARY_HAND,
        });
        let landed = rig.until(10.0, |r| {
            r.client
                .frame()
                .inventory
                .equipped
                .iter()
                .any(|&(l, id)| l == PRIMARY_HAND && id == weapon)
        });
        {
            let objects = rig.client.session().objects();
            if let Some(o) = objects.get(weapon) {
                eprintln!(
                    "item {weapon:#x}: WeenieType {:?} valid locations {:?} preferred {:?} wielder {:?}",
                    o.qualities.inq_int(0x14),
                    o.qualities.inq_int(275),
                    o.qualities.inq_int(282),
                    o.qualities.inq_instance_id(0x105)
                );
            }
        }
        eprintln!(
            "equip {weapon:#x} landed {landed}; equipped now {:x?}",
            rig.client.frame().inventory.equipped
        );
        rig.run(1.0);
        after = rig.client.is_skill_usable_by_inventory(HEAL);
        after_meter = meter(&rig, slot);
        eprintln!("wielded {weapon:#x}: usable {after:?}, slot meter {after_meter:?}");
        if after == Ok(true) {
            break;
        }
    }
    assert_eq!(after, Ok(true), "the wielded weapon makes the skill usable");
    assert!(
        after_meter.is_none_or(|m| m >= 1.0) || before_meter.is_some(),
        "the slot is not left greyed on the long meter: {after_meter:?}"
    );
    // Press the slot with the pointer.
    let at = rig
        .view
        .shortcut_slot_center(slot)
        .expect("the slot is drawn");
    rig.click(at);
    // The recharge shows on the slot and runs out.
    let seen = std::cell::RefCell::new(Vec::new());
    let started = rig.until(10.0, |r| {
        let m = meter(r, slot);
        seen.borrow_mut().push(m);
        m.is_some_and(|m| m < 1.0)
    });
    eprintln!("meter after the press: {:?}", seen.borrow());
    assert!(started, "the used skill's slot shows its recharge");
    assert!(
        rig.until(60.0, |r| meter(r, slot).is_none_or(|m| m >= 1.0)),
        "the recharge clears when the skill is ready"
    );
}

#[test]
fn the_attack_key_toggles_combat_on_a_selected_creature() {
    let Some(mut rig) = rig_for("Combatkey") else {
        return;
    };
    rig.wait("a creature", |r| {
        r.client
            .session()
            .objects()
            .iter()
            .any(|o| o.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F))
    });
    let target = rig
        .client
        .session()
        .objects()
        .iter()
        .find(|o| {
            o.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
                && o.qualities.inq_int(0x102).is_none_or(|h| h > 0)
        })
        .map(|o| o.id)
        .unwrap();
    rig.command(Command::Select(Some(target)));
    rig.run(1.0);
    let ghosted = |rig: &LiveRig, name: &str| {
        let tree = rig.view.ui_tree().unwrap();
        let id = tokenize(name);
        tree.roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id))
            .map(|i| {
                tree.get(i).unwrap().attribute(tokenize("ghosted"))
                    == Some(&AttributeValue::Bool(true))
            })
    };
    assert_eq!(ghosted(&rig, "StartAttackButton"), Some(false));
    let mark = rig.seen.len();
    rig.frame(&[(ATTACK_KEY, 1.0, true)]);
    rig.frame(&[(ATTACK_KEY, 1.0, false)]);
    assert!(
        rig.until(10.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            SessionEvent::Notice(GameplayNotice::UpdateAttackState(true))
        ))),
        "the server starts the attack"
    );
    rig.run(0.5);
    assert_eq!(
        ghosted(&rig, "StartAttackButton"),
        Some(true),
        "combat shows"
    );
    assert_eq!(ghosted(&rig, "StopAttackButton"), Some(false));
    let mark = rig.seen.len();
    rig.frame(&[(ATTACK_KEY, 1.0, true)]);
    rig.frame(&[(ATTACK_KEY, 1.0, false)]);
    assert!(
        rig.until(10.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            SessionEvent::Notice(GameplayNotice::UpdateAttackState(false))
        ))),
        "the server ends the attack"
    );
    rig.run(0.5);
    assert_eq!(
        ghosted(&rig, "StartAttackButton"),
        Some(false),
        "the bar returns"
    );
}
