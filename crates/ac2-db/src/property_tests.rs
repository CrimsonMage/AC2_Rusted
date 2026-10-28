//! The ethereal blocking mask a `PropertyCollection` sets (`CPhysicsObj::ApplyEtherealProperty`),
//! over synthetic `MasterProperty` descs.

use crate::property::{ethereal_mask_with, Ac2PropertyDesc, Ac2PropertyGroup, Ac2PropertyValue};
use crate::property::{ETHEREAL_PROPERTY_GROUP, REPORT_INQUIRABLE};

const BOOL: u32 = 0x4000_0001;
const INTEGER: u32 = 0x4000_0002;
const OTHER_GROUP: u32 = 0x4000_0002;
const NAME_BIT_3: u32 = 0x1000_0003;
const NAME_BIT_9: u32 = 0x1000_0009;
const NAME_INTEGER: u32 = 0x1000_0010;
const NAME_OTHER_GROUP: u32 = 0x1000_0020;
const NAME_BIG_BIT: u32 = 0x1000_0030;

fn desc_of(name: u32) -> Option<Ac2PropertyDesc> {
    match name {
        NAME_BIT_3 => Some(Ac2PropertyDesc {
            value_type: BOOL,
            group: ETHEREAL_PROPERTY_GROUP,
            data: 3,
            report_to_weenie: REPORT_INQUIRABLE,
        }),
        NAME_BIT_9 => Some(Ac2PropertyDesc {
            value_type: BOOL,
            group: ETHEREAL_PROPERTY_GROUP,
            data: 9,
            report_to_weenie: REPORT_INQUIRABLE,
        }),
        NAME_INTEGER => Some(Ac2PropertyDesc {
            value_type: INTEGER,
            group: ETHEREAL_PROPERTY_GROUP,
            data: 5,
            report_to_weenie: REPORT_INQUIRABLE,
        }),
        NAME_OTHER_GROUP => Some(Ac2PropertyDesc {
            value_type: BOOL,
            group: OTHER_GROUP,
            data: 7,
            report_to_weenie: REPORT_INQUIRABLE,
        }),
        NAME_BIG_BIT => Some(Ac2PropertyDesc {
            value_type: BOOL,
            group: ETHEREAL_PROPERTY_GROUP,
            data: 70,
            report_to_weenie: REPORT_INQUIRABLE,
        }),
        _ => None,
    }
}

fn group(names: &[u32]) -> Ac2PropertyGroup {
    Ac2PropertyGroup {
        name: 1,
        properties: names
            .iter()
            .map(|&name| Ac2PropertyValue {
                name,
                value_type: BOOL,
                bytes: Box::new([1, 0, 0, 0]),
            })
            .collect(),
    }
}

/// A group of bool properties with explicit packed values (`word`).
fn valued_group(values: &[(u32, u32)]) -> Ac2PropertyGroup {
    Ac2PropertyGroup {
        name: 1,
        properties: values
            .iter()
            .map(|&(name, word)| Ac2PropertyValue {
                name,
                value_type: BOOL,
                bytes: Box::new(word.to_le_bytes()),
            })
            .collect(),
    }
}

#[test]
fn a_false_bool_ethereal_property_sets_no_bit() {
    let mask = ethereal_mask_with(
        &[valued_group(&[(NAME_BIT_3, 0), (NAME_BIT_9, 1)])],
        desc_of,
    );
    assert_eq!(mask, 1 << 9);
    let mask = ethereal_mask_with(&[valued_group(&[(NAME_BIT_3, 0)])], desc_of);
    assert_eq!(mask, 0);
}

#[test]
fn a_bool_ethereal_property_sets_its_desc_bit() {
    let mask = ethereal_mask_with(&[group(&[NAME_BIT_3, NAME_BIT_9])], desc_of);
    assert_eq!(mask, (1 << 3) | (1 << 9));
}

#[test]
fn other_groups_integers_unknown_names_and_bits_past_63_set_nothing() {
    let mask = ethereal_mask_with(
        &[group(&[
            NAME_INTEGER,
            NAME_OTHER_GROUP,
            NAME_BIG_BIT,
            0x1000_0099,
        ])],
        desc_of,
    );
    assert_eq!(mask, 0);
}

#[test]
fn several_groups_join_their_bits() {
    let mask = ethereal_mask_with(
        &[group(&[NAME_BIT_3]), group(&[NAME_BIT_9, NAME_INTEGER])],
        desc_of,
    );
    assert_eq!(mask, (1 << 3) | (1 << 9));
}

/// A `PStringBase<char>` as the `MasterProperty` record packs it: u16 length, the scrambled
/// bytes, and padding to a 4-byte boundary.
fn packed_pstring(text: &[u8]) -> Vec<u8> {
    let mut scrambled = text.to_vec();
    ac2_core::pstring::munge(&mut scrambled);
    let mut out = (text.len() as u16).to_le_bytes().to_vec();
    out.extend_from_slice(&scrambled);
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
    out
}

fn master_property_bytes(names: &[(u32, &[u8])], descs: &[[u32; 6]]) -> Vec<u8> {
    let word = |v: u32| v.to_le_bytes();
    let mut b = Vec::new();
    b.extend_from_slice(&word(0x4000_0042));
    b.extend_from_slice(&word(0));
    b.extend_from_slice(&word(0));
    b.extend_from_slice(&word(names.len() as u32));
    for (id, text) in names {
        b.extend_from_slice(&word(*id));
        b.extend(packed_pstring(text));
    }
    b.extend_from_slice(&word(descs.len() as u32));
    for desc in descs {
        for v in desc {
            b.extend_from_slice(&word(*v));
        }
    }
    b
}

#[test]
fn master_property_names_decode_and_look_up_both_ways() {
    use crate::ResourcePackedDecode;
    let qdid = ac2_core::QualifiedDataId::new(ac2_core::DataId::new(0x4000_0042), 0x4000_0042);
    let bytes = master_property_bytes(
        &[(0x1000_0001, b"Alpha"), (0x1000_0002, b"Beta!")],
        // name, type, group, data, report to weenie, required
        &[[0x1000_0001, 0x4000_0003, 0x4000_0002, 0, 0, 0]],
    );
    let types = crate::Ac2PropertyTypes::decode_packed(qdid, 0, &bytes).expect("decodes");
    assert_eq!(types.name(0x1000_0001), Some(&b"Alpha"[..]));
    assert_eq!(types.name(0x1000_0002), Some(&b"Beta!"[..]));
    assert_eq!(types.name(0x1000_0009), None);
    assert_eq!(types.name_to_enum(b"Beta!"), Some(0x1000_0002));
    assert_eq!(types.name_to_enum(b"Gamma"), None);
    let desc = types.desc(0x1000_0001).expect("desc");
    assert_eq!((desc.value_type, desc.group), (0x4000_0003, 0x4000_0002));
    assert!(types.desc(0x1000_0002).is_none());
}

#[test]
fn master_property_name_record_must_end_where_its_descs_do() {
    use crate::ResourcePackedDecode;
    let qdid = ac2_core::QualifiedDataId::new(ac2_core::DataId::new(0x4000_0042), 0x4000_0042);
    let mut bytes = master_property_bytes(&[(1, b"A")], &[]);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    assert!(crate::Ac2PropertyTypes::decode_packed(qdid, 0, &bytes).is_err());
}
