//! The option panel on the classic HUD at 2560 x 1440 (1x) over `AC2_DAT_DIR` (a GPU adapter),
//! driven with the pointer as a player does: each page's tab, and the drop-downs of its rows.
//! With `AC2_HUDFIX_SNAPSHOTS` set to a directory, the tests also write the panel as PNGs.
#![cfg(feature = "window")]

mod hud;

use std::path::PathBuf;
use std::sync::Arc;

use ac2_ui::{tokenize, HudStyle, ToolbarChild, UiScaleSetting, UiTree};
use hud::Rig;

const BIG: (u32, u32) = (2560, 1440);

fn temp_dir(tag: &str) -> PathBuf {
    static RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let run = RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ac2-options-{tag}-{}-{run}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The classic HUD at 2560 x 1440 with its own window positions and preferences file.
fn classic_big() -> Option<(Rig, PathBuf)> {
    let dir = temp_dir("rig");
    let mut rig = Rig::new_in(Some(dir.join("positions")))?;
    let store = Arc::new(ac2_db::ResourceStore::new(
        ac2_dat::test_support::env_dataset()?,
    ));
    let prefs_path = dir.join("Preferences.ini");
    let prefs = ac2_client::ClientPrefs::load(prefs_path.clone(), store).expect("load");
    rig.client.attach_prefs(prefs);
    rig.view
        .set_hud_style(&rig.gpu, ac2_gpu::COLOR_FORMAT, HudStyle::Classic);
    rig.wait("the HUD", |r| r.view.hud_tree().is_some());
    rig.view.resize(BIG.0, BIG.1);
    rig.view.set_ui_scale(UiScaleSetting::Retail);
    rig.idle(4);
    Some((rig, prefs_path))
}

fn snapshot(rig: &mut Rig, name: &str) {
    let Ok(out) = std::env::var("AC2_HUDFIX_SNAPSHOTS") else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    let frame = rig.client.frame();
    let aspect = BIG.0 as f32 / BIG.1 as f32;
    rig.view.settle(&rig.gpu, &frame, aspect);
    assert!(rig.view.prepare(&rig.gpu, &frame, aspect, 0.0));
    let target = ac2_gpu::FrameTarget::new(&rig.gpu.device, BIG.0, BIG.1);
    rig.view
        .render(&rig.gpu, &target.color_view, &target.depth_view, true);
    let rgba = target.read_rgba(&rig.gpu.device, &rig.gpu.queue);
    let file = std::fs::File::create(std::path::Path::new(&out).join(name)).unwrap();
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), BIG.0, BIG.1);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&rgba)
        .unwrap();
}

/// A text file beside the snapshots (`AC2_HUDFIX_SNAPSHOTS`), when that is set.
fn write_note(name: &str, text: &str) {
    if let Ok(dir) = std::env::var("AC2_HUDFIX_SNAPSHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(PathBuf::from(dir).join(name), text).unwrap();
    }
}

fn find(tree: &UiTree, name: &str) -> Option<usize> {
    let id = tokenize(name);
    (0..tree.slots_len()).find(|&i| tree.get(i).is_some_and(|e| e.id() == id))
}

fn shown(tree: &UiTree, mut index: usize) -> bool {
    loop {
        let Some(e) = tree.get(index) else {
            return false;
        };
        if !e.visible() {
            return false;
        }
        match e.parent_index() {
            Some(p) => index = p,
            None => return true,
        }
    }
}

fn center(tree: &UiTree, index: usize) -> (i32, i32) {
    let b = tree.screen_box(index);
    ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
}

fn inside(tree: &UiTree, index: usize, field: usize) -> bool {
    let f = tree.screen_box(field);
    let b = tree.screen_box(index);
    shown(tree, index)
        && b.width() > 0
        && b.height() > 0
        && b.x0 >= f.x0
        && b.x1 <= f.x1
        && b.y0 >= f.y0
        && b.y1 <= f.y1
}

