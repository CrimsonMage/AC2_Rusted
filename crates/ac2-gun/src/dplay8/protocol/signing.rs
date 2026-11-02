//! DPL8R packet signing (CONNECTED_SIGNED, FAST and FULL), DX9 `dpnet.dll` only.
//!
//! A host that signs answers a CONNECT with CONNECTED_SIGNED (opcode 3); the connector (this
//! client, which never asks for signing and cannot refuse it: no code in 0x00421f01 tests the
//! connect request, `verify_spec_client_delta.md` 2.3.6) draws two secrets, answers, and from then
//! on appends an 8-byte signature to HARD_DISCONNECT, SACK and DFRAME and checks the host's.
//!
//! * Hash: plain SHA-1 (`FUN_0042824b` init, `FUN_00428282` block: big-endian words, constants
//!   5a827999 6ed9eba1 8f1bbcdc ca62c1d6, `FUN_00428470` padding with the big-endian bit count,
//!   `FUN_00428515` big-endian digest). Every "signature" is **the first 8 digest bytes**, stored
//!   as the wire bytes in digest order (the helpers return the digest in `eax:edx` and callers
//!   store the pair little endian, so memory order is digest order).
//! * Secrets are 8 raw bytes (two little-endian dwords copied from or into frames); in a hash
//!   they are appended as those 8 bytes.
//! * FAST (`conn[5] & 0x80`): the signature field is the sender's secret itself
//!   (0x0042606c:32718-32721, 0x00427839:33998-34001, 0x00427556:33830-33833).
//! * FULL (`conn[5] & 0x100`): signature = digest8(whole datagram with the signature field zero
//!   || secret) (`FUN_00428611`, secret pushed by the callers, e.g. 0x004273e8, 0x004279ea;
//!   receive `FUN_004248ec`/`FUN_00428688`).
//! * Key rotation (FULL only): the secret changes every 256 frames, derived as
//!   `digest8(secret || key)` (`FUN_004286e5`, called with the secret and the `key` at
//!   `conn+0x98`/`+0xa0`). `key` is `digest8(payload)` of the lowest-sequence reliable
//!   non-keep-alive frame since the last rotation (`FUN_004287a5` on receive 0x004259cf:32411,
//!   `FUN_00428742` on send 0x0042606c:32784). Sender: after signing a first transmission with
//!   `bSeq == 0xFF` (0x00427413-0x4273e8). Receiver: when `nextRecv` becomes `0xC0`
//!   (`FUN_00424d1c` 0x00424e76). The previous secret stays for frames with `bSeq >= 0xC0`.

/// SHA-1, as `FUN_0042824b`..`FUN_00428515` compute it.
#[derive(Clone)]
pub struct Sha1 {
    h: [u32; 5],
    block: [u8; 64],
    n: usize,
    bits: u64,
}

impl Default for Sha1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha1 {
    pub fn new() -> Self {
        Sha1 {
            h: [
                0x6745_2301,
                0xefcd_ab89,
                0x98ba_dcfe,
                0x1032_5476,
                0xc3d2_e1f0,
            ],
            block: [0; 64],
            n: 0,
            bits: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        for &b in data {
            self.block[self.n] = b;
            self.n += 1;
            self.bits = self.bits.wrapping_add(8);
            if self.n == 64 {
                self.compress();
            }
        }
    }

    fn compress(&mut self) {
        let mut w = [0u32; 80];
        for (i, c) in self.block.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([c[0], c[1], c[2], c[3]]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = self.h;
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5a82_7999),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6u32),
            };
            let t = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = t;
        }
        for (h, v) in self.h.iter_mut().zip([a, b, c, d, e]) {
            *h = h.wrapping_add(v);
        }
        self.n = 0;
    }

    pub fn finish(mut self) -> [u8; 20] {
        let bits = self.bits;
        self.block[self.n] = 0x80;
        self.n += 1;
        if self.n > 56 {
            while self.n < 64 {
                self.block[self.n] = 0;
                self.n += 1;
            }
            self.compress();
        }
        while self.n < 56 {
            self.block[self.n] = 0;
            self.n += 1;
        }
        self.block[56..].copy_from_slice(&bits.to_be_bytes());
        self.compress();
        let mut out = [0u8; 20];
        for (o, h) in out.chunks_mut(4).zip(self.h) {
            o.copy_from_slice(&h.to_be_bytes());
        }
        out
    }
}

