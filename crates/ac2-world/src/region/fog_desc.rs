//! `CFogDesc` (`0x2x` fog descriptor record), its `FogType`s and the `LightFogInfo` each carries:
//! a fog and a light with a per-field combine rule for blending it over the day's light.

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceTypeId};

use crate::color::{set_color32, Rgba, WHITE};

/// `FogCombine::FogCombineType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FogCombineType {
    MultiplyColors,
    AddColors,
    UseFirstColor,
    UseSecondColor,
    UseMaximumColor,
    UseMinimumColor,
}

/// `FogCombine`: how a field blends and, for `AddColors`, its two multipliers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogCombine {
    pub combine_type: FogCombineType,
    pub color_mult_first: f32,
    pub color_mult_second: f32,
}

impl Default for FogCombine {
    /// `LightFogInfo::LightFogInfo`: use the first colour, multipliers 0.
    fn default() -> Self {
        Self {
            combine_type: FogCombineType::UseFirstColor,
            color_mult_first: 0.0,
            color_mult_second: 0.0,
        }
    }
}

/// `LScapeFogType`.
pub const LSCAPE_FOG_NO_FOG: u32 = 0;
pub const LSCAPE_FOG_LINEAR: u32 = 1;

/// `FogInfo`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogInfo {
    pub fog_min: f32,
    pub fog_max: f32,
    pub fog_sky: f32,
    pub fog_density: f32,
    pub fog_type: u32,
    pub fog_color: Rgba,
}

/// `LightInfo`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightColors {
    pub amb_bright: f32,
    pub amb_color: Rgba,
    pub dir_bright: f32,
    pub dir_color: Rgba,
    pub shadow_color: Rgba,
}

impl Default for LightColors {
    /// `LightInfo::Destroy`: full brightness, white.
    fn default() -> Self {
        Self {
            amb_bright: 1.0,
            amb_color: WHITE,
            dir_bright: 1.0,
            dir_color: WHITE,
            shadow_color: WHITE,
        }
    }
}

/// `LightFogInfo`: the seven combine rules (ambient brightness, ambient colour, direct brightness,
/// direct colour, day light, day fog, distance fog), the fog and the light.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightFogInfo {
    pub combine_ambient_brightness: FogCombine,
    pub combine_ambient_color: FogCombine,
    pub combine_direct_brightness: FogCombine,
    pub combine_direct_color: FogCombine,
    pub combine_day_light: FogCombine,
    pub combine_day_fog: FogCombine,
    pub combine_distance_fog: FogCombine,
    pub fog: FogInfo,
    pub light: LightColors,
}

impl Default for LightFogInfo {
    /// `LightFogInfo::LightFogInfo`: linear fog 0..10000, density 0.4, white.
    fn default() -> Self {
        Self {
            combine_ambient_brightness: FogCombine::default(),
            combine_ambient_color: FogCombine::default(),
            combine_direct_brightness: FogCombine::default(),
            combine_direct_color: FogCombine::default(),
            combine_day_light: FogCombine::default(),
            combine_day_fog: FogCombine::default(),
            combine_distance_fog: FogCombine::default(),
            fog: FogInfo {
                fog_min: 0.0,
                fog_max: 10_000.0,
                fog_sky: 0.0,
                fog_density: 0.4,
                fog_type: LSCAPE_FOG_LINEAR,
                fog_color: WHITE,
            },
            light: LightColors::default(),
        }
    }
}

/// `LightFogInfo::FindCodeType`: codes 1..=4 name a rule; 0 and anything else multiply (the
/// switch's default, so code 5 does not read back as the minimum).
pub fn find_code_type(code: u32) -> FogCombineType {
    match code {
        1 => FogCombineType::AddColors,
        2 => FogCombineType::UseFirstColor,
        3 => FogCombineType::UseSecondColor,
        4 => FogCombineType::UseMaximumColor,
        _ => FogCombineType::MultiplyColors,
    }
}

/// `LightFogInfo::GetCodePart`.
pub fn code_part(combine: &FogCombine) -> u32 {
    match combine.combine_type {
        FogCombineType::MultiplyColors => 0,
        FogCombineType::AddColors => 1,
        FogCombineType::UseFirstColor => 2,
        FogCombineType::UseSecondColor => 3,
        FogCombineType::UseMaximumColor => 4,
        FogCombineType::UseMinimumColor => 5,
    }
}

impl LightFogInfo {
    /// `LightFogInfo::SetCombine`: three bits per rule, ambient brightness in the low bits.
    pub fn set_combine(&mut self, code: u32) {
        let part = |shift: u32| find_code_type((code >> shift) & 7);
        self.combine_ambient_brightness.combine_type = part(0);
        self.combine_ambient_color.combine_type = part(3);
        self.combine_direct_brightness.combine_type = part(6);
        self.combine_direct_color.combine_type = part(9);
        self.combine_day_light.combine_type = part(12);
        self.combine_day_fog.combine_type = part(15);
        self.combine_distance_fog.combine_type = part(18);
    }

