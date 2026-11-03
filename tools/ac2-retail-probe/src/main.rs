//! Read-only reference oracle. Run the installed DAT's character-generation
//! bytecode through the pinned reference interpreter, independently of UE's port.
//! Content modes decode retained records and bytecode for evidence-backed authoring.
use std::{path::PathBuf, sync::Arc};

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_script::{
    data::DatScriptData,
    host::{chargen, ScriptHost},
    native::CharGenResult,
    vm::Program,
};
use serde_json::json;
mod content;
mod bytecode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = PathBuf::from(std::env::args_os().nth(1).ok_or("Expected DAT directory")?);
    let dataset = DatDataset::open(DatOpenOptions::for_retail_install(directory))?;
    let store = ResourceStore::new(dataset.clone());
    if std::env::args().nth(2).as_deref() == Some("--loose-strings") {
        use ac2_db::ResourcePackedDecode;
        let root = PathBuf::from(std::env::args_os().nth(3).ok_or("Expected loose string-table directory")?);
        for arg in std::env::args().skip(4) {
            let did = u32::from_str_radix(arg.trim_start_matches("0x"), 16)?;
            if !(ac2_db::STRING_TABLE_FIRST..=ac2_db::STRING_TABLE_LAST).contains(&did) {
                return Err("Expected StringTable DID".into());
            }
            let path = root.join(format!("0x{did:08X}"));
            if !path.exists() {
                println!("{}", json!({"table":format!("{did:08x}"),"error":"missing loose record"}));
                continue;
            }
            if path.metadata()?.len() > 16 * 1024 * 1024 {
                return Err("Loose StringTable exceeds size bound".into());
            }
            let bytes = std::fs::read(path)?;
            let table = ac2_db::StringTable::decode_packed(
                ac2_core::QualifiedDataId::new(DataId::new(did), 0), 0, &bytes)?;
            let entries: std::collections::BTreeMap<_,_> = table.strings.iter().map(|(key,value)|
                (format!("{key:08x}"), json!({"text":value.text(),"private_table":value.table,
                    "variables":value.variables}))).collect();
            println!("{}", json!({"table":format!("{did:08x}"),"entries":entries}));
        }
        return Ok(());
    }
    if std::env::args().nth(2).as_deref() == Some("--strings") {
        for arg in std::env::args().skip(3) {
            let did=u32::from_str_radix(arg.trim_start_matches("0x"),16)?;
            if let Some(table)=ac2_db::load_string_table(&store,DataId::new(did))? {
                let entries:std::collections::BTreeMap<_,_>=table.strings.iter().map(|(key,value)|
                    (format!("{key:08x}"),json!({"text":value.text(),"variables":value.variables}))).collect();
                println!("{}",json!({"table":format!("{did:08x}"),"entries":entries}));
            }
        }
        return Ok(());
    }
    if std::env::args().nth(2).as_deref() == Some("--state") {
        for arg in std::env::args().skip(3) {
            let did = u32::from_str_radix(arg.trim_start_matches("0x"), 16)?;
            let state = ac2_script::wstate::load_wstate(&store, DataId::new(did))?;
            if let Some(state)=state {
                println!("{did:08x} pid {:?}",state.pid);
                match &state.references {
                    Ok(refs)=>for (id,entry) in refs.ids.iter().zip(&refs.entries) {println!("{id}: {entry:?}");},
                    Err(e)=>println!("{e:?}"),
                }
            } else {println!("{did:08x}: missing");}
        }
        return Ok(());
    }
    if std::env::args().nth(2).as_deref() == Some("--clock") {
        let source = ac2_world::Ac2WorldSource::open(Arc::new(store))?.ok_or("No world source")?;
        let region = source.region().ok_or("No game calendar")?;
        let mut cases = Vec::new();
        for seconds in [0., 1., 1234.5, 2453.999, 2454., 9000., 54321.,
            2640026.999, 2640027., 1.0e6, 1.0e9] {
            let mut clock = region.game_clock();
            clock.use_time(seconds);
            let (year, day, _, _) = clock.calendar();
            cases.push(json!({"seconds":seconds,"year":year,"day":day,
                "fraction":clock.present_time_of_day()}));
        }
        println!("{}", serde_json::to_string_pretty(&json!({
            "source":"Legions DAT calendar through pinned GameTime::UseTime reference",
            "cases":cases}))?);
        return Ok(());
    }
    let library = ac2_script::load_wlib(&store, DataId::new(ac2_script::LEGIONS_WLIB_DID))?
        .ok_or("Missing script library")?;
    if std::env::args().nth(2).as_deref() == Some("--code") {
        println!("{}",serde_json::to_string_pretty(&bytecode::listing(&library.stream,&std::env::args().skip(3).collect::<Vec<_>>()))?);
        return Ok(());
    }
    if std::env::args().nth(2).as_deref() == Some("--content") {
        println!("{}", serde_json::to_string_pretty(&content::recover(&store, &library.stream))?);
        return Ok(());
    }
    let mut host = ScriptHost::new(
        Arc::new(Program::from_stream(&library.stream)),
        Box::new(DatScriptData::new(dataset, store)),
    );
    if std::env::args().nth(2).as_deref() == Some("--radar") {
        for arg in std::env::args().skip(3) {
            let pid: u32 = arg.parse()?;
            host.create_weenie(1, pid, 0);
            println!("{pid}: {:?}",host.inq_radar_look(1));
            host.remove_weenie(1);
        }
        return Ok(());
    }
    let vm = host.vm_mut();
    let (_, races) = chargen::race_sex_data(vm).ok_or("Race list script failed")?;
    let (_, race_info) = chargen::race_sex_info_data(vm).ok_or("Race/sex info script failed")?;
    let mut avatars = Vec::new();
    for (race, sexes) in races {
        for sex in sexes {
            let mut result = CharGenResult {
                race,
                sex,
                ..Default::default()
            };
            if chargen::set_defaults(vm, &mut result) != Some(1) {
                return Err(format!("SetCGRDefaults failed: {:?}", vm.last_fault).into());
            }
            let mut defaults = result.phys_type_values();
            defaults.sort_by_key(|(kind, _)| *kind);
            let mut choices = Vec::new();
            for (kind, value) in defaults {
                // The pinned wrapper pushes arguments in declaration order.
                // Retail CallIFun_InqValidPhysTypeValues pushes kind, sex, race
                // (admin 7148 and Legions 7160), so compensate at this boundary.
                let (_, values) = chargen::valid_phys_type_values(vm, kind, sex, race)
                    .ok_or("Valid-physique-values script failed")?;
                let index = values.iter().position(|v| v.to_bits() == value.to_bits());
                choices.push(json!({"kind": kind, "value": value,
                    "index": index, "count": values.len(), "values": values}));
            }
            let info = race_info.iter().find(|(key, _)| *key == race | sex)
                .map(|(_, info)| info).ok_or("Race/sex info missing")?;
            avatars.push(json!({"race": race, "sex": sex, "choices": choices,
                "physics": info.phys_obj_did, "held_objects": info.obj_in_loc}));
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "source": "DAT SetCGRDefaults / InqValidPhysTypeValues via pinned AC2_Rusted VM",
            "avatars": avatars
        }))?
    );
    Ok(())
}
