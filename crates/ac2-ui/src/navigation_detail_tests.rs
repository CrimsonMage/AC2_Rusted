//! `gmNavigationUI`'s detail view on the real toolbar layout and the region's detail map
//! (`AC2_DAT_DIR`). The expected numbers are the disassembled formulas written out here:
//! `InitDetailMapFromCellCoords`' window (the cell minus 64, kept on the land), `SizeMap`'s
//! detail branch (`MulDiv` of the cell edges over the parent's size, with the tile adjustments
//! `(1,0) (0,0) (1,1) (0,1)`), `ConvertLandscapeToPixel`'s detail scale 250 over 128 cells, and
//! `CenterMarker`'s half sizes.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_db::ResourceStore;
use ac2_world::DetailMap;

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::navigation::{MapNote, NavigationView};
use crate::navigation_detail::{
    mul_div, tile_position, window_origin, MAP_FIELD_PRESS, MAP_PARENT_SWITCH, PAN_STEP,
    PAN_STEP_SHIFT,
};
use crate::token::tokenize;
use crate::{build_gameplay_hud, NavigationUi};

/// The tiles' images in the tests: each tile's picture is this size.
const TILE: (u32, u32) = (300, 200);
const MAP_DID: u32 = 0x0600_0001;

fn pixels(width: u32, height: u32) -> UiPixels {
    UiPixels {
        width,
        height,
        rgba: Arc::from(Vec::new()),
    }
}

/// The region's detail map, from the DAT.
fn detail_map(store: &ResourceStore) -> Arc<DetailMap> {
    let table = ac2_world::ac2_region_detail_map_table(store)
        .unwrap()
        .expect("the region's detail map");
    Arc::new(
        ac2_world::ac2_detail_map(store, table)
            .unwrap()
            .expect("the detail map"),
    )
}

fn note(id: u64, land: (i32, i32), macro_map: bool) -> MapNote {
    MapNote {
        kind: 4,
        id,
        land: Some(land),
        image: 0,
        message: String::new(),
        macro_map,
    }
}

/// The toolbar's navigation panel, shown, with the map image and the detail map set.
fn panel(store: &Arc<ResourceStore>) -> (UiTree, NavigationUi, Arc<DetailMap>) {
    let mut hud = build_gameplay_hud(store, 1024, 768).unwrap();
    let mut nav = hud.navigation.take().expect("NavigationField");
    let field = nav.field();
    hud.tree.set_visible(field, true);
    nav.set_game_map(&mut hud.tree, MAP_DID);
    let map = detail_map(store);
    nav.set_detail_map(Arc::clone(&map));
    (hud.tree, nav, map)
}

/// Every tile's picture: the DIDs of the map's rows, each `TILE` in size, and the map image.
fn images(map: &DetailMap) -> HashMap<u32, UiPixels> {
    let mut images = HashMap::from([(MAP_DID, pixels(1024, 512))]);
    for row in map.rows() {
        images.insert(row.map_did, pixels(TILE.0, TILE.1));
    }
    images
}

fn view(player: Option<(i32, i32)>, notes: Vec<MapNote>) -> NavigationView {
    NavigationView {
        player,
        notes,
        ..Default::default()
    }
}

/// The parent's size (`MapParent`): the frame the tiles and the markers are placed in.
fn parent_size(tree: &UiTree, nav: &NavigationUi) -> (i32, i32) {
    let map = nav.map_field().unwrap();
    let parent = tree.get(map).unwrap().parent_index().unwrap();
    let rect = tree.get(parent).unwrap().rect();
    (rect.width(), rect.height())
}

#[test]
fn the_window_is_the_cell_minus_half_kept_on_the_land() {
    assert_eq!(window_origin((500, 500)), (436, 436));
    // Near the edges the window slides in: the land is 2040 cells, the window 128.
    assert_eq!(window_origin((0, 0)), (0, 0));
    assert_eq!(window_origin((40, 1000)), (0, 936));
    assert_eq!(window_origin((2040, 2040)), (1912, 1912));
}

#[test]
fn mul_div_rounds_to_the_nearest_with_halves_away_from_zero() {
    assert_eq!(mul_div(5, 1, 2), 3);
    assert_eq!(mul_div(-5, 1, 2), -3);
    assert_eq!(mul_div(7, 3, 4), 5);
    assert_eq!(mul_div(21, 1, 8), 3);
    assert_eq!(mul_div(-21, 1, 8), -3);
    assert_eq!(mul_div(1, 1, 0), -1);
}

