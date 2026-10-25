//! The logon session key: the CryptoAPI RC4 key the client makes in `ClientNet::InitCrypto`,
//! exports under the server's RSA key in `ClientNet::Init`, and encrypts queue 6-8 blobs with in
//! `Proto_UI::SendBlob` (all identical in admin 7148 and Legions 7160).
//!
//! `InitCrypto` acquires "Microsoft Base Cryptographic Provider v1.0" (`PROV_RSA_FULL`),
//! imports the 0x54-byte `PUBLICKEYBLOB` embedded in the client (a 512-bit RSA key), then
//! `CryptGenKey(CALG_RC4, CRYPT_EXPORTABLE)`. The Base provider makes that a 40-bit key with an
//! eleven-byte zero salt (no `CRYPT_CREATE_SALT` / `CRYPT_NO_SALT`), so RC4 runs on the 5 key
//! bytes followed by 11 zero bytes. `Init` exports it with `CryptExportKey(SIMPLEBLOB)`:
//! `BLOBHEADER {1, 2, 0, CALG_RC4}`, `CALG_RSA_KEYX`, then the PKCS #1 v1.5 type-2 block
//! `00 02 PS 00 key` raised to the public exponent, stored least significant byte first.
//! `SendBlob` calls `CryptEncrypt(key, 0, TRUE, …)`; `Final = TRUE` resets an RC4 key, so every
//! blob is encrypted from the start of the key stream.

use crate::NetError;

/// `BLOBHEADER {PUBLICKEYBLOB, CUR_BLOB_VERSION, 0, CALG_RSA_KEYX}` and `RSAPUBKEY.magic`.
const PUBLIC_KEY_HEADER: [u8; 12] = [6, 2, 0, 0, 0, 0xA4, 0, 0, b'R', b'S', b'A', b'1'];
/// `BLOBHEADER {SIMPLEBLOB, CUR_BLOB_VERSION, 0, CALG_RC4}` then `ALG_ID CALG_RSA_KEYX`.
const SIMPLE_BLOB_HEADER: [u8; 12] = [1, 2, 0, 0, 0x01, 0x68, 0, 0, 0, 0xA4, 0, 0];
/// The size of the key `ClientNet::InitCrypto` imports (`CryptImportKey(…, 0x54, …)`).
pub const RETAIL_PUBLIC_KEY_LEN: usize = 0x54;
/// Base-provider RC4 key bytes (40 bits) and its zero salt.
pub const SESSION_KEY_LEN: usize = 5;
const SALT_LEN: usize = 11;

/// The server's RSA key-exchange key, from a CryptoAPI `PUBLICKEYBLOB`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPublicKey {
    /// Little-endian 32-bit limbs.
    modulus: Vec<u32>,
    /// Modulus bytes (`bitlen / 8`).
    len: usize,
    exponent: u32,
}

impl ServerPublicKey {
    /// Parses a `PUBLICKEYBLOB`: header, `RSAPUBKEY {magic, bitlen, pubexp}`, little-endian
    /// modulus.
    pub fn from_blob(blob: &[u8]) -> Result<Self, NetError> {
        if blob.len() < 20 || blob[..12] != PUBLIC_KEY_HEADER {
            return Err(NetError::Crypto("not an RSA key-exchange PUBLICKEYBLOB"));
        }
        let word =
            |at: usize| u32::from_le_bytes([blob[at], blob[at + 1], blob[at + 2], blob[at + 3]]);
        let bits = word(12) as usize;
        let exponent = word(16);
        let len = bits / 8;
        if !bits.is_multiple_of(32) || len < 16 || blob.len() != 20 + len || exponent == 0 {
            return Err(NetError::Crypto("PUBLICKEYBLOB size"));
        }
        let modulus: Vec<u32> = blob[20..]
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        if modulus.last() == Some(&0) {
            return Err(NetError::Crypto("modulus shorter than bitlen"));
        }
        Ok(Self {
            modulus,
            len,
            exponent,
        })
    }

    /// The key `ClientNet::InitCrypto` imports, found in a client executable's bytes: the one
    /// 0x54-byte `PUBLICKEYBLOB` it carries.
    pub fn find_in_client(exe: &[u8]) -> Option<Self> {
        let at = exe
            .windows(PUBLIC_KEY_HEADER.len())
            .position(|w| w == PUBLIC_KEY_HEADER)?;
        Self::from_blob(exe.get(at..at + RETAIL_PUBLIC_KEY_LEN)?).ok()
    }

    pub fn bits(&self) -> usize {
        self.len * 8
    }

