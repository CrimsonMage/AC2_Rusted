//! `PropertyCollection` as an `EntityDesc` streams it (its `StreamPack` is the folded
//! one-member pack of `m_propertyGroup`): `PSmartArray<PropertyGroup>::StreamPack` (a u32
//! count), each `PropertyGroup::StreamPack` (group name, then `PSmartArray<BaseProperty>`), each
//! `BaseProperty::StreamPack` (property name, then the value). The value's layout comes from
//! `MasterProperty::InqPropertyType` for the name (`BaseProperty::SetPropertyName`), a table the
//! DAT holds, so decoding takes a [`PropertyTypes`] resolver. Admin 7148 and Legions 7160 are
//! instruction-identical (`BaseProperty::StreamPack` Legions 0x0064B4E0,
//! `MasterProperty::AllocatePropertyValue` 0x0064DBF0).

use crate::protocol::fx::Waveform;
use crate::protocol::reader::Reader;
use crate::protocol::string_info::StringInfo;
use crate::protocol::types::Position;
use crate::protocol::ProtocolError;

/// `MasterProperty::InqPropertyType`: the value type a property name streams as (the
/// `0x4000_00xx` enum `MasterProperty::AllocatePropertyValue` switches on), or `None` when the
/// name is not in the table. Any `Fn(u32) -> Option<u32>` is a resolver.
pub trait PropertyTypes {
    fn property_type(&self, name: u32) -> Option<u32>;
}

impl<F: Fn(u32) -> Option<u32>> PropertyTypes for F {
    fn property_type(&self, name: u32) -> Option<u32> {
        self(name)
    }
}

/// A property value, by the class `MasterProperty::AllocatePropertyValue` makes.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// `0x40000001` `BoolPropertyValue`.
    Bool(u32),
    /// `0x40000002` `IntegerPropertyValue`.
    Integer(i32),
    /// `0x40000003` `FloatPropertyValue`.
    Float(f32),
    /// `0x40000004` `VectorPropertyValue`: three floats.
    Vector([f32; 3]),
    /// `0x40000005` `ColorPropertyValue`: `RGBAColor::StreamPack`, the packed 32-bit colour.
    Color(u32),
    /// `0x40000006` `StringPropertyValue`.
    String(Vec<u8>),
    /// `0x40000007` `EnumPropertyValue`.
    Enum(u32),
    /// `0x40000008` `DataFilePropertyValue`: a DID.
    DataFile(u32),
    /// `0x40000009` `WaveformPropertyValue`: the raw 0x2C-byte `Waveform`.
    Waveform(Waveform),
    /// `0x4000000A` `StringInfoPropertyValue`.
    StringInfo(Box<StringInfo>),
    /// `0x4000000B` `PackageIDPropertyValue`.
    PackageId(u32),
    /// `0x4000000D` `LongIntegerPropertyValue`.
    LongInteger(i64),
    /// `0x4000000E` `PositionPropertyValue`: `Position::UnPack`.
    Position(Position),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: u32,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyGroup {
    pub name: u32,
    pub properties: Vec<Property>,
}

fn value(r: &mut Reader<'_>, kind: u32) -> Result<PropertyValue, ProtocolError> {
    Ok(match kind {
        0x4000_0001 => PropertyValue::Bool(r.u32()?),
        0x4000_0002 => PropertyValue::Integer(r.u32()? as i32),
        0x4000_0003 => PropertyValue::Float(r.f32()?),
        0x4000_0004 => PropertyValue::Vector(r.vec3()?),
        0x4000_0005 => PropertyValue::Color(r.u32()?),
        0x4000_0006 => PropertyValue::String(r.string()?),
        0x4000_0007 => PropertyValue::Enum(r.u32()?),
        0x4000_0008 => PropertyValue::DataFile(r.u32()?),
        0x4000_0009 => PropertyValue::Waveform(Waveform::read(r)?),
        0x4000_000A => PropertyValue::StringInfo(Box::new(StringInfo::read(r)?)),
        0x4000_000B => PropertyValue::PackageId(r.u32()?),
        0x4000_000D => PropertyValue::LongInteger(r.u64()? as i64),
        0x4000_000E => PropertyValue::Position(Position::read(r)?),
        // `AllocatePropertyValue` makes nothing, so `BaseProperty::StreamPack` fails.
        _ => return Err(ProtocolError::Invalid("property value type")),
    })
}

/// Reads a `PropertyCollection`.
pub(crate) fn read_property_collection(
    r: &mut Reader<'_>,
    types: &dyn PropertyTypes,
) -> Result<Vec<PropertyGroup>, ProtocolError> {
    (0..r.count()?)
        .map(|_| {
            let name = r.u32()?;
            let properties = (0..r.count()?)
                .map(|_| {
                    let name = r.u32()?;
                    let kind = types
                        .property_type(name)
                        .ok_or(ProtocolError::Invalid("property not in MasterProperty"))?;
                    Ok(Property {
                        name,
                        value: value(r, kind)?,
                    })
                })
                .collect::<Result<_, ProtocolError>>()?;
            Ok(PropertyGroup { name, properties })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{read_property_collection, PropertyValue};
    use crate::protocol::reader::Reader;

    #[test]
    fn values_follow_the_resolved_types() {
        let types = |name: u32| match name {
            0x10 => Some(0x4000_0003),
            0x11 => Some(0x4000_0004),
            0x12 => Some(0x4000_000D),
            _ => None,
        };
        let mut b = Vec::new();
        for w in [1u32, 0x2000_0001, 3, 0x10] {
            b.extend(w.to_le_bytes());
        }
        b.extend(0.5f32.to_le_bytes());
        b.extend(0x11u32.to_le_bytes());
        for v in [1.0f32, 2.0, 3.0] {
            b.extend(v.to_le_bytes());
        }
        b.extend(0x12u32.to_le_bytes());
        b.extend((-2i64).to_le_bytes());
        let mut r = Reader::new(&b);
        let groups = read_property_collection(&mut r, &types).unwrap();
        r.finish().unwrap();
        assert_eq!(groups[0].name, 0x2000_0001);
        let values: Vec<_> = groups[0].properties.iter().map(|p| &p.value).collect();
        assert_eq!(
            values,
            [
                &PropertyValue::Float(0.5),
                &PropertyValue::Vector([1.0, 2.0, 3.0]),
                &PropertyValue::LongInteger(-2)
            ]
        );
        let unknown = |_: u32| None;
        assert!(read_property_collection(&mut Reader::new(&b), &unknown).is_err());
    }
}
