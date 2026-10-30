use ac2_assets::{Ac2FxNodeGate, CHANNEL_KEYS};
use ac2_prefs::{NameTable, PreferenceBank, Value};

use super::*;

/// Every section and key has a name: the tests do not read the DAT's tables.
struct Names;

impl NameTable for Names {
    fn section_name(&self, id: u32) -> Option<String> {
        Some(format!("S{id:X}"))
    }
    fn key_name(&self, id: u32) -> Option<String> {
        Some(format!("K{id:X}"))
    }
    fn enum_name(&self, _map: u32, label: u32) -> Option<String> {
        Some(format!("L{label}"))
    }
    fn enum_label(&self, _map: u32, name: &str) -> Option<u32> {
        name.strip_prefix('L')?.parse().ok()
    }
}

fn choice_values(bank: &PreferenceBank, key: u32) -> Vec<i32> {
    bank.find(key)
        .expect("registered")
        .choices
        .iter()
        .map(|c| c.value)
        .collect()
}

#[test]
fn the_choices_and_defaults_follow_the_top_levels() {
    let mut bank = PreferenceBank::new(b"");
    assert_eq!(register(&mut bank, &Names, Some((4, 2))), 3);
    assert_eq!(choice_values(&bank, GORE_AMOUNT), vec![1, 2, 3, 4]);
    assert_eq!(choice_values(&bank, GORE_HUE), vec![1, 2]);
    assert_eq!(bank.find(GORE_AMOUNT).unwrap().value, Value::Int(4));
    assert_eq!(bank.find(GORE_HUE).unwrap().value, Value::Int(2));
    assert_eq!(
        bank.find(FOOT_STEP_PARTICLES).unwrap().value,
        Value::Bool(true)
    );
    assert_eq!(bank.owners(GORE_AMOUNT), &[OWNER]);
    // The foot step sounds key is linked and has nothing registered.
    assert_eq!(bank.owners(0x4000_0037), &[OWNER]);
    assert!(bank.find(0x4000_0037).is_none());

    // A language that stops at "none" has two gore choices and one hue.
    let mut low = PreferenceBank::new(b"");
    assert_eq!(register(&mut low, &Names, Some((2, 1))), 3);
    assert_eq!(choice_values(&low, GORE_AMOUNT), vec![1, 2]);
    assert_eq!(choice_values(&low, GORE_HUE), vec![1]);

    // No `StringState`: none of the three, only the links.
    let mut none = PreferenceBank::new(b"");
    assert_eq!(register(&mut none, &Names, None), 0);
    assert!(none.find(GORE_AMOUNT).is_none());
    assert_eq!(none.owners(GORE_HUE), &[OWNER]);
}

#[test]
fn the_bank_values_become_the_channel_table_and_gate_the_nodes() {
    let file = b"[S40000009]\r\nK40000039=0\r\nK40000035=3\r\n";
    let mut bank = PreferenceBank::new(file);
    register(&mut bank, &Names, Some((4, 2)));
    let table = channel_prefs(&bank);
    // The gore amount, the hue and the foot step particles under their channel keys.
    assert_eq!(table.values, [Some(3), Some(2), None, Some(0)]);

    let node = |channel: usize, value: u32| {
        let mut gate = Ac2FxNodeGate {
            has_channels: true,
            ..Ac2FxNodeGate::default()
        };
        gate.channels[channel] = Some(value);
        gate
    };
    // A node that wants low gore (3) plays; one that wants normal gore (4) does not.
    assert!(!node(0, 3).channel_missmatch(&table));
    assert!(node(0, 4).channel_missmatch(&table));
    // Foot step particles are off: a node keyed on them with 1 is held back, with 0 it plays.
    assert!(node(3, 1).channel_missmatch(&table));
    assert!(!node(3, 0).channel_missmatch(&table));
    assert_eq!(CHANNEL_KEYS[3], 0x4000_0008);
}

#[test]
fn applying_sets_the_table_every_fx_manager_reads() {
    let mut bank = PreferenceBank::new(b"");
    register(&mut bank, &Names, Some((4, 2)));
    bank.find_mut(FOOT_STEP_PARTICLES).unwrap().value = Value::Bool(false);
    apply(&bank);
    let mut gate = Ac2FxNodeGate {
        has_channels: true,
        ..Ac2FxNodeGate::default()
    };
    gate.channels[3] = Some(1);
    assert!(
        !gate.plays(false, false),
        "a footstep node waits on the switch"
    );
    assert!(!gate.updates(false));
    bank.find_mut(FOOT_STEP_PARTICLES).unwrap().value = Value::Bool(true);
    apply(&bank);
    assert!(gate.plays(false, false));
    // Leave the shared table as a fresh client has it.
    ac2_assets::set_fx_channel_prefs(Default::default());
}
