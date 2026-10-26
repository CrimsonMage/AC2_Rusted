//! The `glsticket` command line argument's ticket hand-off.
//!
//! `Turbine::Client::EvaluateCommandLineArg` (Legions 0x006A1EB0) answers `glsticket` (a flag,
//! no value) with `RegEdit::QueryStrValue` (0x00610F50) on `HKEY_CURRENT_USER`,
//! `Software\Turbine\ac2`, value `GLSTicket`; when that fails the argument is refused (the
//! evaluation returns false, so `ArgumentParser::ParseArgs` and `Turbine::Client::ParseArgs`
//! report failure and the client does not start). Otherwise the value becomes the ticket
//! (`Client+0x3C`, the slot `glsticketdirect` fills) and `RegEdit::DeleteValue` (0x006110E0)
//! removes it: a launcher waits for that deletion as the sign the client took the ticket.
//!
//! `glsticket FILE` takes the ticket from a named file instead ([`take_from_file`]).
//!
//! Windows reads and deletes the registry value. Elsewhere the stand-in is a file named
//! `GLSTicket` in the user's AC2 configuration directory (`paths::config_file`), read and
//! removed the same way; it is refused unless it is a regular file owned by the user that no
//! one else can read. The ticket is a secret: nothing here prints it.

use std::fmt;

/// Why no ticket was taken. Never carries the ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlsTicketError {
    /// No ticket is waiting (`QueryStrValue` failed).
    Missing,
    /// The stand-in file is not a regular file owned by the user with mode 0600 or stricter.
    Insecure(&'static str),
    /// The ticket could not be read or removed.
    Io(String),
}

impl fmt::Display for GlsTicketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("no GLSTicket is waiting"),
            Self::Insecure(why) => write!(f, "GLSTicket refused: {why}"),
            Self::Io(message) => write!(f, "GLSTicket: {message}"),
        }
    }
}

impl std::error::Error for GlsTicketError {}

/// Takes the waiting ticket: reads it and deletes it, so it is read at most once.
#[cfg(windows)]
pub fn take_gls_ticket() -> Result<String, GlsTicketError> {
    windows::take()
}

/// Takes the waiting ticket: reads it and deletes it, so it is read at most once.
#[cfg(not(windows))]
pub fn take_gls_ticket() -> Result<String, GlsTicketError> {
    let dir = crate::paths::config_file(FILE_NAME)
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_default();
    take_from_dir(&dir)
}

/// The stand-in file's name.
pub const FILE_NAME: &str = "GLSTicket";

/// The stand-in for `QueryStrValue` plus `DeleteValue` on a directory (not on Windows).
#[cfg(not(windows))]
pub fn take_from_dir(dir: &std::path::Path) -> Result<String, GlsTicketError> {
    take_from_file(&dir.join(FILE_NAME))
}

