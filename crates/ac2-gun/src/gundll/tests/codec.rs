//! The blob codec, Base64, the escape function, the decimal and hex accessors
//! (`0a_send_session.md` section 7, `0a_receive_messages.md` 1.3 and the verifier's corrections).

use crate::gundll::blob::*;
use crate::gundll::msg::escape;
use crate::gundll::xml_parse::{parse_decimal, parse_hex};
use crate::zt::*;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[test]
fn alphabet_is_the_dll_string() {
    assert_eq!(
        std::str::from_utf8(ALPHABET).unwrap(),
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!."
    );
}

/// The spec's hand examples (7.2): `[01]` -> "ba", `FF FF FF` -> "....".
#[test]
fn encode_examples_and_tail_groups() {
    assert_eq!(encode_for_attribute(&[]).unwrap(), "");
    assert_eq!(encode_for_attribute(&[0x01]).unwrap(), "ba");
    assert_eq!(encode_for_attribute(&[0xFF, 0xFF, 0xFF]).unwrap(), "....");
    // 2 bytes -> 3 characters, 1 byte -> 2: v = 0x0201 -> 1, 8, 0
    assert_eq!(encode_for_attribute(&[0x01, 0x02]).unwrap(), "bia");
    // 4 bytes: a full group then a 1-byte tail group (b3 = 0x03: 3, 0)
    assert_eq!(
        encode_for_attribute(&[0xFF, 0xFF, 0xFF, 0x03]).unwrap(),
        "....da"
    );
    // no padding characters
    assert!(!encode_for_attribute(&[1, 2, 3, 4, 5])
        .unwrap()
        .contains('='));
}

#[test]
fn encode_checks_space_for_the_nul() {
    // `(offset + L + 1) >= outSize` is ZT_E_BUFFER_TOO_SMALL
    assert_eq!(encode(&[1, 2, 3], 4), Err(ZT_E_BUFFER_TOO_SMALL));
    assert_eq!(encode(&[1, 2, 3], 5).unwrap().len(), 4);
    assert_eq!(encode(&[1, 2, 3, 4], 6), Err(ZT_E_BUFFER_TOO_SMALL));
    assert_eq!(encode(&[1, 2, 3, 4], 7).unwrap().len(), 6);
}

#[test]
fn decode_inverts_encode_for_every_tail() {
    for n in 0..40usize {
        let data: Vec<u8> = (0..n).map(|i| (i * 37 + 11) as u8).collect();
        let text = encode_for_attribute(&data).unwrap();
        assert_eq!(decode(&wide(&text), 1000).unwrap(), data, "n={n}");
    }
}

#[test]
fn decode_errors() {
    // a character outside the alphabet
    assert_eq!(decode(&wide("b-"), 10), Err(ZT_E_INVALID_CHARACTER));
    assert_eq!(decode(&wide("b="), 10), Err(ZT_E_INVALID_CHARACTER));
    // non-zero leftover bits: "bb" = 1 + (1 << 6) = 65 -> byte 0x41 then 4 bits of 0 ... "bb" leaves 0
    // but "bc" leaves bits: 1 + (2 << 6) = 129 -> byte 0x81, leftover 0x0 -> ok; "b." leaves 63 >> 2
    assert!(decode(&wide("bc"), 10).is_ok());
    assert_eq!(decode(&wide("b."), 10), Err(ZT_E_INVALID_CHARACTER));
    // output full
    assert_eq!(decode(&wide("...."), 2), Err(ZT_E_BUFFER_TOO_SMALL));
    // stops at the NUL
    assert_eq!(decode(&[0x62, 0x61, 0, 0x2d], 10).unwrap(), vec![1]);
}

#[test]
fn char_values_match_the_table() {
    assert_eq!(char_value('a' as u16), Ok(0));
    assert_eq!(char_value('z' as u16), Ok(25));
    assert_eq!(char_value('A' as u16), Ok(26));
    assert_eq!(char_value('Z' as u16), Ok(51));
    assert_eq!(char_value('0' as u16), Ok(52));
    assert_eq!(char_value('9' as u16), Ok(61));
    assert_eq!(char_value('!' as u16), Ok(62));
    assert_eq!(char_value('.' as u16), Ok(63));
    assert_eq!(char_value('+' as u16), Err(ZT_E_INVALID_CHARACTER));
}

