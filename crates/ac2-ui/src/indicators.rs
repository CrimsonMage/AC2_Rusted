//! `gmIndicatorsUI`: the status bar's indicators framework, the third child of `gmStatusbarUI`
//! after `gmVitalsUI` and `gmEffectsUI`. Its `SetupChildren` makes the net status
//! (`gmNetStatusUI`), the PK status (`gmPKStatusUI`) and the vitae indicator (`gmVitaeStatusUI`)
//! and keeps them in a list. Only the vitae indicator is ported so far, so only it is made here.
//! The framework has no root element of its own, so its `Show` only records the shown flag.

use crate::element::UiTree;
use crate::vitae::VitaeStatusUi;

/// `gmIndicatorsUI`'s children that are ported.
#[derive(Debug, Clone)]
pub struct IndicatorsUi {
    /// `gmVitaeStatusUI`, the vitae indicator.
    pub vitae: VitaeStatusUi,
}

impl IndicatorsUi {
    /// `gmIndicatorsUI::gmIndicatorsUI` (its `SetupChildren`): the vitae indicator is made. The
    /// net status and PK status children are not (their frameworks are not ported yet).
    pub fn create(tree: &mut UiTree) -> Self {
        Self {
            vitae: VitaeStatusUi::create(tree),
        }
    }
}
