//! Live-play HUD bugs, reproduced on the classic HUD at 2560 x 1440 (1x) over `AC2_DAT_DIR` (a
//! GPU adapter). With `AC2_HUDFIX_SNAPSHOTS` set to a directory, the tests also write the HUD
//! they checked there as PNGs.
#![cfg(feature = "window")]

mod hud;

use ac2_ui::{HudStyle, UiScaleSetting};
use hud::Rig;
use legions_server_double::SWORD;

const BIG: (u32, u32) = (2560, 1440);

/// The rig with its window positions in a fresh directory of its own: a resized inventory is
/// saved there, and the player's AC2 directory (or an earlier run) must not size the next one.
fn classic_big() -> Option<Rig> {
    static RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let run = RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ac2-hudfix-{}-{run}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut rig = Rig::new_in(Some(dir))?;
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

/// The inventory's top grip pulled up by `dy` pixels with the pointer, the way a player does.
fn pull_inventory_up(rig: &mut Rig, dy: i32) {
    let tree = rig.view.hud_tree().unwrap();
    let id = ac2_ui::tokenize("InventoryResizeBar");
    let grip = (0..tree.slots_len())
        .find(|&i| tree.get(i).is_some_and(|e| e.id() == id))
        .expect("the inventory's grip");
    let b = tree.screen_box(grip);
    let from = ((b.x0 + b.x1) / 2, b.y0 + 2);
    rig.drag(from, (from.0, from.1 - dy));
}

fn right_panel(rig: &Rig) -> ac2_ui::UiBox {
    let tree = rig.view.hud_tree().unwrap();
    let id = ac2_ui::tokenize("RightSidePanel");
    let root = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == id)
        .unwrap();
    tree.screen_box(root)
}

#[test]
fn the_classic_inventory_resizes_taller() {
    let Some(mut rig) = classic_big() else {
        return;
    };
    rig.open_inventory();
    rig.idle(8);
    let before = right_panel(&rig);
    pull_inventory_up(&mut rig, 150);
    rig.idle(4);
    let after = right_panel(&rig);
    snapshot(&mut rig, "inventory_tall.png");
    // 150 pixels is four rows (144) and a bit: the window grows by whole pack rows, upward, and
    // the list shows them.
    assert_eq!(after.height(), before.height() + 4 * 36);
    assert_eq!(
        after.y1, before.y1,
        "the bottom edge stays on the display's"
    );
}

fn shortcut(rig: &mut Rig, index: usize) -> ac2_net::protocol::ShortcutInfo {
    rig.client.shortcut_array().unwrap()[index].clone()
}

/// A shortcut icon pressed anywhere inside its slot on the classic bar (the header band of the
/// bar's window, its top grip and the slot numbers are all there) follows the pointer and drops
/// into the next slot: the way a player moves an icon, at the user's setup.
#[test]
fn a_shortcut_icon_pressed_anywhere_in_its_slot_moves_on_the_classic_bar() {
    use ac2_net::protocol::ShortcutInfo;
    let Some(mut rig) = classic_big() else {
        return;
    };
    rig.open_inventory();
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    let centre = rig.view.shortcut_slot_center(3).unwrap();
    rig.drag(from, centre);
    assert_eq!(shortcut(&mut rig, 3), ShortcutInfo::Instance(SWORD));
    let other = rig.view.shortcut_slot_center(7).unwrap();
    for dy in [-14, -8, 0, 8, 14] {
        for dx in [-14, -8, 0, 8, 14] {
            let at = (centre.0 + dx, centre.1 + dy);
            // A player rests the pointer on the icon first: its tooltip is up at the press.
            rig.point(at);
            rig.idle(30);
            assert!(
                rig.view.hud_tree().unwrap().tooltip_element().is_some() || (dx, dy) != (0, 0),
                "the tooltip is up"
            );
            rig.press_and_move(at, other);
            let dragging = rig.view.is_dragging();
            // The icon that follows the pointer is the slot's size, not stretched over the screen.
            let tree = rig.view.hud_tree().unwrap();
            if let Some(copy) = tree.dragged() {
                let b = tree.screen_box(copy);
                assert!(
                    b.width() <= 64 && b.height() <= 64,
                    "the dragged icon is {b:?}"
                );
            }
            rig.release();
            assert!(
                dragging,
                "the icon pressed at {dx}, {dy} from its centre follows the pointer"
            );
            assert_eq!(shortcut(&mut rig, 7), ShortcutInfo::Instance(SWORD));
            assert_eq!(shortcut(&mut rig, 3), ShortcutInfo::Empty(0));
            rig.drag(other, centre);
            assert_eq!(shortcut(&mut rig, 3), ShortcutInfo::Instance(SWORD));
        }
    }
}

