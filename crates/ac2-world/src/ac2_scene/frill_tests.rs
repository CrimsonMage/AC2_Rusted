//! Grass frill density as `CLandBlock::ProcScene` places it: a `SceneFile`'s frill group runs
//! through `ProcGroup` with `_bIsFrill` set, so each sub-group passes `SceneGroup::CheckFreq`,
//! `CLandBlock::CheckGroupPlacement` (bounds, `SceneGroup::CheckSlope`, `CheckFinalHeight`,
//! `ObjectCollides`) and lands in `CLandBlock::ProcFrill` without a clearance of its own. These
//! synthetic scenes pin how many frills a vertex yields, so a density change shows up here first.

use std::collections::HashMap;
use std::rc::Rc;

use super::clearance::Clearance;
use super::format::{Ac2SceneEntity, Ac2SceneFile, Ac2SceneGroup};
use super::generate::Generator;
use super::geometry::{Ac2SceneTerrain, IDENTITY, POLYGON, SIDE};
use super::Ac2SceneLookup;

const VERTEX_SCENE: u32 = 0x0200_0001;
const GRID_SCENE: u32 = 0x0200_0002;
const GRASS: u32 = 0x4700_0006;
/// The vertex the scene stands on, (80, 80) in the block.
const VERTEX: (usize, usize) = (8, 8);

struct Files(HashMap<u32, Rc<Ac2SceneFile>>);

impl Ac2SceneLookup for Files {
    type File = Rc<Ac2SceneFile>;

    fn scene_file(&self, did: u32) -> Option<Self::File> {
        self.0.get(&did).cloned()
    }
}

/// A group placing `subs` at a fixed offset (`min_loc == max_loc`), on dry land up to
/// `max_slope` (the cosine of the steepest slope), with frequency `freq`.
fn group(offset: [f64; 2], freq: f64, max_slope: f64, subs: Vec<Ac2SceneEntity>) -> Ac2SceneGroup {
    Ac2SceneGroup {
        min_loc: [offset[0], offset[1], 0.0],
        max_loc: [offset[0], offset[1], 0.0],
        freq,
        min_scale: [1.0; 3],
        max_scale: [1.0; 3],
        min_slope: 1.0,
        max_slope,
        // `ALLOW_ALL_ABOVE_MIN` from just under the ground, as the DAT's grass groups author it.
        above_dry_type: 1,
        min_above_dry: -0.0002,
        max_above_dry: 0.0002,
        below_ground_type: 2,
        min_below_ground: -0.0002,
        max_below_ground: 0.0002,
        clearance_radius: 0.1,
        subs,
        ..Ac2SceneGroup::default()
    }
}

/// The DAT's meadow grass layout (scene `0x020000B6`): a 4 x 6 grid of freq-1 groups 2 to 2.5 m
/// apart, each one grass clump, under a vertex scene whose frill group nests it.
fn meadow(freq: f64) -> Files {
    let mut grid = Vec::new();
    for dx in [-2.5, -0.5, 2.5, 5.0] {
        for dy in [-5.0, -3.0, -1.0, 1.0, 3.0, 5.0] {
            grid.push(Ac2SceneEntity::Group(Box::new(group(
                [dx, dy],
                freq,
                0.966,
                vec![Ac2SceneEntity::Object(GRASS)],
            ))));
        }
    }
    let mut frill = group(
        [0.0, 0.0],
        1.0,
        0.766,
        vec![Ac2SceneEntity::Scene(GRID_SCENE)],
    );
    frill.no_legal_check = true;
    let files = [
        (
            VERTEX_SCENE,
            Ac2SceneFile {
                frill: Some(frill),
                degrades: vec![Vec::new()],
                degrade_distances: vec![[500.0; 3]],
            },
        ),
        (
            GRID_SCENE,
            Ac2SceneFile {
                frill: None,
                degrades: vec![grid],
                degrade_distances: vec![[100.0; 3]],
            },
        ),
    ];
    Files(
        files
            .into_iter()
            .map(|(did, file)| (did, Rc::new(file)))
            .collect(),
    )
}

