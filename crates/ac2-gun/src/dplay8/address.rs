//! Addresses and host-name resolution: `IDirectPlay8Address` as the TCP/IP provider uses it
//! (dpnaddr.dll) and the provider's own reading of it (dpnwsock.dll).
//!
//! Sources, all in `decompiled-pdb/dplay8/`:
//! * the component list, `SetSP` and `AddComponent`: dpnaddr 0x00402a02 (`FUN_00402a02`, adds a
//!   named, typed component), dpnaddr 0x0040239d (`SetSP`);
//! * `BuildAddress(hostname, port)`: dpnaddr 0x0040370e, adds the provider GUID
//!   `{EBFE7BA0-628D-11D2-AE0F-006097B01411}` (the 16 bytes at dpnaddr 0x004013a0), then
//!   `hostname` (type 1, string), then `port` (type 2, DWORD);
//! * `BuildFromSockAddr`: dpnaddr 0x004035e1, family 2 only, provider, `hostname` =
//!   `inet_ntoa`, `port` = `ntohs`, in that order;
//! * `GetURLA`: dpnaddr 0x0040296c (`FUN_0040296c`): prefix `x-directplay:/` (0x00402553), the
//!   components `name=value` joined by `;` (0x00402834), then the user data (0x0040295a);
//!   strings escape through 0x004024cf, GUIDs print as `%7B%-08.8X-%-04.4X-%-04.4X-%02.2X%02.2X-
//!   %02.2X%02.2X%02.2X%02.2X%02.2X%02.2X%7D`, DWORDs as `%d`;
//! * `GetSockAddress`: dpnaddr 0x004038df (the provider check, `hostname`, `port`) and
//!   0x00403a2a (`inet_addr`, the `"255.255.255.255"` special case, `gethostbyname`, one
//!   `sockaddr_in` per `h_addr_list` entry);
//! * the provider's use of it: 8.0 dpnwsock 0x00404860 (default port 6073, the first sockaddr
//!   only); DX 9.0b merged it into dpnet.dll: `GetSockAddress` 0x0043445a (getaddrinfo, up to 12
//!   results, `AI_NUMERICHOST` on the first pass whose failure is 0x80158630, 0x80158370 on the
//!   name pass), host mode 3 callers 0x00436d7e / 0x004372f4.

use std::net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs};

use crate::Hresult;

use super::hresult::*;

/// The TCP/IP service provider GUID `{EBFE7BA0-628D-11D2-AE0F-006097B01411}` in COM byte order
/// (data1 LE, data2 LE, data3 LE, data4 as is): the 16 bytes at dpnaddr 0x004013a0, and
/// `third_party/directx8/include/dplay8.h:61` (`CLSID_DP8SP_TCPIP`).
pub const CLSID_DP8SP_TCPIP: [u8; 16] = [
    0xA0, 0x7B, 0xFE, 0xEB, 0x8D, 0x62, 0xD2, 0x11, 0xAE, 0x0F, 0x00, 0x60, 0x97, 0xB0, 0x14, 0x11,
];

/// The port the provider inserts into a host address that has none: `0x17b9` = 6073, dpnwsock
/// 0x00404860 (the `param_2 == 3` branch writes it back as component `port`, type 2).
pub const DEFAULT_HOST_PORT: u32 = 6073;

/// One typed component value (`DPNA_DATATYPE_STRING` 1, `DPNA_DATATYPE_DWORD` 2,
/// `DPNA_DATATYPE_GUID` 3; dpnaddr 0x00402834 switches on the same numbers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
    String(String),
    Dword(u32),
    Guid([u8; 16]),
}

/// An `IDirectPlay8Address`: an ordered list of named components. `SetSP` stores the provider
/// as a component named `provider`, so it prints first when it is set first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DpAddress {
    components: Vec<(String, Component)>,
}

impl DpAddress {
    pub fn new() -> Self {
        Self::default()
    }

    /// `AddComponent` (dpnaddr 0x00402a02): a component of the same name is replaced in place,
    /// else the new one goes last.
    pub fn add_component(&mut self, name: &str, value: Component) {
        if let Some(slot) = self
            .components
            .iter_mut()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
        {
            slot.1 = value;
        } else {
            self.components.push((name.to_owned(), value));
        }
    }

