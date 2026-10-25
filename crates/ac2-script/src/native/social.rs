//! The fellowship and allegiance adaptors scripts fill for the UI (`FellowAdaptor` 0x93,
//! `FellowshipAdaptor` 0x94, `AllegianceDataAdaptor` 0x82, `AllegianceProfileAdaptor` 0x83),
//! kept as ac2-weenie's plain types. Their hashes are ordered as retail iterates them when a
//! notice hands them on: the fellows (`HashTable<InstanceID, FellowAdaptor>`, 47 buckets) and
//! the officers (`HashSet<InstanceID>`, `IntrusiveHashTable(10)`: 11 buckets), each element at
//! its bucket's head, a repeated id refused (`HashTable::add`).

use ac2_net::protocol::StringInfo;
use ac2_weenie::{AllegianceMember, AllegianceProfile, FellowInfo, FellowshipInfo};

use super::hash_table::{bucket_size_index, BUCKET_SIZES, DEFAULT_BUCKETS};

/// `INVALID_IID`.
pub const INVALID_IID: u64 = u64::MAX;

/// `AllegianceDataAdaptor::AllegianceDataAdaptor`.
pub fn allegiance_member() -> AllegianceMember {
    AllegianceMember {
        id: INVALID_IID,
        name: StringInfo::default(),
        title: StringInfo::default(),
        ..AllegianceMember::default()
    }
}

/// `AllegianceProfileAdaptor::AllegianceProfileAdaptor`.
pub fn allegiance_profile() -> AllegianceProfile {
    AllegianceProfile {
        monarch: allegiance_member(),
        patron: allegiance_member(),
        member: allegiance_member(),
        ..AllegianceProfile::default()
    }
}

/// `FellowshipAdaptor::FellowshipAdaptor`.
pub fn fellowship() -> FellowshipInfo {
    FellowshipInfo {
        leader: INVALID_IID,
        ..FellowshipInfo::default()
    }
}

/// `FellowAdaptor::FellowAdaptor`.
pub fn fellow() -> FellowInfo {
    FellowInfo::default()
}

/// Retail's iteration order of `ids` added in order to an `IntrusiveHashTable` of `buckets`
/// buckets (`key % buckets`, `_aullrem`), the first of a repeated id kept.
fn hash_order(ids: &[u64], buckets: u32) -> Vec<usize> {
    let mut table: Vec<Vec<usize>> = vec![Vec::new(); buckets as usize];
    for (i, id) in ids.iter().enumerate() {
        let b = (id % u64::from(buckets)) as usize;
        if !table[b].iter().any(|&j| ids[j] == *id) {
            table[b].insert(0, i);
        }
    }
    table.into_iter().flatten().collect()
}

/// `FellowshipAdaptor::AddFellow`: false when the fellow is there already.
pub fn add_fellow(f: &mut FellowshipInfo, id: u64, info: FellowInfo) -> bool {
    if f.fellows.iter().any(|(k, _)| *k == id) {
        return false;
    }
    f.fellows.push((id, info));
    true
}

/// The fellowship with its fellows in table order (what the UI iterates).
pub fn fellowship_ordered(f: &FellowshipInfo) -> FellowshipInfo {
    let ids: Vec<u64> = f.fellows.iter().map(|(id, _)| *id).collect();
    let fellows = hash_order(&ids, DEFAULT_BUCKETS)
        .into_iter()
        .map(|i| f.fellows[i].clone())
        .collect();
    FellowshipInfo {
        fellows,
        ..f.clone()
    }
}

/// The profile with its officers in `HashSet` order (11 buckets).
pub fn allegiance_ordered(p: &AllegianceProfile) -> AllegianceProfile {
    let buckets = BUCKET_SIZES[bucket_size_index(10)];
    let officers = hash_order(&p.officers, buckets)
        .into_iter()
        .map(|i| p.officers[i])
        .collect();
    AllegianceProfile {
        officers,
        ..p.clone()
    }
}
