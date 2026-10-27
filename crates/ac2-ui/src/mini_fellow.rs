//! `gmMiniFellowUI`: the fellowship mini panel (`FellowMiniPanel`, a root of the gameplay layout),
//! the members' names and vitals in `FellowMiniPanelListBox`. It starts hidden; the
//! `FellowshipMiniButton` shows or hides it (`gmMiniFellowUI::ListenToMessage`). Its member rows
//! are a [`FellowRows`] over the gameplay layout's `FellowMiniPanelTemplate`
//! (`FellowshipHelper` mode 1): no level, no XP share, health and vigor numbers hidden at first.
//!
//! Not modelled yet: the saved panel location (`UIFramework::LocationApplied` loads nothing, and
//! the location change is not sent, `CM_UI::SendNotice_SaveLocationsChanged`).

use std::sync::Arc;

use crate::element::UiTree;
use crate::fellow_rows::{FellowNotice, FellowRows};
use crate::fellowship::{FellowshipData, FellowshipSource};
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmMiniFellowUI`'s state.
#[derive(Debug, Clone)]
pub struct MiniFellowUi {
    layout: Arc<LayoutDesc>,
    /// `_miniPanelField`: the `FellowMiniPanel` root.
    panel: Option<usize>,
    /// The framework's shown state (`UIFramework::Show` / `IsShown`).
    shown: bool,
    /// `_fellowshipHelper`: the members, `None` without a fellowship.
    helper: Option<FellowRows>,
}

impl MiniFellowUi {
    /// `gmMiniFellowUI::gmMiniFellowUI`: the `FellowMiniPanel` root from the gameplay layout
    /// (`CreateAndAddRootElement(0x41000009, FellowMiniPanel)`), then `Show(0)`.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Self {
        let panel = tree.create_root(Arc::clone(gameplay), id("FellowMiniPanel"));
        let mut ui = Self {
            layout: Arc::clone(gameplay),
            panel,
            shown: false,
            helper: None,
        };
        ui.show(tree, false);
        ui
    }

    /// Whether the panel is shown (`UIFramework::IsShown`).
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// `gmMiniFellowUI::Show`: the framework's shown state, then the panel follows it.
    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.shown = shown;
        if let Some(panel) = self.panel {
            tree.set_visible(panel, shown);
        }
    }

    /// `gmMiniFellowUI::LocationApplied`: the framework's location is applied (none saved yet),
    /// then the shown state is applied again.
    pub fn location_applied(&mut self, tree: &mut UiTree) {
        self.show(tree, self.shown);
    }

    /// `gmMiniFellowUI::ListenToMessage`: the `FellowshipMiniButton`'s release toggles the panel.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage) {
        if message.message == MSG_BUTTON_RELEASE_ON
            && message.element_id == id("FellowshipMiniButton")
        {
            let shown = !self.shown;
            self.show(tree, shown);
        }
    }

    /// The members' rows, top first, and their ids.
    pub fn rows(&self, tree: &UiTree) -> Vec<(u64, usize)> {
        self.helper.as_ref().map_or_else(Vec::new, |h| h.rows(tree))
    }

    /// `gmMiniFellowUI::RecvNotice_UpdateFellowship`: the panel's state is `fellowship`, the old
    /// helper goes, and a new one lists the members over `FellowMiniPanelListBox`.
    pub fn update_fellowship(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        data: &FellowshipData,
    ) {
        let Some(panel) = self.panel else {
            return;
        };
        if tree.get(panel).map(|e| e.state()) != Some(id("fellowship")) {
            tree.set_state(panel, id("fellowship"));
        }
        if let Some(mut old) = self.helper.take() {
            old.remove_all(tree);
        }
        let Some(list) = tree.child_recursive(panel, id("FellowMiniPanelListBox")) else {
            return;
        };
        let mut rows = FellowRows::new(list, Arc::clone(&self.layout), true);
        rows.populate(tree, source, &data.fellows, data.leader);
        self.helper = Some(rows);
    }

    /// `gmMiniFellowUI::RecvNotice_ClearFellowship`: the helper goes and the panel hides.
    pub fn clear_fellowship(&mut self, tree: &mut UiTree) {
        if let Some(mut old) = self.helper.take() {
            old.remove_all(tree);
        }
        self.show(tree, false);
    }

    /// A member-level notice for the mini panel's list (the helper's `RecvNotice_*`).
    pub fn fellow_notice(
        &mut self,
        tree: &mut UiTree,
        source: &dyn FellowshipSource,
        notice: &FellowNotice,
    ) {
        if let Some(helper) = &mut self.helper {
            helper.notice(tree, source, notice);
        }
    }
}
