//! Native objects a script heap object carries across the wire and in `WState`s, by native
//! type id (`CNativeFactory::GetNativeTypeName` 0x01-0x76 and 0x2Exx, `gmCNativeFactory::
//! GetNativeTypeName` 0x82-0xAF): `VM::PackNativeObject` / `VM::UnPackNativeObject` call
//! `CNativeFactory::PackNativeObject` / `gmCNativeFactory::PackNativeObject`, which pack the
//! type's own `StreamPack` with no size or type prefix (the heap object holds the type).
//!
//! Every native type the Legions `WState`s hold is here, with those the gameplay messages
//! carry; the adaptors (`AllegianceDataAdaptor`, `AllegianceProfileAdaptor`, `FellowAdaptor`,
//! `FellowshipAdaptor`) pack nothing. Other types (the iterators, `Frame`, `Plane`, ...) are
//! refused as unsupported.

mod containers;
mod examination;
mod gm;
mod options;
mod quest;
mod structs;

pub use containers::{HashKey, ScriptHash, ScriptList};
pub use examination::{
    ExaminationDataNode, ExaminationProfile, ExaminationRequest, ExaminationValue, SelectionInfo,
};
pub use gm::{EffectUiNode, GmKeyframe, GmRaceSexInfo, GmSceneInfo, SkillUiNode};
pub use options::{GameplayOptionsProfile, ShortcutInfo, UiLocation, UiSaveLocations, WordTable};
pub use quest::GmQuestInfo;
pub use structs::{MissileParameters, RandomSelectionTable, SelectionElement};

use crate::protocol::physics::BehaviorParams;
use crate::protocol::reader::{Reader, Writer};
use crate::protocol::string_info::StringInfo;
use crate::protocol::types::Position;
use crate::protocol::ProtocolError;

/// Native type ids.
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
    pub const UI_SAVE_LOCATIONS: u32 = 0x6F;
    pub const VECTOR: u32 = 0x70;
    /// `wpstring` (`PStringBase<unsigned short>`).
    pub const WPSTRING: u32 = 0x76;
    pub const LA_HASH_SET: u32 = 0x2E0B;
    pub const ALLEGIANCE_DATA_ADAPTOR: u32 = 0x82;
    pub const ALLEGIANCE_PROFILE_ADAPTOR: u32 = 0x83;
    pub const FELLOW_ADAPTOR: u32 = 0x93;
    pub const FELLOWSHIP_ADAPTOR: u32 = 0x94;
    pub const EXAMINATION_PROFILE: u32 = 0x99;
    pub const EXAMINATION_REQUEST: u32 = 0x9A;
    pub const GAMEPLAY_OPTIONS_PROFILE: u32 = 0x9B;
    pub const GM_QUEST_INFO: u32 = 0xA0;
    pub const GM_QUEST_INFO_LIST: u32 = 0xA1;
    pub const GM_RACE_SEX_INFO: u32 = 0xA2;
    pub const GM_SCENE_INFO: u32 = 0xA3;
    /// `PList<gmSceneInfo>`: the scenes a play sequence or a portal scene carries.
    pub const GM_SCENE_INFO_LIST: u32 = 0xA4;
    pub const SELECTION_INFO: u32 = 0xAB;
    pub const SHORTCUT_INFO: u32 = 0xAC;
    pub const SKILL_UI_NODE: u32 = 0xAE;
    pub const EFFECT_UI_NODE: u32 = 0x98;
}

/// One native object.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeObject {
    /// `AList`, `RList`, `AArray`, `RArray`, `LList`, `LArray`.
    List(ScriptList),
    /// The `XYHash` family, `AAMultiHash`, `AHashSet`, `LAHashSet`.
    Hash(ScriptHash),
    StringInfo(StringInfo),
    /// `wpstring`: `PStringBase<unsigned short>::Pack`.
    WString(String),
    /// `Vector::StreamPack`: x, y, z.
    Vector([f32; 3]),
    /// `Position::Pack`.
    Position(Position),
    MissileParameters(MissileParameters),
    BehaviorParams(Box<BehaviorParams>),
    RandomSelectionTable(RandomSelectionTable),
    UiSaveLocations(UiSaveLocations),
    SelectionInfo(SelectionInfo),
    ExaminationRequest(ExaminationRequest),
    ExaminationProfile(ExaminationProfile),
    GameplayOptionsProfile(Box<GameplayOptionsProfile>),
    GmRaceSexInfo(GmRaceSexInfo),
    GmSceneInfo(Box<GmSceneInfo>),
    /// `PList<gmSceneInfo>` (native 0xA4): the scenes in list order.
    GmSceneInfoList(Vec<GmSceneInfo>),
    ShortcutInfo(ShortcutInfo),
    SkillUiNode(Box<SkillUiNode>),
    EffectUiNode(Box<EffectUiNode>),
    /// `gmQuestInfo`.
    GmQuestInfo(Box<GmQuestInfo>),
    /// `gmQuestInfoList` (`PList<gmQuestInfo>`), in list order.
    GmQuestInfoList(Vec<GmQuestInfo>),
    /// A type that packs no data (the allegiance and fellowship adaptors): its type id.
    Empty(u32),
}

