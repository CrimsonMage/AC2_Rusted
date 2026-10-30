//! `gmSkillTreeUI`: the skills panel. A panel of skill nodes (`gSkillPanelManager`'s node hash
//! for a panel enum) is laid out in the `SkillTree` element, one `SkillNodeTemplate` per node at
//! its cell, its parents added first and linked (`LoadPanel`, `AddNode`). Each node shows its
//! icon and, for a trained skill, its level (`RefreshSkillNode`, `MakeLevelText`); its icon area
//! shows whether it can be trained or raised (`FastRefreshSkillNode`). The selected skill fills
//! the name, icon, text and train / raise / untrain buttons (`UpdateSelectedSkillDisplay`).
//!
//! What the panel shows follows the examination request (`RecvNotice_ExaminationRequestChanged`:
//! a panel request loads the panel, a skill request selects the skill); the tab buttons and node
//! icons make examination requests ([`SkillRequest::Examine`]), and the train / raise / untrain
//! buttons ask the player's interface ([`SkillRequest`]). The player's skills and the string
//! table text come from a [`SkillSource`].
//!
//! The train, untrain and cancel buttons ask first: each makes its confirmation dialog
//! (`MakeTrainDialog`, `MakeUntrainDialog`, `MakeUntrainCancelDialog`) and the answer comes back
//! through [`SkillTreeUi::dialog_outcomes`] (`RecvNotice_CloseConfirmationDialog`). Showing or
//! hiding the panel closes its dialogs (`Show`). The buttons and nodes carry tooltips, and the
//! selected name is truncated with an ellipsis (`SetStringAndTruncate`).
//!
//! A trained, active skill's icon drags to the shortcut bar (its `Token_ShortcutType` and data;
//! the drop is `shortcut.rs`).

use std::collections::HashMap;
use std::sync::Arc;

use crate::dialog::{DialogFactory, DialogOutcome, DIALOG_CONFIRMATION};
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::panel::MSG_OPENED;
use crate::token::{
    tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS,
    MSG_MOUSE_PRESS_TOP, STATELESS,
};

/// The race-independent root panels the tab buttons name (`Root*Panel_SkillPanel`).
pub const ROOT_MELEE_PANEL: u32 = 0x8000_0000;
pub const ROOT_MISSILE_PANEL: u32 = 0x8000_0001;
pub const ROOT_MAGIC_PANEL: u32 = 0x8000_0002;
pub const ROOT_NONCOMBAT_PANEL: u32 = 0x8000_0003;

/// `SkillUINode::mType` of a skill (other nodes link to panels).
pub const NODE_SKILL: u32 = 0x4000_0001;
/// `ExaminationRequest` types: a skill, a skill panel.
pub const EXAMINE_SKILL: u32 = 0x4000_0003;
pub const EXAMINE_PANEL: u32 = 0x4000_0004;
/// `CanSkillBeTrainedByEnum`'s answer for a skill that can never be trained here.
const CANT_TRAIN: u32 = 0x7558;

/// A node of a skill panel (`SkillUINode`), its strings rendered.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillNode {
    /// `mEnum`: the skill (or the panel a link node opens).
    pub skill: u32,
    /// `mType`.
    pub kind: u32,
    pub name: String,
    pub icon: u32,
    pub parents: Vec<u32>,
    pub dependencies: Vec<u32>,
    pub x: i32,
    pub y: i32,
    pub passive: bool,
    pub untrainable: bool,
}

/// A skill panel (`InqNodeHash`, `InqName`, `InqIcon`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillPanel {
    pub name: String,
    pub icon: u32,
    pub nodes: Vec<SkillNode>,
}

/// The player's answers about one skill (`gmCWInterface::*ByEnum`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SkillStatus {
    pub trained: bool,
    pub untraining: bool,
    pub hero: bool,
    /// `CanSkillBeTrainedByEnum` / `CanSkillBeRaisedByEnum` / `CanSkillBeUntrainedByEnum`: 0
    /// when it can.
    pub can_train: u32,
    pub can_raise: u32,
    pub can_untrain: u32,
    /// `InqSkillLevelByEnum`: the level and the raw (buffed) level.
    pub level: u32,
    pub raw_level: u32,
    pub cost_to_train: u64,
    pub xp_to_level: u64,
    pub skill_xp: u64,
}

