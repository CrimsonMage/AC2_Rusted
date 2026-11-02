//! Byte-exact renders (`RenderUnicode` templates of `0a_receive_messages.md` Part 2) of the generic
//! writer: header forms, attribute order, self-closing versus container, the trailing spaces, blob
//! children, list rows, `Cnt`, escaping and the 0xFFFF-character cap.

use crate::gundll::catalog::{all, by_id, Header};
use crate::gundll::msg::{GunMsg, Value};
use crate::zt::*;
use crate::Hresult;

fn msg(id: u32) -> GunMsg {
    GunMsg::new(by_id(id).unwrap())
}

#[test]
fn request_header_and_self_closing_form() {
    let mut m = msg(0x4101);
    m.cx = 0x2f4a10;
    m.set_str("User", Some("Bob@Shard"));
    assert_eq!(
        m.render().unwrap(),
        r#"<FriendAdd Cx="0x2f4a10" User="Bob@Shard" />"#
    );
}

#[test]
fn request_without_attributes() {
    let mut m = msg(0x4100);
    m.cx = 0;
    assert_eq!(m.render().unwrap(), r#"<GetFriendsList Cx="0x0" />"#);
}

#[test]
fn response_header_has_cx_and_hr_in_lower_case_hex() {
    let mut m = msg(0x4201);
    m.cx = 0xABCDEF;
    m.hr = 0xC1120007;
    assert_eq!(
        m.render().unwrap(),
        r#"<FriendAddRes Cx="0xabcdef" HR="0xc1120007" />"#
    );
}

#[test]
fn event_has_no_header_and_container_rows_end_with_a_space() {
    let mut m = msg(0x430d);
    m.set_str("User", Some("Bob@Shard"));
    m.set_dec("Mode", 1);
    assert_eq!(
        m.render().unwrap(),
        r#"<FriendStat User="Bob@Shard" Mode="1" />"#
    );
    let mut m = msg(0x1300);
    assert_eq!(m.render().unwrap(), "<ConnectEv />");
    m.cx = 5;
    assert_eq!(m.render().unwrap(), "<ConnectEv />");
}

#[test]
fn blob_children_use_the_6_bit_text_and_a_container() {
    let mut m = msg(0x511d);
    m.cx = 0x2f4a10;
    m.set_str("User", Some("Bob@Shard"));
    m.set_str("Text", Some("hi"));
    m.set_blob("Extra", None);
    assert_eq!(
        m.render().unwrap(),
        r#"<SendToUserEx Cx="0x2f4a10" User="Bob@Shard" Text="hi" ><Extra Val=""/></SendToUserEx> "#
    );
    m.set_blob("Extra", Some(&[1]));
    assert!(m.render().unwrap().contains(r#"<Extra Val="ba"/>"#));
}

#[test]
fn list_rows_and_cnt() {
    let mut m = msg(0x4200);
    m.cx = 0x2f4a10;
    m.hr = 0;
    m.set_rows(
        "FriendStatus",
        vec![vec![
            Value::Str(Some("Bob@Shard".into())),
            Value::Dec(1),
            Value::Str(Some("".into())),
        ]],
    );
    assert_eq!(
        m.render().unwrap(),
        r#"<GetFriendsListRes Cx="0x2f4a10" HR="0x0" Cnt="1" ><FriendStatus User="Bob@Shard" Mode="1" Location="" /></GetFriendsListRes> "#
    );
    let m = msg(0x4200);
    assert_eq!(
        m.render().unwrap(),
        r#"<GetFriendsListRes Cx="0x0" HR="0x0" Cnt="0" ></GetFriendsListRes> "#
    );
}

#[test]
fn values_are_escaped_and_signed_decimals_printed() {
    let mut m = msg(0x5105);
    m.set_dec("RoomId", -3);
    m.set_str("Text", Some("a\"b&c<d>e"));
    assert_eq!(
        m.render().unwrap(),
        r#"<SendToRoom Cx="0x0" RoomId="-3" Text="a&quot;b&amp;c&lt;d&gt;e" />"#
    );
}

#[test]
fn a_null_string_prints_null() {
    // [inferred] the MSVC CRT prints a NULL %ls as "(null)"
    let m = msg(0x4101);
    assert_eq!(
        m.render().unwrap(),
        r#"<FriendAdd Cx="0x0" User="(null)" />"#
    );
}

#[test]
fn the_render_buffer_holds_at_most_0xffff_characters() {
    let mut m = msg(0x5105);
    // fixed overhead with an empty (but present) Text
    m.set_str("Text", Some(""));
    let base = m.render().unwrap().encode_utf16().count();
    let fits = 0xFFFF - base;
    m.set_str("Text", Some(&"x".repeat(fits)));
    assert_eq!(m.render().unwrap().encode_utf16().count(), 0xFFFF);
    m.set_str("Text", Some(&"x".repeat(fits + 1)));
    assert_eq!(m.render(), Err(ZT_E_STR_TOO_LONG));
}

/// Every catalogued class renders, parses back to the same fields, and its id is the table's.
#[test]
fn every_class_round_trips_through_render_and_parse() {
    use crate::gundll::msg::parse_message;
    for def in all() {
        if def.id == 0x1302 || def.id == 0x2200 {
            continue; // not in the 216-entry table: raised locally only
        }
        if crate::gundll::catalog::LOCAL_ONLY.contains(&def.id) {
            // the factory has no case: rejected from the wire
            let mut m = GunMsg::new(def);
            m.cx = 0x1234;
            let mut bytes = m.render().unwrap().into_bytes();
            bytes.push(0);
            assert_eq!(parse_message(&bytes).unwrap_err(), Hresult(0x8007_000E));
            continue;
        }
        let mut m = GunMsg::new(def);
        m.cx = 0x1234;
        if def.header == Header::Response {
            m.hr = 0x8000_0001;
        }
        let text = m.render().unwrap();
        let mut bytes = text.into_bytes();
        bytes.push(0);
        let back = parse_message(&bytes).unwrap_or_else(|e| panic!("{}: {e:?}", def.element));
        assert_eq!(back.def.id, def.id, "{}", def.element);
        match def.header {
            Header::Event => {}
            Header::Request => assert_eq!(back.cx, 0x1234, "{}", def.element),
            Header::Response => {
                assert_eq!((back.cx, back.hr), (0x1234, 0x8000_0001), "{}", def.element)
            }
        }
    }
}

#[test]
fn the_catalogue_matches_the_216_entry_table() {
    use crate::gundll::catalog::{element_id, is_open};
    let mut open = 0;
    for def in all() {
        if def.id == 0x1302 || def.id == 0x2200 {
            assert_eq!(
                element_id(def.element),
                None,
                "{} is not in the table",
                def.element
            );
            continue;
        }
        assert_eq!(element_id(def.element), Some(def.id), "{}", def.element);
    }
    for id in [0x3100u32, 0x3301, 0x4106, 0x4206, 0x4107, 0x4207] {
        assert!(is_open(id), "{id:#x}");
        open += 1;
    }
    assert_eq!(open, 6);
    // 210 catalogued classes + the 6 ids without a class = the 216 table entries
    assert_eq!(all().count() - 2 + open, 216);
}

#[test]
fn every_class_has_its_render_and_init_addresses() {
    // vtable slots 0 and 2 (final messages audit N6); LoginRes 0x10017be0 / 0x10017b60
    for def in all() {
        assert!(def.render != 0 && def.init != 0, "{}", def.element);
    }
    assert_eq!(
        (by_id(0x2200).unwrap().render, by_id(0x2200).unwrap().init),
        (0x10017be0, 0x10017b60)
    );
    let ps = by_id(0x7204).unwrap();
    assert_eq!((ps.render, ps.init), (0x10024580, 0x100243d0));
    let ds = by_id(0xc300).unwrap();
    assert_eq!((ds.render, ds.init), (0x10029d80, 0x10029ba0));
}
