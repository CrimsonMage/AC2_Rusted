//! `--ac2-action MODE[:BEHAVIOR]`: the snapshot's player enters an AC2 mode (`gmmodes.emp`,
//! `combat`, `combat_missile`, ...) and plays a behaviour in it (`gmbehaviors.emp`,
//! `Melee_Auto1`, ...), as a server's `RecvEvt_DoMode` and `RecvEvt_DoBehavior` make it.
//! Names are the DAT's (the mappers' `GetEnum`, case-insensitive); a `0x` number is taken as is.
//! `--ac2-actions` lists both mappers' names.

use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_script::data::{DatScriptData, EnumMapper};
use ac2_script::vm::ScriptData;

/// `gmmodes.emp` and `gmbehaviors.emp` (their bases are the engine's).
const MODES: u32 = 0x2300_0026;
const BEHAVIORS: u32 = 0x2300_0027;
/// Deepest base chain followed.
const BASE_CHAIN: usize = 8;

/// A mode and the behaviour played in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ac2Action {
    pub mode: u32,
    pub behavior: Option<u32>,
}

/// The mapper `did` and its bases.
fn mappers(data: &DatScriptData, mut did: u32) -> Vec<EnumMapper> {
    let mut chain = Vec::new();
    for _ in 0..BASE_CHAIN {
        let Some(mapper) = data.enum_mapper(did) else {
            break;
        };
        let mapper = (*mapper).clone();
        did = mapper.base;
        chain.push(mapper);
        if did == 0 {
            break;
        }
    }
    chain
}

fn lookup(data: &DatScriptData, did: u32, name: &str) -> Result<u32, String> {
    if let Some(hex) = name.strip_prefix("0x") {
        return u32::from_str_radix(hex, 16).map_err(|e| format!("{name}: {e}"));
    }
    mappers(data, did)
        .iter()
        .find_map(|mapper| mapper.value(name.as_bytes()))
        .ok_or_else(|| format!("no {name} in enum mapper {did:#010x} (see --ac2-actions)"))
}

fn data(store: &Arc<ResourceStore>) -> DatScriptData {
    DatScriptData::new(Arc::clone(store.dataset()), (**store).clone())
}

impl Ac2Action {
    /// `MODE` or `MODE:BEHAVIOR`, by name or `0x` number.
    pub fn parse(store: &Arc<ResourceStore>, spec: &str) -> Result<Self, String> {
        let data = data(store);
        let (mode, behavior) = match spec.split_once(':') {
            Some((mode, behavior)) => (mode, Some(behavior)),
            None => (spec, None),
        };
        Ok(Self {
            mode: lookup(&data, MODES, mode)?,
            behavior: behavior
                .map(|name| lookup(&data, BEHAVIORS, name))
                .transpose()?,
        })
    }
}

/// Prints every mode's and behaviour's name and value.
pub fn list(store: &Arc<ResourceStore>) {
    let data = data(store);
    for (title, did) in [("mode", MODES), ("behavior", BEHAVIORS)] {
        for mapper in mappers(&data, did) {
            for (id, name) in &mapper.names {
                println!("{title} {id:#010x} {}", String::from_utf8_lossy(name));
            }
        }
    }
}
