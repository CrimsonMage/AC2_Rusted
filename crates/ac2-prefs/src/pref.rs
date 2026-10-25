//! One preference: its section and key IDs, its current, applied (`previous`) and default values,
//! and the constraints of its type. The four types are `PreferenceBool`, `PreferenceFloat`,
//! `PreferenceEnum` and `PreferenceString` (`portal/shared/src/prefs`); the bank holds them.
//!
//! Retail stores text in the file and converts both ways: a bool is `0` or `1`, a float is `%f`
//! (six decimals) and parses with `strtod`, an enum is a name from its `EnumMapper` or `%d`, and a
//! string is itself. A parse that fails keeps the value (`SetValue` returns 0). No setter clamps:
//! a float's range is what its slider shows, and `SetRange` refuses an inverted one.

use crate::cnum;
use crate::names::NameTable;

/// The value a preference holds.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Float(f32),
    /// An enum's choice value (`PreferenceEnum::m_current`, an `i32`).
    Int(i32),
    Text(String),
}

/// The four preference types (`PreferenceBool`, `PreferenceFloat`, `PreferenceEnum`,
/// `PreferenceString`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bool,
    Float,
    Enum,
    Text,
}

/// One choice of an enum: its value and the label id its name comes from (`AssignChoice`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    pub value: i32,
    pub label: u32,
}

/// A preference: its section and key, its values, and its constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct Pref {
    /// `mSection`: the section's ID (a name from the enum tables, see [`NameTable`]).
    pub section: u32,
    /// `mKey`: the key's ID.
    pub key: u32,
    /// `mCurrent`: the value the page and the subsystems read.
    pub value: Value,
    /// `mPrevious`: the value last applied, which `changed` compares with.
    pub previous: Value,
    /// `mDefault`.
    pub default: Value,
    /// `PreferenceFloat::mMin` and `mMax`.
    pub min: f32,
    pub max: f32,
    /// `PreferenceEnum::mStrings` and `mLabels`: the choices, in the order they were assigned.
    pub choices: Vec<Choice>,
    /// `PreferenceEnum::mMap`: the `EnumMapper` the labels' names come from, when there is one.
    pub map: Option<u32>,
}

impl Pref {
    /// A `PreferenceBool` with its default.
    pub fn boolean(section: u32, key: u32, default: bool) -> Self {
        Self::base(section, key, Value::Bool(default))
    }

    /// A `PreferenceFloat`: its default and its range (`SetRange` then `SetDefault`). An inverted
    /// range is refused, as `SetRange` refuses it.
    pub fn float(section: u32, key: u32, default: f32, min: f32, max: f32) -> Option<Self> {
        if max < min {
            return None;
        }
        let mut pref = Self::base(section, key, Value::Float(default));
        pref.min = min;
        pref.max = max;
        Some(pref)
    }

    /// A `PreferenceString` with its default.
    pub fn text(section: u32, key: u32, default: &str) -> Self {
        Self::base(section, key, Value::Text(default.to_owned()))
    }

    /// A `PreferenceEnum`: its choices, the mapper of their names, and its default (which must
    /// be one of the choices, as `SetDefault` requires).
    pub fn enumeration(
        section: u32,
        key: u32,
        map: Option<u32>,
        choices: Vec<Choice>,
        default: i32,
    ) -> Option<Self> {
        if !choices.iter().any(|c| c.value == default) {
            return None;
        }
        let mut pref = Self::base(section, key, Value::Int(default));
        pref.map = map;
        pref.choices = choices;
        Some(pref)
    }

    fn base(section: u32, key: u32, value: Value) -> Self {
        Self {
            section,
            key,
            previous: value.clone(),
            default: value.clone(),
            value,
            min: 0.0,
            max: 1.0,
            choices: Vec::new(),
            map: None,
        }
    }

    pub fn kind(&self) -> Kind {
        match self.value {
            Value::Bool(_) => Kind::Bool,
            Value::Float(_) => Kind::Float,
            Value::Int(_) => Kind::Enum,
            Value::Text(_) => Kind::Text,
        }
    }

    /// `SetValue(PString)`: the value the text names. `false` (and no change) when it names none.
    pub fn set_text(&mut self, text: &str, names: &dyn NameTable) -> bool {
        match self.parse(text, names) {
            Some(value) => {
                self.value = value;
                true
            }
            None => false,
        }
    }

    /// `SetDefault(PString)`: the default, and the value with it. A bool whose text names none
    /// still resets the value to the default (`PreferenceBool::SetDefault`).
    pub fn set_default_text(&mut self, text: &str, names: &dyn NameTable) -> bool {
        let parsed = self.parse(text, names);
        if let Some(value) = &parsed {
            self.default = value.clone();
        }
        // A bool resets even when its text names nothing; the others only when it parsed.
        if self.kind() == Kind::Bool || parsed.is_some() {
            self.value = self.default.clone();
        }
        parsed.is_some()
    }

