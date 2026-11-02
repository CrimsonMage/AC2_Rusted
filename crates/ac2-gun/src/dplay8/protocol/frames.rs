//! The DPL8R wire frames as dpnet 5.1.2258.400 (DirectX 8.0) builds and reads them.
//! Everything is little endian (x86 stores).
//!
//! * CFRAME (command frame), 16 bytes: `Dpl8r_BuildCFrame` (dpnet 0x004266b0).
//! * SACK, 12 + 4n bytes: `Dpl8r_BuildSack` (dpnet 0x00426880), parsed by `Dpl8r_HandleCFrame`
//!   case 6 (dpnet 0x004220e0).
//! * DFRAME (data frame), 4 + 4n header bytes: `Dpl8r_BuildDFrame` (dpnet 0x00425230),
//!   `Dpl8r_RebuildRetransmit` (dpnet 0x00425520), parsed by `Dpl8r_HandleDFrame` (dpnet
//!   0x00422340).
//! * The dispatcher `Dpl8r_DispatchFrame` (dpnet 0x00421f50) tests bit 0 of byte 0: set = DFRAME.

/// Protocol version dword in every CFRAME: major 1, minor 6 (DX 9.0b `BuildCFrame` 0x00427556
/// dpnet9.dll.c:33820, `BuildSack`-family 0x00427720:33892; 8.0 wrote 0x10000).
pub const PROTOCOL_VERSION: u32 = 0x0001_0006;

/// CFRAME `bCommand`: 0x80, POLL (0x08) added (`*pbVar5 = 0x80`, `|= 8`).
pub const CFRAME_COMMAND: u8 = 0x80;
pub const CFRAME_POLL: u8 = 0x08;

/// `bExtOpCode` values (`Dpl8r_HandleCFrame` switch, `Dpl8r_BuildCFrame` callers).
pub const OP_CONNECT: u8 = 1;
pub const OP_CONNECTED: u8 = 2;
/// CONNECTED_SIGNED, DX9 only (`FUN_004257da` 0x004257da:32207 `bVar1 == 3`, length > 0x2f).
pub const OP_CONNECTED_SIGNED: u8 = 3;
pub const OP_HARD_DISCONNECT: u8 = 4;
pub const OP_SACK: u8 = 6;

/// DFRAME `bCommand` bits (`Dpl8r_QueueUserMessage` 0x0041dcc0, `Dpl8r_SendPump` 0x00425930,
/// `Dpl8r_BuildDFrame`).
pub const CMD_DATA: u8 = 0x01;
pub const CMD_RELIABLE: u8 = 0x02;
pub const CMD_SEQUENTIAL: u8 = 0x04;
pub const CMD_POLL: u8 = 0x08;
pub const CMD_NEW_MSG: u8 = 0x10;
pub const CMD_END_MSG: u8 = 0x20;
pub const CMD_USER1: u8 = 0x40;
pub const CMD_USER2: u8 = 0x80;

/// DFRAME `bControl` bits (`Dpl8r_BuildDFrame`, `Dpl8r_HandleDFrame`).
pub const CTL_RETRY: u8 = 0x01;
pub const CTL_CORRELATE: u8 = 0x02;
/// The payload starts with 2-byte coalescing sub-headers (DX9 0x004259cf:32344).
pub const CTL_COALESCE: u8 = 0x04;
pub const CTL_END_STREAM: u8 = 0x08;
pub const CTL_SACK1: u8 = 0x10;
pub const CTL_SACK2: u8 = 0x20;
pub const CTL_SEND1: u8 = 0x40;
pub const CTL_SEND2: u8 = 0x80;

/// A CONNECT, CONNECTED or HARD_DISCONNECT frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CFrame {
    /// `bCommand` bit 0x08.
    pub poll: bool,
    pub op: u8,
    pub msg_id: u8,
    pub rsp_id: u8,
    pub version: u32,
    pub sess_id: u32,
    pub tick: u32,
}

impl CFrame {
    /// The 16 bytes `Dpl8r_BuildCFrame` writes at +0x88 of the frame object: `0x80 | poll`,
    /// op, bMsgID, bRspId, version, sessID, `timeGetTime()`.
    pub fn encode(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[0] = CFRAME_COMMAND | if self.poll { CFRAME_POLL } else { 0 };
        b[1] = self.op;
        b[2] = self.msg_id;
        b[3] = self.rsp_id;
        b[4..8].copy_from_slice(&self.version.to_le_bytes());
        b[8..12].copy_from_slice(&self.sess_id.to_le_bytes());
        b[12..16].copy_from_slice(&self.tick.to_le_bytes());
        b
    }
}

