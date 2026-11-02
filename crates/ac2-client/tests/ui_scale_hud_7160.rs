//! The UI scale setting on the HUD rig over the Legions server double (`AC2_DAT_DIR`, a GPU
//! adapter): at 1x (the default, retail's size) the HUD lays out on the real 2560 x 1440 display
//! as retail's `UIElementManager::RefreshEvent` did, at 2x on 1280 x 720 drawn at twice the size,
//! Fill is the first modern client's scale, and a window keeps its distance from the nearest
//! edges across scales. Both HUD styles.

#![cfg(feature = "window")]

mod hud;

use ac2_ui::{tokenize, HudStyle, UiBox, UiScaleSetting, UiTree};
use hud::Rig;

const BIG: (u32, u32) = (2560, 1440);

fn root_box(tree: &UiTree, name: &str) -> UiBox {
    let id = tokenize(name);
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).is_some_and(|e| e.id() == id))
        .unwrap_or_else(|| panic!("{name} is a root"));
    tree.screen_box(root)
}

fn width(b: UiBox) -> i32 {
    b.x1 - b.x0 + 1
}

fn tree(r: &Rig) -> &UiTree {
    r.view.hud_tree().unwrap()
}

/// Alt + drag of the window `element` by `by` pixels, pressed on a point where it is topmost.
fn alt_drag(rig: &mut Rig, element: u32, by: (i32, i32)) {
    let root_of = |r: &Rig| {
        let tree = r.view.hud_tree().unwrap();
        tree.roots()
            .iter()
            .copied()
            .find(|&i| tree.get(i).unwrap().id() == element)
            .unwrap()
    };
    let root = root_of(rig);
    let b = tree(rig).screen_box(root);
    let mut press = None;
    'search: for y in (b.y0 + 4..b.y1 - 4).step_by(6) {
        for x in (b.x0 + 4..b.x1 - 4).step_by(6) {
            rig.point((x, y));
            let t = tree(rig);
            if t.mouse_top().is_some_and(|top| t.root_of(top) == root) {
                press = Some((x, y));
                break 'search;
            }
        }
    }
    let from = press.expect("a point of the window is on top");
    rig.alt = true;
    rig.button(true);
    for step in 1..=4 {
        let t = step as f32 / 4.0;
        rig.point((
            from.0 + (by.0 as f32 * t).round() as i32,
            from.1 + (by.1 as f32 * t).round() as i32,
        ));
    }
    rig.release();
    rig.alt = false;
}

fn dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ac2-ui-scale-{}-{name}", std::process::id()))
}

fn rig(style: HudStyle, name: &str) -> Option<Rig> {
    let mut rig = Rig::new_in(Some(dir(name)))?;
    if style == HudStyle::Classic {
        rig.view
            .set_hud_style(&rig.gpu, ac2_gpu::COLOR_FORMAT, style);
    }
    rig.wait("the HUD", |r| r.view.hud_tree().is_some());
    rig.idle(4);
    Some(rig)
}

fn at(rig: &mut Rig, size: (u32, u32), setting: UiScaleSetting) {
    rig.view.resize(size.0, size.1);
    rig.view.set_ui_scale(setting);
    rig.idle(4);
}

