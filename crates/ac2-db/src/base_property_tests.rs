//! `BaseProperty` and `BasePropertyValue` semantics: typed inquiry and set by type, the
//! same-type copy, per-subclass equality and the type change.

use ac2_math::waveform::kind;
use ac2_math::{AFrame, Quat, Vec3, Waveform};

use crate::base_property::{BaseProperty, BasePropertyValue, PropertyPosition, RgbaColor};
use crate::property::{
    TYPE_BOOL, TYPE_COLOR, TYPE_DATA_FILE, TYPE_ENUM, TYPE_FLOAT, TYPE_INTEGER, TYPE_LONG_INTEGER,
    TYPE_PACKAGE_ID, TYPE_POSITION, TYPE_STRING, TYPE_VECTOR, TYPE_WAVEFORM,
};

#[test]
fn every_master_property_type_has_a_value_class_of_that_type() {
    let types = [
        TYPE_BOOL,
        TYPE_INTEGER,
        TYPE_FLOAT,
        TYPE_VECTOR,
        TYPE_COLOR,
        TYPE_STRING,
        TYPE_ENUM,
        TYPE_DATA_FILE,
        TYPE_WAVEFORM,
        TYPE_PACKAGE_ID,
        TYPE_LONG_INTEGER,
        TYPE_POSITION,
    ];
    for t in types {
        let v = BasePropertyValue::new_for_type(t).expect("value class");
        assert_eq!(v.value_type(), t);
    }
    // 0x4000000A (StringInfo) has no value class in this port yet.
    assert!(BasePropertyValue::new_for_type(0x4000_000A).is_none());
    assert!(BasePropertyValue::new_for_type(0).is_none());
}

#[test]
fn fresh_values_match_their_constructors() {
    assert_eq!(
        BasePropertyValue::new_for_type(TYPE_BOOL),
        Some(BasePropertyValue::Bool(false))
    );
    assert_eq!(
        BasePropertyValue::new_for_type(TYPE_COLOR),
        Some(BasePropertyValue::Color(RgbaColor::WHITE))
    );
    let w = match BasePropertyValue::new_for_type(TYPE_WAVEFORM) {
        Some(BasePropertyValue::Waveform(w)) => w,
        other => panic!("waveform expected, got {other:?}"),
    };
    assert_eq!(w.kind, kind::NONE);
    assert_eq!((w.amplitude, w.frequency), (1.0, 1.0));
}

#[test]
fn inquiry_answers_only_for_its_own_type() {
    let b = BasePropertyValue::Bool(true);
    assert_eq!(b.inq_bool(), Some(true));
    assert_eq!(b.inq_integer(), None);
    assert_eq!(b.inq_float(), None);
    let i = BasePropertyValue::Integer(-7);
    assert_eq!(i.inq_integer(), Some(-7));
    assert_eq!(i.inq_enum(), None);
    let s = BasePropertyValue::String(b"abc".to_vec());
    assert_eq!(s.inq_string(), Some(&b"abc"[..]));
    assert_eq!(s.inq_vector(), None);
}

#[test]
fn set_writes_only_into_a_value_of_its_own_type() {
    let mut v = BasePropertyValue::Float(1.0);
    assert!(v.set_float(2.5));
    assert_eq!(v.inq_float(), Some(2.5));
    assert!(!v.set_integer(9));
    assert_eq!(v, BasePropertyValue::Float(2.5));
    assert!(!v.set_string(b"hi"));
    assert_eq!(v.inq_string(), None);
}

#[test]
fn set_from_copies_a_same_type_value_only() {
    let mut dst = BasePropertyValue::Enum(1);
    assert!(dst.set_from(&BasePropertyValue::Enum(5)));
    assert_eq!(dst.inq_enum(), Some(5));
    assert!(!dst.set_from(&BasePropertyValue::PackageId(9)));
    assert_eq!(dst.inq_enum(), Some(5));
}

#[test]
fn integer_and_long_integer_compare_exactly() {
    let a = BasePropertyValue::LongInteger(1 << 40);
    assert!(a.is_equal(&BasePropertyValue::LongInteger(1 << 40)));
    assert!(!a.is_equal(&BasePropertyValue::LongInteger((1 << 40) + 1)));
    assert!(!BasePropertyValue::Integer(3).is_equal(&BasePropertyValue::Enum(3)));
}