    /// `m ^ e mod n` for a big-endian `block` of the modulus size, returned little-endian.
    fn encrypt_block(&self, block: &[u8]) -> Vec<u8> {
        let mut m: Vec<u32> = block
            .rchunks(4)
            .map(|c| {
                let mut w = [0u8; 4];
                w[4 - c.len()..].copy_from_slice(c);
                u32::from_be_bytes(w)
            })
            .collect();
        m.resize(self.modulus.len(), 0);
        let n = &self.modulus;
        let mut result = vec![0u32; n.len()];
        result[0] = 1;
        for bit in (0..32 - self.exponent.leading_zeros()).rev() {
            result = mul_mod(&result, &result, n);
            if self.exponent >> bit & 1 != 0 {
                result = mul_mod(&result, &m, n);
            }
        }
        result.iter().flat_map(|w| w.to_le_bytes()).collect()
    }
}

/// `a * b mod n` on little-endian limbs, reducing the product bit by bit.
fn mul_mod(a: &[u32], b: &[u32], n: &[u32]) -> Vec<u32> {
    let mut product = vec![0u32; a.len() + b.len()];
    for (i, &x) in a.iter().enumerate() {
        let mut carry = 0u64;
        for (j, &y) in b.iter().enumerate() {
            let t = u64::from(x) * u64::from(y) + u64::from(product[i + j]) + carry;
            product[i + j] = t as u32;
            carry = t >> 32;
        }
        product[i + b.len()] = carry as u32;
    }
    let mut r = vec![0u32; n.len() + 1];
    for bit in (0..product.len() * 32).rev() {
        let mut carry = product[bit / 32] >> (bit % 32) & 1;
        for limb in &mut r {
            let next = *limb >> 31;
            *limb = *limb << 1 | carry;
            carry = next;
        }
        if !less_than(&r, n) {
            let mut borrow = 0i64;
            for (i, limb) in r.iter_mut().enumerate() {
                let t = i64::from(*limb) - i64::from(n.get(i).copied().unwrap_or(0)) - borrow;
                *limb = t as u32;
                borrow = i64::from(t < 0);
            }
        }
    }
    r.truncate(n.len());
    r
}

fn less_than(r: &[u32], n: &[u32]) -> bool {
    for i in (0..r.len()).rev() {
        let m = n.get(i).copied().unwrap_or(0);
        if r[i] != m {
            return r[i] < m;
        }
    }
    false
}

/// The 40-bit RC4 session key (`ClientNet::hSessionKey_`).
#[derive(Clone, PartialEq, Eq)]
pub struct SessionKey([u8; SESSION_KEY_LEN]);

impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionKey(..)")
    }
}

impl SessionKey {
    pub fn from_bytes(key: [u8; SESSION_KEY_LEN]) -> Self {
        Self(key)
    }

    /// `CryptGenKey(CALG_RC4, CRYPT_EXPORTABLE)`: five random bytes.
    pub fn generate(fill: &mut dyn FnMut(&mut [u8])) -> Self {
        let mut key = [0; SESSION_KEY_LEN];
        fill(&mut key);
        Self(key)
    }

    /// `CryptExportKey(hSessionKey_, hKey_, SIMPLEBLOB, 0)`: the blob
    /// `NetAuthenticator::SetCryptoKey` stores. `fill` supplies the nonzero padding.
    pub fn export(&self, server: &ServerPublicKey, fill: &mut dyn FnMut(&mut [u8])) -> Vec<u8> {
        let len = server.len;
        let mut block = vec![0u8; len];
        block[1] = 2;
        let padding = &mut block[2..len - SESSION_KEY_LEN - 1];
        fill(padding);
        for byte in padding.iter_mut() {
            while *byte == 0 {
                fill(std::slice::from_mut(byte));
            }
        }
        block[len - SESSION_KEY_LEN..].copy_from_slice(&self.0);
        let mut out = SIMPLE_BLOB_HEADER.to_vec();
        out.extend(server.encrypt_block(&block));
        out
    }

    /// `CryptEncrypt(hSessionKey_, 0, TRUE, 0, …)`: RC4 over the key and its zero salt, from
    /// the start of the key stream on every call.
    pub fn encrypt(&self, data: &mut [u8]) {
        let mut key = [0u8; SESSION_KEY_LEN + SALT_LEN];
        key[..SESSION_KEY_LEN].copy_from_slice(&self.0);
        let mut s: [u8; 256] = std::array::from_fn(|i| i as u8);
        let mut j = 0u8;
        for i in 0..256 {
            j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
            s.swap(i, usize::from(j));
        }
        let (mut i, mut j) = (0u8, 0u8);
        for byte in data {
            i = i.wrapping_add(1);
            j = j.wrapping_add(s[usize::from(i)]);
            s.swap(usize::from(i), usize::from(j));
            *byte ^= s[usize::from(s[usize::from(i)].wrapping_add(s[usize::from(j)]))];
        }
    }
}

