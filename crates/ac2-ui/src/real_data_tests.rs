//! The input maps against real data (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use super::*;

#[test]
fn the_default_key_maps_and_action_map_decode() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    for (enum_value, label) in [
        (KEYMAP_MAIN, "main"),
        (KEYMAP_CONTROL, "control"),
        (KEYMAP_SHIFT, "shift"),
    ] {
        let map = load_key_map(&store, enum_value)
            .unwrap()
            .unwrap_or_else(|| panic!("{label} key map"));
        eprintln!("{label} ({}): {} mappings", map.name, map.mappings.len());
        // Descrambled names read as text.
        assert!(
            map.name.chars().all(|c| c.is_ascii_graphic() || c == ' '),
            "{:?}",
            map.name
        );
        assert!(!map.mappings.is_empty());
    }
    let actions = load_action_map(&store).unwrap().expect("action map");
    eprintln!(
        "action map: {} toggles, {} actions, {} descriptions",
        actions.toggles.len(),
        actions.actions.len(),
        actions.descriptions.len()
    );
    // W moves forward: the main map binds it, the action map gives the movement class.
    let main = load_key_map(&store, KEYMAP_MAIN).unwrap().unwrap();
    let forward = main
        .mappings
        .iter()
        .find(|m| m.semantic == 0x8100_0411)
        .expect("W is bound");
    let value = actions
        .values(forward.action, 0)
        .expect("W's action has values");
    eprintln!("W -> {:08x}: {value:x?}", forward.action);
    assert_eq!(value.class, ac2_weenie::ACTION_CLASS_MOVEMENT);
}

#[test]
fn the_default_controls_walk_zoom_and_toggle_mouse_look() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let mut controls = Controls::load(&store).unwrap().expect("controls");
    let press = |c: &mut Controls, scan: u32, pressed: bool, at: f64| {
        c.event(InputEvent::Key { scan, pressed }, (at * 1000.0) as u32, at)
    };
    press(&mut controls, 0x11, true, 0.0);
    let frame = controls.frame();
    assert_eq!(frame.actions.len(), 1);
    assert_eq!(frame.actions[0].class, CLASS_MOVEMENT);
    assert_eq!(frame.actions[0].enum_value, ac2_weenie::ACTION_FORWARD);
    press(&mut controls, 0x11, false, 0.1);
    assert!(controls.frame().actions.is_empty());
    // The wheel zooms through the camera UI.
    controls.event(InputEvent::Wheel { delta: 120 }, 200, 0.2);
    assert_eq!(
        controls.frame().camera_actions,
        vec![(0x4000_002D, 1.0, true)]
    );
    // The middle button toggles mouse look through the camera UI.
    controls.event(
        InputEvent::Button {
            button: 2,
            pressed: true,
        },
        300,
        0.3,
    );
    let frame = controls.frame();
    assert_eq!(frame.mouse_look_state, Some(true));
    // The up arrow walks forward through the camera UI's re-sent action.
    press(&mut controls, 0xC8, true, 1.0);
    let frame = controls.frame();
    assert!(frame
        .actions
        .iter()
        .any(|a| a.class == CLASS_MOVEMENT && a.enum_value == ac2_weenie::ACTION_FORWARD));
}

#[test]
fn every_ui_layout_decodes_to_its_last_byte() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Local, LAYOUT_FIRST, LAYOUT_LAST)
        .collect();
    assert!(!ids.is_empty(), "the local DAT has UI layouts");
    assert_eq!(
        dataset
            .ids_in_range(ac2_dat::DatFileKind::Portal, LAYOUT_FIRST, LAYOUT_LAST)
            .count(),
        0,
        "layouts live only in the local DAT"
    );
    let store = ResourceStore::new(dataset);
    let (mut elements, mut media) = (0, 0);
    fn walk(element: &ElementDesc, elements: &mut usize, media: &mut usize) {
        *elements += 1;
        for state in &element.states {
            *media += state.media.len();
            for child in &state.children {
                walk(child, elements, media);
            }
        }
    }
    for did in &ids {
        let layout = load_layout(&store, ac2_core::DataId::new(*did))
            .unwrap_or_else(|e| panic!("{did:08x}: {e}"))
            .unwrap_or_else(|| panic!("{did:08x} missing"));
        eprintln!(
            "{did:08x}: {}x{}, {} top-level elements",
            layout.width,
            layout.height,
            layout.elements.len()
        );
        for element in &layout.elements {
            walk(element, &mut elements, &mut media);
        }
    }
    eprintln!(
        "{} layouts, {elements} elements, {media} media items",
        ids.len()
    );
}

