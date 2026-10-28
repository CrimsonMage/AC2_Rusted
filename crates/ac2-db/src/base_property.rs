//! `BaseProperty` and its typed `BasePropertyValue` (`property/baseproperty.cpp`,
//! `property/masterproperty.cpp`): one named property holds one value of its `MasterProperty`
//! type. The `Inq*` getters answer only for their own type, as the retail base class's virtuals
//! do (a wrong-type inquiry leaves the out-value untouched and returns false), and `Set*` writes
//! only into a value of its own type. Equality is `BasePropertyValue::Compare` per subclass:
//! floats and integers compare exactly, while vectors, colours, waveforms and frames compare
//! within `0.0002` per component.

use ac2_math::waveform::kind;
use ac2_math::{AFrame, Quat, Vec3, Waveform};

use crate::property::{
    TYPE_BOOL, TYPE_COLOR, TYPE_DATA_FILE, TYPE_ENUM, TYPE_FLOAT, TYPE_INTEGER, TYPE_LONG_INTEGER,
    TYPE_PACKAGE_ID, TYPE_POSITION, TYPE_STRING, TYPE_VECTOR, TYPE_WAVEFORM,
};

/// The per-component tolerance of `VectorPropertyValue`, `ColorPropertyValue`,
/// `WaveformPropertyValue` and `PositionPropertyValue` equality (`ABS(a - b) < 0.0002`).
pub const COMPARE_EPSILON: f32 = 0.0002;

/// `RGBAColor`: four floats.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbaColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaColor {
    /// The colour a fresh `ColorPropertyValue` starts at (`RGBAColor_White`). The startup
    /// assignment of that global is not recovered yet, so opaque white is assumed.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

/// `PositionPropertyValue::m_pos`: a `Position`, its cell id and its frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyPosition {
    pub cell: u32,
    pub frame: AFrame,
}

impl Default for PropertyPosition {
    fn default() -> Self {
        Self {
            cell: 0,
            frame: AFrame::IDENTITY,
        }
    }
}

/// One property's value, by its `MasterProperty` type.
///
/// `Integer` is 32-bit (`long` in the 32-bit client). `DataFile` holds the DID as a raw `ulong`,
/// `Enum`/`PackageId` the raw enum word. `String` keeps the stored bytes (`PStringBase<char>`).
#[derive(Debug, Clone, PartialEq)]
pub enum BasePropertyValue {
    Bool(bool),
    Integer(i32),
    LongInteger(i64),
    Float(f32),
    Vector(Vec3),
    Color(RgbaColor),
    String(Vec<u8>),
    Enum(u32),
    DataFile(u32),
    Waveform(Waveform),
    PackageId(u32),
    Position(PropertyPosition),
}

/// The waveform a fresh `WaveformPropertyValue` starts at (`Waveform::Waveform`).
fn default_waveform() -> Waveform {
    Waveform {
        kind: kind::NONE,
        base: 0.0,
        base_velocity: 0.0,
        amplitude: 1.0,
        amplitude_velocity: 0.0,
        phase: 0.0,
        phase_velocity: 0.0,
        frequency: 1.0,
        frequency_velocity: 0.0,
        roughness: 0.0,
        roughness_velocity: 0.0,
    }
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < COMPARE_EPSILON
}

fn near_vec(a: Vec3, b: Vec3) -> bool {
    near(a.x, b.x) && near(a.y, b.y) && near(a.z, b.z)
}

fn near_quat(a: Quat, b: Quat) -> bool {
    near(a.w, b.w) && near(a.x, b.x) && near(a.y, b.y) && near(a.z, b.z)
}

/// `AFrame::operator==`: origin and orientation within the tolerance.
fn near_frame(a: &AFrame, b: &AFrame) -> bool {
    near_vec(a.origin, b.origin) && near_quat(a.orientation, b.orientation)
}

fn near_waveform(a: &Waveform, b: &Waveform) -> bool {
    a.kind == b.kind
        && near(a.base, b.base)
        && near(a.base_velocity, b.base_velocity)
        && near(a.amplitude, b.amplitude)
        && near(a.amplitude_velocity, b.amplitude_velocity)
        && near(a.phase, b.phase)
        && near(a.phase_velocity, b.phase_velocity)
        && near(a.frequency, b.frequency)
        && near(a.frequency_velocity, b.frequency_velocity)
        && near(a.roughness, b.roughness)
        && near(a.roughness_velocity, b.roughness_velocity)
}

