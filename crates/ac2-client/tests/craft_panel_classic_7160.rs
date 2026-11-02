//! The craft panel (`gmCraftUI`) on the classic HUD at 2560 x 1440 (1x) over `AC2_DAT_DIR`, driven
//! with the pointer: the craft tab, a double click down to the selected craft skill's categories
//! and its recipes. With `AC2_CRAFT_SNAPSHOTS` set to a directory the tests write the HUD they
//! checked there as PNGs.
#![cfg(feature = "window")]

mod hud;

use ac2_ui::{tokenize, HudStyle, ToolbarChild, UiScaleSetting};
use hud::Rig;
use legions_server_double::CRAFT_SKILL;

const BIG: (u32, u32) = (2560, 1440);
/// The second press of a double click as the input layer sends it.
const DOUBLE_CLICK: u32 = 0x4000_0007;

fn classic_big() -> Option<Rig> {
    static RUN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let run = RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("ac2-craftpanel-{}-{run}", std::process::id()));
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
    let Ok(out) = std::env::var("AC2_CRAFT_SNAPSHOTS") else {
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

/// The craft page's element `name`.
fn page_child(rig: &Rig, name: &str) -> Option<usize> {
    let tree = rig.view.hud_tree()?;
    let page = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("CraftPage")))?;
    tree.child_recursive(page, tokenize(name))
}

fn text_of(rig: &Rig, name: &str) -> String {
    let tree = rig.view.hud_tree().unwrap();
    page_child(rig, name)
        .and_then(|e| tree.get(e).and_then(|e| e.text()))
        .unwrap_or_default()
}

fn rows(rig: &Rig, list: &str) -> Vec<usize> {
    let tree = rig.view.hud_tree().unwrap();
    page_child(rig, list)
        .map(|l| tree.list_items(l).to_vec())
        .unwrap_or_default()
}

fn double_click(rig: &mut Rig, row: usize) {
    let at = rig.view.ui_index_center(row).expect("the row shows");
    rig.point(at);
    rig.button(true);
    rig.button(false);
    rig.frame(&[(DOUBLE_CLICK, 1.0, true)]);
    rig.frame(&[(DOUBLE_CLICK, 1.0, false)]);
    rig.idle(8);
}

/// The selected craft skill's display (`gmCraftUI::RefreshCraftSkillDisplay`, Legions 0x44a360):
/// `SelectedName` the skill's name, `SelectedLevel` its level (`MakeLevelText`), `ButtonField` in
/// its `selected` state (which brings the raise button), and `MiscText` the raise text
/// (`ID_Craft_CantRaise` or `ID_Craft_AvailableXPRaise`), each on its own element. The client set
/// a `SelectedField` the layout does not have, so the raise button never showed, and wrote the
/// raise text over the level.
#[test]
fn a_selected_craft_skill_shows_its_name_level_and_raise_text() {
    let Some(mut rig) = classic_big() else {
        return;
    };
    rig.open_inventory();
    rig.server.borrow_mut().send_craft_registry();
    for _ in 0..600 {
        if rig
            .client
            .recipes(CRAFT_SKILL)
            .is_some_and(|x| !x.is_empty())
        {
            break;
        }
        rig.idle(1);
    }
    rig.panel(ToolbarChild::Inventory, false);
    rig.panel(ToolbarChild::Skill, true);
    let tab = rig
        .view
        .ui_element_center("CraftTab")
        .expect("the craft tab");
    rig.click(tab);
    rig.wait("the skill row", |r| !rows(r, "CraftSkillList").is_empty());
    snapshot(&mut rig, "craft_skills.png");
    // The row's forward arrow carries the skill (`MakeCraftSkill` with `text` 0 sets `did` on
    // `ForwardButton`, Legions 0x44a67a), so its press opens that skill's categories.
    let skill = rows(&rig, "CraftSkillList")[0];
    let forward = {
        let tree = rig.view.hud_tree().unwrap();
        tree.child_recursive(skill, tokenize("ForwardButton"))
            .expect("the row's forward button")
    };
    let at = rig.view.ui_index_center(forward).expect("the arrow shows");
    rig.click(at);
    rig.idle(8);
    rig.wait("the categories", |r| {
        !rows(r, "CraftSkillCategoryList").is_empty()
    });
    rig.idle(8);
    snapshot(&mut rig, "craft_categories.png");
    let name = text_of(&rig, "SelectedName");
    let level = text_of(&rig, "SelectedLevel");
    let misc = text_of(&rig, "MiscText");
    assert!(!name.is_empty(), "the skill's name");
    assert!(
        level.contains(char::is_numeric),
        "the level text shows the level: {level:?}"
    );
    assert!(!misc.is_empty(), "the raise text is in MiscText");
    assert_ne!(level, misc, "the raise text does not replace the level");
    // `ButtonField` takes the `selected` state, whose children hold the raise button; the button
    // is ghosted unless the skill can be raised (`CanCraftSkillBeRaised`).
    let tree = rig.view.hud_tree().unwrap();
    let field = page_child(&rig, "ButtonField").expect("the button field");
    assert_eq!(tree.get(field).unwrap().state(), tokenize("selected"));
    let raise = page_child(&rig, "RaiseCraftButton").expect("the raise button");
    let ghosted = matches!(
        tree.get(raise).unwrap().attribute(tokenize("ghosted")),
        Some(ac2_ui::AttributeValue::Bool(true))
    );
    let can = rig.client.can_craft_skill_be_raised(CRAFT_SKILL);
    assert_eq!(ghosted, can != Some(true), "the raise button's ghost");
    let categories = rows(&rig, "CraftSkillCategoryList");
    double_click(&mut rig, categories[0]);
    rig.idle(8);
    snapshot(&mut rig, "craft_recipes.png");
    let level = text_of(&rig, "SelectedLevel");
    assert!(
        level.contains(char::is_numeric),
        "the recipe view keeps the level: {level:?}"
    );
}
