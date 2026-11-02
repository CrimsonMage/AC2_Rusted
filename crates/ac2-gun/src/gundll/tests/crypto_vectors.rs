//! Known-answer vectors from Wine's rsaenh test suite (`msdocs/wine/`, [wine-tests]) for the
//! CryptoAPI behaviour `crypto.rs` relies on: the Base provider's 40-bit RC4 key (key || salt), and the
//! SIMPLEBLOB byte order and contents. The expected values are the arrays in Wine's tests, not output
//! of this code.

use md5::{Digest, Md5};
use rsa::{BigUint, Pkcs1v15Encrypt, RsaPrivateKey};

use crate::gundll::crypto::{rc4, SessionKey};

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

/// `test_rc4()`: MD5 of the 2000 bytes `0, 1, .. (i as u8)`, a 40-bit RC4 key with CRYPT_CREATE_SALT,
/// 16 bytes of 0xAF encrypted with Final = TRUE; the Base provider's answer.
#[test]
fn rc4_40_bit_with_created_salt_is_keyed_with_key_and_salt() {
    let buf: Vec<u8> = (0..2000usize).map(|i| i as u8).collect();
    let hash: [u8; 16] = Md5::digest(&buf).into();
    let mut data = [0xAFu8; 16];
    SessionKey::from_md5(&hash, true).encrypt(&mut data);
    let rc4_40_salt_base = [
        0x2F, 0xAC, 0xEA, 0xEA, 0xFF, 0x68, 0x7E, 0x77, 0xF4, 0xB9, 0x48, 0x7C, 0x4E, 0x79, 0xA6,
        0xB5,
    ];
    assert_eq!(data, rc4_40_salt_base);
    // the Enhanced provider (no salt) is RC4 over the first 5 bytes only
    let rc4_40_salt = [
        0x41, 0xE6, 0x33, 0xC9, 0x50, 0xA1, 0xBF, 0x88, 0x12, 0x4D, 0xD3, 0xE3, 0x47, 0x88, 0x6D,
        0xA5,
    ];
    let mut data = [0xAFu8; 16];
    rc4(&hash[..5], &mut data);
    assert_eq!(data, rc4_40_salt);
}

const PRIVATE_KEY_BLOB: &str = concat!(
    "0702000000a400005253413200040000010001009b64efce317cad56e21e9b96",
    "b3f029886ea8c21133d6cc8c69b21afdfc2321304d2945b63a6711801a91f29f",
    "01acc011505fcdb9ad769f6e915571da9796962275b48344899ef844407cd6cd",
    "9d88d688bc56b764e92c242f0d78551cb267b15ebc0ccf1ce9d39ea2152473d6",
    "db6f83b2f8bce7473b01ef490898d6a3f92557e9393c53301bf2c96231435d84",
    "2430219aaddb6291c807d92fd6b5376ffe7a12bcd9d22bbfd7b1fa7dc048dd74",
    "dd5504a18bc10ac4a55762ee088bf9196c5206f8730f24c9719fc545173eae06",
    "81a2964006bfeb9e802b27208f38cfebff3b3841356966131d3c013bf637ca9c",
    "617498cfc96ee890c7b733c0073cf8c8f6f2d7f02162588a55bfa12d3da669c5",
    "021931f0940f455c951b53bcf5b01a8fbf40e0c773e7726eebb10f38c5f8ee04",
    "ed341a10f95334f33ee65cd14765cdbdf106cbb4b126399f71fe3df862ab228b",
    "0edcb9e87406fc8c25a1a9cf07f9ac21017b1cdc94bd47e1a08659356a6fb970",
    "267c3cfdbd81393642c2bdbe84279a6981da9927c24f6233f47930c5635471f1",
    "4722259b6c002f1cf41f85bcf6676ae3f6558aefd00bd3a2c55170150af0984c",
    "b719620e2d2a4a7d7a0ac417e35d2052a998c3aa11f6bf4c94998189f07f66aa",
    "c888d7318471b66409760b7f1a1f2efecd592a541184d46a61dfaa76669d8211",
    "563dd252e6425a77929834f3566c9610405916cb7761e3bf4bd439fbb14ec174",
    "ec7aea3d68bb0be6c606bfdd7f9442c00fe492336c6e1bba73f97984df4500e4",
    "94889d0889cff2a4c547458586a5cca8f25d5807",
);

