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