#[test]
fn a_dragged_pack_item_keeps_its_slot_size() {
    let Some(mut rig) = classic_big() else {
        return;
    };
    rig.open_inventory();
    rig.wait("the sword in the pack", |r| {
        r.view
            .item_icon_center("PackMainContainer", SWORD)
            .is_some()
    });
    let from = rig
        .view
        .item_icon_center("PackMainContainer", SWORD)
        .unwrap();
    rig.press_and_move(from, (from.0 - 300, from.1 - 200));
    rig.idle(4);
    snapshot(&mut rig, "dragging.png");
    let tree = rig.view.hud_tree().unwrap();
    let copy = tree.dragged().expect("an item is dragged");
    let b = tree.screen_box(copy);
    eprintln!("dragged {copy} box {b:?}");
    assert!(
        b.x1 - b.x0 <= 64 && b.y1 - b.y0 <= 64,
        "the dragged copy is the size of a slot, not {b:?}"
    );
    rig.release();
}

/// A trained skill dragged from the classic skill panel onto the bar at 2560 x 1440 becomes a
/// shortcut there (`gmSkillTreeUI`'s node, `gmShortcutUI::SetShortcut`).
#[test]
fn a_skill_dragged_from_the_classic_skill_panel_lands_on_the_bar() {
    use ac2_net::protocol::ShortcutInfo;
    const SKILL: u32 = 0xB0;
    let Some(mut rig) = classic_big() else {
        return;
    };
    {
        let mut server = rig.server.borrow_mut();
        server.give_starter_items();
        server.send_init_player();
        server.grant_xp(100);
        server.set_skill_cost(
            SKILL,
            legions_server_double::SkillCost { credits: 2, xp: 7 },
        );
    }
    rig.idle(16);
    rig.command(ac2_client::Command::Skill(ac2_weenie::SkillRequest::Train(
        SKILL,
    )));
    rig.idle(16);
    rig.wait("the bar", |r| r.view.shortcut_slots() > 0);
    rig.panel(ac2_ui::ToolbarChild::Skill, true);
    let melee = rig
        .client
        .root_panel_from_race(ac2_ui::ROOT_MELEE_PANEL)
        .unwrap()
        .unwrap();
    rig.command(ac2_client::Command::Examine(
        ac2_net::protocol::ExaminationRequest {
            kind: ac2_ui::EXAMINE_PANEL,
            data_enum: melee,
            ..Default::default()
        },
    ));
    rig.wait("the skill node", |r| {
        r.view.skill_node_center(SKILL).is_some()
    });
    rig.idle(8);
    snapshot(&mut rig, "skill_panel.png");
    let from = rig.view.skill_node_center(SKILL).unwrap();
    let to = rig.view.shortcut_slot_center(5).unwrap();
    eprintln!("skill node at {from:?}, slot 5 at {to:?}");
    rig.press_and_move(from, to);
    let dragging = rig.view.is_dragging();
    snapshot(&mut rig, "skill_dragging.png");
    // Over the bar, the dragged icon is the last surface drawn (`UISurfaceObject(1, -1.0)`).
    {
        let tree = rig.view.hud_tree().unwrap();
        let copy = tree.dragged().expect("the skill is dragged");
        let drawn = tree.placements().last().map(|p| p.root);
        assert_eq!(drawn, Some(copy), "the dragged skill is drawn over the bar");
    }
    rig.release();
    rig.idle(8);
    assert!(dragging, "the skill follows the pointer");
    assert_eq!(
        shortcut(&mut rig, 5),
        ShortcutInfo::Enum {
            kind: 0x4000_0001,
            value: SKILL
        }
    );
}