#[test]
fn the_tiles_of_a_window_sit_around_its_own_tile() {
    // A 16 x 16 block tile at blocks (96, 192): its cells are (768, 1536) to (896, 1664), which is
    // the window `window_origin` gives for the cell (832, 1600). The parent is 400 x 300.
    let window = (768, 1536);
    let parent = (400, 300);
    let row = |x0, y0| ac2_world::DetailMapRow {
        value: 0,
        map_did: 0,
        x0,
        y0,
        height: 16,
        width: 16,
        map_notes: Vec::new(),
    };
    // Tile 2 (the window's own tile, adjustment (1, 1)): its top right edge is the parent's top
    // left, so it fills the parent from (0, 0).
    assert_eq!(tile_position(2, &row(96, 192), window, parent), (0, 0));
    // Tile 3 (0, 1): the next tile to the right, its left edge at the parent's right edge.
    assert_eq!(tile_position(3, &row(112, 192), window, parent), (400, 0));
    // Tile 0 (1, 0): the tile above, its bottom edge at the parent's top.
    assert_eq!(tile_position(0, &row(96, 208), window, parent), (0, -300));
    // Tile 1 (0, 0): the tile above and to the right.
    assert_eq!(
        tile_position(1, &row(112, 208), window, parent),
        (400, -300)
    );
    // A row with no width or height sits at the origin.
    let blank = ac2_world::DetailMapRow {
        width: 0,
        ..row(96, 192)
    };
    assert_eq!(tile_position(2, &blank, window, parent), (0, 0));
}

#[test]
fn zooming_to_the_character_places_the_tiles_marker_and_notes() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut nav, map) = panel(&store);
    let images = images(&map);
    // The first row's block centre: the window's origin is its corner, eight cells per block.
    let row = map.rows()[0].clone();
    let origin = ((row.x0 * 8) as i32, (row.y0 * 8) as i32);
    let player = (origin.0 + 64, origin.1 + 64);
    nav.update(&mut tree, &view(Some(player), Vec::new()), &images);
    assert!(!nav.is_detail(), "the panel starts in the macro view");
    assert!(nav.init_detail_from_cell(&mut tree, &images, player));
    assert_eq!(nav.map_frame().origin, origin);
    assert_eq!(nav.map_frame().size, (128, 128));

    // The corners' rows: tile 2 is the window's own tile (the bottom left corner's block).
    assert_eq!(nav.tile_row(2).map(|r| r.value), Some(row.value));
    let parent = parent_size(&tree, &nav);
    let tile = |t: usize| {
        let element = nav.tile(t).unwrap();
        let rect = tree.get(element).unwrap().rect();
        (rect.x0, rect.y0, rect.width(), rect.height())
    };
    // Tile 2 at (0, 0) with its image's size; the others where `tile_position` puts them.
    assert_eq!(tile(2), (0, 0, TILE.0 as i32, TILE.1 as i32));
    if let Some(row) = nav.tile_row(3).cloned() {
        assert_eq!(
            tile(3).0,
            parent.0,
            "the tile to the right starts at the parent's width"
        );
        assert_eq!(tile(3).1, 0);
        assert_eq!(
            tile_position(3, &row, origin, parent),
            (tile(3).0, tile(3).1)
        );
    }
    for t in [0, 1] {
        if let Some(row) = nav.tile_row(t).cloned() {
            assert_eq!(
                tile_position(t, &row, origin, parent),
                (tile(t).0, tile(t).1)
            );
        }
    }

    // The player's marker at the detail pixel of the cell (64, 64) in the window: 250 * 64 / 128
    // = 125 across, and 250 - 125 = 125 down; the marker's centre is there.
    let marker = nav.player_marker().unwrap();
    let marker_rect = tree.get(marker).unwrap().rect();
    let (cw, ch) = (marker_rect.width(), marker_rect.height());
    assert_eq!(
        (marker_rect.x0, marker_rect.y0),
        (
            125 - (f64::from(cw) * 0.5) as i32,
            125 - (f64::from(ch) * 0.5) as i32
        )
    );

    // A server map note (`macro == 0`) at cell (32, 96) of the window: 250 * 32 / 128 = 62.5, and
    // 250 - 250 * 96 / 128 = 62.5, both truncated as the centre is placed.
    let server_note = note(7, (origin.0 + 32, origin.1 + 96), false);
    nav.update(&mut tree, &view(Some(player), vec![server_note]), &images);
    // The server's notes come first, then the tiles' notes.
    let element = nav.note_elements()[0];
    let note_rect = tree.get(element).unwrap().rect();
    assert!(tree.get(element).unwrap().visible());
    assert_eq!(
        (note_rect.x0, note_rect.y0),
        (
            62 - (f64::from(note_rect.width()) * 0.5) as i32,
            62 - (f64::from(note_rect.height()) * 0.5) as i32
        )
    );

    // The macro view hides the same note (`PlaceMarker`: `m_fMacroMap && !macro`).
    nav.init_macro_map(&mut tree, &images);
    nav.update(
        &mut tree,
        &view(
            Some(player),
            vec![note(7, (origin.0 + 32, origin.1 + 96), false)],
        ),
        &images,
    );
    assert!(
        !tree.get(element).unwrap().visible(),
        "the detail note is hidden in the macro view"
    );
}

