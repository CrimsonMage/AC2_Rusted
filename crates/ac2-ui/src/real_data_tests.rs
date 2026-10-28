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

/// `LayoutDesc::GetSubDataIDs` reports `subIDArray` for loading. Across the local DAT every sub id
/// is a string table (`StringTable`'s `0x25000000..=0x26FFFFFE`), the layout's own string table,
/// so the sub-data adds no element to any layout.
#[test]
fn every_layout_sub_id_is_a_string_table_and_adds_no_element() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Local, LAYOUT_FIRST, LAYOUT_LAST)
        .collect();
    let store = ResourceStore::new(dataset);
    let mut sub_ids = 0usize;
    for did in &ids {
        let layout = load_layout(&store, ac2_core::DataId::new(*did))
            .unwrap()
            .unwrap();
        for sub in &layout.sub_ids {
            assert!(
                (0x2500_0000..=0x26FF_FFFE).contains(sub),
                "{did:08x}: sub id {sub:08x}"
            );
            assert_eq!(*sub, layout.string_table, "{did:08x}: its own string table");
        }
        sub_ids += layout.sub_ids.len();
    }
    eprintln!(
        "{} layouts, {sub_ids} sub ids, all string tables",
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
    let tree = build_gameplay_hud(&store_arc, 1024, 768).unwrap().tree;
    // The HUD roots, gm3DTextUI's seven reticle objects, the shortcut bar, gmChatUI's four
    // windows, the hidden fields of gmStackUI and gmExternalContainerUI, gmOverviewMapUI's
    // hidden OverviewMap, gmMiniFellowUI's hidden FellowMiniPanel, gmPetUI's hidden PetPanel and
    // gmStoryUI's hidden background and two story fields, the help window (a second `Book`
    // made as a root), and the spew box's holder (`gmChatUI::gmChatUI`'s list).
    assert_eq!(
        tree.roots().len(),
        GAMEPLAY_ROOTS.len() + 7 + 1 + 4 + 2 + 1 + 1 + 1 + 3 + 1 + 1
    );
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
    // The hidden roots are not drawn: of the chat windows only the first shows, and the spew
    // box's holder is shown.
    assert_eq!(surfaces.len(), GAMEPLAY_ROOTS.len() + 7 + 1 + 1 + 1);
    let drawn = surfaces
        .iter()
        .filter(|s| s.rgba.chunks_exact(4).any(|p| p[3] != 0))
        .count();
    assert!(drawn >= 5, "{drawn} surfaces have pixels");

    // A larger display keeps the toolbar centred on the bottom edge and the side panels on the
    // bottom corners; a smaller one keeps the radar on the screen (`UIElement::MoveTo`).
    let tree = build_gameplay_hud(&store_arc, 1280, 1024).unwrap().tree;
    let toolbar = tree.screen_box(root(&tree, "ToolbarField"));
    assert_eq!((toolbar.x0, toolbar.y1), ((1280 - 798) / 2, 1023));
    let right = tree.screen_box(root(&tree, "RightSidePanel"));
    assert_eq!((right.x1, right.y1), (1279, 1023));
    let tree = build_gameplay_hud(&store_arc, 800, 600).unwrap().tree;
    assert_eq!(tree.screen_box(root(&tree, "RadarField")).x1, 799);
}

#[test]
fn the_wheel_zooms_over_the_world_and_not_over_the_toolbar() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
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

