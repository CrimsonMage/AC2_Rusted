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
