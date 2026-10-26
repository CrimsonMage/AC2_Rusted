//! `gmVitaeStatusUI`: the status bar's vitae indicator (`VitaeStatusField`). A player at full
//! vitae (100%) has the field in its `stateless` state and the full-vitae tooltip; below that it
//! is `notfull`, the tooltip tells the current vitae, how much a payment of experience restores
//! and what it costs (`ID_Indicator_TT_VitaeStatus` with the variables `cur`, `inc` and `xp`,
//! each truncated by `_ftol2`). Pressing a `notfull` field examines the vitae
//! (`ExaminationRequest` type `0x4000000A`), which shows the field's `Overlay` while that request
//! stands.

use crate::element::UiTree;
use crate::fellow_info::FellowStrings;
use crate::mouse::UiMessage;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `ExaminationRequest::_type` of the vitae examination.
pub const EXAMINE_VITAE: u32 = 0x4000_000A;

/// `gmVitaeStatusUI`'s state.
#[derive(Debug, Clone)]
pub struct VitaeStatusUi {
    field: Option<usize>,
    overlay: Option<usize>,
}

impl VitaeStatusUi {
    /// `gmVitaeStatusUI::gmVitaeStatusUI`: the field (an `IUIManager` lookup), its `Overlay`
    /// child hidden.
    pub fn create(tree: &mut UiTree) -> Self {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("VitaeStatusField")));
        let overlay = field.and_then(|f| tree.child_recursive(f, id("Overlay")));
        if let Some(overlay) = overlay {
            tree.set_visible(overlay, false);
        }
        Self { field, overlay }
    }

    /// The field's state (`stateless` at full vitae, else `notfull`).
    pub fn state(&self, tree: &UiTree) -> Option<u32> {
        Some(tree.get(self.field?)?.state())
    }

    /// `RecvNotice_UpdateVitae`: the state and the tooltip for `vitae` percent, the `increment`
    /// a payment restores and its experience `cost`.
    pub fn update_vitae(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn FellowStrings,
        vitae: f32,
        increment: f32,
        cost: f32,
    ) {
        let (state, text) = if 100.0 <= vitae {
            (
                id("stateless"),
                strings.render(id("ID_Indicator_TT_FullVitae"), &[]),
            )
        } else {
            // `_ftol2` truncates.
            let int = |name: &str, value: f32| (id(name), SkillVar::Int(value as i64));
            let vars = [int("cur", vitae), int("inc", increment), int("xp", cost)];
            (
                id("notfull"),
                strings.render(id("ID_Indicator_TT_VitaeStatus"), &vars),
            )
        };
        if let Some(field) = self.field {
            tree.set_state(field, state);
            tree.set_tooltip(field, Some(text));
        }
    }

    /// `RecvNotice_ExaminationRequestChanged`: the overlay shows while the vitae is examined and
    /// the indicator is not full.
    pub fn examination_request_changed(&mut self, tree: &mut UiTree, kind: u32) {
        let show = kind == EXAMINE_VITAE && self.state(tree) == Some(id("notfull"));
        if let Some(overlay) = self.overlay {
            tree.set_visible(overlay, show);
        }
    }

    /// `ListenToMessage`: a left or right press on a `notfull` field asks to examine the vitae
    /// (`SetSelection(INVALID_IID)`, `SetExaminationRequest`, `OpenExaminationPanel`); whether it
    /// did.
    pub fn message(&mut self, tree: &UiTree, message: &UiMessage) -> bool {
        message.message == MSG_MOUSE_PRESS_TOP
            && message.element_id == id("VitaeStatusField")
            && self.field.is_some()
            && self.state(tree) == Some(id("notfull"))
            && matches!(message.data as u32, 0x4000_0004 | 0x4000_0005)
    }
}