/// CONNECTED_SIGNED, 0x30 bytes (`FUN_00427720` 0x00427720:33866-33899 writes it): the CFRAME
/// header, `ullConnectSig` +0x10, `ullSenderSecret` +0x18, `ullReceiverSecret` +0x20,
/// `dwSigningOpts` +0x28 (1 FAST, 2 FULL), `dwEchoTimestamp` +0x2c.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignedFrame {
    pub base: CFrame,
    pub connect_sig: [u8; 8],
    pub sender_secret: [u8; 8],
    pub receiver_secret: [u8; 8],
    pub options: u32,
    pub echo_tick: u32,
}

impl SignedFrame {
    /// The connector's answer (`FUN_00427720`): cmd 0x80 without POLL, opcode 3, `bMsgID` the
    /// counter, **`bRspId` 0** (0x00427720:33887 `*(undefined1*)((int)puVar4+0x73) = 0`),
    /// version 0x10006, the session id of the frame, `timestamp` (the receive time passed in),
    /// the host's cookie echoed, our two secrets, the options and the host's timestamp echoed.
    pub fn answer(
        msg_id: u8,
        sess_id: u32,
        timestamp: u32,
        host: &SignedFrame,
        sender: [u8; 8],
        receiver: [u8; 8],
    ) -> [u8; 0x30] {
        let mut b = [0u8; 0x30];
        b[0] = CFRAME_COMMAND;
        b[1] = OP_CONNECTED_SIGNED;
        b[2] = msg_id;
        b[3] = 0;
        b[4..8].copy_from_slice(&PROTOCOL_VERSION.to_le_bytes());
        b[8..12].copy_from_slice(&sess_id.to_le_bytes());
        b[12..16].copy_from_slice(&timestamp.to_le_bytes());
        b[0x10..0x18].copy_from_slice(&host.connect_sig);
        b[0x18..0x20].copy_from_slice(&sender);
        b[0x20..0x28].copy_from_slice(&receiver);
        b[0x28..0x2c].copy_from_slice(&host.options.to_le_bytes());
        b[0x2c..0x30].copy_from_slice(&host.base.tick.to_le_bytes());
        b
    }
}

/// The SACK frame (`bExtOpCode` 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SackFrame {
    /// `bRetry`: the last received DFRAME had RETRY (conn byte `+0x138`).
    pub retry: u8,
    /// `bNSeq`: the next sequence the sender will use (conn byte `+0x130`).
    pub nseq: u8,
    /// `bNRcv`: the next sequence expected.
    pub nrcv: u8,
    /// Local receive tick of the last DFRAME (conn `[0x4d]`).
    pub tick: u32,
    pub sack1: u32,
    pub sack2: u32,
    pub send1: u32,
    pub send2: u32,
}

/// SACK `bFlags` bits (`Dpl8r_BuildSack`).
pub const SACK_RESPONSE: u8 = 0x01;
pub const SACK_MASK1: u8 = 0x02;
pub const SACK_MASK2: u8 = 0x04;
pub const SACK_SEND1: u8 = 0x08;
pub const SACK_SEND2: u8 = 0x10;

impl SackFrame {
    /// `Dpl8r_BuildSack` layout: bytes 0-5 as listed, bytes 6-7 zero (the builder leaves pool
    /// memory there; the port sends zeros), tick at 8, then the non-zero masks in the order
    /// SACK1, SACK2, SEND1, SEND2, each present only when its mask is non-zero.
    pub fn encode(&self) -> Vec<u8> {
        let mut flags = SACK_RESPONSE;
        let mut masks: Vec<u32> = Vec::new();
        if self.sack1 != 0 {
            flags |= SACK_MASK1;
            masks.push(self.sack1);
        }
        if self.sack2 != 0 {
            flags |= SACK_MASK2;
            masks.push(self.sack2);
        }
        if self.send1 != 0 {
            flags |= SACK_SEND1;
            masks.push(self.send1);
        }
        if self.send2 != 0 {
            flags |= SACK_SEND2;
            masks.push(self.send2);
        }
        let mut b = vec![0u8; 12];
        b[0] = CFRAME_COMMAND;
        b[1] = OP_SACK;
        b[2] = flags;
        b[3] = self.retry;
        b[4] = self.nseq;
        b[5] = self.nrcv;
        b[8..12].copy_from_slice(&self.tick.to_le_bytes());
        for m in masks {
            b.extend_from_slice(&m.to_le_bytes());
        }
        b
    }
}