impl NativeObject {
    /// The native type id the heap object names.
    pub fn native_type(&self) -> u32 {
        match self {
            Self::List(list) => list.native_type,
            Self::Hash(hash) => hash.native_type,
            Self::StringInfo(_) => ntype::STRING_INFO,
            Self::WString(_) => ntype::WPSTRING,
            Self::Vector(_) => ntype::VECTOR,
            Self::Position(_) => ntype::POSITION,
            Self::MissileParameters(_) => ntype::MISSILE_PARAMETERS,
            Self::BehaviorParams(_) => ntype::BEHAVIOR_PARAMS,
            Self::RandomSelectionTable(_) => ntype::RANDOM_SELECTION_TABLE_INT,
            Self::UiSaveLocations(_) => ntype::UI_SAVE_LOCATIONS,
            Self::SelectionInfo(_) => ntype::SELECTION_INFO,
            Self::ExaminationRequest(_) => ntype::EXAMINATION_REQUEST,
            Self::ExaminationProfile(_) => ntype::EXAMINATION_PROFILE,
            Self::GameplayOptionsProfile(_) => ntype::GAMEPLAY_OPTIONS_PROFILE,
            Self::GmRaceSexInfo(_) => ntype::GM_RACE_SEX_INFO,
            Self::GmSceneInfo(_) => ntype::GM_SCENE_INFO,
            Self::GmSceneInfoList(_) => ntype::GM_SCENE_INFO_LIST,
            Self::ShortcutInfo(_) => ntype::SHORTCUT_INFO,
            Self::SkillUiNode(_) => ntype::SKILL_UI_NODE,
            Self::EffectUiNode(_) => ntype::EFFECT_UI_NODE,
            Self::GmQuestInfo(_) => ntype::GM_QUEST_INFO,
            Self::GmQuestInfoList(_) => ntype::GM_QUEST_INFO_LIST,
            Self::Empty(native_type) => *native_type,
        }
    }

    /// `VM::UnPackNativeObject`.
    pub(crate) fn read(native_type: u32, r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        if let Some(container) = containers::read(native_type, r) {
            return container;
        }
        Ok(match native_type {
            ntype::STRING_INFO => Self::StringInfo(StringInfo::read(r)?),
            ntype::WPSTRING => Self::WString(r.wide_string()?),
            ntype::VECTOR => Self::Vector(r.vec3()?),
            ntype::POSITION => Self::Position(Position::read(r)?),
            ntype::MISSILE_PARAMETERS => Self::MissileParameters(MissileParameters::read(r)?),
            ntype::BEHAVIOR_PARAMS => Self::BehaviorParams(Box::new(BehaviorParams::read(r)?)),
            ntype::RANDOM_SELECTION_TABLE_INT => {
                Self::RandomSelectionTable(RandomSelectionTable::read(r)?)
            }
            ntype::UI_SAVE_LOCATIONS => Self::UiSaveLocations(UiSaveLocations::read(r)?),
            ntype::SELECTION_INFO => Self::SelectionInfo(SelectionInfo::read(r)?),
            ntype::EXAMINATION_REQUEST => Self::ExaminationRequest(ExaminationRequest::read(r)?),
            ntype::EXAMINATION_PROFILE => Self::ExaminationProfile(ExaminationProfile::read(r)?),
            ntype::GAMEPLAY_OPTIONS_PROFILE => {
                Self::GameplayOptionsProfile(Box::new(GameplayOptionsProfile::read(r)?))
            }
            ntype::GM_RACE_SEX_INFO => Self::GmRaceSexInfo(GmRaceSexInfo::read(r)?),
            ntype::GM_SCENE_INFO => Self::GmSceneInfo(Box::new(GmSceneInfo::read(r)?)),
            ntype::GM_SCENE_INFO_LIST => Self::GmSceneInfoList(GmSceneInfo::read_list(r)?),
            ntype::SHORTCUT_INFO => Self::ShortcutInfo(ShortcutInfo::read(r)?),
            ntype::SKILL_UI_NODE => Self::SkillUiNode(Box::new(SkillUiNode::read(r)?)),
            ntype::EFFECT_UI_NODE => Self::EffectUiNode(Box::new(EffectUiNode::read(r)?)),
            ntype::GM_QUEST_INFO => Self::GmQuestInfo(Box::new(GmQuestInfo::read(r)?)),
            ntype::GM_QUEST_INFO_LIST => Self::GmQuestInfoList(GmQuestInfo::read_list(r)?),
            ntype::ALLEGIANCE_DATA_ADAPTOR
            | ntype::ALLEGIANCE_PROFILE_ADAPTOR
            | ntype::FELLOW_ADAPTOR
            | ntype::FELLOWSHIP_ADAPTOR => Self::Empty(native_type),
            _ => return Err(ProtocolError::Unsupported("native type")),
        })
    }