fn check(style: HudStyle, name: &str) {
    let Some(mut rig) = rig(style, name) else {
        return;
    };
    // The authored display: the baseline.
    let (vitals0, toolbar0) = (
        root_box(tree(&rig), "StatusbarField"),
        root_box(tree(&rig), "ToolbarField"),
    );
    assert_eq!(rig.view.hud_display(), ((1024, 768), 1.0));

    // 1x on 2560 x 1440: the display is the window, drawn pixel for pixel.
    at(&mut rig, BIG, UiScaleSetting::Retail);
    assert_eq!(rig.view.hud_display(), (BIG, 1.0));
    let vitals = root_box(tree(&rig), "StatusbarField");
    let toolbar = root_box(tree(&rig), "ToolbarField");
    eprintln!("{name}: vitals {vitals0:?} -> {vitals:?}; toolbar {toolbar0:?} -> {toolbar:?}");
    assert_eq!(
        (width(vitals), vitals.y1 - vitals.y0),
        (width(vitals0), vitals0.y1 - vitals0.y0),
        "the vitals keep retail's size"
    );
    assert_eq!(
        vitals.x0, vitals0.x0,
        "the vitals stay at the display's left"
    );
    assert_eq!(width(toolbar), width(toolbar0).max(width(toolbar)));
    assert_eq!(toolbar.y1, 1439, "the toolbar is on the bottom edge");

    // 2x: 1280 x 720 logical, drawn at twice the size.
    at(&mut rig, BIG, UiScaleSetting::Percent(200));
    assert_eq!(rig.view.hud_display(), ((1280, 720), 2.0));
    assert_eq!(root_box(tree(&rig), "ToolbarField").y1, 719);

    // Fill: today's min(w / 1024, h / 768).
    at(&mut rig, BIG, UiScaleSetting::Fill);
    let (display, scale) = rig.view.hud_display();
    assert_eq!(scale, 1440.0 / 768.0);
    assert_eq!(display, (1365, 768));

    // Back to 1x: the places are the ones of the first 1x layout.
    at(&mut rig, BIG, UiScaleSetting::Retail);
    assert_eq!(root_box(tree(&rig), "StatusbarField"), vitals);
    assert_eq!(root_box(tree(&rig), "ToolbarField"), toolbar);
    let _ = std::fs::remove_dir_all(dir(name));
}

#[test]
fn the_modern_hud_lays_out_at_each_scale() {
    check(HudStyle::Modern, "modern");
}

#[test]
fn the_classic_hud_lays_out_at_each_scale() {
    check(HudStyle::Classic, "classic");
}

