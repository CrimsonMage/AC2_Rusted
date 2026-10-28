//! Plain data natives whose glue only stores and returns fields: `LinkRequirements` (0x38),
//! `MissileInfo` (0x45), `MissileTargetingParameters` (0x47) and `FXAndBehaviorInfo` (0x2B).
//! Field order and offsets follow the retail classes (`PackObj` header first, so the first field
//! is at +4).

use super::{CollisionParameters, MissileParameters, Position};
use crate::native::BehaviorParams;

/// `LinkRequirements`: the five words `gLinkRequirements`'s efuns read and write (+4 .. +0x14).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LinkRequirements {
    /// +4, `IsInputLink` / `SetInputLink`.
    pub input_link: u32,
    /// +8, `GetLinkType` / `SetLinkType`.
    pub link_type: u32,
    /// +0xC, `GetNumLinks` / `SetNumLinks`.
    pub num_links: u32,
    /// +0x10, `IsRequired` / `SetRequired`.
    pub required: u32,
    /// +0x14, `GetNumReqLinks` / `SetNumReqLinks`.
    pub num_req_links: u32,
}

/// `MissileTargetingParameters` (`m_bitfield` names the groups a script set).
#[derive(Debug, Clone, PartialEq)]
pub struct MissileTargeting {
    pub bitfield: u32,
    pub target_id: u64,
    pub target_height_adjustment: f32,
    pub use_target_scale: u32,
    pub target_offset: [f32; 3],
    pub target_pos: Position,
    pub base_pos: Position,
    pub source_id: u64,
    pub source_height_adjustment: f32,
    pub use_source_scale: u32,
    pub source_offset: [f32; 3],
    pub initial_velocity: f32,
    pub initial_pitch: f32,
    pub missile_range: f32,
    pub missile_detonation_chance: f32,
}

impl Default for MissileTargeting {
    /// Nothing set: every group bit clear, ids 0, identity positions.
    fn default() -> Self {
        let identity = Position {
            cell: 0,
            origin: [0.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0],
        };
        Self {
            bitfield: 0,
            target_id: 0,
            target_height_adjustment: 0.0,
            use_target_scale: 0,
            target_offset: [0.0; 3],
            target_pos: identity,
            base_pos: identity,
            source_id: 0,
            source_height_adjustment: 0.0,
            use_source_scale: 0,
            source_offset: [0.0; 3],
            initial_velocity: 0.0,
            initial_pitch: 0.0,
            missile_range: 0.0,
            missile_detonation_chance: 0.0,
        }
    }
}

/// `MissileInfo`: what a missile launch hands the physics object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MissileInfo {
    /// `m_entityDID` (+4).
    pub entity_did: u32,
    /// `m_holdingLocation` (+8).
    pub holding_location: u32,
    pub missile_params: Option<MissileParameters>,
    pub targeting_params: MissileTargeting,
    pub collision_params: CollisionParameters,
    /// `SetThrown`.
    pub thrown: bool,
    /// `SetClone`.
    pub clone: bool,
}

/// `FXAndBehaviorInfo`: the FX ids and behaviours of an attack.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FxAndBehavior {
    pub attacker_fx: u32,
    pub target_fx: u32,
    pub weapon_fx: u32,
    pub target_behavior: Option<Box<BehaviorParams>>,
    pub weapon_behavior: Option<Box<BehaviorParams>>,
}
