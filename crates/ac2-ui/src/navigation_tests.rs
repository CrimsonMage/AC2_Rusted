//! `gmNavigationUI`'s macro map and `gmOverviewMapUI` on the real toolbar and gameplay layouts
//! (`AC2_DAT_DIR`). The expected numbers are the disassembled formulas written out here:
//! `ConvertLandscapeToPixel` (0x431760: `(lY - MapY) * 238 / MapH`, then `238 - pY`),
//! `CenterMarker` (0x431950: half the element's width and height, truncated), `SizeMap`'s macro
//! branch (0x4319b0), and the layout's macro attributes (`MacroXOffset` 321, `MacroYOffset` 195,
//! `MacroScale` 0.8435 on `NavigationField`).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::navigation_map::{center_offset, macro_extent, size_macro_map, MapFrame};
use crate::token::tokenize;
use crate::{build_gameplay_hud, load_layout_by_enum, MapNote, NavigationView, OverviewMapUi};

/// The region's game map as the tests give it: a DID and an image of `width` x `height`.
const MAP_DID: u32 = 0x0600_0001;

fn image(width: u32, height: u32) -> HashMap<u32, UiPixels> {
    HashMap::from([(
        MAP_DID,
        UiPixels {
            width,
            height,
            rgba: Arc::from(Vec::new()),
        },
    )])
}

/// The text of the panel's element `name` (a child of the field).
fn text_of(tree: &UiTree, field: usize, name: &str) -> Option<String> {
    let element = tree.child_recursive(field, tokenize(name))?;
    tree.get(element)?.text()
}

fn note(kind: u32, id: u64, land: Option<(i32, i32)>, macro_map: bool) -> MapNote {
    MapNote {
        kind,
        id,
        land,
        image: 0,
        message: String::new(),
        macro_map,
    }
}

/// `MacroXOffset`, `MacroYOffset`, `MacroScale` as the layout gives them, and the map extent
/// the macro branch of `InitMacroMap` computes from them: `2040 * 0.8435`, truncated.
const ORIGIN: (i32, i32) = (321, 195);
const EXTENT: i32 = 1720;

#[test]
fn the_macro_frame_is_the_layout_window_on_the_land() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let nav = hud.navigation.expect("NavigationField");
    let frame = nav.map_frame();
    assert_eq!(frame.origin, ORIGIN);
    assert_eq!(frame.size, (EXTENT, EXTENT));
    assert!(frame.macro_map);
    // 2040 cells (255 blocks of 8) times the scale, in f64, truncated.
    assert_eq!(
        macro_extent(0.8435),
        (2040.0_f64 * 0.8435_f32 as f64) as i32
    );
    assert!(nav.player_marker().is_some());
}

