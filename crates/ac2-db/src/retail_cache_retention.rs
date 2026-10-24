//! Cache retention per decode family: how many **free** decoded entries the typed cache keeps
//! once every external `Arc` and residency lease has gone away. These are entry counts, not byte
//! budgets.

use crate::ResourceTypeId;

/// Idle time a free decoded resource remains eligible for reuse before the host cache use-time
/// pass may reclaim it.
pub const RETAIL_CACHE_IDLE_GRACE_SECONDS: u64 = 30;

/// Ideal and maximum free-entry counts of one decode family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetailCacheRetention {
    ideal_free_entries: usize,
    max_free_entries: usize,
}

impl RetailCacheRetention {
    pub const fn new(ideal_free_entries: usize, max_free_entries: usize) -> Self {
        Self {
            ideal_free_entries,
            max_free_entries,
        }
    }

    pub const fn ideal_free_entries(self) -> usize {
        self.ideal_free_entries
    }

    pub const fn max_free_entries(self) -> usize {
        self.max_free_entries
    }
}

const DEFAULT: RetailCacheRetention = RetailCacheRetention::new(16, 16);
/// Landblock height records are revisited constantly while the viewer moves.
const LAND_BLOCK: RetailCacheRetention = RetailCacheRetention::new(21, 144);

/// Retention of the decode family keyed by a raw type id.
pub const fn retail_cache_retention_for_type(type_id: u32) -> RetailCacheRetention {
    if type_id == ResourceTypeId::LandBlock as u32 {
        LAND_BLOCK
    } else {
        DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::{retail_cache_retention_for_type, RETAIL_CACHE_IDLE_GRACE_SECONDS};

    #[test]
    fn landblocks_keep_more_free_entries_than_other_families() {
        let land = retail_cache_retention_for_type(1);
        assert_eq!(
            (land.ideal_free_entries(), land.max_free_entries()),
            (21, 144)
        );
        let other = retail_cache_retention_for_type(0);
        assert_eq!(
            (other.ideal_free_entries(), other.max_free_entries()),
            (16, 16)
        );
        assert_eq!(RETAIL_CACHE_IDLE_GRACE_SECONDS, 30);
    }
}
