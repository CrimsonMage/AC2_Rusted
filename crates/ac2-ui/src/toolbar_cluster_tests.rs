//! The toolbar's corner cluster: the plan's geometry, the pointer's groups, and the cluster on
//! the real gameplay layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::element::UiBox;
use crate::mouse::{UI_POINTER_X, UI_POINTER_Y};
use crate::token::tokenize;
use crate::toolbar_cluster::{hover, plan, FIELD_HEIGHT, LEFT_GROUP, PLATE_IMAGE, RIGHT_GROUP};
use crate::{build_styled_hud, GameplayHud, HudStyle, ToolbarChild, UiTree};

const WIDTH: i32 = 1024;
const SHORTCUT: (i32, i32) = (348, 675);

fn inside(outer: &UiBox, inner: &UiBox) -> bool {
    inner.x0 >= outer.x0 && inner.x1 <= outer.x1 && inner.y0 >= outer.y0 && inner.y1 <= outer.y1
}

#[test]
fn the_plate_keeps_its_right_edge_and_grows_left_as_groups_open() {
    let closed = plan(WIDTH, SHORTCUT, [false; 2], [false; 2]);
    let right = plan(WIDTH, SHORTCUT, [false, true], [false; 2]);
    let both = plan(WIDTH, SHORTCUT, [true; 2], [false; 2]);
    assert_eq!(closed.plate.x1, right.plate.x1);
    assert_eq!(closed.plate.x1, both.plate.x1);
    assert_eq!(closed.plate.y1, FIELD_HEIGHT - 1 - 6);
    assert!(closed.plate.x1 < WIDTH - 1, "a margin from the corner");
    assert!(right.plate.x0 < closed.plate.x0);
    assert!(both.plate.x0 < right.plate.x0);
    // Opening the right group leaves the right group's own right edge where it was, so the
    // pointer over it stays over it.
    assert_eq!(
        closed.right.last().unwrap().0.x1 - closed.right.first().unwrap().0.x1,
        3 * 5
    );
    assert_eq!(
        right.right.first().unwrap().0.x0 + 3 * 30,
        right.right[3].0.x0
    );
    for p in [&closed, &right, &both] {
        for (at, _) in p.left.iter().chain(&p.right) {
            assert!(inside(&p.plate, at), "{at:?} on {:?}", p.plate);
        }
    }
}

#[test]
fn a_resting_group_stacks_its_first_button_on_top_and_an_open_one_lays_them_out() {
    let p = plan(WIDTH, SHORTCUT, [false, true], [false; 2]);
    // A lower z draws in front: the first button is on top of the stack.
    let z: Vec<i32> = p.left.iter().map(|&(_, z)| z).collect();
    assert!(z.windows(2).all(|w| w[0] < w[1]), "{z:?}");
    // Open: one z, and no two buttons overlap.
    assert!(p.right.windows(2).all(|w| w[0].1 == w[1].1));
    assert!(p.right.windows(2).all(|w| w[0].0.x1 < w[1].0.x0));
    assert_eq!(p.left.len(), LEFT_GROUP.len());
    assert_eq!(p.right.len(), RIGHT_GROUP.len());
}

#[test]
fn the_status_buttons_join_the_plate_only_while_shown() {
    let none = plan(WIDTH, SHORTCUT, [false; 2], [false; 2]);
    assert_eq!((none.consignments, none.dismount), (None, None));
    let both = plan(WIDTH, SHORTCUT, [true; 2], [true; 2]);
    let (consignments, dismount) = (both.consignments.unwrap(), both.dismount.unwrap());
    // The dismount button outermost, both on the plate, right of the groups.
    assert!(consignments.x1 < dismount.x0);
    assert!(inside(&both.plate, &consignments) && inside(&both.plate, &dismount));
    assert!(both.right.last().unwrap().0.x1 < consignments.x0);
    assert_eq!(none.plate.x1, both.plate.x1);
}