#[test]
fn the_player_marker_sits_at_the_pixel_retail_maps_the_cell_to() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut nav = hud.navigation.take().expect("NavigationField");
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("NavigationField")))
        .unwrap();
    tree.set_visible(field, true);
    nav.set_game_map(tree, MAP_DID);
    // A 1024 x 512 map fitted into the map parent.
    let images = image(1024, 512);
    let view = NavigationView {
        player: Some((ORIGIN.0 + EXTENT / 2, ORIGIN.1 + EXTENT / 2)),
        coordinates: Some("1.0N, 2.0E".into()),
        notes: Vec::new(),
        ..Default::default()
    };
    nav.update(tree, &view, &images);

    // The map field, sized the way `SizeMap`'s macro branch does (scale = parent width over the
    // image width, the image wider than tall, so x = 0 and y centred).
    let map = nav.map_field().unwrap();
    let parent = tree.get(map).unwrap().parent_index().unwrap();
    let pbox = tree.get(parent).unwrap().rect;
    let (pw, ph) = (pbox.width(), pbox.height());
    let scale = f64::from(pw) / 1024.0;
    let (new_w, new_h) = ((1024.0 * scale) as f32, (512.0 * scale) as f32);
    let y = ((f64::from(ph) - f64::from(new_h)) * 0.5) as f32;
    let field_box = tree.get(map).unwrap().rect;
    assert_eq!((field_box.x0, field_box.y0), (0, y as i32));
    assert_eq!(
        (field_box.width(), field_box.height()),
        (new_w as i32, new_h as i32)
    );
    assert_eq!(
        size_macro_map((pw, ph), (1024, 512)).unwrap().size,
        (new_w as i32, new_h as i32)
    );

    // The cell at the middle of the map: pixel x = (lX - MapX) * 238 / MapW = 860 * 238 / 1720
    // = 119, pixel y = 238 - (lY - MapY) * 238 / MapH = 238 - 119 = 119 (the field is
    // `new_w` x `new_h`; the marker is centred on the pixel by `CenterMarker`).
    let marker = nav.player_marker().unwrap();
    let mbox = tree.get(marker).unwrap().rect;
    let (cx, cy) = center_offset((mbox.width(), mbox.height()));
    assert_eq!(mbox.x0, 119 - cx);
    assert_eq!(mbox.y0, 119 - cy);
    assert_eq!(tree.get(marker).unwrap().parent_index(), Some(parent));
    let coordinates = tree
        .child_recursive(field, tokenize("CoordinatesText"))
        .unwrap();
    assert_eq!(
        tree.get(coordinates).unwrap().text().as_deref(),
        Some("1.0N, 2.0E")
    );
    hud.navigation = Some(nav);
}

#[test]
fn the_game_date_and_time_show_in_the_macro_view_only() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut nav = hud.navigation.take().expect("NavigationField");
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("NavigationField")))
        .unwrap();
    tree.set_visible(field, true);
    nav.set_game_map(tree, MAP_DID);
    let images = image(1024, 512);
    let view = |date: Option<&str>, time: Option<&str>| NavigationView {
        player: Some((ORIGIN.0 + EXTENT / 2, ORIGIN.1 + EXTENT / 2)),
        coordinates: Some("1.0N, 2.0E".into()),
        date: date.map(Into::into),
        time: time.map(Into::into),
        ..Default::default()
    };
    // `UpdateMacroMap`: the date and the time texts, the coordinates in `CoordinatesText`.
    nav.update(tree, &view(Some("Year 5, day 3"), Some("Dawn")), &images);
    assert_eq!(
        text_of(tree, field, "DateText").as_deref(),
        Some("Year 5, day 3")
    );
    assert_eq!(text_of(tree, field, "TimeText").as_deref(), Some("Dawn"));
    assert_eq!(
        text_of(tree, field, "CoordinatesText").as_deref(),
        Some("1.0N, 2.0E")
    );
    // The detail view has no date and time in its layout state, and the update there writes none.
    nav.init_detail_from_cell(tree, &images, (ORIGIN.0 + 64, ORIGIN.1 + 64));
    nav.update(tree, &view(Some("Year 5, day 4"), Some("Noon")), &images);
    assert_ne!(
        text_of(tree, field, "DateText").as_deref(),
        Some("Year 5, day 4")
    );
    // Back in the macro view the texts are set again; without a game time they keep them.
    nav.init_macro_map(tree, &images);
    nav.update(tree, &view(Some("Year 5, day 4"), Some("Noon")), &images);
    assert_eq!(
        text_of(tree, field, "DateText").as_deref(),
        Some("Year 5, day 4")
    );
    assert_eq!(text_of(tree, field, "TimeText").as_deref(), Some("Noon"));
    nav.update(tree, &view(None, None), &images);
    assert_eq!(
        text_of(tree, field, "DateText").as_deref(),
        Some("Year 5, day 4")
    );
    hud.navigation = Some(nav);
}