/// `glsticket FILE`: reads the ticket from `path` and deletes the file, which is how a launcher
/// sees the ticket was taken. Outside Windows the file must be a regular file owned by the
/// user with mode 0600 or stricter, else it is refused and left alone. Surrounding whitespace
/// is trimmed.
pub fn take_from_file(path: &std::path::Path) -> Result<String, GlsTicketError> {
    use std::io::Read;

    let io = |e: std::io::Error| GlsTicketError::Io(e.kind().to_string());
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(GlsTicketError::Insecure("it is a symbolic link"))
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(GlsTicketError::Missing),
        Err(e) => return Err(io(e)),
    }
    let mut file = std::fs::File::open(path).map_err(io)?;
    let meta = file.metadata().map_err(io)?;
    if !meta.is_file() {
        return Err(GlsTicketError::Insecure("it is not a regular file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let dir = path.parent().filter(|d| !d.as_os_str().is_empty());
        if meta.uid() != own_uid(dir.unwrap_or(std::path::Path::new(".")))? {
            return Err(GlsTicketError::Insecure("it is not owned by the user"));
        }
        if meta.mode() & 0o077 != 0 {
            return Err(GlsTicketError::Insecure("its mode is looser than 0600"));
        }
    }
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(io)?;
    std::fs::remove_file(path).map_err(io)?;
    Ok(text.trim().to_owned())
}

/// The user's id: the owner of a file this process just made in `dir`.
#[cfg(unix)]
fn own_uid(dir: &std::path::Path) -> Result<u32, GlsTicketError> {
    use std::os::unix::fs::MetadataExt;
    let probe = dir.join(format!(".gls-probe-{}", std::process::id()));
    let made = std::fs::File::create(&probe)
        .and_then(|file| file.metadata())
        .map(|meta| meta.uid());
    let _ = std::fs::remove_file(&probe);
    made.map_err(|e| GlsTicketError::Io(e.kind().to_string()))
}

#[cfg(windows)]
mod windows {
    use super::GlsTicketError;
    use windows_sys::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueA, RegOpenKeyExA, RegQueryValueExA, HKEY, HKEY_CURRENT_USER,
        KEY_QUERY_VALUE, KEY_SET_VALUE,
    };

    const KEY: &[u8] = b"Software\\Turbine\\ac2\0";
    const VALUE: &[u8] = b"GLSTicket\0";
    /// `RegEdit::QueryStrValue`'s buffer in `EvaluateCommandLineArg` (`0xfff`).
    const CAPACITY: usize = 0xFFF;

    pub(super) fn take() -> Result<String, GlsTicketError> {
        let mut key: HKEY = std::ptr::null_mut();
        // SAFETY: NUL-terminated names; `key` receives a handle closed below.
        let status = unsafe {
            RegOpenKeyExA(
                HKEY_CURRENT_USER,
                KEY.as_ptr(),
                0,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                &mut key,
            )
        };
        if status != 0 {
            return Err(GlsTicketError::Missing);
        }
        let mut buffer = [0u8; CAPACITY + 1];
        let mut size = CAPACITY as u32;
        // SAFETY: `buffer` holds `size` bytes; the key is open.
        let status = unsafe {
            RegQueryValueExA(
                key,
                VALUE.as_ptr(),
                std::ptr::null(),
                std::ptr::null_mut(),
                buffer.as_mut_ptr(),
                &mut size,
            )
        };
        let result = if status != 0 {
            Err(GlsTicketError::Missing)
        } else {
            // SAFETY: the key is open and the name is NUL-terminated.
            let deleted = unsafe { RegDeleteValueA(key, VALUE.as_ptr()) };
            let bytes = &buffer[..(size as usize).min(CAPACITY)];
            let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
            if deleted != 0 {
                Err(GlsTicketError::Io("the value could not be deleted".into()))
            } else {
                Ok(String::from_utf8_lossy(&bytes[..end]).into_owned())
            }
        };
        // SAFETY: `key` came from `RegOpenKeyExA`.
        unsafe { RegCloseKey(key) };
        result
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::{take_from_dir, GlsTicketError, FILE_NAME};
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ac2-gls-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &std::path::Path, text: &str, mode: u32) {
        let path = dir.join(FILE_NAME);
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[test]
    fn the_ticket_is_read_once_and_deleted() {
        let dir = temp_dir("take");
        write(&dir, "secret-ticket\r\n", 0o600);
        assert_eq!(take_from_dir(&dir).unwrap(), "secret-ticket");
        assert!(!dir.join(FILE_NAME).exists());
        assert_eq!(take_from_dir(&dir), Err(GlsTicketError::Missing));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_named_file_is_taken_with_its_whitespace_trimmed() {
        let dir = temp_dir("named");
        let path = dir.join("ticket.txt");
        std::fs::write(&path, "  tk\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(super::take_from_file(&path).unwrap(), "tk");
        assert!(!path.exists());
        assert_eq!(super::take_from_file(&path), Err(GlsTicketError::Missing));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_stricter_mode_is_accepted() {
        let dir = temp_dir("strict");
        write(&dir, "t", 0o400);
        assert_eq!(take_from_dir(&dir).unwrap(), "t");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_readable_file_is_refused_and_kept() {
        for mode in [0o644, 0o640, 0o604] {
            let dir = temp_dir(&format!("loose{mode:o}"));
            write(&dir, "secret-ticket", mode);
            let error = take_from_dir(&dir).unwrap_err();
            assert!(matches!(error, GlsTicketError::Insecure(_)));
            assert!(!error.to_string().contains("secret-ticket"));
            assert!(dir.join(FILE_NAME).exists());
            let _ = std::fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn a_symlink_or_directory_is_refused() {
        let dir = temp_dir("link");
        write(&dir, "x", 0o600);
        std::fs::rename(dir.join(FILE_NAME), dir.join("real")).unwrap();
        symlink(dir.join("real"), dir.join(FILE_NAME)).unwrap();
        assert!(matches!(
            take_from_dir(&dir),
            Err(GlsTicketError::Insecure(_))
        ));
        std::fs::remove_file(dir.join(FILE_NAME)).unwrap();
        std::fs::create_dir(dir.join(FILE_NAME)).unwrap();
        assert!(matches!(
            take_from_dir(&dir),
            Err(GlsTicketError::Insecure(_))
        ));
        let _ = std::fs::remove_dir_all(dir);
    }
}
