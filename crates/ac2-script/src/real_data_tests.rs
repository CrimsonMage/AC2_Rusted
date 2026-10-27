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
    let mut mismatched = Vec::new();
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
            // Through the VM's run-time form and back out byte for byte.
            let back = crate::wire::to_net(refs).unwrap_or_else(|e| panic!("{did:08x}: {e:?}"));
            if back.to_bytes() != state.package_state {
                mismatched.push(*did);
            }
        }
    }
    eprintln!(
        "{current} WStates decode to the last byte, {legacy} use the older layout; \
         {stale} hold an instance whose slot count differs from the library's package size \
         (retail's HeapObject::UnPack rejects those)"
    );
    eprintln!("native objects: {natives:?}");
    // One graph repacks differently, as retail's would: its NRHash keys were packed with
    // `m_bUseCache` 0, and `ReferenceIDWrapper::StreamPack` sets it to 1 on unpack.
    assert_eq!(mismatched, vec![0x7000_1ABD]);
    assert_eq!(current + legacy, ids.len());
    assert_eq!(legacy, 1638);
}

fn wire_name(native: &ac2_net::protocol::NativeObject) -> &'static str {
    use ac2_net::protocol::NativeObject as W;
    match native {
        W::ExaminationRequest(_) => "ExaminationRequest (wire)",
        W::ExaminationProfile(_) => "ExaminationProfile (wire)",
        W::GameplayOptionsProfile(_) => "GameplayOptionsProfile (wire)",
        W::GmRaceSexInfo(_) => "gmRaceSexInfo (wire)",
        W::GmSceneInfo(_) => "gmSceneInfo (wire)",
        W::SkillUiNode(_) => "SkillUINode (wire)",
        W::ShortcutInfo(_) => "ShortcutInfo (wire)",
        W::UiSaveLocations(_) => "UISaveLocations (wire)",
        W::SelectionInfo(_) => "SelectionInfo (wire)",
        W::Empty(_) => "adaptor (wire)",
        _ => "other (wire)",
    }
}

fn native_name(native: &NativeObject) -> &'static str {
    match native {
        NativeObject::List(_) => "list",
        NativeObject::Hash(_) => "hash",
        NativeObject::StringInfo(_) => "StringInfo",
        NativeObject::EntityDesc(_) => "EntityDesc",
        NativeObject::WString(_) => "wpstring",
        NativeObject::Vector(_) => "Vector",
        NativeObject::Heading(_) => "Heading",
        NativeObject::CharGenResult(_) => "gmCharGenResult",
        NativeObject::VisualDesc(_) => "VisualDesc",
        NativeObject::AppliedKey(_) => "AppliedAppearanceKey",
        NativeObject::IconDesc(_) => "IconDesc",
        NativeObject::AppearanceInfo(_) => "AppearanceInfo",
        NativeObject::Position(_) => "Position",
        NativeObject::MissileParameters(_) => "MissileParameters",
        NativeObject::CollisionParameters(_) => "CollisionParameters",
        NativeObject::CollisionProfile(_) => "CollisionProfile",
        NativeObject::CollideProperty(_) => "CollideProperty",
        NativeObject::ObjCollisionProfile(_) => "ObjCollisionProfile",
        NativeObject::Plane(_) => "Plane",
        NativeObject::BehaviorParams(_) => "BehaviorParams",
        NativeObject::RandomSelectionTable(_) => "RandomSelectionTable",
        NativeObject::GameplayOptions(_) => "GameplayOptionsProfile",
        NativeObject::DataIds(_) => "QualifiedDataIDArray",
        NativeObject::Fellow(_)
        | NativeObject::Fellowship(_)
        | NativeObject::AllegianceData(_)
        | NativeObject::AllegianceProfile(_) => "adaptor",
        NativeObject::Shop(_)
        | NativeObject::ShopSale(_)
        | NativeObject::CraftSkill(_)
        | NativeObject::DamageText(_)
        | NativeObject::Color(_) => "UI native",
        NativeObject::Shortcut(_) => "ShortcutInfo",
        NativeObject::Recipe(_) => "RecipeAdaptor",
        NativeObject::Ingredient(_) => "IngredientAdaptor",
        NativeObject::SceneInfo(_) => "gmSceneInfo",
        NativeObject::Keyframe(_) => "gmKeyframe",
        NativeObject::SceneList(_) => "gmSceneInfoList",
        NativeObject::TurbineUserStatus(_) => "TurbineUserStatus",
        NativeObject::ListIter { .. } => "list iterator",
        NativeObject::HashIter { .. } => "hash iterator",
        NativeObject::MultiHash(_) => "multi-hash",
        NativeObject::MultiMap(_) => "multi-map",
        NativeObject::PackBuffer(_) => "PackBuffer",
        NativeObject::Wire(w) => wire_name(w),
        NativeObject::MultiIter { .. } => "multi iterator",
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

/// Cache-only data never reads: preloaded tables answer, a string table that is not resident is
/// requested (and the efun told to wait), and after a worker's `load` it answers.
#[test]
fn cache_only_data_requests_what_is_not_resident() {
    use crate::data::{DatScriptData, MASTER_MAP_DID};
    use crate::vm::{DataRequest, ScriptData};
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset.clone());
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    data.set_cache_only(true);
    let worker = data.clone();
    assert!(data.enum_id_map(MASTER_MAP_DID).is_some());
    assert!(!data.take_missed());
    let table = 0x2500_0002;
    assert!(data.string_table(table).is_none());
    assert!(data.take_missed());
    let requests = data.take_requests();
    assert_eq!(requests, vec![DataRequest::StringTable(table)]);
    for request in requests {
        worker.load(request);
    }
    assert!(data.string_table(table).is_some());
    assert!(!data.take_missed());
    // A record the DATs lack is noted absent and not requested again.
    worker.load(DataRequest::StringTable(0x2500_FFFF));
    assert!(data.string_table(0x2500_FFFF).is_none());
    assert!(!data.take_missed());
}

