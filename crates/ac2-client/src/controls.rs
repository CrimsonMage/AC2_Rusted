//! The controls' frame as the client core's input: `ac2_ui::Controls` resolves keys and mouse
//! through the retail key maps, action interpreter and camera UI; this turns its output into a
//! [`FrameInput`].

use crate::client::{CameraAction, FrameInput};
use crate::gameplay::Command;

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
        commands: stop_attacking_commands(controls.stop_attacking_if_auto_moving),
    }
}

/// `PlayerMovementModule::StopAttackingIfAutoMoving` -> `gmCWInterface::StopAttackingIfAutoMoving`
/// (the script's IFun), sent after the frame's movement.
fn stop_attacking_commands(started: bool) -> Vec<Command> {
    if started {
        vec![Command::StopAttackingIfAutoMoving]
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turn_key_start_queues_the_script_stop_only_when_it_starts() {
        let started = ac2_ui::ControlFrame {
            stop_attacking_if_auto_moving: true,
            ..Default::default()
        };
        assert_eq!(
            frame_input(started).commands,
            vec![Command::StopAttackingIfAutoMoving]
        );
        assert!(frame_input(ac2_ui::ControlFrame::default())
            .commands
            .is_empty());
    }
}
