//! Every script record in the Legions DATs against the decoders (`AC2_DAT_DIR`).

use std::collections::BTreeMap;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use crate::native::NativeObject;
use crate::reference::PackedEntry;
use crate::wlib::{WLIB_FIRST, WLIB_LAST};
use crate::wstate::{WSTATE_FIRST, WSTATE_LAST};
use crate::{load_wlib, load_wstate, LEGIONS_WLIB_DID};

#[test]
fn the_script_library_decodes_to_its_last_byte() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, WLIB_FIRST, WLIB_LAST)
        .collect();
    assert_eq!(ids, vec![LEGIONS_WLIB_DID], "Legions ships one WLib");
    let store = ResourceStore::new(dataset);
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .expect("the library");
    let s = &wlib.stream;
    eprintln!(
        "WLib {:08x}: {} opcode bytes, {} string literals, {} exports, {} package ids, \
         {} vtables, {} package names",
        wlib.did,
        s.opcodes.len(),
        s.string_literals.len(),
        s.exports.len(),
        s.vtables.func_mapper.len(),
        s.vtables.vtables.len(),
        s.vtables.package_names.len()
    );
    assert!(!s.opcodes.is_empty());
    assert_eq!(s.string_literals.len(), s.string_literal_uses.len());
    assert_eq!(s.vtables.vtables.len(), s.exports.len());
    assert_eq!(s.vtables.package_id_map.len(), s.vtables.func_mapper.len());
    assert_eq!(s.vtables.package_info.len(), s.vtables.func_mapper.len());
    // The descrambled literals are text, and the package names resolve to exported packages.
    let first = String::from_utf16_lossy(&s.string_literals[0]);
    assert!(first.is_ascii() && !first.is_empty(), "{first:?}");
    for (name, id) in &s.vtables.package_names {
        assert!(name.iter().all(|c| c.is_ascii_graphic()), "{name:?}");
        let index = s.vtables.package_id_map[*id as usize];
        assert!(
            (index as usize) < s.exports.len(),
            "package {id} -> {index}"
        );
    }
    assert!(s.package_id("CharacterGenSystem").is_some());
}

#[test]
fn every_wstate_decodes_to_its_last_byte() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, WSTATE_FIRST, WSTATE_LAST)
        .collect();
    assert!(ids.len() > 40_000, "{} WStates", ids.len());
    let library = load_wlib(
        &ResourceStore::new(dataset.clone()),
        DataId::new(LEGIONS_WLIB_DID),
    )
    .unwrap()
    .unwrap();
    let package_info = &library.stream.vtables.package_info;
    let (mut current, mut legacy, mut stale) = (0, 0, 0);
    let mut natives: BTreeMap<&'static str, usize> = BTreeMap::new();
    for chunk in ids.chunks(4096) {
        // A fresh store per chunk keeps the decoded states from piling up in one cache.
        let store = ResourceStore::new(dataset.clone());
        for did in chunk {
            let state = load_wstate(&store, DataId::new(*did))
                .unwrap_or_else(|e| panic!("{did:08x}: {e}"))
                .unwrap_or_else(|| panic!("{did:08x} missing"));
            let Some(_pid) = state.pid else {
                // The older layout (see `WState::pid`).
                assert!(matches!(did >> 20, 0x700 | 0x710), "{did:08x}");
                legacy += 1;
                continue;
            };
            let refs = state
                .references
                .as_ref()
                .unwrap_or_else(|e| panic!("{did:08x}: {e}"));
            current += 1;
            let mut fits = true;
            for entry in &refs.entries {
                if let PackedEntry::Heap { object, .. } = entry {
                    fits &= object.matches_library(package_info);
                    if let Some(native) = &object.native {
                        *natives.entry(native_name(native)).or_default() += 1;
                    }
                }
            }
            if !fits {
                stale += 1;
            }
        }
    }
    eprintln!(
        "{current} WStates decode to the last byte, {legacy} use the older layout; \
         {stale} hold an instance whose slot count differs from the library's package size \
         (retail's HeapObject::UnPack rejects those)"
    );
    eprintln!("native objects: {natives:?}");
    assert_eq!(current + legacy, ids.len());
    assert_eq!(legacy, 1638);
}

fn native_name(native: &NativeObject) -> &'static str {
    match native {
        NativeObject::List(_) => "list",
        NativeObject::Hash(_) => "hash",
        NativeObject::StringInfo(_) => "StringInfo",
        NativeObject::WString(_) => "wpstring",
        NativeObject::Vector(_) => "Vector",
        NativeObject::Position(_) => "Position",
        NativeObject::MissileParameters(_) => "MissileParameters",
        NativeObject::BehaviorParams(_) => "BehaviorParams",
        NativeObject::RandomSelectionTable(_) => "RandomSelectionTable",
        NativeObject::ExaminationRequest(_) => "ExaminationRequest",
        NativeObject::GameplayOptionsProfile(_) => "GameplayOptionsProfile",
        NativeObject::GmRaceSexInfo(_) => "gmRaceSexInfo",
        NativeObject::GmSceneInfo(_) => "gmSceneInfo",
        NativeObject::SkillUiNode(_) => "SkillUINode",
        NativeObject::Unbound(_) => "unbound",
    }
}

#[test]
fn every_enum_table_decodes_and_the_master_map_names_the_mapper_group() {
    use crate::data::{DatScriptData, EnumIdMap, EnumMapper, EMAPPER_GROUP, MASTER_MAP_DID};
    use crate::vm::ScriptData;
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset.clone());
    let load = |did: u32| ac2_db::TypedResourceRef::new(DatFileKind::Portal, DataId::new(did), 0);
    let mappers: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x2300_0000, 0x23FF_FFFE)
        .collect();
    let id_maps: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x2800_0000, 0x28FF_FFFE)
        .collect();
    for &did in &mappers {
        let m = store
            .load_cached_packed::<EnumMapper>(load(did))
            .unwrap()
            .unwrap();
        assert!(
            m.names.iter().all(|(_, n)| n.iter().all(u8::is_ascii)),
            "{did:08x}"
        );
    }
    for &did in &id_maps {
        store
            .load_cached_packed::<EnumIdMap>(load(did))
            .unwrap()
            .unwrap();
    }
    eprintln!(
        "{} enum mappers, {} DID mappers",
        mappers.len(),
        id_maps.len()
    );
    assert_eq!((mappers.len(), id_maps.len()), (87, 18));
    let data = DatScriptData::new(dataset, store);
    let master = data.enum_id_map(MASTER_MAP_DID).unwrap();
    assert_eq!(master.name(EMAPPER_GROUP), Some(&b"EMAPPER"[..]));
    // Every mapper id the EMAPPER group lists resolves to a mapper or DID mapper.
    let group = data
        .enum_id_map(master.enum_to_did(EMAPPER_GROUP).unwrap())
        .unwrap();
    for (mapid, _) in &group.to_id {
        let did = data.enum_did(*mapid, EMAPPER_GROUP).unwrap();
        assert!(
            matches!(data.db_type(did), Some(0x4000_0028 | 0x4000_002A)),
            "{mapid:08x} -> {did:08x}"
        );
    }
}
