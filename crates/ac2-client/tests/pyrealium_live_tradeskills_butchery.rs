//! Butchery against a running Pyrealium (see `tests/live/mod.rs`; `PYREALIUM_LIVE_TEST=pyrealium_live_tradeskills_butchery
//! scripts/pyrealium-live-test.sh`): a new character crafts the starter Hunting Knife at the craft panel,
//! kills a training creature, uses the knife on its corpse and the trait the creature's family gives. The data gives the tool, the five traits and the
//! messages; the yield is Pyrealium policy (`butchery.rs`).

#![cfg(feature = "window")]

mod live;

use ac2_ui::ToolbarChild;
use ac2_weenie::{GameplayNotice, SessionEvent};
use live::trade::*;
use live::LiveRig;

/// `rcp_toolnew_huntingknife_starter`, `wsl_toolnew_huntingknife_starter`.
const KNIFE_RECIPE: u32 = 0x6F00_2F27;
const KNIFE: u32 = 0x4700_2F73;
/// `wsl_mote_tallow`.
const TALLOW: u32 = 0x4700_27A7;
/// `wsl_mote_sinew`.
const SINEW: u32 = 0x4700_141F;

fn units(rig: &LiveRig, definition: u32) -> i32 {
    let items = &rig.client.frame().inventory.items;
    rig.client
        .session()
        .objects()
        .iter()
        .filter(|o| o.entity == Some(definition))
        .filter_map(|o| items.iter().find(|i| i.id == o.id))
        .map(|i| i.stats.quantity)
        .sum()
}

/// The nearest monster that is alive and not fought yet.
fn next_monster(rig: &LiveRig, fought: &[u64]) -> Option<u64> {
    let frame = rig.client.frame();
    let player = frame.player.as_ref()?;
    frame
        .objects
        .iter()
        .filter(|o| o.id != player.id && !fought.contains(&o.id))
        .filter(|o| {
            rig.client.session().objects().get(o.id).is_some_and(|x| {
                x.qualities.inq_int(0x14).is_some_and(|t| t & 0x4F == 0x4F)
                    && x.qualities.inq_int(0x102).is_none_or(|h| h > 0)
            })
        })
        .map(|o| {
            let d =
                (o.origin[0] - player.origin[0]).powi(2) + (o.origin[1] - player.origin[1]).powi(2);
            (o.id, d)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id)
}

/// Kills the next attackable creature and returns it (its body stays where it fell).
fn kill_a_creature(rig: &mut LiveRig, fought: &mut Vec<u64>) -> u64 {
    for _ in 0..16 {
        assert!(
            rig.until(60.0, |r| next_monster(r, fought).is_some()),
            "a creature"
        );
        let target = next_monster(rig, fought).unwrap();
        fought.push(target);
        let before = rig.seen.len();
        rig.command(ac2_client::Command::Select(Some(target)));
        rig.run(0.5);
        rig.command(ac2_client::Command::SetAttacking(true));
        rig.until(90.0, |r| {
            let said = said_since(r, before);
            said.iter().any(|l| {
                l.contains("Creature defeated")
                    || l.contains("not an attackable")
                    || l.contains("out of attack range")
            })
        });
        let dead = said_since(rig, before)
            .iter()
            .any(|l| l.contains("Creature defeated"));
        rig.command(ac2_client::Command::SetAttacking(false));
        rig.run(0.5);
        if dead {
            return target;
        }
    }
    panic!("no creature could be attacked");
}

#[test]
fn a_hunting_knife_butchers_a_corpse_once() {
    let Some(mut rig) = rig_for("Butcherone") else {
        return;
    };
    open_recipe(&mut rig, TOOLMAKING, KNIFE_RECIPE);
    craft_once(&mut rig, KNIFE_RECIPE);
    rig.panel(ToolbarChild::Skill, false);
    assert!(
        rig.until(10.0, |r| !objects_of(r, |e| e == KNIFE).is_empty()),
        "the knife in the pack"
    );
    let knife = objects_of(&rig, |e| e == KNIFE)[0];
    let kit = units(&rig, TALLOW);
    assert_eq!(kit, 100, "the training kit's tallow");

    let corpse = kill_a_creature(&mut rig, &mut Vec::new());
    // As the player does: double-click the knife in the pack. The client reads the knife's
    // `Usage_*` qualities from the DAT's class defaults (`Usage_NumberOfEntitiesUsedWith` 720 = 1,
    // `Usage_InventoryRequired` 305, `Usage_ValidTargetTypes` 722 = 8), so Pyrealium sends none of
    // them, and the player's script waits for a target (`SetTargetingPointer`) instead of
    // sending the use.
    rig.panel(ToolbarChild::Inventory, true);
    wait(&mut rig, "the knife's icon", |r| {
        r.view.item_icon_center(PACK, knife).is_some()
    });
    let mark = rig.seen.len();
    double_click_item(&mut rig, knife);
    assert!(
        rig.until(5.0, |r| r.seen[mark..].iter().any(|e| matches!(
            e,
            SessionEvent::Notice(GameplayNotice::SetTargetingPointer(true))
        ))),
        "the double-clicked knife waits for a target"
    );
    assert_eq!(units(&rig, TALLOW), kit, "the knife alone butchers nothing");
    // The targeting click on the body, as the player does it: the player's script checks the
    // body's `Usage_TargetType` (721) against the knife's `Usage_ValidTargetTypes` (722 = 8) and
    // sends `WM_Usage` with both. Pyrealium's body carries 8 (the DAT default of a creature is 4).
    let before = rig.seen.len();
    rig.command(ac2_client::Command::Select(Some(corpse)));
    rig.run(0.3);
    rig.command(ac2_client::Command::UseSelection);
    assert!(
        rig.until(10.0, |r| units(r, TALLOW) == kit + 10
            && units(r, SINEW) == 10),
        "ten units each of sinew and tallow; said {:?}",
        said_since(&rig, before)
    );
    // AC2Hero `traits.html`: a Spawn (the training Idol Spawns) gives sinew and tallow.
    for trait_name in ["sinew", "tallow"] {
        assert!(
            said_since(&rig, before)
                .iter()
                .any(|l| l.starts_with(&format!(
                    "You butcher 10 units of {trait_name} from the corpse"
                ))),
            "{:?}",
            said_since(&rig, before)
        );
    }
    // A corpse butchers once.
    let before = rig.seen.len();
    double_click_item(&mut rig, knife);
    rig.run(0.5);
    rig.command(ac2_client::Command::Select(Some(corpse)));
    rig.run(0.3);
    rig.command(ac2_client::Command::UseSelection);
    assert!(
        rig.until(10.0, |r| said_since(r, before)
            .iter()
            .any(|l| l.contains("already been butchered"))),
        "{:?}",
        said_since(&rig, before)
    );
    assert_eq!(units(&rig, TALLOW), kit + 10);
}