    /// `IDirectPlay8Address::SetSP` (dpnaddr 0x0040239d).
    pub fn set_sp(&mut self, guid: [u8; 16]) {
        self.add_component("provider", Component::Guid(guid));
    }

    /// `GetComponentByName`.
    pub fn component(&self, name: &str) -> Option<&Component> {
        self.components
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, c)| c)
    }

    /// `IDirectPlay8AddressIP::BuildAddress` (dpnaddr 0x0040370e): provider, `hostname`,
    /// `port` (a `u16` widened to a DWORD).
    pub fn build_address(hostname: &str, port: u16) -> Self {
        let mut a = Self::new();
        a.set_sp(CLSID_DP8SP_TCPIP);
        a.add_component("hostname", Component::String(hostname.to_owned()));
        a.add_component("port", Component::Dword(u32::from(port)));
        a
    }

    /// `IDirectPlay8AddressIP::BuildFromSockAddr` (dpnaddr 0x004035e1).
    pub fn build_from_sockaddr(addr: SocketAddrV4) -> Self {
        Self::build_address(&addr.ip().to_string(), addr.port())
    }

    /// `IDirectPlay8Address::GetURLA` (dpnaddr 0x0040296c) without user data: the text GunDll's
    /// peer sees in the 0xC1 URL blob (without the terminating NUL, which the caller adds).
    pub fn to_url(&self) -> String {
        let mut s = String::from("x-directplay:/");
        for (i, (name, value)) in self.components.iter().enumerate() {
            if i > 0 {
                s.push(';');
            }
            s.push_str(name);
            s.push('=');
            match value {
                Component::String(v) => escape_url(&mut s, v),
                Component::Dword(v) => s.push_str(&(*v as i32).to_string()),
                Component::Guid(g) => push_url_guid(&mut s, g),
            }
        }
        s
    }
}

/// dpnaddr 0x00402478 (`FUN_00402478`): a character is escaped unless it is a letter, a digit,
/// or one of `- ? . , _`. The binary also compares against `0x4c2b` (not `'+'`), so `+` is
/// escaped like the rest. dpnaddr 0x004024cf prints an escaped character as `%XX` (`%25` for
/// `%`).
fn escape_url(out: &mut String, text: &str) {
    for ch in text.encode_utf16() {
        let plain = matches!(ch, 0x41..=0x5A | 0x61..=0x7A | 0x30..=0x39)
            || matches!(ch, 0x2D | 0x3F | 0x2E | 0x2C | 0x5F);
        if plain {
            out.push(char::from(ch as u8));
        } else {
            out.push_str(&format!("%{ch:02X}"));
        }
    }
}

/// The GUID format string at dpnaddr 0x004012d0: `%7B` + `%-08.8X-%-04.4X-%-04.4X-` + two bytes +
/// `-` + six bytes + `%7D`, hex upper case. The 16 bytes are the in-memory `GUID` (data1..3 little
/// endian).
fn push_url_guid(out: &mut String, g: &[u8; 16]) {
    let d1 = u32::from_le_bytes([g[0], g[1], g[2], g[3]]);
    let d2 = u16::from_le_bytes([g[4], g[5]]);
    let d3 = u16::from_le_bytes([g[6], g[7]]);
    out.push_str(&format!(
        "%7B{d1:08X}-{d2:04X}-{d3:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}%7D",
        g[8], g[9], g[10], g[11], g[12], g[13], g[14], g[15]
    ));
}