impl BasePropertyValue {
    /// `MasterProperty::AllocatePropertyValue`: a fresh value of `value_type`, or `None` for a
    /// type this client has no value class for.
    pub fn new_for_type(value_type: u32) -> Option<Self> {
        Some(match value_type {
            TYPE_BOOL => Self::Bool(false),
            TYPE_INTEGER => Self::Integer(0),
            TYPE_FLOAT => Self::Float(0.0),
            TYPE_VECTOR => Self::Vector(Vec3::ZERO),
            TYPE_COLOR => Self::Color(RgbaColor::WHITE),
            TYPE_STRING => Self::String(Vec::new()),
            TYPE_ENUM => Self::Enum(0),
            TYPE_DATA_FILE => Self::DataFile(0),
            TYPE_WAVEFORM => Self::Waveform(default_waveform()),
            TYPE_PACKAGE_ID => Self::PackageId(0),
            TYPE_LONG_INTEGER => Self::LongInteger(0),
            TYPE_POSITION => Self::Position(PropertyPosition::default()),
            _ => return None,
        })
    }

    /// The `MasterProperty` value type this value is.
    pub fn value_type(&self) -> u32 {
        match self {
            Self::Bool(_) => TYPE_BOOL,
            Self::Integer(_) => TYPE_INTEGER,
            Self::LongInteger(_) => TYPE_LONG_INTEGER,
            Self::Float(_) => TYPE_FLOAT,
            Self::Vector(_) => TYPE_VECTOR,
            Self::Color(_) => TYPE_COLOR,
            Self::String(_) => TYPE_STRING,
            Self::Enum(_) => TYPE_ENUM,
            Self::DataFile(_) => TYPE_DATA_FILE,
            Self::Waveform(_) => TYPE_WAVEFORM,
            Self::PackageId(_) => TYPE_PACKAGE_ID,
            Self::Position(_) => TYPE_POSITION,
        }
    }

    /// `BoolPropertyValue::InqBool`.
    pub fn inq_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// `IntegerPropertyValue` getter.
    pub fn inq_integer(&self) -> Option<i32> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// `LongIntegerPropertyValue::InqLongInteger`.
    pub fn inq_long_integer(&self) -> Option<i64> {
        match self {
            Self::LongInteger(v) => Some(*v),
            _ => None,
        }
    }

    /// `FloatPropertyValue` getter.
    pub fn inq_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    /// `VectorPropertyValue::InqVector`.
    pub fn inq_vector(&self) -> Option<Vec3> {
        match self {
            Self::Vector(v) => Some(*v),
            _ => None,
        }
    }

    /// `ColorPropertyValue::InqColor`.
    pub fn inq_color(&self) -> Option<RgbaColor> {
        match self {
            Self::Color(v) => Some(*v),
            _ => None,
        }
    }

