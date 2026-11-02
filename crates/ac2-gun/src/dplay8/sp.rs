//! The TCP/IP service provider's UDP socket, merged into dpnet.dll in DirectX 9.0b (there is no
//! separate dpnwsock), behind traits so the protocol can be driven by a fake peer.
//!
//! DX9 opens one overlapped Winsock socket per local binding (0x00439727, dpnet9.dll.c:51667-51699):
//! `SO_BROADCAST`, `SO_RCVBUF` only if configured, `SO_SNDBUF` = `DAT_00455114` (0), the
//! non-blocking `WSAIoctl`, `SO_EXCLUSIVEADDRUSE` = 1, then `bind` through `BindLocalPort`
//! (0x00437f16). Receives are overlapped completions and sends `WSASendTo`; the port replaces
//! both with a non-blocking `std` socket that the pump polls (see the crate docs).

use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

use crate::Hresult;

use super::hresult::{DPNERR_ALREADYINITIALIZED, DPNERR_NOCONNECTION};

/// The first and last port of the scan `SpWsock_BindLocalPort` (dpnwsock 0x00407f7a) tries when
/// no port was given: `iVar3 = 0x8fe` while `(ushort)iVar3 < 0x961`, so 2302 through 2400.
pub const BIND_SCAN_FIRST: u16 = 2302;
pub const BIND_SCAN_LAST: u16 = 2400;

/// One bound, non-blocking UDP socket.
pub trait DatagramSocket {
    /// `WSASendTo` / `sendto`.
    fn send_to(&mut self, data: &[u8], to: SocketAddrV4) -> io::Result<usize>;
    /// One pending datagram, or `None` when the socket would block.
    fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<Option<(usize, SocketAddrV4)>>;
    /// `getsockname`.
    fn local_addr(&self) -> io::Result<SocketAddrV4>;
}

/// Creates a socket bound to `ip:port` (port 0 = let the OS choose), with the options of DX9
/// 0x00439727:51667-51699 applied: `SO_BROADCAST`, `SO_SNDBUF`, and, when `exclusive`,
/// `SO_EXCLUSIVEADDRUSE` (level 0xffff, option 0xfffffffb).
pub trait SocketOpener {
    fn bind(
        &mut self,
        ip: Ipv4Addr,
        port: u16,
        exclusive: bool,
    ) -> io::Result<Box<dyn DatagramSocket>>;
}

/// The adapters the SP enumerates (`EnumAdapters`, DX9 `FUN_004047f3` / `FUN_00412b22`), first
/// IPv4 address of each, in the SP's order. `towards` is the host being connected to, for
/// implementations that can only learn the routed adapter.
pub trait AdapterEnumerator {
    fn adapters(&self, towards: Ipv4Addr) -> Vec<Ipv4Addr>;
}

/// The local host name (`gethostname`); `std` has no call for it, so the environment and the
/// usual files stand in.
fn local_host_name() -> Option<String> {
    for var in ["COMPUTERNAME", "HOSTNAME"] {
        if let Ok(v) = std::env::var(var) {
            if !v.trim().is_empty() {
                return Some(v.trim().to_owned());
            }
        }
    }
    for f in ["/proc/sys/kernel/hostname", "/etc/hostname"] {
        if let Ok(v) = std::fs::read_to_string(f) {
            if !v.trim().is_empty() {
                return Some(v.trim().to_owned());
            }
        }
    }
    None
}

/// The OS implementation. DX9's list is `gethostbyname(gethostname())` order with 192.168.0.1
/// (the Windows ICS address) moved to the front (`FUN_00433934`, dpnet9.dll.c:46233-46290;
/// `GetAdaptersInfo` only names the adapters). That order is reproduced from the local host
/// name's IPv4 addresses (loopback and unspecified ones dropped).
// open: if the host name does not resolve to any usable address (common on Linux, where it maps
// to 127.0.1.1), the adapter that routes to the host (a connected UDP socket's local address;
// nothing is sent) stands for the list: the same address on a single-homed machine.
pub struct StdAdapters;

