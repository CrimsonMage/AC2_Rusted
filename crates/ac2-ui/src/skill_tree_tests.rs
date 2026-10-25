//! The skills panel against the real layout (`AC2_DAT_DIR`), over a stub player.

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::*;

const PANEL: u32 = 0x8000_0010;
const BASE: u32 = 0x100;
const CHILD: u32 = 0x101;

struct Stub;

impl SkillSource for Stub {
    fn panel(&self, panel: u32) -> Option<SkillPanel> {
        (panel == PANEL).then(|| SkillPanel {
            name: "Melee".into(),
            icon: 0,
            nodes: vec![
                SkillNode {
                    skill: BASE,
                    kind: NODE_SKILL,
                    name: "Swords".into(),
                    dependencies: vec![CHILD],
                    x: 0,
                    y: 0,
                    ..SkillNode::default()
                },
                SkillNode {
                    skill: CHILD,
                    kind: NODE_SKILL,
                    name: "Lunge".into(),
                    parents: vec![BASE],
                    x: 0,
                    y: 1,
                    ..SkillNode::default()
                },
            ],
        })
    }

    fn root_panel_from_race(&self, root: u32) -> Option<u32> {
        (root == ROOT_MELEE_PANEL).then_some(PANEL)
    }

    fn status(&self, skill: u32) -> SkillStatus {
        if skill == BASE {
            SkillStatus {
                trained: true,
                level: 5,
                raw_level: 7,
                can_raise: 0,
                xp_to_level: 900,
                ..SkillStatus::default()
            }
        } else {
            SkillStatus {
                cost_to_train: 2,
                ..SkillStatus::default()
            }
        }
    }

    fn available(&self) -> (u64, u32, u32) {
        (5000, 3, 0)
    }

    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        let vars: Vec<String> = vars
            .iter()
            .map(|(_, v)| match v {
                SkillVar::UInt(n) => n.to_string(),
                SkillVar::Int(n) => n.to_string(),
                SkillVar::Text(t) => t.clone(),
                SkillVar::Float { value, .. } => value.to_string(),
                other => format!("{other:?}"),
            })
            .collect();
        format!("{string_id:08x}[{}]", vars.join(","))
    }
}

#[test]
fn a_panel_lays_out_its_nodes_and_shows_the_selected_skill() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut tree = build_gameplay_hud(&store, 1024, 768).unwrap().tree;
    let layout = crate::hud::load_layout_by_enum(&store, 0x4100_0003)
        .unwrap()
        .unwrap();
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("SkillField")))
        .unwrap();
    let mut ui = SkillTreeUi::create(&tree, &layout, field);
    let skill_tree = tree.child_recursive(field, tokenize("SkillTree")).unwrap();
    // The melee tab asks for the race's melee root panel.
    let melee = tree
        .child_recursive(field, tokenize("MeleeButton"))
        .unwrap();
    let press = UiMessage {
        element_id: tokenize("MeleeButton"),
        element: Some(melee),
        message: crate::token::MSG_MOUSE_PRESS,
        data: 0x4000_0004,
        item: None,
    };
    ui.message(&mut tree, &Stub, &press);
    assert_eq!(
        ui.take_requests(),
        vec![SkillRequest::Examine {
            kind: EXAMINE_PANEL,
            data: PANEL
        }]
    );
    ui.update_skill_panel(&mut tree, &Stub);
    ui.examination_request_changed(&mut tree, &Stub, EXAMINE_PANEL, PANEL);
    assert_eq!(ui.current(), (PANEL, 0));
    let base = tree.tree_item(skill_tree, 0, 0).expect("base node");
    let child = tree.tree_item(skill_tree, 0, 1).expect("child node");
    // A vertical tree: the child sits above its parent.
    assert!(tree.get(child).unwrap().rect.y0 < tree.get(base).unwrap().rect.y0);
    // A skill node takes the template's `skill` state, which holds the level text.
    assert_eq!(tree.get(base).unwrap().state(), tokenize("skill"));
    let level = tree
        .child_recursive(base, tokenize("SkillNodeLevelText"))
        .expect("level text");
    assert_eq!(
        tree.get(level).unwrap().text().as_deref(),
        Some(&*format!("{:08x}[7]", tokenize("ID_Skill_LevelNormal")))
    );
    assert_eq!(tree.get(level).unwrap().state(), tokenize("buff"));
    let area = |tree: &UiTree, node: usize| {
        tree.get(tree.child_recursive(node, tokenize("IconArea")).unwrap())
            .unwrap()
            .state()
    };
    assert_eq!(area(&tree, base), tokenize("canraise"));
    assert_eq!(area(&tree, child), tokenize("cantrain"));
    // Selecting the untrained skill shows its train text and enables training.
    ui.examination_request_changed(&mut tree, &Stub, EXAMINE_SKILL, CHILD);
    assert_eq!(ui.current(), (PANEL, CHILD));
    let modify = tree
        .child_recursive(field, tokenize("ModifyField"))
        .unwrap();
    assert_eq!(tree.get(modify).unwrap().state(), tokenize("untrained"));
    let misc = tree.child_recursive(field, tokenize("MiscText")).unwrap();
    assert_eq!(
        tree.get(misc).unwrap().text().as_deref(),
        Some(&*format!(
            "{:08x}[3,2]",
            tokenize("ID_Skill_AvailableSPTrain")
        ))
    );
    let name = tree
        .child_recursive(field, tokenize("SelectedName"))
        .unwrap();
    assert_eq!(tree.get(name).unwrap().text().as_deref(), Some("Lunge"));
    // Train asks for confirmation (the skill can be untrained later).
    let train = tree
        .child_recursive(field, tokenize("TrainButton"))
        .unwrap();
    let release = UiMessage {
        element_id: tokenize("TrainButton"),
        element: Some(train),
        message: crate::token::MSG_BUTTON_RELEASE_ON,
        data: 0x4000_0004,
        item: None,
    };
    ui.message(&mut tree, &Stub, &release);
    assert_eq!(ui.take_requests(), vec![SkillRequest::ConfirmTrain(CHILD)]);
}

