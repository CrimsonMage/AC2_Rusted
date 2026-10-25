//! `gmSelectionUI`: clicking the world selects. `ListenToMessage` takes `ItemSelectionField`'s
//! `mouse_press_top` for the left button (`0x40000004`), its double click (`0x40000007`) and the
//! right button (`0x40000005`): the first starts a 3D selection (`Start3DSelection`, the cursor
//! as a point) and later presses before it finishes add a double or right click.
//! `Finish3DSelection` then selects what the selection render picked (or clears the selection
//! when nothing was), uses it on a double click, and opens the examination panel on a right
//! click with something selected.
//!
//! Not modelled yet: the selectability checks (`0x270` on the target, the player's `0x14`),
//! `CWPhysicsObject::LookAtSelection`, and the double click's check that the target is not the
//! player's own item (`0x105`).

use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_MOUSE_PRESS_TOP};

const LEFT: u32 = 0x4000_0004;
const RIGHT: u32 = 0x4000_0005;
const DOUBLE: u32 = 0x4000_0007;

/// A finished 3D selection's clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionClick {
    pub double_click: bool,
    pub right_click: bool,
}

/// What `Finish3DSelection` does with the picked object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionOutcome {
    /// `gmCWInterface::SetSelection` (`None`: nothing picked, the selection clears).
    pub select: Option<u64>,
    /// `gmCWInterface::UseSelection`.
    pub use_selection: bool,
    /// `CM_UI::SendNotice_OpenExaminationPanel`.
    pub open_examination: bool,
}

/// `gmSelectionUI`'s state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionUi {
    selecting: bool,
    click: SelectionClick,
}

impl SelectionUi {
    /// `gmSelectionUI::ListenToMessage`.
    pub fn message(&mut self, message: &UiMessage) {
        let field = tokenize("ItemSelectionField");
        let action = message.data as u32;
        if message.element_id != field
            || message.message != MSG_MOUSE_PRESS_TOP
            || !matches!(action, LEFT | DOUBLE | RIGHT)
        {
            return;
        }
        if !self.selecting {
            // `Start3DSelection`.
            self.selecting = true;
        }
        self.click.double_click |= action == DOUBLE;
        self.click.right_click |= action == RIGHT;
    }

    /// A 3D selection waits for the selection render (`WorldRender::m_wantSelection`).
    pub fn wants_pick(&self) -> bool {
        self.selecting
    }

    /// `gmSelectionUI::Finish3DSelection` with the render's pick.
    pub fn finish(&mut self, picked: Option<u64>) -> SelectionOutcome {
        let click = std::mem::take(&mut self.click);
        self.selecting = false;
        SelectionOutcome {
            select: picked,
            use_selection: click.double_click,
            open_examination: click.right_click && picked.is_some(),
        }
    }
}
