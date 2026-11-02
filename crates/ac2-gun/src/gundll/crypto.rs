//! GunDll's use of the Microsoft CryptoAPI (`FUN_10004170` .. `FUN_10004d00`), which is the Base
//! Cryptographic Provider v1.0 (`PROV_RSA_FULL`, container "GunKey"): `0a_send_session.md` section 5.
//! The provider's own behaviour is taken from Microsoft's documentation, saved under
//! `decompiled-pdb/gundll/spec/msdocs/` and tagged [ms-docs] here.
//!
//! * `FUN_100048d0` `CryptImportKey` of the server's `RSAKey` blob: a PUBLICKEYBLOB [ms-docs:
//!   base-provider-key-blobs.md]: `BLOBHEADER { bType 6, bVersion 2, reserved 0, aiKeyAlg }`,
//!   `RSAPUBKEY { magic 'RSA1' 0x31415352, bitlen, pubexp }`, `modulus[bitlen / 8]` little endian.
//! * `FUN_10004940` `CryptGenRandom(4)`: the client random.
//! * `FUN_10004cd0` / `FUN_10004bd0` / `FUN_10004560`: `CryptCreateHash(CALG_MD5 0x8003)`,
//!   `CryptHashData(clientRandom LE || serverRandom LE)`, `CryptDeriveKey(CALG_RC4 0x6801, hash,
//!   0x280001)`: key length 0x28 = 40 bits in the upper word and `CRYPT_EXPORTABLE`, no salt flag.
//!   [ms-docs] the key is the first n = 5 bytes of the hash value; no `CRYPT_CREATE_SALT` so the salt
//!   is zero; the Base Provider gives a 40-bit key without `CRYPT_NO_SALT` eleven bytes of zero salt
//!   (cryptderivekey.md, salt-value-functionality.md). [wine-tests] Wine's `test_rc4`
//!   (`msdocs/wine/rc4_40_salt.md`) derives a 40-bit key with `CRYPT_CREATE_SALT` from an MD5 and
//!   encrypts 16 bytes of 0xAF; the known answer for the Base provider equals RC4 keyed with the
//!   FIRST 16 BYTES OF THE MD5 (the 5 key bytes followed by the 11 salt bytes), and the Enhanced
//!   provider's (no salt) equals RC4 keyed with the first 5 bytes. So the RC4 key is key || salt
//!   (reproduced byte for byte in `tests/crypto_vectors.rs`). // open: the vector covers a NON-zero
//!   salt; that a zero salt is eleven 0x00 bytes in the same position (the case GunDll uses) follows
//!   from the same code path and Microsoft's "eleven bytes of zero-value salt" but is not itself a
//!   captured vector.
//! * `FUN_10004a30` `CryptExportKey(sessionKey, serverKey, SIMPLEBLOB)`: `BLOBHEADER { 1, 2, 0,
//!   CALG_RC4 }`, `ALG_ID CALG_RSA_KEYX` (0xa400), then the key data as a PKCS #1 type 2 block the
//!   size of the modulus (`00 02 PS 00 key`), little endian. [ms-docs: base-provider-key-blobs.md]
//!   for the layout; [wine-tests] Wine's `test_import_private` (`msdocs/wine/simpleblob.md`) holds a
//!   SIMPLEBLOB (`01 02 00 00 01 68 00 00 00 a4 00 00` + 128 bytes) with the private key that opens
//!   it: reversing the 128 bytes and RSA-decrypting gives `00 02 .. 00` followed by exactly the 7 key
//!   bytes of a 56-bit key, and RC4 keyed with those 7 bytes decrypts the test's message to
//!   "Wine rocks!" (reproduced in `tests/crypto_vectors.rs`). That fixes the byte order and shows the
//!   block holds the key bytes only (no salt). // open: that vector is a 56-bit key; for the 40-bit
//!   zero-salt key the block holding just the 5 key bytes is the same code path but not a captured
//!   vector.
//! * `FUN_10004630` `CryptEncrypt(key, 0, Final = TRUE, ...)`: RC4, same length out. [ms-docs:
//!   cryptencrypt.md] "If the cipher is a stream cipher, the next CryptEncrypt resets the cipher to
//!   its initial state": every call with `Final = TRUE` restarts the key stream, so each encrypted
//!   field (CdKey, password, ticket) is `RC4(key, data)` from a fresh state. This closes the spec's
//!   open item 1.

