use ac2_weenie::GameplayNotice;

use crate::element::UiTree;
use crate::pk_status::PkStatusUi;
use crate::status_notices::pk_notice;
use crate::tokenize;

/// The PK field takes each faction status's state, as the notice carries it (the Legions
/// `gmPKStatusUI::RecvNotice_UpdateFactionStatus`, at 0x0044E290, maps 1 to 4 and any other
/// value to `stateless`).
#[test]
fn the_faction_notice_sets_the_pk_field_state() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    let mut hud = crate::build_gameplay_hud(&store, 1024, 768).unwrap();
    let mut pk = Some(hud.indicators.pk.clone());
    let field = |tree: &UiTree, pk: &Option<PkStatusUi>| pk.as_ref().and_then(|p| p.state(tree));
    let state = |name: &str| tokenize(name);
    pk_notice(
        &mut pk,
        &mut hud.tree,
        &GameplayNotice::UpdateFactionStatus(2),
    );
    assert_eq!(field(&hud.tree, &pk), Some(state("control")));
    pk_notice(
        &mut pk,
        &mut hud.tree,
        &GameplayNotice::UpdateFactionStatus(4),
    );
    assert_eq!(field(&hud.tree, &pk), Some(state("ffa")));
    pk_notice(
        &mut pk,
        &mut hud.tree,
        &GameplayNotice::UpdateFactionStatus(9),
    );
    assert_eq!(field(&hud.tree, &pk), Some(state("stateless")));
}
