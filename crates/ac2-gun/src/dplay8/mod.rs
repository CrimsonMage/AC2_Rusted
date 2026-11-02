//! The client side of Microsoft DirectPlay 8 as DirectX 9.0b ships it (`dpnet.dll` 5.3.0000.900,
//! which merged the Winsock service provider and the address object), ported from the decompile
//! in `decompiled-pdb/dplay9/` (delta spec `spec_client_delta.md`, whose verification
//! `verify_spec_client_delta.md` and lead adjudication supersede it) on top of the DX 8.0 port in
//! `decompiled-pdb/dplay8/`; GunDll requires the DX9 thread pool, so DX9 is the behaviour that
//! matters. Addresses cited are DX9's (`dpnet9.dll.c` lines) unless a comment says 8.0, which is
//! cited only where the behaviour is the same and the 8.0 function was the one read.
//!
//! Layers, bottom up:
//! * [`address`]: `IDirectPlay8Address` as the TCP/IP provider uses it (components, URL,
//!   host-name resolution);
//! * [`sp`]: the provider's UDP socket and local-port choice, behind [`sp::DatagramSocket`];
//! * [`protocol`]: DPL8R, the reliable protocol (CFRAME/DFRAME/SACK, windows, timers, pacing,
//!   keep-alive, graceful disconnect), a state machine fed datagrams and a clock;
//! * [`core`] and [`connect_info`]: the session layer (0xC1/0xC2/0xC3, 0xC5, 0xDF, 0xE0/0xE1);
//! * [`client`]: `IDirectPlay8Client` for the subset GunDll calls, and the DX9 thread pool's
//!   `DoWork` as an explicit pump; [`messages`] are the `DPN_MSGID_*` structs it hands the
//!   handler.
//!
//! # Time and sockets
//! The clock ([`clock::Clock`], `timeGetTime`) and the UDP socket ([`sp::SocketOpener`]) are
//! traits; [`clock::FakeClock`] and the fakes in the tests drive everything deterministically.
//! No threads and no async runtime: nothing happens except inside
//! [`client::Dplay8Client::do_work`], which receives, ticks the timer wheel, runs the ready
//! callbacks, sends, and calls the message handler on the caller's thread (GunDll set the thread
//! pool to zero workers and calls `DoWork` from `DispatchQueue`).
//!
//! # Differences forced by the platform
//! * Retail receives with overlapped `WSARecvFrom` on an I/O completion port and sends with
//!   `WSASendTo`, on completion and timer threads; here a non-blocking `std` socket is polled
//!   by the pump. Datagrams are therefore handled when the pump runs, not when they arrive, and
//!   the timer wheel ticks when the pump calls it, not on the pool's 4 ms waitable timer
//!   ([`protocol::timers::Timers::period`] is the fixed 4 ms of DX9 `FUN_0042a742`; with zero
//!   worker threads retail runs the timers inside `DoWork` too): drive the pump every 4 ms for
//!   retail timing; a slower pump delays expiry by at most its period. A received datagram and its replies
//!   are handled with one `timeGetTime()` reading per call instead of one per function.
//! * `SO_SNDBUF = 0` is set with `libc` on Unix (std has no API); `bind` retries create a new
//!   socket instead of re-binding one (an unbound failed socket carries no state).
//! * Socket options are set after the bind rather than before it (std binds on creation).
//!
//! # Signing and coalescing (DX9 only)
//! * [`protocol::signing`]: CONNECTED_SIGNED (opcode 3, `FUN_00421f01`), SHA-1 FAST/FULL
//!   signatures on HARD_DISCONNECT, SACK and DFRAME (receive checks and our own signing), the
//!   secret rotation and its key derivation. The client never asks for signing and cannot refuse
//!   it: the host decides. Byte order: signatures are the first 8 SHA-1 digest bytes in digest
//!   order, secrets are their 8 raw bytes; the FULL tests use SHA-1 values computed outside the
//!   crate.
//! * Receive coalescing: [`protocol::frames::decode_coalesced`]; send-side rules and encoder:
//!   [`protocol::frames::encode_coalesced`] (not fed by the pump: GunDll's flags 0x9 never
//!   coalesce and the default `ProtocolCoalesceMode` 0 only coalesces public flag 0x100).
//!
//! # Deviations
//! * Platform: see above (pump instead of threads, socket options after bind, `libc`
//!   `SO_SNDBUF`).
//! * Registry knobs are constants (no registry): see [`knobs`].
//! * The tuning-on congestion code is the 8.0 one (`credit`, `loss_events`); DX9 has a different
//!   loss history (`+0x44/+0x48`). Never reached with Initialize flag 4 (GunDll's first attempt).
//!
//! # DX9 behaviour kept as documented, not as gaps
//! * `DoWork` returns `S_OK` once the pump drained where DX9 returns `DPNSUCCESS_PENDING` while
//!   work remains: `CGun` ignores the success value (verified: no behaviour change).
//! * `Send` does not tell "no host player" (0x80158420) from "no connection" (0x80158160): GunDll
//!   maps every Send failure to 0xC1120405, so the distinction is unreachable.
//! * Initialize flag 1 (no parameter validation) and flag 2 (LAN hint, initial window 32 with
//!   tuning on) are modelled; GunDll passes 4, then 0.
//!
//! # Open items
//! Every `// open:` in this tree; the remaining ones: whether the queued END_STREAM leaves when no
//! `DoWork` follows Close, sub-header RELIABLE/SEQUENTIAL bits of coalesced frames (the DX9
//! delivery flag 0x100 has no reader), `CryptGenRandom` replaced by the OS random source, the
//! listener branch of CONNECTED_SIGNED, tuning-on congestion (8.0 code), the due-list order and
//! skew compensation of the 4 ms wheel, the adapter list when the host name does not resolve,
//! the 0xC2 name-table entries, `DPNSEND_SYNC`/`DPNCONNECT_SYNC`, send time-outs, USER_2, the
//! `DPN_CAPS_EX` values not driving the timers, and unfilled connection-info counters
//! (throughput, retried, dropped, queued per priority).
//!
pub mod address;
pub mod client;
pub mod clock;
pub mod connect_info;
pub mod core;
pub mod hresult;
pub mod knobs;
pub mod messages;
pub mod protocol;
pub mod sp;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod tests_core;
#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_signed;