    /// `StringPropertyValue::InqString`: the stored bytes.
    pub fn inq_string(&self) -> Option<&[u8]> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }

    /// `EnumPropertyValue` getter.
    pub fn inq_enum(&self) -> Option<u32> {
        match self {
            Self::Enum(v) => Some(*v),
            _ => None,
        }
    }

    /// `DataFilePropertyValue` getter: the DID word.
    pub fn inq_data_file(&self) -> Option<u32> {
        match self {
            Self::DataFile(v) => Some(*v),
            _ => None,
        }
    }

    /// `WaveformPropertyValue::InqWaveform`.
    pub fn inq_waveform(&self) -> Option<Waveform> {
        match self {
            Self::Waveform(v) => Some(*v),
            _ => None,
        }
    }

    /// `PackageIDPropertyValue` getter.
    pub fn inq_package_id(&self) -> Option<u32> {
        match self {
            Self::PackageId(v) => Some(*v),
            _ => None,
        }
    }

    /// `PositionPropertyValue::InqPosition`.
    pub fn inq_position(&self) -> Option<PropertyPosition> {
        match self {
            Self::Position(v) => Some(*v),
            _ => None,
        }
    }

    /// `BoolPropertyValue` setter: writes only into a bool value, and reports whether it did.
    pub fn set_bool(&mut self, value: bool) -> bool {
        self.set_with(Self::Bool(value))
    }

    /// `IntegerPropertyValue` setter.
    pub fn set_integer(&mut self, value: i32) -> bool {
        self.set_with(Self::Integer(value))
    }

    /// `LongIntegerPropertyValue::SetLongInteger`.
    pub fn set_long_integer(&mut self, value: i64) -> bool {
        self.set_with(Self::LongInteger(value))
    }

    /// `FloatPropertyValue` setter.
    pub fn set_float(&mut self, value: f32) -> bool {
        self.set_with(Self::Float(value))
    }

    /// `VectorPropertyValue::SetVector`.
    pub fn set_vector(&mut self, value: Vec3) -> bool {
        self.set_with(Self::Vector(value))
    }

    /// `ColorPropertyValue::SetColor`.
    pub fn set_color(&mut self, value: RgbaColor) -> bool {
        self.set_with(Self::Color(value))
    }

    /// `StringPropertyValue::SetString`.
    pub fn set_string(&mut self, value: &[u8]) -> bool {
        self.set_with(Self::String(value.to_vec()))
    }

    /// `EnumPropertyValue` setter.
    pub fn set_enum(&mut self, value: u32) -> bool {
        self.set_with(Self::Enum(value))
    }

    /// `DataFilePropertyValue` setter.
    pub fn set_data_file(&mut self, value: u32) -> bool {
        self.set_with(Self::DataFile(value))
    }

    /// `WaveformPropertyValue::SetWaveform`.
    pub fn set_waveform(&mut self, value: Waveform) -> bool {
        self.set_with(Self::Waveform(value))
    }

    /// `PackageIDPropertyValue` setter.
    pub fn set_package_id(&mut self, value: u32) -> bool {
        self.set_with(Self::PackageId(value))
    }

    /// `PositionPropertyValue::SetPosition`.
    pub fn set_position(&mut self, value: PropertyPosition) -> bool {
        self.set_with(Self::Position(value))
    }

    /// `XPropertyValue::Set(BasePropertyValue*)`: copies `source` in when both are the same
    /// type, and reports whether it did (the source's typed getter answers only for its type).
    pub fn set_from(&mut self, source: &Self) -> bool {
        if self.value_type() == source.value_type() {
            *self = source.clone();
            true
        } else {
            false
        }
    }

    /// `XPropertyValue::Compare(BasePropertyValue*)`: a value of another type is never equal.
    pub fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::LongInteger(a), Self::LongInteger(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Vector(a), Self::Vector(b)) => near_vec(*a, *b),
            (Self::Color(a), Self::Color(b)) => {
                near(a.r, b.r) && near(a.g, b.g) && near(a.b, b.b) && near(a.a, b.a)
            }
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Enum(a), Self::Enum(b)) => a == b,
            (Self::DataFile(a), Self::DataFile(b)) => a == b,
            (Self::Waveform(a), Self::Waveform(b)) => near_waveform(a, b),
            (Self::PackageId(a), Self::PackageId(b)) => a == b,
            (Self::Position(a), Self::Position(b)) => {
                a.cell == b.cell && near_frame(&a.frame, &b.frame)
            }
            _ => false,
        }
    }

    fn set_with(&mut self, value: Self) -> bool {
        if self.value_type() == value.value_type() {
            *self = value;
            true
        } else {
            false
        }
    }
}

/// `BaseProperty`: a property's name, its `MasterProperty` type and its value. The value is
/// `None` until a type is set and a value class exists for it (retail's `m_propertyValue`).
#[derive(Debug, Clone, PartialEq)]
pub struct BaseProperty {
    pub name: u32,
    pub value_type: u32,
    pub value: Option<BasePropertyValue>,
}

impl BaseProperty {
    /// `BaseProperty::BaseProperty`: no name, no type and no value.
    pub fn new(name: u32) -> Self {
        Self {
            name,
            value_type: 0,
            value: None,
        }
    }

    /// `BaseProperty::SetPropertyType`: keeps the value when the type is unchanged; otherwise
    /// frees it and allocates a fresh one of the new type. Reports whether a value now exists.
    pub fn set_property_type(&mut self, value_type: u32) -> bool {
        if self.value_type == value_type {
            return true;
        }
        self.value_type = value_type;
        self.value = BasePropertyValue::new_for_type(value_type);
        self.value.is_some()
    }

    /// `BaseProperty::Compare`: the same name and type, and equal values.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.name == other.name
            && self.value_type == other.value_type
            && match (&self.value, &other.value) {
                (Some(a), Some(b)) => a.is_equal(b),
                _ => false,
            }
    }
}