#[test]
fn the_reticle_frames_the_selection_with_its_name_above() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let GameplayHud {
        mut tree, reticle, ..
    } = build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut reticle = reticle.expect("the gameplay layout has the reticle");
    reticle.selection_changed(&mut tree, Some(7), Some("Gnome"));
    // A box spanning -0.1..0.1 of the screen around its centre.
    let project = |p: [f32; 3]| ([p[0], p[1]], true);
    let target = |id| ReticleTarget {
        id,
        bounds: ([-0.1, -0.1, 0.0], [0.1, 0.1, 0.0]),
        project: &project,
        planar_distance: Some(10.0),
    };
    let outcome = reticle.update(&mut tree, Some(target(7)), false);
    assert!(!outcome.deselect);
    let placed: std::collections::HashMap<usize, (i32, i32, i32, bool)> = tree
        .placements()
        .into_iter()
        .map(|p| (p.root, (p.x, p.y, p.rotation, p.shown)))
        .collect();
    fn by_name(tree: &UiTree, name: &str) -> Vec<usize> {
        tree.roots()
            .iter()
            .copied()
            .filter(|&r| tree.get(r).unwrap().id() == tokenize(name))
            .collect()
    }
    // `TransformClipPointToPixel`: x 461..563, y 345 (top)..421 (bottom); 20-pixel corners
    // outside it, turned a quarter more each.
    let mut corners: Vec<_> = by_name(&tree, "TargettingReticle")
        .iter()
        .map(|r| placed[r])
        .collect();
    corners.sort_by_key(|c| c.2);
    assert_eq!(
        corners,
        vec![
            (441, 325, 0, true),
            (441, 421, 90, true),
            (563, 325, 180, true),
            (563, 421, 270, true)
        ]
    );
    let name = by_name(&tree, "SelectionName")[0];
    let (name_w, name_h) = {
        let e = tree.get(name).unwrap();
        (e.rect.width(), e.rect.height())
    };
    assert_eq!(tree.get(name).unwrap().text().as_deref(), Some("Gnome"));
    assert_eq!(
        placed[&name],
        (441 - name_w / 2 + 71, 325 - name_h, 0, true)
    );
    // Out of range clears the selection; a selection without a box hides everything.
    let far = ReticleTarget {
        planar_distance: Some(1000.0),
        ..target(7)
    };
    assert!(reticle.update(&mut tree, Some(far), false).deselect);
    reticle.update(&mut tree, None, false);
    let corners = by_name(&tree, "TargettingReticle");
    assert!(tree
        .placements()
        .iter()
        .all(|p| !corners.contains(&p.root) || !p.shown));
}

#[test]
fn the_toolbar_panels_start_hidden_and_toggle_by_side() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let GameplayHud {
        mut tree, toolbar, ..
    } = build_gameplay_hud(&store, 1024, 768).unwrap();
    let shown = |tree: &UiTree| -> Vec<ToolbarChild> {
        ToolbarChild::ALL
            .into_iter()
            .filter(|&c| toolbar.shown(tree, c))
            .collect()
    };
    assert!(shown(&tree).is_empty());
    let key = |tree: &mut UiTree, action: u32| {
        tree.start_input(action, 1.0);
        tree.stop_input(action);
        for message in tree.take_messages() {
            toolbar.message(tree, &message);
        }
    };
    key(&mut tree, ToolbarChild::Skill.key());
    assert_eq!(shown(&tree), vec![ToolbarChild::Skill]);
    // Inventory shares the right panel: skills hide. Examination is on the left.
    key(&mut tree, ToolbarChild::Inventory.key());
    key(&mut tree, ToolbarChild::Examination.key());
    assert_eq!(
        shown(&tree),
        vec![ToolbarChild::Examination, ToolbarChild::Inventory]
    );
    key(&mut tree, ToolbarChild::Inventory.key());
    assert_eq!(shown(&tree), vec![ToolbarChild::Examination]);
}

