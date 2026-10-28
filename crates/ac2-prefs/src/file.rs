//! `PrefsFile`: the client's preferences file as one owned value. It holds the
//! [`PreferenceBank`] (the preferences the subsystems register), the [`Preferences`] cache (the
//! values read by key), the name table the file is read and written with, and the file's path.
//!
//! The frame thread never touches the file: [`PrefsFile::load`] runs before the frame loop, and
//! [`PrefsFile::save`] builds the bytes on the caller and writes them on a worker thread. Which
//! subsystems register their preferences is the caller's (the client's `ClientPrefs::load`).

use std::io;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use crate::bank::PreferenceBank;
use crate::names::NameTable;
use crate::paths;
use crate::store::Preferences;

/// The client's preferences: the bank, the session cache of the other values, the names they
/// are written with, and the file they live in.
pub struct PrefsFile<N: NameTable> {
    bank: PreferenceBank,
    cache: Preferences,
    names: N,
    path: PathBuf,
}

impl<N: NameTable> PrefsFile<N> {
    /// Reads `path` (an empty file when there is none) into the bank and the cache. Nothing is
    /// registered yet; the caller registers the preferences through [`PrefsFile::parts_mut`].
    pub fn load(path: PathBuf, names: N) -> io::Result<Self> {
        let bytes = paths::read_or_empty(&path)?;
        Ok(Self {
            bank: PreferenceBank::new(&bytes),
            cache: Preferences::new(&bytes),
            names,
            path,
        })
    }

    /// The bank and the names together, for registering preferences (the names are read while
    /// the bank is written, so both are borrowed at once).
    pub fn parts_mut(&mut self) -> (&mut PreferenceBank, &N) {
        (&mut self.bank, &self.names)
    }

    pub fn bank(&self) -> &PreferenceBank {
        &self.bank
    }

    pub fn bank_mut(&mut self) -> &mut PreferenceBank {
        &mut self.bank
    }

    pub fn names(&self) -> &N {
        &self.names
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// `Preferences::InqBool`: the saved value of (`section`, `key`) as an unsigned integer, from
    /// the session's values first and then the file.
    pub fn inq_bool(&self, section: u32, key: u32) -> Option<u32> {
        self.cache.inq_bool(&self.names, section, key)
    }

    /// `PreferenceBank::Save`: every preference is written into the file image, and the image is
    /// written to the path on a worker thread (the caller does not wait for the disk).
    pub fn save(&mut self) -> JoinHandle<io::Result<()>> {
        self.bank.save(&self.names);
        self.write_file()
    }

    /// `Preferences::SetUInt32` for the session, then the value is written into the file at once
    /// (`WritePrivateProfileStringA` on the bank's and the cache's images, as
    /// `Render::SaveOverallGraphicsQuality` does). The file is written on a worker thread.
    pub fn save_uint_now(
        &mut self,
        (section, key): (u32, u32),
        (file_section, file_key): (&str, &str),
        value: u32,
    ) -> JoinHandle<io::Result<()>> {
        self.cache.set_uint32(section, key, value);
        let text = value.to_string();
        self.bank.write_file_value(file_section, file_key, &text);
        self.cache.write_file_value(file_section, file_key, &text);
        self.write_file()
    }

    /// The file image, written to the path on a worker thread.
    fn write_file(&self) -> JoinHandle<io::Result<()>> {
        let bytes = self.bank.file_bytes();
        let path = self.path.clone();
        std::thread::spawn(move || paths::write_atomic(&path, &bytes))
    }
}
