//! `FellowInfo`: one fellowship member's row (`FellowTemplate`): the name (in the `leader` state
//! while the member is the selection), health and vigor meters with their text, the PK damage and
//! vigor loss meters under them (their text hidden), the level and the share of the experience.

use crate::element::UiTree;
use crate::meter::VitalDisplay;
use crate::skill_tree::SkillVar;
use crate::token::tokenize;

/// A member as the server describes it (`FellowAdaptor`), its name rendered.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fellow {
    pub id: u64,
    pub name: String,
    pub level: u32,
    pub health: u32,
    pub max_health: u32,
    pub vigor: u32,
    pub max_vigor: u32,
    pub pk_damage: u32,
    pub pk_vigor_loss: u32,
    pub cell: u32,
}

/// The UI table's strings the rows render.
pub trait FellowStrings {
    /// `StringInfo::SetStringIDandTableEnum(string_id, 0x41000018)` with these variables.
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String;
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `ID_Vital_DisplayText` with `current` and `max`.
pub(crate) fn vital_text(strings: &dyn FellowStrings, cur: u64, max: u64) -> String {
    strings.render(
        id("ID_Vital_DisplayText"),
        &[
            (id("current"), SkillVar::UInt(cur)),
            (id("max"), SkillVar::UInt(max)),
        ],
    )
}

/// `FellowInfo`'s state.
#[derive(Debug, Clone)]
pub(crate) struct FellowInfo {
    pub id: u64,
    /// `_root`: the row in the list box.
    pub root: usize,
    pub name: String,
    pub cell: u32,
    name_text: Option<usize>,
    health: VitalDisplay,
    pk_damage: VitalDisplay,
    vigor: VitalDisplay,
    pk_vigor_loss: VitalDisplay,
    level_text: Option<usize>,
    percent_text: Option<usize>,
}

impl FellowInfo {
    /// `FellowInfo::FellowInfo`: the row's parts found and filled.
    pub fn new(
        tree: &mut UiTree,
        root: usize,
        fellow: &Fellow,
        strings: &dyn FellowStrings,
    ) -> Self {
        let child = |tree: &UiTree, name: &str| tree.child_recursive(root, id(name));
        let display = |tree: &UiTree, meter: &str, text: &str| {
            VitalDisplay::new(tree, child(tree, meter), child(tree, text))
        };
        let name_text = child(tree, "FellowTemplateNameText");
        if let Some(text) = name_text {
            tree.set_text(text, &fellow.name);
        }
        let vital = |cur, max| vital_text(strings, cur, max);
        let mut health = display(
            tree,
            "FellowTemplateHealthMeter",
            "FellowTemplateHealthText",
        );
        health.update(tree, fellow.health.into(), fellow.max_health.into(), &vital);
        let mut pk_damage = display(
            tree,
            "FellowTemplatePKDamageMeter",
            "FellowTemplatePKDamageText",
        );
        pk_damage.set_text_state(tree, false);
        pk_damage.update(
            tree,
            u64::from(fellow.pk_damage) + u64::from(fellow.health),
            fellow.max_health.into(),
            &vital,
        );
        let mut vigor = display(tree, "FellowTemplateVigorMeter", "FellowTemplateVigorText");
        vigor.update(tree, fellow.vigor.into(), fellow.max_vigor.into(), &vital);
        let mut pk_vigor_loss = display(
            tree,
            "FellowTemplatePKVigorlossMeter",
            "FellowTemplatePKVigorlossText",
        );
        pk_vigor_loss.set_text_state(tree, false);
        pk_vigor_loss.update(
            tree,
            u64::from(fellow.pk_vigor_loss) + u64::from(fellow.vigor),
            fellow.max_vigor.into(),
            &vital,
        );
        let mut info = Self {
            id: fellow.id,
            root,
            name: fellow.name.clone(),
            cell: fellow.cell,
            name_text,
            health,
            pk_damage,
            vigor,
            pk_vigor_loss,
            level_text: child(tree, "FellowTemplateLevelText"),
            percent_text: None,
        };
        if info.level_text.is_some() {
            info.update_level(tree, fellow.level, strings);
            info.percent_text = child(tree, "FellowTemplatePercentText");
            info.update_percentage(tree, 0.0, strings);
        }
        info
    }

