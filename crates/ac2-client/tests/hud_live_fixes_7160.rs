//! Live-play HUD bugs, reproduced on the classic HUD at 2560 x 1440 (1x) over `AC2_DAT_DIR` (a
//! GPU adapter). With `AC2_HUDFIX_SNAPSHOTS` set to a directory, the tests also write the HUD
//! they checked there as PNGs.
#![cfg(feature = "window")]

mod hud;

use ac2_ui::{HudStyle, UiScaleSetting};
use hud::Rig;
use legions_server_double::SWORD;

const BIG: (u32, u32) = (2560, 1440);

fn classic_big() -> Option<Rig> {
    let mut rig = Rig::new()?;
    rig.view
        .set_hud_style(&rig.gpu, ac2_gpu::COLOR_FORMAT, HudStyle::Classic);
    rig.wait("the HUD", |r| r.view.hud_tree().is_some());
    rig.view.resize(BIG.0, BIG.1);
    rig.view.set_ui_scale(UiScaleSetting::Retail);
    rig.idle(4);
    Some(rig)
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

#[test]
fn an_item_tooltip_shows_over_the_inventory() {
    let Some(mut rig) = classic_big() else {
        return;
    };
    rig.open_inventory();
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    let at = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    rig.point(at);
    rig.idle(60);
    snapshot(&mut rig, "tooltip.png");
    let tree = rig.view.hud_tree().unwrap();
    let tip = tree.tooltip_element().expect("a tooltip is shown");
    // `UISurfaceObject(1, -1.0)`: the last surface drawn, over the inventory it describes.
    let drawn = tree.placements().last().map(|p| p.root);
    assert_eq!(drawn, Some(tip), "the tooltip is the last root drawn");
}
