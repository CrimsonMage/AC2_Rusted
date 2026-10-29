//! The modern additions without a session, as the world viewer drives them (`ClientView::
//! offline_frameworks`) on the real DATs (`AC2_DAT_DIR`): the quest guide and the Tracker's
//! window open on the HUD, and a press and typing reach the guide's search field. Skipped without
//! a GPU adapter.

#![cfg(feature = "window")]

use std::sync::Arc;
use std::time::Duration;

use ac2_client::view::ClientView;
use ac2_client::{ClientFrame, DatWorld};
use ac2_gpu::{Gpu, COLOR_FORMAT};
use ac2_ui::Modifiers;

const SIZE: (u32, u32) = (1024, 768);
/// `UIElementManager::StartInput`'s pointer axes and left button.
const UI_POINTER_X: u32 = 0x4000_0001;
const UI_POINTER_Y: u32 = 0x4000_0002;
const UI_LEFT_BUTTON: u32 = 0x4000_0004;

fn pump(gpu: &Gpu, view: &mut ClientView, frame: &ClientFrame) {
    view.update(gpu, frame, Duration::ZERO);
    view.offline_frameworks(frame);
    view.prepare(gpu, frame, SIZE.0 as f32 / SIZE.1 as f32, 0.0);
}

#[test]
fn the_viewer_opens_the_guide_and_the_tracker_and_types_into_the_search() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        return;
    };
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
    let dat_world = DatWorld::new(store).unwrap();
    let world = Arc::clone(dat_world.world());
    let frame = ClientFrame {
        in_world: true,
        cell_id: Some(0xCB4D_0025),
        ..ClientFrame::default()
    };
    let mut view = ClientView::new(world, &gpu, COLOR_FORMAT, 1, SIZE);
    view.update(&gpu, &frame, Duration::ZERO);
    view.settle(&gpu, &frame, SIZE.0 as f32 / SIZE.1 as f32);
    pump(&gpu, &mut view, &frame);
    let closed = view.hud_layers();

    view.toggle_guide();
    pump(&gpu, &mut view, &frame);
    assert_eq!(view.hud_layers(), closed + 1, "the guide's window is drawn");
    view.toggle_tracker();
    pump(&gpu, &mut view, &frame);
    assert_eq!(
        view.hud_layers(),
        closed + 2,
        "the Tracker's window is drawn"
    );

    // A press in the search field gives it the typing; the list follows the query.
    let field = view
        .element_box("GuideSearchInput")
        .expect("the guide's search field");
    let (x, y) = ((field.x0 + field.x1) / 2, (field.y0 + field.y1) / 2);
    view.ui_input(
        &[
            (UI_POINTER_X, x as f32, true),
            (UI_POINTER_Y, y as f32, true),
            (UI_LEFT_BUTTON, 1.0, true),
        ],
        Modifiers::default(),
    );
    view.ui_input(&[(UI_LEFT_BUTTON, 1.0, false)], Modifiers::default());
    pump(&gpu, &mut view, &frame);
    assert!(view.text_mode(), "the search field takes the typing");
    let typed: Vec<(u16, u32)> = "tusk".encode_utf16().map(|ch| (ch, 0)).collect();
    view.text_input(&[], &typed, Modifiers::default());
    pump(&gpu, &mut view, &frame);
    assert_eq!(
        view.element_text("GuideSearchInput").as_deref(),
        Some("tusk")
    );

    view.toggle_guide();
    view.toggle_tracker();
    pump(&gpu, &mut view, &frame);
    assert_eq!(view.hud_layers(), closed, "both close again");
}
