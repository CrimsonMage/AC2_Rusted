//! Real-data test support: one mounted dataset per test process.

use std::sync::{Arc, OnceLock};

use crate::{DatDataset, DatOpenOptions};

/// Mounts the DATs at `AC2_DAT_DIR` once per process and hands every test the same `Arc`, so a
/// test binary's parallel tests share one set of file handles and B-tree node caches. `None` when
/// the variable is unset, which callers treat as "skip".
pub fn env_dataset() -> Option<Arc<DatDataset>> {
    static DATASET: OnceLock<Option<Arc<DatDataset>>> = OnceLock::new();
    DATASET
        .get_or_init(|| {
            let root = std::env::var_os("AC2_DAT_DIR")?;
            Some(DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount"))
        })
        .clone()
}

/// The mounted DATs only when they are the Legions DVD's release, for tests built on its records
/// (the `*_7160` scenarios); `None` otherwise, which callers treat as "skip".
pub fn env_legions_dataset() -> Option<Arc<DatDataset>> {
    let dataset = env_dataset()?;
    (ac2_core::data_release() == ac2_core::DataRelease::Legions).then_some(dataset)
}

/// A count pinned per release: the Legions DVD's or the 7230 set's (the mounted DATs' release).
pub fn pinned<T>(legions: T, release_7230: T) -> T {
    match ac2_core::data_release() {
        ac2_core::DataRelease::Legions => legions,
        ac2_core::DataRelease::Release7230 => release_7230,
    }
}
