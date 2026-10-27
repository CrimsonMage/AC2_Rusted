//! `gmStatusbarUI::Show` and the indicators' vitae on the real gameplay layout (`AC2_DAT_DIR`).

use crate::statusbar::{show_statusbar, statusbar_field};
use crate::{build_gameplay_hud, GameplayHud};

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

#[test]
fn the_status_bar_shows_and_hides_its_field_and_the_indicators_hold_the_vitae() {
    let Some(mut hud) = hud() else {
        return;
    };
    // `gmStatusbarUI::gmStatusbarUI` shows the bar once its children are made.
    let field = statusbar_field(&hud.tree).expect("StatusbarField");
    assert!(hud.tree.get(field).unwrap().visible());
    show_statusbar(&mut hud.tree, false);
    assert!(!hud.tree.get(field).unwrap().visible());
    show_statusbar(&mut hud.tree, true);
    assert!(hud.tree.get(field).unwrap().visible());
    // `gmIndicatorsUI`'s vitae indicator is made on the status bar's field.
    assert!(hud.indicators.vitae.state(&hud.tree).is_some());
}
