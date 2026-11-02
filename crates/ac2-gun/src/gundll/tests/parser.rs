//! The XML parser's rules (`0a_receive_messages.md` 1.3 and `verify_0a_receive_messages.md`):
//! double quotes only, the four entities, name characters, 63 attributes, depth 128, trailing bytes
//! ignored, `_stricmp` lookups, the `IXmlPacket` accessors and their error codes.

use crate::gundll::xml_parse::XmlPacket;
use crate::zt::*;

fn parse(s: &str) -> Result<XmlPacket, crate::Hresult> {
    let w: Vec<u16> = s.encode_utf16().collect();
    XmlPacket::parse(&w)
}

fn val(p: &XmlPacket, n: Option<usize>, name: &str) -> String {
    String::from_utf16_lossy(p.attr_string(n, name).unwrap())
}

#[test]
fn a_message_with_attributes_children_and_trailing_space() {
    let p = parse(r#"<GetFriendsListRes Cx="0x1" HR="0x0" Cnt="2" ><FriendStatus User="a" Mode="1" Location="x" /><FriendStatus User="b" Mode="2" Location="" /></GetFriendsListRes> "#).unwrap();
    assert_eq!(p.name(None), "GetFriendsListRes");
    assert_eq!(p.attr_count(None), 3);
    assert_eq!(p.child_count(None), 2);
    let c1 = p.child(None, 1).unwrap();
    assert_eq!(p.name(Some(c1)), "FriendStatus");
    assert_eq!(val(&p, Some(c1), "User"), "b");
    assert_eq!(p.attr_decimal(Some(c1), "Mode"), Ok(2));
    assert_eq!(p.attr_hex(None, "Cx"), Ok(1));
    // lookups are case-insensitive; the first duplicate wins
    assert_eq!(p.attr_decimal(None, "cnt"), Ok(2));
    assert_eq!(
        p.find_child(None, "friendstatus"),
        Ok(p.child(None, 0).unwrap())
    );
}

#[test]
fn accessor_error_codes() {
    let p = parse(r#"<A x="1" ><B y="2" /></A>"#).unwrap();
    assert_eq!(p.attr_string(None, "nope"), Err(ZT_E_ATTR_NOT_FOUND));
    assert_eq!(p.child(None, 1), Err(ZT_E_INDEX_TOO_GREAT));
    assert_eq!(p.attr_by_index(None, 1), Err(ZT_E_INDEX_TOO_GREAT));
    assert_eq!(p.find_child(None, "C"), Err(ZT_E_NOT_FOUND));
    assert_eq!(p.attr_by_index(None, 0).unwrap().0, "x");
}

#[test]
fn duplicate_attributes_first_wins() {
    let p = parse(r#"<A x="1" x="2" />"#).unwrap();
    assert_eq!(p.attr_decimal(None, "x"), Ok(1));
}

#[test]
fn only_double_quotes_and_four_entities() {
    assert!(parse("<A x='1' />").is_err());
    assert!(parse("<A x=1 />").is_err());
    let p = parse(r#"<A x="&quot;&amp;&lt;&gt;" y="a&amp;amp;b" />"#).unwrap();
    assert_eq!(val(&p, None, "x"), "\"&<>");
    assert_eq!(val(&p, None, "y"), "a&amp;b");
    assert_eq!(parse(r#"<A x="&apos;" />"#).unwrap_err(), ZT_E_BAD_AMP_CODE);
    assert_eq!(parse(r#"<A x="&#65;" />"#).unwrap_err(), ZT_E_BAD_AMP_CODE);
    // case sensitive
    assert_eq!(parse(r#"<A x="&QUOT;" />"#).unwrap_err(), ZT_E_BAD_AMP_CODE);
    // a `>` or a newline inside a value is fine, a `<` is not
    let p = parse("<A x=\"1>\n2\" />").unwrap();
    assert_eq!(val(&p, None, "x"), "1>\n2");
    assert!(parse("<A x=\"1<2\" />").is_err());
}

#[test]
fn names_start_with_a_letter_and_use_letters_digits_underscore() {
    assert!(parse("<A_1 b_2=\"x\" />").is_ok());
    assert!(parse("<1A />").is_err());
    assert!(parse("<_A />").is_err());
    assert!(parse("<A-B />").is_err());
    assert!(parse("<A b-c=\"x\" />").is_err());
    assert!(parse("<A:b />").is_err());
}

#[test]
fn unsupported_constructs_are_errors() {
    assert!(parse("<?xml version=\"1.0\"?><A />").is_err());
    assert!(parse("<!-- c --><A />").is_err());
    assert!(parse("<!DOCTYPE A><A />").is_err());
    assert!(parse("<A><![CDATA[x]]></A>").is_err());
}

#[test]
fn white_space_between_tokens() {
    let p = parse("  \r\n\t<A\tx = \"1\"\n y=\"2\" >\n</A >").unwrap();
    assert_eq!(p.attr_decimal(None, "y"), Ok(2));
    // the tokenizer would allow it, but the descent (FUN_10039240) only accepts white space, `>` or
    // `/>` after an attribute: a name straight after a closing quote is a syntax error
    assert!(parse("<A x=\"1\"y=\"2\"/>").is_err());
    assert!(parse("<A x=\"1\"/>").is_ok());
}

#[test]
fn sixty_three_attributes_fit_sixty_four_do_not() {
    let make = |n: usize| {
        let attrs: String = (0..n).map(|i| format!("a{i}=\"{i}\" ")).collect();
        format!("<A {attrs}/>")
    };
    assert_eq!(parse(&make(63)).unwrap().attr_count(None), 63);
    assert_eq!(parse(&make(64)).unwrap_err(), ZT_E_TOO_MANY_ATTRIBUTES);
}

#[test]
fn nesting_is_limited_to_128() {
    let make = |n: usize| format!("{}{}", "<A>".repeat(n), "</A>".repeat(n));
    assert!(parse(&make(128)).is_ok());
    assert_eq!(parse(&make(129)).unwrap_err(), ZT_E_NESTING_TOO_DEEP);
}

#[test]
fn end_tags_match_case_sensitively() {
    assert_eq!(parse("<A></a>").unwrap_err(), ZT_E_MISMATCHED_XML_TAGS);
    assert_eq!(
        parse("<A><B></A></B>").unwrap_err(),
        ZT_E_MISMATCHED_XML_TAGS
    );
    // an element name that is too long: 127 characters
    let long = "A".repeat(127);
    assert_eq!(
        parse(&format!("<{long} />")).unwrap_err(),
        ZT_E_STR_TOO_LONG
    );
    assert!(parse(&format!("<{} />", "A".repeat(126))).is_ok());
}

#[test]
fn trailing_bytes_after_the_root_are_ignored() {
    assert!(parse("<A /> trailing <junk").is_ok());
    assert!(parse("<A x=\"1\"></A>\u{1}\u{2}garbage").is_ok());
    assert!(parse("<A />").is_ok());
}

#[test]
fn unterminated_documents_fail_with_the_descent_hresults() {
    assert_eq!(parse("").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A x=\"1").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A x=").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A>").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A><B></B>").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A>text").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<A></").unwrap_err(), ZT_E_MISMATCHED_XML_TAGS);
    // a truncated END-tag name: the name token's length is a pointer difference, far over 127
    assert_eq!(parse("<A></A").unwrap_err(), ZT_E_STR_TOO_LONG);
    assert_eq!(parse("<A></A ").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
}

#[test]
fn white_space_after_lt_is_not_tolerated() {
    // `< a>`: the token after `<` is white space, not a name: syntax error
    assert_eq!(parse("< a></a>").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    // `</ a>`: FUN_10039470 records the error but returns 0; the open element is then reported by
    // FUN_10033450
    assert_eq!(parse("<a></ a>").unwrap_err(), ZT_E_MISMATCHED_XML_TAGS);
    assert_eq!(parse("<a></>").unwrap_err(), ZT_E_MISMATCHED_XML_TAGS);
}

#[test]
fn the_end_callback_runs_after_the_next_token_was_read() {
    // `</a b>`: the callback sees the name `a` (the token read is white space) and succeeds; then the
    // `b` is not a `>`: syntax error
    assert_eq!(parse("<a></a b>").unwrap_err(), ZT_E_XML_SYNTAX_ERROR);
    assert_eq!(parse("<a></b c>").unwrap_err(), ZT_E_MISMATCHED_XML_TAGS);
}

#[test]
fn a_bad_entity_in_text_is_ignored_and_the_text_stays_unset() {
    // FUN_100393c0 discards the text callback's result
    let p = parse("<A>&bogus;</A>").unwrap();
    assert_eq!(p.text(None), None);
    let p = parse("<A>fine &lt;</A>").unwrap();
    assert_eq!(String::from_utf16_lossy(p.text(None).unwrap()), "fine <");
    // an attribute value is different: the error propagates
    assert_eq!(parse("<A x=\"&bogus;\" />").unwrap_err(), ZT_E_BAD_AMP_CODE);
}

#[test]
fn text_content_is_kept_and_children_may_not_follow_it() {
    let p = parse("<A>hello &amp; bye</A>").unwrap();
    assert_eq!(
        String::from_utf16_lossy(p.text(None).unwrap()),
        "hello & bye"
    );
    // the descent takes text OR children (FUN_100393c0 / FUN_100391f0)
    assert!(parse("<A>text<B /></A>").is_err());
    assert!(parse("<A><B />text</A>").is_err());
    assert!(parse("<A><B /><C /></A>").is_ok());
}

#[test]
fn non_ascii_is_allowed_in_values_only() {
    let p = parse("<A x=\"h\u{e9}llo \u{4e16}\" />").unwrap();
    assert_eq!(val(&p, None, "x"), "h\u{e9}llo \u{4e16}");
    assert!(parse("<A\u{e9} />").is_err());
}

#[test]
fn tag_name_lookup_is_case_insensitive() {
    use crate::gundll::catalog::{by_element, element_id};
    assert_eq!(element_id("friendadd"), Some(0x4101));
    assert_eq!(element_id("FRIENDADD"), Some(0x4101));
    assert_eq!(element_id("NoSuchMessage"), None);
    assert_eq!(by_element("joinroomres").unwrap().id, 0x5201);
}

#[test]
fn attribute_names_up_to_127_characters_parse_and_128_fail_with_insufficient_buffer() {
    // 0x10033470 gives FUN_100024a0 the TAG name's length for each attribute; the real limit is the
    // 0x80-byte WideCharToMultiByte buffer of FUN_10002320 (ERROR_INSUFFICIENT_BUFFER, 0x8007007A)
    let ok = format!("<A {}=\"1\" />", "b".repeat(127));
    assert!(parse(&ok).is_ok());
    let bad = format!("<A {}=\"1\" />", "b".repeat(128));
    assert_eq!(parse(&bad).unwrap_err(), crate::Hresult(0x8007_007A));
    // tag names keep their limit: 126 ok, 127 STR_TOO_LONG
    assert!(parse(&format!("<{} />", "A".repeat(126))).is_ok());
    assert_eq!(
        parse(&format!("<{} />", "A".repeat(127))).unwrap_err(),
        ZT_E_STR_TOO_LONG
    );
}