use md5::{Digest, Md5};
use rsa::{BigUint, Pkcs1v15Encrypt, RsaPublicKey};

use crate::zt::*;
use crate::Hresult;

/// `NTE_BAD_DATA` (the provider's error for a key blob it cannot read; a negative value, so
/// `FUN_100048d0` returns it unchanged).
// open: the exact Win32 error `CryptImportKey` reports for a bad magic, a bit length that is not a
// multiple of 8, a short blob and a key length outside the provider's range (the documentation
// [ms-docs: cryptimportkey-errors.md] names none for these; `NTE_BAD_DATA` is the closest).
pub const NTE_BAD_DATA: Hresult = Hresult(0x8009_0005);
/// `NTE_BAD_TYPE`: "The key BLOB type is not supported by this CSP" [ms-docs: cryptimportkey-errors.md].
pub const NTE_BAD_TYPE: Hresult = Hresult(0x8009_000A);
/// `NTE_BAD_VER`: "The version number of the key BLOB does not match the CSP version".
pub const NTE_BAD_VER: Hresult = Hresult(0x8009_0007);
/// `CALG_RSA_SIGN`.
pub const CALG_RSA_SIGN: u32 = 0x2400;
/// The Base provider's RSA key exchange key lengths in bits [ms-docs: base-provider-algorithms.md]:
/// 384 to 1024; signature keys 384 to 16384.
pub const KEYX_BITS: std::ops::RangeInclusive<usize> = 384..=1024;
pub const SIGN_BITS: std::ops::RangeInclusive<usize> = 384..=16384;

pub const CALG_RC4: u32 = 0x6801;
pub const CALG_RSA_KEYX: u32 = 0xa400;
pub const PUBLICKEYBLOB: u8 = 6;
pub const SIMPLEBLOB: u8 = 1;
pub const RSA1_MAGIC: u32 = 0x3141_5352;

/// RC4 (the Base Provider's stream cipher), keyed per call.
pub fn rc4(key: &[u8], data: &mut [u8]) {
    let mut s: [u8; 256] = std::array::from_fn(|i| i as u8);
    let mut j = 0u8;
    for i in 0..256 {
        j = j.wrapping_add(s[i]).wrapping_add(key[i % key.len()]);
        s.swap(i, j as usize);
    }
    let (mut i, mut j) = (0u8, 0u8);
    for b in data.iter_mut() {
        i = i.wrapping_add(1);
        j = j.wrapping_add(s[i as usize]);
        s.swap(i as usize, j as usize);
        *b ^= s[s[i as usize].wrapping_add(s[j as usize]) as usize];
    }
}

/// The random source (`CryptGenRandom`).
pub type RandomSource = Box<dyn FnMut(&mut [u8])>;

/// An `RngCore` over the injected source, for the PKCS #1 padding.
struct SourceRng<'a>(&'a mut RandomSource);

impl rand_core::RngCore for SourceRng<'_> {
    fn next_u32(&mut self) -> u32 {
        let mut b = [0u8; 4];
        (self.0)(&mut b);
        u32::from_le_bytes(b)
    }
    fn next_u64(&mut self) -> u64 {
        let mut b = [0u8; 8];
        (self.0)(&mut b);
        u64::from_le_bytes(b)
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (self.0)(dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        (self.0)(dest);
        Ok(())
    }
}

impl rand_core::CryptoRng for SourceRng<'_> {}

/// The operating system's random bytes.
pub fn os_random() -> RandomSource {
    Box::new(|buf: &mut [u8]| {
        // open: CryptGenRandom cannot fail here; a failing OS source is not handled by the port.
        getrandom::getrandom(buf).expect("operating system random source");
    })
}

/// The crypto object at `CGun+0x7c` (`FUN_10004170`/`FUN_100041c0(obj, "GunKey", 1)`): the provider
/// context and the imported server key (`obj+0xc`).
pub struct CryptContext {
    server_key: Option<RsaPublicKey>,
    /// The imported key's `aiKeyAlg` was `CALG_RSA_KEYX`; only an exchange key wraps a session key.
    server_key_is_keyx: bool,
}

