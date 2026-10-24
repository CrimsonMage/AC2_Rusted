use std::fs::File;
use std::path::{Path, PathBuf};

use memmap2::Mmap;
#[cfg(unix)]
use memmap2::UncheckedAdvice;

use crate::error::{DatError, DatResult};

#[derive(Debug)]
pub(crate) struct DatMmapFile {
    _file: File,
    map: Mmap,
    path: PathBuf,
}

impl DatMmapFile {
    pub(crate) fn open(path: impl AsRef<Path>) -> DatResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => DatError::MissingDatFile {
                kind: crate::entry::DatFileKind::Portal,
                path: path.clone(),
            },
            _ => DatError::Io(error),
        })?;
        let map = unsafe { Mmap::map(&file)? };
        Ok(Self {
            _file: file,
            map,
            path,
        })
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.map
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn release_backing_pages(&self) -> DatResult<()> {
        #[cfg(unix)]
        {
            // SAFETY: this immutable file-backed map is backed by a read-only descriptor for its
            // lifetime. MADV_DONTNEED preserves the mapping and later reads refault identical bytes.
            unsafe { self.map.unchecked_advise(UncheckedAdvice::DontNeed) }.map_err(DatError::Io)
        }
        #[cfg(not(unix))]
        {
            // Windows exposes no equivalent portable `memmap2` advisory operation. Keeping the
            // immutable mapping intact preserves all DAT read semantics.
            Ok(())
        }
    }
}
