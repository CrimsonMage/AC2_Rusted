//! Lists the playable race and sex player models (`gCharacterGenSystem::CallIFun_GetPhysObj`)
//! from the DATs, one `race<TAB>sex<TAB>phys_obj` line each (hex), for `world-viewer`.
//!
//! ```text
//! race-table [--dat-dir DIR] [--out FILE]
//! ```
//! Without `--out` the table goes to standard output.

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_script::data::DatScriptData;
use ac2_script::host::{chargen, ScriptHost};
use ac2_script::vm::Program;

fn main() {
    if let Err(message) = run() {
        eprintln!("race-table: {message}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut dat_dir = std::env::var_os("AC2_DAT_DIR").map(PathBuf::from);
    let mut out: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--dat-dir" => dat_dir = Some(value()?.into()),
            "--out" => out = Some(value()?.into()),
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let dat_dir = dat_dir.ok_or("set AC2_DAT_DIR or pass --dat-dir")?;
    let dataset = DatDataset::open(DatOpenOptions::new(&dat_dir)).map_err(|e| e.to_string())?;
    let store = ResourceStore::new(Arc::clone(&dataset));
    let wlib = ac2_script::load_wlib(&store, DataId::new(ac2_script::LEGIONS_WLIB_DID))
        .map_err(|e| e.to_string())?
        .ok_or("the DATs hold no script library")?;
    let program = Program::from_stream(&wlib.stream);
    let mut host = ScriptHost::new(
        Arc::new(program),
        Box::new(DatScriptData::new(dataset, store)),
    );
    let vm = host.vm_mut();
    let (_, races) = chargen::race_sex_data(vm).ok_or("the script has no race and sex data")?;
    let mut table = String::new();
    for (race, sexes) in races {
        for sex in sexes {
            let phys_obj = chargen::phys_obj(vm, race, sex)
                .filter(|&did| did != 0)
                .ok_or(format!("no player model for race {race:x} sex {sex:x}"))?;
            table.push_str(&format!("{race:x}\t{sex:x}\t{phys_obj:08x}\n"));
        }
    }
    match out {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, table).map_err(|e| format!("{}: {e}", path.display()))
        }
        None => std::io::stdout()
            .write_all(table.as_bytes())
            .map_err(|e| e.to_string()),
    }
}
