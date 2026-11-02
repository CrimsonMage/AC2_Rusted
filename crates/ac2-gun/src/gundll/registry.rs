//! The Win32 registry values GunDll reads and writes, behind a trait the caller supplies (the registry
//! is not available cross-platform). All of them live under
//! `HKEY_CURRENT_USER\Software\Microsoft\Microsoft Games` (`FUN_1000b830`, `FUN_1000b8a0`,
//! `FUN_1000b900`; `0a_send_session.md` section 6).

/// The subkey (GunDll 0x10059bf8).
pub const KEY: &str = "Software\\Microsoft\\Microsoft Games";
/// `LaunchTicketData`, REG_BINARY (0x10059be4).
pub const LAUNCH_TICKET_DATA: &str = "LaunchTicketData";
/// `LaunchTicketSize`, REG_DWORD (0x10059c1c).
pub const LAUNCH_TICKET_SIZE: &str = "LaunchTicketSize";
/// `Email` (0x1005b728): read as bytes and used as a UTF-16 string by `PresentTicket`.
pub const EMAIL: &str = "Email";

/// The registry as `CGun` uses it. A real implementation maps `name` under [`KEY`].
pub trait GunRegistry {
    /// `FUN_1000b830`: `RegOpenKeyExA(KEY_QUERY_VALUE)` + `RegQueryValueExA(name, type ignored,
    /// buffer of max_len bytes)`. `None` when the key or value is missing or the data does not fit.
    fn query_value(&self, name: &str, max_len: usize) -> Option<Vec<u8>>;
    /// `FUN_1000b900`: `RegCreateKeyExA` + `RegSetValueExA(name, REG_BINARY, data)`; false on failure.
    fn set_binary(&self, name: &str, data: &[u8]) -> bool;
    /// `FUN_1000b8a0`: `RegSetValueExA(name, REG_DWORD, value)`.
    fn set_dword(&self, name: &str, value: u32) -> bool;
    /// `FUN_1000b900` with NULL data: `RegOpenKeyExA(KEY_WRITE)` + `RegDeleteValueA(name)`.
    fn delete_value(&self, name: &str) -> bool;
}
