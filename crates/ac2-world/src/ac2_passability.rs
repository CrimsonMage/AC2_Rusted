//! A landblock's passability: `CLandBlockData::m_bPassable` as `CLandBlockData::CacheNearValues`
//! sets it from the region's current `PropertyDesc`.

use ac2_core::{CellId, DataId};
use ac2_dat::DatFileKind;
use ac2_db::{CachedResource, ResourceResult, ResourceStore, TypedResourceRef};

use crate::region::{ac2_region_info, InfoList, PropertyDesc};

/// The region's current property description (`CRegionDesc::GetCurrentPropertyInfo`, DB type
/// `0x40000012`), which answers `PropertyDesc::GetPassiblity` for each block.
pub(crate) struct Ac2Passability {
    desc: CachedResource<PropertyDesc>,
}

impl Ac2Passability {
    /// Loads the region's current property description; `None` when the region lists none or
    /// the record is missing (`CacheNearValues` then leaves `m_bPassable` alone).
    pub(crate) fn load(store: &ResourceStore) -> ResourceResult<Option<Self>> {
        let Some(did) = ac2_region_info(store)?.and_then(|info| info.current(InfoList::Property))
        else {
            return Ok(None);
        };
        let desc = store.load_cached_packed::<PropertyDesc>(TypedResourceRef::new(
            DatFileKind::Portal,
            did,
            0,
        ))?;
        Ok(desc.map(|desc| Self { desc }))
    }

    /// `PropertyDesc::GetPassiblity` of the block at the start of `block_cell_id`
    /// (`LandDefs::blockpart_to_blockcoord`: x is the top byte, y the next).
    pub(crate) fn block_passable(&self, store: &ResourceStore, block_cell_id: CellId) -> bool {
        let raw = block_cell_id.raw();
        let (block_x, block_y) = (raw >> 24, (raw >> 16) & 0xFF);
        let map_for = |did: DataId| crate::load_ac2_block_map(store, did).ok().flatten();
        self.desc.passability(&map_for, block_x, block_y)
    }
}

/// `CLandBlockData::m_bPassable` before `CacheNearValues` runs (the constructor sets 1), which
/// is what a region without property info keeps.
pub(crate) const DEFAULT_PASSABLE: bool = true;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_landblocks_split_into_passable_land_and_impassable_sea() {
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let store = std::sync::Arc::new(ResourceStore::new(dataset));
        let source = crate::Ac2WorldSource::open(store.clone())
            .unwrap()
            .expect("an AC2 dataset");
        let passability = Ac2Passability::load(&store)
            .unwrap()
            .expect("the region lists a property description");
        assert!(source.has_landblock(CellId::new(0x7F7F_FFFF)));
        let (mut passable, mut impassable) = (0usize, 0usize);
        for x in 0..=255u32 {
            for y in 0..=255u32 {
                let block = CellId::new((x << 24) | (y << 16) | 0xFFFF);
                if !source.has_landblock(block) {
                    continue;
                }
                let desc = source.load_landblock(block).unwrap().expect("block record");
                assert_eq!(
                    desc.is_passable(),
                    passability.block_passable(&store, block)
                );
                if desc.is_passable() {
                    passable += 1;
                } else {
                    impassable += 1;
                }
            }
        }
        eprintln!("resident-record blocks: passable {passable}, impassable {impassable}");
        // The map's land mass is passable and the sea and the unreachable rim around it is not:
        // roughly 40% of the shipped blocks are passable. A loaded block carries the same answer
        // as the property description gives for it.
        assert!(passable > 20_000, "the land mass is passable");
        assert!(impassable > 20_000, "the sea and rim are impassable");
    }
}
