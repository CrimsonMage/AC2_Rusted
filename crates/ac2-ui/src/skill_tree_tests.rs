//! The skills panel against the real layout (`AC2_DAT_DIR`), over a stub player.

use std::sync::Arc;

use ac2_db::ResourceStore;

use super::*;
use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION, LAYOUT_DIALOGS};

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
    let mut dialogs = DialogFactory::new(None);
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
    ui.message(&mut tree, &mut dialogs, &Stub, &press);
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
    ui.message(&mut tree, &mut dialogs, &Stub, &release);
    // The question is up; nothing is asked until it is answered.
    let (context, kind) = dialogs.shown().expect("the train confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert!(ui.take_requests().is_empty());
    dialogs.answer(&mut tree, context, true);
    ui.dialog_outcomes(&dialogs.take_outcomes());
    assert_eq!(ui.take_requests(), vec![SkillRequest::Train(CHILD)]);
}

/// The skill tree over the real layout, its dialogs over the dialogs' layout.
fn panel_with_dialogs(
    store: &Arc<ResourceStore>,
    source: &dyn SkillSource,
) -> (UiTree, SkillTreeUi, DialogFactory, usize) {
    let mut tree = build_gameplay_hud(store, 1024, 768).unwrap().tree;
    let layout = crate::hud::load_layout_by_enum(store, 0x4100_0003)
        .unwrap()
        .unwrap();
    let dialog_layout = crate::hud::load_layout_by_enum(store, LAYOUT_DIALOGS)
        .unwrap()
        .unwrap();
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("SkillField")))
        .unwrap();
    let mut ui = SkillTreeUi::create(&tree, &layout, field);
    let dialogs = DialogFactory::new(Some(dialog_layout));
    ui.update_skill_panel(&mut tree, source);
    ui.examination_request_changed(&mut tree, source, EXAMINE_PANEL, PANEL);
    ui.examination_request_changed(&mut tree, source, EXAMINE_SKILL, CHILD);
    // The panel is shown, as the toolbar shows it.
    tree.set_visible(field, true);
    (tree, ui, dialogs, field)
}

/// The question text of the shown confirmation dialog.
fn question(tree: &UiTree) -> Option<String> {
    tree.roots().iter().find_map(|&r| {
        let text = tree.child_recursive(r, tokenize("QuestionText"))?;
        tree.get(text)?.text()
    })
}

fn release(tree: &UiTree, field: usize, name: &str) -> UiMessage {
    let element = tree
        .child_recursive(field, tokenize(name))
        .unwrap_or_else(|| panic!("no {name} under the skill field"));
    UiMessage {
        element_id: tokenize(name),
        element: Some(element),
        message: crate::token::MSG_BUTTON_RELEASE_ON,
        data: 0x4000_0004,
        item: None,
    }
}

#[test]
fn untrain_and_cancel_ask_by_name_and_only_a_yes_asks_the_player() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut ui, mut dialogs, field) = panel_with_dialogs(&store, &Stub);
    // The trained skill's untrain asks about "Swords".
    ui.examination_request_changed(&mut tree, &Stub, EXAMINE_SKILL, BASE);
    let untrain = release(&tree, field, "UntrainButton");
    ui.message(&mut tree, &mut dialogs, &Stub, &untrain);
    let (context, kind) = dialogs.shown().expect("the untrain confirmation");
    assert_eq!(kind, DIALOG_CONFIRMATION);
    assert_eq!(
        question(&tree).as_deref(),
        Some(&*format!(
            "{:08x}[Swords]",
            tokenize("ID_Skill_UntrainText")
        ))
    );
    dialogs.answer(&mut tree, context, false);
    ui.dialog_outcomes(&dialogs.take_outcomes());
    assert!(ui.take_requests().is_empty(), "a no asks nothing");
    // Cancelling the untrain asks its own question, and a yes asks the cancel.
    // The cancel button is in the layout's `untraining` state, which the source's status sets.
    ui.examination_request_changed(&mut tree, &Untraining, EXAMINE_SKILL, CHILD);
    ui.examination_request_changed(&mut tree, &Untraining, EXAMINE_SKILL, BASE);
    let cancel = release(&tree, field, "CancelUntrainButton");
    ui.message(&mut tree, &mut dialogs, &Stub, &cancel);
    assert_eq!(
        question(&tree).as_deref(),
        Some(&*format!(
            "{:08x}[]",
            tokenize("ID_Skill_UntrainCancelConfirm")
        ))
    );
    let (context, _) = dialogs.shown().unwrap();
    dialogs.answer(&mut tree, context, true);
    ui.dialog_outcomes(&dialogs.take_outcomes());
    assert_eq!(ui.take_requests(), vec![SkillRequest::CancelUntrain(BASE)]);
}