/// `{ cryptoCtx*, HCRYPTKEY }` (the 8-byte session key object of `FUN_10004530`): the derived
/// 40-bit RC4 key.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionKey {
    /// The 5 key bytes (the first bytes of the MD5).
    key: [u8; 5],
    /// The 11 salt bytes: zero without `CRYPT_CREATE_SALT`, the hash's next 11 bytes with it.
    salt: [u8; 11],
}

impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionKey(..)")
    }
}

impl SessionKey {
    /// The RC4 key: the 5 key bytes followed by the 11 salt bytes (see the module docs).
    pub fn rc4_key(&self) -> [u8; 16] {
        let mut k = [0u8; 16];
        k[..5].copy_from_slice(&self.key);
        k[5..].copy_from_slice(&self.salt);
        k
    }

    /// `CryptDeriveKey(CALG_RC4, md5, (40 << 16) | flags)` on the Base provider: the first 5 hash
    /// bytes are the key and the salt is the next 11 with `CRYPT_CREATE_SALT`, else zero.
    pub fn from_md5(hash: &[u8; 16], create_salt: bool) -> SessionKey {
        let mut key = [0u8; 5];
        key.copy_from_slice(&hash[..5]);
        let mut salt = [0u8; 11];
        if create_salt {
            salt.copy_from_slice(&hash[5..]);
        }
        SessionKey { key, salt }
    }

    /// `FUN_10004630`: `CryptEncrypt(.., Final = TRUE, ..)` in place; the key stream restarts for every
    /// call [ms-docs: cryptencrypt.md].
    pub fn encrypt(&self, data: &mut [u8]) {
        rc4(&self.rc4_key(), data);
    }
}

impl Default for CryptContext {
    fn default() -> Self {
        CryptContext::new()
    }
}

impl CryptContext {
    pub fn new() -> CryptContext {
        CryptContext {
            server_key: None,
            server_key_is_keyx: false,
        }
    }

    /// `FUN_100048d0(ctx, &RSAKey)`: `CryptImportKey(hProv, blob, len, 0, 0)` of the server's
    /// PUBLICKEYBLOB [ms-docs: base-provider-key-blobs.md]. The previous key is replaced.
    pub fn import_public_key(&mut self, blob: &[u8]) -> Hresult {
        match parse_public_key_blob(blob) {
            Ok((k, keyx)) => {
                self.server_key = Some(k);
                self.server_key_is_keyx = keyx;
                Hresult::S_OK
            }
            Err(e) => e,
        }
    }

    /// `FUN_10004940(ctx, 4, &buf)`: `CryptGenRandom` of the 4-byte client random.
    pub fn gen_random_u32(&mut self, random: &mut RandomSource) -> u32 {
        let mut b = [0u8; 4];
        random(&mut b);
        u32::from_le_bytes(b)
    }

    /// `FUN_10004cd0(ctx, &key, clientRandom, serverRandom)` -> `FUN_10004bd0` -> `FUN_10004560`:
    /// MD5 over the two dwords as they lie in memory (little endian, client first), then
    /// `CryptDeriveKey(CALG_RC4, hash, 0x280001)`: the first 5 bytes of the hash [ms-docs:
    /// cryptderivekey.md].
    pub fn derive_session_key(&self, client_random: u32, server_random: u32) -> SessionKey {
        let mut buf = [0u8; 8];
        buf[..4].copy_from_slice(&client_random.to_le_bytes());
        buf[4..].copy_from_slice(&server_random.to_le_bytes());
        let hash: [u8; 16] = Md5::digest(buf).into();
        SessionKey::from_md5(&hash, false)
    }

    /// `FUN_10004a30(sessionKey, &blob)`: `CryptExportKey(key, serverKey, SIMPLEBLOB)`.
    pub fn export_simple_blob(
        &mut self,
        key: &SessionKey,
        random: &mut RandomSource,
    ) -> Result<Vec<u8>, Hresult> {
        let Some(server) = self.server_key.as_ref().filter(|_| self.server_key_is_keyx) else {
            // CryptExportKey with no key exchange key (none imported, or a CALG_RSA_SIGN key)
            // open: the exact error (NTE_NO_KEY / NTE_BAD_KEY) is not documented for this case.
            return Err(NTE_BAD_DATA);
        };
        let ct = {
            let mut rng = SourceRng(random);
            server
                .encrypt(&mut rng, Pkcs1v15Encrypt, &key.key)
                .map_err(|_| NTE_BAD_DATA)?
        };
        let mut out = Vec::with_capacity(12 + ct.len());
        out.extend_from_slice(&[SIMPLEBLOB, 2, 0, 0]);
        out.extend_from_slice(&CALG_RC4.to_le_bytes());
        out.extend_from_slice(&CALG_RSA_KEYX.to_le_bytes());
        // the ciphertext is returned in little-endian format [ms-docs: cryptencrypt.md]
        out.extend(ct.iter().rev());
        Ok(out)
    }
}

