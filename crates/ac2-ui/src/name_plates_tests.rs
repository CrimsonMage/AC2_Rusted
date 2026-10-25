//! `gm3DTextUI`'s name plates on the real gameplay layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};
use crate::{build_gameplay_hud, NameLook, NamePlace};

fn look(id: u64, name: &str) -> NameLook {
    NameLook {
        id,
        name: name.into(),
        color: [1.0, 0.0, 0.0, 1.0],
    }
}

/// World x, z straight onto clip space: x from -10..10, z from -10..10 (y is depth).
fn project(p: [f32; 3]) -> ([f32; 2], bool) {
    ([p[0] / 10.0, p[2] / 10.0], true)
}

#[test]
fn name_plates_sit_over_their_objects_and_keep_clear_of_each_other() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let names = hud.names.as_mut().expect("the name plates");
    let looks = [look(1, "Creature"), look(2, "Creature"), look(3, "Scout")];
    // Off until the options show names.
    names.sync(tree, &looks);
    assert!(names.named().is_empty());
    names.set_showing(tree, true, &looks);
    assert_eq!(names.named(), [1, 2, 3]);
    let plate = names.plate(1).unwrap();
    assert_eq!(tree.get(plate).unwrap().text().as_deref(), Some("Creature"));
    let (w, h) = {
        let r = tree.get(plate).unwrap().rect;
        (r.width(), r.height())
    };
    assert!(w > 0 && h > 0);

    // 1 and 2 stand at the same place: 2 is moved off 1. 3 is out of range.
    let bounds = ([-0.5, -0.5, 0.0], [0.5, 0.5, 2.0]);
    let places = [
        NamePlace {
            id: 1,
            bounds,
            project: &project,
            xy_distance_sq: 25.0,
        },
        NamePlace {
            id: 2,
            bounds,
            project: &project,
            xy_distance_sq: 25.0,
        },
        NamePlace {
            id: 3,
            bounds,
            project: &project,
            xy_distance_sq: 100.0 * 100.0,
        },
    ];
    names.update(tree, &places, 0, None, 75.0 * 75.0, (0, 0), false);
    fn at(tree: &crate::UiTree, names: &crate::NamePlates, id: u64) -> (i32, i32, bool) {
        let root = names.plate(id).unwrap();
        tree.placements()
            .into_iter()
            .find(|p| p.root == root)
            .map(|p| (p.x, p.y, p.shown))
            .unwrap()
    }
    let (x1, y1, shown1) = at(tree, names, 1);
    let (x2, y2, shown2) = at(tree, names, 2);
    assert!(shown1 && shown2);
    assert!(!at(tree, names, 3).2, "out of range");
    // Centred over the box's top: x = 512 - w / 2; the box top is clip 0.2 (y up).
    assert_eq!(x1, (512.0 - f64::from(w as u32) * 0.5) as i32);
    let top = ((1.0 - (1.0 / 768.0 + 0.2)) * 768.0 * 0.5) as i32;
    assert_eq!(y1, top - h);
    assert!(x2 != x1 || y2 != y1, "the second plate moved off the first");
    // The selection has no plate.
    names.update(tree, &places, 1, None, 75.0 * 75.0, (0, 0), false);
    assert!(!at(tree, names, 1).2);
    // A press on a plate selects its object.
    let press = UiMessage {
        element_id: tokenize("NameTextTemplate"),
        element: Some(names.plate(2).unwrap()),
        message: MSG_MOUSE_PRESS_TOP,
        ..UiMessage::default()
    };
    assert_eq!(names.message(tree, &press), Some(2));
    // A changed look is made again; a gone object loses its plate.
    names.sync(
        tree,
        &[
            look(1, "Creature"),
            NameLook {
                color: [0.0, 1.0, 0.0, 1.0],
                ..look(3, "Scout")
            },
        ],
    );
    assert_eq!(names.named(), [1, 3]);
    names.set_showing(tree, false, &[]);
    assert!(names.named().is_empty());
}
