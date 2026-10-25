//! AC2 gameplay structures held as native objects (`gmCNativeFactory`): `ExaminationRequest`,
//! `GameplayOptionsProfile` (with `ShortcutInfo`), `gmRaceSexInfo`, `gmSceneInfo` (with
//! `gmKeyframe`) and `SkillUINode`.

use super::StringInfo;
use crate::unpack::{Reader, UnpackResult};

/// `ExaminationRequest::StreamPack`: type, admin flag, then the datum the type names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExaminationRequest {
    pub kind: u32,
    pub admin: u32,
    /// `_data_enum` / `_data_DataID` (32-bit kinds) or `_data_iid` (0x40000002).
    pub data: Option<u64>,
}

impl ExaminationRequest {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let kind = r.u32()?;
        let admin = r.u32()?;
        let data = match kind {
            0x4000_0001 | 0x4000_0003..=0x4000_0007 | 0x4000_000B => Some(u64::from(r.u32()?)),
            0x4000_0002 => Some(r.u64()?),
            _ => None,
        };
        Ok(Self { kind, admin, data })
    }
}

/// `ShortcutInfo::StreamPack`: type, then the datum the type names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutInfo {
    /// 0x40000001, 0x40000004: `_data_enum`.
    Enum { kind: u32, value: u32 },
    /// 0x40000002: `_data_iid`.
    Instance(u64),
    /// 0x40000003: `_data_str`.
    Text(Vec<u16>),
    /// 0x40000006: `_data_dataid`.
    DataId(u32),
    /// Any other type: nothing more is packed.
    Empty(u32),
}

impl ShortcutInfo {
    fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let kind = r.u32()?;
        Ok(match kind {
            0x4000_0001 | 0x4000_0004 => Self::Enum {
                kind,
                value: r.u32()?,
            },
            0x4000_0002 => Self::Instance(r.u64()?),
            0x4000_0003 => Self::Text(r.wpstring()?),
            0x4000_0006 => Self::DataId(r.u32()?),
            _ => Self::Empty(kind),
        })
    }
}

/// `GameplayOptionsProfile` as the Legions `GameplayOptionsProfile::StreamPack` reads it (the
/// Legions build adds the version-6 table under bit 0x2000 that admin 7148 lacks). Fields keep
/// the packed values; the version upgrades `StreamPack` applies afterwards (default filters,
/// broadcast colour and popup flag) belong to the glue that uses the profile.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameplayOptionsProfile {
    pub pack_bitfield: u64,
    /// 0x1: `m_aliasTable`.
    pub aliases: Vec<(Vec<u16>, Vec<u16>)>,
    /// 0x2: `m_shortcutArray`.
    pub shortcuts: Vec<ShortcutInfo>,
    /// 0x4: `m_whichShortcutSet`.
    pub which_shortcut_set: u32,
    /// 0x1000: `m_iShortcutHeight`.
    pub shortcut_height: u32,
    /// 0x8: `m_fDamageTextRangeOther`.
    pub damage_text_range_other: f32,
    /// 0x20: `m_radarMask`.
    pub radar_mask: u32,
    /// 0x40: `m_filterHash`.
    pub filters: Vec<(u32, u32)>,
    /// 0x80: `m_bitField`.
    pub bit_field: u32,
    /// `m_version`, packed when `m_bitField` has 0x80000000.
    pub version: Option<u32>,
    /// 0x100 (version >= 1): `m_chatFontColors`.
    pub chat_font_colors: Vec<(u32, u32)>,
    /// 0x200 (version >= 1): `m_chatFontSizes`.
    pub chat_font_sizes: Vec<(u32, u32)>,
    /// 0x2000 (version >= 6, Legions only).
    pub version6_table: Vec<(u32, u32)>,
    /// 0x400 (version >= 2): `m_chatPopupFlags`.
    pub chat_popup_flags: Vec<(u32, u32)>,
    /// 0x800 (version >= 2): `m_windowOpacities`.
    pub window_opacities: Vec<(u32, u32)>,
}

fn pairs(r: &mut Reader<'_>) -> UnpackResult<Vec<(u32, u32)>> {
    let n = r.hash_count(8, "hash count")?;
    (0..n).map(|_| Ok((r.u32()?, r.u32()?))).collect()
}