/// An empty shortcut bar.
struct EmptyBar;

impl ShortcutSource for EmptyBar {
    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![Shortcut::default(); 100]
    }
    fn icon(&self, _: usize, _: &Shortcut) -> u32 {
        0
    }
    fn options(&self) -> ShortcutOptions {
        ShortcutOptions {
            which_set: 1,
            rows: 1,
            ..ShortcutOptions::default()
        }
    }
    fn skill_trained(&self, _: u32) -> bool {
        true
    }
    fn has_item(&self, _: u64) -> bool {
        false
    }
}

#[test]
fn a_trained_skill_dragged_onto_the_bar_becomes_a_shortcut() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let (mut tree, mut bar) = (hud.tree, hud.shortcuts.expect("the shortcut bar"));
    let mut ui = hud.skills.expect("the skills panel");
    bar.reload(&mut tree, &EmptyBar);
    bar.take_requests();
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("SkillField")))
        .unwrap();
    ui.update_skill_panel(&mut tree, &Stub);
    ui.examination_request_changed(&mut tree, &Stub, EXAMINE_PANEL, PANEL);
    // The skills panel shown.
    let mut at = Some(field);
    while let Some(e) = at {
        tree.set_visible(e, true);
        at = tree.get(e).unwrap().parent;
    }
    let skill_tree = tree.child_recursive(field, tokenize("SkillTree")).unwrap();
    let base = tree.tree_item(skill_tree, 0, 0).expect("base node");
    let icon = tree
        .child_recursive(base, tokenize("SkillNodeIcon"))
        .unwrap();
    assert!(
        tree.bool_attribute(icon, tokenize("dragable")),
        "a trained skill drags"
    );
    let centre = |tree: &UiTree, e: usize| {
        let b = tree.screen_box(e);
        ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2)
    };
    let list = bar.elements().1.unwrap();
    let (x, y) = centre(&tree, icon);
    let (tx, ty) = centre(&tree, tree.list_items(list)[2]);
    tree.start_input(crate::UI_POINTER_X, x as f32);
    tree.start_input(crate::UI_POINTER_Y, y as f32);
    tree.start_input(0x4000_0004, 1.0);
    tree.start_input(crate::UI_POINTER_X, tx as f32);
    tree.start_input(crate::UI_POINTER_Y, ty as f32);
    assert!(tree.dragged().is_some(), "the skill is dragged");
    tree.stop_input(0x4000_0004);
    for message in tree.take_messages() {
        bar.message(&mut tree, &EmptyBar, &message);
    }
    let skill = Shortcut {
        kind: SHORTCUT_SKILL,
        skill: BASE,
        ..Shortcut::default()
    };
    assert_eq!(bar.take_requests(), vec![ShortcutRequest::Set(2, skill)]);
}
