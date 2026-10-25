//! The controls' frame as the client core's input: `ac2_ui::Controls` resolves keys and mouse
//! through the retail key maps, action interpreter and camera UI; this turns its output into a
//! [`FrameInput`].

use crate::client::{CameraAction, FrameInput};

/// A [`FrameInput`] from one frame of controls.
pub fn frame_input(controls: ac2_ui::ControlFrame) -> FrameInput {
    FrameInput {
        actions: controls.actions,
        camera_actions: controls
            .camera_actions
            .into_iter()
            .map(|(action, extent, start)| CameraAction {
                action,
                extent,
                start,
            })
            .collect(),
        head: None,
        mouse_look: controls.mouse_look,
        mouse_look_state: controls.mouse_look_state,
        first_person_state: controls.first_person_state,
        behaviors: controls.behaviors,
        interrupt_move_to: controls.interrupt_move_to,
    }
}