/// `inet_addr` (WSOCK32 ordinal 10, called at dpnaddr 0x00403a2a): 1 to 4 dotted parts, each
/// decimal, `0x` hex or leading-`0` octal; the last part fills the remaining bytes. `None` is
/// `INADDR_NONE` (the caller then tries the name service, except for `"255.255.255.255"`).
pub fn inet_addr(text: &str) -> Option<Ipv4Addr> {
    let parts: Vec<&str> = text.split('.').collect();
    if parts.is_empty() || parts.len() > 4 || parts.iter().any(|p| p.is_empty()) {
        return None;
    }
    let mut vals = Vec::with_capacity(4);
    for p in &parts {
        let (digits, radix) = if let Some(h) = p.strip_prefix("0x").or_else(|| p.strip_prefix("0X"))
        {
            (h, 16)
        } else if p.len() > 1 && p.starts_with('0') {
            (&p[1..], 8)
        } else {
            (*p, 10)
        };
        if digits.is_empty() {
            if radix == 16 {
                return None;
            }
            vals.push(0u64);
            continue;
        }
        vals.push(u64::from_str_radix(digits, radix).ok()?);
    }
    let n = vals.len();
    let last = vals[n - 1];
    let head_ok = vals[..n - 1].iter().all(|&v| v <= 0xFF);
    let max_last = [u64::from(u32::MAX), 0xFF_FFFF, 0xFFFF, 0xFF][n - 1];
    if !head_ok || last > max_last {
        return None;
    }
    let mut value = last;
    for (i, &v) in vals[..n - 1].iter().enumerate() {
        value |= v << (8 * (3 - i));
    }
    Some(Ipv4Addr::from(value as u32))
}

/// The name service (`gethostbyname`, WSOCK32 ordinal 52): the IPv4 addresses of a host in the
/// order the resolver returns them.
pub trait HostResolver {
    fn resolve(&self, host: &str) -> Option<Vec<Ipv4Addr>>;
}

/// The operating system's resolver through `std`. Synchronous, as `gethostbyname` is in the
/// Connect call.
pub struct SystemResolver;

impl HostResolver for SystemResolver {
    fn resolve(&self, host: &str) -> Option<Vec<Ipv4Addr>> {
        let list: Vec<Ipv4Addr> = (host, 0u16)
            .to_socket_addrs()
            .ok()?
            .filter_map(|a| match a {
                std::net::SocketAddr::V4(v4) => Some(*v4.ip()),
                std::net::SocketAddr::V6(_) => None,
            })
            .collect();
        if list.is_empty() {
            None
        } else {
            Some(list)
        }
    }
}

/// `IDirectPlay8AddressIP::GetSockAddress` (dpnaddr 0x004038df + 0x00403a2a) as the provider
/// uses it: every IPv4 socket address the address names, port in host order. Errors, as the
/// binary returns them: a provider other than TCP/IP -> `0x80158290`
/// (`DPNERR_INVALIDADDRESSFORMAT`), no `hostname` -> `DPNERR_INCOMPLETEADDRESS` (dpnaddr
/// 0x004038df), a name the resolver cannot find -> `0x80158370` (`DPNERR_INVALIDHOSTADDRESS`,
/// the `mov edi, 0x80158370` at dpnaddr 0x00403b35).
pub fn get_sock_addrs(
    addr: &DpAddress,
    resolver: &dyn HostResolver,
) -> Result<Vec<SocketAddrV4>, Hresult> {
    match addr.component("provider") {
        Some(Component::Guid(g)) if *g == CLSID_DP8SP_TCPIP => {}
        _ => return Err(Hresult(0x8015_8290)),
    }
    // DX9 0x0043445a:47169-47260: a missing hostname is 0x80158280; a hostname that is not a
    // string, or a port that is not a DWORD, is 0x80004005 (8.0 0x004038df; DX9 type-check
    // result open); a missing OR zero port is 6073.
    let host = match addr.component("hostname") {
        Some(Component::String(h)) => h,
        Some(_) => return Err(DPNERR_GENERIC),
        None => return Err(DPNERR_INCOMPLETEADDRESS),
    };
    let port = match addr.component("port") {
        Some(Component::Dword(p)) if *p != 0 => *p as u16,
        Some(Component::Dword(_)) | None => DEFAULT_HOST_PORT as u16,
        Some(_) => return Err(DPNERR_GENERIC),
    };
    // An empty hostname is the broadcast address (47194).
    if host.is_empty() {
        return Ok(vec![SocketAddrV4::new(Ipv4Addr::BROADCAST, port)]);
    }
    // dpnaddr 0x00403a2a: `inet_addr` == -1 and the text is not exactly "255.255.255.255" ->
    // `gethostbyname`; the broadcast text keeps INADDR_NONE as the address.
    if let Some(ip) = inet_addr(host) {
        return Ok(vec![SocketAddrV4::new(ip, port)]);
    }
    if host == "255.255.255.255" {
        return Ok(vec![SocketAddrV4::new(Ipv4Addr::BROADCAST, port)]);
    }
    match resolver.resolve(host) {
        Some(list) => Ok(list
            .into_iter()
            .map(|ip| SocketAddrV4::new(ip, port))
            .collect()),
        None => Err(DPNERR_INVALIDHOSTADDRESS),
    }
}

