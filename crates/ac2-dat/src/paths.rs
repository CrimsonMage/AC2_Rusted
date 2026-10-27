use std::path::{Path, PathBuf};

use crate::entry::DatFileKind;

/// Returns whether a directory contains the three required DAT mount files. High-resolution
/// data is optional and therefore is not part of installation validity.
pub fn is_dat_mount_root(path: &Path) -> bool {
    [DatFileKind::Portal, DatFileKind::Cell, DatFileKind::Local]
        .into_iter()
        .all(|kind| path.join(kind.default_file_name()).is_file())
}

/// The local DAT of a language (`CLCache::SetLanguageInternal`): retail builds `local_<name>.dat`
/// from the language's name (its `0x4000002a` enum string, which is the file's suffix on the DVD:
/// `English`, `Deutsch`, `Francais`, `Japanese`, `Korean`). The installed `local_*.dat` files in
/// `dir` answer it; the match ignores case, and a language with no file is an error that names
/// the installed ones.
pub fn local_dat_for_language(dir: &Path, language: &str) -> Result<String, String> {
    let entries = std::fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let mut installed = Vec::new();
    for entry in entries.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        let name = file
            .strip_prefix("local_")
            .and_then(|rest| rest.strip_suffix(".dat"))
            .map(str::to_owned);
        if let Some(name) = name {
            installed.push((file, name));
        }
    }
    installed
        .iter()
        .find(|(_, name)| name.eq_ignore_ascii_case(language))
        .map(|(file, _)| file.clone())
        .ok_or_else(|| {
            let mut names: Vec<&str> = installed.iter().map(|(_, n)| n.as_str()).collect();
            names.sort_unstable();
            format!(
                "no local_{language}.dat in {} (installed: {})",
                dir.display(),
                names.join(", ")
            )
        })
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
    use super::{is_dat_mount_root, local_dat_for_language, DatOpenOptions};
    use crate::DatFileKind;

    #[test]
    fn a_language_names_its_installed_local_dat() {
        let root = tempfile::tempdir().expect("temporary DAT root");
        for name in ["portal.dat", "local_English.dat", "local_Francais.dat"] {
            std::fs::File::create(root.path().join(name)).expect("DAT fixture");
        }
        assert_eq!(
            local_dat_for_language(root.path(), "Francais").as_deref(),
            Ok("local_Francais.dat")
        );
        assert_eq!(
            local_dat_for_language(root.path(), "francais").as_deref(),
            Ok("local_Francais.dat")
        );
        assert_eq!(
            local_dat_for_language(root.path(), "English").as_deref(),
            Ok("local_English.dat")
        );
        let error = local_dat_for_language(root.path(), "Deutsch").unwrap_err();
        assert!(error.contains("local_Deutsch.dat"), "{error}");
        assert!(error.contains("English, Francais"), "{error}");
    }

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