#[test]
fn the_lock_follows_the_character_and_stops_the_pan() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut nav, map) = panel(&store);
    let images = images(&map);
    let row = map.rows()[0].clone();
    let origin = ((row.x0 * 8) as i32, (row.y0 * 8) as i32);
    let player = (origin.0 + 64, origin.1 + 64);
    nav.update(&mut tree, &view(Some(player), Vec::new()), &images);
    nav.init_detail_from_cell(&mut tree, &images, player);
    let scroll = nav.scroll_buttons();
    let shown = |tree: &UiTree| {
        scroll
            .iter()
            .flatten()
            .all(|b| tree.get(*b).unwrap().visible())
    };
    // Unlocked in the detail view, the scroll buttons show; the right button moves the window
    // nine cells, Shift three.
    assert!(shown(&tree));
    assert!(nav.button(&mut tree, &images, tokenize("NavigateRightButton"), false));
    assert_eq!(nav.map_frame().origin, (origin.0 + PAN_STEP, origin.1));
    assert!(nav.button(&mut tree, &images, tokenize("NavigateDownButton"), true));
    assert_eq!(
        nav.map_frame().origin,
        (origin.0 + PAN_STEP, origin.1 - PAN_STEP_SHIFT)
    );
    assert_eq!((PAN_STEP, PAN_STEP_SHIFT), (9, 3));

    // Locked: the view goes back to the character, the pan does nothing, the buttons hide.
    nav.set_lock(&mut tree, &images, true);
    assert!(nav.is_locked());
    assert_eq!(nav.map_frame().origin, origin);
    assert!(!shown(&tree));
    assert!(nav.button(&mut tree, &images, tokenize("NavigateLeftButton"), false));
    assert_eq!(nav.map_frame().origin, origin);

    // The player walks 100 cells east: the locked view follows (`OnPlayerBlockChanged`).
    let walked = (player.0 + 100, player.1);
    nav.update(&mut tree, &view(Some(walked), Vec::new()), &images);
    assert_eq!(nav.map_frame().origin, (origin.0 + 100, origin.1));

    // The lock button's tooltip names the state (`UpdateLockPositionButtonToolTip`): on while
    // locked, off unlocked, and the error text while the player is off the land.
    assert_eq!(
        nav.lock_tooltip_id(),
        tokenize("ID_Navigation_TT_LockButtonOn")
    );
    nav.update(&mut tree, &view(None, Vec::new()), &images);
    assert_eq!(
        nav.lock_tooltip_id(),
        tokenize("ID_Navigation_TT_LockButtonError")
    );
    nav.update(&mut tree, &view(Some(walked), Vec::new()), &images);

    // Unlocked again: the buttons show, and the pan works from the window.
    nav.set_lock(&mut tree, &images, false);
    assert_eq!(
        nav.lock_tooltip_id(),
        tokenize("ID_Navigation_TT_LockButtonOff")
    );
    assert!(!nav.is_locked());
    assert!(shown(&tree));
    assert!(nav.button(&mut tree, &images, tokenize("NavigateUpButton"), false));
    assert_eq!(
        nav.map_frame().origin,
        (origin.0 + 100, origin.1 + PAN_STEP)
    );
}

#[test]
fn a_click_on_the_macro_map_zooms_to_its_cell_and_the_parent_returns_to_the_macro_view() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut nav, map) = panel(&store);
    let images = images(&map);
    nav.update(&mut tree, &view(None, Vec::new()), &images);
    assert!(!nav.is_detail());
    // The macro map's click (`MapField` press with the press data): the cell under it, then the
    // detail view on the next frame.
    let pixel = (119, 119);
    let macro_frame = nav.map_frame();
    assert!(nav.map_press(
        &mut tree,
        &images,
        tokenize("MapField"),
        MAP_FIELD_PRESS,
        pixel
    ));
    nav.update(&mut tree, &view(None, Vec::new()), &images);
    let cell = macro_frame.pixel_to_landscape(pixel);
    assert!(cell.is_some(), "the click is on the map");
    assert!(nav.is_detail());
    assert_eq!(nav.map_frame().origin, window_origin(cell.unwrap()));
    // The parent's press with the switch's code returns to the macro view.
    assert!(nav.map_press(
        &mut tree,
        &images,
        tokenize("MapParentField"),
        MAP_PARENT_SWITCH,
        (0, 0)
    ));
    nav.update(&mut tree, &view(None, Vec::new()), &images);
    assert!(!nav.is_detail());
    assert_eq!(nav.map_frame().origin, (321, 195));
}