#[test]
fn the_marker_is_parked_off_the_map_indoors() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut nav = hud.navigation.take().expect("NavigationField");
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("NavigationField")))
        .unwrap();
    tree.set_visible(field, true);
    nav.update(tree, &NavigationView::default(), &HashMap::new());
    // `UpdatePlayerPos` with no land cell: x = -W, y = -H, then `CenterMarker`.
    let marker = nav.player_marker().unwrap();
    let mbox = tree.get(marker).unwrap().rect;
    let (w, h) = (mbox.width(), mbox.height());
    let (cx, cy) = center_offset((w, h));
    assert_eq!((mbox.x0, mbox.y0), (-w - cx, -h - cy));
    hud.navigation = Some(nav);
}

#[test]
fn map_notes_show_at_their_cells_in_the_macro_view_only_when_they_are_macro_notes() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut nav = hud.navigation.take().expect("NavigationField");
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("NavigationField")))
        .unwrap();
    tree.set_visible(field, true);
    nav.set_game_map(tree, MAP_DID);
    // The field is 238 x 119 here (a 1024 x 512 image in a 238 x 238 parent), so pixel y must be
    // below 119. Land (1181, 1195): pixel x = 860 * 238 / 1720 = 119, y = 238 - 1000 * 238 / 1720
    // = 99.63, stored and truncated to 99.
    let inside = (ORIGIN.0 + EXTENT / 2, ORIGIN.1 + 1000);
    let view = NavigationView {
        player: None,
        coordinates: None,
        notes: vec![
            note(4, 1, Some(inside), true),
            note(5, 2, Some(inside), false),
            note(6, 3, Some((0, 0)), true),
        ],
        ..Default::default()
    };
    let images = image(1024, 512);
    nav.update(tree, &view, &images);
    let elements = nav.note_elements();
    assert_eq!(elements.len(), 3);
    let visible: Vec<bool> = elements
        .iter()
        .map(|&e| tree.get(e).unwrap().visible)
        .collect();
    // The macro note shows; the detail-only note hides at the same cell; the note off the map hides.
    assert_eq!(visible, vec![true, false, false]);
    let mbox = tree.get(elements[0]).unwrap().rect;
    let (cx, cy) = center_offset((mbox.width(), mbox.height()));
    assert_eq!((mbox.x0, mbox.y0), (119 - cx, 99 - cy));

    // A note that leaves the frame's list leaves the map.
    nav.update(
        tree,
        &NavigationView {
            notes: vec![note(4, 1, Some(inside), true)],
            ..Default::default()
        },
        &images,
    );
    assert_eq!(nav.note_elements().len(), 1);
    hud.navigation = Some(nav);
}

#[test]
fn the_scroll_buttons_stay_hidden_in_the_macro_view() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let mut nav = hud.navigation.take().expect("NavigationField");
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("NavigationField")))
        .unwrap();
    tree.set_visible(field, true);
    nav.update(tree, &NavigationView::default(), &HashMap::new());
    for name in [
        "NavigateUpButton",
        "NavigateLeftButton",
        "NavigateRightButton",
        "NavigateDownButton",
    ] {
        let button = tree.child_recursive(field, tokenize(name)).unwrap();
        assert!(!tree.get(button).unwrap().visible, "{name}");
    }
    hud.navigation = Some(nav);
}

#[test]
fn the_overview_map_shows_one_map_and_closes_on_its_button() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let tree: &mut UiTree = &mut hud.tree;
    let overview = OverviewMapUi::create(tree, &gameplay).expect("OverviewMap");
    assert!(!overview.is_shown(tree));
    let client = overview.client_area().expect("OverviewMapClientArea");
    let label = overview.label().expect("the label with the map states");
    // The client area's first map state (a layout state other than the stateless one).
    let mode = tree
        .get(client)
        .unwrap()
        .desc
        .states
        .iter()
        .map(|s| s.id)
        .find(|&id| id != tokenize("stateless"))
        .unwrap();
    overview.show_map(tree, mode);
    assert!(overview.is_shown(tree));
    assert_eq!(tree.get(client).unwrap().state(), mode);
    assert_eq!(tree.get(label).unwrap().state(), mode);
    assert!(overview.message(tree, tokenize("OverviewMapCloseButton")));
    assert!(!overview.is_shown(tree));
    assert!(!overview.message(tree, tokenize("OverviewMapHelpButton")));
}

