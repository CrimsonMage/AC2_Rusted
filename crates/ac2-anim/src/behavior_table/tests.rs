use super::*;

fn words(words: &[u32]) -> Vec<u8> {
    words.iter().flat_map(|w| w.to_le_bytes()).collect()
}

const G: u32 = 0x4000_0001;
const A: u32 = 0x4000_0002;
const PEACE: u32 = 0x4000_0001;
const COMBAT: u32 = 0x4100_0005;

/// Ground peace <-> ground combat, ground peace <-> air peace, air peace <-> air combat.
fn table_bytes() -> Vec<u8> {
    let mut w = vec![0x4400_0099];
    // Ramp times: ground peace -> ground combat 0.5.
    w.extend([1, G, PEACE, G, COMBAT, 0.5_f32.to_bits()]);
    // Combos: (ground, peace, 0x40000002) -> anim 7 on layer index 0.
    w.extend([1, G, PEACE, 0x4000_0002, 7, 0]);
    // Transitions: ground peace -> ground combat plays behaviour 0x40000099.
    w.extend([1, G, PEACE, G, COMBAT, 0x4000_0099, 0, 0]);
    w.extend([1, 0x8000_0001]);
    w.extend([1, 0]);
    let edges: [((u32, u32), &[(u32, u32)]); 4] = [
        ((G, PEACE), &[(G, COMBAT), (A, PEACE)]),
        ((G, COMBAT), &[(G, PEACE)]),
        ((A, PEACE), &[(A, COMBAT), (G, PEACE)]),
        ((A, COMBAT), &[(A, PEACE)]),
    ];
    w.push(edges.len() as u32);
    for (from, tos) in edges {
        w.extend([from.0, from.1, tos.len() as u32]);
        for to in tos {
            w.extend([to.0, to.1]);
        }
    }
    w.extend([4, G, PEACE, 0, G, COMBAT, 0, A, PEACE, 0, A, COMBAT, 0]);
    w.extend([1, 0x4000_0002, 0.1_f32.to_bits(), 0]);
    w.extend([2, 0x4000_0002, 0x4000_0003]);
    w.push(PEACE);
    words(&w)
}

fn table() -> Ac2BehaviorTable {
    let qdid = QualifiedDataId::new(ac2_core::DataId::new(0x4400_0099), 0);
    Ac2BehaviorTable::decode_packed(qdid, 0, &table_bytes()).expect("synthetic table")
}

#[test]
fn the_table_decodes_every_section() {
    let t = table();
    let key = Ac2ModeTransition {
        src_element: G,
        src_mode: PEACE,
        dst_element: G,
        dst_mode: COMBAT,
    };
    assert_eq!(t.mode_transition_ramp_time(&key), 0.5);
    assert_eq!(
        t.combo_info(G, PEACE, 0x4000_0002),
        Some(Ac2ComboInfo {
            anim_key: 7,
            layer: 0
        })
    );
    assert_eq!(t.combo_info(A, PEACE, 0x4000_0002), None);
    assert_eq!(
        t.transition_info(&key).map(|i| i.behavior),
        Some(0x4000_0099)
    );
    assert_eq!(t.layer_count(), 1);
    assert_eq!(t.behavior_info(0x4000_0002).ramp_time, 0.1);
    assert_eq!(t.behavior_info(0x4000_0042).ramp_time, 0.25);
    assert_eq!(t.init_behaviors, vec![0x4000_0002, 0x4000_0003]);
    assert_eq!(t.init_mode, PEACE);
}

#[test]
fn mode_paths_step_through_the_allowed_elements() {
    let t = table();
    // Ground combat -> air combat: through ground peace and air peace; the first step is ground
    // peace.
    let mut key = Ac2ModeTransition {
        src_element: G,
        src_mode: COMBAT,
        dst_element: A,
        dst_mode: COMBAT,
    };
    assert!(t.next_transition_along_mode_path(&mut key));
    assert_eq!((key.dst_element, key.dst_mode), (G, PEACE));
    // A same-element path may not leave the element: air peace -> air combat is direct.
    let mut key = Ac2ModeTransition {
        src_element: A,
        src_mode: PEACE,
        dst_element: A,
        dst_mode: COMBAT,
    };
    assert!(t.next_transition_along_mode_path(&mut key));
    assert_eq!((key.dst_element, key.dst_mode), (A, COMBAT));
    // An unknown destination has no path.
    let mut key = Ac2ModeTransition {
        src_element: A,
        src_mode: PEACE,
        dst_element: 0x4000_0003,
        dst_mode: PEACE,
    };
    assert!(!t.next_transition_along_mode_path(&mut key));
}

/// The human behaviour table and the character template decode completely (only with
/// `AC2_DAT_DIR`); every combo's layer indexes the table's layers and every init behaviour is defined.
#[test]
fn retail_behavior_tables_decode() {
    use std::sync::Arc;

    use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};
    use ac2_db::{ResourceStore, TypedResourceRef};

    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = ResourceStore::new(Arc::clone(&dataset));
    let template = store
        .load_cached_packed::<Ac2CharTemplate>(TypedResourceRef::new(
            DatFileKind::Portal,
            ac2_core::DataId::new(0x2F00_0000),
            0,
        ))
        .expect("char template decodes")
        .expect("char template present");
    let table = store
        .load_cached_packed::<Ac2BehaviorTable>(TypedResourceRef::new(
            DatFileKind::Portal,
            ac2_core::DataId::new(0x4400_0001),
            0,
        ))
        .expect("human behaviour table decodes")
        .expect("human behaviour table present");
    assert!(table
        .combos
        .values()
        .all(|combo| (combo.layer as usize) < table.layer_count()));
    assert!(table
        .init_behaviors
        .iter()
        .all(|&id| template.behavior(id).is_some()));
    assert!(table
        .modes
        .contains_key(&(template.default_element, table.init_mode)));
}
