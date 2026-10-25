//! `gmRadarUI`'s drawing on the real gameplay layout (`AC2_DAT_DIR`).

use ac2_db::ResourceStore;

use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};
use crate::{
    build_gameplay_hud, RadarDot, RadarRequest, RadarView, UiString, UiStrings, UI_POINTER_X,
    UI_POINTER_Y,
};

struct Strings;

impl UiStrings for Strings {
    fn render_string(&self, string: &UiString) -> String {
        format!("{:08x}", string.string_id)
    }
}

fn dot(id: u64, shape: u32, offset: [f32; 2]) -> RadarDot {
    RadarDot {
        id,
        color: [1.0, 0.0, 0.0, 1.0],
        shape,
        offset,
        in_range: true,
        name: format!("object {id}"),
    }
}

#[test]
fn the_radar_plots_its_blips_and_turns_its_heading() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = std::sync::Arc::new(ResourceStore::new(dataset));
    let mut hud = build_gameplay_hud(&store, 1024, 768).unwrap();
    let tree = &mut hud.tree;
    let field = hud.radar.field().expect("RadarField");
    let int = |name: &str| match tree.get(field).unwrap().attribute(tokenize(name)) {
        Some(AttributeValue::Int(v)) => *v,
        other => panic!("{name}: {other:?}"),
    };
    let (cx, cy, radius) = (
        int("RadarCenter_X"),
        int("RadarCenter_Y"),
        int("RadarRadius"),
    );
    eprintln!("centre ({cx}, {cy}), radius {radius}");
    assert!(radius > 0);
    // An X 8 m east, a point 20 m north and selected, one out of range, one outside the square.
    let view = RadarView {
        range: 75.0,
        heading_degrees: Some(37.6),
        dots: vec![
            dot(1, 0x4000_0004, [8.0, 0.0]),
            dot(2, 0x4000_0001, [0.0, 20.0]),
            RadarDot {
                in_range: false,
                ..dot(3, 0x4000_0001, [1.0, 1.0])
            },
            dot(4, 0x4000_0001, [0.0, -200.0]),
        ],
        selection: 2,
        coordinates: Some("12.3N, 45.6E".into()),
    };
    // The pointer over the X.
    let scale = f64::from((f64::from(radius) / 75.0) as f32);
    let east = cx + (8.0 * scale) as i32;
    let north = cy - (20.0 * scale) as i32;
    let (fx, fy) = tree.get(field).unwrap().object.unwrap_or((0, 0));
    tree.start_input(UI_POINTER_X, (fx + east + 1) as f32);
    tree.start_input(UI_POINTER_Y, (fy + cy + 2) as f32);
    hud.radar.update(tree, &view);
    let pixels: Vec<(i32, i32)> = hud.radar.pixels().iter().map(|p| (p.0, p.1)).collect();
    let red = [255, 0, 0, 255];
    assert!(hud.radar.pixels().iter().all(|p| p.2 == red));
    // The X: its centre and corners.
    for (dx, dy) in [(0, 0), (1, 1), (-1, -1), (-1, 1), (1, -1)] {
        assert!(pixels.contains(&(east + dx, cy + dy)), "{dx},{dy}");
    }
    // The selected point and its box; nothing from the other two.
    assert!(pixels.contains(&(cx, north)));
    assert!(pixels.contains(&(cx - 2, north + 3)) && pixels.contains(&(cx + 3, north - 2)));
    assert_eq!(pixels.len(), 5 + 1 + 4 * 5);
    // The X is under the pointer: BaseField's tooltip names it.
    let base = tree.child_recursive(field, tokenize("BaseField")).unwrap();
    assert_eq!(tree.tooltip_text(base).as_deref(), Some("object 1"));
    // The coordinates text.
    let coords = tree
        .child_recursive(field, tokenize("CoordinatesText"))
        .unwrap();
    assert_eq!(
        tree.get(coords).unwrap().text().as_deref(),
        Some("12.3N, 45.6E")
    );
    // The heading ring turned to the whole degrees.
    let heading = tree
        .roots()
        .iter()
        .copied()
        .find(|&r| tree.get(r).unwrap().id() == tokenize("HeadingField"))
        .unwrap();
    let placement = tree
        .placements()
        .into_iter()
        .find(|p| p.root == heading)
        .unwrap();
    assert_eq!(placement.rotation, 37);
    // Painted over the composed radar surface.
    let images = std::collections::HashMap::new();
    let mut surface = tree.compose_root(field, &images);
    hud.radar.draw(&mut surface);
    let at = ((cy as usize) * surface.width as usize + east as usize) * 4;
    assert_eq!(&surface.rgba[at..at + 4], &red);

    // A press on the base selects the X under the pointer.
    let press = UiMessage {
        element_id: tokenize("BaseField"),
        element: Some(base),
        message: MSG_MOUSE_PRESS_TOP,
        ..UiMessage::default()
    };
    assert_eq!(
        hud.radar.message(tree, &Strings, &press),
        Some(RadarRequest::Select(1))
    );
    // The options show the coordinates: the box opens; the button closes it and says so.
    hud.radar.reload_options(tree, &Strings, true);
    assert_eq!(tree.get(base).unwrap().state(), tokenize("open"));
    let button = tree
        .child_recursive(field, tokenize("CoordBoxButton"))
        .unwrap();
    let shown = format!("{:08x}", tokenize("ID_Radar_TT_CurLoc_Shown"));
    assert_eq!(tree.tooltip_text(button), Some(shown));
    let release = UiMessage {
        element_id: tokenize("CoordBoxButton"),
        element: Some(button),
        message: MSG_BUTTON_RELEASE_ON,
        ..UiMessage::default()
    };
    assert_eq!(
        hud.radar.message(tree, &Strings, &release),
        Some(RadarRequest::ShowCoords(false))
    );
    assert_eq!(tree.get(base).unwrap().state(), tokenize("closed"));
    let hidden = format!("{:08x}", tokenize("ID_Radar_TT_CurLoc_Hidden"));
    assert_eq!(tree.tooltip_text(button), Some(hidden));
}
