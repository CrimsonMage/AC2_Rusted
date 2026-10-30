//! The AC1 DAT mount: `client_portal.dat`, `client_cell_1.dat`, `client_local_English.dat` and,
//! when present, `client_highres.dat`, opened through `ac2-dat` like the AC2 set.

use std::path::Path;
use std::sync::Arc;

use ac2_dat::{DatDataset, DatError, DatFileKind, DatOpenOptions};

use crate::formats::{Ac1Error, Ac1Result};

/// The mounted AC1 DATs.
#[derive(Debug, Clone)]
pub struct Ac1Dats {
    dataset: Arc<DatDataset>,
}

impl Ac1Dats {
    /// Mounts the AC1 DATs in `dir` (the high-resolution DAT when it is there).
    pub fn open(dir: impl AsRef<Path>) -> Result<Self, DatError> {
        let mut options = DatOpenOptions::new(dir);
        options.portal_dat_file_name = "client_portal.dat".into();
        options.cell_dat_file_name = "client_cell_1.dat".into();
        options.local_dat_file_name = "client_local_English.dat".into();
        options.high_res_dat_file_name = "client_highres.dat".into();
        options.high_res_enabled = options.path_for(DatFileKind::HighRes).is_file();
        let dataset = DatDataset::open(options)?;
        Ok(Self { dataset })
    }

    /// The AC1 DATs of `AC1_DAT_DIR`, or `None` when it is unset.
    pub fn from_env() -> Option<Result<Self, DatError>> {
        std::env::var_os(crate::AC1_DAT_DIR_ENV).map(Self::open)
    }

    /// A portal record's bytes, `None` when the DAT has no such record.
    pub(crate) fn portal(&self, did: u32) -> Ac1Result<Option<Vec<u8>>> {
        self.read(DatFileKind::Portal, did)
    }

    /// An image: the portal DAT's, else the high-resolution DAT's.
    pub(crate) fn image(&self, did: u32) -> Ac1Result<Option<Vec<u8>>> {
        match self.read(DatFileKind::Portal, did)? {
            Some(bytes) => Ok(Some(bytes)),
            None => self.read(DatFileKind::HighRes, did),
        }
    }

    fn read(&self, kind: DatFileKind, did: u32) -> Ac1Result<Option<Vec<u8>>> {
        self.dataset.read_packed(kind, did).map_err(|_| Ac1Error {
            did,
            context: "the DAT read failed",
        })
    }

    /// A portal record decoded by `decode`; a missing record is an error.
    pub(crate) fn decode<T>(
        &self,
        did: u32,
        decode: impl FnOnce(u32, &[u8]) -> Ac1Result<T>,
    ) -> Ac1Result<T> {
        let bytes = self.portal(did)?.ok_or(Ac1Error {
            did,
            context: "no such portal record",
        })?;
        decode(did, &bytes)
    }
}
