//! The iteration lists `CLCache::OnServerInterrogation` answers a DAT interrogation with.
//!
//! `CLCache::Init` opens five DAT slots in a fixed order: portal, the `local_<language>` DAT,
//! the region's cell DAT, `highres.dat` and `internal.dat`. For each slot that opened
//! (`DiskConBase::initialized_fm`), the response lists `DiskConBase::GetDatFileID` (the header's
//! data set in the high word, its subset in the low) with the iteration set
//! `DiskController::LoadIterationList` reads from record `0xFFFF0001`. A slot whose history
//! cannot be read makes the client send no response at all.

use ac2_dat::{DatDataset, DatFileKind, RETAIL_DDD_ITERATION_DID};
use ac2_net::protocol::read_int_set;

/// `CLCache::m_DatFiles` order (`internal.dat`, slot 4, is a development file the retail
/// install does not ship and `ac2-dat` does not open).
const SLOT_ORDER: [DatFileKind; 4] = [
    DatFileKind::Portal,
    DatFileKind::Local,
    DatFileKind::Cell,
    DatFileKind::HighRes,
];

/// `DiskConBase::GetDatFileID`.
pub fn dat_file_id(data_set: u32, subset: u32) -> u64 {
    u64::from(data_set) << 32 | u64::from(subset)
}

/// Each open DAT's file id and iterations, in slot order.
pub fn dat_iterations(dataset: &DatDataset) -> Result<Vec<(u64, Vec<i32>)>, String> {
    let mut lists = Vec::new();
    for kind in SLOT_ORDER {
        let Some(meta) = dataset.file_meta(kind) else {
            continue;
        };
        let id = dat_file_id(meta.dataset_type, meta.subset);
        let history = dataset
            .read_packed(kind, RETAIL_DDD_ITERATION_DID)
            .map_err(|e| format!("{kind:?}: {e}"))?
            .ok_or(format!("Could not get history from datfile {id:#018X}"))?;
        let iterations = read_int_set(&history.copy_to_vec())
            .map_err(|e| format!("datfile {id:#018X} history: {e}"))?;
        lists.push((id, iterations));
    }
    Ok(lists)
}

#[cfg(test)]
mod tests {
    use super::{dat_file_id, dat_iterations};
    use ac2_dat::{DatDataset, DatOpenOptions};

    #[test]
    fn the_file_id_puts_the_data_set_in_the_high_word() {
        assert_eq!(dat_file_id(3, 0x8000_0001), 0x0000_0003_8000_0001);
    }

    #[test]
    fn every_open_dat_contributes_its_history() {
        let Ok(dir) = std::env::var("AC2_DAT_DIR") else {
            return;
        };
        let dataset = DatDataset::open(DatOpenOptions::new(dir)).unwrap();
        let lists = dat_iterations(&dataset).unwrap();
        assert!(lists.len() >= 3);
        assert_eq!(lists[0].0 >> 32, 1, "portal first");
        assert!(lists.iter().all(|(_, iterations)| !iterations.is_empty()));
    }
}