/// A parsed CFRAME.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Connect(CFrame),
    Connected(CFrame),
    ConnectedSigned(SignedFrame),
    HardDisconnect(CFrame),
    Sack(SackFrame),
    /// Any other `bExtOpCode`: `Dpl8r_HandleCFrame`'s default arm calls `FUN_00421ed0`, which
    /// tears the connection down.
    Unknown(u8),
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    match b.get(at..at + 4) {
        Some(s) => u32::from_le_bytes([s[0], s[1], s[2], s[3]]),
        None => 0,
    }
}

/// `Dpl8r_HandleCFrame` (dpnet 0x004220e0): needs `length > 11`, switches on byte 1. Fields past
/// the end of a short frame read as zero (the binary reads whatever follows).
pub fn parse_command(b: &[u8]) -> Option<Command> {
    if b.len() <= 11 {
        return None;
    }
    let cf = || CFrame {
        poll: b[0] & CFRAME_POLL != 0,
        op: b[1],
        msg_id: b[2],
        rsp_id: b[3],
        version: u32_at(b, 4),
        sess_id: u32_at(b, 8),
        tick: u32_at(b, 12),
    };
    Some(match b[1] {
        OP_CONNECT => Command::Connect(cf()),
        OP_CONNECTED => Command::Connected(cf()),
        OP_CONNECTED_SIGNED if b.len() > 0x2f => {
            let sec = |at: usize| {
                let mut o = [0u8; 8];
                o.copy_from_slice(&b[at..at + 8]);
                o
            };
            Command::ConnectedSigned(SignedFrame {
                base: cf(),
                connect_sig: sec(0x10),
                sender_secret: sec(0x18),
                receiver_secret: sec(0x20),
                options: u32_at(b, 0x28),
                echo_tick: u32_at(b, 0x2c),
            })
        }
        OP_HARD_DISCONNECT => Command::HardDisconnect(cf()),
        OP_SACK => {
            // Masks sit from +0xc in the order SACK1, SACK2, SEND1, SEND2 (dpnet.dll.c:27597-27643).
            let flags = b[2];
            let mut at = 12;
            let mut take = |bit: u8| {
                if flags & bit != 0 {
                    let v = u32_at(b, at);
                    at += 4;
                    v
                } else {
                    0
                }
            };
            let sack1 = take(SACK_MASK1);
            let sack2 = take(SACK_MASK2);
            let send1 = take(SACK_SEND1);
            let send2 = take(SACK_SEND2);
            Command::Sack(SackFrame {
                retry: b[3],
                nseq: b[4],
                nrcv: b[5],
                tick: u32_at(b, 8),
                sack1,
                sack2,
                send1,
                send2,
            })
        }
        other => Command::Unknown(other),
    })
}

/// A parsed DFRAME.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DFrame {
    pub cmd: u8,
    pub ctl: u8,
    pub seq: u8,
    pub nrcv: u8,
    pub sack1: u32,
    pub sack2: u32,
    pub send1: u32,
    pub send2: u32,
    /// Offset of the 8-byte signature (`4 + mask bytes`) when the connection signs; the payload
    /// then starts 8 bytes later.
    pub sig_at: Option<usize>,
    pub payload: Vec<u8>,
    /// A coalesced DFRAME (`bControl & 4`): the sub-messages in order.
    pub subs: Option<Vec<SubMessage>>,
}

/// One message of a coalesced DFRAME (DX9 0x004259cf:32344-32372).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubMessage {
    /// The sub-header's flag byte: 0x01 END_COALESCE (last), 0x02 RELIABLE, 0x04 SEQUENTIAL,
    /// 0x08/0x10/0x20 size bits 8-10, 0x40/0x80 USER_1/USER_2.
    pub flags: u8,
    pub data: Vec<u8>,
}