#[test]
fn the_gameplay_hud_builds_and_places_its_roots() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store_arc = std::sync::Arc::new(ResourceStore::new(dataset));
    let store = &*store_arc;
    let root = |tree: &UiTree, name: &str| {
        *tree
            .roots()
            .iter()
            .find(|&&r| tree.get(r).unwrap().id() == tokenize(name))
            .unwrap_or_else(|| panic!("{name}"))
    };
    let tree = build_gameplay_hud(&store_arc, 1024, 768).unwrap();
    assert_eq!(tree.roots().len(), GAMEPLAY_ROOTS.len());
    // The radar sits where the layout authored it; the toolbar is centred on the bottom edge.
    let radar = root(&tree, "RadarField");
    assert_eq!(tree.screen_box(radar), UiBox::new(875, 60, 1017, 236));
    let toolbar = tree.screen_box(root(&tree, "ToolbarField"));
    assert_eq!((toolbar.y1, toolbar.width()), (767, 798));

    // Text elements read their strings: the skills panel's tabs.
    let texts: Vec<String> = (0..tree.slots_len())
        .filter_map(|i| tree.get(i).and_then(|e| e.text()))
        .collect();
    for label in ["Melee", "Missile", "Magic", "Special", "Craft"] {
        assert!(texts.iter().any(|t| t == label), "{label} in {texts:?}");
    }
    let images = load_ui_images(store, &tree).unwrap();
    assert_eq!(
        images.len(),
        tree.image_files().len(),
        "every HUD image loads"
    );
    let mut tree = tree;
    let surfaces = tree.compose(&images);
    assert_eq!(surfaces.len(), GAMEPLAY_ROOTS.len());
    let drawn = surfaces
        .iter()
        .filter(|s| s.rgba.chunks_exact(4).any(|p| p[3] != 0))
        .count();
    assert!(drawn >= 5, "{drawn} surfaces have pixels");

    // A larger display keeps the toolbar centred on the bottom edge and the side panels on the
    // bottom corners; a smaller one keeps the radar on the screen (`UIElement::MoveTo`).
    let tree = build_gameplay_hud(&store_arc, 1280, 1024).unwrap();
    let toolbar = tree.screen_box(root(&tree, "ToolbarField"));
    assert_eq!((toolbar.x0, toolbar.y1), ((1280 - 798) / 2, 1023));
    let right = tree.screen_box(root(&tree, "RightSidePanel"));
    assert_eq!((right.x1, right.y1), (1279, 1023));
    let tree = build_gameplay_hud(&store_arc, 800, 600).unwrap();
    assert_eq!(tree.screen_box(root(&tree, "RadarField")).x1, 799);
}

#[test]
fn the_wheel_zooms_over_the_world_and_not_over_the_toolbar() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut controls = Controls::load(&store).unwrap().expect("controls");
    controls.ui_attached = true;
    let world = tokenize("ItemSelectionField");
    let wheel_at = |tree: &mut UiTree, controls: &mut Controls, x: i32, y: i32| {
        controls.event(InputEvent::Cursor { x, y }, 0, 0.0);
        controls.event(InputEvent::Wheel { delta: 120 }, 10, 0.01);
        for (action, extent, start) in controls.take_ui_input() {
            if start {
                tree.start_input(action, extent);
            } else {
                tree.stop_input(action);
            }
        }
        let messages = tree.take_messages();
        controls.ui_messages(&messages, 0.02);
        controls.frame().camera_actions
    };
    // The middle of the screen is open world: the selection field is the top region.
    let zoom = wheel_at(&mut tree, &mut controls, 512, 300);
    let top = tree.mouse_top().map(|i| tree.get(i).unwrap().id());
    assert_eq!(top, Some(world));
    assert_eq!(zoom, vec![(0x4000_002D, 1.0, true)]);
    // The toolbar covers the bottom middle.
    let zoom = wheel_at(&mut tree, &mut controls, 512, 750);
    assert_ne!(
        tree.mouse_top().map(|i| tree.get(i).unwrap().id()),
        Some(world)
    );
    assert!(zoom.is_empty(), "{zoom:?}");
}