#[test]
fn the_attack_buttons_and_the_set_disc_flank_the_shortcut_bar() {
    let p = plan(WIDTH, SHORTCUT, [false; 2], [false; 2]);
    assert!(p.start_attack.x1 < SHORTCUT.0 && p.stop_attack.x0 < p.start_attack.x0);
    assert!(p.set_up.x0 > SHORTCUT.1);
    assert_eq!(
        (p.set_up.x0, p.set_up.y1 + 1),
        (p.set_down.x0, p.set_down.y0)
    );
    for at in [p.start_attack, p.stop_attack, p.set_down] {
        assert_eq!(at.y1, FIELD_HEIGHT - 1, "{at:?} on the bottom edge");
    }
    assert!(
        p.set_up.y0 >= 0 && p.stop_attack.y0 >= 0,
        "inside the field"
    );
}

#[test]
fn the_group_under_the_pointer_opens_and_both_fold_off_the_plate() {
    let closed = plan(WIDTH, SHORTCUT, [false; 2], [false; 2]);
    let centre = |b: &UiBox| ((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2);
    let over_right = centre(&closed.right[0].0);
    assert_eq!(hover(&closed, [false; 2], over_right), [false, true]);
    // Open, the right group pushed the left one left; over it, both are open.
    let right = plan(WIDTH, SHORTCUT, [false, true], [false; 2]);
    let over_left = centre(&right.left[0].0);
    assert_eq!(hover(&right, [false, true], over_left), [true, true]);
    // The open left group keeps the pointer over it (it grew away from it).
    let both = plan(WIDTH, SHORTCUT, [true; 2], [false; 2]);
    assert!(inside(
        &both.spans[0],
        &UiBox::new(over_left.0, over_left.1, over_left.0, over_left.1)
    ));
    // Off the plate, both fold.
    assert_eq!(hover(&both, [true; 2], (500, 10)), [false; 2]);
    assert_eq!(hover(&closed, [false; 2], (500, 10)), [false; 2]);
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    Some(build_styled_hud(&store, 1024, 768, HudStyle::Modern).unwrap())
}

fn root(tree: &UiTree, name: &str) -> usize {
    *tree
        .roots()
        .iter()
        .find(|&&r| tree.get(r).unwrap().id() == tokenize(name))
        .unwrap_or_else(|| panic!("{name}"))
}

fn element(tree: &UiTree, name: &str) -> usize {
    tree.child_recursive(root(tree, "ToolbarField"), tokenize(name))
        .unwrap_or_else(|| panic!("{name}"))
}

/// The pointer to `at`, its messages to the toolbar.
fn point(hud: &mut GameplayHud, at: (i32, i32)) {
    hud.tree.start_input(UI_POINTER_X, at.0 as f32);
    hud.tree.start_input(UI_POINTER_Y, at.1 as f32);
    for message in hud.tree.take_messages() {
        hud.toolbar.message(&mut hud.tree, &message);
    }
}

#[test]
fn the_toolbar_field_spans_the_bottom_edge_without_its_bar_or_the_pointer() {
    let Some(hud) = hud() else {
        return;
    };
    let tree = &hud.tree;
    let field = root(tree, "ToolbarField");
    assert_eq!(
        tree.screen_box(field),
        UiBox::new(0, 768 - FIELD_HEIGHT, 1023, 767)
    );
    assert_eq!(tree.get(field).unwrap().image(), None);
    for wing in ["LeftWingField", "RightWingField"] {
        assert!(!tree.get(element(tree, wing)).unwrap().visible());
    }
    let plate = element(tree, "ToolbarClusterPlate");
    assert_eq!(
        tree.get(plate).unwrap().image().map(|i| i.file),
        Some(PLATE_IMAGE)
    );
    let plate = tree.screen_box(plate);
    assert!(
        plate.x1 > 1000 && plate.y1 > 750,
        "{plate:?} in the bottom-right corner"
    );
    for child in ToolbarChild::ALL {
        let button = tree.screen_box(element(tree, &format!("{child:?}Button")));
        assert!(inside(&plate, &button), "{child:?} {button:?} on {plate:?}");
    }
    // The open world between the shortcut bar and the plate takes the pointer.
    let mut hud = hud;
    point(&mut hud, (800, 760));
    assert_eq!(
        hud.tree.mouse_top().map(|i| hud.tree.get(i).unwrap().id()),
        Some(tokenize("ItemSelectionField"))
    );
}

#[test]
fn hovering_a_group_spreads_its_buttons_and_leaving_folds_them() {
    let Some(mut hud) = hud() else {
        return;
    };
    let skill = |hud: &GameplayHud| hud.tree.screen_box(element(&hud.tree, "SkillButton"));
    let inventory = |hud: &GameplayHud| hud.tree.screen_box(element(&hud.tree, "InventoryButton"));
    let resting = skill(&hud).x0 - inventory(&hud).x0;
    assert_eq!(resting, 5);
    let at = inventory(&hud);
    point(&mut hud, (at.x0 + 3, at.y0 + 10));
    assert_eq!(hud.toolbar.open_groups(), [false, true]);
    assert_eq!(skill(&hud).x0 - inventory(&hud).x0, 30);
    // The spread buttons still toggle their panels.
    let skill_box = skill(&hud);
    point(&mut hud, (skill_box.x0 + 13, skill_box.y0 + 13));
    hud.tree
        .start_input(crate::window_locations::UI_LEFT_BUTTON, 1.0);
    hud.tree.stop_input(crate::window_locations::UI_LEFT_BUTTON);
    for message in hud.tree.take_messages() {
        hud.toolbar.message(&mut hud.tree, &message);
    }
    assert!(hud.toolbar.shown(&hud.tree, ToolbarChild::Skill));
    // Away from the plate, the group folds.
    point(&mut hud, (500, 300));
    assert_eq!(hud.toolbar.open_groups(), [false; 2]);
    assert_eq!(skill(&hud).x0 - inventory(&hud).x0, 5);
}

#[test]
fn a_shown_panel_lights_its_button_and_rests_on_the_plate() {
    let Some(mut hud) = hud() else {
        return;
    };
    let art = |hud: &GameplayHud| {
        hud.tree
            .get(element(&hud.tree, "InventoryButton"))
            .unwrap()
            .image()
            .map(|i| i.file)
    };
    let unlit = art(&hud);
    hud.toolbar
        .show(&mut hud.tree, ToolbarChild::Inventory, true);
    assert_ne!(art(&hud), unlit, "lit while the inventory shows");
    let panel = hud.tree.screen_box(root(&hud.tree, "RightSidePanel"));
    let plate = hud
        .tree
        .screen_box(element(&hud.tree, "ToolbarClusterPlate"));
    assert!(panel.y1 < plate.y0, "{panel:?} over {plate:?}");
    hud.toolbar
        .show(&mut hud.tree, ToolbarChild::Inventory, false);
    assert_eq!(art(&hud), unlit);
}

#[test]
fn the_status_buttons_take_their_place_on_the_plate_when_shown() {
    let Some(mut hud) = hud() else {
        return;
    };
    hud.toolbar.mount_state(&mut hud.tree, true);
    hud.toolbar.sale_reminder(&mut hud.tree, true);
    let plate = hud
        .tree
        .screen_box(element(&hud.tree, "ToolbarClusterPlate"));
    let dismount = hud.tree.screen_box(element(&hud.tree, "DismountButton"));
    let consignments = hud
        .tree
        .screen_box(element(&hud.tree, "ConsignmentsButton"));
    assert!(inside(&plate, &dismount) && inside(&plate, &consignments));
    assert!(consignments.x1 < dismount.x0);
    let social = hud.tree.screen_box(element(&hud.tree, "SocialButton"));
    assert!(social.x1 < consignments.x0, "the groups make room");
}