/// The DFRAME header length without signature: `4 + 4 x` the mask bits of `bControl` (`FUN_004241e0`
/// 0x004241e0:30801 counts bits 0x10, 0x20, 0x40, 0x80).
pub fn dframe_header_len(ctl: u8) -> usize {
    4 + 4 * (ctl >> 4).count_ones() as usize
}

/// `Dpl8r_HandleDFrame` (dpnet 0x00422340): `bControl` bits 4-7 say which of the four mask
/// words follow the 4-byte header, in the order SACK1, SACK2, SEND1, SEND2.
pub fn parse_dframe(b: &[u8]) -> Option<DFrame> {
    parse_dframe_ex(b, false)
}

/// DX9 `FUN_004259cf` (0x004259cf:32295-32372): as [`parse_dframe`], plus the 8-byte signature
/// after the masks when `signed` (the frame must then hold 4 + masks + 8 bytes) and, for
/// `bControl & 4`, the coalescing table: `bCommand` must have NEW_MSG and END_MSG (0x30); 2-byte
/// headers `[size_lo][flags]` with `size = size_lo | (flags & 0x38) << 5` (zero is invalid)
/// until one has flag 0x01; an odd count is followed by 2 pad bytes; every payload but the last
/// is padded to 4 bytes; the table plus payloads must fit the datagram. The keep-alive form
/// (`bControl & 2` from a v1.6 peer) is handled by the caller, not here.
// open: the DX9 receive path has no limit on the number of sub-messages (the 32 limit is a
// sender rule, 0x004266a8); a table longer than the datagram is invalid.
pub fn parse_dframe_ex(b: &[u8], signed: bool) -> Option<DFrame> {
    if b.len() < 4 {
        return None;
    }
    let ctl = b[1];
    let n = (ctl >> 4).count_ones() as usize;
    let mut header = 4 + 4 * n;
    if b.len() < header {
        return None;
    }
    let mut at = 4;
    let mut take = |bit: u8| {
        if ctl & bit != 0 {
            let v = u32_at(b, at);
            at += 4;
            v
        } else {
            0
        }
    };
    let sack1 = take(CTL_SACK1);
    let sack2 = take(CTL_SACK2);
    let send1 = take(CTL_SEND1);
    let send2 = take(CTL_SEND2);
    let sig_at = if signed {
        let s = header;
        header += 8;
        if b.len() < header {
            return None;
        }
        Some(s)
    } else {
        None
    };
    Some(DFrame {
        cmd: b[0],
        ctl,
        seq: b[2],
        nrcv: b[3],
        sack1,
        sack2,
        send1,
        send2,
        sig_at,
        payload: b[header..].to_vec(),
        subs: None,
    })
}

/// `DAT_004517d8` default (`ProtocolMaxCoalesceSize`, range 0x80..0x7ff): the largest message
/// that may join a coalesced DFRAME (DX9 spec_client_delta 2.1.4, 0x00423f42:30715-30722).
pub const MAX_COALESCE_SIZE: usize = 0x7ff;
/// The most messages in one coalesced DFRAME (0x004266a8, DPL8R 2.2.3).
pub const MAX_COALESCE_COUNT: usize = 32;

/// Send side (`FUN_0042606c` 0x0042606c:32708-32745, candidates chosen by `FUN_004270ea` /
/// `FUN_004266a8`): build the payload of a coalesced DFRAME from single-fragment messages
/// `(flags, data)` (`flags` = RELIABLE 0x02, SEQUENTIAL 0x04, USER_1/2 0x40/0x80 and the size
/// bits 0x38 are set here): 2-byte headers `[size_lo][flags]`, `0x01` on the last, 2 pad bytes
/// after an odd count, every payload but the last padded to 4 bytes. `None` when a rule fails:
/// 2..=32 messages, each 1..=`MAX_COALESCE_SIZE`, the whole payload within `max_payload`
/// (`conn+0x38`, 1356). The caller sends it with `bControl` 0x04 and `bCommand` NEW_MSG|END_MSG.
/// Only messages whose item flag 0x100000 is clear may be offered: with the default
/// `ProtocolCoalesceMode` 0 that is the public flag 0x100 and the internal system messages
/// (0xC1.., flags 0x141); GunDll's flags 0x9 never qualify.
pub fn encode_coalesced(msgs: &[(u8, &[u8])], max_payload: usize) -> Option<Vec<u8>> {
    if msgs.len() < 2 || msgs.len() > MAX_COALESCE_COUNT {
        return None;
    }
    let mut out = Vec::new();
    let last = msgs.len() - 1;
    for (i, (fl, d)) in msgs.iter().enumerate() {
        if d.is_empty() || d.len() > MAX_COALESCE_SIZE {
            return None;
        }
        let size = d.len();
        let mut f = (fl & 0xc6) | (((size >> 8) as u8 & 7) << 3);
        if i == last {
            f |= 1;
        }
        out.push(size as u8);
        out.push(f);
    }
    if msgs.len() & 1 != 0 {
        out.extend_from_slice(&[0, 0]);
    }
    for (i, (_, d)) in msgs.iter().enumerate() {
        out.extend_from_slice(d);
        if i != last {
            while out.len() & 3 != 0 {
                out.push(0);
            }
        }
    }
    (out.len() <= max_payload).then_some(out)
}

