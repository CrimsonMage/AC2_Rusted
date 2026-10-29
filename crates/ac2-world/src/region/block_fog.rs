//! The region's fog areas as `LScapeLighting::GetFogCorners` reads them: the current `CFogDesc`
//! (`CRegionDesc::GetCurrentFogInfo`) and its `BlockMap` of fog types, one per landblock corner.

use std::sync::Arc;

use ac2_db::{ResourceResult, ResourceStore};

use super::ac2_region::{ac2_region_fog_desc, load};
use super::{FogDesc, LightFogInfo};
use crate::ac2_water::{load_ac2_block_map, Ac2BlockMap};

/// The current fog descriptor and its fog type map, loaded with the region.
#[derive(Debug, Clone)]
pub struct RegionFog {
    desc: Arc<FogDesc>,
    map: Ac2BlockMap,
}

/// The fog and light of a landblock's four corners (`m_swBlockFog`, `m_seBlockFog`,
/// `m_nwBlockFog`, `m_neBlockFog`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockFogCorners {
    pub sw: LightFogInfo,
    pub se: LightFogInfo,
    pub nw: LightFogInfo,
    pub ne: LightFogInfo,
}

impl RegionFog {
    /// The region's current fog descriptor and its map; `None` when either is missing (retail's
    /// `m_bHasFog` stays 0).
    pub(crate) fn load(store: &ResourceStore) -> ResourceResult<Option<Self>> {
        let Some(did) = ac2_region_fog_desc(store)? else {
            return Ok(None);
        };
        let Some(desc) = load::<FogDesc>(store, did)? else {
            return Ok(None);
        };
        // `GetFogCorners` reads the map the descriptor names first (its distance fog map).
        let Some(map) = load_ac2_block_map(store, desc.distance_fog_map)? else {
            return Ok(None);
        };
        Ok(Some(Self {
            desc: Arc::new((*desc).clone()),
            map,
        }))
    }

    /// `LScapeLighting::GetFogCorners` for the landblock at block coordinates `(x, y)`: the fog
    /// type at each corner (the map at `(x, y)`, `(x, y + 1)`, `(x + 1, y + 1)` and `(x + 1, y)`),
    /// or `None` when none of the four has fog.
    pub fn corners(&self, x: u32, y: u32) -> Option<BlockFogCorners> {
        let [sw, nw, ne, se] = [(x, y), (x, y + 1), (x + 1, y + 1), (x + 1, y)]
            .map(|(cx, cy)| u32::from(self.map.value(cx, cy)));
        if ![sw, nw, ne, se]
            .iter()
            .any(|index| self.desc.has_fog(*index))
        {
            return None;
        }
        let fogging = |index| self.desc.fogging(index).unwrap_or_default();
        Some(BlockFogCorners {
            sw: fogging(sw),
            se: fogging(se),
            nw: fogging(nw),
            ne: fogging(ne),
        })
    }
}
