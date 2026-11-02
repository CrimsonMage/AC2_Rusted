//! The `DPNERR_*` / `DPNSUCCESS_*` codes the client path uses, from the DX8 SDK header
//! `third_party/directx8/include/dplay8.h` (lines 1102-1176: `MAKE_DPNHRESULT(x)` is
//! `0x8015_0000 | x`, `_DPNHRESULT_BASE` is 0). Each code is also the literal the dpnet
//! decompile compares or returns; the address of one such use is given where the code is not
//! obvious from its name.

use crate::Hresult;

/// `DPNSUCCESS_PENDING` (dplay8.h:1104, `0x0015800E`): the async call was accepted
/// (dpnet 0x00409809 returns it from Connect, 0x0040a0c8 from Send).
pub const DPNSUCCESS_PENDING: Hresult = Hresult(0x0015_800E);
/// `DPNERR_ALREADYCLOSING` 0x50 (Connect while the object is closing, 0x00409809).
pub const DPNERR_ALREADYCLOSING: Hresult = Hresult(0x8015_8050);
/// `DPNERR_ALREADYCONNECTED` 0x60 (Connect when connected, 0x00409809).
pub const DPNERR_ALREADYCONNECTED: Hresult = Hresult(0x8015_8060);
/// `DPNERR_ALREADYINITIALIZED` 0x80 (Initialize twice, 0x0040968a-area; Initialize at
/// dpnet.dll.c:7146-7160).
pub const DPNERR_ALREADYINITIALIZED: Hresult = Hresult(0x8015_8080);
/// `DPNERR_CONNECTING` 0x150 (Connect while a connect is under way, 0x00409809).
pub const DPNERR_CONNECTING: Hresult = Hresult(0x8015_8150);
/// `DPNERR_CONNECTIONLOST` 0x160: what the core reports when the protocol connection ends
/// (dpnet `Dpl8r_TearDownConnection` 0x0041f690 calls the core's connection-lost slot with it).
pub const DPNERR_CONNECTIONLOST: Hresult = Hresult(0x8015_8160);
/// `DPNERR_HOSTREJECTEDCONNECTION` 0x260 (0xC5 result codes, spec 3.3).
pub const DPNERR_HOSTREJECTEDCONNECTION: Hresult = Hresult(0x8015_8260);
/// `DPNERR_HOSTTERMINATEDSESSION` 0x270 (`DnCore_OnTerminateSession` 0x00403de6).
pub const DPNERR_HOSTTERMINATEDSESSION: Hresult = Hresult(0x8015_8270);
/// `DPNERR_INCOMPLETEADDRESS` 0x280 (dpnaddr 0x004038df: no `hostname`).
pub const DPNERR_INCOMPLETEADDRESS: Hresult = Hresult(0x8015_8280);
/// `DPNERR_INVALIDDEVICEADDRESS` 0x320.
pub const DPNERR_INVALIDDEVICEADDRESS: Hresult = Hresult(0x8015_8320);
/// `DPNERR_INVALIDFLAGS` 0x340 (Initialize with a flag other than bit 0: dpnet.dll.c:7146).
pub const DPNERR_INVALIDFLAGS: Hresult = Hresult(0x8015_8340);
/// `DPNERR_INVALIDHANDLE` 0x360.
pub const DPNERR_INVALIDHANDLE: Hresult = Hresult(0x8015_8360);
/// `DPNERR_INVALIDHOSTADDRESS` 0x370.
pub const DPNERR_INVALIDHOSTADDRESS: Hresult = Hresult(0x8015_8370);
/// `DPNERR_INVALIDPARAM` (`E_INVALIDARG`).
pub const DPNERR_INVALIDPARAM: Hresult = Hresult(0x8007_0057);
/// `DPNERR_INVALIDPOINTER` (`E_POINTER`).
pub const DPNERR_INVALIDPOINTER: Hresult = Hresult(0x8000_4003);
/// `DPNERR_NOCONNECTION` 0x480 (a Send with no connection; GunDll's TERMINATE map reads it).
pub const DPNERR_NOCONNECTION: Hresult = Hresult(0x8015_8480);
/// `DPNERR_NORESPONSE` 0x510: the plain connect-retry exhaustion code (`Dpl8r_ConnectRetryTimer`
/// 0x0041ecd0 starts from `-0x7fea7af0`).
pub const DPNERR_NORESPONSE: Hresult = Hresult(0x8015_8510);
/// `DPNERR_TIMEDOUT` 0x630 (connect timed out with flag 8, 0x0041ecd0).
pub const DPNERR_TIMEDOUT: Hresult = Hresult(0x8015_8630);
/// `DPNERR_UNINITIALIZED` 0x640.
pub const DPNERR_UNINITIALIZED: Hresult = Hresult(0x8015_8640);
/// `DPNERR_USERCANCEL` 0x650 (connect cancelled, 0x0041ecd0 / 0x0041eaf0).
pub const DPNERR_USERCANCEL: Hresult = Hresult(0x8015_8650);
/// `DPNERR_ABORTED` 0x30: a connect torn down before it completed
/// (`Dpl8r_TearDownConnection` 0x0041f690 passes `-0x7fea7fd0`).
pub const DPNERR_ABORTED: Hresult = Hresult(0x8015_8030);
/// `DPNERR_OUTOFMEMORY` (`E_OUTOFMEMORY`).
pub const DPNERR_OUTOFMEMORY: Hresult = Hresult(0x8007_000E);
/// `DPNERR_GENERIC` (`E_FAIL`): the 0xC5 result a host sends for an unspecified failure.
pub const DPNERR_GENERIC: Hresult = Hresult(0x8000_4005);
/// `DPNERR_INVALIDAPPLICATION` 0x300 (0xC5, spec 3.3).
pub const DPNERR_INVALIDAPPLICATION: Hresult = Hresult(0x8015_8300);
/// `DPNERR_INVALIDINSTANCE` 0x380 (0xC5).
pub const DPNERR_INVALIDINSTANCE: Hresult = Hresult(0x8015_8380);
/// `DPNERR_INVALIDINTERFACE` 0x390 (0xC5).
pub const DPNERR_INVALIDINTERFACE: Hresult = Hresult(0x8015_8390);
/// `DPNERR_INVALIDPASSWORD` 0x410 (0xC5).
pub const DPNERR_INVALIDPASSWORD: Hresult = Hresult(0x8015_8410);
/// `DPNERR_INVALIDVERSION` 0x460 (0xC5).
pub const DPNERR_INVALIDVERSION: Hresult = Hresult(0x8015_8460);
/// `DPNERR_NOTHOST` 0x530 (0xC5).
pub const DPNERR_NOTHOST: Hresult = Hresult(0x8015_8530);
/// `DPNERR_ADDRESSING` 0x40.
pub const DPNERR_ADDRESSING: Hresult = Hresult(0x8015_8040);
