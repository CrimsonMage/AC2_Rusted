//! The 13 classes the first catalogue left open (final messages audit N4). Expected texts are the DLL's
//! own render output as the audit recorded it (`spec/audit_final_messages.md` 3.4) and, for the
//! `HostData` block, the order of the hash tables `FUN_1000c8a0` / `FUN_1000c670` (bucket `key & 7`,
//! newest first) with the attribute hash `LAB_10035880`.

use crate::gundll::msg::parse_message;
use crate::zt::*;
use crate::Hresult;

fn round(xml: &str) -> String {
    let mut b = xml.as_bytes().to_vec();
    b.push(0);
    parse_message(&b)
        .unwrap_or_else(|e| panic!("{xml}: {e:?}"))
        .render()
        .unwrap()
}

#[test]
fn view_list_result_and_the_generic_row_classes() {
    assert_eq!(
        round(
            r#"<ViewListRes Cx="0x1" HR="0x0" Cnt="2" ><ViewDesc User="a" ItemId="3" /><ViewDesc User="b&amp;" ItemId="4" /></ViewListRes> "#
        ),
        r#"<ViewListRes Cx="0x1" HR="0x0" Cnt="2" ><ViewDesc User="a" ItemId="3" /><ViewDesc User="b&amp;" ItemId="4" /></ViewListRes> "#
    );
    // every attribute of a Row is printed under the literal name A; a row without attributes counts in
    // Cnt and prints nothing
    let page = r#"<PageRes Cx="0x1" HR="0x0" Vid="5" VIdx="1" VTotal="9" ><Row x="1" y="2" /><Row /><row q="z" /></PageRes> "#;
    assert_eq!(
        round(page),
        r#"<PageRes Cx="0x1" HR="0x0" Cnt="3" Vid="5" VIdx="1" VTotal="9" ><Row A="1" A="2" /><Row A="z" /></PageRes> "#
    );
    for el in ["RowPgRes", "MPageRes", "MRowPageRes", "MPageWFrdsRes"] {
        let x = format!(
            r#"<{el} Cx="0x1" HR="0x0" Cnt="1" Vid="1" VIdx="2" VTotal="3" ><Row k="v" /></{el}> "#
        );
        assert!(round(&x).contains(r#"<Row A="v" />"#), "{el}");
    }
    // HdrRowRes / MGetHeadRowRes: the Hdrs child may be missing
    assert_eq!(
        round(r#"<HdrRowRes Cx="0x1" HR="0x0" ><Hdrs a="1" b="2" /></HdrRowRes> "#),
        r#"<HdrRowRes Cx="0x1" HR="0x0" ><Hdrs A="1" A="2" /></HdrRowRes> "#
    );
    assert_eq!(
        round(r#"<MGetHeadRowRes Cx="0x1" HR="0x0" ></MGetHeadRowRes> "#),
        r#"<MGetHeadRowRes Cx="0x1" HR="0x0" ></MGetHeadRowRes> "#
    );
    let row = |n: &str| {
        let langs = [
            "English",
            "French",
            "German",
            "Korean",
            "Japanese",
            "Taiwanese",
            "Portuguese",
            "Italian",
            "Spanish",
            "Dutch",
        ];
        let l: String = langs.iter().map(|x| format!(r#"{x}="{n}" "#)).collect();
        format!(r#"<ViewDesc Vid="7" VType="2" {l}/>"#)
    };
    let x = format!(
        r#"<MGetViewListRes Cx="0x1" HR="0x0" Cnt="1" >{}</MGetViewListRes> "#,
        row("t")
    );
    assert_eq!(round(&x), x);
}

#[test]
fn directory_classes_carry_a_guid_child() {
    let reg = r#"<DsRegisterServer Cx="0x1" Namespace="ns" Address="1.2.3.4" Port="80" ><Guid Val="{0123abcd-0000-1111-2222-aabbccddeeff}"/></DsRegisterServer> "#;
    assert_eq!(
        round(reg),
        r#"<DsRegisterServer Cx="0x1" Namespace="ns" Address="1.2.3.4" Port="80" ><Guid Val="{0123ABCD-0000-1111-2222-AABBCCDDEEFF}"/></DsRegisterServer> "#
    );
    let ev = r#"<DsAddServerEv Category="1" Address="a" Port="2" SId="3" ><Guid Val="{0123ABCD-0000-1111-2222-AABBCCDDEEFF}"/></DsAddServerEv> "#;
    assert_eq!(round(ev), ev);
    // CLSIDFromString failure
    let bad = reg.replace("{0123abcd", "0123abcd");
    let mut b = bad.into_bytes();
    b.push(0);
    assert_eq!(parse_message(&b).unwrap_err(), Hresult(0x8004_01F3));
    // a missing child is ZT_E_NOT_FOUND
    let mut b = r#"<DsAddServerEv Category="1" />"#.as_bytes().to_vec();
    b.push(0);
    assert_eq!(parse_message(&b).unwrap_err(), ZT_E_NOT_FOUND);
}

#[test]
fn host_data_is_new_mod_and_del_with_hash_table_order() {
    // items: bucket id & 7, newest first within a bucket (ids 1 and 9 share bucket 1, 9 was added
    // later); attributes: bucket = hash(name) & 7 (hash adds isupper per character), so `a` (0) comes
    // before `B` (1) whatever the document order
    let xml = r#"<HostData Cx="0x1" ><HostData><New><Item ItemId="1" B="y" a="x" /><Item ItemId="9" /><Item ItemId="2" /></New><Mod></Mod><Del><Item Num="4" /><Item Num="-5" /></Del></HostData></HostData>"#;
    assert_eq!(
        round(xml),
        r#"<HostData Cx="0x1" ><HostData ><New ><Item ItemId="9" /><Item ItemId="1" a="x" B="y" /><Item ItemId="2" /></New> <Mod ></Mod> <Del ><Item Num="4" /><Item Num="-5" /></Del> </HostData> </HostData> "#
    );
    // the same name again replaces the value (FUN_10034e60 finds it with lstrcmpiA)
    let dup = r#"<HostData Cx="0x1" ><HostData><New><Item ItemId="1" a="x" A="z" /></New><Mod/><Del/></HostData></HostData>"#;
    assert!(round(dup).contains(r#"<Item ItemId="1" a="z" />"#));
    // a missing New/Mod/Del is ZT_E_NOT_FOUND; an Item without ItemId is tolerated (block NULL)
    let mut b = r#"<HostData Cx="0x1" ><HostData><New/></HostData></HostData>"#
        .as_bytes()
        .to_vec();
    b.push(0);
    assert_eq!(parse_message(&b).unwrap_err(), ZT_E_NOT_FOUND);
    let noid = r#"<HostData Cx="0x1" ><HostData><New><Item a="1" /></New><Mod/><Del/></HostData></HostData>"#;
    assert!(round(noid).contains("<New ></New> "));
}

#[test]
fn host_data_attributes_are_ordered_by_the_tolower_hash() {
    // LAB_10035880: h = h * 0x11 + tolower(c) over <= 16 characters, bucket h & 7, buckets 0..7,
    // newest first inside one: "x" (0x78 & 7 = 0) before "a" (0x61 & 7 = 1) before "B" (0x62 & 7 = 2);
    // "a" and "i" (0x69 & 7 = 1) share a bucket: newest first
    let xml = r#"<HostData Cx="0x1" ><HostData><New><Item ItemId="1" B="1" a="2" x="3" /></New><Mod/><Del/></HostData></HostData>"#;
    assert!(round(xml).contains(r#"<Item ItemId="1" x="3" a="2" B="1" />"#));
    let same = r#"<HostData Cx="0x1" ><HostData><New><Item ItemId="1" a="2" i="3" /></New><Mod/><Del/></HostData></HostData>"#;
    assert!(round(same).contains(r#"<Item ItemId="1" i="3" a="2" />"#));
}

#[test]
fn m_host_data_stores_and_renders_the_block_it_read() {
    // CMsg_MHostData::InitFromPacket stores IpAddr, the block and ChanId (asm 0x10025c4c-0x10025c55)
    let xml = r#"<MHostData Cx="0x1" IpAddr="1.2.3.4" ChanId="5" ><HostData><New><Item ItemId="1" a="x" /></New><Mod/><Del><Item Num="7" /></Del></HostData></MHostData>"#;
    assert_eq!(
        round(xml),
        r#"<MHostData Cx="0x1" IpAddr="1.2.3.4" ChanId="5" ><HostData ><New ><Item ItemId="1" a="x" /></New> <Mod ></Mod> <Del ><Item Num="7" /></Del> </HostData> </MHostData> "#
    );
}

#[test]
fn view_list_rows_sit_at_their_child_index() {
    let row = r#"<ViewDesc User="a" ItemId="3" />"#;
    let x = format!(r#"<ViewListRes Cx="0x1" HR="0x0" Cnt="2" ><Other/>{row}{row}</ViewListRes> "#);
    assert_eq!(
        round(&x),
        r#"<ViewListRes Cx="0x1" HR="0x0" Cnt="2" ><ViewDesc User="(null)" ItemId="0" /><ViewDesc User="a" ItemId="3" /></ViewListRes> "#
    );
}

#[test]
fn an_item_without_its_key_attribute_leaves_a_null_block() {
    // N10: accepted by Init (0xC1120007 is tolerated), the DLL's render then faults; the port prints
    // the empty block
    let noid =
        r#"<HostData Cx="0x1" ><HostData><New/><Mod/><Del><Item /></Del></HostData></HostData>"#;
    assert!(round(noid).contains("<Del ></Del> "));
}