impl AdapterEnumerator for StdAdapters {
    fn adapters(&self, towards: Ipv4Addr) -> Vec<Ipv4Addr> {
        use std::net::ToSocketAddrs;
        let mut list: Vec<Ipv4Addr> = Vec::new();
        if let Some(name) = local_host_name() {
            if let Ok(it) = (name.as_str(), 0u16).to_socket_addrs() {
                for a in it {
                    if let SocketAddr::V4(a) = a {
                        let ip = *a.ip();
                        if !ip.is_loopback() && !ip.is_unspecified() && !list.contains(&ip) {
                            list.push(ip);
                        }
                    }
                }
            }
        }
        if list.is_empty() {
            let probe = || -> io::Result<Ipv4Addr> {
                let s = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
                s.connect((towards, 9))?;
                match s.local_addr()? {
                    SocketAddr::V4(a) => Ok(*a.ip()),
                    SocketAddr::V6(_) => Err(io::Error::from(io::ErrorKind::Unsupported)),
                }
            };
            list.extend(probe().ok().filter(|i| !i.is_unspecified()));
        }
        order_adapters(list)
    }
}

/// 192.168.0.1 first (0x0100a8c0, FUN_00433934:46757), the rest in resolver order.
pub fn order_adapters(mut list: Vec<Ipv4Addr>) -> Vec<Ipv4Addr> {
    let ics = Ipv4Addr::new(192, 168, 0, 1);
    if let Some(i) = list.iter().position(|a| *a == ics) {
        let a = list.remove(i);
        list.insert(0, a);
    }
    list
}

/// Where the 2302..2400 search starts (DX9 FUN_00439727:51615-51645): 2302 (`BaseDPlayPort`),
/// unless `sp+0x58` is set, which `FUN_00433934` (.c:46331-46343) does only for a host with at
/// least two addresses of which at least one is private and one public; then `2302 + 49` = 2351
/// when the adapter address being bound is private (10/8, 172.16/12, 192.168/16), 2375 otherwise.
pub fn range_start(ip: Ipv4Addr, multi_homed: bool) -> u16 {
    if !multi_homed {
        BIND_SCAN_FIRST
    } else if ip.is_private() {
        BIND_SCAN_FIRST + 49
    } else {
        BIND_SCAN_FIRST + 49 + 24
    }
}

/// `sp+0x58` (FUN_00433934:46331-46343): two or more addresses, one private and one public.
pub fn is_multi_homed(addrs: &[Ipv4Addr]) -> bool {
    addrs.len() >= 2
        && addrs.iter().any(|a| a.is_private())
        && addrs.iter().any(|a| !a.is_private())
}

/// `BindLocalPort` (DX9 0x00437f16, dpnet9.dll.c:50271-50315):
/// * a non-zero requested port (GunDll passes its probed port) is bound exactly, once, with one
///   retry without `SO_EXCLUSIVEADDRUSE` on `WSAEACCES`; any other failure is `0x80158080`;
/// * port 0 searches 2302..=2400 from [`range_start`], moving on at `WSAEADDRINUSE`, clearing
///   the exclusive option once at `WSAEACCES`, failing with `0x80158480` for any other error;
///   if all are busy the original (port 0) address is bound, failure `0x80158480`.
pub fn bind_local_port(
    opener: &mut dyn SocketOpener,
    ip: Ipv4Addr,
    port: u16,
    multi_homed: bool,
) -> Result<Box<dyn DatagramSocket>, Hresult> {
    let mut exclusive = true;
    if port != 0 {
        return match opener.bind(ip, port, exclusive) {
            Ok(s) => Ok(s),
            Err(e) if e.kind() == io::ErrorKind::PermissionDenied => opener
                .bind(ip, port, false)
                .map_err(|_| DPNERR_ALREADYINITIALIZED),
            Err(_) => Err(DPNERR_ALREADYINITIALIZED),
        };
    }
    let start = range_start(ip, multi_homed);
    // asm 0x438004-0x4380d1: past the last port the loop just ends; there is no wrap to 2302
    // (ports below `start` are never tried), and the port-0 bind below follows.
    for p in start..=BIND_SCAN_LAST {
        loop {
            match opener.bind(ip, p, exclusive) {
                Ok(s) => return Ok(s),
                Err(e) if e.kind() == io::ErrorKind::AddrInUse => break,
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied && exclusive => {
                    exclusive = false;
                }
                // a second WSAEACCES (exclusive already cleared) moves to the next port
                // (asm 0x437fd5 -> 0x4380a0)
                Err(e) if e.kind() == io::ErrorKind::PermissionDenied => break,
                Err(_) => return Err(DPNERR_NOCONNECTION),
            }
        }
    }
    opener
        .bind(ip, 0, exclusive)
        .map_err(|_| DPNERR_NOCONNECTION)
}

