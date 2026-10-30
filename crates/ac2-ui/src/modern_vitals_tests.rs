//! The modern vitals (`modern_vitals.rs`) on the real layouts (`AC2_DAT_DIR`): the bars side by
//! side over the experience bar, stacked after the switch, the indicators in a column at their
//! right, and the classic style left as retail's.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::modern_vitals::VitalsLayout;
use crate::token::tokenize;
use crate::{build_styled_hud, GameplayHud, HudStyle, UiTree};

fn hud(style: HudStyle) -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    Some(build_styled_hud(&store, 1365, 768, style).unwrap())
}

fn at(tree: &UiTree, name: &str) -> crate::element::UiBox {
    let id = tokenize(name);
    let e = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, id))
        .unwrap_or_else(|| panic!("{name}"));
    tree.screen_box(e)
}

#[test]
fn the_bars_sit_side_by_side_over_the_experience_bar_and_stack_after_the_switch() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let vitals = hud.vitals.modern().unwrap();
    assert_eq!(vitals.layout(), VitalsLayout::SideBySide);
    let (h, v, f, xp) = (
        at(tree, "HealthField"),
        at(tree, "VigorField"),
        at(tree, "FocusField"),
        at(tree, "ExperienceField"),
    );
    assert!(h.y0 == v.y0 && v.y0 == f.y0 && h.x1 < v.x0 && v.x1 < f.x0);
    assert!(xp.y0 > h.y1 && xp.x0 == h.x0 && xp.x1 == f.x1);
    // The indicators' column is right of the bars.
    let net = at(tree, "NetStatusField");
    assert!(net.x0 > f.x1);
    // The switch (shown under the pointer) stacks them.
    let switch = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("ModernVitalsLayout")))
        .unwrap();
    let message = crate::mouse::UiMessage {
        element_id: tokenize("ModernVitalsLayout"),
        element: Some(switch),
        message: crate::token::MSG_MOUSE_PRESS_TOP,
        data: 0x4000_0004,
        item: None,
    };
    struct NoStrings;
    impl crate::fellow_info::FellowStrings for NoStrings {
        fn render(&self, _: u32, _: &[(u32, crate::skill_tree::SkillVar)]) -> String {
            String::new()
        }
    }
    hud.vitals.message(tree, &message, &NoStrings);
    assert_eq!(hud.vitals.modern().unwrap().layout(), VitalsLayout::Stacked);
    let (h, v, f, xp) = (
        at(tree, "HealthField"),
        at(tree, "VigorField"),
        at(tree, "FocusField"),
        at(tree, "ExperienceField"),
    );
    assert!(h.x0 == v.x0 && v.x0 == f.x0 && h.y1 < v.y0 && v.y1 < f.y0 && f.y1 < xp.y0);
}

#[test]
fn the_classic_style_keeps_retails_status_bar() {
    let Some(hud) = hud(HudStyle::Classic) else {
        return;
    };
    assert!(hud.vitals.modern().is_none());
    let tree = &hud.tree;
    assert!(tree.roots().iter().all(|&r| tree
        .child_recursive(r, tokenize("ModernVitalsGlass"))
        .is_none()));
}

fn named(tree: &UiTree, name: &str) -> usize {
    let id = tokenize(name);
    tree.roots()
        .iter()
        .find_map(|&r| {
            if tree.el(r).id() == id {
                Some(r)
            } else {
                tree.child_recursive(r, id)
            }
        })
        .unwrap_or_else(|| panic!("{name}"))
}

/// At 30% health or less the health bar grows, the LOW tag and the screen's red edge show, and
/// the edge covers the display; back above, they go.
#[test]
fn low_health_grows_the_bar_and_shows_the_red_edge() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let text = |c: u64, m: u64| format!("{c} / {m}");
    hud.vitals
        .update_vital(tree, crate::vitals::VITAL_HEALTH, 100, 100, &text);
    hud.vitals.modern_tick(tree);
    let before = at(tree, "HealthField").height();
    let edge = named(tree, "ModernLowHealthEdge");
    let tag = named(tree, "ModernVitalsLow");
    assert!(!tree.el(edge).visible && !tree.el(tag).visible);
    hud.vitals
        .update_vital(tree, crate::vitals::VITAL_HEALTH, 25, 100, &text);
    hud.vitals.modern_tick(tree);
    assert!(tree.el(edge).visible && tree.el(tag).visible);
    assert!(at(tree, "HealthField").height() > before);
    assert_eq!(tree.el(edge).rect.width(), 1365);
    hud.vitals
        .update_vital(tree, crate::vitals::VITAL_HEALTH, 80, 100, &text);
    hud.vitals.modern_tick(tree);
    assert!(!tree.el(edge).visible && !tree.el(tag).visible);
    assert_eq!(at(tree, "HealthField").height(), before);
}

/// Every vital full: the veil dims the bars until the pointer is over the panel.
#[test]
fn full_vitals_are_veiled_until_the_pointer_comes_over() {
    let Some(mut hud) = hud(HudStyle::Modern) else {
        return;
    };
    let tree = &mut hud.tree;
    let text = |c: u64, m: u64| format!("{c} / {m}");
    for vital in [crate::vitals::VITAL_HEALTH, crate::vitals::VITAL_VIGOR] {
        hud.vitals.update_vital(tree, vital, 50, 50, &text);
    }
    tree.pointer = (900, 600);
    hud.vitals.modern_tick(tree);
    let veil = named(tree, "ModernVitalsVeil");
    assert!(tree.el(veil).visible);
    let health = at(tree, "HealthField");
    tree.pointer = (health.x0 + 2, health.y0 + 2);
    hud.vitals.modern_tick(tree);
    assert!(!tree.el(veil).visible);
    // Hurt: no veil even away from the panel.
    tree.pointer = (900, 600);
    hud.vitals
        .update_vital(tree, crate::vitals::VITAL_HEALTH, 40, 50, &text);
    hud.vitals.modern_tick(tree);
    assert!(!tree.el(veil).visible);
}
