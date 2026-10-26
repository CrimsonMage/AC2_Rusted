//! The server double's character rules on the real `CharacterGenSystem` script (`AC2_DAT_DIR`):
//! `IsValid`, the applied appearance, and the default human male `Explorer` wears.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_net::protocol::CharGenResult;
use ac2_script::data::DatScriptData;
use ac2_script::host::{chargen as host_chargen, ScriptHost};
use ac2_script::native::CharGenResult as NativeCgr;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::Appearance;
use legions_server_double::CharGenRules;

/// The script host the client's screens and the server's rules run on.
pub fn script_host() -> Option<(ScriptHost, Arc<ac2_db::ResourceStore>)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset.clone()));
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, (*store).clone());
    // As the client runs it: everything resident, then cache-only (the frame thread never
    // reads).
    data.preload_tables();
    data.preload_states();
    data.set_cache_only(true);
    Some((ScriptHost::new(program, Box::new(data)), store))
}

/// The native `gmCharGenResult` of the wire result (`gmCharGenResult::operator=`).
pub fn native_of(result: &CharGenResult) -> NativeCgr {
    let mut out = NativeCgr {
        name: result.name.encode_utf16().collect(),
        entity_did: result.entity_did,
        race: result.race,
        sex: result.sex,
        class: result.class,
        implement: result.implement,
        start_area: result.start_area,
        ..NativeCgr::default()
    };
    for &(phys_type, value) in &result.physical {
        out.set_phys_type_value(phys_type, value);
    }
    out
}

/// The server's rules on the `CharacterGenSystem` script: `IsValid` and the applied appearance.
pub struct ScriptRules(pub ScriptHost);

/// A `CharacterGenSystem` host of its own, for the server's rules.
pub fn new_host(store: &Arc<ac2_db::ResourceStore>) -> ScriptHost {
    let wlib = load_wlib(store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let dataset = ac2_dat::test_support::env_dataset().unwrap();
    let data = DatScriptData::new(dataset, (**store).clone());
    data.preload_tables();
    data.preload_states();
    data.set_cache_only(true);
    ScriptHost::new(program, Box::new(data))
}

impl CharGenRules for ScriptRules {
    fn validate(&mut self, result: &CharGenResult) -> u32 {
        host_chargen::is_valid(self.0.vm_mut(), &native_of(result)).unwrap_or(0)
    }

    fn appearance(&mut self, result: &CharGenResult) -> Option<Appearance> {
        host_chargen::applied_appearance(
            self.0.vm_mut(),
            Appearance::default(),
            &native_of(result),
            u32::MAX,
        )
        .map(|(_, appearance)| appearance)
    }

    fn default_appearance(&mut self, race: u32, sex: u32) -> Option<Appearance> {
        let mut cgr = NativeCgr {
            race,
            sex,
            ..NativeCgr::default()
        };
        cgr.entity_did = host_chargen::avatar_entity_did(self.0.vm_mut(), false)?;
        host_chargen::set_defaults(self.0.vm_mut(), &mut cgr)?;
        host_chargen::applied_appearance(self.0.vm_mut(), Appearance::default(), &cgr, u32::MAX)
            .map(|(_, appearance)| appearance)
    }
}