/// A PUBLICKEYBLOB as the Base provider's `CryptImportKey` takes it [ms-docs:
/// base-provider-key-blobs.md, base-provider-algorithms.md, cryptimportkey-errors.md]: `bType` 6
/// (else `NTE_BAD_TYPE`), `bVersion` 2 (else `NTE_BAD_VER`), `aiKeyAlg` `CALG_RSA_KEYX` or
/// `CALG_RSA_SIGN` (else `NTE_BAD_DATA`, "the algorithm ... is not supported by this CSP"), magic
/// `RSA1`, a bit length in the algorithm's range (keyx 384..1024, sign 384..16384, multiples of 8).
/// Returns the key and whether it is an exchange key.
fn parse_public_key_blob(blob: &[u8]) -> Result<(RsaPublicKey, bool), Hresult> {
    if blob.len() < 20 {
        return Err(NTE_BAD_DATA);
    }
    if blob[0] != PUBLICKEYBLOB {
        return Err(NTE_BAD_TYPE);
    }
    if blob[1] != 2 {
        return Err(NTE_BAD_VER);
    }
    let alg = u32::from_le_bytes(blob[4..8].try_into().map_err(|_| NTE_BAD_DATA)?);
    let keyx = match alg {
        CALG_RSA_KEYX => true,
        CALG_RSA_SIGN => false,
        _ => return Err(NTE_BAD_DATA),
    };
    let magic = u32::from_le_bytes(blob[8..12].try_into().map_err(|_| NTE_BAD_DATA)?);
    let bitlen = u32::from_le_bytes(blob[12..16].try_into().map_err(|_| NTE_BAD_DATA)?) as usize;
    let pubexp = u32::from_le_bytes(blob[16..20].try_into().map_err(|_| NTE_BAD_DATA)?);
    let range = if keyx { KEYX_BITS } else { SIGN_BITS };
    if magic != RSA1_MAGIC
        || !range.contains(&bitlen)
        || !bitlen.is_multiple_of(8)
        || blob.len() < 20 + bitlen / 8
    {
        return Err(NTE_BAD_DATA);
    }
    let n = BigUint::from_bytes_le(&blob[20..20 + bitlen / 8]);
    let e = BigUint::from(pubexp);
    RsaPublicKey::new_with_max_size(n, e, 16384)
        .map(|k| (k, keyx))
        .map_err(|_| NTE_BAD_DATA)
}

/// Builds a PUBLICKEYBLOB (the server side of `RSAKey`; used by the tests and fake servers).
pub fn build_public_key_blob(modulus_be: &[u8], pubexp: u32) -> Vec<u8> {
    build_public_key_blob_alg(modulus_be, pubexp, CALG_RSA_KEYX)
}

/// [`build_public_key_blob`] with an explicit `aiKeyAlg`.
pub fn build_public_key_blob_alg(modulus_be: &[u8], pubexp: u32, alg: u32) -> Vec<u8> {
    let mut out = vec![PUBLICKEYBLOB, 2, 0, 0];
    out.extend_from_slice(&alg.to_le_bytes());
    out.extend_from_slice(&RSA1_MAGIC.to_le_bytes());
    out.extend_from_slice(&((modulus_be.len() * 8) as u32).to_le_bytes());
    out.extend_from_slice(&pubexp.to_le_bytes());
    out.extend(modulus_be.iter().rev());
    out
}

/// `ZT_E_NOT_CONNECTED` when no session key exists (`FUN_10001a40`, `FUN_10001b00`: `CGun+0x80 == 0`).
pub fn require_key(key: &Option<SessionKey>) -> Result<&SessionKey, Hresult> {
    key.as_ref().ok_or(ZT_E_NOT_CONNECTED)
}