#[test]
fn a_second_confirmation_of_one_kind_waits_for_the_first() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut ui, mut dialogs, field) = panel_with_dialogs(&store, &Stub);
    let train = release(&tree, field, "TrainButton");
    ui.message(&mut tree, &mut dialogs, &Stub, &train);
    let first = dialogs.shown().unwrap().0;
    ui.message(&mut tree, &mut dialogs, &Stub, &train);
    assert_eq!(dialogs.shown().unwrap().0, first, "no second train dialog");
    assert!(dialogs.queued().is_empty());
}

#[test]
fn showing_or_hiding_the_panel_closes_its_dialogs() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut ui, mut dialogs, field) = panel_with_dialogs(&store, &Stub);
    ui.follow_show_state(&mut tree, &mut dialogs);
    let train = release(&tree, field, "TrainButton");
    ui.message(&mut tree, &mut dialogs, &Stub, &train);
    assert!(dialogs.shown().is_some());
    tree.set_visible(field, false);
    ui.follow_show_state(&mut tree, &mut dialogs);
    assert!(dialogs.shown().is_none(), "hiding closes the question");
    // The answer comes too late: nothing is asked.
    ui.dialog_outcomes(&[DialogOutcome::Confirmation {
        context: 1,
        yes: true,
    }]);
    assert!(ui.take_requests().is_empty());
}

#[test]
fn the_buttons_and_nodes_carry_their_tooltips() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (mut tree, mut ui, _dialogs, field) = panel_with_dialogs(&store, &Stub);
    // The untrained child can be trained: its train button says so, the value being 0.
    let train = tree
        .child_recursive(field, tokenize("TrainButton"))
        .unwrap();
    assert_eq!(
        tree.tooltip_text(train).as_deref(),
        Some(&*format!("{:08x}[0]", tokenize("ID_Skill_TT_ClickToTrain")))
    );
    // The trained base can be raised: the raise button shows the experience to level.
    ui.examination_request_changed(&mut tree, &Stub, EXAMINE_SKILL, BASE);
    let raise = tree
        .child_recursive(field, tokenize("RaiseButton"))
        .unwrap();
    assert_eq!(
        tree.tooltip_text(raise).as_deref(),
        Some(&*format!(
            "{:08x}[900]",
            tokenize("ID_Skill_TT_ClickToRaise")
        ))
    );
    // A trained node's tooltip is its name with the level.
    let skill_tree = tree.child_recursive(field, tokenize("SkillTree")).unwrap();
    let base_item = tree.tree_item(skill_tree, 0, 0).unwrap();
    assert_eq!(
        tree.tooltip_text(base_item).as_deref(),
        Some(&*format!(
            "{:08x}[Swords,{:08x}[7,2]]",
            tokenize("ID_Skill_NameLevel"),
            tokenize("ID_Skill_LevelBuffed")
        ))
    );
    let child_item = tree.tree_item(skill_tree, 0, 1).unwrap();
    assert_eq!(tree.tooltip_text(child_item).as_deref(), Some("Lunge"));
}

/// The same skills under names too long for the selected name's field.
struct LongNames;

/// The base skill being untrained: the status the layout's `untraining` state reads.
struct Untraining;

impl SkillSource for Untraining {
    fn panel(&self, panel: u32) -> Option<SkillPanel> {
        Stub.panel(panel)
    }
    fn root_panel_from_race(&self, root: u32) -> Option<u32> {
        Stub.root_panel_from_race(root)
    }
    fn status(&self, skill: u32) -> SkillStatus {
        let mut status = Stub.status(skill);
        if skill == BASE {
            status.untraining = true;
        }
        status
    }
    fn available(&self) -> (u64, u32, u32) {
        Stub.available()
    }
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        Stub.render(string_id, vars)
    }
}

impl SkillSource for LongNames {
    fn panel(&self, panel: u32) -> Option<SkillPanel> {
        Stub.panel(panel).map(|mut p| {
            for node in &mut p.nodes {
                node.name = "Wide ".repeat(60);
            }
            p
        })
    }
    fn root_panel_from_race(&self, root: u32) -> Option<u32> {
        Stub.root_panel_from_race(root)
    }
    fn status(&self, skill: u32) -> SkillStatus {
        Stub.status(skill)
    }
    fn available(&self) -> (u64, u32, u32) {
        Stub.available()
    }
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        Stub.render(string_id, vars)
    }
}

#[test]
fn a_long_name_is_cut_with_the_ellipsis_to_fit_the_field() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let (tree, _ui, _dialogs, field) = panel_with_dialogs(&store, &LongNames);
    let name = tree
        .child_recursive(field, tokenize("SelectedName"))
        .unwrap();
    let text = tree.get(name).unwrap().text().unwrap();
    let ellipsis = format!("{:08x}[]", tokenize("ID_Ellipsis"));
    assert!(text.ends_with(&ellipsis), "cut text ends with the ellipsis");
    let width = tree.get(name).unwrap().rect().width();
    assert!(tree.text_line_width(name, &text).unwrap() <= width + 1);
    assert!(text.len() < 300 * 2, "the text was shortened");
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