/// The first 8 digest bytes of SHA-1 over the parts, in order.
pub fn digest8(parts: &[&[u8]]) -> [u8; 8] {
    let mut s = Sha1::new();
    for p in parts {
        s.update(p);
    }
    let d = s.finish();
    let mut o = [0u8; 8];
    o.copy_from_slice(&d[..8]);
    o
}

/// `conn[5] & 0x80` (FAST) or `& 0x100` (FULL).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Fast,
    Full,
}

/// The reset value of both markers (`conn+0xa8`, `+0xa9`: 0xC0, 0x00421f01:28984-28985).
pub const MARKER_RESET: u8 = 0xc0;

/// The signing state of the connection (`conn+0x78..+0xa9`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigState {
    pub mode: Mode,
    /// `conn+0x78`: signs what we send.
    pub send: [u8; 8],
    /// `conn+0x80`: the previous send secret.
    pub send_prev: [u8; 8],
    /// `conn+0x98`: the send rotation key.
    pub send_key: [u8; 8],
    /// `conn+0xa8`.
    pub send_marker: u8,
    /// `conn+0x88`: checks what we receive.
    pub recv: [u8; 8],
    /// `conn+0x90`.
    pub recv_prev: [u8; 8],
    /// `conn+0xa0`.
    pub recv_key: [u8; 8],
    /// `conn+0xa9`.
    pub recv_marker: u8,
}

impl SigState {
    /// The state after the connector drew its secrets (0x00421f01:28975-28993): current and
    /// previous secrets equal, rotation keys equal to the initial secrets.
    pub fn new(mode: Mode, send: [u8; 8], recv: [u8; 8]) -> Self {
        SigState {
            mode,
            send,
            send_prev: send,
            send_key: send,
            send_marker: MARKER_RESET,
            recv,
            recv_prev: recv,
            recv_key: recv,
            recv_marker: MARKER_RESET,
        }
    }

    /// `FUN_004286e5`: `digest8(secret || key)`.
    pub fn rotate(secret: &[u8; 8], key: &[u8; 8]) -> [u8; 8] {
        digest8(&[secret, key])
    }

    /// `FUN_004248ec`: does the signature at `sig_at` of `datagram` verify? `seq` is the frame's
    /// sequence byte (DFRAME `bSeq`, SACK `bNSeq`, HARD_DISCONNECT `bRspId`), `next_recv` the
    /// connection's `+0x135`. FAST compares the secret; FULL hashes the datagram with the field
    /// zeroed and the secret the rotation window selects.
    pub fn verify(&self, next_recv: u8, datagram: &[u8], seq: u8, sig_at: usize) -> bool {
        let Some(sig) = datagram.get(sig_at..sig_at + 8) else {
            return false;
        };
        match self.mode {
            Mode::Fast => sig == self.recv,
            Mode::Full => {
                let mut copy = datagram.to_vec();
                copy[sig_at..sig_at + 8].fill(0);
                let secret = if (next_recv >= 0xc0 || next_recv < 0x40) && seq >= 0xc0 {
                    &self.recv_prev
                } else {
                    &self.recv
                };
                digest8(&[&copy, secret]) == sig
            }
        }
    }

    /// The receive-side key update (0x004259cf:32407-32412): a reliable (`cmd & 2`), non-keep-alive
    /// (`ctl & 2 == 0`) frame with `seq` below the marker sets the marker and the rotation key to
    /// `digest8(payload)` (the first reliable sub-payload of a coalesced frame).
    pub fn note_received(&mut self, cmd: u8, ctl: u8, seq: u8, payload: &[u8]) {
        if self.mode == Mode::Full && cmd & 2 != 0 && ctl & 2 == 0 && seq < self.recv_marker {
            self.recv_marker = seq;
            self.recv_key = digest8(&[payload]);
        }
    }

    /// The receive rotation when `nextRecv` has just become `new_next_recv`
    /// (`FUN_00424d1c` 0x00424e67-0x424eb1).
    pub fn on_next_recv(&mut self, new_next_recv: u8) {
        if self.mode == Mode::Full && new_next_recv == 0xc0 {
            self.recv_prev = self.recv;
            self.recv_marker = MARKER_RESET;
            self.recv = Self::rotate(&self.recv, &self.recv_key);
        }
    }

    /// The send-side key update (0x0042606c:32773-32785), at build time of a new frame.
    pub fn note_sent(&mut self, cmd: u8, ctl: u8, seq: u8, payload: &[u8]) {
        if self.mode == Mode::Full && cmd & 2 != 0 && ctl & 2 == 0 && seq < self.send_marker {
            self.send_marker = seq;
            self.send_key = digest8(&[payload]);
        }
    }