#[test]
fn the_knorr_button_is_in_the_layout_and_shows_its_map() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let nav = hud.navigation.take().expect("NavigationField");
    let tree: &mut UiTree = &mut hud.tree;
    let button = tokenize("NavigationZoomToOverviewKnorrButton");
    assert!(nav
        .child(tree, "NavigationZoomToOverviewKnorrButton")
        .is_some());
    assert!(nav.overview_buttons.iter().all(Option::is_some));
    let overview = OverviewMapUi::create(tree, &gameplay).expect("OverviewMap");
    assert!(overview.message(tree, button));
    assert!(overview.is_shown(tree));
    let client = overview.client_area().unwrap();
    assert_eq!(tree.get(client).unwrap().state(), tokenize("KnorrMap"));
    hud.navigation = Some(nav);
}

#[test]
fn the_pixel_maps_invert_inside_the_map_and_reject_outside() {
    let map = MapFrame {
        origin: ORIGIN,
        size: (EXTENT, EXTENT),
        macro_map: true,
    };
    // (lX - MapX) * 238 / MapW = 860 * 238 / 1720 = 119; 238 - 119 = 119 for Y.
    let (pixel, inside) = map.landscape_to_pixel((256, 256), (1181, 1055));
    assert!(inside);
    assert_eq!(pixel, [119.0, 119.0]);
    assert_eq!(map.pixel_to_landscape((119, 119)), Some((1181, 1055)));
    // Left of the map's origin: the pixel is written, but outside the field.
    let (_, inside) = map.landscape_to_pixel((256, 256), (ORIGIN.0 - 5, 1055));
    assert!(!inside);
    assert_eq!(map.pixel_to_landscape((-1, 10)), None);
    assert_eq!(map.pixel_to_landscape((5, 1720)), None);
}

#[test]
fn the_map_image_fits_the_parent_the_way_size_map_does() {
    // Wider than tall: scale by width, pinned left, centred vertically.
    let wide = size_macro_map((256, 200), (1024, 512)).unwrap();
    assert_eq!(wide.origin, (0, 36));
    assert_eq!(wide.size, (256, 128));
    // Taller than wide: scale by height, centred across, pinned top.
    let tall = size_macro_map((256, 200), (300, 400)).unwrap();
    assert_eq!(tall.origin, (53, 0));
    assert_eq!(tall.size, (150, 200));
    // Square: scale by height, centred both ways.
    let square = size_macro_map((200, 150), (100, 100)).unwrap();
    assert_eq!(square.origin, (25, 0));
    assert_eq!(square.size, (150, 150));
    assert!(size_macro_map((256, 200), (0, 0)).is_none());
}

#[test]
fn the_hovered_marker_s_tooltip_reads_into_the_note_text() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let nav = hud.navigation.as_mut().expect("NavigationField");
    let tree = &mut hud.tree;
    let images = image(256, 256);
    // Shown, with one server note in the detail view (its tooltip is the marker's text).
    nav.show(tree, true);
    let view = NavigationView {
        player: Some((5, 5)),
        notes: vec![MapNote {
            message: "Shrine: 12.0 N, 3.0 E".into(),
            macro_map: false,
            ..note(7, 1, Some((6, 5)), false)
        }],
        ..NavigationView::default()
    };
    nav.update(tree, &view, &images);
    let Some(&marker) = nav.note_elements().first() else {
        panic!("the note is placed");
    };
    // Hovering the marker: its tooltip is the text of `NoteText` (`ReadMarkerInfo`).
    tree.set_tooltip_owner(Some(marker));
    nav.update(tree, &view, &images);
    assert_eq!(
        tree.tooltip_owner(),
        Some(marker),
        "the marker is the hovered owner"
    );
    assert_eq!(
        tree.tooltip_text(marker).as_deref(),
        Some("Shrine: 12.0 N, 3.0 E")
    );
    // `NoteText` is looked up across the tree: the toolbar layout names it outside the field in
    // some builds, and then there is no text to read (ReadMarkerInfo does nothing).
    let note_text = tree
        .roots()
        .iter()
        .find_map(|&root| tree.child_recursive(root, tokenize("NoteText")));
    if let Some(note_text) = note_text {
        assert_eq!(
            tree.get(note_text).and_then(|e| e.text()).as_deref(),
            Some("Shrine: 12.0 N, 3.0 E")
        );
    }
}

