use std::path::{Path, PathBuf};

use crate::entry::DatFileKind;

/// Returns whether a directory contains the three required DAT mount files. High-resolution
/// data is optional and therefore is not part of installation validity.
pub fn is_dat_mount_root(path: &Path) -> bool {
    [DatFileKind::Portal, DatFileKind::Cell, DatFileKind::Local]
        .into_iter()
        .all(|kind| path.join(kind.default_file_name()).is_file())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatOpenOptions {
    pub dat_directory: PathBuf,
    pub portal_dat_file_name: String,
    pub cell_dat_file_name: String,
    pub local_dat_file_name: String,
    pub high_res_dat_file_name: String,
    pub high_res_enabled: bool,
}

impl DatOpenOptions {
    pub fn new(dat_directory: impl AsRef<Path>) -> Self {
        let directory = dat_directory.as_ref();
        Self {
            dat_directory: directory.to_path_buf(),
            portal_dat_file_name: DatFileKind::Portal.default_file_name().to_string(),
            cell_dat_file_name: DatFileKind::Cell.default_file_name().to_string(),
            local_dat_file_name: DatFileKind::Local.default_file_name().to_string(),
            high_res_dat_file_name: DatFileKind::HighRes.default_file_name().to_string(),
            high_res_enabled: false,
        }
    }

    pub fn path_for(&self, kind: DatFileKind) -> PathBuf {
        let file_name = match kind {
            DatFileKind::Portal => &self.portal_dat_file_name,
            DatFileKind::Cell => &self.cell_dat_file_name,
            DatFileKind::Local => &self.local_dat_file_name,
            DatFileKind::HighRes => &self.high_res_dat_file_name,
        };
        self.dat_directory.join(file_name)
    }

    /// Builds the ordinary retail client mount set, including the optional high-resolution DAT
    /// when it is present beside the three required files.
    pub fn for_retail_install(dat_directory: impl AsRef<Path>) -> Self {
        let mut options = Self::new(dat_directory);
        options.high_res_enabled = options.path_for(DatFileKind::HighRes).is_file();
        options
    }
}

#[cfg(test)]
mod tests {
    use super::{is_dat_mount_root, DatOpenOptions};
    use crate::DatFileKind;

    #[test]
    fn mount_root_requires_portal_cell_and_local_dat_files() {
        let root = tempfile::tempdir().expect("temporary DAT root");
        for kind in [DatFileKind::Portal, DatFileKind::Cell] {
            std::fs::File::create(root.path().join(kind.default_file_name()))
                .expect("required DAT fixture");
        }
        assert!(!is_dat_mount_root(root.path()));

        std::fs::File::create(root.path().join(DatFileKind::Local.default_file_name()))
            .expect("local DAT fixture");
        assert!(is_dat_mount_root(root.path()));
    }

    #[test]
    fn retail_install_options_admit_only_an_available_optional_high_res_dat() {
        let root = tempfile::tempdir().expect("temporary DAT root");
        assert!(!DatOpenOptions::for_retail_install(root.path()).high_res_enabled);
        std::fs::File::create(root.path().join(DatFileKind::HighRes.default_file_name()))
            .expect("high-resolution DAT fixture");
        assert!(DatOpenOptions::for_retail_install(root.path()).high_res_enabled);
    }
}
