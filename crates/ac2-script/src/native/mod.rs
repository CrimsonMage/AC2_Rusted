//! Native objects: the engine types a heap object can hold instead of script slots, by native
//! type id (`CNativeFactory::GetNativeTypeName`, `gmCNativeFactory::GetNativeTypeName`).
//!
//! These are the run-time forms the glue works on. The byte codecs
//! (`CNativeFactory::UnPackNativeObject` / `PackNativeObject`) are ac2-net's; [`crate::wire`]
//! converts, and natives no glue edits yet stay in their wire form ([`NativeObject::Wire`]).

mod chargen;
mod containers;
mod hash_table;
mod multi;
mod options;
pub mod social;
mod string_info;
mod structs;

/// `Position` (cell id, origin, rotation w x y z).
pub use ac2_net::protocol::Position;
pub use chargen::CharGenResult;
pub use containers::{RefKey, ScriptList};
pub use hash_table::{
    bucket_size_index, HashNode, HashPos, NativeKeys, PlainKeys, ScriptHash, BUCKET_SIZES,
    DEFAULT_BUCKETS,
};
pub use multi::{MultiNode, ScriptMultiHash, ScriptMultiMap, STL_PRIMES};
pub use options::{
    assign_ui_locations, bits as option_bits, word_assign, word_get, word_set, AliasTable,
    GameplayOptions,
};
pub use string_info::{GetStringError, StringInfo, StringVar, StringVarValue};
pub use structs::{
    behavior_defaults, behavior_pack_bitfield, BehaviorParams, MissileParameters,
    RandomSelectionTable,
};

/// Native type ids (`CNativeFactory` 0x01-0x76 and the 0x2Exx-0x38xx extensions,
/// `gmCNativeFactory` 0x82-0xAF).
pub mod ntype {
    pub const AA_HASH: u32 = 0x01;
    pub const AA_MULTI_HASH: u32 = 0x03;
    pub const A_ARRAY: u32 = 0x05;
    pub const A_HASH_SET: u32 = 0x06;
    pub const AL_HASH: u32 = 0x08;
    pub const A_LIST: u32 = 0x0A;
    pub const A_LIST_ITER: u32 = 0x0B;
    pub const AR_HASH: u32 = 0x11;
    pub const BEHAVIOR_PARAMS: u32 = 0x17;
    pub const LA_HASH: u32 = 0x32;
    pub const L_ARRAY: u32 = 0x37;
    pub const LL_HASH: u32 = 0x39;
    pub const L_LIST: u32 = 0x3B;
    pub const L_LIST_ITER: u32 = 0x3C;
    pub const LR_HASH: u32 = 0x40;
    pub const MISSILE_PARAMETERS: u32 = 0x46;
    pub const NA_HASH: u32 = 0x4B;
    pub const NR_HASH: u32 = 0x4D;
    pub const HEADING: u32 = 0x2D;
    pub const CHAR_GEN_RESULT: u32 = 0x9E;
    pub const PACK_BUFFER: u32 = 0x50;
    pub const POSITION: u32 = 0x5A;
    pub const RANDOM_SELECTION_TABLE_INT: u32 = 0x60;
    pub const R_ARRAY: u32 = 0x61;
    pub const R_LIST: u32 = 0x65;
    pub const R_LIST_ITER: u32 = 0x66;
    pub const STRING_INFO: u32 = 0x6A;
    pub const VECTOR: u32 = 0x70;
    pub const WPSTRING: u32 = 0x76;
    pub const LA_HASH_SET: u32 = 0x2E0B;

    /// The hash and hash-set types; each one's iterator type is the next id.
    pub const HASHES: [u32; 10] = [
        AA_HASH,
        A_HASH_SET,
        AL_HASH,
        AR_HASH,
        LA_HASH,
        LL_HASH,
        LR_HASH,
        NA_HASH,
        NR_HASH,
        LA_HASH_SET,
    ];

