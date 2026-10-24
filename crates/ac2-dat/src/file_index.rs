use std::sync::Arc;

use crate::btree::{collect_file_entries, find_file_entry};
use crate::entry::{DatFileKind, DatRawFileEntry};
use crate::error::DatResult;
use crate::file::DatFileStorage;
use crate::index::DatFlatIndex;
use crate::iteration::RETAIL_DDD_ITERATION_DID;
use crate::read_telemetry::DatReadTelemetry;

pub(crate) fn build_startup(
    storage: &DatFileStorage,
    kind: DatFileKind,
    telemetry: Option<Arc<DatReadTelemetry>>,
) -> DatResult<DatFlatIndex> {
    let mut entries = Vec::<DatRawFileEntry>::new();
    let raw = match storage.file_btree.as_deref() {
        Some(runtime) => runtime.find_file_entry(RETAIL_DDD_ITERATION_DID)?,
        None => find_file_entry(
            storage
                .mmap
                .as_ref()
                .expect("mmap storage required")
                .bytes(),
            storage.header,
            storage.mmap.as_ref().expect("mmap storage required").path(),
            RETAIL_DDD_ITERATION_DID,
        )?,
    };
    if let Some(raw) = raw {
        entries.push(raw);
    }
    Ok(DatFlatIndex::build_with_telemetry(kind, entries, telemetry))
}

pub(crate) fn build_full(
    storage: &DatFileStorage,
    kind: DatFileKind,
    telemetry: Option<Arc<DatReadTelemetry>>,
) -> DatResult<DatFlatIndex> {
    let entries = match storage.file_btree.as_deref() {
        Some(runtime) => runtime.collect_file_entries()?,
        None => collect_file_entries(
            storage
                .mmap
                .as_ref()
                .expect("mmap storage required")
                .bytes(),
            storage.header,
            storage.mmap.as_ref().expect("mmap storage required").path(),
        )?,
    };
    Ok(DatFlatIndex::build_with_telemetry(kind, entries, telemetry))
}