/// Decode the coalescing table of `d.payload` into `d.subs` (see [`parse_dframe_ex`]); `None`
/// when the frame is invalid and must be dropped.
pub fn decode_coalesced(mut d: DFrame) -> Option<DFrame> {
    let p = &d.payload;
    if d.cmd & (CMD_NEW_MSG | CMD_END_MSG) != (CMD_NEW_MSG | CMD_END_MSG) {
        return None;
    }
    let mut sizes: Vec<(u8, usize)> = Vec::new();
    let mut at = 0usize;
    loop {
        let (lo, fl) = (*p.get(at)?, *p.get(at + 1)?);
        at += 2;
        let size = lo as usize | ((fl as usize & 0x38) << 5);
        if size == 0 {
            return None;
        }
        sizes.push((fl, size));
        if fl & 1 != 0 {
            break;
        }
    }
    if sizes.len() & 1 != 0 {
        at += 2;
    }
    let mut subs = Vec::new();
    let last = sizes.len() - 1;
    for (i, (fl, size)) in sizes.iter().enumerate() {
        let data = p.get(at..at + size)?.to_vec();
        at += if i == last { *size } else { (size + 3) & !3 };
        subs.push(SubMessage { flags: *fl, data });
    }
    d.subs = Some(subs);
    Some(d)
}

/// A DFRAME header being built: `bCommand`, `bControl`, `bSeq`, `bNRcv` and the masks that go in
/// (`Dpl8r_BuildDFrame` writes the control bits for the masks it attaches).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DHeader {
    pub cmd: u8,
    pub ctl: u8,
    pub seq: u8,
    pub nrcv: u8,
    pub sack1: u32,
    pub sack2: u32,
    pub send1: u32,
    pub send2: u32,
}