/// The provider's reading of a connect target (dpnwsock 0x00404860, branch `param_2 == 3`): a
/// host address without `port` gets `port = 6073` written into it; the socket address is the
/// FIRST one `GetSockAddress` returns (`vtbl+0xc(local_c, 0x10)` copies 16 bytes).
pub fn provider_host_sockaddr(
    host: &mut DpAddress,
    resolver: &dyn HostResolver,
) -> Result<SocketAddrV4, Hresult> {
    if matches!(host.component("port"), None | Some(Component::Dword(0))) {
        host.add_component("port", Component::Dword(DEFAULT_HOST_PORT));
    }
    let list = get_sock_addrs(host, resolver)?;
    list.into_iter().next().ok_or(DPNERR_INVALIDHOSTADDRESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(Option<Vec<Ipv4Addr>>);
    impl HostResolver for Fixed {
        fn resolve(&self, _: &str) -> Option<Vec<Ipv4Addr>> {
            self.0.clone()
        }
    }

    #[test]
    fn url_of_a_built_address_lists_provider_hostname_port() {
        // dpnaddr 0x0040370e order + 0x0040296c format.
        let a = DpAddress::build_address("1.2.3.4", 2302);
        assert_eq!(
            a.to_url(),
            "x-directplay:/provider=%7BEBFE7BA0-628D-11D2-AE0F-006097B01411%7D;hostname=1.2.3.4;port=2302"
        );
    }

    #[test]
    fn url_escapes_by_dpnaddr_rule() {
        let mut a = DpAddress::new();
        a.add_component("hostname", Component::String("a b+c%".into()));
        // space -> %20, '+' -> %2B (0x4c2b compare), '%' -> %25
        assert_eq!(a.to_url(), "x-directplay:/hostname=a%20b%2Bc%25");
    }

    #[test]
    fn inet_addr_forms() {
        assert_eq!(inet_addr("10.1.2.3"), Some(Ipv4Addr::new(10, 1, 2, 3)));
        assert_eq!(inet_addr("0x7f.1"), Some(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(inet_addr("017.0.0.1"), Some(Ipv4Addr::new(15, 0, 0, 1)));
        assert_eq!(inet_addr("1.2.3.4.5"), None);
        assert_eq!(inet_addr("1.2.3.256"), None);
        assert_eq!(inet_addr("host.example"), None);
        assert_eq!(inet_addr("255.255.255.255"), Some(Ipv4Addr::BROADCAST));
    }

    #[test]
    fn missing_port_becomes_6073_and_first_record_wins() {
        let mut a = DpAddress::new();
        a.set_sp(CLSID_DP8SP_TCPIP);
        a.add_component("hostname", Component::String("gun.example".into()));
        let r = Fixed(Some(vec![
            Ipv4Addr::new(9, 9, 9, 9),
            Ipv4Addr::new(8, 8, 8, 8),
        ]));
        let s = provider_host_sockaddr(&mut a, &r).unwrap();
        assert_eq!(s, SocketAddrV4::new(Ipv4Addr::new(9, 9, 9, 9), 6073));
        assert_eq!(a.component("port"), Some(&Component::Dword(6073)));
    }

    #[test]
    fn errors_follow_the_binary() {
        let r = Fixed(None);
        let mut no_host = DpAddress::new();
        no_host.set_sp(CLSID_DP8SP_TCPIP);
        assert_eq!(get_sock_addrs(&no_host, &r), Err(DPNERR_INCOMPLETEADDRESS));
        let mut unknown = DpAddress::build_address("nope.invalid", 1);
        assert_eq!(
            provider_host_sockaddr(&mut unknown, &r),
            Err(DPNERR_INVALIDHOSTADDRESS)
        );
        let mut wrong_sp = DpAddress::build_address("1.1.1.1", 1);
        wrong_sp.set_sp([0; 16]);
        assert_eq!(get_sock_addrs(&wrong_sp, &r), Err(Hresult(0x8015_8290)));
    }
}