#[test]
fn the_skill_panel_tabs_make_their_pages_and_open_one_at_a_time() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    let find = |tree: &UiTree, name: &str| {
        tree.roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, tokenize(name)))
    };
    let skill_tree = find(&tree, "SkillTree").expect("the combat tab's page holds the skill tree");
    let craft = find(&tree, "CraftTab").expect("craft tab");
    let open = tokenize("open");
    let tabs: Vec<usize> = tree
        .children(tree.get(craft).unwrap().parent.unwrap())
        .iter()
        .copied()
        .filter(|&c| tree.is_panel_page(c))
        .collect();
    assert_eq!(tabs.len(), 2);
    let open_tabs = |tree: &UiTree| -> Vec<usize> {
        tabs.iter()
            .copied()
            .filter(|&t| tree.get(t).unwrap().state() == open)
            .collect()
    };
    let combat = *tabs.iter().find(|&&t| t != craft).unwrap();
    eprintln!(
        "open at start: {:?} (combat {combat}, craft {craft})",
        open_tabs(&tree)
    );
    // The skill tree's page shows exactly when its tab is open.
    let page_of = |tree: &UiTree, tab: usize| tree.get(tab).unwrap().page.unwrap();
    let combat_page = page_of(&tree, combat);
    assert_eq!(
        tree.get(combat_page).unwrap().visible,
        tree.get(combat).unwrap().state() == open
    );
    assert_eq!(tree.root_of(skill_tree), tree.root_of(combat_page));
    // Opening the craft tab closes the combat tab and swaps the pages.
    tree.set_state(craft, open);
    assert_eq!(open_tabs(&tree), vec![craft]);
    assert!(tree.get(page_of(&tree, craft)).unwrap().visible);
    assert!(!tree.get(combat_page).unwrap().visible);
    tree.set_state(combat, open);
    assert_eq!(open_tabs(&tree), vec![combat]);
    assert!(tree.get(combat_page).unwrap().visible);
}

/// The logout key: the shift key map binds Escape (DIK 0x01, semantic `0x81000400 | 0x01`) to
/// the action `gmGamePlayUI::ListenToMessage` ends the session on.
#[test]
fn the_shift_map_binds_escape_to_the_logout_action() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let shift = load_key_map(&store, KEYMAP_SHIFT).unwrap().unwrap();
    let escape = shift
        .mappings
        .iter()
        .find(|m| m.semantic == 0x8100_0401)
        .expect("Escape is bound in the shift map");
    eprintln!("shift Escape -> {:08x} ({})", escape.action, escape.name);
    assert_eq!(escape.action, LOGOUT_ACTION);
}

#[test]
fn the_chat_entry_selects_copies_cuts_and_pastes_on_the_real_layout() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    let clip = std::sync::Arc::new(MemoryClipboard::with_text("pasted"));
    tree.set_text_clipboard(clip.clone());
    // The chat windows' entry lines; the shown one takes the activation.
    let entries: Vec<usize> = (0..tree.slots_len())
        .filter(|&i| {
            tree.get(i)
                .is_some_and(|e| e.id() == tokenize("ChatEntry") && e.text.is_some())
        })
        .collect();
    let entry = entries
        .into_iter()
        .find(|&i| tree.activate(i))
        .expect("a shown chat entry line");
    assert!(tree.text_mode(), "the entry line takes typing");
    let ctrl = Modifiers {
        ctrl: true,
        ..Modifiers::default()
    };
    let plain = Modifiers::default();
    let text = |tree: &UiTree| tree.get(entry).unwrap().text().unwrap();
    for c in "hi there".chars() {
        tree.character(c as u16, 0, plain);
    }
    assert_eq!(text(&tree), "hi there");
    // Ctrl+A selects the line, Ctrl+C copies it and leaves it in place.
    tree.character(u16::from(b'a'), 0, ctrl);
    assert_eq!(tree.selection(entry), Some((0, 8)));
    tree.character(u16::from(b'c'), 0, ctrl);
    assert_eq!(clip.text().as_deref(), Some("hi there"));
    assert_eq!(text(&tree), "hi there");
    // Ctrl+X cuts it: the line empties and the clipboard keeps it.
    tree.character(u16::from(b'x'), 0, ctrl);
    assert_eq!(text(&tree), "");
    assert_eq!(clip.text().as_deref(), Some("hi there"));
    // Ctrl+V pastes at the cursor, and the field's length limit holds on the real layout.
    let max = tree
        .get(entry)
        .unwrap()
        .text
        .as_ref()
        .unwrap()
        .max_characters;
    assert!(max > 0 && max < 0xFFFF, "the entry line has a length limit");
    assert!(clip.set_text(&"x".repeat(max + 10)));
    tree.character(u16::from(b'v'), 0, ctrl);
    assert_eq!(text(&tree).chars().count(), max);
    // Shift+Home from the end selects back to the start.
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    tree.text_action(0x4000_001B, 1.0, shift);
    assert_eq!(tree.selection(entry), Some((0, max)));
}
