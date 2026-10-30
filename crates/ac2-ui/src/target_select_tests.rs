//! `gmSelectionUI::Select` over a fake player: the foe keys ask the script's
//! `CanAutoAttackTarget` of what they would take.

use std::cell::RefCell;

use crate::radar_state::RadarBlip;
use crate::selection_notices::SelectionSnapshot;
use crate::target_select::{select_key_calls, SelectionCall, SelectionObject, SelectionSource};

const PLAYER: u64 = 0x8000_0000_0000_0001;
const FOE: u64 = 0x8000_0000_0000_0010;
const FRIEND: u64 = 0x8000_0000_0000_0011;
/// `TargetTypeVsKey`'s creature bits.
const CREATURE: u32 = 0x4F;
/// `0x41000024`: the auto-target, the next foe (closest).
const NEXT_FOE: u32 = 0x4100_0024;

struct World {
    blips: Vec<RadarBlip>,
    attackable: Vec<u64>,
    asked: RefCell<Vec<u64>>,
}

fn blip(id: u64, x: f32) -> RadarBlip {
    RadarBlip {
        id,
        color: [1.0; 4],
        shape: 0x4000_0001,
        cell: 0,
        origin: [0.0; 3],
        offset: [x, 0.0, 0.0],
        in_range: true,
        selected: false,
    }
}

impl SelectionSource for World {
    fn snapshot(&self) -> Option<SelectionSnapshot> {
        Some(SelectionSnapshot {
            player: PLAYER,
            selection: None,
            last_selection: None,
            last_attacker: None,
        })
    }
    fn auto_target(&self) -> bool {
        true
    }
    fn radar_range(&self) -> f32 {
        50.0
    }
    fn radar_blips(&self) -> &[RadarBlip] {
        &self.blips
    }
    fn object(&self, _: u64) -> Option<SelectionObject> {
        Some(SelectionObject {
            kind: CREATURE,
            selectable: true,
            owner: None,
        })
    }
    fn is_fellow(&self, _: u64) -> bool {
        false
    }
    fn can_auto_attack(&self, id: u64) -> bool {
        self.asked.borrow_mut().push(id);
        self.attackable.contains(&id)
    }
}

#[test]
fn the_foe_key_takes_what_the_script_lets_the_player_attack() {
    let world = World {
        blips: vec![blip(FRIEND, 3.0), blip(FOE, 9.0)],
        attackable: vec![FOE],
        asked: RefCell::default(),
    };
    assert_eq!(
        select_key_calls(&world, NEXT_FOE, false),
        vec![SelectionCall::Select(Some(FOE))]
    );
    assert!(world.asked.borrow().contains(&FRIEND));
    // A key that is not a foe key never asks the script.
    world.asked.borrow_mut().clear();
    select_key_calls(&world, 0x4100_001B, false);
    assert!(world.asked.borrow().is_empty());
}