    /// `LightFogInfo::GetCode`: the inverse of [`set_combine`](Self::set_combine).
    pub fn code(&self) -> u32 {
        code_part(&self.combine_ambient_brightness)
            | code_part(&self.combine_ambient_color) << 3
            | code_part(&self.combine_direct_brightness) << 6
            | code_part(&self.combine_direct_color) << 9
            | code_part(&self.combine_day_light) << 12
            | code_part(&self.combine_day_fog) << 15
            | code_part(&self.combine_distance_fog) << 18
    }

    /// `LightFogInfo::unpack`: fog min/max/sky/density and type, the fog colour, the directional
    /// brightness and colour, the ambient brightness and colour, the combine code, then the two
    /// multipliers of each `AddColors` rule. Like retail, a field the record is too short for
    /// keeps its default.
    pub fn unpack(cursor: &mut ArchiveCursor<'_>) -> Self {
        let mut info = Self::default();
        let word = |cursor: &mut ArchiveCursor<'_>| {
            (cursor.remaining() >= 4)
                .then(|| cursor.read_u32().ok())
                .flatten()
        };
        let float = |cursor: &mut ArchiveCursor<'_>, into: &mut f32| {
            if let Some(bits) = word(cursor) {
                *into = f32::from_bits(bits);
            }
        };
        if cursor.remaining() >= 4 {
            float(cursor, &mut info.fog.fog_min);
            float(cursor, &mut info.fog.fog_max);
            float(cursor, &mut info.fog.fog_sky);
            float(cursor, &mut info.fog.fog_density);
            if let Some(kind) = word(cursor) {
                info.fog.fog_type = kind;
            }
        }
        if let Some(color) = word(cursor) {
            info.fog.fog_color = set_color32(color);
        }
        float(cursor, &mut info.light.dir_bright);
        if let Some(color) = word(cursor) {
            info.light.dir_color = set_color32(color);
        }
        float(cursor, &mut info.light.amb_bright);
        if let Some(color) = word(cursor) {
            info.light.amb_color = set_color32(color);
        }
        if let Some(code) = word(cursor) {
            info.set_combine(code);
        }
        for combine in [
            &mut info.combine_ambient_brightness,
            &mut info.combine_ambient_color,
            &mut info.combine_direct_brightness,
            &mut info.combine_direct_color,
            &mut info.combine_day_light,
            &mut info.combine_day_fog,
            &mut info.combine_distance_fog,
        ] {
            if combine.combine_type == FogCombineType::AddColors && cursor.remaining() >= 4 {
                float(cursor, &mut combine.color_mult_first);
                float(cursor, &mut combine.color_mult_second);
            }
        }
        info
    }
}

/// `FogType`: a fog and light preset keyed by `index`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogType {
    pub index: u32,
    pub fog_lighting: LightFogInfo,
}

impl FogType {
    /// `FogType::unpack`: the index (when the record has 4 bytes), then the `LightFogInfo`.
    pub fn unpack(cursor: &mut ArchiveCursor<'_>) -> Self {
        let index = if cursor.remaining() >= 4 {
            cursor.read_u32().unwrap_or(0)
        } else {
            0
        };
        Self {
            index,
            fog_lighting: LightFogInfo::unpack(cursor),
        }
    }

    /// `FogType::HasFog`.
    pub fn has_fog(&self) -> bool {
        self.fog_lighting.fog.fog_type != LSCAPE_FOG_NO_FOG
    }

    /// `FogType::GetFogging`: the `LightFogInfo` copied out.
    pub fn fogging(&self) -> LightFogInfo {
        self.fog_lighting
    }
}

/// `CFogDesc`.
#[derive(Debug, Clone, PartialEq)]
pub struct FogDesc {
    pub version: u32,
    pub distance_fog_map: DataId,
    pub volume_fog_map: DataId,
    pub bottom_fog_map: DataId,
    pub top_fog_map: DataId,
    pub default_fog: Option<FogType>,
    pub fog_types: Vec<FogType>,
}

impl FogDesc {
    /// `CFogDesc::FindFogType`: the type with that index, else the default fog.
    pub fn find_fog_type(&self, index: u32) -> Option<&FogType> {
        self.fog_types
            .iter()
            .find(|fog| fog.index == index)
            .or(self.default_fog.as_ref())
    }

    /// `CFogDesc::HasFog`.
    pub fn has_fog(&self, index: u32) -> bool {
        self.find_fog_type(index).is_some_and(FogType::has_fog)
    }

    /// `CFogDesc::GetFogging`: the `LightFogInfo` of the type, `None` where retail answers 0.
    pub fn fogging(&self, index: u32) -> Option<LightFogInfo> {
        self.find_fog_type(index).map(FogType::fogging)
    }

    /// `CFogDesc::GetSubDataIDs`: the four fog maps, in order (the `FogType`s add none).
    pub fn sub_data_ids(&self) -> [DataId; 4] {
        [
            self.distance_fog_map,
            self.volume_fog_map,
            self.bottom_fog_map,
            self.top_fog_map,
        ]
    }
}