/// A string variable for [`SkillSource::render`] (`StringInfo::AddVariable_*`).
#[derive(Debug, Clone, PartialEq)]
pub enum SkillVar {
    UInt(u64),
    Int(i64),
    Text(String),
    /// `AddVariable_Float(value, precision)`.
    Float {
        value: f64,
        precision: u32,
    },
    /// `AddVariable_StringInfo` of the UI's own string.
    Info(Box<crate::ui_string::UiString>),
    /// `AddVariable_StringInfo` of a string the server sent.
    Wire(ac2_net::protocol::StringInfo),
}

/// What the skills panel asks of the player and the string tables.
pub trait SkillSource {
    /// `gSkillPanelManager`'s panel, its names rendered.
    fn panel(&self, panel: u32) -> Option<SkillPanel>;
    /// `gmCWInterface::InqRootPanelFromRace`.
    fn root_panel_from_race(&self, root: u32) -> Option<u32>;
    fn status(&self, skill: u32) -> SkillStatus;
    /// The available experience, skill credits and hero skill credits.
    fn available(&self) -> (u64, u32, u32);
    /// A string of the chat / UI table (`0x41000018`) with its variables.
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String;
}

/// What the skills panel asks of the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillRequest {
    /// `SetSelection(INVALID_IID)`, `SetExaminationRequest`, `SendNotice_OpenExaminationPanel`.
    Examine { kind: u32, data: u32 },
    /// `gmCWInterface::RequestTrainSkill` (an untrainable skill is trained at once).
    Train(u32),
    /// `RequestRaiseSkill`.
    Raise(u32),
    /// `RequestUntrainSkill`: the untrain dialog was answered yes.
    Untrain(u32),
    /// `RequestUntrainSkillCancel`: the cancel dialog was answered yes.
    CancelUntrain(u32),
}