/// A session key and its exported blob, made once per `ClientNet::Init`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCrypto {
    pub key: SessionKey,
    pub blob: Vec<u8>,
}

impl SessionCrypto {
    /// `ClientNet::InitCrypto` then the export in `ClientNet::Init`, from the system RNG.
    pub fn new(server: &ServerPublicKey) -> Result<Self, NetError> {
        let mut failed = false;
        let mut fill = |buf: &mut [u8]| failed |= getrandom::fill(buf).is_err();
        let key = SessionKey::generate(&mut fill);
        let blob = key.export(server, &mut fill);
        if failed {
            return Err(NetError::Crypto("no system random source"));
        }
        Ok(Self { key, blob })
    }
}

#[cfg(test)]
mod tests {
    use super::{ServerPublicKey, SessionKey};

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    // A synthetic 512-bit key, session key and padding run through Wine's rsaenh
    // (`CryptGenKey`, `CryptExportKey(SIMPLEBLOB)`, `CryptEncrypt(Final = TRUE)` on 16 zeros).
    const PUBLIC: &str = "0602000000a4000052534131000200000100010039607fbac78de663d47d46e4\
        34320d9f5ec3cb048712fbbfdcc1479cde395443e23b643657736ef26444927d876026eee4fb0a4c3362\
        0a49bfe6d0b983aaed96";
    const KEY: [u8; 5] = [0x86, 0x85, 0x25, 0x7E, 0x18];
    const PADDING: &str = "df9726b8bd19b1690974bbb454b74bb9378c38a1940d5ef67a6858934b9d4c7d\
        1d7d62e39b1d9b9158b6af303b4fd34706214b1b5e44c1ef";
    const SIMPLE: &str = "010200000168000000a4000051046d53220f1c15c742bd8d65f2d219c712cc5f88\
        460b4ddef1ceb5e4ff5eea1a7e295be1e498692a71930f0cf2ff957d902c2940c69c1f7a2d0c55a0883447";
    const STREAM: &str = "3306824abfc44ef8f484d5bd5b73de94";

    #[test]
    fn simple_blob_matches_cryptoapi_byte_for_byte() {
        let server = ServerPublicKey::from_blob(&hex(PUBLIC)).unwrap();
        assert_eq!(server.bits(), 512);
        let padding = hex(PADDING);
        let blob = SessionKey::from_bytes(KEY).export(&server, &mut |buf| {
            buf.copy_from_slice(&padding[..buf.len()])
        });
        assert_eq!(blob, hex(SIMPLE));
        assert_eq!(blob.len(), 76);
    }

    #[test]
    fn rc4_uses_the_zero_salt_and_restarts_each_call() {
        let key = SessionKey::from_bytes(KEY);
        for _ in 0..2 {
            let mut data = [0u8; 16];
            key.encrypt(&mut data);
            assert_eq!(data.to_vec(), hex(STREAM));
        }
    }

    #[test]
    fn zero_padding_bytes_are_drawn_again() {
        let server = ServerPublicKey::from_blob(&hex(PUBLIC)).unwrap();
        let padding = hex(PADDING);
        let mut calls = 0;
        let blob = SessionKey::from_bytes(KEY).export(&server, &mut |buf| {
            calls += 1;
            if buf.len() > 1 {
                buf.copy_from_slice(&padding[..buf.len()]);
                buf[3] = 0;
            } else {
                buf[0] = padding[3];
            }
        });
        assert_eq!((blob, calls), (hex(SIMPLE), 2));
    }

    #[test]
    fn the_retail_sized_key_is_found_in_client_bytes() {
        let mut exe = vec![0x90; 100];
        exe.extend(hex(PUBLIC));
        exe.extend([0xCC; 7]);
        let found = ServerPublicKey::find_in_client(&exe).unwrap();
        assert_eq!(found, ServerPublicKey::from_blob(&hex(PUBLIC)).unwrap());
        assert!(ServerPublicKey::find_in_client(&exe[..150]).is_none());
        let mut bad = hex(PUBLIC);
        bad[0] = 7;
        assert!(ServerPublicKey::from_blob(&bad).is_err());
        assert!(ServerPublicKey::from_blob(&hex(PUBLIC)[..83]).is_err());
    }
}
