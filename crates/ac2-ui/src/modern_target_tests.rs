//! The modern style's selection marks (`modern_target`): brackets, the name's card, slim meters
//! and the other names' pills, with retail's placement.

use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::hud_style::HudStyle;
use crate::modern_art_target as art;
use crate::token::tokenize;
use crate::UiTree;

fn root_named(tree: &UiTree, name: &str) -> usize {
    tree.roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == tokenize(name))
        .unwrap_or_else(|| panic!("{name}"))
}

#[test]
fn the_modern_selection_is_bracketed_and_carded() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = crate::build_styled_hud(&store, 1024, 768, HudStyle::Modern).unwrap();
    let tree = &mut hud.tree;
    let mut reticle = hud.reticle.take().expect("the reticle");
    reticle.selection_changed(tree, Some(7), Some("Gnome"), None);
    reticle.selection_info(
        tree,
        &ac2_net::protocol::SelectionInfo {
            kind: ac2_net::protocol::SelectionInfo::CREATURE,
            cur_health: 50,
            pk_damage: 0,
            max_health: 100,
            cur_vigor: 100,
            pk_vigor_loss: 0,
            max_vigor: 100,
        },
    );
    let project = |p: [f32; 3]| ([p[0], p[1]], true);
    let target = crate::ReticleTarget {
        id: 7,
        bounds: ([-0.1, -0.1, 0.0], [0.1, 0.1, 0.0]),
        project: &project,
        planar_distance: Some(10.0),
    };
    reticle.update(tree, Some(target), false);
    // The corners where retail puts them, each its own bracket, unturned.
    let corners: Vec<usize> = tree
        .roots()
        .iter()
        .copied()
        .filter(|&r| tree.get(r).unwrap().id() == tokenize("TargettingReticle"))
        .collect();
    let mut seen: Vec<(i32, i32, u32)> = corners
        .iter()
        .map(|&c| {
            let e = tree.get(c).unwrap();
            assert_eq!(e.rotation, 0);
            let (x, y) = e.object.unwrap();
            (x, y, e.image().unwrap().file - art::BRACKETS)
        })
        .collect();
    seen.sort();
    assert_eq!(
        seen,
        vec![(441, 325, 0), (441, 421, 1), (563, 325, 2), (563, 421, 3)]
    );
    // The meters are the design's: slim, its width.
    let stuff = root_named(tree, "SelectionStuff");
    let meters = tree.get(stuff).unwrap().rect;
    assert_eq!((meters.width(), meters.height()), (art::METER_WIDTH, 11));
    // The card goes round the name and the meters, behind them.
    let card = root_named(tree, "ModernSelectionCard");
    let name = root_named(tree, "SelectionName");
    let at = |tree: &UiTree, r: usize| {
        let e = tree.get(r).unwrap();
        let (x, y) = e.object.unwrap();
        (x, y, x + e.rect.width(), y + e.rect.height(), e.shown)
    };
    let (cx0, cy0, cx1, cy1, shown) = at(tree, card);
    assert!(shown);
    for part in [name, stuff] {
        let (x0, y0, x1, y1, shown) = at(tree, part);
        assert!(shown);
        assert!(cx0 < x0 && cy0 <= y0 && x1 < cx1 && y1 < cy1);
    }
    assert!(tree.get(card).unwrap().z > tree.get(name).unwrap().z);
    // It is part of the selection's box the name plates keep clear of.
    let rect = reticle.selection_rect().unwrap();
    assert!(rect[0] <= cx0 && rect[1] <= cy0);
    // Out of range, the card goes with the rest.
    reticle.update(tree, None, false);
    assert!(!tree.get(card).unwrap().shown);
}

#[test]
fn the_classic_selection_keeps_retails_turned_gems() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let hud = crate::build_styled_hud(&store, 1024, 768, HudStyle::Classic).unwrap();
    assert!(!hud
        .tree
        .roots()
        .iter()
        .any(|&r| hud.tree.get(r).unwrap().id() == tokenize("ModernSelectionCard")));
    let stuff = root_named(&hud.tree, "SelectionStuff");
    assert_eq!(hud.tree.get(stuff).unwrap().rect.width(), 100);
}

#[test]
fn other_names_sit_in_glass_pills() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(dataset));
    let mut hud = crate::build_styled_hud(&store, 1024, 768, HudStyle::Modern).unwrap();
    let tree = &mut hud.tree;
    let mut names = hud.names.take().expect("the name plates");
    let look = crate::name_plates::NameLook {
        id: 8,
        name: "Gatekeeper Ilsa".into(),
        color: [0.5, 0.84, 1.0, 1.0],
    };
    names.set_showing(tree, true, std::slice::from_ref(&look));
    let plate = names.plate(8).unwrap();
    let e = tree.get(plate).unwrap();
    assert_eq!(e.image().map(|i| i.file), Some(art::NAME_PILL));
    // The pill's room either side of the name.
    let bare = tree.text_line_width(plate, "Gatekeeper Ilsa").unwrap();
    assert!(e.rect.width() >= bare);
    assert_eq!(e.text().as_deref(), Some("Gatekeeper Ilsa"));
}
