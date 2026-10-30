//! The ACE weenies of `ACE_WEENIE_DIR` built from the AC1 DATs of `AC1_DAT_DIR`; skipped when
//! either is unset.

use ac1_assets::{Ac1Dats, AceWeenie};
use ac2_core::DataId;

#[test]
fn every_weenie_builds_a_drawn_model_with_an_idle_cycle() {
    let (Some(dats), Some(dir)) = (Ac1Dats::from_env(), std::env::var_os("ACE_WEENIE_DIR")) else {
        eprintln!("AC1_DAT_DIR or ACE_WEENIE_DIR unset; skipped");
        return;
    };
    let dats = dats.expect("AC1 DATs mount");
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .expect("weenie dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "sql"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for (index, path) in paths.iter().enumerate() {
        let weenie = AceWeenie::load(path).expect("weenie parses");
        let creature = dats
            .creature(&weenie, DataId::new(0xAC00_0000 + index as u32))
            .unwrap_or_else(|e| panic!("{}: {e}", weenie.name));
        let triangles: usize = creature
            .model
            .parts
            .iter()
            .flat_map(|p| &p.batches)
            .map(|b| b.indices.len() / 3)
            .sum();
        println!(
            "{:<20} parts {:>2} triangles {:>5} images {:>2} idle frames {:?}",
            weenie.name,
            creature.model.parts.len(),
            triangles,
            creature.images.len(),
            creature.idle.as_ref().map(|i| i.frame_count())
        );
        assert!(triangles > 0, "{} draws nothing", weenie.name);
        assert!(creature.images.iter().all(|(_, i)| i.is_some()));
        assert!(creature.idle.is_some(), "{} has no idle cycle", weenie.name);
    }
}
