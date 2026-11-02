//! A skill that wants a weapon stays usable across a relog with the weapon still wielded, against a
//! running AC2e (`AC2E_LIVE_TEST=ac2e_live_skills_relog scripts/ac2e-live-test.sh`, the skills
//! profile): the login's equipment records carry what `EquipItem_Done` recorded during play.

#![cfg(feature = "window")]

mod live;

use ac2_client::Command;
use ac2_weenie::SkillRequest;
use live::LiveRig;

const HEAL: u32 = 0x55;
const HEAL_PREREQUISITE: u32 = 0x59;
const PRIMARY_HAND: u32 = 0x40;

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
fn a_wielded_weapon_keeps_the_skill_usable_after_a_relog() {
    if std::env::var_os("AC2E_DEV_START_XP").is_none() {
        eprintln!("run through scripts/ac2e-live-test.sh with AC2E_LIVE_TEST=ac2e_live_skills_relog");
        return;
    }
    let Some(mut rig) = LiveRig::new("Skillrelog") else {
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
    let candidates: Vec<u64> = rig
        .client
        .frame()
        .inventory
        .items
        .iter()
        .filter(|i| i.stats.preferred_location & PRIMARY_HAND != 0)
        .map(|i| i.id)
        .collect();
    let mut weapon = None;
    for item in candidates {
        rig.command(Command::EquipItem {
            item,
            location: PRIMARY_HAND,
        });
        rig.until(10.0, |r| {
            r.client
                .frame()
                .inventory
                .equipped
                .iter()
                .any(|&(l, id)| l == PRIMARY_HAND && id == item)
        });
        rig.run(1.0);
        if rig.client.is_skill_usable_by_inventory(HEAL) == Ok(true) {
            weapon = Some(item);
            break;
        }
    }
    let weapon = weapon.expect("a starter weapon that makes the heal usable");
    eprintln!("usable with {weapon:#x} before the relog");
    rig.relog();
    wait(&mut rig, "the script after the relog", |r| {
        r.client.skill_credits().is_ok()
    });
    rig.run(3.0);
    let worn = rig
        .client
        .frame()
        .inventory
        .equipped
        .iter()
        .any(|&(l, _)| l == PRIMARY_HAND);
    let usable = rig.client.is_skill_usable_by_inventory(HEAL);
    eprintln!("after the relog: weapon worn {worn}, usable {usable:?}");
    assert!(worn, "the weapon is still wielded after the relog");
    assert_eq!(usable, Ok(true), "the wielded weapon still makes the skill usable");
}
