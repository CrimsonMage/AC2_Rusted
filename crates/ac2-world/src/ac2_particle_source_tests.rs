//! Real-data checks of AC2 static particle sources. They run only when `AC2_DAT_DIR`
//! points at an AC2 DAT directory, so ordinary test runs stay hermetic.

use std::sync::Arc;

use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};
use ac2_core::CellId;
use ac2_math::Vec3;
use ac2_db::ResourceStore;

use crate::{ac2_block_particle_sources, ac2_blocks_within, Ac2WorldSource};

fn ac2_world() -> Option<(Arc<DatDataset>, Arc<Ac2WorldSource>)> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(store)
        .expect("AC2 vocabulary decodes")
        .expect("AC2 dataset recognised");
    Some((dataset, world))
}

#[test]
fn grassland_campfire_block_starts_particle_systems() {
    let Some((_, world)) = ac2_world() else {
        return;
    };
    let block = CellId::new(0xC654_FFFF);
    let sources = ac2_block_particle_sources(&world, block).expect("block sources resolve");
    for source in sources.iter() {
        eprintln!(
            "{:08X} at {:?} scale {:?} particle scale {}",
            source.particle_desc.raw(),
            source.world_frame.origin,
            source.object_scale,
            source.particle_scale
        );
    }
    assert!(!sources.is_empty(), "C654 has a campfire");
    let (x, y) = (0xC6 as f32 * 160.0, 0x54 as f32 * 160.0);
    for source in sources.iter() {
        assert_eq!(source.particle_desc.raw() >> 24, 0x39);
        let origin = source.world_frame.origin;
        assert!((x..x + 160.0).contains(&origin.x) && (y..y + 160.0).contains(&origin.y));
    }
    // From the block centre the diagonal neighbours are 113 m away: 100 m reaches the four edge
    // neighbours only, 120 m all eight.
    let centre = Vec3::new(x + 80.0, y + 80.0, 0.0);
    let near = ac2_blocks_within(&world, centre, 100.0);
    assert!(near.contains(&block) && near.len() == 5, "{near:?}");
    assert_eq!(ac2_blocks_within(&world, centre, 120.0).len(), 9);
}

/// Dense survey of every landblock's startup particle sources (slow; run on demand with
/// `--ignored`). Prints the blocks with the most sources, e.g. towns full of torches.
#[test]
#[ignore]
fn survey_landblocks_with_the_most_particle_sources() {
    let Some((dataset, world)) = ac2_world() else {
        return;
    };
    let blocks = dataset
        .ids_in_range(DatFileKind::Cell, 0, u32::MAX)
        .filter(|did| did & 0xFFFF == 0xFFFE)
        .collect::<Vec<_>>();
    let mut counts = Vec::new();
    let (mut total, mut failed) = (0, 0);
    for did in &blocks {
        match ac2_block_particle_sources(&world, CellId::new(did | 0xFFFF)) {
            Ok(sources) => {
                total += sources.len();
                if !sources.is_empty() {
                    counts.push((sources.len(), *did));
                }
            }
            Err(_) => failed += 1,
        }
    }
    counts.sort_unstable_by(|a, b| b.cmp(a));
    eprintln!(
        "{} blocks, {total} particle sources, {} blocks with sources, {failed} failed",
        blocks.len(),
        counts.len()
    );
    for (count, did) in counts.iter().take(25) {
        eprintln!("{did:08X}: {count}");
    }
    if let Some((_, did)) = counts.first() {
        let sources = ac2_block_particle_sources(&world, CellId::new(did | 0xFFFF)).unwrap();
        for source in sources.iter().take(12) {
            eprintln!(
                "  {:08X} at {:?}",
                source.particle_desc.raw(),
                source.world_frame.origin
            );
        }
    }
    assert_eq!(failed, 0);
}