/// A block rising to the east at `degrees`.
fn terrain(degrees: f64) -> Ac2SceneTerrain {
    let rise = degrees.to_radians().tan() * POLYGON;
    Ac2SceneTerrain {
        block_x: 0x40,
        block_y: 0x8E,
        heights: (0..SIDE * SIDE).map(|i| (i / SIDE) as f64 * rise).collect(),
        cell_info: vec![0; SIDE * SIDE],
        road_width: 4.0,
        road_field: (0, 0),
    }
}

/// The frills `files` place at `VERTEX` over `terrain`, with `clearances` already standing.
fn frills(files: &Files, terrain: &Ac2SceneTerrain, clearances: Vec<Clearance>) -> Vec<[f64; 3]> {
    let mut generator = Generator::new(terrain, files, None, clearances);
    generator.generate(|ix, iy| if (ix, iy) == VERTEX { VERTEX_SCENE } else { 0 });
    assert!(generator.placements.iter().all(|p| p.frill));
    assert!(generator.placements.iter().all(|p| p.object == GRASS));
    generator.placements.iter().map(|p| p.origin).collect()
}

#[test]
fn a_meadow_vertex_on_level_ground_places_every_grid_clump() {
    let placed = frills(&meadow(1.0), &terrain(0.0), Vec::new());
    assert_eq!(placed.len(), 24);
    // `GenerateSceneObjectList` spaces vertices `square_length / vertex_per_cell - 0.0002` apart.
    let vertex = 8.0 * f64::from((POLYGON - 0.0002) as f32);
    for [x, y, z] in &placed {
        let (dx, dy) = (x - vertex, y - vertex);
        assert!(
            [-2.5, -0.5, 2.5, 5.0].iter().any(|v| (dx - v).abs() < 1e-3),
            "{dx}"
        );
        assert!(
            [-5.0, -3.0, -1.0, 1.0, 3.0, 5.0]
                .iter()
                .any(|v| (dy - v).abs() < 1e-3),
            "{dy}"
        );
        assert_eq!(*z, 0.0);
    }
}

#[test]
fn check_freq_thins_the_grid_by_its_frequency_and_zero_places_nothing() {
    assert!(frills(&meadow(0.0), &terrain(0.0), Vec::new()).is_empty());
    let half = frills(&meadow(0.5), &terrain(0.0), Vec::new()).len();
    assert!((4..=20).contains(&half), "{half} of 24 at freq 0.5");
}

#[test]
fn grid_grass_stops_where_the_ground_passes_its_slope_limit() {
    // The grid groups allow 15 degrees (cos 0.966); the nesting frill group allows 40.
    assert_eq!(frills(&meadow(1.0), &terrain(10.0), Vec::new()).len(), 24);
    assert!(frills(&meadow(1.0), &terrain(20.0), Vec::new()).is_empty());
}

#[test]
fn a_finished_group_clearance_keeps_grass_out_and_an_open_one_does_not() {
    // A tree group placed earlier at the vertex with a 3 m clearance (`StartSceneGroup`, then
    // `ObjectClearance::SetEnd`): `ObjectCollides` rejects every clump within 3 + 0.1 m.
    let vertex = 8.0 * f64::from((POLYGON - 0.0002) as f32);
    let mut tree = Clearance::group([vertex, vertex, 0.0], IDENTITY, 3.0, false);
    tree.group_done = true;
    let placed = frills(&meadow(1.0), &terrain(0.0), vec![tree.clone()]);
    let outside = |p: &[f64; 3]| (p[0] - vertex).hypot(p[1] - vertex) > 3.1;
    assert!(placed.iter().all(outside));
    // Clumps at (-0.5, +-1), (-0.5, +-3), (-2.5, +-1) and (2.5, +-1) are within 3.1 m of it.
    assert_eq!(placed.len(), 24 - 8);
    // `InSuperGroup`: a group still placing its children clears nothing yet.
    tree.group_done = false;
    assert_eq!(frills(&meadow(1.0), &terrain(0.0), vec![tree]).len(), 24);
}