#[test]
fn float_compares_exactly_but_vector_within_the_tolerance() {
    assert!(!BasePropertyValue::Float(1.0).is_equal(&BasePropertyValue::Float(1.0001)));
    let a = BasePropertyValue::Vector(Vec3::new(1.0, 2.0, 3.0));
    assert!(a.is_equal(&BasePropertyValue::Vector(Vec3::new(1.0001, 2.0, 3.0))));
    assert!(!a.is_equal(&BasePropertyValue::Vector(Vec3::new(1.001, 2.0, 3.0))));
}

#[test]
fn color_compares_each_channel_within_the_tolerance() {
    let white = BasePropertyValue::Color(RgbaColor::WHITE);
    let grey = BasePropertyValue::Color(RgbaColor {
        r: 0.5,
        ..RgbaColor::WHITE
    });
    assert!(white.is_equal(&BasePropertyValue::Color(RgbaColor {
        a: 1.0001,
        ..RgbaColor::WHITE
    })));
    assert!(!white.is_equal(&grey));
}

#[test]
fn waveform_compares_kind_exactly_and_terms_within_the_tolerance() {
    let base = Waveform::constant(0.5);
    let mut near = base;
    near.phase = 0.0001;
    let mut other_kind = base;
    other_kind.kind = kind::SINE;
    let a = BasePropertyValue::Waveform(base);
    assert!(a.is_equal(&BasePropertyValue::Waveform(near)));
    assert!(!a.is_equal(&BasePropertyValue::Waveform(other_kind)));
}

#[test]
fn position_compares_cell_exactly_and_frame_within_the_tolerance() {
    let frame = AFrame {
        origin: Vec3::new(10.0, 0.0, 0.0),
        orientation: Quat::IDENTITY,
    };
    let p = BasePropertyValue::Position(PropertyPosition {
        cell: 0x1234_0001,
        frame,
    });
    let moved = AFrame {
        origin: Vec3::new(10.0001, 0.0, 0.0),
        ..frame
    };
    assert!(p.is_equal(&BasePropertyValue::Position(PropertyPosition {
        cell: 0x1234_0001,
        frame: moved
    })));
    assert!(!p.is_equal(&BasePropertyValue::Position(PropertyPosition {
        cell: 0x1234_0002,
        frame
    })));
}

#[test]
fn string_and_data_file_compare_exactly() {
    assert!(BasePropertyValue::String(b"a".to_vec())
        .is_equal(&BasePropertyValue::String(b"a".to_vec())));
    assert!(!BasePropertyValue::String(b"a".to_vec())
        .is_equal(&BasePropertyValue::String(b"b".to_vec())));
    assert!(!BasePropertyValue::DataFile(1).is_equal(&BasePropertyValue::DataFile(2)));
}

#[test]
fn base_property_type_change_allocates_a_fresh_value() {
    let mut p = BaseProperty::new(0x1000_0001);
    assert!(p.value.is_none());
    assert!(p.set_property_type(TYPE_INTEGER));
    assert!(p.value.as_ref().and_then(BasePropertyValue::inq_integer) == Some(0));
    if let Some(v) = p.value.as_mut() {
        v.set_integer(42);
    }
    assert!(
        p.set_property_type(TYPE_INTEGER),
        "same type keeps the value"
    );
    assert_eq!(
        p.value.as_ref().and_then(BasePropertyValue::inq_integer),
        Some(42)
    );
    assert!(p.set_property_type(TYPE_FLOAT));
    assert_eq!(p.value, Some(BasePropertyValue::Float(0.0)));
    assert!(!p.set_property_type(0x4000_00FF));
    assert!(p.value.is_none());
}

#[test]
fn base_property_compare_needs_name_type_and_value() {
    let mut a = BaseProperty::new(7);
    a.set_property_type(TYPE_ENUM);
    let mut b = a.clone();
    assert!(a.is_equal(&b));
    if let Some(v) = b.value.as_mut() {
        v.set_enum(3);
    }
    assert!(!a.is_equal(&b));
    let other_name = BaseProperty {
        name: 8,
        ..a.clone()
    };
    assert!(!a.is_equal(&other_name));
}