    pub fn is_hash_iter(t: u32) -> bool {
        HASHES.iter().any(|&h| h + 1 == t)
    }

    /// `AAMultiHash`, `ALMultiHash`, `ARMultiHash`, `LAMultiHash`, `LLMultiHash`,
    /// `LRMultiHash`; each one's iterator type is the next id.
    pub const MULTI_HASHES: [u32; 6] = [0x03, 0x0C, 0x13, 0x34, 0x3D, 0x42];
    /// `AAMultiMap`, `ALMultiMap`, `LAMultiMap`, `LLMultiMap`; iterators are the next id.
    pub const MULTI_MAPS: [u32; 4] = [0x3691, 0x3693, 0x3695, 0x3697];

    pub fn is_multi_iter(t: u32) -> bool {
        MULTI_HASHES.iter().chain(&MULTI_MAPS).any(|&h| h + 1 == t)
    }
    pub const EXAMINATION_REQUEST: u32 = 0x9A;
    pub const GAMEPLAY_OPTIONS_PROFILE: u32 = 0x9B;
    pub const SHORTCUT_INFO: u32 = 0xAC;
    pub const RGBA_COLOR: u32 = 0x64;
    pub const DAMAGE_TEXT_BLOB: u32 = 0x97;
    pub const CRAFT_SKILL_ADAPTOR: u32 = 0x33A5;
    pub const UI_SHOP: u32 = 0x380A;
    pub const UI_SHOP_SALE: u32 = 0x380B;
    pub const GM_QUEST_INFO: u32 = 0xA0;
    pub const GM_QUEST_INFO_LIST: u32 = 0xA1;
    pub const ALLEGIANCE_DATA_ADAPTOR: u32 = 0x82;
    pub const ALLEGIANCE_PROFILE_ADAPTOR: u32 = 0x83;
    pub const FELLOW_ADAPTOR: u32 = 0x93;
    pub const FELLOWSHIP_ADAPTOR: u32 = 0x94;
    pub const QUALIFIED_DATA_ID_ARRAY: u32 = 0x5E;
    pub const SELECTION_INFO: u32 = 0xAB;
    pub const UI_SAVE_LOCATIONS: u32 = 0x6F;
    pub const GM_RACE_SEX_INFO: u32 = 0xA2;
    pub const GM_SCENE_INFO: u32 = 0xA3;
    pub const SKILL_UI_NODE: u32 = 0xAE;
}

/// One `PackBuffer` value (`gPackBuffer::CallEFun_Pack*` tags 1-5); a package is packed when
/// it is added (`PackBuffer::PackPackage` calls `VM::PackReference` then).
#[derive(Debug, Clone, PartialEq)]
pub enum PackItem {
    Int(u32),
    LongInt(u64),
    Float(u32),
    LongFloat(u64),
    Package(crate::reference::PackedReference),
}

