//! `gmCharGenResult` and `gCharacterGenSystem`: the character the creation screens build
//! (name, appearance entity, race, sex, class, implement, start area and the physique sliders'
//! values, as `ac2_net::protocol::CharGenResult`), and the questions the screens ask the
//! `CharacterGenSystem` script about it ([`CharGenSystem`], which the client answers from the
//! script VM).

use ac2_db::RenderInfo;
use ac2_net::protocol::{CharGenResult, GmRaceSexInfo};
use ac2_weenie::Appearance;

/// The physique types (`*_PhysiqueType`), one bit each.
pub mod phys_type {
    pub const BUILD: u32 = 1;
    pub const HEIGHT: u32 = 2;
    pub const SKIN_TONE: u32 = 4;
    pub const SKIN_DETAIL: u32 = 8;
    pub const HEAD_DETAIL: u32 = 16;
    pub const HEAD_FRILL: u32 = 32;
    pub const FRILL_COLOR: u32 = 64;
    pub const SPECIAL: u32 = 128;
    pub const SHIRT_CLOTHING_COLOR: u32 = 256;
    pub const PANTS_CLOTHING_COLOR: u32 = 512;
    pub const BOOTS_CLOTHING_COLOR: u32 = 1024;
    pub const FACE_DETAIL: u32 = 2048;
    /// `ClothingMask_PhysiqueType`.
    pub const CLOTHING_MASK: u32 = 0x700;
}

/// `EnumMapper` enums the screens name races and sexes by.
pub const RACE_NAMES: u32 = 0x4100_0009;
pub const SEX_NAMES: u32 = 0x4100_000B;
/// The avatar locations in the avatar selection scene, by `sex + race` name.
pub const AVATAR_LOCATIONS: u32 = 0x4000_000B;

/// `gCharacterGenSystem::CallIFun_*` (the `CharacterGenSystem` singleton script) and the
/// `EnumMapper` lookups the creation screens make.
pub trait CharGenSystem {
    /// `CallIFun_InqRaceSexData`: each race and its sexes (`None`: the call failed).
    fn race_sex_data(&mut self) -> Option<Vec<(u32, Vec<u32>)>>;
    /// `CallIFun_InqRaceSexInfoData`: per `race + sex`, the physics object, appearance file,
    /// music and the objects held at locations.
    fn race_sex_info(&mut self) -> Vec<(u32, GmRaceSexInfo)>;
    /// `CallIFun_SetCGRDefaults`.
    fn set_defaults(&mut self, cgr: &mut CharGenResult) -> bool;
    /// `CallIFun_RandomizeCGR(physMask, cgr)`.
    fn randomize(&mut self, phys_mask: u32, cgr: &mut CharGenResult) -> bool;
    /// `CallIFun_GetPhysObj(race, sex)`.
    fn phys_obj(&mut self, race: u32, sex: u32) -> u32;
    /// `CallIFun_InqValidPhysTypeValues(race, sex, physType)` (`None`: not offered).
    fn valid_phys_type_values(&mut self, race: u32, sex: u32, phys_type: u32) -> Option<Vec<f32>>;
    /// `CallIFun_InqPhysTypeLabel(race, sex, physType)`.
    fn phys_type_label(&mut self, race: u32, sex: u32, phys_type: u32) -> Option<RenderInfo>;
    /// `CallIFun_InqAdminAvatarEntityDID` / `InqPlayerAvatarEntityDID`.
    fn avatar_entity_did(&mut self, admin: bool) -> u32;
    /// `gmCharGenResult::InqAppliedAppearanceVDesc(0xFFFFFFFF, cgr)`: the appearance the result's
    /// physique values apply (`None`: the script failed).
    fn applied_appearance(&mut self, cgr: &CharGenResult) -> Option<Appearance>;
    /// `EnumMapper::GetString(mapper, value)`.
    fn enum_string(&mut self, mapper: u32, value: u32) -> Option<String>;
    /// `EnumMapper::GetEnum(mapper, name)`.
    fn enum_value(&mut self, mapper: u32, name: &str) -> Option<u32>;
}

/// No `CharacterGenSystem`: every call fails.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCharGen;

impl CharGenSystem for NoCharGen {
    fn race_sex_data(&mut self) -> Option<Vec<(u32, Vec<u32>)>> {
        None
    }

    fn race_sex_info(&mut self) -> Vec<(u32, GmRaceSexInfo)> {
        Vec::new()
    }

    fn set_defaults(&mut self, _: &mut CharGenResult) -> bool {
        false
    }

    fn randomize(&mut self, _: u32, _: &mut CharGenResult) -> bool {
        false
    }

    fn phys_obj(&mut self, _: u32, _: u32) -> u32 {
        0
    }

    fn valid_phys_type_values(&mut self, _: u32, _: u32, _: u32) -> Option<Vec<f32>> {
        None
    }

    fn phys_type_label(&mut self, _: u32, _: u32, _: u32) -> Option<RenderInfo> {
        None
    }

    fn avatar_entity_did(&mut self, _: bool) -> u32 {
        0
    }

    fn applied_appearance(&mut self, _: &CharGenResult) -> Option<Appearance> {
        None
    }

    fn enum_string(&mut self, _: u32, _: u32) -> Option<String> {
        None
    }

    fn enum_value(&mut self, _: u32, _: &str) -> Option<u32> {
        None
    }
}

/// `gmCharGenResult::gmCharGenResult(race, sex)`: the player avatar (`SetAdmin(0)`), then the
/// script's defaults (`InitDefaults`, `CallIFun_SetCGRDefaults`).
pub fn new_char_gen(system: &mut dyn CharGenSystem, race: u32, sex: u32) -> CharGenResult {
    let mut cgr = CharGenResult {
        race,
        sex,
        ..CharGenResult::default()
    };
    set_admin(system, &mut cgr, false);
    system.set_defaults(&mut cgr);
    cgr
}

/// `gmCharGenResult::SetAdmin`: the admin or the player avatar entity.
pub fn set_admin(system: &mut dyn CharGenSystem, cgr: &mut CharGenResult, admin: bool) {
    cgr.entity_did = system.avatar_entity_did(admin);
}

/// `gmCharGenResult::GetPhysTypeValue`: -1 when unset.
pub fn phys_type_value(cgr: &CharGenResult, phys_type: u32) -> f32 {
    cgr.physical
        .iter()
        .find(|(t, _)| *t == phys_type)
        .map_or(-1.0, |(_, v)| *v)
}

/// `gmCharGenResult::SetPhysTypeValue` (`HashTable::set`: a set key keeps its place).
pub fn set_phys_type_value(cgr: &mut CharGenResult, phys_type: u32, value: f32) {
    match cgr.physical.iter_mut().find(|(t, _)| *t == phys_type) {
        Some(slot) => slot.1 = value,
        None => cgr.physical.push((phys_type, value)),
    }
}

/// The key a race and sex's result is kept under (`UIPersistantData::SetCharGenResult`).
pub fn result_key(race: u32, sex: u32) -> u32 {
    race.wrapping_add(sex)
}
