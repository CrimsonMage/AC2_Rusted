//! `gmIndicatorsUI`: the status bar's indicators framework, the third child of `gmStatusbarUI`
//! after `gmVitalsUI` and `gmEffectsUI`. Its `SetupChildren` makes the net status
//! (`gmNetStatusUI`), the PK status (`gmPKStatusUI`) and the vitae indicator (`gmVitaeStatusUI`)
//! and keeps them in a list. The framework has no root element of its own, so its `Show` only
//! records the shown flag.

use crate::element::UiTree;
use crate::net_status::NetStatusUi;
use crate::pk_status::PkStatusUi;
use crate::vitae::VitaeStatusUi;

/// `gmIndicatorsUI`'s children.
#[derive(Debug, Clone)]
pub struct IndicatorsUi {
    /// `gmNetStatusUI`, the net indicator.
    pub net: NetStatusUi,
    /// `gmPKStatusUI`, the PK indicator.
    pub pk: PkStatusUi,
    /// `gmVitaeStatusUI`, the vitae indicator.
    pub vitae: VitaeStatusUi,
}

impl IndicatorsUi {
    /// `gmIndicatorsUI::gmIndicatorsUI` (its `SetupChildren`): the net status, the PK status and
    /// the vitae indicator are made in that order.
    pub fn create(tree: &mut UiTree) -> Self {
        Self {
            net: NetStatusUi::create(tree),
            pk: PkStatusUi::create(tree),
            vitae: VitaeStatusUi::create(tree),
        }
    }
}