#[test]
fn the_overview_buttons_show_their_map_and_show_in_the_macro_view() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let nav = hud.navigation.take().expect("NavigationField");
    // The four zoom buttons (`gmNavigationUI`'s constructor) are all in the layout.
    let buttons = nav.overview_buttons;
    assert!(buttons.iter().all(Option::is_some), "{buttons:?}");
    // `SetupElementsForMapMode`: the buttons show in the macro view only.
    let tree = &mut hud.tree;
    let first = buttons[0].unwrap();
    assert!(tree.get(first).unwrap().visible);
    let mut nav = nav;
    nav.setup_elements_for_map_mode(tree, false);
    assert!(buttons
        .iter()
        .flatten()
        .all(|&b| !tree.get(b).unwrap().visible));
    nav.setup_elements_for_map_mode(tree, true);
    assert!(buttons
        .iter()
        .flatten()
        .all(|&b| tree.get(b).unwrap().visible));

    // `gmNavigationUI::ListenToMessage` → `gmOverviewMapUI::ShowMap(Map_Omishan)`: the overview
    // shows with the label and the client area in the Omishan map's state.
    let gameplay = load_layout_by_enum(&store, 0x4100_0009).unwrap().unwrap();
    let overview = OverviewMapUi::create(tree, &gameplay).expect("OverviewMap");
    assert!(overview.message(tree, tokenize("NavigationZoomToOverviewOmishanButton")));
    assert!(overview.is_shown(tree));
    let client = overview.client_area().expect("OverviewMapClientArea");
    let label = overview.label().expect("the label");
    assert_eq!(tree.get(client).unwrap().state(), tokenize("OmishanMap"));
    assert_eq!(tree.get(label).unwrap().state(), tokenize("OmishanMap"));
    // Another button switches the map; a non-overview element is not taken.
    assert!(overview.message(tree, tokenize("NavigationZoomToOverviewArramoraButton")));
    assert_eq!(tree.get(client).unwrap().state(), tokenize("ArramoraMap"));
    assert!(!overview.message(tree, tokenize("NavigateUpButton")));
    hud.navigation = Some(nav);
}

#[test]
fn a_flush_deletes_only_its_marker_type() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut nav = hud.navigation.take().expect("NavigationField");
    let tree = &mut hud.tree;
    // A static note (the tiles' `AddMapNotes`, type 3) and a server note of another type.
    let static_note = note(3, 1, Some((10, 10)), false);
    let other_note = note(7, 2, Some((20, 20)), true);
    nav.reconcile_notes(tree, &[static_note, other_note]);
    let elements = nav.note_elements();
    assert_eq!(elements.len(), 2);
    let (static_elem, other_elem) = (elements[0], elements[1]);

    // `FlushMarkers(Static_MarkerType)` drops the static note's element and keeps the other.
    nav.flush_marker_type(tree, 3);
    assert_eq!(nav.note_elements(), vec![other_elem]);
    assert!(tree
        .get(static_elem)
        .is_none_or(|e| e.parent_index().is_none()));
    // `SetupElementsForMapMode` flushes the static type too, and keeps the other notes.
    nav.setup_elements_for_map_mode(tree, true);
    assert_eq!(nav.note_elements(), vec![other_elem]);
    hud.navigation = Some(nav);
}
