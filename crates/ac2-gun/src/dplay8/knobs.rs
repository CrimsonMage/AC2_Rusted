//! The DX9 registry protocol knobs as constants (`FUN_0042f71a`, dpnet9.dll.c:42161-42359; there
//! is no registry here, so the defaults hold). All are DWORD values under
//! `HKLM\SOFTWARE\Microsoft\DirectPlay8`; the `Protocol*` ones sit in a sub-key of that name
//! with a `Default...` value and per-executable overrides named after the exe.

/// `ProtocolSigningMode` (`DAT_00455118`, range 0..=3): 0 follow the connect flags, 1 always
/// FAST, 2 always FULL, 3 never. A Listen-only knob for a client.
pub const PROTOCOL_SIGNING_MODE: u32 = 0;
/// `ProtocolCoalesceMode` (`DAT_0045511c`, 0..=2): 0 only sends with public flag 0x100, 1 all
/// sends, 2 none (`FUN_00423f42`:30715-30722).
pub const PROTOCOL_COALESCE_MODE: u32 = 0;
/// `ProtocolLinkTuningMode` (`DAT_00455120`, 0..=2): 0 follow Initialize flag 4, 1 on, 2 off.
pub const PROTOCOL_LINK_TUNING_MODE: u32 = 0;
/// `ProtocolMaxCoalesceSize` (`DAT_004517d8`, 0x80..=0x7ff).
pub const PROTOCOL_MAX_COALESCE_SIZE: u32 = 0x7ff;
/// `MaxUserDataSize` (`DAT_004517b4`, 0x3c..=0x598): the datagram size, 1384.
pub const MAX_USER_DATA_SIZE: u32 = 1384;
/// `MaxEnumDataSize` (`DAT_004517b8`, 0x1c..=0x578).
pub const MAX_ENUM_DATA_SIZE: u32 = 1000;
/// `ExtendedFrameSize` (`DAT_00454f0c`, below 0xfa68).
pub const EXTENDED_FRAME_SIZE: u32 = 0;
/// `BaseDPlayPort` (`DAT_004517bc`, below 0xff9a) and `MaxDPlayPort` (`DAT_004517c0`, below
/// 0xffff, and above the base else base + 100): the 2302..=2400 range.
pub const BASE_DPLAY_PORT: u32 = 2302;
pub const MAX_DPLAY_PORT: u32 = 2400;
/// `ThreadCount` (`DAT_00454eb0`): 0 means 3 x the processors.
pub const THREAD_COUNT: u32 = 0;
/// `DefaultPathTestCount`/`Interval`, `NATResolverQueryCount`/`Interval`/`Timeout`
/// (`DAT_004517c4..d4`): NAT traversal, unused by the client port.
pub const DEFAULT_PATH_TEST_COUNT: u32 = 7;
pub const DEFAULT_PATH_TEST_INTERVAL: u32 = 375;
pub const NAT_RESOLVER_QUERY_COUNT: u32 = 4;
pub const NAT_RESOLVER_QUERY_INTERVAL: u32 = 1000;
pub const NAT_RESOLVER_QUERY_TIMEOUT: u32 = 1000;
/// `WinsockSendBufferSize` (`DAT_00455114`): 0; the provider sets `SO_SNDBUF` to it.
pub const WINSOCK_SEND_BUFFER_SIZE: u32 = 0;
/// `IPAddressFamilySettings` (`DAT_00454f00`: 0, 2 or 0x17) and `TraversalModeSettings`
/// (`DAT_004517ac`): 0 and 1.
pub const IP_ADDRESS_FAMILY: u32 = 0;
pub const TRAVERSAL_MODE: u32 = 1;
/// `TreatAllResponsesAsProxied` (`DAT_004517b0`): 1. Enumeration/NAT only; unused by the client.
pub const TREAT_ALL_RESPONSES_AS_PROXIED: u32 = 1;
// Knobs this port never reads, listed for completeness: the `ProtocolSigningMode` (a Listen
// knob), `WinsockReceiveBufferSize`, `DisableDPNHGatewaySupport`, `DisableDPNHFirewallSupport`,
// `DefaultNATResolver`, `BannedIPv4Addresses` and the per-exe `AppsToIgnoreEnums`,
// `AppsToDisconnectOnICMP`, `AppsWithoutGamePortDirectedEnums` flags; the protocol constants
// above are documentation, the code uses the equal literals.