/// One unpacked native object.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeObject {
    /// `AList`, `RList`, `AArray`, `RArray` (32-bit elements) and `LList`, `LArray` (64-bit).
    List(ScriptList),
    /// The `XYHash` family and the hash sets.
    Hash(ScriptHash),
    /// The `XYMultiHash` family.
    MultiHash(ScriptMultiHash),
    /// The `XYMultiMap` family.
    MultiMap(ScriptMultiMap),
    /// `BaseMultiHashIter` / `BaseMultiMapIter`: the container and the element (`None` at the
    /// end).
    MultiIter {
        container: u32,
        node: Option<u32>,
    },
    /// `BaseHashIter` / `BaseHashSetIter`: the hash (`m_ref`) and the position.
    HashIter {
        hash: u32,
        pos: HashPos,
    },
    StringInfo(StringInfo),
    /// `wpstring` (`PStringBase<unsigned short>`).
    WString(Vec<u16>),
    /// `gmCharGenResult`.
    CharGenResult(Box<CharGenResult>),
    /// `Heading` (degrees in [0, 360)).
    Heading(ac2_physics::Heading),
    /// `Vector` (x, y, z).
    Vector([f32; 3]),
    Position(Position),
    MissileParameters(MissileParameters),
    BehaviorParams(Box<BehaviorParams>),
    RandomSelectionTable(RandomSelectionTable),
    /// `QualifiedDataIDArray`: the set of (DB type, DID) pairs (`std::set<QualifiedDataID>`;
    /// scripts only add to it).
    DataIds(std::collections::BTreeSet<(u32, u32)>),
    /// `FellowAdaptor`.
    Fellow(Box<ac2_weenie::FellowInfo>),
    /// `FellowshipAdaptor` (fellows in the order added; see [`social`]).
    Fellowship(Box<ac2_weenie::FellowshipInfo>),
    /// `AllegianceDataAdaptor`.
    AllegianceData(Box<ac2_weenie::AllegianceMember>),
    /// `AllegianceProfileAdaptor`.
    AllegianceProfile(Box<ac2_weenie::AllegianceProfile>),
    /// `UIShop`.
    Shop(Box<ac2_weenie::Shop>),
    /// `UIShopSale`.
    ShopSale(Box<ac2_weenie::ShopSale>),
    /// `CraftSkillAdaptor`.
    CraftSkill(Box<ac2_weenie::CraftSkill>),
    /// `DamageTextBlob`.
    DamageText(Box<ac2_weenie::DamageText>),
    /// `RGBAColor` (r, g, b, a).
    Color([f32; 4]),
    /// `GameplayOptionsProfile` (Legions layout).
    GameplayOptions(Box<GameplayOptions>),
    /// `BaseListIter`: a position in a list (`None` past the end). Retail holds the list node;
    /// the port holds its index, which agrees for everything done through the iterator.
    ListIter {
        list: u32,
        pos: Option<usize>,
    },
    /// `PackBuffer`: the tagged values a script packs for a server call, in order.
    PackBuffer(std::collections::VecDeque<PackItem>),
    /// A native that came over the wire and has no run-time form yet; it goes back out as it
    /// came.
    Wire(Box<ac2_net::protocol::NativeObject>),
    /// A native type made by `NewNative` that the port has no value for yet.
    Unbound(u32),
}