/// `PlacesTable_UniqueDBID` names 0x0E000004, which the Legions DATs do not hold: retail's
/// `PlacesTable::InqPlace` reports the missing data and answers 0, and so does the port.
#[test]
fn the_legions_dats_hold_no_places_table() {
    use crate::data::{DatScriptData, PLACES_TABLE_ENUM, UNIQUEDB_GROUP_ENUM};
    use crate::vm::ScriptData;
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset.clone());
    let data = DatScriptData::new(dataset.clone(), store);
    let did = data.enum_did(PLACES_TABLE_ENUM, UNIQUEDB_GROUP_ENUM);
    assert_eq!(did, Some(0x0E00_0004));
    assert_eq!(
        dataset
            .ids_in_range(DatFileKind::Portal, 0x0E00_0004, 0x0E00_0004)
            .count(),
        0
    );
    assert!(data.places_table().is_none());
}

/// `ScriptData::render_string` renders through ac2-db's meta-language renderer, with the
/// `StringState` resident after `preload_tables`: the INIT_PLAYER welcome line renders.
#[test]
fn script_strings_render_through_the_meta_language() {
    use crate::data::DatScriptData;
    use crate::native::StringInfo;
    use crate::vm::ScriptData;
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset.clone());
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let info = StringInfo {
        string_id: 0x7365_1CBE,
        table_id: 0x2500_0002,
        ..Default::default()
    };
    let table = data.string_table(info.table_id).expect("table");
    let text = data.render_string(&table, &info).expect("renders");
    assert!(!text.is_empty());
    eprintln!("{}", String::from_utf16_lossy(&text));
}

/// The human's visual description (`0x1F000023`, the parent the creation screens and the double
/// use) read from the DATs: cache-only it is a request first, then the record's appearance; its
/// parent chain and its part groups come out as the record has them.
#[test]
fn a_dat_visual_description_is_an_appearance_once_requested() {
    use crate::data::DatScriptData;
    use crate::vm::{DataRequest, ScriptData};
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset.clone());
    let data = DatScriptData::new(dataset, store);
    data.set_cache_only(true);
    let worker = data.clone();
    let human = 0x1F00_0023;
    assert!(data.visual_desc(human).is_none());
    assert_eq!(data.take_requests(), vec![DataRequest::VisualDesc(human)]);
    worker.load(DataRequest::VisualDesc(human));
    let visual = data.visual_desc(human).expect("the human visual");
    // The tree is built from the groups' parent keys: a root, and every held child names a group.
    assert!(visual.root.is_some() && !visual.part_groups.is_empty());
    for (parent, children) in &visual.children {
        assert!(visual.group(*parent).is_some());
        assert!(children.iter().all(|c| visual.group(*c).is_some()));
    }
    eprintln!(
        "human visual: parent {:x} scale {:?} groups {} global tables {}",
        visual.parent,
        visual.scale,
        visual.part_groups.len(),
        visual.global.appearance.len()
    );
}

/// Every qualities record of the DATs decodes as a weenie's qualities, and a class's record
/// is the defaults `CBaseQualities::m_defaults` reads (`DBObj::Get` of the record at its DID).
#[test]
fn a_qualities_record_is_the_class_defaults() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x8000_0000, 0x81FF_FFFE)
        .collect();
    assert!(!ids.is_empty(), "the DATs hold qualities records");
    let store = ResourceStore::new(dataset.clone());
    let data = crate::data::DatScriptData::new(dataset, store);
    let defaults = crate::vm::ScriptData::class_defaults(&data, ids[0]).expect("the record");
    assert!(!defaults.ints.is_empty() || !defaults.floats.is_empty());
    assert!(crate::vm::ScriptData::class_defaults(&data, 0x8000_0000).is_none());
}
