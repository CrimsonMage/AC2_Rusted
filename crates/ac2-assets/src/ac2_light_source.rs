//! The light of an FX node (`LightSourceFX`): its `EntityDesc`'s properties, read as
//! `LightSource::ApplyPropertyCollection` reads them. The decode runs on the loader thread with
//! the FX table, so the frame thread gets the finished light.

use ac2_dat::ArchiveCursor;
use ac2_db::read_ac2_property_collection;

/// `LightSource`'s property ids (group 0x4000000A's light entries, as `ui_scene` reads them).
const LIGHT_TYPE: u32 = 0x4000_0002;
const LIGHT_DIRECTION: u32 = 0x4000_0003;
const LIGHT_RANGE: u32 = 0x4000_0004;
const LIGHT_DIFFUSE: u32 = 0x4000_0008;
const LIGHT_INTENSITY: u32 = 0x4000_0009;
const LIGHT_SPECULAR: u32 = 0x4000_000A;
/// `LightSource::LightSource`'s `m_Type` (point).
pub const LIGHT_POINT: u32 = 0x4000_0001;

/// A `LightSource` as an FX light: its type, range and diffuse intensity waveforms' base values,
/// and its colours (RGB in 0..1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2LightSource {
    /// `m_Type`: `LIGHT_POINT` for the lights an FX makes.
    pub kind: u32,
    pub direction: [f32; 3],
    /// `m_wRange` base value (`LightSource::LightSource`: 5).
    pub range: f32,
    /// `m_wDiffuseIntensity` base value (0.5 by default).
    pub intensity: f32,
    /// `m_cDiffuse` and `m_cSpecular` (white by default).
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
}

impl Default for Ac2LightSource {
    fn default() -> Self {
        Self {
            kind: LIGHT_POINT,
            direction: [0.0; 3],
            range: 5.0,
            intensity: 0.5,
            diffuse: [1.0; 3],
            specular: [1.0; 3],
        }
    }
}

/// Reads a light's packed `PropertyCollection` over the defaults of `LightSource::LightSource`.
/// Properties it does not know, and a malformed collection, leave the defaults in place.
pub fn read_ac2_light_source(properties: &[u8]) -> Ac2LightSource {
    let mut light = Ac2LightSource::default();
    let Ok(properties) = read_ac2_property_collection(&mut ArchiveCursor::new(properties)) else {
        return light;
    };
    for property in &properties {
        match property.name {
            LIGHT_TYPE => light.kind = property.word().unwrap_or(light.kind),
            LIGHT_DIRECTION => {
                if let Some(v) = property.vector() {
                    light.direction = v;
                }
            }
            // A `Waveform`: its kind, then the base value.
            LIGHT_RANGE => light.range = property.float_at(1).unwrap_or(light.range),
            LIGHT_INTENSITY => light.intensity = property.float_at(1).unwrap_or(light.intensity),
            LIGHT_DIFFUSE => light.diffuse = property.word().map_or(light.diffuse, rgb),
            LIGHT_SPECULAR => light.specular = property.word().map_or(light.specular, rgb),
            _ => {}
        }
    }
    light
}

/// An ARGB colour word as RGB in 0..1 (`RGBAColor::SetColor32`).
fn rgb(argb: u32) -> [f32; 3] {
    let channel = |shift: u32| ((argb >> shift) & 0xFF) as f32 / 255.0;
    [channel(16), channel(8), channel(0)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_or_malformed_collection_keeps_the_light_defaults() {
        assert_eq!(read_ac2_light_source(&[]), Ac2LightSource::default());
        assert_eq!(read_ac2_light_source(&[0xFF; 3]), Ac2LightSource::default());
    }

    #[test]
    fn colour_words_are_argb() {
        assert_eq!(rgb(0xFF_80_00_FF), [128.0 / 255.0, 0.0, 1.0]);
    }
}