impl NativeObject {
    /// `CNativeFactory::NewNativeObject`: an empty object of `native_type`.
    pub fn new_empty(native_type: u32) -> Self {
        let list = |wide: bool| {
            NativeObject::List(ScriptList {
                wide,
                items: Vec::new(),
            })
        };
        match native_type {
            ntype::A_LIST | ntype::R_LIST | ntype::A_ARRAY | ntype::R_ARRAY => list(false),
            ntype::L_LIST | ntype::L_ARRAY => list(true),
            t if ntype::MULTI_HASHES.contains(&t) => {
                NativeObject::MultiHash(ScriptMultiHash::default())
            }
            t if ntype::MULTI_MAPS.contains(&t) => {
                NativeObject::MultiMap(ScriptMultiMap::default())
            }
            t if ntype::is_multi_iter(t) => NativeObject::MultiIter {
                container: u32::MAX,
                node: None,
            },
            ntype::AA_HASH
            | ntype::A_HASH_SET
            | ntype::AL_HASH
            | ntype::AR_HASH
            | ntype::LA_HASH
            | ntype::LL_HASH
            | ntype::LR_HASH
            | ntype::NA_HASH
            | ntype::NR_HASH
            | ntype::LA_HASH_SET => NativeObject::Hash(ScriptHash::default()),
            ntype::STRING_INFO => NativeObject::StringInfo(StringInfo::default()),
            ntype::WPSTRING => NativeObject::WString(Vec::new()),
            ntype::VECTOR => NativeObject::Vector([0.0; 3]),
            t if ntype::is_hash_iter(t) => NativeObject::HashIter {
                hash: u32::MAX,
                pos: HashPos {
                    bucket: 0,
                    node: None,
                },
            },
            ntype::A_LIST_ITER | ntype::L_LIST_ITER | ntype::R_LIST_ITER => {
                NativeObject::ListIter {
                    list: u32::MAX,
                    pos: None,
                }
            }
            ntype::POSITION => NativeObject::Position(Position {
                cell: 0,
                origin: [0.0; 3],
                rotation: [1.0, 0.0, 0.0, 0.0],
            }),
            ntype::RANDOM_SELECTION_TABLE_INT => {
                NativeObject::RandomSelectionTable(RandomSelectionTable::default())
            }
            ntype::BEHAVIOR_PARAMS => NativeObject::BehaviorParams(Box::new(behavior_defaults())),
            ntype::PACK_BUFFER => NativeObject::PackBuffer(Default::default()),
            ntype::HEADING => NativeObject::Heading(ac2_physics::Heading::ZERO),
            ntype::CHAR_GEN_RESULT => NativeObject::CharGenResult(Box::default()),
            ntype::GM_RACE_SEX_INFO => NativeObject::Wire(Box::new(
                ac2_net::protocol::NativeObject::GmRaceSexInfo(crate::glue::empty_race_sex_info()),
            )),
            ntype::SKILL_UI_NODE => {
                NativeObject::Wire(Box::new(ac2_net::protocol::NativeObject::SkillUiNode(
                    Box::new(crate::glue::empty_skill_node()),
                )))
            }
            ntype::GAMEPLAY_OPTIONS_PROFILE => NativeObject::GameplayOptions(Box::default()),
            ntype::UI_SHOP => NativeObject::Shop(Box::default()),
            ntype::UI_SHOP_SALE => NativeObject::ShopSale(Box::default()),
            ntype::CRAFT_SKILL_ADAPTOR => NativeObject::CraftSkill(Box::default()),
            ntype::DAMAGE_TEXT_BLOB => NativeObject::DamageText(Box::default()),
            ntype::RGBA_COLOR => NativeObject::Color([0.0; 4]),
            ntype::GM_QUEST_INFO => NativeObject::Wire(Box::new(
                ac2_net::protocol::NativeObject::GmQuestInfo(Box::default()),
            )),
            ntype::GM_QUEST_INFO_LIST => NativeObject::Wire(Box::new(
                ac2_net::protocol::NativeObject::GmQuestInfoList(Vec::new()),
            )),
            ntype::FELLOW_ADAPTOR => NativeObject::Fellow(Box::new(social::fellow())),
            ntype::FELLOWSHIP_ADAPTOR => NativeObject::Fellowship(Box::new(social::fellowship())),
            ntype::ALLEGIANCE_DATA_ADAPTOR => {
                NativeObject::AllegianceData(Box::new(social::allegiance_member()))
            }
            ntype::ALLEGIANCE_PROFILE_ADAPTOR => {
                NativeObject::AllegianceProfile(Box::new(social::allegiance_profile()))
            }
            ntype::QUALIFIED_DATA_ID_ARRAY => NativeObject::DataIds(Default::default()),
            ntype::SELECTION_INFO => NativeObject::Wire(Box::new(
                ac2_net::protocol::NativeObject::SelectionInfo(Default::default()),
            )),
            ntype::EXAMINATION_REQUEST => NativeObject::Wire(Box::new(
                ac2_net::protocol::NativeObject::ExaminationRequest(Default::default()),
            )),
            ntype::SHORTCUT_INFO => {
                NativeObject::Wire(Box::new(ac2_net::protocol::NativeObject::ShortcutInfo(
                    ac2_net::protocol::ShortcutInfo::Empty(0),
                )))
            }
            ntype::UI_SAVE_LOCATIONS => {
                NativeObject::Wire(Box::new(ac2_net::protocol::NativeObject::UiSaveLocations(
                    GameplayOptions::default().ui_locations,
                )))
            }
            ntype::MISSILE_PARAMETERS => {
                NativeObject::MissileParameters(MissileParameters::default())
            }
            other => NativeObject::Unbound(other),
        }
    }
}
