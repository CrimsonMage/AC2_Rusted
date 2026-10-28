//! `CRegionDesc`'s eleven `DIDLookup` info lists (sound, scene, terrain, surface, encounter table,
//! detail map table, sky, water, fog, property, world map) and the "current" pick of each.

use ac2_core::DataId;

/// `DIDLookup`: a data id and the region index (`rindex`) it is keyed by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DidLookup {
    pub rindex: u32,
    pub data_id: DataId,
}

/// The info lists in `CRegionDesc::UnPack` stream order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoList {
    Sound,
    Scene,
    Terrain,
    Surface,
    Encounter,
    DetailMap,
    Sky,
    Water,
    Fog,
    Property,
    WorldMap,
}

impl InfoList {
    /// The lists in stream order.
    pub const ALL: [InfoList; 11] = [
        InfoList::Sound,
        InfoList::Scene,
        InfoList::Terrain,
        InfoList::Surface,
        InfoList::Encounter,
        InfoList::DetailMap,
        InfoList::Sky,
        InfoList::Water,
        InfoList::Fog,
        InfoList::Property,
        InfoList::WorldMap,
    ];
}

/// The fields of `CRegionDesc` that name other records: `game_map`, `autotest_map`, `land_defs_id`,
/// the world extents and the eleven info lists with their `current_*` indices.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegionInfo {
    pub region_number: u32,
    pub game_map: u32,
    pub autotest_map: u32,
    pub land_defs_id: u32,
    pub num_block_length: u32,
    pub num_block_width: u32,
    lists: [Vec<DidLookup>; 11],
    current: [u32; 11],
}

impl RegionInfo {
    /// A region with the given lists, every `current_*` index 0 as `CRegionDesc::CRegionDesc`
    /// leaves it.
    pub fn new(lists: [Vec<DidLookup>; 11]) -> Self {
        Self {
            lists,
            ..Self::default()
        }
    }

    /// One info list's entries.
    pub fn list(&self, list: InfoList) -> &[DidLookup] {
        &self.lists[list as usize]
    }

    /// The `current_*` index of a list.
    pub fn current_index(&self, list: InfoList) -> u32 {
        self.current[list as usize]
    }

    /// Picks the list entry `GetCurrent*Info` answers from.
    pub fn set_current_index(&mut self, list: InfoList, index: u32) {
        self.current[list as usize] = index;
    }

    /// `CRegionDesc::GetCurrentSoundInfo`, `GetCurrentSceneInfo`, `GetCurrentTerrainInfo`,
    /// `GetCurrentSurfaceInfo`, `GetCurrentDetailMapTable`, `GetCurrentSkyInfo`,
    /// `GetCurrentWaterInfo`, `GetCurrentFogInfo` and `GetCurrentPropertyInfo`: the data id at the
    /// list's current index, `None` where retail answers 0 (an empty list). An index past the end
    /// also answers `None` (retail would read past the array).
    pub fn current(&self, list: InfoList) -> Option<DataId> {
        self.list(list)
            .get(self.current_index(list) as usize)
            .map(|lookup| lookup.data_id)
    }

    /// `CRegionDesc::GetGameMap`: 0 for none.
    pub fn game_map_id(&self) -> Option<DataId> {
        (self.game_map != 0).then(|| DataId::new(self.game_map))
    }

    /// `CRegionDesc::GetSubDataIDs`: the game map, the autotest map and the land defs when set,
    /// then every entry of the eleven info lists in order.
    pub fn sub_data_ids(&self) -> Vec<DataId> {
        let mut ids = Vec::new();
        for id in [self.game_map, self.autotest_map, self.land_defs_id] {
            if id != 0 {
                ids.push(DataId::new(id));
            }
        }
        for list in InfoList::ALL {
            ids.extend(self.list(list).iter().map(|lookup| lookup.data_id));
        }
        ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(did: u32) -> DidLookup {
        DidLookup {
            rindex: 0,
            data_id: DataId::new(did),
        }
    }

    #[test]
    fn current_info_is_the_entry_at_the_current_index_or_nothing() {
        let mut lists: [Vec<DidLookup>; 11] = Default::default();
        lists[InfoList::Sky as usize] = vec![lookup(0x1500_0001), lookup(0x1500_0002)];
        let mut region = RegionInfo::new(lists);
        assert_eq!(
            region.current(InfoList::Sky),
            Some(DataId::new(0x1500_0001))
        );
        assert_eq!(region.current(InfoList::Fog), None);
        region.set_current_index(InfoList::Sky, 1);
        assert_eq!(
            region.current(InfoList::Sky),
            Some(DataId::new(0x1500_0002))
        );
        region.set_current_index(InfoList::Sky, 2);
        assert_eq!(region.current(InfoList::Sky), None);
    }

    #[test]
    fn sub_data_ids_lists_the_maps_then_every_info_entry() {
        let mut lists: [Vec<DidLookup>; 11] = Default::default();
        lists[InfoList::Sound as usize] = vec![lookup(0x1000_0001)];
        lists[InfoList::WorldMap as usize] = vec![lookup(0x2000_0009)];
        let mut region = RegionInfo::new(lists);
        region.game_map = 0x0600_0001;
        region.land_defs_id = 0x2500_0001;
        let ids: Vec<u32> = region.sub_data_ids().iter().map(|id| id.raw()).collect();
        assert_eq!(ids, [0x0600_0001, 0x2500_0001, 0x1000_0001, 0x2000_0009]);
    }
}