/// One confirmation dialog of the panel: its context (0 when none is up) and the skill it asks
/// about (`_untrainContext`, `_untrainCancelContext`, `_trainContext` and the skill they take).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Confirm {
    context: u32,
    skill: u32,
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmSkillTreeUI`'s state.
#[derive(Debug, Clone)]
pub struct SkillTreeUi {
    layout: Arc<LayoutDesc>,
    field: usize,
    tree: Option<usize>,
    selected_name: Option<usize>,
    misc_text: Option<usize>,
    selected_icon: Option<usize>,
    modify_field: Option<usize>,
    current_panel: u32,
    selected_skill: u32,
    nodes: HashMap<u32, SkillNode>,
    available: (u64, u32, u32),
    requests: Vec<SkillRequest>,
    train: Confirm,
    untrain: Confirm,
    cancel: Confirm,
    /// The field's shown state at the last [`SkillTreeUi::follow_show_state`].
    shown: Option<bool>,
}

/// `InitElements`: the tab buttons and their root panels.
const ELEMENT_PANELS: [(&str, u32); 4] = [
    ("MeleeButton", ROOT_MELEE_PANEL),
    ("MissileButton", ROOT_MISSILE_PANEL),
    ("MagicButton", ROOT_MAGIC_PANEL),
    ("NoncombatButton", ROOT_NONCOMBAT_PANEL),
];

impl SkillTreeUi {
    /// `gmSkillTreeUI::gmSkillTreeUI` over the `SkillField` element.
    pub fn create(tree: &UiTree, layout: &Arc<LayoutDesc>, field: usize) -> Self {
        let child = |name: &str| tree.child_recursive(field, id(name));
        Self {
            layout: Arc::clone(layout),
            field,
            tree: child("SkillTree"),
            selected_name: child("SelectedName"),
            misc_text: child("MiscText"),
            selected_icon: child("SelectedIcon"),
            modify_field: child("ModifyField"),
            current_panel: 0,
            selected_skill: 0,
            nodes: HashMap::new(),
            available: (0, 0, 0),
            requests: Vec::new(),
            train: Confirm::default(),
            untrain: Confirm::default(),
            cancel: Confirm::default(),
            shown: None,
        }
    }

    pub fn take_requests(&mut self) -> Vec<SkillRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The panel shown (0 for none) and the selected skill.
    pub fn current(&self) -> (u32, u32) {
        (self.current_panel, self.selected_skill)
    }

    /// `RecvNotice_ExaminationRequestChanged`.
    pub fn examination_request_changed(
        &mut self,
        tree: &mut UiTree,
        source: &dyn SkillSource,
        kind: u32,
        data: u32,
    ) {
        match kind {
            EXAMINE_PANEL if data != self.current_panel => self.load_panel(tree, source, data),
            EXAMINE_SKILL if data != self.selected_skill => self.select_skill(tree, source, data),
            EXAMINE_PANEL | EXAMINE_SKILL => {}
            _ => self.clear_selected_info(tree),
        }
    }

    /// `RecvNotice_UpdateSkillPanel`: the new totals, then every node's icon state.
    pub fn update_skill_panel(&mut self, tree: &mut UiTree, source: &dyn SkillSource) {
        self.available = source.available();
        let nodes: Vec<SkillNode> = self.nodes.values().cloned().collect();
        for node in &nodes {
            self.fast_refresh_skill_node(tree, source, node);
        }
    }

    /// `RecvNotice_UpdateSkill`: the nodes that depend on `skill` refresh.
    pub fn update_skill(&mut self, tree: &mut UiTree, source: &dyn SkillSource, skill: u32) {
        let Some(node) = self.nodes.get(&skill) else {
            return;
        };
        for dependency in node.dependencies.clone() {
            if let Some(node) = self.nodes.get(&dependency).cloned() {
                self.refresh_skill_node(tree, source, &node);
            }
        }
    }

    /// `gmSkillTreeUI::LoadPanel`.
    fn load_panel(&mut self, tree: &mut UiTree, source: &dyn SkillSource, panel: u32) {
        self.current_panel = panel;
        self.nodes.clear();
        let data = source.panel(panel).unwrap_or_default();
        if let Some(skill_tree) = self.tree {
            tree.tree_flush(skill_tree);
            self.nodes = data.nodes.iter().map(|n| (n.skill, n.clone())).collect();
            tree.tree_multi_op_start(skill_tree);
            for node in &data.nodes {
                self.add_node(tree, source, node);
            }
            tree.tree_multi_op_complete(skill_tree);
            tree.tree_end(skill_tree);
        }
        self.selected_skill = 0;
        if let Some(name) = self.selected_name {
            tree.set_text(name, &data.name);
        }
        if let Some(misc) = self.misc_text {
            tree.clear_text(misc);
        }
        if let Some(icon) = self.selected_icon {
            tree.set_image(icon, data.icon, 2);
        }
        if let Some(modify) = self.modify_field {
            tree.set_state(modify, id("panel"));
        }
    }

    /// `gmSkillTreeUI::AddNode`: the node in its cell, after its parents, linked to them.
    fn add_node(&mut self, tree: &mut UiTree, source: &dyn SkillSource, node: &SkillNode) {
        let Some(skill_tree) = self.tree else {
            return;
        };
        if tree.tree_is_occupied(skill_tree, node.x, node.y) {
            return;
        }
        let Some(template) = self.layout.element(id("SkillNodeTemplate")).cloned() else {
            return;
        };
        let Some(item) = tree.create_element(Arc::clone(&self.layout), template, false) else {
            return;
        };
        tree.tree_add_item(skill_tree, item, node.x, node.y);
        for parent in &node.parents {
            let Some(parent) = self.nodes.get(parent).cloned() else {
                continue;
            };
            if parent.x >= 0 && parent.y >= 0 {
                self.add_node(tree, source, &parent);
                tree.tree_set_items_parent(skill_tree, item, parent.x, parent.y);
            }
        }
        self.refresh_skill_node(tree, source, node);
    }

    fn node_child(&self, tree: &UiTree, node: &SkillNode, name: &str) -> Option<usize> {
        let item = tree.tree_item(self.tree?, node.x, node.y)?;
        tree.child_recursive(item, id(name))
    }

    /// `gmSkillTreeUI::MakeLevelText`: the level, with the buff shown (`state`) when it differs;
    /// and the level text's state (`stateless`, `buff`, `debuff`).
    fn level_text(source: &dyn SkillSource, status: &SkillStatus, state: bool) -> (String, u32) {
        let diff = status.raw_level as i32 - status.level as i32;
        let mut vars = vec![(id("level"), SkillVar::UInt(u64::from(status.raw_level)))];
        let mut string = id("ID_Skill_LevelNormal");
        let mut look = STATELESS;
        if diff != 0 {
            if state {
                string = if diff < 0 {
                    id("ID_Skill_LevelDebuffed")
                } else {
                    id("ID_Skill_LevelBuffed")
                };
                vars.push((id("value"), SkillVar::Int(i64::from(diff.unsigned_abs()))));
            }
            look = if diff < 0 { id("debuff") } else { id("buff") };
        }
        (source.render(string, &vars), look)
    }

    /// `gmSkillTreeUI::RefreshSkillNode`.
    fn refresh_skill_node(
        &mut self,
        tree: &mut UiTree,
        source: &dyn SkillSource,
        node: &SkillNode,
    ) {
        let Some(icon) = self.node_child(tree, node, "SkillNodeIcon") else {
            return;
        };
        tree.add_image_to_state(icon, node.icon, 2, crate::token::STATELESS);
        let attrs = &mut tree.el_mut(icon).attributes;
        attrs.insert(id("Token_ShortcutType"), AttributeValue::Id(NODE_SKILL));
        attrs.insert(id("Token_ShortcutData"), AttributeValue::Id(node.skill));
        // The node's look by its type (retail falls back to the icon id as the state).
        let look = match node.kind {
            NODE_SKILL => id("skill"),
            0x4000_0002 => id("plus"),
            0x4000_0003 => id("minus"),
            _ => node.icon,
        };
        if let Some(item) = self.tree.and_then(|t| tree.tree_item(t, node.x, node.y)) {
            tree.set_state(item, look);
        }
        if node.kind == NODE_SKILL {
            let status = source.status(node.skill);
            if let Some(level) = self.node_child(tree, node, "SkillNodeLevelText") {
                if status.trained {
                    let (text, look) = Self::level_text(source, &status, false);
                    tree.set_state(level, look);
                    tree.set_text(level, &text);
                } else {
                    tree.set_state(level, STATELESS);
                    tree.clear_text(level);
                }
            }
            tree.set_bool(icon, id("dragable"), status.trained && !node.passive);
        } else {
            tree.set_bool(icon, id("dragable"), false);
        }
        // The node's tooltip: its name, with the level text when it is trained.
        let trained = (node.kind == NODE_SKILL)
            .then(|| source.status(node.skill))
            .filter(|status| status.trained);
        let tooltip = match trained {
            Some(status) => {
                let (level, _) = Self::level_text(source, &status, true);
                source.render(
                    id("ID_Skill_NameLevel"),
                    &[
                        (id("value"), SkillVar::Text(node.name.clone())),
                        (id("level"), SkillVar::Text(level)),
                    ],
                )
            }
            None => node.name.clone(),
        };
        if let Some(item) = self.tree.and_then(|t| tree.tree_item(t, node.x, node.y)) {
            tree.set_tooltip(item, Some(tooltip));
        }
        self.fast_refresh_skill_node(tree, source, node);
    }

    /// `gmSkillTreeUI::FastRefreshSkillNode`: the icon area's look.
    fn fast_refresh_skill_node(
        &mut self,
        tree: &mut UiTree,
        source: &dyn SkillSource,
        node: &SkillNode,
    ) {
        if node.kind != NODE_SKILL {
            return;
        }
        let status = source.status(node.skill);
        let look = if !status.trained {
            if status.can_train != 0 {
                id("greyed")
            } else {
                id("cantrain")
            }
        } else if status.can_raise == 0 {
            id("canraise")
        } else {
            STATELESS
        };
        let Some(area) = self.node_child(tree, node, "IconArea") else {
            return;
        };
        if tree.get(area).map(|e| e.state()) != Some(look) {
            tree.set_state(area, look);
        }
        if node.skill == self.selected_skill {
            self.update_selected_skill_display(tree, source, node);
        }
    }

    /// `gmSkillTreeUI::SelectNode`.
    fn select_node(&self, tree: &mut UiTree, node: &SkillNode, select: bool) {
        if let Some(background) = self.node_child(tree, node, "Background") {
            tree.set_state(background, if select { id("selected") } else { STATELESS });
        }
    }

    /// `gmSkillTreeUI::SelectSkill`.
    fn select_skill(&mut self, tree: &mut UiTree, source: &dyn SkillSource, skill: u32) {
        if skill != self.selected_skill {
            if let Some(old) = self.nodes.get(&self.selected_skill).cloned() {
                self.select_node(tree, &old, false);
            }
        }
        self.selected_skill = 0;
        let Some(node) = self.nodes.get(&skill).cloned() else {
            return;
        };
        self.select_node(tree, &node, true);
        self.selected_skill = skill;
        self.update_selected_skill_display(tree, source, &node);
    }

    /// `gmSkillTreeUI::ClearSelectedInfo`.
    fn clear_selected_info(&mut self, tree: &mut UiTree) {
        if let Some(node) = self.nodes.get(&self.selected_skill).cloned() {
            self.select_node(tree, &node, false);
        }
        self.current_panel = 0;
        self.selected_skill = 0;
        for text in [self.selected_name, self.misc_text].into_iter().flatten() {
            tree.clear_text(text);
        }
        if let Some(icon) = self.selected_icon {
            tree.el_mut(icon).image = None;
            tree.changed = true;
        }
        if let Some(modify) = self.modify_field {
            tree.set_state(modify, STATELESS);
        }
    }

    /// `gmSkillTreeUI::UpdateSelectedSkillDisplay`.
    fn update_selected_skill_display(
        &mut self,
        tree: &mut UiTree,
        source: &dyn SkillSource,
        node: &SkillNode,
    ) {
        if let Some(icon) = self.selected_icon {
            tree.set_image(icon, node.icon, 2);
        }
        let Some(modify) = self.modify_field else {
            return;
        };
        let status = source.status(node.skill);
        let (xp, sp, hsp) = self.available;
        let ghost = |tree: &mut UiTree, name: &str, ghosted: bool| {
            if let Some(button) = tree.child_recursive(modify, id(name)) {
                tree.set_bool(button, BUTTON_GHOSTED, ghosted);
            }
        };
        tree.set_state(modify, STATELESS);
        let (token, value, cost, name);
        if !status.trained {
            tree.set_state(modify, id("untrained"));
            token = match (status.hero, status.can_train == CANT_TRAIN) {
                (_, true) => id("ID_Skill_CantTrain"),
                (false, false) => id("ID_Skill_AvailableSPTrain"),
                (true, false) => id("ID_Skill_AvailableHSPTrain"),
            };
            value = u64::from(if status.hero { hsp } else { sp });
            cost = status.cost_to_train;
            name = node.name.clone();
            ghost(tree, "TrainButton", status.can_train != 0);
            // `ID_Skill_TT_ClickToTrain` only where the skill can be trained.
            if status.can_train == 0 {
                let text = source.render(
                    id("ID_Skill_TT_ClickToTrain"),
                    &[(id("value"), SkillVar::UInt(0))],
                );
                set_button_tooltip(tree, modify, "TrainButton", text);
            }
        } else {
            if status.untraining {
                tree.set_state(modify, id("untraining"));
                token = id("ID_Skill_AvailableXPUntrain");
                cost = status.skill_xp;
            } else {
                tree.set_state(modify, id("trained"));
                token = if status.can_raise != 0 {
                    id("ID_Skill_CantRaise")
                } else {
                    id("ID_Skill_AvailableXPRaise")
                };
                cost = status.xp_to_level;
                ghost(tree, "RaiseButton", status.can_raise != 0);
                ghost(tree, "UntrainButton", status.can_untrain != 0);
                // The raise button's tooltip: the experience it takes, click or can't-raise text.
                let string = if status.can_raise == 0 {
                    "ID_Skill_TT_ClickToRaise"
                } else {
                    "ID_Skill_CantRaise"
                };
                let text = source.render(
                    id(string),
                    &[(id("value"), SkillVar::UInt(status.xp_to_level))],
                );
                set_button_tooltip(tree, modify, "RaiseButton", text);
            }
            value = xp;
            let (level, _) = Self::level_text(source, &status, true);
            name = source.render(
                id("ID_Skill_NameLevel"),
                &[
                    (id("value"), SkillVar::Text(node.name.clone())),
                    (id("level"), SkillVar::Text(level)),
                ],
            );
        }
        if let Some(misc) = self.misc_text {
            let text = source.render(
                token,
                &[
                    (id("value"), SkillVar::UInt(value)),
                    (id("cost"), SkillVar::UInt(cost)),
                ],
            );
            tree.set_text(misc, &text);
        }
        if let Some(selected) = self.selected_name {
            tree.set_text(selected, &name);
            let ellipsis = source.render(id("ID_Ellipsis"), &[]);
            truncate_text(tree, selected, &ellipsis);
        }
    }

    /// `gmSkillTreeUI::HandleElement`: a tab button asks for its race's root panel; the pressed
    /// one is highlighted and ghosted, the others not.
    fn handle_element(&mut self, tree: &mut UiTree, source: &dyn SkillSource, element: u32) {
        for (name, panel) in ELEMENT_PANELS {
            let pressed = element == id(name);
            if pressed {
                if let Some(root) = source.root_panel_from_race(panel) {
                    self.requests.push(SkillRequest::Examine {
                        kind: EXAMINE_PANEL,
                        data: root,
                    });
                }
            }
            if let Some(button) = tree.child_recursive(self.field, id(name)) {
                tree.set_bool(button, BUTTON_HIGHLIGHTED, pressed);
                tree.set_bool(button, BUTTON_GHOSTED, pressed);
            }
        }
    }

    /// `gmSkillTreeUI::MakeTrainDialog`: one train confirmation at a time.
    fn make_train_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        source: &dyn SkillSource,
        skill: u32,
    ) {
        if self.train.context != 0 {
            return;
        }
        let text = source.render(id("ID_Skill_TrainConfirm"), &[]);
        let context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
        self.train = Confirm { context, skill };
    }

    /// `gmSkillTreeUI::MakeUntrainDialog`: asked for a node the panel holds, naming the skill.
    fn make_untrain_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        source: &dyn SkillSource,
        skill: u32,
    ) {
        let Some(node) = self.nodes.get(&skill) else {
            return;
        };
        if self.untrain.context != 0 {
            return;
        }
        let text = source.render(
            id("ID_Skill_UntrainText"),
            &[(id("value"), SkillVar::Text(node.name.clone()))],
        );
        let context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
        self.untrain = Confirm { context, skill };
    }

    /// `gmSkillTreeUI::MakeUntrainCancelDialog`: one cancel confirmation at a time.
    fn make_cancel_dialog(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        source: &dyn SkillSource,
        skill: u32,
    ) {
        if self.cancel.context != 0 {
            return;
        }
        let text = source.render(id("ID_Skill_UntrainCancelConfirm"), &[]);
        let context = dialogs.make_dialog(tree, &text, DIALOG_CONFIRMATION, false);
        self.cancel = Confirm { context, skill };
    }

    /// `gmSkillTreeUI::Show`'s dialog part: the panel's shown state changed, so its dialogs
    /// close and their contexts clear. Called each frame; it acts only on a change.
    pub fn follow_show_state(&mut self, tree: &mut UiTree, dialogs: &mut DialogFactory) {
        let Some(field) = tree.get(self.field) else {
            return;
        };
        let shown = field.visible();
        if self.shown == Some(shown) {
            return;
        }
        self.shown = Some(shown);
        for confirm in [&mut self.train, &mut self.untrain, &mut self.cancel] {
            dialogs.close_dialog(tree, confirm.context);
            *confirm = Confirm::default();
        }
    }

    /// The confirmations' answers among the dialogs' outcomes
    /// (`gmSkillTreeUI::RecvNotice_CloseConfirmationDialog`).
    pub fn dialog_outcomes(&mut self, outcomes: &[DialogOutcome]) {
        for outcome in outcomes {
            if let DialogOutcome::Confirmation { context, yes } = *outcome {
                self.recv_close_confirmation_dialog(context, yes);
            }
        }
    }

    /// `gmSkillTreeUI::RecvNotice_CloseConfirmationDialog`: a yes asks the player for the skill
    /// the dialog was made for.
    fn recv_close_confirmation_dialog(&mut self, context: u32, yes: bool) {
        for (confirm, make) in [
            (
                &mut self.train,
                SkillRequest::Train as fn(u32) -> SkillRequest,
            ),
            (&mut self.untrain, SkillRequest::Untrain),
            (&mut self.cancel, SkillRequest::CancelUntrain),
        ] {
            if confirm.context == context && context != 0 {
                let skill = confirm.skill;
                *confirm = Confirm::default();
                if yes {
                    self.requests.push(make(skill));
                }
            }
        }
    }

    /// `gmSkillTreeUI::ListenToMessage`.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        dialogs: &mut DialogFactory,
        source: &dyn SkillSource,
        message: &UiMessage,
    ) {
        let element_id = message.element_id;
        let action = message.data as u32;
        if element_id == id("SkillNodeIcon")
            && message.message == MSG_MOUSE_PRESS_TOP
            && matches!(action, 0x4000_0004 | 0x4000_0005)
        {
            let skill = message.element.and_then(|e| {
                match tree.get(e)?.attribute(id("Token_ShortcutData")) {
                    Some(AttributeValue::Id(skill)) => Some(*skill),
                    _ => None,
                }
            });
            if let Some(node) = skill.and_then(|s| self.nodes.get(&s)) {
                let kind = if node.kind == NODE_SKILL {
                    EXAMINE_SKILL
                } else {
                    EXAMINE_PANEL
                };
                self.requests.push(SkillRequest::Examine {
                    kind,
                    data: node.skill,
                });
            }
        }
        let buttons = [
            "TrainButton",
            "UntrainButton",
            "RaiseButton",
            "CancelUntrainButton",
        ];
        if message.message == MSG_BUTTON_RELEASE_ON && buttons.iter().any(|b| id(b) == element_id) {
            let skill = self.selected_skill;
            if skill == 0 {
                return;
            }
            // `ListenToMessage`: train asks first unless the node is untrainable; untrain and
            // cancel ask; raise is asked at once.
            if element_id == id("TrainButton") {
                if source.status(skill).can_train != 0 {
                    return;
                }
                match self.nodes.get(&skill).map(|node| node.untrainable) {
                    Some(false) => self.make_train_dialog(tree, dialogs, source, skill),
                    Some(true) => self.requests.push(SkillRequest::Train(skill)),
                    None => {}
                }
            } else if element_id == id("UntrainButton") {
                self.make_untrain_dialog(tree, dialogs, source, skill);
            } else if element_id == id("RaiseButton") {
                self.requests.push(SkillRequest::Raise(skill));
            } else {
                self.make_cancel_dialog(tree, dialogs, source, skill);
            }
        }
        let tab = ELEMENT_PANELS
            .iter()
            .any(|(name, _)| id(name) == element_id)
            && message.message == MSG_MOUSE_PRESS;
        let craft = element_id == id("CraftTab") && message.message == MSG_OPENED;
        if tab || craft {
            self.handle_element(tree, source, element_id);
        }
    }
}

/// `UIElement::SetTooltip` on a named button under the modify field.
fn set_button_tooltip(tree: &mut UiTree, modify: usize, name: &str, text: String) {
    if let Some(button) = tree.child_recursive(modify, id(name)) {
        tree.set_tooltip(button, Some(text));
    }
}

/// `UIElement_Text::TruncateText`: while the text is wider than the element (its width plus
/// one), its last character goes and the ellipsis is added, until it fits. True when the text
/// was cut.
pub(crate) fn truncate_text(tree: &mut UiTree, index: usize, ellipsis: &str) -> bool {
    let Some(text) = tree.get(index).and_then(|e| e.text()) else {
        return false;
    };
    let limit = tree.get(index).map_or(0, |e| e.rect().width());
    let mut kept: Vec<char> = text.chars().collect();
    let mut shown = text.clone();
    while tree
        .text_line_width(index, &shown)
        .is_some_and(|w| limit + 1 < w)
    {
        if kept.pop().is_none() {
            break;
        }
        shown = format!("{}{ellipsis}", kept.iter().collect::<String>());
    }
    if shown != text {
        tree.set_text(index, &shown);
        return true;
    }
    false
}