impl ResourcePackedDecode for FogDesc {
    const TYPE_NAME: &'static str = "AC2 FogDesc";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `CFogDesc::UnPack` (the `CFogDesc` stream): version, DID, four map DIDs, the has-default word (4-byte aligned), the
    /// default `FogType`, a count and the `FogType`s.
    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        let version = cursor.read_u32()?;
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        let distance_fog_map = DataId::new(cursor.read_u32()?);
        let volume_fog_map = DataId::new(cursor.read_u32()?);
        let bottom_fog_map = DataId::new(cursor.read_u32()?);
        let top_fog_map = DataId::new(cursor.read_u32()?);
        let has_default = cursor.read_u32()? != 0;
        cursor.align_to(4)?;
        let default_fog = has_default.then(|| FogType::unpack(&mut cursor));
        let count = cursor.read_u32_count()?;
        if count > cursor.remaining() / 4 + 1 {
            return Err(malformed("item count exceeded the record"));
        }
        let fog_types = (0..count)
            .map(|_| FogType::unpack(&mut cursor))
            .collect::<Vec<_>>();
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after FogDesc"));
        }
        Ok(Self {
            version,
            distance_fog_map,
            volume_fog_map,
            bottom_fog_map,
            top_fog_map,
            default_fog,
            fog_types,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.fog_types.len() * std::mem::size_of::<FogType>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_codes_round_trip_except_the_minimum() {
        let mut info = LightFogInfo::default();
        info.set_combine(0b100_011_010_001_000_001_010);
        assert_eq!(
            info.combine_ambient_brightness.combine_type,
            FogCombineType::UseFirstColor
        );
        assert_eq!(
            info.combine_ambient_color.combine_type,
            FogCombineType::AddColors
        );
        assert_eq!(
            info.combine_direct_brightness.combine_type,
            FogCombineType::MultiplyColors
        );
        assert_eq!(
            info.combine_distance_fog.combine_type,
            FogCombineType::UseMaximumColor
        );
        assert_eq!(info.code(), 0b100_011_010_001_000_001_010);
        // GetCodePart answers 5 for the minimum rule, but FindCodeType has no case for it.
        assert_eq!(find_code_type(5), FogCombineType::MultiplyColors);
        let minimum = FogCombine {
            combine_type: FogCombineType::UseMinimumColor,
            ..FogCombine::default()
        };
        assert_eq!(code_part(&minimum), 5);
    }

    fn words(values: &[u32]) -> Vec<u8> {
        values.iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn light_fog_unpack_reads_multipliers_only_for_add_rules() {
        let bytes = words(&[
            0.0f32.to_bits(),
            500.0f32.to_bits(),
            0.25f32.to_bits(),
            0.5f32.to_bits(),
            LSCAPE_FOG_LINEAR,
            0xFF00_8000,
            0.75f32.to_bits(),
            0xFFFF_FFFF,
            0.5f32.to_bits(),
            0xFF00_00FF,
            // ambient colour adds, the rest use the first colour
            (2 | 1 << 3 | 2 << 6 | 2 << 9 | 2 << 12 | 2 << 15 | 2 << 18) as u32,
            2.0f32.to_bits(),
            3.0f32.to_bits(),
        ]);
        let mut cursor = ArchiveCursor::new(&bytes);
        let info = LightFogInfo::unpack(&mut cursor);
        assert_eq!(cursor.remaining(), 0);
        assert_eq!(info.fog.fog_max, 500.0);
        assert!((info.fog.fog_color[1] - 128.0 / 255.0).abs() < 1e-5);
        assert_eq!(info.light.dir_bright, 0.75);
        assert_eq!(info.light.amb_bright, 0.5);
        assert_eq!(info.combine_ambient_color.color_mult_first, 2.0);
        assert_eq!(info.combine_ambient_color.color_mult_second, 3.0);
        assert_eq!(info.combine_day_fog.color_mult_first, 0.0);
    }

    #[test]
    fn fog_desc_lookup_falls_back_to_the_default_fog() {
        let mut none = FogType::unpack(&mut ArchiveCursor::new(&[0_u8; 0]));
        none.index = 9;
        none.fog_lighting.fog.fog_type = LSCAPE_FOG_NO_FOG;
        let mut linear = none;
        linear.index = 3;
        linear.fog_lighting.fog.fog_type = LSCAPE_FOG_LINEAR;
        let desc = FogDesc {
            version: 1,
            distance_fog_map: DataId::new(0),
            volume_fog_map: DataId::new(0),
            bottom_fog_map: DataId::new(0),
            top_fog_map: DataId::new(0),
            default_fog: Some(none),
            fog_types: vec![linear],
        };
        assert!(desc.has_fog(3));
        assert!(!desc.has_fog(77));
        assert_eq!(desc.fogging(77), Some(none.fog_lighting));
        let bare = FogDesc {
            default_fog: None,
            ..desc
        };
        assert!(!bare.has_fog(77));
        assert_eq!(bare.fogging(77), None);
    }
}
