//! The DAT's scripts (`AC2_DAT_DIR`): every `DMSC` script parses, and every routine a segment's
//! script track calls runs without error at every groove level, playing only objects its container
//! holds. The checks read the DAT; nothing from it is stored.

use std::sync::Arc;

use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use super::{bind_segment, parse, Instance, ScriptHost};
use crate::music::library::MusicLibrary;
use crate::music::segment::TrackData;
use crate::music::MusicObject;

/// The `DMUS_SEGF_QUEUE` value `AtFinish` names (see the module doc).
const AT_FINISH: f64 = 256.0;

/// A host that records the segments a routine plays, at a fixed groove level.
struct Plays {
    groove: f64,
    played: Vec<(String, f64)>,
}

impl ScriptHost for Plays {
    fn constant(&self, name: &str) -> Option<f64> {
        name.eq_ignore_ascii_case("atfinish").then_some(AT_FINISH)
    }

    fn function(&mut self, name: &str, args: &[f64]) -> Result<f64, String> {
        if name.eq_ignore_ascii_case("getmastergroovelevel") && args.is_empty() {
            Ok(self.groove)
        } else {
            Err(format!("no function {name}"))
        }
    }

    fn method(&mut self, object: &str, method: &str, args: &[f64]) -> Result<(), String> {
        if !method.eq_ignore_ascii_case("play") {
            return Err(format!("{method} is not in the subset"));
        }
        self.played.push((
            object.to_ascii_lowercase(),
            args.first().copied().unwrap_or(0.0),
        ));
        Ok(())
    }
}

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

/// Every `DBMusicObj` record of the portal DAT: its DID and object.
fn records(library: &MusicLibrary, store: &ResourceStore) -> Vec<(u32, MusicObject)> {
    store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .filter_map(|did| library.record(did).map(|object| (did, object)))
        .collect()
}

#[test]
fn every_dat_script_parses_and_has_its_routine() {
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let scripts: Vec<(u32, String)> = records(&library, &store)
        .into_iter()
        .filter_map(|(did, object)| match object {
            MusicObject::Script(script) => Some((did, script.source.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(scripts.len(), 14, "the DAT holds 14 scripts");
    for (did, source) in &scripts {
        let program = parse(source).unwrap_or_else(|e| panic!("script {did:08X}: {e}"));
        assert!(
            program.routine("ChooseGroove").is_some(),
            "script {did:08X} has ChooseGroove"
        );
    }
}

#[test]
fn every_routine_a_track_calls_runs_at_every_groove_level_and_plays_its_container() {
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let mut calls_checked = 0;
    for (_, object) in records(&library, &store) {
        let MusicObject::Segment(segment) = &object else {
            continue;
        };
        if !segment
            .tracks
            .iter()
            .any(|t| matches!(t.data, TrackData::Scripts(_)))
        {
            continue;
        }
        let bound = bind_segment(&library.closure(object.clone()));
        assert!(!bound.calls.is_empty(), "the script track's call binds");
        for call in &bound.calls {
            for groove in (0..=30).chain([40, 100]) {
                let mut host = Plays {
                    groove: f64::from(groove),
                    played: Vec::new(),
                };
                Instance::default()
                    .call(&call.script.program, &call.routine, &mut host)
                    .unwrap_or_else(|e| panic!("{} at groove {groove}: {e}", call.routine));
                assert_eq!(
                    host.played.len(),
                    1,
                    "one branch plays at groove {groove}: {:?}",
                    host.played
                );
                let (alias, flags) = &host.played[0];
                assert_eq!(*flags, AT_FINISH, "the segment plays at finish");
                assert!(
                    call.script.targets.contains_key(alias),
                    "{alias} is in the script's container"
                );
                calls_checked += 1;
            }
        }
    }
    assert!(calls_checked > 0, "some script calls were checked");
}

#[test]
fn the_hum_dorian_script_plays_the_branch_its_source_names() {
    // 'Cue_Hum_C_Dorian' (0x2400_0467): groove below 5 plays Dorian, 5 to 9 Phrygian, above 9
    // WholeHalf (the source's If / ElseIf).
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let cue = library.record(0x2400_0467).expect("the segment");
    let bound = bind_segment(&library.closure(cue));
    let call = &bound.calls[0];
    for (groove, expected) in [
        (0.0, "mi_hum_c_dorian"),
        (4.0, "mi_hum_c_dorian"),
        (5.0, "mi_hum_c_phrygian"),
        (9.0, "mi_hum_c_phrygian"),
        (10.0, "mi_hum_c_wholehalf"),
    ] {
        let mut host = Plays {
            groove,
            played: Vec::new(),
        };
        Instance::default()
            .call(&call.script.program, &call.routine, &mut host)
            .unwrap();
        assert_eq!(host.played[0].0, expected, "at groove {groove}");
    }
}