/// The shown elements named `name` inside the option panel, top first.
fn in_panel(tree: &UiTree, name: &str) -> Vec<usize> {
    let id = tokenize(name);
    let field = find(tree, "OptionField").expect("the option field");
    let mut found: Vec<usize> = (0..tree.slots_len())
        .filter(|&i| tree.get(i).is_some_and(|e| e.id() == id) && inside(tree, i, field))
        .collect();
    found.sort_by_key(|&i| tree.screen_box(i).y0);
    found
}

fn press_named(rig: &mut Rig, name: &str) {
    let at = {
        let tree = rig.view.hud_tree().unwrap();
        center(tree, find(tree, name).unwrap_or_else(|| panic!("{name}")))
    };
    rig.click(at);
    rig.idle(6);
}

fn click_element(rig: &mut Rig, index: usize) {
    let at = center(rig.view.hud_tree().unwrap(), index);
    rig.click(at);
    rig.idle(6);
}

/// The panel shown on `tab`'s page.
fn open_page(rig: &mut Rig, tab: &str) {
    rig.panel(ToolbarChild::Option, true);
    rig.idle(10);
    press_named(rig, tab);
}

/// `OptionExpander::ListenToMessage`: a press on an expander's header (`TreeHeader`) lists its
/// options under it (`Construct`), on every page of the panel; the header keeps its own height
/// (`Refit`), so the next header starts below it.
#[test]
fn every_page_opens_an_expander_from_its_header() {
    let Some((mut rig, _prefs)) = classic_big() else {
        return;
    };
    for tab in ["CharacterTab", "ClientTab", "ChatTab"] {
        open_page(&mut rig, tab);
        let headers = in_panel(rig.view.hud_tree().unwrap(), "TreeHeader");
        assert!(headers.len() >= 2, "{tab} lists its expanders");
        {
            let tree = rig.view.hud_tree().unwrap();
            let (a, b) = (tree.screen_box(headers[0]), tree.screen_box(headers[1]));
            assert!(
                b.y0 > a.y1,
                "{tab}: the headers do not overlap ({a:?}, {b:?})"
            );
        }
        let next_at = rig.view.hud_tree().unwrap().screen_box(headers[1]).y0;
        click_element(&mut rig, headers[0]);
        snapshot(&mut rig, &format!("{tab}_expanded.png"));
        let tree = rig.view.hud_tree().unwrap();
        let controls = ["DropDownMenu", "CheckboxButton", "SliderBar"]
            .iter()
            .map(|n| in_panel(tree, n).len())
            .sum::<usize>();
        assert!(controls > 0, "{tab}: the first expander lists its options");
        assert!(
            tree.screen_box(headers[1]).y0 > next_at,
            "{tab}: the next expander moves down under the options"
        );
        rig.panel(ToolbarChild::Option, false);
    }
}