/// The real socket: `std::net::UdpSocket`, non-blocking, `SO_BROADCAST`, `SO_SNDBUF` = 0.
pub struct StdSocketOpener;

struct StdSocket {
    sock: UdpSocket,
}

impl SocketOpener for StdSocketOpener {
    fn bind(
        &mut self,
        ip: Ipv4Addr,
        port: u16,
        _exclusive: bool,
    ) -> io::Result<Box<dyn DatagramSocket>> {
        // SO_EXCLUSIVEADDRUSE is Winsock-only (Unix binds are exclusive without SO_REUSEADDR).
        let sock = UdpSocket::bind(SocketAddrV4::new(ip, port))?;
        sock.set_broadcast(true)?;
        set_sndbuf_zero(&sock);
        sock.set_nonblocking(true)?;
        Ok(Box::new(StdSocket { sock }))
    }
}

/// `setsockopt(SOL_SOCKET, SO_SNDBUF, 0)` (dpnwsock.dll.c:7799-7802; a failure there only logs
/// `WSAGetLastError`, so it is ignored here too).
#[cfg(unix)]
fn set_sndbuf_zero(sock: &UdpSocket) {
    use std::os::fd::AsRawFd;
    let zero: libc::c_int = 0;
    // SAFETY: the fd is open for the life of `sock`; `zero` outlives the call and the length
    // matches its type.
    unsafe {
        libc::setsockopt(
            sock.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            std::ptr::addr_of!(zero).cast(),
            std::mem::size_of::<libc::c_int>() as libc::socklen_t,
        );
    }
}

#[cfg(not(unix))]
fn set_sndbuf_zero(_sock: &UdpSocket) {
    // open: SO_SNDBUF = 0 needs a Winsock call that std does not expose; the OS default stays.
}

impl DatagramSocket for StdSocket {
    fn send_to(&mut self, data: &[u8], to: SocketAddrV4) -> io::Result<usize> {
        self.sock.send_to(data, to)
    }

    fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<Option<(usize, SocketAddrV4)>> {
        match self.sock.recv_from(buf) {
            Ok((n, SocketAddr::V4(a))) => Ok(Some((n, a))),
            Ok(_) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn local_addr(&self) -> io::Result<SocketAddrV4> {
        match self.sock.local_addr()? {
            SocketAddr::V4(a) => Ok(a),
            SocketAddr::V6(_) => Err(io::Error::from(io::ErrorKind::Unsupported)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct Scripted {
        busy: Vec<u16>,
        denied: Vec<(u16, bool)>,
        hard_error_at: Option<u16>,
        tried: Rc<RefCell<Vec<(u16, bool)>>>,
    }
    struct Null;
    impl DatagramSocket for Null {
        fn send_to(&mut self, d: &[u8], _: SocketAddrV4) -> io::Result<usize> {
            Ok(d.len())
        }
        fn recv_from(&mut self, _: &mut [u8]) -> io::Result<Option<(usize, SocketAddrV4)>> {
            Ok(None)
        }
        fn local_addr(&self) -> io::Result<SocketAddrV4> {
            Ok(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
        }
    }
    impl SocketOpener for Scripted {
        fn bind(
            &mut self,
            _ip: Ipv4Addr,
            port: u16,
            ex: bool,
        ) -> io::Result<Box<dyn DatagramSocket>> {
            self.tried.borrow_mut().push((port, ex));
            if self.hard_error_at == Some(port) {
                return Err(io::Error::from(io::ErrorKind::Other));
            }
            if self.denied.contains(&(port, ex)) {
                return Err(io::Error::from(io::ErrorKind::PermissionDenied));
            }
            if self.busy.contains(&port) {
                return Err(io::Error::from(io::ErrorKind::AddrInUse));
            }
            Ok(Box::new(Null))
        }
    }
    fn sc(busy: Vec<u16>) -> (Scripted, Rc<RefCell<Vec<(u16, bool)>>>) {
        let tried = Rc::new(RefCell::new(vec![]));
        (
            Scripted {
                busy,
                denied: vec![],
                hard_error_at: None,
                tried: tried.clone(),
            },
            tried,
        )
    }
    const PRIV: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 5);
    const PUB: Ipv4Addr = Ipv4Addr::new(8, 8, 4, 4);

    #[test]
    fn the_ics_address_goes_first() {
        let a = Ipv4Addr::new(10, 0, 0, 2);
        let ics = Ipv4Addr::new(192, 168, 0, 1);
        assert_eq!(order_adapters(vec![a, PUB, ics]), vec![ics, a, PUB]);
        assert_eq!(order_adapters(vec![a, PUB]), vec![a, PUB]);
    }

    #[test]
    fn range_starts_at_2302_and_at_2351_or_2375_only_for_a_private_and_public_host() {
        let (mut o, t) = sc(vec![]);
        bind_local_port(&mut o, PRIV, 0, false).unwrap();
        assert_eq!(*t.borrow(), vec![(2302, true)]);
        assert!(!is_multi_homed(&[PRIV]));
        assert!(!is_multi_homed(&[PRIV, Ipv4Addr::new(10, 0, 0, 1)]));
        assert!(is_multi_homed(&[PRIV, PUB]));
        let (mut o, t) = sc(vec![]);
        bind_local_port(&mut o, PRIV, 0, true).unwrap();
        assert_eq!(*t.borrow(), vec![(2351, true)]);
        let (mut o, t) = sc(vec![2375]);
        bind_local_port(&mut o, PUB, 0, true).unwrap();
        assert_eq!(*t.borrow(), vec![(2375, true), (2376, true)]);
    }

    #[test]
    fn an_explicit_port_is_bound_exactly_with_one_retry_without_exclusive_use() {
        let (mut o, t) = sc(vec![]);
        o.denied = vec![(5000, true)];
        bind_local_port(&mut o, PRIV, 5000, false).unwrap();
        assert_eq!(*t.borrow(), vec![(5000, true), (5000, false)]);
        // in use: no search, 0x80158080
        let (mut o, t) = sc(vec![5000]);
        assert_eq!(
            bind_local_port(&mut o, PRIV, 5000, false).err(),
            Some(Hresult(0x8015_8080))
        );
        assert_eq!(t.borrow().len(), 1);
    }

    #[test]
    fn all_busy_falls_back_to_the_os_port_after_the_range_without_wrapping() {
        let (mut o, t) = sc((2302..=2400).collect());
        o.busy.retain(|p| *p != 0);
        assert!(bind_local_port(&mut o, PRIV, 0, false).is_ok());
        let t = t.borrow();
        // 2302..=2400 (99 ports), no wrap, then the port-0 bind
        assert_eq!(t.len(), 100);
        assert_eq!(t[99].0, 0);
    }

    #[test]
    fn other_errors_stop_the_search_with_0x80158480() {
        let (mut o, _) = sc(vec![2302]);
        o.hard_error_at = Some(2303);
        assert_eq!(
            bind_local_port(&mut o, PRIV, 0, false).err(),
            Some(Hresult(0x8015_8480))
        );
    }

    #[test]
    fn std_socket_binds_and_reports_a_port() {
        let mut o = StdSocketOpener;
        let s = o.bind(Ipv4Addr::LOCALHOST, 0, true).unwrap();
        assert_ne!(s.local_addr().unwrap().port(), 0);
    }
}
