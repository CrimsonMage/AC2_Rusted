//! `gmVitalsUI` on the real status bar layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::{
    build_gameplay_hud, vital_text, FellowStrings, SkillVar, UiTree, VITAL_EXPERIENCE, VITAL_HEALTH,
};

struct Strings;

impl FellowStrings for Strings {
    fn render(&self, _: u32, vars: &[(u32, SkillVar)]) -> String {
        let vars: Vec<String> = vars.iter().map(|(_, v)| format!("{v:?}")).collect();
        vars.join("/")
    }
}

fn find(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize(name)))
        .unwrap_or_else(|| panic!("{name}"))
}

fn goal(tree: &UiTree, meter: usize) -> Option<f32> {
    match tree.get(meter)?.attribute(tokenize("goal_position")) {
        Some(AttributeValue::Float(v)) => Some(*v),
        _ => None,
    }
}

#[test]
fn the_status_bar_shows_health_and_experience() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let text = |cur, max| vital_text(&Strings, cur, max);
    hud.vitals.update_vital(tree, VITAL_HEALTH, 30, 120, &text);
    hud.vitals
        .update_vital(tree, VITAL_EXPERIENCE, 500, 1000, &text);
    let health_text = find(tree, "HealthText");
    assert_eq!(
        tree.get(health_text).unwrap().text().unwrap(),
        "UInt(30)/UInt(120)"
    );
    // The health meter goes to a quarter (or, filling from the right, three quarters).
    let goal = goal(tree, find(tree, "HealthMeter")).unwrap();
    assert!(goal == 0.25 || goal == 0.75, "{goal}");
    let experience = find(tree, "ExperienceText");
    assert_eq!(
        tree.get(experience).unwrap().text().unwrap(),
        "UInt(500)/UInt(1000)"
    );
    // A click on the health field hides its numbers; another shows them.
    let field = find(tree, "HealthField");
    let click = UiMessage {
        element_id: tokenize("HealthField"),
        element: Some(field),
        message: MSG_MOUSE_PRESS_TOP,
        data: 0x4000_0004,
    };
    let shown = tree.get(health_text).unwrap().visible;
    hud.vitals.message(tree, &click);
    assert_eq!(tree.get(health_text).unwrap().visible, !shown);
}

#[test]
fn the_reticle_meters_show_the_selected_creatures_health() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let reticle = hud.reticle.as_mut().expect("the reticle");
    reticle.selection_changed(tree, Some(7), Some("Birch"));
    let info = ac2_net::protocol::SelectionInfo {
        kind: ac2_net::protocol::SelectionInfo::CREATURE,
        cur_health: 30,
        max_health: 120,
        cur_vigor: 50,
        max_vigor: 50,
        ..Default::default()
    };
    reticle.selection_info(tree, &info);
    let health = goal(tree, find(tree, "SelectionHealthMeter")).unwrap();
    assert!(health == 0.25 || health == 0.75, "{health}");
    let vigor = goal(tree, find(tree, "SelectionVigorMeter")).unwrap();
    assert!(vigor == 1.0 || vigor == 0.0, "{vigor}");
}