impl GameplayOptionsProfile {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let bits = r.u64()?;
        let has = |bit: u64| bits & bit != 0;
        let mut p = Self {
            pack_bitfield: bits,
            ..Self::default()
        };
        if has(0x1) {
            let n = r.hash_count(4, "alias count")?;
            p.aliases = (0..n)
                .map(|_| Ok((r.wpstring()?, r.wpstring()?)))
                .collect::<UnpackResult<_>>()?;
        }
        if has(0x2) {
            let n = r.count(4, "shortcut count")?;
            p.shortcuts = (0..n)
                .map(|_| ShortcutInfo::unpack(r))
                .collect::<UnpackResult<_>>()?;
        }
        if has(0x4) {
            p.which_shortcut_set = r.u32()?;
        }
        if has(0x1000) {
            p.shortcut_height = r.u32()?;
        }
        if has(0x8) {
            p.damage_text_range_other = r.f32()?;
        }
        if has(0x10) {
            // `UISaveLocations::StreamPack` is not ported; no Legions WState packs it.
            return Err(r.bad("GameplayOptionsProfile saved UI locations not ported"));
        }
        if has(0x20) {
            p.radar_mask = r.u32()?;
        }
        if has(0x40) {
            p.filters = pairs(r)?;
        }
        if has(0x80) {
            p.bit_field = r.u32()?;
        }
        if p.bit_field & 0x8000_0000 != 0 {
            let version = r.u32()?;
            p.version = Some(version);
            if version >= 1 && has(0x100) {
                p.chat_font_colors = pairs(r)?;
            }
            if version >= 1 && has(0x200) {
                p.chat_font_sizes = pairs(r)?;
            }
            if version >= 6 && has(0x2000) {
                p.version6_table = pairs(r)?;
            }
            if version >= 2 && has(0x400) {
                p.chat_popup_flags = pairs(r)?;
            }
            if version >= 2 && has(0x800) {
                p.window_opacities = pairs(r)?;
            }
        }
        Ok(p)
    }
}

/// `gmRaceSexInfo::StreamPack`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GmRaceSexInfo {
    pub phys_obj_did: u32,
    pub apr_file_did: u32,
    pub music_file_did: u32,
    /// `_objInLocHash`.
    pub obj_in_loc: Vec<(u32, u32)>,
}

impl GmRaceSexInfo {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        Ok(Self {
            phys_obj_did: r.u32()?,
            apr_file_did: r.u32()?,
            music_file_did: r.u32()?,
            obj_in_loc: pairs(r)?,
        })
    }
}

/// `gmKeyframe::StreamPack`.
#[derive(Debug, Clone, PartialEq)]
pub struct GmKeyframe {
    pub scene_id: u32,
    pub frame_num: u32,
    pub duration: f64,
    pub video_fragment: u32,
    pub audio_fragment: u32,
    pub voice_fragment: u32,
    pub text_fragment: StringInfo,
    pub movie_fragment_filename: Vec<u16>,
    pub defined_tracks: u32,
}

/// `gmSceneInfo::StreamPack`.
#[derive(Debug, Clone, PartialEq)]
pub struct GmSceneInfo {
    pub id: u32,
    pub name: StringInfo,
    pub act_id: u32,
    pub scene_num: u32,
    pub is_hidden: u32,
    pub is_playable: u32,
    pub keyframes: Vec<GmKeyframe>,
}

impl GmSceneInfo {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        let id = r.u32()?;
        let name = StringInfo::unpack(r)?;
        let act_id = r.u32()?;
        let scene_num = r.u32()?;
        let is_hidden = r.u32()?;
        let is_playable = r.u32()?;
        let n = r.count(4, "keyframe count")?;
        let keyframes = (0..n)
            .map(|_| {
                Ok(GmKeyframe {
                    scene_id: r.u32()?,
                    frame_num: r.u32()?,
                    duration: r.f64()?,
                    video_fragment: r.u32()?,
                    audio_fragment: r.u32()?,
                    voice_fragment: r.u32()?,
                    text_fragment: StringInfo::unpack(r)?,
                    movie_fragment_filename: r.wpstring()?,
                    defined_tracks: r.u32()?,
                })
            })
            .collect::<UnpackResult<_>>()?;
        Ok(Self {
            id,
            name,
            act_id,
            scene_num,
            is_hidden,
            is_playable,
            keyframes,
        })
    }
}

/// `SkillUINode::StreamPack`.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillUiNode {
    pub skill_enum: u32,
    pub skill_type: u32,
    pub name: StringInfo,
    pub desc: StringInfo,
    pub icon: u32,
    pub x: i32,
    pub y: i32,
    pub parents: Vec<u32>,
    pub dependencies: Vec<u32>,
    pub is_passive: u32,
    pub untrainable: u32,
}

impl SkillUiNode {
    pub(super) fn unpack(r: &mut Reader<'_>) -> UnpackResult<Self> {
        Ok(Self {
            skill_enum: r.u32()?,
            skill_type: r.u32()?,
            name: StringInfo::unpack(r)?,
            desc: StringInfo::unpack(r)?,
            icon: r.u32()?,
            x: r.i32()?,
            y: r.i32()?,
            parents: r.u32_array()?,
            dependencies: r.u32_array()?,
            is_passive: r.u32()?,
            untrainable: r.u32()?,
        })
    }
}
