//! The server address from the `host` and `port` command line arguments.
//!
//! `Turbine::Client::EvaluateCommandLineArg` (Legions 0x006A1EB0) stores `host` as the client's
//! host string and `port` (`strtol`, base 0) at `Client+0x50`, which starts as 7304 (0x1C88).
//! `Turbine::Client::Connect` (0x006A0AE0) hands both to `ClientNet::Init` (0x004BD5A0), whose
//! `ClientNet::InitAddresses` (0x004BD090) cuts the host at the first `,` and then at the first
//! `:`, sets `sin_family` to `AF_INET` and `sin_port` to `htons(port)`, and resolves the host
//! with `NetUtils::GetSinAddr` (0x005F0170): `inet_addr` first, then `gethostbyname`, taking the
//! first IPv4 address of the answer. Retail is IPv4 only.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};

use crate::NetError;

/// `Client+0x50`'s initial value: the port retail contacts when `port` is not given.
pub const DEFAULT_PORT: u16 = 0x1C88;

/// The `port` argument (`ArgumentParser::EvaluateCommandLineArg`, type 0x22: `strtol(text, 0,
/// 0)`, so `0x` hex and leading-zero octal are read). The usage text says `<1-65535>`.
pub fn parse_port(text: &str) -> Result<u16, NetError> {
    parse_c_number(text)
        .and_then(|n| u16::try_from(n).ok())
        .filter(|&port| port != 0)
        .ok_or_else(|| NetError::Io(format!("port must be 1-65535, got {text:?}")))
}

/// `strtoul(text, 0, 0)` that must consume the whole text.
pub fn parse_c_number(text: &str) -> Option<u32> {
    let text = text.trim();
    let (digits, radix) = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(hex) => (hex, 16),
        None if text.len() > 1 && text.starts_with('0') => (&text[1..], 8),
        None => (text, 10),
    };
    if digits.is_empty() || !digits.bytes().all(|b| char::from(b).is_digit(radix)) {
        return None;
    }
    u32::from_str_radix(digits, radix).ok()
}

/// WinSock `inet_addr`: one to four numbers separated by dots, the last one filling the
/// remaining bytes of the address. `255.255.255.255` is also its error value, so retail sent
/// that text on to `gethostbyname`; it does the same here.
fn inet_addr(text: &str) -> Option<Ipv4Addr> {
    let parts: Vec<u32> = text.split('.').map(parse_c_number).collect::<Option<_>>()?;
    let (last, leading) = parts.split_last()?;
    if leading.len() > 3 || leading.iter().any(|&p| p > 0xFF) {
        return None;
    }
    let room = 8 * (4 - leading.len() as u32);
    if room < 32 && *last >= 1 << room {
        return None;
    }
    let mut address = *last;
    for (index, part) in leading.iter().enumerate() {
        address |= part << (24 - 8 * index);
    }
    (address != u32::MAX).then(|| Ipv4Addr::from(address))
}

/// The host as `ClientNet::InitAddresses` reads it: cut at the first `,`, then the first `:`.
fn host_part(host: &str) -> &str {
    let host = host.split(',').next().unwrap_or_default();
    host.split(':').next().unwrap_or_default()
}

/// `NetUtils::GetSinAddr` and the `sockaddr_in` of `ClientNet::InitAddresses`: the server a
/// `host` (an IPv4 literal or a DNS name) and `port` name. Only the DNS path touches the network.
pub fn resolve_server(host: &str, port: u16) -> Result<SocketAddrV4, NetError> {
    let host = host_part(host);
    if let Some(address) = inet_addr(host) {
        return Ok(SocketAddrV4::new(address, port));
    }
    if host.is_empty() {
        return Err(NetError::Io("empty server host".into()));
    }
    (host, port)
        .to_socket_addrs()
        .map_err(|e| NetError::Io(format!("cannot resolve {host}: {e}")))?
        .find_map(|address| match address {
            SocketAddr::V4(v4) => Some(v4),
            SocketAddr::V6(_) => None,
        })
        .ok_or_else(|| NetError::Io(format!("{host} has no IPv4 address")))
}

#[cfg(test)]
mod tests {
    use super::{parse_port, resolve_server, DEFAULT_PORT};
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn the_default_port_is_retails() {
        assert_eq!(DEFAULT_PORT, 7304);
    }

    #[test]
    fn ip_literals_resolve_without_the_network() {
        let expected = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 20), 8610);
        assert_eq!(resolve_server("192.168.1.20", 8610), Ok(expected));
        // Cut at ',' and ':' as `ClientNet::InitAddresses` does.
        assert_eq!(resolve_server("192.168.1.20:9999", 8610), Ok(expected));
        assert_eq!(resolve_server("192.168.1.20,x", 8610), Ok(expected));
    }

    #[test]
    fn inet_addr_short_forms() {
        let at = |text: &str| resolve_server(text, 1).map(|a| *a.ip());
        assert_eq!(at("127.1"), Ok(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(at("0x7f.0.0.1"), Ok(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(at("10.0.0.0377"), Ok(Ipv4Addr::new(10, 0, 0, 255)));
        assert_eq!(at("0x7f000001"), Ok(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn port_is_read_like_strtol_base_zero() {
        assert_eq!(parse_port("8610"), Ok(8610));
        assert_eq!(parse_port("0x2182"), Ok(0x2182));
        assert_eq!(parse_port("020642"), Ok(8610));
        assert!(parse_port("0").is_err());
        assert!(parse_port("65536").is_err());
        assert!(parse_port("eight").is_err());
        assert!(parse_port("").is_err());
    }
}