    /// `VM::PackNativeObject`.
    pub(crate) fn write(&self, w: &mut Writer) {
        match self {
            Self::List(list) => list.write(w),
            Self::Hash(hash) => hash.write(w),
            Self::StringInfo(info) => info.write(w),
            Self::WString(text) => {
                w.wide_string(text);
            }
            Self::Vector(v) => {
                w.f32(v[0]).f32(v[1]).f32(v[2]);
            }
            Self::Position(position) => position.write(w),
            Self::MissileParameters(params) => params.write(w),
            Self::BehaviorParams(params) => params.write(w),
            Self::RandomSelectionTable(table) => table.write(w),
            Self::UiSaveLocations(locations) => locations.write(w),
            Self::SelectionInfo(info) => info.write(w),
            Self::ExaminationRequest(request) => request.write(w),
            Self::ExaminationProfile(profile) => profile.write(w),
            Self::GameplayOptionsProfile(profile) => profile.write(w),
            Self::GmRaceSexInfo(info) => info.write(w),
            Self::GmSceneInfo(info) => info.write(w),
            Self::GmSceneInfoList(list) => GmSceneInfo::write_list(list, w),
            Self::ShortcutInfo(shortcut) => shortcut.write(w),
            Self::SkillUiNode(node) => node.write(w),
            Self::EffectUiNode(node) => node.write(w),
            Self::GmQuestInfo(quest) => quest.write(w),
            Self::GmQuestInfoList(list) => GmQuestInfo::write_list(list, w),
            Self::Empty(_) => {}
        }
    }

    /// Whether `native_type` is a container (its reference elements are the graph's
    /// dependent references, `ReferenceTable::GetDependentReferences_Native`).
    pub fn is_container(native_type: u32) -> bool {
        containers::is_container(native_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_structures_and_adaptors_round_trip() {
        let position = Position {
            cell: 0x0299_0109,
            origin: [1.0, 2.0, 3.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
        };
        let params = BehaviorParams {
            bitfield: 0x1 | 0x2000 | 0x1_0000 | 0x200,
            behavior: 9,
            target: 0x55,
            vdesc_to_clone: 1,
            cloned_apr: 2,
            cloned_app_apr: vec![(3, 0.5)],
            cloned_app_apr_buckets: 1,
            send_event: 1,
            ..BehaviorParams::default()
        };
        for object in [
            NativeObject::Position(position),
            NativeObject::BehaviorParams(Box::new(params)),
            NativeObject::Empty(ntype::FELLOWSHIP_ADAPTOR),
            NativeObject::WString("Birch".into()),
        ] {
            let mut w = Writer::new();
            object.write(&mut w);
            let mut r = Reader::new(&w.bytes);
            assert_eq!(
                NativeObject::read(object.native_type(), &mut r).unwrap(),
                object
            );
            r.finish().unwrap();
        }
        let mut r = Reader::new(&[]);
        assert!(NativeObject::read(0x2A, &mut r).is_err());
    }
}