/// A drop-down of the client page (`OptionChoice` over `UIElement_Menu`): a press opens its list
/// over the panel (`UIElement_Menu::Open`), a press on an item picks it and closes the list
/// (`NewSelection`), the preference changes (`gmClientUI::MonitorChange`), and Apply writes it
/// to the preferences file (`gmClientUI::Apply`, `PreferenceBank::Save`).
#[test]
fn a_client_drop_down_opens_picks_applies_and_saves() {
    let Some((mut rig, prefs_path)) = classic_big() else {
        return;
    };
    open_page(&mut rig, "ClientTab");
    // The Graphics expander (the second, `SECTION_ORDER`) holds the detail drop-downs.
    let graphics = in_panel(rig.view.hud_tree().unwrap(), "TreeHeader")[1];
    click_element(&mut rig, graphics);
    let menus = in_panel(rig.view.hud_tree().unwrap(), "DropDownMenu");
    let menu = *menus.first().expect("a drop-down in the Graphics section");
    let (count, before) = {
        let tree = rig.view.hud_tree().unwrap();
        (tree.menu_items(menu).len(), tree.menu_selected_index(menu))
    };
    assert!(count >= 2, "the drop-down lists its choices");
    click_element(&mut rig, menu);
    snapshot(&mut rig, "client_drop_down_open.png");
    let pick = {
        let tree = rig.view.hud_tree().unwrap();
        assert!(tree.menu_is_open(menu), "a press opens the list");
        let popup = tree.menu_popup(menu).unwrap();
        let order: Vec<usize> = tree.placements().iter().map(|p| p.root).collect();
        let panel_root = find(tree, "OptionField")
            .map(|mut i| {
                while let Some(p) = tree.get(i).and_then(|e| e.parent_index()) {
                    i = p;
                }
                i
            })
            .unwrap();
        let at = |r| order.iter().position(|&x| x == r);
        assert!(
            at(popup) > at(panel_root),
            "the list draws over the panel ({:?} over {:?})",
            at(popup),
            at(panel_root)
        );
        let pick = (0..count).find(|&i| Some(i) != before).unwrap();
        tree.menu_item(menu, pick).unwrap()
    };
    click_element(&mut rig, pick);
    snapshot(&mut rig, "client_drop_down_picked.png");
    {
        let tree = rig.view.hud_tree().unwrap();
        assert!(!tree.menu_is_open(menu), "picking closes the list");
        assert_ne!(tree.menu_selected_index(menu), before, "the pick is shown");
    }
    let bank = rig.client.prefs().unwrap().bank();
    let changed = bank.submit(0x4000_0004);
    assert_eq!(changed.len(), 1, "the pick changed one graphics preference");
    let key = changed[0];
    let picked = bank.find(key).unwrap().value.clone();
    let apply = in_panel(rig.view.hud_tree().unwrap(), "ApplyButton")[0];
    click_element(&mut rig, apply);
    rig.idle(30);
    let bank = rig.client.prefs().unwrap().bank();
    assert!(bank.submit(0x4000_0004).is_empty(), "Apply applied it");
    assert_eq!(bank.find(key).unwrap().value, picked);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let saved = loop {
        let text = std::fs::read_to_string(&prefs_path).unwrap_or_default();
        if !text.is_empty() || std::time::Instant::now() > deadline {
            break text;
        }
        rig.idle(2);
    };
    assert!(!saved.is_empty(), "Apply saved the preferences file");
}

/// Whether `index` is a descendant of `page`.
fn under(tree: &UiTree, index: usize, page: usize) -> bool {
    let mut at = index;
    while let Some(p) = tree.get(at).and_then(|e| e.parent_index()) {
        if p == page {
            return true;
        }
        at = p;
    }
    false
}

/// The elements named `name` on `page` (listed, shown or scrolled out of view), top first.
fn on_page(tree: &UiTree, page: usize, name: &str) -> Vec<usize> {
    let id = tokenize(name);
    let mut found: Vec<usize> = (0..tree.slots_len())
        .filter(|&i| tree.get(i).is_some_and(|e| e.id() == id) && under(tree, i, page))
        .collect();
    found.sort_by_key(|&i| tree.screen_box(i).y0);
    found
}

/// Opens every expander of `page` (top first, nested ones too), scrolling each header into view.
fn open_every_expander(rig: &mut Rig, page: usize) {
    let mut opened = std::collections::HashSet::new();
    loop {
        let next = on_page(rig.view.hud_tree().unwrap(), page, "TreeHeader")
            .into_iter()
            .find(|h| !opened.contains(h));
        let Some(header) = next else {
            return;
        };
        opened.insert(header);
        assert!(scroll_into_view(rig, header), "a header scrolls into view");
        click_element(rig, header);
    }
}

/// The pages' scrolled lists (`gmCharacterUI`'s `CharacterList`, `gmClientUI`'s `ClientList`,
/// `gmChatOptionsUI`'s `ChatOptionsList`).
const PAGE_LISTS: [&str; 3] = ["CharacterList", "ClientList", "ChatOptionsList"];

