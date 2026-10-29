//! The world map in the navigation panel on the real layouts (`AC2_DAT_DIR`), without terrain:
//! the panel's elements moved for the taller map, the wheel zooming about the pointer, and the
//! layers list opened by its button, a row press turning its layer off, docked beside the panel.

use ac2_db::ResourceStore;

use super::*;
use crate::build_gameplay_hud;
use crate::mouse::GLOBAL_MOUSE_DOWN;
use crate::token::MSG_MOUSE_PRESS_TOP;

fn message(element: Option<usize>, element_id: u32, message: u32, data: i32) -> UiMessage {
    UiMessage {
        element_id,
        element,
        message,
        data,
        item: None,
    }
}

#[test]
fn the_map_grows_zooms_and_lists_its_layers() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut nav = hud.navigation.take().expect("the navigation panel");
    let mut map = hud.world_map.take().expect("the world map");
    assert!(
        map.notes.len() > 300,
        "the region's notes load: {}",
        map.notes.len()
    );
    nav.show(tree, true);
    let mut images = HashMap::new();
    map.frame(tree, &mut nav, &mut images, &WorldMapView::default());

    // The map area is taller; the date's own row is gone and the layers button is made.
    let area = nav.map_parent.unwrap();
    assert_eq!(tree.get(area).unwrap().rect().height(), MAP_HEIGHT);
    let date = nav.child(tree, "DateText").unwrap();
    assert!(!tree.get(date).unwrap().visible());
    let button = map.layers_button.expect("the layers button");
    // The image for the map area went into the HUD's images.
    assert!(images.contains_key(&MAP_IMAGE));

    // A wheel notch in over the map zooms in about the pointer (the area's middle pixel, within
    // two pixels of its exact centre).
    let rect = tree.screen_box(area);
    tree.start_input(crate::UI_POINTER_X, ((rect.x0 + rect.x1) / 2) as f32);
    tree.start_input(crate::UI_POINTER_Y, ((rect.y0 + rect.y1) / 2) as f32);
    let before = map.window();
    let wheel = message(None, 0, GLOBAL_MOUSE_DOWN, UI_WHEEL_IN as i32);
    assert!(map.message(tree, &nav, &wheel));
    map.frame(tree, &mut nav, &mut images, &WorldMapView::default());
    let after = map.window();
    assert!(after.pixels_per_cell > before.pixels_per_cell);
    assert!(
        (after.centre.0 - before.centre.0).abs() < 2.0 / f64::from(before.pixels_per_cell),
        "zoomed about the middle"
    );

    // The layers button opens the list beside the panel; a press on the towns' row turns them off.
    assert!(map.message(
        tree,
        &nav,
        &message(Some(button), 0, MSG_BUTTON_RELEASE_ON, 0)
    ));
    let sidecar = map.sidecar.as_ref().unwrap();
    assert!(sidecar.is_shown());
    map.frame(tree, &mut nav, &mut images, &WorldMapView::default());
    let sidecar = map.sidecar.as_ref().unwrap();
    let panel = tree.screen_box(nav.field());
    let list = tree.screen_box(sidecar_root(sidecar));
    assert!(
        list.x0 == panel.x1 + 1 || list.x1 == panel.x0 - 1,
        "docked: {list:?} {panel:?}"
    );
    let row = sidecar.row_of(Layer::Note(NoteKind::Town)).unwrap() as i32;
    let text = tree
        .child_recursive(sidecar_root(sidecar), tokenize("BookClientAreaText"))
        .unwrap();
    let text_box = tree.screen_box(text);
    let line = crate::book_rows::line_height(tree, text);
    tree.start_input(crate::UI_POINTER_X, (text_box.x0 + 10) as f32);
    tree.start_input(
        crate::UI_POINTER_Y,
        (text_box.y0 + row * line + line / 2) as f32,
    );
    let press = message(
        Some(text),
        tokenize("BookClientAreaText"),
        MSG_MOUSE_PRESS_TOP,
        0,
    );
    assert!(map.message(tree, &nav, &press));
    assert!(!map.layer(Layer::Note(NoteKind::Town)));
    assert!(map.layer(Layer::Note(NoteKind::Outpost)));
    assert!(!map.layer(Layer::ZoneLevels), "the zone levels start off");
}

fn sidecar_root(sidecar: &crate::world_map_sidecar::Sidecar) -> usize {
    sidecar.root()
}
