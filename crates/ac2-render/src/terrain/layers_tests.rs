use ac2_core::DataId;
use ac2_world::Ac2SurfaceInfo;

use super::{terrain_layers, AlphaRotation, TerrainLayer};

fn surface(material: u32) -> Ac2SurfaceInfo {
    Ac2SurfaceInfo {
        base_material: Some(DataId::new(0x2B00_0000 | material)),
        base_tiling: 20.0,
        corner_alpha: Some(DataId::new(0x4000_0100 | material)),
        side_alpha: Some(DataId::new(0x4000_0200 | material)),
        three_corner_alpha: Some(DataId::new(0x4000_0300 | material)),
        diagonal_alpha: Some(DataId::new(0x4000_0400 | material)),
        ..Ac2SurfaceInfo::default()
    }
}

fn layers(ids: [u32; 4], roads: [Option<u32>; 4]) -> Vec<(u32, Option<(u32, AlphaRotation)>)> {
    let corners = ids.map(surface);
    let roads = roads.map(|road| road.map_or_else(Ac2SurfaceInfo::default, surface));
    terrain_layers(
        [&corners[0], &corners[1], &corners[2], &corners[3]],
        [&roads[0], &roads[1], &roads[2], &roads[3]],
    )
    .expect("layers")
    .0
    .iter()
    .map(|layer: &TerrainLayer| {
        (
            layer.material.raw() & 0xFF,
            layer.alpha.map(|(did, rot)| (did.raw() & 0xFFFF, rot)),
        )
    })
    .collect()
}

const NO_ROAD: [Option<u32>; 4] = [None; 4];

#[test]
fn uniform_polygon_draws_one_opaque_layer() {
    assert_eq!(layers([1, 1, 1, 1], NO_ROAD), vec![(1, None)]);
}

#[test]
fn a_lone_corner_overlays_its_own_corner_mask_turned_onto_it() {
    use AlphaRotation::*;
    // Corner order SW, SE, NW, NE.
    assert_eq!(
        layers([1, 1, 2, 1], NO_ROAD),
        vec![(1, None), (2, Some((0x0102, Rot0)))]
    );
    assert_eq!(
        layers([1, 1, 1, 2], NO_ROAD),
        vec![(1, None), (2, Some((0x0102, Rot90)))]
    );
    assert_eq!(
        layers([1, 2, 1, 1], NO_ROAD),
        vec![(1, None), (2, Some((0x0102, Rot180)))]
    );
    assert_eq!(
        layers([2, 1, 1, 1], NO_ROAD),
        vec![(1, None), (2, Some((0x0102, Rot270)))]
    );
}

#[test]
fn halves_blend_with_the_side_mask_of_the_ne_corner() {
    use AlphaRotation::*;
    // South half 1, north half 2: north is the base, south overlays with NE's side mask.
    assert_eq!(
        layers([1, 1, 2, 2], NO_ROAD),
        vec![(2, None), (1, Some((0x0202, Rot180)))]
    );
    // West half 1, east half 2.
    assert_eq!(
        layers([1, 2, 1, 2], NO_ROAD),
        vec![(1, None), (2, Some((0x0202, Rot90)))]
    );
}

#[test]
fn four_different_corners_keep_three_layers() {
    use AlphaRotation::*;
    assert_eq!(
        layers([1, 2, 3, 4], NO_ROAD),
        vec![
            (1, None),
            (3, Some((0x0103, Rot0))),
            (4, Some((0x0104, Rot90)))
        ]
    );
}

#[test]
fn roads_cover_or_strip_the_polygon() {
    use AlphaRotation::*;
    let all = [Some(9); 4];
    assert_eq!(layers([1, 1, 1, 1], all), vec![(9, None)]);
    // A road along the north edge: NW road with its side mask, unrotated.
    assert_eq!(
        layers([1, 1, 1, 1], [None, None, Some(9), Some(9)]),
        vec![(1, None), (9, Some((0x0209, Rot0)))]
    );
    // Only the SE corner touches a road.
    assert_eq!(
        layers([1, 1, 1, 1], [None, Some(9), None, None]),
        vec![(1, None), (9, Some((0x0109, Rot180)))]
    );
}

#[test]
fn rotations_put_the_mask_origin_on_each_corner() {
    // The mask's (0, 0) texel lands on NW, NE, SE, SW for ROT_0..ROT_270.
    let origin = |rot: AlphaRotation| (0..4).find(|i| rot.corner_uv(*i) == [0.0, 0.0]).unwrap();
    assert_eq!(origin(AlphaRotation::Rot0), 2);
    assert_eq!(origin(AlphaRotation::Rot90), 3);
    assert_eq!(origin(AlphaRotation::Rot180), 1);
    assert_eq!(origin(AlphaRotation::Rot270), 0);
}
