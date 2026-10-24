//! Real-data checks of interiors; they run only when `AC2_DAT_DIR` points at the DATs.
//! Expectations are invariants (nothing recorded from the data).

use std::sync::Arc;

use ac2_core::{CellId, DataId};
use ac2_dat::{DatDataset, DatOpenOptions};
use ac2_db::ResourceStore;
use ac2_world::{load_ac2_motion_interp_desc, Ac2WorldSource};

use crate::body::{PhysicsBody, PHYSICS_QUANTUM};
use crate::cells::CellSource;
use crate::frame::Frame;
use crate::landdefs::is_outside;
use crate::loader::{
    load_block_interiors, load_block_objects, load_body_shape, load_terrain_block,
};
use crate::physics_obj_place::{SetPositionError, SLIDE_SPF};
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::Quaternion;

const HUMAN_MOTION_INTERP_DESC: u32 = 0x3200_0000;
const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
/// A landblock with buildings whose cells open to the outside.
const BLOCK: u32 = 0x7097_0000;

#[test]
fn interiors_load_connect_and_hold_a_body_on_their_floors() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = Ac2WorldSource::open(Arc::clone(&store))
        .expect("world source opens")
        .expect("AC2 landscape");
    let mut terrain = Terrain::new();
    let (bx, by) = (BLOCK >> 24, (BLOCK >> 16) & 0xFF);
    for dx in -1..=1_i32 {
        for dy in -1..=1_i32 {
            let block = ((bx as i32 + dx) as u32) << 24 | ((by as i32 + dy) as u32) << 16;
            load_terrain_block(&world, &mut terrain, CellId::new(block | 0xFFFF))
                .expect("landblock decodes");
        }
    }
    let block = CellId::new(BLOCK | 0xFFFF);
    let interiors = load_block_interiors(&world, &mut terrain, block).expect("interiors load");
    load_block_objects(&world, &mut terrain, block).expect("objects load");
    assert!(interiors > 0);
    let touching = (1..=64)
        .filter_map(|local| terrain.land_cell(BLOCK | local))
        .filter(|cell| !cell.touching.is_empty())
        .count();
    assert!(touching > 0, "outside-reaching cells touch land cells");

    let desc = load_ac2_motion_interp_desc(&store, DataId::new(HUMAN_MOTION_INTERP_DESC))
        .expect("motion interp desc decodes")
        .expect("human motion interp desc");
    let desc: Arc<_> = Arc::new((*desc).clone());
    let shape = load_body_shape(&store, DataId::new(HUMAN_MALE_VISUAL))
        .expect("body setup decodes")
        .expect("human body setup");
    let (mut settled, mut tried) = (0, 0);
    for index in 0..interiors as u32 {
        let cell_id = BLOCK | (0x0100 + index);
        let Some(cell) = terrain.env_cell(cell_id) else {
            continue;
        };
        let Some(sphere) = cell
            .structure
            .physics
            .as_ref()
            .and_then(|t| t.bounding_sphere())
        else {
            continue;
        };
        // Start at the middle of the cell's physics tree, if that point is inside the cell.
        let start = cell.position.local_to_global(&cell.position, sphere.center);
        if !cell.point_in_cell(start) {
            continue;
        }
        let position = Position::new(cell_id, Frame::new(start, Quaternion::IDENTITY));
        let mut body = PhysicsBody::new(position, shape.clone(), Some(Arc::clone(&desc)));
        // A start inside a solid leaf of the cell's physics tree cannot be placed (retail
        // `BSPTREE::placement_insert` keeps adjusting by nothing); the rest must place.
        tried += 1;
        if body.place(position, SLIDE_SPF, &terrain) != SetPositionError::Ok {
            continue;
        }
        assert!(
            !is_outside(body.position().cell_id),
            "cell {cell_id:08x}: placed outdoors"
        );
        for _ in 0..(4.0 / PHYSICS_QUANTUM) as usize {
            body.step_quantum(&terrain);
        }
        let state = body.movement_state();
        assert!(
            !is_outside(state.cell_id),
            "cell {cell_id:08x}: fell out to {state:?}"
        );
        assert!(state.contact, "cell {cell_id:08x}: {state:?}");
        assert!(
            state.origin[2] <= start.z + 0.01,
            "cell {cell_id:08x}: {state:?}"
        );
        settled += 1;
    }
    assert!(
        settled * 10 >= tried * 9,
        "{settled} of {tried} cells placed"
    );
}
