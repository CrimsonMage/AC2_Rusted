//! The raw-material traders against a running Pyrealium server (see `tests/live/mod.rs`): the store
//! records of the DAT (`store_cragstone.wc`, `store_cavendo.wc`) sell coal and flawless pyreal,
//! which no other record gives a source for. Needs Pyrealium's tradeskills profile with a starting purse
//! (`scripts/pyrealium-live-test.sh` with `PYREALIUM_LIVE_TEST=pyrealium_live_traders` sets that up).

#![cfg(feature = "window")]

mod live;

use live::trade::*;
use live::LiveRig;

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

#[test]
fn wolfgang_of_cragstone_sells_coal() {
    let Some(mut rig) = rig_for("Traderone") else {
        return;
    };
    assert_eq!(units(&rig, COAL), 0);
    assert_eq!(units(&rig, PYREAL), 0);

    // Wolfgang's `store_cragstone.wc`: coal at half a coin, then the six starter tools at 50.
    open_trader(&mut rig, WOLFGANG);
    assert_eq!(
        sale_names(&rig),
        [
            "Coal",
            "Crude Hatchet",
            "Crude Rock Hammer",
            "Crude Chisel",
            "Crude Mattock",
            "Crude Pickaxe",
            "Crude Hunting Knife"
        ]
    );
    buy_tens(&mut rig, "Coal", 2);
    assert!(rig.until(10.0, |r| units(r, COAL) == 20), "20 coal bought");
    // AC2Hero `resource.html`: Coal "200 = 100gp", the same half a coin a unit.
    assert!(
        lines(&rig).iter().any(|l| l == "Bought 20 for 10 gold."),
        "half a coin a unit: {:?}",
        lines(&rig).iter().rev().take(3).collect::<Vec<_>>()
    );
}

#[test]
fn sabine_of_cavendo_sells_flawless_pyreal() {
    let Some(mut rig) = rig_for("Tradertwo") else {
        return;
    };
    // Sabine's `store_cavendo.wc`: flawless pyreal at 50 a unit.
    open_trader(&mut rig, SABINE);
    assert_eq!(sale_names(&rig)[0], "Flawless Pyreal");
    buy_tens(&mut rig, "Flawless Pyreal", 5);
    assert!(
        rig.until(10.0, |r| units(r, PYREAL) == 50),
        "50 pyreal bought"
    );
    // AC2Hero `resource.html` lists Sabine's Flawless Pyreal as "50 = 2500gp": the DAT's unit
    // price of 50 makes that bundle cost what the page says.
    assert!(
        lines(&rig).iter().any(|l| l == "Bought 50 for 2500 gold."),
        "{:?}",
        lines(&rig).iter().rev().take(3).collect::<Vec<_>>()
    );
}