    /// `FellowInfo::UpdateLevel`: `ID_Fellowship_Level` with `value`.
    pub fn update_level(&self, tree: &mut UiTree, level: u32, strings: &dyn FellowStrings) {
        if let Some(text) = self.level_text {
            let value = strings.render(
                id("ID_Fellowship_Level"),
                &[(id("value"), SkillVar::UInt(level.into()))],
            );
            tree.set_text(text, &value);
        }
    }

    /// `FellowInfo::UpdatePercentage`: cleared, then `ID_Fellowship_Percentage` with the share
    /// times 100 (precision 0) when there is one.
    pub fn update_percentage(&self, tree: &mut UiTree, share: f32, strings: &dyn FellowStrings) {
        let Some(text) = self.percent_text else {
            return;
        };
        tree.set_text(text, "");
        if 0.0 < share {
            let value = strings.render(
                id("ID_Fellowship_Percentage"),
                &[(
                    id("value"),
                    SkillVar::Float {
                        value: f64::from(share * 100.0),
                        precision: 0,
                    },
                )],
            );
            tree.set_text(text, &value);
        }
    }

    /// `FellowInfo::ToggleTextState`: health and vigor text shown or hidden.
    pub fn toggle_text_state(&self, tree: &mut UiTree) {
        self.vigor.toggle_text_state(tree);
        self.health.toggle_text_state(tree);
    }

    /// `FellowInfo::ShowSelectionHighlight`: the name in `leader` or `notleader`.
    pub fn show_selection_highlight(&self, tree: &mut UiTree, selected: bool) {
        if let Some(text) = self.name_text {
            tree.set_state(text, id(if selected { "leader" } else { "notleader" }));
            tree.set_text(text, &self.name);
        }
    }

    /// `FellowInfo::UpdateInfo`.
    pub fn update_info(&mut self, tree: &mut UiTree, fellow: &Fellow, strings: &dyn FellowStrings) {
        let vital = |cur, max| vital_text(strings, cur, max);
        self.health
            .update(tree, fellow.health.into(), fellow.max_health.into(), &vital);
        self.pk_damage.update(
            tree,
            u64::from(fellow.pk_damage) + u64::from(fellow.health),
            fellow.max_health.into(),
            &vital,
        );
        self.vigor
            .update(tree, fellow.vigor.into(), fellow.max_vigor.into(), &vital);
        self.pk_vigor_loss.update(
            tree,
            u64::from(fellow.pk_vigor_loss) + u64::from(fellow.vigor),
            fellow.max_vigor.into(),
            &vital,
        );
        self.level_text = tree.child_recursive(self.root, id("FellowTemplateLevelText"));
        self.update_level(tree, fellow.level, strings);
    }

    /// `RecvNotice_UpdateFellowHealth`: health and PK damage.
    pub fn update_health(&mut self, tree: &mut UiTree, value: u32, pk: u32, s: &dyn FellowStrings) {
        let vital = |cur, max| vital_text(s, cur, max);
        self.health.update_current(tree, value.into(), &vital);
        self.pk_damage.update_current(tree, pk.into(), &vital);
    }

    /// `RecvNotice_UpdateFellowMaxHealth`.
    pub fn update_max_health(&mut self, tree: &mut UiTree, value: u32, s: &dyn FellowStrings) {
        let vital = |cur, max| vital_text(s, cur, max);
        self.health.update_max(tree, value.into(), &vital);
        self.pk_damage.update_max(tree, value.into(), &vital);
    }

    /// `RecvNotice_UpdateFellowVigor`: vigor and PK vigor loss.
    pub fn update_vigor(&mut self, tree: &mut UiTree, value: u32, pk: u32, s: &dyn FellowStrings) {
        let vital = |cur, max| vital_text(s, cur, max);
        self.vigor.update_current(tree, value.into(), &vital);
        self.pk_vigor_loss.update_current(tree, pk.into(), &vital);
    }

    /// `RecvNotice_UpdateFellowMaxVigor`.
    pub fn update_max_vigor(&mut self, tree: &mut UiTree, value: u32, s: &dyn FellowStrings) {
        let vital = |cur, max| vital_text(s, cur, max);
        self.vigor.update_max(tree, value.into(), &vital);
        self.pk_vigor_loss.update_max(tree, value.into(), &vital);
    }
}
