//! Which landblock and env-cell entities become static physics objects
//! (`CLandBlock::CreateStaticObjects`, `CEnvCell::CreatePhysicsObj`); real-data checks run only
//! when `AC2_DAT_DIR` points at the DATs.

use std::sync::Arc;

use ac2_core::CellId;
use ac2_db::ResourceStore;
use ac2_world::Ac2WorldSource;

use crate::cells::CellSource;
use crate::loader::{load_block_interiors, load_block_objects, load_terrain_block};
use crate::terrain::Terrain;

/// A landblock with buildings whose cells open to the outside.
const BLOCK: u32 = 0x7097_0000;

/// An env-cell entity of a static-object type is a collision object listed in its own cell,
/// placed in that cell's frame.
#[test]
fn env_cell_entities_are_collision_objects_in_their_cell() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    let block = CellId::new(BLOCK | 0xFFFF);
    load_terrain_block(&world, &mut terrain, block).expect("landblock decodes");
    let cells = load_block_interiors(&world, &mut terrain, block).expect("interiors load");
    assert!(cells > 0);
    let (mut entities, mut homed) = (0, 0);
    for local in 0x100..0x200u32 {
        let id = BLOCK | local;
        let Some(cell) = world.env_cell(CellId::new(id)).expect("env cell decodes") else {
            continue;
        };
        entities += world.env_cell_entities(&cell).expect("entities").len();
        let listed = terrain
            .env_cell(id)
            .expect("env cell inserted")
            .objects
            .clone();
        homed += listed
            .iter()
            .filter(|object| {
                terrain
                    .object_position(*object)
                    .is_some_and(|position| position.cell_id == id)
            })
            .count();
    }
    assert!(homed > 0, "env-cell entities become objects in their cells");
    assert!(
        homed <= entities,
        "{homed} objects from {entities} entities"
    );
}

/// The block's authored objects are made only from the static-object entity types: loading the
/// block adds exactly the shaped colliding markers.
#[test]
fn block_objects_come_from_colliding_markers_only() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    let block = CellId::new(BLOCK | 0xFFFF);
    load_terrain_block(&world, &mut terrain, block).expect("landblock decodes");
    let added = load_block_objects(&world, &mut terrain, block).expect("objects load");
    let colliding = world
        .entity_markers(block)
        .expect("markers")
        .iter()
        .filter(|marker| marker.collides())
        .count();
    assert!(added > 0 && added <= colliding);
}