const SIMPLE_BLOB: &str = concat!(
    "010200000168000000a40000b8a4df5e9eb1bf853d242d1e69b767138e78f2df",
    "c669ce467ef2f233206fa1a5598325cb3ab18a1263023cfb4afaef8ef72957b1",
    "9ea7f302fdcadf5a1f71b626092439dac0de2a0ecd1fe5b64f82a0a990d3d96a",
    "43142af7acd5a05493c4b9e724844d695ecc2a32b5fbe4b408d536585940fb29",
    "7fa71725c40d7837048c4992",
);

/// `test_import_private()`: a SIMPLEBLOB made on Windows, opened with its private key.
#[test]
fn simpleblob_is_little_endian_and_holds_just_the_key_bytes() {
    let p = hex(PRIVATE_KEY_BLOB);
    assert_eq!(p.len(), 596);
    // PRIVATEKEYBLOB: header (8), RSAPUBKEY (12), modulus 128, prime1 64, prime2 64, exponent1 64,
    // exponent2 64, coefficient 64, privateExponent 128 (all little endian)
    let le = |a: usize, b: usize| BigUint::from_bytes_le(&p[a..b]);
    let key = RsaPrivateKey::from_components(
        le(20, 148),
        BigUint::from(0x10001u32),
        le(468, 596),
        vec![le(148, 212), le(212, 276)],
    )
    .unwrap();
    let blob = hex(SIMPLE_BLOB);
    assert_eq!(blob.len(), 140);
    // BLOBHEADER { SIMPLEBLOB 1, version 2, 0, CALG_RC4 } and ALG_ID CALG_RSA_KEYX
    assert_eq!(
        &blob[..12],
        &[1, 2, 0, 0, 0x01, 0x68, 0, 0, 0x00, 0xa4, 0, 0]
    );
    let mut ct = blob[12..].to_vec();
    ct.reverse();
    let session = key.decrypt(Pkcs1v15Encrypt, &ct).unwrap();
    assert_eq!(session, hex("d5 e1 a7 e9 ac 38 c1"), "the key bytes only");
    let mut msg = hex("40 64 28 e8 8a e7 a4 d4 1c fd de 71");
    rc4(&session, &mut msg);
    assert_eq!(msg, b"Wine rocks!\0");
}

/// Our export decodes the way Windows' does: the same procedure opens it.
#[test]
fn our_simpleblob_has_the_windows_layout() {
    use crate::gundll::crypto::{build_public_key_blob, CryptContext};
    let p = hex(PRIVATE_KEY_BLOB);
    let key = RsaPrivateKey::from_components(
        BigUint::from_bytes_le(&p[20..148]),
        BigUint::from(0x10001u32),
        BigUint::from_bytes_le(&p[468..596]),
        vec![
            BigUint::from_bytes_le(&p[148..212]),
            BigUint::from_bytes_le(&p[212..276]),
        ],
    )
    .unwrap();
    let mut ctx = CryptContext::new();
    // the PUBLICKEYBLOB from the private key's public half
    let pubblob = build_public_key_blob(
        &p[20..148].iter().rev().copied().collect::<Vec<_>>(),
        0x10001,
    );
    assert_eq!(&pubblob[..8], &[6, 2, 0, 0, 0, 0xa4, 0, 0]);
    assert!(ctx.import_public_key(&pubblob).succeeded());
    let sk = ctx.derive_session_key(1, 2);
    let mut rng: crate::gundll::crypto::RandomSource = super::fakes::test_random();
    let blob = ctx.export_simple_blob(&sk, &mut rng).unwrap();
    assert_eq!(blob.len(), 140);
    assert_eq!(
        &blob[..12],
        &[1, 2, 0, 0, 0x01, 0x68, 0, 0, 0x00, 0xa4, 0, 0]
    );
    let mut ct = blob[12..].to_vec();
    ct.reverse();
    assert_eq!(key.decrypt(Pkcs1v15Encrypt, &ct).unwrap().len(), 5);
}
