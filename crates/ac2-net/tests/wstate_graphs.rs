//! Every Legions `WState`'s reference graph (`ReferenceTable::PackReference` output) through
//! ac2-net's codec: it decodes to its last byte and packs back to the same bytes (`AC2_DAT_DIR`).

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_db::{
    ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId, TypedResourceRef,
};
use ac2_net::protocol::PackedReference;

/// `WState` DIDs (`DB_TYPE_WSTATE`).
const FIRST: u32 = 0x6F00_0000;
const LAST: u32 = 0x7FFF_FFFE;

/// A `WState` record's package state, or `None` for the older layout that ends right after it
/// (no `m_pid`; retail cannot load those).
struct PackageState(Option<Vec<u8>>);

impl ResourcePackedDecode for PackageState {
    const TYPE_NAME: &'static str = "WState package state";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(_: QualifiedDataId, _: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let word = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let n = word(4) as usize;
        let state = bytes[8..8 + n].to_vec();
        Ok(Self((bytes.len() >= 8 + n + 4).then_some(state)))
    }

    fn cache_retained_bytes(&self) -> usize {
        self.0.as_ref().map_or(0, Vec::len)
    }
}

#[test]
fn every_wstate_graph_round_trips_byte_exact() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, FIRST, LAST)
        .collect();
    let (mut checked, mut legacy) = (0, 0);
    for chunk in ids.chunks(4096) {
        let store = ResourceStore::new(dataset.clone());
        for &did in chunk {
            let record = store
                .load_cached_packed::<PackageState>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    DataId::new(did),
                    0,
                ))
                .unwrap()
                .unwrap();
            let Some(bytes) = &record.0 else {
                legacy += 1;
                continue;
            };
            let graph =
                PackedReference::from_bytes(bytes).unwrap_or_else(|e| panic!("{did:08x}: {e}"));
            assert_eq!(&graph.to_bytes(), bytes, "{did:08x}");
            checked += 1;
        }
    }
    assert!(checked > 39_000, "{checked} WStates");
    assert_eq!(legacy, 1638);
}