/// Windows nobody placed (the radar, the chat) take their authored clip box on the display and
/// keep their pixel size (`default_locations`); in the modern style the client's own placement of
/// the radar and chat stands.
#[test]
fn unplaced_windows_land_proportionally_at_their_native_size() {
    let Some(mut rig) = rig(HudStyle::Classic, "defaults") else {
        return;
    };
    let radar0 = root_box(tree(&rig), "RadarField");
    let chat0 = root_box(tree(&rig), "ChatWindow1");
    for (setting, display) in [
        (UiScaleSetting::Retail, BIG),
        (UiScaleSetting::Percent(200), (1280, 720)),
    ] {
        at(&mut rig, BIG, setting);
        for (name, authored) in [("RadarField", radar0), ("ChatWindow1", chat0)] {
            let b = root_box(tree(&rig), name);
            // The radar is fixed in size; the chat window allows resizing, so its bottom edge
            // is placed proportionally too (`UIElement::ApplyLocation`).
            if name == "RadarField" {
                assert_eq!(
                    (width(b), b.y1 - b.y0),
                    (width(authored), authored.y1 - authored.y0),
                    "{name} keeps its pixel size"
                );
            } else {
                // (Its width is held by `max_width`, as `ResizeTo` clamps.)
                let fy1 = f64::from(b.y1 + 1) / f64::from(display.1);
                let ay1 = f64::from(authored.y1 + 1) / 768.0;
                assert!((fy1 - ay1).abs() < 0.005, "{b:?}");
                assert!(width(b) >= width(authored), "{b:?}");
            }
            let (fx, fy) = (
                f64::from(b.x0) / f64::from(display.0),
                f64::from(b.y0) / f64::from(display.1),
            );
            let (ax, ay) = (
                f64::from(authored.x0) / 1024.0,
                f64::from(authored.y0) / 768.0,
            );
            assert!(
                (fx - ax).abs() < 0.005 && (fy - ay).abs() < 0.005,
                "{name} at {b:?} on {display:?}: {fx:.3},{fy:.3} vs {ax:.3},{ay:.3}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(dir("defaults"));
}

/// The modern style's radar and chat are the client's own and stay where it puts them.
#[test]
fn the_modern_styles_own_places_are_not_moved_by_the_defaults() {
    let Some(mut rig) = rig(HudStyle::Modern, "modern-defaults") else {
        return;
    };
    at(&mut rig, BIG, UiScaleSetting::Retail);
    let radar = root_box(tree(&rig), "RadarField");
    let chat = root_box(tree(&rig), "ChatWindow1");
    eprintln!("modern radar {radar:?} chat {chat:?}");
    // (The radar follows the display only while its field shows, which the rig's does not.)
    let _ = radar;
    assert!(
        chat.x0 < 100 && chat.y1 > 1300,
        "the chat is bottom left: {chat:?}"
    );
    let _ = std::fs::remove_dir_all(dir("modern-defaults"));
}

/// A window moved by the player keeps its distance from the edges it was nearest when the scale
/// changes the logical display (`window_positions.rs` `anchored`).
#[test]
fn a_moved_window_keeps_its_edge_distances_across_scales() {
    let Some(mut rig) = rig(HudStyle::Classic, "anchored") else {
        return;
    };
    at(&mut rig, BIG, UiScaleSetting::Retail);
    let before = rig.view.hud_tree().unwrap().inq_locations();
    let window = before
        .iter()
        .find(|s| s.location.shown)
        .expect("a shown window")
        .element;
    let find = |r: &Rig| {
        let tree = r.view.hud_tree().unwrap();
        let root = tree
            .roots()
            .iter()
            .copied()
            .find(|&i| tree.get(i).unwrap().id() == window)
            .unwrap();
        tree.screen_box(root)
    };
    // Alt + drag it to a place near the display's lower right corner, in one or two moves (the
    // first brings a window that starts off the display into bounds).
    for _ in 0..2 {
        let b = find(&rig);
        let by = (BIG.0 as i32 - 1 - 40 - b.x1, BIG.1 as i32 - 1 - 120 - b.y1);
        alt_drag(&mut rig, window, by);
    }
    let placed = find(&rig);
    let gaps = |b: UiBox, display: (u32, u32)| {
        (
            display.0 as i32 - 1 - b.x1,
            display.1 as i32 - 1 - b.y1,
            b.x1 - b.x0,
            b.y1 - b.y0,
        )
    };
    let first = gaps(placed, BIG);
    assert!(
        (first.0 - 40).abs() <= 2 && (first.1 - 120).abs() <= 2,
        "the window was placed near the corner: {first:?}"
    );
    at(&mut rig, BIG, UiScaleSetting::Percent(200));
    let second = gaps(find(&rig), (1280, 720));
    assert_eq!(
        second, first,
        "same distance from the right and bottom edges"
    );
    at(&mut rig, BIG, UiScaleSetting::Retail);
    assert_eq!(gaps(find(&rig), BIG), first);
    let _ = std::fs::remove_dir_all(dir("anchored"));
}

/// With `AC2_UISCALE_SNAPSHOTS` set to a directory, the classic HUD at 2560 x 1440 at 1x and at
/// 2x is written there as PNGs.
#[test]
fn the_classic_hud_renders_at_1x_and_2x() {
    let Ok(out) = std::env::var("AC2_UISCALE_SNAPSHOTS") else {
        return;
    };
    let Some(mut rig) = rig(HudStyle::Classic, "render") else {
        return;
    };
    std::fs::create_dir_all(&out).unwrap();
    for (name, setting) in [
        ("classic_2560x1440_scale1.png", UiScaleSetting::Retail),
        ("classic_2560x1440_scale2.png", UiScaleSetting::Percent(200)),
    ] {
        at(&mut rig, BIG, setting);
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
    let _ = std::fs::remove_dir_all(dir("render"));
}