impl DHeader {
    /// 4 bytes plus one word per control bit 0x10..0x80 that is set.
    pub fn encode(&self) -> Vec<u8> {
        let mut b = vec![self.cmd, self.ctl, self.seq, self.nrcv];
        for (bit, v) in [
            (CTL_SACK1, self.sack1),
            (CTL_SACK2, self.sack2),
            (CTL_SEND1, self.send1),
            (CTL_SEND2, self.send2),
        ] {
            if self.ctl & bit != 0 {
                b.extend_from_slice(&v.to_le_bytes());
            }
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected bytes are written out from spec_client.md section 2.2/2.3 and the adjudication
    // (CONNECT 0x88, CONNECTED 0x80/0x88, lone frames 0x3F), not produced by the encoder.

    #[test]
    fn connect_cframe_bytes() {
        let f = CFrame {
            poll: true,
            op: OP_CONNECT,
            msg_id: 0,
            rsp_id: 0,
            version: PROTOCOL_VERSION,
            sess_id: 0x1122_3344,
            tick: 0x0000_0102,
        };
        assert_eq!(
            f.encode(),
            [
                0x88, 0x01, 0x00, 0x00, 0x06, 0x00, 0x01, 0x00, 0x44, 0x33, 0x22, 0x11, 0x02, 0x01,
                0x00, 0x00
            ]
        );
    }

    #[test]
    fn connected_and_hard_disconnect_cframe_bytes() {
        let c = CFrame {
            poll: false,
            op: OP_CONNECTED,
            msg_id: 5,
            rsp_id: 9,
            version: PROTOCOL_VERSION,
            sess_id: 1,
            tick: 2,
        };
        assert_eq!(c.encode()[..4], [0x80, 0x02, 0x05, 0x09]);
        let h = CFrame {
            poll: false,
            op: OP_HARD_DISCONNECT,
            msg_id: 1,
            rsp_id: 0,
            version: PROTOCOL_VERSION,
            sess_id: 1,
            tick: 2,
        };
        assert_eq!(h.encode()[..4], [0x80, 0x04, 0x01, 0x00]);
    }

    #[test]
    fn sack_with_two_masks_is_20_bytes_and_flags_0x07() {
        let s = SackFrame {
            retry: 1,
            nseq: 3,
            nrcv: 7,
            tick: 0x10,
            sack1: 0b101,
            sack2: 0x8000_0000,
            send1: 0,
            send2: 0,
        };
        let b = s.encode();
        assert_eq!(b.len(), 20);
        assert_eq!(&b[..8], &[0x80, 0x06, 0x07, 0x01, 0x03, 0x07, 0x00, 0x00]);
        assert_eq!(&b[8..12], &[0x10, 0, 0, 0]);
        assert_eq!(&b[12..16], &[0x05, 0, 0, 0]);
        assert_eq!(&b[16..20], &[0, 0, 0, 0x80]);
    }

    #[test]
    fn sack_roundtrip_through_the_parser_reads_masks_in_order() {
        let s = SackFrame {
            retry: 0,
            nseq: 1,
            nrcv: 2,
            tick: 3,
            sack1: 0,
            sack2: 0,
            send1: 0xAA,
            send2: 0xBB,
        };
        assert_eq!(parse_command(&s.encode()), Some(Command::Sack(s)));
    }

    #[test]
    fn short_cframe_is_ignored_and_unknown_opcode_reported() {
        assert_eq!(parse_command(&[0x80, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]), None);
        let mut b = [0u8; 16];
        b[0] = 0x80;
        b[1] = 3;
        assert_eq!(parse_command(&b), Some(Command::Unknown(3)));
    }

    #[test]
    fn coalesced_payload_round_trips_with_padding_and_odd_count_pad() {
        let msgs: Vec<(u8, &[u8])> = vec![(0x02, b"abc"), (0x42, b"de"), (0x82, &[7u8; 300])];
        let p = encode_coalesced(&msgs, 1356).unwrap();
        // 3 headers (6 bytes) + 2 pad, "abc" + 1 pad, "de" + 2 pad, 300 bytes
        assert_eq!(&p[..8], &[3, 0x02, 2, 0x42, 44, 0x8b, 0, 0]);
        assert_eq!(p.len(), 8 + 4 + 4 + 300);
        let d = DFrame {
            cmd: 0x37,
            ctl: 4,
            seq: 0,
            nrcv: 0,
            sack1: 0,
            sack2: 0,
            send1: 0,
            send2: 0,
            sig_at: None,
            payload: p,
            subs: None,
        };
        let subs = decode_coalesced(d).unwrap().subs.unwrap();
        assert_eq!(subs.len(), 3);
        assert_eq!(subs[0].data, b"abc");
        assert_eq!(subs[2].data.len(), 300);
        assert_eq!(subs[2].flags, 0x8b);
        assert!(encode_coalesced(&msgs[..1], 1356).is_none());
        assert!(encode_coalesced(&msgs, 100).is_none());
    }

    #[test]
    fn dframe_header_with_masks() {
        let h = DHeader {
            cmd: 0x3F,
            ctl: CTL_SACK1 | CTL_SEND2,
            seq: 4,
            nrcv: 5,
            sack1: 6,
            sack2: 0,
            send1: 0,
            send2: 7,
        };
        let mut wire = h.encode();
        assert_eq!(wire, [0x3F, 0x90, 4, 5, 6, 0, 0, 0, 7, 0, 0, 0]);
        wire.extend_from_slice(b"hi");
        let d = parse_dframe(&wire).unwrap();
        assert_eq!((d.cmd, d.seq, d.nrcv, d.sack1, d.send2), (0x3F, 4, 5, 6, 7));
        assert_eq!(d.payload, b"hi");
    }

    #[test]
    fn truncated_dframe_is_dropped() {
        assert_eq!(parse_dframe(&[0x37, 0x10, 0, 0, 1, 0]), None);
        assert_eq!(parse_dframe(&[0x37, 0, 0]), None);
    }
}
