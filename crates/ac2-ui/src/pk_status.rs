//! `gmPKStatusUI`: the status bar's PK indicator (`PKStatusField`). Its state shows the player's
//! faction status: `stateless`, `peace`, `control`, `factional` or `ffa`. The admin 7148 build's
//! `RecvNotice_UpdateFactionStatus` maps the notice's status to that state. The Legions 7160
//! decompile carries no such handler (its constructor still registers the notices), so the
//! mapping here is admin's and no notice feeds it yet.

use crate::element::UiTree;
use crate::token::tokenize;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmPKStatusUI`'s state.
#[derive(Debug, Clone)]
pub struct PkStatusUi {
    field: Option<usize>,
}

impl PkStatusUi {
    /// `gmPKStatusUI::gmPKStatusUI`: the field (an `IUIManager` lookup of `PKStatusField`).
    pub fn create(tree: &UiTree) -> Self {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("PKStatusField")));
        Self { field }
    }

    /// The field's state.
    pub fn state(&self, tree: &UiTree) -> Option<u32> {
        Some(tree.get(self.field?)?.state())
    }

    /// `gmPKStatusUI::RecvNotice_UpdateFactionStatus`: status 1 is `peace`, 2 `control`, 3
    /// `factional` and 4 `ffa`; any other status is `stateless`.
    pub fn update_faction_status(&mut self, tree: &mut UiTree, status: u32) {
        let state = match status {
            1 => id("peace"),
            2 => id("control"),
            3 => id("factional"),
            4 => id("ffa"),
            _ => id("stateless"),
        };
        if let Some(field) = self.field {
            tree.set_state(field, state);
        }
    }
}
