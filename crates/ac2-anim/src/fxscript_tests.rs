//! `FxScript` decode: a synthetic record, and every `0x57` FX script in the DAT.

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_db::{ResourcePackedDecode, ResourceStore};

use super::*;

fn record(did: u32, effects: &[[u32; 7]], impulses: &[[u32; 3]], period: f32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut word = |w: u32| out.extend_from_slice(&w.to_le_bytes());
    word(did);
    word(effects.len() as u32);
    for effect in effects {
        effect.iter().for_each(|&w| word(w));
    }
    word(impulses.len() as u32);
    for impulse in impulses {
        // A jump streams no velocity: the jump flag, the number, the time.
        word(impulse[0]);
        if impulse[0] == 0 {
            [0.0_f32; 3].iter().for_each(|v| word(v.to_bits()));
        }
        word(impulse[1]);
        word(impulse[2]);
    }
    word(period.to_bits());
    out
}

#[test]
fn a_synthetic_record_decodes_its_triggers_and_period() {
    let did = 0x5700_0004;
    let bytes = record(
        did,
        &[[
            0x3000_0001,
            0.25_f32.to_bits(),
            0,
            1.0_f32.to_bits(),
            1.0_f32.to_bits(),
            0.0_f32.to_bits(),
            1,
        ]],
        &[[1, 2, 0.5_f32.to_bits()]],
        2.0,
    );
    let qdid = QualifiedDataId::new(DataId::new(did), 0);
    let script = Ac2FxScript::decode_packed(qdid, 0, &bytes).expect("decodes");
    assert_eq!(script.period, 2.0);
    assert_eq!(script.effects.len(), 1);
    assert_eq!(script.effects[0].fx, 0x3000_0001);
    assert_eq!(script.effects[0].time, 0.25);
    assert!(script.effects[0].play != 0 && !script.effects[0].stop);
    assert_eq!(script.impulses.len(), 1);
    assert!(script.impulses[0].jump);
    assert_eq!(script.impulses[0].num, 2);
    assert_eq!(script.impulses[0].time, 0.5);
}

#[test]
fn a_record_under_another_did_or_with_trailing_bytes_is_refused() {
    let did = 0x5700_0004;
    let bytes = record(did, &[], &[], 1.0);
    let other = QualifiedDataId::new(DataId::new(0x5700_0005), 0);
    assert!(Ac2FxScript::decode_packed(other, 0, &bytes).is_err());
    let mut long = bytes.clone();
    long.extend_from_slice(&0_u32.to_le_bytes());
    let qdid = QualifiedDataId::new(DataId::new(did), 0);
    assert!(Ac2FxScript::decode_packed(qdid, 0, &long).is_err());
}

#[test]
fn every_fx_script_in_the_dat_decodes() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let ids: Vec<u32> = dataset
        .ids_in_range(
            ac2_dat::DatFileKind::Portal,
            FX_SCRIPT_PREFIX,
            FX_SCRIPT_PREFIX | 0x00FF_FFFF,
        )
        .collect();
    assert_eq!(ids.len(), 22, "FX script records in portal.dat");
    let mut effects = 0;
    let mut impulses = 0;
    for did in &ids {
        let script = load_ac2_fx_script(&store, *did)
            .unwrap_or_else(|e| panic!("{did:08X} decode error: {e:?}"))
            .unwrap_or_else(|| panic!("{did:08X} missing"));
        assert!(
            script.period.is_finite() && script.period > 0.0,
            "{did:08X} period"
        );
        for effect in &script.effects {
            assert!(effect.time.is_finite(), "{did:08X} effect time");
        }
        effects += script.effects.len();
        impulses += script.impulses.len();
    }
    // The script counts are checked against the DAT, not against retail.
    assert!(effects + impulses > 0, "some FX script carries triggers");
}
