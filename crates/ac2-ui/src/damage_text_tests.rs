//! `gm3DTextUI`'s floating damage numbers on the real gameplay layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::token::tokenize;
use crate::{build_gameplay_hud, DamagePlace, GameplayHud, UiPlacement, UiTree};
use ac2_weenie::DamageText;

/// The object's box, in its own space: a unit-ish box at the origin.
const BOUNDS: ([f32; 3], [f32; 3]) = ([-0.5, -0.5, 0.0], [0.5, 0.5, 2.0]);

/// World x, z straight onto clip space (y is depth), as the name plate tests use.
fn project(p: [f32; 3]) -> ([f32; 2], bool) {
    ([p[0] / 10.0, p[2] / 10.0], true)
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

fn text(text: &str, duration: f32) -> DamageText {
    DamageText {
        text: text.into(),
        color: [1.0, 0.0, 0.0, 1.0],
        duration,
        scale: 1.0,
    }
}

/// The damage text's root, the only one with its template.
fn damage_root(tree: &UiTree) -> Option<usize> {
    tree.roots().iter().copied().find(|&r| {
        tree.get(r)
            .is_some_and(|e| e.id() == tokenize("DamageTextTemplate"))
    })
}

fn placed(tree: &UiTree, root: usize) -> UiPlacement {
    *tree
        .placements()
        .iter()
        .find(|p| p.root == root)
        .expect("the damage text is placed")
}

#[test]
fn damage_text_rises_and_fades_over_its_object_and_is_destroyed_at_its_end() {
    let Some(mut hud) = hud() else {
        return;
    };
    let travel = hud.reticle.as_ref().expect("the reticle").floaty_travel();
    assert!(travel > 0, "FloatyTextTravel on the reticle");
    let damage = hud.damage.as_mut().expect("the damage texts");
    let tree = &mut hud.tree;
    assert!(damage.add(tree, 7, &text("120", 2.0), 10.0));
    let root = damage_root(tree).expect("a DamageTextTemplate");
    let shown_text = tree.get(root).unwrap().text();
    assert_eq!(shown_text.as_deref(), Some("120"));
    let colour = tree
        .get(root)
        .unwrap()
        .attribute(tokenize("font_color_red"));
    assert!(colour.is_some(), "the blob's red");
    assert_eq!(damage.objects(), [7]);
    // Hidden until a frame places it.
    assert!(!placed(tree, root).shown);

    let places = [DamagePlace {
        id: 7,
        bounds: BOUNDS,
        project: &project,
    }];
    damage.update(tree, &places, (0, 0), (0, 0), 10.0, false);
    let start = placed(tree, root);
    assert!(start.shown);
    assert_eq!(start.opacity, 1.0);

    // Half its duration: fades to one half and has risen by height * travel * one half.
    damage.update(tree, &places, (0, 0), (0, 0), 11.0, false);
    let half = placed(tree, root);
    assert!((half.opacity - 0.5).abs() < 1e-6, "{}", half.opacity);
    let h = tree.get(root).unwrap().rect.height();
    assert_eq!(half.x, start.x);
    assert_eq!(
        start.y - half.y,
        (f64::from(h) * f64::from(travel) * 0.5) as i32
    );

    // Its duration spent: destroyed.
    damage.update(tree, &places, (0, 0), (0, 0), 12.0, false);
    assert!(damage.is_empty());
    assert!(damage_root(tree).is_none());
}

#[test]
fn a_selected_objects_damage_text_rises_above_the_reticle_and_name() {
    let Some(mut hud) = hud() else {
        return;
    };
    let offset = hud
        .reticle
        .as_ref()
        .expect("the reticle")
        .selection_offset(&hud.tree);
    assert!(offset > 0, "the selection's reticle and name");
    let damage = hud.damage.as_mut().expect("the damage texts");
    let tree = &mut hud.tree;
    assert!(damage.add(tree, 7, &text("80", 2.0), 0.0));
    let root = damage_root(tree).unwrap();
    let places = [DamagePlace {
        id: 7,
        bounds: BOUNDS,
        project: &project,
    }];
    damage.update(tree, &places, (0, 0), (0, 0), 0.0, false);
    let plain = placed(tree, root);
    damage.update(tree, &places, (7, offset), (0, 0), 0.0, false);
    let selected = placed(tree, root);
    assert_eq!(plain.y - selected.y, offset);
}

#[test]
fn damage_texts_are_placed_clear_of_each_other_and_hidden_in_portal_space() {
    let Some(mut hud) = hud() else {
        return;
    };
    assert!(hud.reticle.as_ref().unwrap().name_tuning().1 > 0);
    let damage = hud.damage.as_mut().expect("the damage texts");
    let tree = &mut hud.tree;
    assert!(damage.add(tree, 7, &text("120", 2.0), 0.0));
    assert!(damage.add(tree, 7, &text("45", 2.0), 0.0));
    let places = [DamagePlace {
        id: 7,
        bounds: BOUNDS,
        project: &project,
    }];
    damage.update(tree, &places, (0, 0), (0, 0), 0.0, false);
    let roots: Vec<usize> = tree
        .roots()
        .iter()
        .copied()
        .filter(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("DamageTextTemplate"))
        })
        .collect();
    assert_eq!(roots.len(), 2);
    let boxes: Vec<_> = roots
        .iter()
        .map(|&r| {
            let p = placed(tree, r);
            let rect = tree.get(r).unwrap().rect;
            assert!(p.shown);
            (p.x, p.y, rect.width(), rect.height())
        })
        .collect();
    // The second moved off the first's spot; the layout's attempts may run out before the two
    // are clear of each other (as retail's do).
    assert_ne!(
        (boxes[0].0, boxes[0].1),
        (boxes[1].0, boxes[1].1),
        "{boxes:?}"
    );

    // Portal space hides them; the update after it places them again.
    damage.update(tree, &places, (0, 0), (0, 0), 0.5, true);
    assert!(roots.iter().all(|&r| !placed(tree, r).shown));
    damage.update(tree, &places, (0, 0), (0, 0), 0.5, false);
    assert!(roots.iter().all(|&r| placed(tree, r).shown));

    // `CleanUp`: every text destroyed.
    damage.clear(tree);
    assert!(damage.is_empty());
    assert!(damage_root(tree).is_none());
}
