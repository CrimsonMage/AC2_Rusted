//! `gmSelectionUI`: clicking the world selects. `ListenToMessage` takes `ItemSelectionField`'s
//! `mouse_press_top` for the left button (`0x40000004`), its double click (`0x40000007`) and the
//! right button (`0x40000005`): the first starts a 3D selection (`Start3DSelection`, the cursor
//! as a point) and later presses before it finishes add a double or right click.
//! `Finish3DSelection` then selects the first of the potential list the selection render picked
//! (or clears the selection when nothing was), uses it on a double click, and opens the
//! examination panel on a right click with something selected. A first target that cannot be
//! selected (`0x270` false) and the player's type (`0x14`) without the selectable bits (`0x3f`)
//! ends the pick with nothing selected and the clicks still pending, as retail does.
//!
//! A double click does not use a target the player wields (`0x105` is the player's id).
//! `CWPhysicsObject::LookAtSelection` follows each `SetSelection` in the client (`CLookAt`).

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

/// One entry of the potential list, as `Finish3DSelection` reads the object: its `0x270`
/// selectability, or `None` when the object has no record (the checks are skipped), and the
/// wielder it records (`0x105`, 0 for none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PotentialTarget {
    pub id: u64,
    pub selectable: Option<bool>,
    pub wielder: u64,
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

    /// `gmSelectionUI::Finish3DSelection` with the render's potential list (nearest first) and
    /// the player's type (`0x14`) and id. `None` when the first target cannot be selected: nothing
    /// changes and the clicks stay pending.
    pub fn finish(
        &mut self,
        potential: &[PotentialTarget],
        player_type: u32,
        player: u64,
    ) -> Option<SelectionOutcome> {
        self.selecting = false;
        let first = potential.first().copied();
        if let Some(PotentialTarget {
            selectable: Some(false),
            ..
        }) = first
        {
            // Only the player's selectable bits (`0x3f`) let an unselectable target through.
            if player_type & 0x3f != 0x3f {
                return None;
            }
        }
        let click = std::mem::take(&mut self.click);
        Some(SelectionOutcome {
            select: first.map(|target| target.id),
            // `GetInstanceID(0x105)` of the target is the player: its own wielded item is not used.
            use_selection: click.double_click
                && !first
                    .is_some_and(|target| target.selectable.is_some() && target.wielder == player),
            open_examination: click.right_click && first.is_some(),
        })
    }
}
