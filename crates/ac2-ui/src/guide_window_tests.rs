//! The guide window on the real gameplay layouts (`AC2_DAT_DIR`) with the guide baked in
//! (`AC2HERO_DIR` at build time; skipped when the guide is empty): it opens and loads, lists
//! quests by level, opens a page with its pictures, shares it as a chat token that opens it again,
//! follows a resize, switches tabs, and lets its guide and pictures go when it closes.

use std::collections::HashMap;

use ac2_db::ResourceStore;

use super::*;
use crate::build_gameplay_hud;
use crate::mouse::UiMessage;

fn message(element: Option<usize>, message: u32) -> UiMessage {
    UiMessage {
        element_id: element.map_or(0, |_| 0),
        element,
        message,
        data: 0,
        item: None,
    }
}

/// Runs frames until the guide has loaded (a few seconds at most).
fn wait_loaded(window: &mut GuideWindow, tree: &mut UiTree, images: &mut HashMap<u32, UiPixels>) {
    for _ in 0..400 {
        window.frame(tree, images);
        if window.loaded.is_some() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    panic!("the guide did not load");
}

fn press(window: &mut GuideWindow, tree: &mut UiTree, at: (i32, i32)) -> Option<GuideRequest> {
    tree.start_input(crate::UI_POINTER_X, at.0 as f32);
    tree.start_input(crate::UI_POINTER_Y, at.1 as f32);
    let target = window.page.or(window.list);
    window
        .message(tree, &message(target, MSG_MOUSE_PRESS_TOP))
        .1
}

/// The screen point of list row `row`'s middle.
fn list_point(window: &GuideWindow, tree: &UiTree, row: usize) -> (i32, i32) {
    let list = window.list.unwrap();
    let rect = tree.screen_box(list);
    let line = line_height(tree, list);
    (
        rect.x0 + margin_left(tree, list) + 8,
        rect.y0 + margin_top(tree, list) + row as i32 * line + line / 2 - window.list_scroll,
    )
}

#[test]
fn the_guide_window_browses_shares_and_lets_go() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    if ac2_guide::baked().pages.is_empty() {
        return;
    }
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1920, 1080).unwrap();
    let mut window = hud.guide.take().expect("the guide window");
    let mut images = crate::load_ui_images(&store, &hud.tree).unwrap();
    let tree = &mut hud.tree;
    assert!(!window.is_shown());
    window.open(tree);
    wait_loaded(&mut window, tree, &mut images);
    assert!(images.contains_key(&GRIP_ART));

    // The quests, counted, under level bands.
    let ListRow::Text(count) = &window.list_rows[0] else {
        panic!("the count first");
    };
    assert!(count.ends_with(" quests"), "{count}");
    assert!(matches!(window.list_rows[1], ListRow::Heading(_)));

    // A search narrows the list (titles and rewards).
    let (field, _) = window.search.expect("the search field");
    tree.set_text(field, "ancient knights");
    window.frame(tree, &mut images);
    let opened: Vec<View> = window
        .list_rows
        .iter()
        .filter_map(|r| match r {
            ListRow::Open(view, _) => Some(*view),
            _ => None,
        })
        .collect();
    assert!(!opened.is_empty() && opened.len() < 40, "{}", opened.len());

    // A press on the first quest opens it.
    let row = window
        .list_rows
        .iter()
        .position(|r| matches!(r, ListRow::Open(..)))
        .unwrap();
    let point = list_point(&window, tree, row);
    assert_eq!(press(&mut window, tree, point), None);
    let Some(View::Page(page)) = window.view else {
        panic!("a page is shown: {:?}", window.view);
    };
    assert!(window.laid.rows.len() > 5);
    // Its pictures decode and reach the images.
    for _ in 0..200 {
        window.frame(tree, &mut images);
        if !window.laid.pictures.is_empty() && window.uploaded.len() == window.requested.len() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    assert!(window.uploaded.iter().all(|id| images.contains_key(id)));

    // Share gives the page's chat token, which opens the page again.
    let share = window.share.unwrap();
    let (_, request) = window.message(tree, &message(Some(share), MSG_BUTTON_RELEASE_ON));
    let Some(GuideRequest::Share(text)) = request else {
        panic!("Share asks for the chat line");
    };
    let column = text.find("[wiki:").unwrap() + 2;
    let slug = ac2_guide::browse::token_at(&text, column)
        .unwrap()
        .to_owned();
    window.show(tree, View::Page(0));
    window.open_slug(tree, &slug);
    assert_eq!(window.view, Some(View::Page(page)));
    // Back returns to the page before.
    let back = window.back.unwrap();
    window.message(tree, &message(Some(back), MSG_BUTTON_RELEASE_ON));
    assert_eq!(window.view, Some(View::Page(0)));

    // A resize lays the panes out again and the page is wrapped to the new width.
    let root = window.root();
    let before = window.laid_width;
    tree.resize_to(root, 900, 600);
    window.frame(tree, &mut images);
    assert_eq!(window.size, (900, 600));
    assert!(window.laid_width > before);
    let page_box = tree.screen_box(window.page.unwrap());
    assert!(page_box.x1 <= tree.screen_box(root).x1 - PAD + 1);

    // The Rewards tab lists items.
    let (rewards, _) = *window
        .tabs
        .iter()
        .find(|(_, t)| *t == Tab::Rewards)
        .unwrap();
    tree.set_text(field, "");
    window.message(tree, &message(Some(rewards), MSG_BUTTON_RELEASE_ON));
    window.frame(tree, &mut images);
    assert!(window
        .list_rows
        .iter()
        .any(|r| matches!(r, ListRow::Open(View::Item(_), _))));

    if let Ok(dir) = std::env::var("GUIDE_RENDER_DIR") {
        window.show(tree, View::Page(page));
        for _ in 0..100 {
            window.frame(tree, &mut images);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        render(tree, &images, root, &format!("{dir}/guide_window.png"));
        // Scrolled to the page's first picture.
        if let Some(picture) = window.laid.pictures.iter().find(|p| p.width > 64).copied() {
            let line = line_height(tree, window.page.unwrap());
            window.page_scroll = (picture.row as i32 - 3) * line;
            window.sync_page(tree);
            render(
                tree,
                &images,
                root,
                &format!("{dir}/guide_window_picture.png"),
            );
        }
    }

    // Closing lets the guide and its pictures go.
    let uploaded = window.uploaded.clone();
    window.close(tree);
    window.frame(tree, &mut images);
    assert!(window.loaded.is_none() && window.uploaded.is_empty());
    assert!(uploaded.iter().all(|id| !images.contains_key(id)));
}

/// The window composed to the PNG at `path` (for looking at, when asked for).
fn render(tree: &mut UiTree, images: &HashMap<u32, UiPixels>, root: usize, path: &str) {
    for surface in tree.compose(images) {
        if surface.root != root {
            continue;
        }
        let file = std::fs::File::create(path).unwrap();
        let mut encoder =
            png::Encoder::new(std::io::BufWriter::new(file), surface.width, surface.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&surface.rgba)
            .unwrap();
    }
}