    /// The signature for an outgoing datagram whose signature field (at `sig_at`) is zero in
    /// `datagram`. FAST: the secret. FULL: `digest8(datagram || secret)`; a retransmission
    /// (`retry`) of a frame with `seq >= 0xC0` after the sequence wrapped (`next_send < 0x40`)
    /// uses the previous secret (0x004273c6-0x4273f7); a first transmission with `seq == 0xFF`
    /// rotates afterwards (0x0042740d-0x42743e).
    pub fn sign_dframe(&mut self, datagram: &[u8], seq: u8, retry: bool, next_send: u8) -> [u8; 8] {
        match self.mode {
            Mode::Fast => self.send,
            Mode::Full => {
                if retry {
                    let secret = if next_send < 0x40 && seq >= 0xc0 {
                        &self.send_prev
                    } else {
                        &self.send
                    };
                    digest8(&[datagram, secret])
                } else {
                    let sig = digest8(&[datagram, &self.send]);
                    if seq == 0xff {
                        self.send_prev = self.send;
                        self.send = Self::rotate(&self.send, &self.send_key);
                        self.send_marker = MARKER_RESET;
                    }
                    sig
                }
            }
        }
    }

    /// SACK and HARD_DISCONNECT: always the current send secret, no rotation
    /// (0x00427839:33998-34017, 0x00427556:33825-33839, 0x00427683).
    pub fn sign_control(&self, datagram: &[u8]) -> [u8; 8] {
        match self.mode {
            Mode::Fast => self.send,
            Mode::Full => digest8(&[datagram, &self.send]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn sha1_matches_the_fips_vectors() {
        let mut s = Sha1::new();
        s.update(b"abc");
        assert_eq!(hex(&s.finish()), "a9993e364706816aba3e25717850c26c9cd0d89d");
        let mut s = Sha1::new();
        s.update(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(hex(&s.finish()), "84983e441c3bd26ebaae4aa1f95129e5e54670f1");
        let mut s = Sha1::new();
        for _ in 0..1_000_000 / 100 {
            s.update(&[b'a'; 100]);
        }
        assert_eq!(hex(&s.finish()), "34aa973cd4c4daa4f61eeb2bdbad27316534016f");
    }

    #[test]
    fn digest8_is_the_first_eight_digest_bytes() {
        assert_eq!(hex(&digest8(&[b"a", b"bc"])), "a9993e364706816a");
    }

    #[test]
    fn full_signature_verifies_and_rejects_changes() {
        let mut tx = SigState::new(Mode::Full, [1; 8], [2; 8]);
        let rx = SigState::new(Mode::Full, [2; 8], [1; 8]);
        let mut d = vec![0x37, 0, 5, 0];
        d.extend_from_slice(&[0; 8]);
        d.extend_from_slice(b"payload");
        let sig = tx.sign_dframe(&d, 5, false, 6);
        d[4..12].copy_from_slice(&sig);
        assert!(rx.verify(5, &d, 5, 4));
        d[12] ^= 1;
        assert!(!rx.verify(5, &d, 5, 4));
    }

    #[test]
    fn rotation_keeps_the_previous_secret_for_the_tail_of_the_cycle() {
        let mut tx = SigState::new(Mode::Full, [1; 8], [2; 8]);
        let mut rx = SigState::new(Mode::Full, [2; 8], [1; 8]);
        let base = {
            let mut d = vec![0x37, 0, 0, 0];
            d.extend_from_slice(&[0; 8]);
            d.extend_from_slice(b"x");
            d
        };
        // first reliable frame of the cycle seeds the rotation keys on both sides
        tx.note_sent(0x37, 0, 0, b"x");
        rx.note_received(0x37, 0, 0, b"x");
        assert_eq!(tx.send_key, rx.recv_key);
        for seq in 0..=255u8 {
            let mut d = base.clone();
            d[2] = seq;
            let sig = tx.sign_dframe(&d, seq, false, seq.wrapping_add(1));
            d[4..12].copy_from_slice(&sig);
            assert!(rx.verify(seq, &d, seq, 4), "seq {seq}");
            rx.on_next_recv(seq.wrapping_add(1));
        }
        // both rotated: the next cycle signs and verifies under the new secret
        let mut d = base.clone();
        let sig = tx.sign_dframe(&d, 0, false, 1);
        d[4..12].copy_from_slice(&sig);
        assert!(rx.verify(0, &d, 0, 4));
        assert_ne!(tx.send, [1; 8]);
        assert_eq!(tx.send_prev, [1; 8]);
    }
}