#[test]
fn escape_replaces_exactly_four_characters() {
    assert_eq!(escape("a\"b&c<d>e'f\tg"), "a&quot;b&amp;c&lt;d&gt;e'f\tg");
    assert_eq!(escape("plain \u{e9}"), "plain \u{e9}");
    assert_eq!(escape("&amp;"), "&amp;amp;");
}

/// 1.3 s14: a single `-` anywhere; "5-" is -55; `+` and spaces are rejected; empty is 0.
#[test]
fn decimal_accessor_quirks() {
    assert_eq!(parse_decimal(&wide("")), Ok(0));
    assert_eq!(parse_decimal(&wide("42")), Ok(42));
    assert_eq!(parse_decimal(&wide("-5")), Ok(-5));
    assert_eq!(parse_decimal(&wide("5-")), Ok(-55));
    assert_eq!(parse_decimal(&wide("--5")), Err(ZT_E_INVALID_CHARACTER));
    assert_eq!(parse_decimal(&wide("+5")), Err(ZT_E_INVALID_CHARACTER));
    assert_eq!(parse_decimal(&wide(" 5")), Err(ZT_E_INVALID_CHARACTER));
    // no overflow check: 32-bit wrap
    assert_eq!(parse_decimal(&wide("4294967296")), Ok(0));
}

/// 1.3 s1c: lower-case hex with an optional `0x`; upper case gives wrong values; `X` is rejected;
/// an `x` re-adds the previous digit.
#[test]
fn hex_accessor_quirks() {
    assert_eq!(parse_hex(&wide("0x1f")), Ok(0x1f));
    assert_eq!(parse_hex(&wide("0x2f4a10")), Ok(0x2f4a10));
    assert_eq!(parse_hex(&wide("0xc1120007")), Ok(0xc1120007));
    assert_eq!(parse_hex(&wide("0x0")), Ok(0));
    // 32-bit wrap
    assert_eq!(parse_hex(&wide("0x1ffffffff")), Ok(0xffffffff));
    // upper case digits map to c - 0x57: 'A' = -22
    assert_eq!(parse_hex(&wide("0xA")), Ok((-22i32) as u32));
    assert_eq!(parse_hex(&wide("0X1f")), Err(ZT_E_INVALID_CHARACTER));
    assert_eq!(parse_hex(&wide("-1")), Err(ZT_E_INVALID_CHARACTER));
    // "1x": acc = 1, then `x` adds the previous digit again: 1 * 16 + 1
    assert_eq!(parse_hex(&wide("1x")), Ok(17));
}

/// `FUN_10002e90`: standard Base64, with the reported length of the spec's 7.4 (right for 2 padding
/// characters). The IV of a ticket is 16 bytes: 22 characters, `==`.
#[test]
fn base64_decodes_a_16_byte_iv() {
    let iv: Vec<u8> = (0u8..16).collect();
    // standard Base64 of 00 01 .. 0f
    let text = b"AAECAwQFBgcICQoLDA0ODw==";
    let (out, len) = base64_decode(text, text.len() as u32 + 1);
    assert_eq!(&out[..16], &iv[..]);
    // 5 full groups (15) + a 2-character tail: 15 + (4 - 2) - 1 = 16 (7.4: right for `==`)
    assert_eq!(len, 16);
}

#[test]
fn base64_unpadded_and_invalid() {
    // "TWFu" = "Man"
    let (out, _) = base64_decode(b"TWFu", 5);
    assert_eq!(&out[..3], b"Man");
}

#[test]
fn extract_tag_finds_text_up_to_the_next_lt() {
    let doc = b"<ticket><IV>AAEC</iv><encryptedText>QUJD</encryptedText></ticket>";
    let (t, n) = extract_tag("<iv>", doc).unwrap();
    assert_eq!(t, b"AAEC");
    assert_eq!(n, 5);
    let (t, n) = extract_tag("<encryptedText>", doc).unwrap();
    assert_eq!(t, b"QUJD");
    assert_eq!(n, 5);
    assert_eq!(
        extract_tag("<missing>", doc),
        Err(crate::Hresult(0x8000_FFFF))
    );
}
