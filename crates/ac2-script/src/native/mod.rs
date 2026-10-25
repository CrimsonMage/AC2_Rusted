//! Native objects: the engine types a heap object can hold instead of script slots, by native
//! type id (`CNativeFactory::GetNativeTypeName`, `gmCNativeFactory::GetNativeTypeName`).
//!
//! [`unpack`] is `CNativeFactory::UnPackNativeObject` and `gmCNativeFactory::UnPackNativeObject`
//! for every native type the Legions WStates hold; the others report "not ported" until a
//! script needs them.

mod containers;
mod gm;
mod string_info;
mod structs;

pub use containers::{RefKey, ScriptHash, ScriptList};
pub use gm::{
    ExaminationRequest, GameplayOptionsProfile, GmKeyframe, GmRaceSexInfo, GmSceneInfo,
    ShortcutInfo, SkillUiNode,
};
pub use string_info::{StringInfo, StringVar, StringVarValue};
pub use structs::{BehaviorParams, MissileParameters, Position, RandomSelectionTable};

use crate::unpack::{Reader, UnpackResult};

/// Native type ids (`CNativeFactory` 0x01-0x76 and the 0x2Exx-0x38xx extensions,
/// `gmCNativeFactory` 0x82-0xAF).
pub mod ntype {
    pub const AA_HASH: u32 = 0x01;
    pub const AA_MULTI_HASH: u32 = 0x03;
    pub const A_ARRAY: u32 = 0x05;
    pub const A_HASH_SET: u32 = 0x06;
    pub const AL_HASH: u32 = 0x08;
    pub const A_LIST: u32 = 0x0A;
    pub const AR_HASH: u32 = 0x11;
    pub const BEHAVIOR_PARAMS: u32 = 0x17;
    pub const LA_HASH: u32 = 0x32;
    pub const L_ARRAY: u32 = 0x37;
    pub const LL_HASH: u32 = 0x39;
    pub const L_LIST: u32 = 0x3B;
    pub const LR_HASH: u32 = 0x40;
    pub const MISSILE_PARAMETERS: u32 = 0x46;
    pub const NA_HASH: u32 = 0x4B;
    pub const NR_HASH: u32 = 0x4D;
    pub const POSITION: u32 = 0x5A;
    pub const RANDOM_SELECTION_TABLE_INT: u32 = 0x60;
    pub const R_ARRAY: u32 = 0x61;
    pub const R_LIST: u32 = 0x65;
    pub const STRING_INFO: u32 = 0x6A;
    pub const VECTOR: u32 = 0x70;
    pub const WPSTRING: u32 = 0x76;
    pub const LA_HASH_SET: u32 = 0x2E0B;
    pub const EXAMINATION_REQUEST: u32 = 0x9A;
    pub const GAMEPLAY_OPTIONS_PROFILE: u32 = 0x9B;
    pub const GM_RACE_SEX_INFO: u32 = 0xA2;
    pub const GM_SCENE_INFO: u32 = 0xA3;
    pub const SKILL_UI_NODE: u32 = 0xAE;
}

/// One unpacked native object.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeObject {
    /// `AList`, `RList`, `AArray`, `RArray` (32-bit elements) and `LList`, `LArray` (64-bit).
    List(ScriptList),
    /// The `XYHash` family, `AAMultiHash`, and the hash sets.
    Hash(ScriptHash),
    StringInfo(StringInfo),
    /// `wpstring` (`PStringBase<unsigned short>`).
    WString(Vec<u16>),
    /// `Vector` (x, y, z).
    Vector([f32; 3]),
    Position(Position),
    MissileParameters(MissileParameters),
    BehaviorParams(Box<BehaviorParams>),
    RandomSelectionTable(RandomSelectionTable),
    ExaminationRequest(ExaminationRequest),
    GameplayOptionsProfile(Box<GameplayOptionsProfile>),
    GmRaceSexInfo(GmRaceSexInfo),
    GmSceneInfo(GmSceneInfo),
    SkillUiNode(Box<SkillUiNode>),
}

/// `CNativeFactory::UnPackNativeObject` / `gmCNativeFactory::UnPackNativeObject`.
pub fn unpack(native_type: u32, r: &mut Reader<'_>) -> UnpackResult<NativeObject> {
    use containers::{Elem, Key};
    Ok(match native_type {
        ntype::A_LIST | ntype::R_LIST | ntype::A_ARRAY | ntype::R_ARRAY => {
            NativeObject::List(ScriptList::unpack(r, Elem::U32)?)
        }
        ntype::L_LIST | ntype::L_ARRAY => NativeObject::List(ScriptList::unpack(r, Elem::U64)?),
        ntype::AA_HASH | ntype::AL_HASH | ntype::AR_HASH => {
            let value = value_width(native_type);
            NativeObject::Hash(ScriptHash::unpack(r, Key::U32, Some(value))?)
        }
        ntype::LA_HASH | ntype::LL_HASH | ntype::LR_HASH => {
            let value = value_width(native_type);
            NativeObject::Hash(ScriptHash::unpack(r, Key::U64, Some(value))?)
        }
        ntype::NA_HASH | ntype::NR_HASH => {
            NativeObject::Hash(ScriptHash::unpack(r, Key::Ref, Some(Elem::U32))?)
        }
        ntype::AA_MULTI_HASH => NativeObject::Hash(ScriptHash::unpack_multi(r)?),
        ntype::A_HASH_SET => NativeObject::Hash(ScriptHash::unpack(r, Key::U32, None)?),
        ntype::LA_HASH_SET => NativeObject::Hash(ScriptHash::unpack(r, Key::U64, None)?),
        ntype::STRING_INFO => NativeObject::StringInfo(StringInfo::unpack(r)?),
        ntype::WPSTRING => NativeObject::WString(r.wpstring()?),
        ntype::VECTOR => NativeObject::Vector([r.f32()?, r.f32()?, r.f32()?]),
        ntype::POSITION => NativeObject::Position(Position::unpack(r)?),
        ntype::MISSILE_PARAMETERS => NativeObject::MissileParameters(MissileParameters::unpack(r)?),
        ntype::BEHAVIOR_PARAMS => {
            NativeObject::BehaviorParams(Box::new(BehaviorParams::unpack(r)?))
        }
        ntype::RANDOM_SELECTION_TABLE_INT => {
            NativeObject::RandomSelectionTable(RandomSelectionTable::unpack(r)?)
        }
        ntype::EXAMINATION_REQUEST => {
            NativeObject::ExaminationRequest(ExaminationRequest::unpack(r)?)
        }
        ntype::GAMEPLAY_OPTIONS_PROFILE => {
            NativeObject::GameplayOptionsProfile(Box::new(GameplayOptionsProfile::unpack(r)?))
        }
        ntype::GM_RACE_SEX_INFO => NativeObject::GmRaceSexInfo(GmRaceSexInfo::unpack(r)?),
        ntype::GM_SCENE_INFO => NativeObject::GmSceneInfo(GmSceneInfo::unpack(r)?),
        ntype::SKILL_UI_NODE => NativeObject::SkillUiNode(Box::new(SkillUiNode::unpack(r)?)),
        _ => return Err(r.bad("native type not ported")),
    })
}

/// The value width of an `XYHash`: `L` values are 64-bit, `A` and `R` values 32-bit.
fn value_width(native_type: u32) -> containers::Elem {
    match native_type {
        ntype::AL_HASH | ntype::LL_HASH => containers::Elem::U64,
        _ => containers::Elem::U32,
    }
}
