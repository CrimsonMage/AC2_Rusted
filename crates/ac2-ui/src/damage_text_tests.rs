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

/// The root's composed surface (`UIRegion::ForceDraw`) and how many of its pixels are inked.
fn composed(tree: &mut UiTree, root: usize) -> (u32, u32, usize) {
    let surfaces = tree.compose(&std::collections::HashMap::new());
    let surface = surfaces
        .iter()
        .find(|s| s.root == root)
        .expect("the damage text's surface");
    let ink = surface.rgba.chunks_exact(4).filter(|p| p[3] > 0).count();
    (surface.width, surface.height, ink)
}

fn scaled(text: &str, scale: f32) -> DamageText {
    DamageText {
        scale,
        ..self::text(text, 2.0)
    }
}

/// `RecvNotice_Do3DDamage`: the element takes the measured size (`ResizeTo`), the surface is shown
/// at that size times the blob's scale (`SetScreenPosition`'s view size), and the lettering is
/// drawn at the scale (retail stretched the small surface; here it is composed at that size), so a
/// scale of 2 draws about four times the ink. The rise is the shown height's.
#[test]
fn a_damage_text_is_drawn_at_its_blobs_scale() {
    let Some(mut hud) = hud() else {
        return;
    };
    let travel = hud.reticle.as_ref().unwrap().floaty_travel();
    let damage = hud.damage.as_mut().expect("the damage texts");
    assert_eq!(
        damage.size(),
        1.0,
        "retail's size until the client sets one"
    );
    let tree = &mut hud.tree;
    assert!(damage.add(tree, 7, &scaled("120", 1.0), 10.0));
    let small = damage_root(tree).unwrap();
    assert!(damage.add(tree, 8, &scaled("120", 2.0), 10.0));
    let large = *tree
        .roots()
        .iter()
        .rev()
        .find(|&&r| {
            r != small
                && tree
                    .get(r)
                    .is_some_and(|e| e.id() == tokenize("DamageTextTemplate"))
        })
        .unwrap();
    let (w, h) = {
        let rect = tree.get(small).unwrap().rect();
        (rect.width(), rect.height())
    };
    assert!(w > 0 && h > 0);
    // The template's own font (`impact_24`, read before the text data froze) took the blob's
    // colour: a font that fails to load leaves the default font and its white.
    assert_eq!(
        tree.get(small).unwrap().text_color(),
        Some([1.0, 0.0, 0.0, 1.0])
    );
    assert!(h > 20, "impact_24's height, {h}");
    assert_eq!(
        tree.get(large).unwrap().rect(),
        tree.get(small).unwrap().rect()
    );
    assert_eq!(damage.view_size(7), Some((w, h)));
    assert_eq!(damage.view_size(8), Some((w * 2, h * 2)));

    let (sw, sh, small_ink) = composed(tree, small);
    let (lw, lh, large_ink) = composed(tree, large);
    assert_eq!((sw, sh), (w as u32, h as u32));
    assert_eq!((lw, lh), (2 * w as u32, 2 * h as u32));
    assert!(small_ink > 0);
    let ratio = large_ink as f64 / small_ink as f64;
    assert!(
        (3.0..5.0).contains(&ratio),
        "ink {small_ink} -> {large_ink}"
    );

    let places = [8u64].map(|id| DamagePlace {
        id,
        bounds: BOUNDS,
        project: &project,
    });
    damage.update(tree, &places, (0, 0), (0, 0), 10.0, false);
    let start = placed(tree, large);
    damage.update(tree, &places, (0, 0), (0, 0), 11.0, false);
    let half = placed(tree, large);
    assert_eq!(
        start.y - half.y,
        (f64::from(2 * h) * f64::from(travel) * 0.5) as i32
    );
}

/// The client's damage text size preference multiplies the blob's scale.
#[test]
fn the_damage_text_size_multiplies_the_blobs_scale() {
    let Some(mut hud) = hud() else {
        return;
    };
    let damage = hud.damage.as_mut().expect("the damage texts");
    let tree = &mut hud.tree;
    assert!(damage.add(tree, 7, &scaled("45", 1.0), 0.0));
    let (w, h) = damage.view_size(7).unwrap();
    damage.clear(tree);
    damage.set_size(1.5);
    assert!(damage.add(tree, 7, &scaled("45", 2.0), 0.0));
    assert_eq!(damage.view_size(7), Some((w * 3, h * 3)));
    let root = damage_root(tree).unwrap();
    let (sw, sh, _) = composed(tree, root);
    assert_eq!((sw, sh), (3 * w as u32, 3 * h as u32));
    // A bad size is retail's.
    damage.set_size(f64::NAN);
    assert_eq!(damage.size(), 1.0);
}