/// Scrolls the shown page's list so `index` lies inside it (the wheel over the list, as a player
/// does); false when it never comes into view.
fn scroll_into_view(rig: &mut Rig, index: usize) -> bool {
    for _ in 0..200 {
        let tree = rig.view.hud_tree().unwrap();
        let list = PAGE_LISTS
            .iter()
            .filter_map(|n| find(tree, n))
            .find(|&l| shown(tree, l))
            .expect("a page's list is shown");
        if inside(tree, index, list) {
            return true;
        }
        let list_box = tree.screen_box(list);
        let below = tree.screen_box(index).y0 > list_box.y1 - 30;
        let at = (list_box.x0 + 60, (list_box.y0 + list_box.y1) / 2);
        rig.point(at);
        let wheel = if below {
            ac2_ui::UI_WHEEL_OUT
        } else {
            ac2_ui::UI_WHEEL_IN
        };
        rig.frame(&[(wheel, 1.0, true)]);
        rig.frame(&[(wheel, 1.0, false)]);
    }
    false
}

/// Every drop-down on every page (`OptionChoice`): it opens its list with a press, a press on
/// another item picks it and closes the list, and the menu shows the pick.
#[test]
fn every_drop_down_on_every_page_opens_and_picks() {
    let Some((mut rig, _prefs)) = classic_big() else {
        return;
    };
    let mut report = String::new();
    for (tab, page_name) in [
        ("CharacterTab", "CharacterPage"),
        ("ClientTab", "ClientPage"),
        ("ChatTab", "ChatPage"),
    ] {
        open_page(&mut rig, tab);
        let page = find(rig.view.hud_tree().unwrap(), page_name).unwrap();
        open_every_expander(&mut rig, page);
        let all = on_page(rig.view.hud_tree().unwrap(), page, "DropDownMenu");
        report.push_str(&format!("{tab}: {} drop-downs\n", all.len()));
        assert!(!all.is_empty(), "{tab} has drop-downs");
        for (n, &menu) in all.iter().enumerate() {
            assert!(
                scroll_into_view(&mut rig, menu),
                "{tab} drop-down {n} scrolls into view"
            );
            let (count, before) = {
                let tree = rig.view.hud_tree().unwrap();
                (tree.menu_items(menu).len(), tree.menu_selected_index(menu))
            };
            let menu_box = rig.view.hud_tree().unwrap().screen_box(menu);
            click_element(&mut rig, menu);
            let tree = rig.view.hud_tree().unwrap();
            if !tree.menu_is_open(menu) {
                report.push_str(&format!(
                    "  {n}: did not open at {menu_box:?}, state {:08x}, top {:?}\n",
                    tree.get(menu).unwrap().state(),
                    tree.mouse_top().map(|t| (t, tree.get(t).unwrap().id()))
                ));
                write_note("drop_downs.txt", &report);
                snapshot(&mut rig, &format!("{tab}_drop_down_{n}_failed.png"));
            }
            let tree = rig.view.hud_tree().unwrap();
            assert!(tree.menu_is_open(menu), "{tab} drop-down {n} opens");
            if n == 0 {
                snapshot(&mut rig, &format!("{tab}_drop_down.png"));
            }
            let tree = rig.view.hud_tree().unwrap();
            let pick = (0..count).find(|&i| Some(i) != before);
            report.push_str(&format!("  {n}: {count} items, {before:?} -> {pick:?}\n"));
            let Some(pick) = pick else {
                rig.click((1500, 300));
                continue;
            };
            let item = tree.menu_item(menu, pick).unwrap();
            let popup = tree.menu_popup(menu).unwrap();
            let (item_box, popup_box) = (tree.screen_box(item), tree.screen_box(popup));
            assert!(
                item_box.y0 >= popup_box.y0 && item_box.y1 <= popup_box.y1,
                "{tab} drop-down {n}: item {pick} shows in the list ({item_box:?} in {popup_box:?})"
            );
            click_element(&mut rig, item);
            let tree = rig.view.hud_tree().unwrap();
            assert!(
                !tree.menu_is_open(menu),
                "{tab} drop-down {n} closes on a pick"
            );
            assert_eq!(
                tree.menu_selected_index(menu),
                Some(pick),
                "{tab} drop-down {n} shows the pick"
            );
        }
        write_note("drop_downs.txt", &report);
    }
}
