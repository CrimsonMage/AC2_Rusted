//! Character generation's questions (`gCharacterGenSystem::CallIFun_*`, `EnumMapper`) answered
//! from the script VM: a script host of its own runs the `CharacterGenSystem` singleton (a
//! stateless system script) over the client's script data, so the pregame UI can call it while
//! the player's host waits for a player. Results cross as `ac2_net::protocol::CharGenResult`
//! (the screens' plain data) and `ac2_script::native::CharGenResult` (the script's object).

use ac2_db::RenderInfo;
use ac2_net::protocol::{CharGenResult, GmRaceSexInfo};
use ac2_script::data::{DB_TYPE_DID_MAPPER, DB_TYPE_ENUM_MAPPER, EMAPPER_GROUP};
use ac2_script::host::{chargen, ScriptHost};
use ac2_script::native::CharGenResult as ScriptCgr;
use ac2_ui::pregame::CharGenSystem;
use ac2_weenie::Appearance;

use ac2_ui::chat_text::render_info;

/// The `CharacterGenSystem` on a script host.
pub struct ScriptCharGen {
    host: ScriptHost,
}

impl ScriptCharGen {
    pub fn new(host: ScriptHost) -> Self {
        Self { host }
    }
}

/// The screens' result as the script's object (`gmCharGenResult::operator=`).
fn to_script(cgr: &CharGenResult) -> ScriptCgr {
    let mut out = ScriptCgr {
        name: cgr.name.encode_utf16().collect(),
        entity_did: cgr.entity_did,
        race: cgr.race,
        sex: cgr.sex,
        class: cgr.class,
        implement: cgr.implement,
        start_area: cgr.start_area,
        ..ScriptCgr::default()
    };
    for &(phys_type, value) in &cgr.physical {
        out.set_phys_type_value(phys_type, value);
    }
    out
}

/// The script's object back into the screens' result (values in table order).
fn from_script(cgr: &ScriptCgr, out: &mut CharGenResult) {
    out.name = String::from_utf16_lossy(&cgr.name);
    out.entity_did = cgr.entity_did;
    out.race = cgr.race;
    out.sex = cgr.sex;
    out.class = cgr.class;
    out.implement = cgr.implement;
    out.start_area = cgr.start_area;
    out.physical = cgr.phys_type_values();
}

impl ScriptCharGen {
    fn edit(
        &mut self,
        cgr: &mut CharGenResult,
        call: impl FnOnce(&mut ac2_script::vm::Vm, &mut ScriptCgr) -> Option<u32>,
    ) -> bool {
        let mut script = to_script(cgr);
        let ok = call(self.host.vm_mut(), &mut script).is_some_and(|r| r != 0);
        if ok {
            from_script(&script, cgr);
        }
        ok
    }

    /// `FileArray::GetDIDFromEnum(mapper, EMAPPER_GROUP)`: an `EnumMapper` (with its base
    /// chain) or an `EnumIDMap`, looked up by `lookup`.
    fn lookup<T>(
        &self,
        mapper: u32,
        in_mapper: impl Fn(&ac2_script::data::EnumMapper) -> Option<T>,
        in_id_map: impl Fn(&ac2_script::data::EnumIdMap) -> Option<T>,
    ) -> Option<T> {
        let data = self.host.vm().data();
        let mut did = data.enum_did(mapper, EMAPPER_GROUP).unwrap_or(0);
        if did != 0 && data.db_type(did) == Some(DB_TYPE_DID_MAPPER) {
            return data.enum_id_map(did).and_then(|m| in_id_map(&m));
        }
        for _ in 0..64 {
            if did == 0 || data.db_type(did) != Some(DB_TYPE_ENUM_MAPPER) {
                return None;
            }
            let m = data.enum_mapper(did)?;
            if let Some(found) = in_mapper(&m) {
                return Some(found);
            }
            did = m.base;
        }
        None
    }
}

impl CharGenSystem for ScriptCharGen {
    fn race_sex_data(&mut self) -> Option<Vec<(u32, Vec<u32>)>> {
        chargen::race_sex_data(self.host.vm_mut())
            .filter(|(ret, _)| *ret != 0)
            .map(|(_, races)| races)
    }

    fn race_sex_info(&mut self) -> Vec<(u32, GmRaceSexInfo)> {
        chargen::race_sex_info_data(self.host.vm_mut()).map_or_else(Vec::new, |(_, info)| info)
    }

    fn set_defaults(&mut self, cgr: &mut CharGenResult) -> bool {
        self.edit(cgr, chargen::set_defaults)
    }

    fn randomize(&mut self, phys_mask: u32, cgr: &mut CharGenResult) -> bool {
        self.edit(cgr, |vm, c| chargen::randomize(vm, phys_mask, c))
    }

    fn phys_obj(&mut self, race: u32, sex: u32) -> u32 {
        chargen::phys_obj(self.host.vm_mut(), race, sex).unwrap_or(0)
    }

    fn valid_phys_type_values(&mut self, race: u32, sex: u32, phys_type: u32) -> Option<Vec<f32>> {
        chargen::valid_phys_type_values(self.host.vm_mut(), race, sex, phys_type)
            .filter(|(ret, _)| *ret != 0)
            .map(|(_, values)| values)
    }

    fn phys_type_label(&mut self, race: u32, sex: u32, phys_type: u32) -> Option<RenderInfo> {
        chargen::phys_type_label(self.host.vm_mut(), race, sex, phys_type)
            .filter(|(ok, _)| *ok != 0)
            .map(|(_, label)| render_info(&label))
    }

    fn avatar_entity_did(&mut self, admin: bool) -> u32 {
        chargen::avatar_entity_did(self.host.vm_mut(), admin).unwrap_or(0)
    }

    fn applied_appearance(&mut self, cgr: &CharGenResult) -> Option<Appearance> {
        let script = to_script(cgr);
        chargen::applied_appearance(self.host.vm_mut(), Appearance::default(), &script, u32::MAX)
            .map(|(_, appearance)| appearance)
    }

    fn enum_string(&mut self, mapper: u32, value: u32) -> Option<String> {
        let text = |n: &[u8]| String::from_utf8_lossy(n).into_owned();
        self.lookup(
            mapper,
            |m| m.name(value).map(text),
            |m| m.name(value).map(text),
        )
    }

    fn enum_value(&mut self, mapper: u32, name: &str) -> Option<u32> {
        self.lookup(
            mapper,
            |m| m.value(name.as_bytes()),
            |m| m.name_to_enum(name.as_bytes()),
        )
    }
}
