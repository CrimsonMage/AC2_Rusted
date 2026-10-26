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
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        // A string-id variable (`AddVariable_StringID`) renders as its id.
        let vars: Vec<String> = vars
            .iter()
            .map(|(_, v)| match v {
                SkillVar::Info(info) => info.string_id.to_string(),
                v => format!("{v:?}"),
            })
            .collect();
        if vars.len() == 1 && vars[0].chars().all(|c| c.is_ascii_digit()) {
            return format!("{string_id}/{}", vars[0]);
        }
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
        item: None,
    };
    let shown = tree.get(health_text).unwrap().visible;
    hud.vitals.message(tree, &click, &Strings);
    assert_eq!(tree.get(health_text).unwrap().visible, !shown);
}

/// The tooltip a field carries, as `Strings` renders it: the string id, then its `value`
/// variable's (`ID_Vital_TT_Hide` while the numbers show, `ID_Vital_TT_Show` while hidden).
fn tooltip_of(tree: &UiTree, field: &str, string: &str, value: &str) -> Option<String> {
    tree.tooltip_text(find(tree, field)).map(|text| {
        assert_eq!(text, format!("{}/{}", tokenize(string), tokenize(value)));
        text
    })
}

#[test]
fn the_vitals_fields_carry_tooltips_that_follow_the_numbers() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    // Nothing yet; the constructor gives health and vigor theirs (numbers shown: "hide").
    assert!(tree.tooltip_text(find(tree, "HealthField")).is_none());
    hud.vitals.init_tooltips(tree, &Strings);
    assert!(tooltip_of(
        tree,
        "HealthField",
        "ID_Vital_TT_Health",
        "ID_Vital_TT_Hide"
    )
    .is_some());
    assert!(tooltip_of(tree, "VigorField", "ID_Vital_TT_Vigor", "ID_Vital_TT_Hide").is_some());
    // The focus field has none until the player is a hero.
    assert!(tree.tooltip_text(find(tree, "FocusField")).is_none());
    let click = |tree: &UiTree, field: &str| UiMessage {
        element_id: tokenize(field),
        element: Some(find(tree, field)),
        message: MSG_MOUSE_PRESS_TOP,
        data: 0x4000_0004,
        item: None,
    };
    // A click hides the numbers: the tooltip now offers to show them, and back.
    let message = click(tree, "VigorField");
    hud.vitals.message(tree, &message, &Strings);
    assert!(tooltip_of(tree, "VigorField", "ID_Vital_TT_Vigor", "ID_Vital_TT_Show").is_some());
    hud.vitals.message(tree, &message, &Strings);
    assert!(tooltip_of(tree, "VigorField", "ID_Vital_TT_Vigor", "ID_Vital_TT_Hide").is_some());
    // A hero's focus field gets one; a click toggles it; losing hero status clears it.
    let text = |cur, max| vital_text(&Strings, cur, max);
    hud.vitals
        .update_hero_status(tree, true, 5, 100, &text, &Strings);
    assert!(tooltip_of(tree, "FocusField", "ID_Vital_TT_Focus", "ID_Vital_TT_Hide").is_some());
    let message = click(tree, "FocusField");
    hud.vitals.message(tree, &message, &Strings);
    assert!(tooltip_of(tree, "FocusField", "ID_Vital_TT_Focus", "ID_Vital_TT_Show").is_some());
    hud.vitals
        .update_hero_status(tree, false, 0, 0, &text, &Strings);
    assert!(tree.tooltip_text(find(tree, "FocusField")).is_none());
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

#[test]
fn the_effect_strip_orders_effects_by_expiry_and_runs_their_meters() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let strip = hud.effects.as_mut().expect("the effect strip");
    let effect = |eid, time_cast, duration, pulsed| crate::EffectEntry {
        eid,
        name: format!("effect {eid}"),
        icon: 0x0600_2000 + eid,
        time_cast,
        duration,
        pulsed,
    };
    // A long timed effect, a short one, and a pulsed one without a duration.
    let effects = [
        effect(1, 100.0, 600.0, false),
        effect(2, 100.0, 60.0, false),
        effect(3, -1.0, -1.0, true),
    ];
    // The meter's animation runs on the tree's clock, which the frame's global loop sets.
    tree.global_loop(130.0);
    strip.init(tree, &effects, 130.0);
    assert_eq!(strip.shown(tree), vec![2, 1, 3]);
    let short = strip.element(tree, 2).unwrap();
    assert_eq!(tree.get(short).unwrap().state(), tokenize("active"));
    assert_eq!(tree.tooltip_text(short).as_deref(), Some("effect 2"));
    let meter = tree
        .child_recursive(short, tokenize("EffectTimeMeter"))
        .unwrap();
    // Half of the short effect is left: the meter starts there and runs down.
    let position = |tree: &UiTree| match tree.get(meter).unwrap().attribute(tokenize("position")) {
        Some(AttributeValue::Float(v)) => *v,
        _ => -1.0,
    };
    assert_eq!(position(tree), 0.5);
    tree.global_loop(145.0);
    assert!((position(tree) - 0.25).abs() < 1e-3, "{}", position(tree));
    // An effect that ended leaves the strip.
    strip.init(tree, &effects[..2], 140.0);
    assert_eq!(strip.shown(tree), vec![2, 1]);
}