    /// The text the value is written as (`InqValue`).
    pub fn value_text(&self, names: &dyn NameTable) -> String {
        format_value(&self.value, self, names)
    }

    /// The text the default is written as (`InqDefault`).
    pub fn default_text(&self, names: &dyn NameTable) -> String {
        format_value(&self.default, self, names)
    }

    /// `SetValue(float)` (no clamp), `SetValue(bool)`.
    pub fn set_float(&mut self, value: f32) {
        self.value = Value::Float(value);
    }

    pub fn set_bool(&mut self, value: bool) {
        self.value = Value::Bool(value);
    }

    /// `PreferenceEnum::SetValue(long)`: the choice `value`, which must be one of the choices.
    pub fn set_choice(&mut self, value: i32) -> bool {
        if self.choices.iter().any(|c| c.value == value) {
            self.value = Value::Int(value);
            true
        } else {
            false
        }
    }

    /// `PreferenceFloat::SetDefault(float)` and the bool and enum defaults: the default and the
    /// value with it.
    pub fn set_default(&mut self, value: Value) -> bool {
        match (&value, self.kind()) {
            (Value::Int(v), Kind::Enum) if !self.choices.iter().any(|c| c.value == *v) => {
                return false;
            }
            _ => {}
        }
        self.default = value.clone();
        self.value = value;
        true
    }

    /// `PreferenceFloat::SetRange`: refused (no change) when the maximum is below the minimum.
    pub fn set_range(&mut self, min: f32, max: f32) -> bool {
        if max < min {
            return false;
        }
        self.min = min;
        self.max = max;
        true
    }

    /// `Changed`: the value differs from the one last applied.
    pub fn changed(&self) -> bool {
        self.value != self.previous
    }

    /// `Apply`: the value becomes the applied one.
    pub fn apply(&mut self) {
        self.previous = self.value.clone();
    }

    /// `Recall`: the applied value comes back.
    pub fn recall(&mut self) {
        self.value = self.previous.clone();
    }

    /// `Reset`: the default comes back (the applied value is kept).
    pub fn reset(&mut self) {
        self.value = self.default.clone();
    }

    /// `Copy`: the values, the range and the choices of `source` (the registration of a
    /// preference that already exists).
    pub fn copy_from(&mut self, source: &Pref) {
        self.value = source.value.clone();
        self.default = source.default.clone();
        self.min = source.min;
        self.max = source.max;
        self.choices = source.choices.clone();
        self.map = source.map;
    }

    /// The text a value of this preference parses from (`SetValue(PString)` on its type).
    fn parse(&self, text: &str, names: &dyn NameTable) -> Option<Value> {
        match self.kind() {
            Kind::Bool => {
                if text.eq_ignore_ascii_case("0") {
                    Some(Value::Bool(false))
                } else if text.eq_ignore_ascii_case("1") {
                    Some(Value::Bool(true))
                } else {
                    None
                }
            }
            Kind::Float => cnum::is_double(text).then(|| {
                let parsed = cnum::strtod(text.as_bytes()).value;
                Value::Float(parsed as f32)
            }),
            Kind::Enum => {
                let value = if cnum::is_int32(text) {
                    cnum::strtol(text.as_bytes()).value
                } else {
                    let map = self.map?;
                    let label = names.enum_label(map, text)?;
                    // `FindValue`: the choice of that label, or the default when none has it.
                    self.choices
                        .iter()
                        .find(|c| c.label == label)
                        .map_or(self.default_int(), |c| c.value)
                };
                self.choices
                    .iter()
                    .any(|c| c.value == value)
                    .then_some(Value::Int(value))
            }
            Kind::Text => Some(Value::Text(text.to_owned())),
        }
    }

    fn default_int(&self) -> i32 {
        match self.default {
            Value::Int(v) => v,
            _ => 0,
        }
    }

    /// The choice's label id for `value`, when it has one.
    pub fn label_of(&self, value: i32) -> Option<u32> {
        self.choices
            .iter()
            .find(|c| c.value == value)
            .map(|c| c.label)
    }
}

/// The text of a value: a bool as `1` or `0`, a float as `%f`, an enum as its name (when its
/// label has one) or `%d`, and a string as it is.
fn format_value(value: &Value, pref: &Pref, names: &dyn NameTable) -> String {
    match value {
        Value::Bool(b) => if *b { "1" } else { "0" }.to_owned(),
        Value::Float(f) => format_fixed(f64::from(*f)),
        Value::Int(v) => {
            if let (Some(map), Some(label)) = (pref.map, pref.label_of(*v)) {
                if let Some(name) = names.enum_name(map, label) {
                    return name;
                }
            }
            v.to_string()
        }
        Value::Text(t) => t.clone(),
    }
}

/// C's `%f`: six decimals, with the C spelling of the special values.
pub fn format_fixed(value: f64) -> String {
    if value.is_nan() {
        "nan".to_owned()
    } else if value.is_infinite() {
        if value < 0.0 { "-inf" } else { "inf" }.to_owned()
    } else {
        format!("{value:.6}")
    }
}
