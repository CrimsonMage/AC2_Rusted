//! The modern style's stack dialog (AGENTS.md: the HUD's two styles): `gmStackUI`'s
//! `StackableField` laid out as the design's popover on the drawer's glass. Its title and the
//! requester's question over a row of a slider beside the amount's entry, the error line under
//! them, then the all / half toggle, the most allowed and OK and Cancel. The slider is the
//! client's: dragging it writes the amount into `AmountEntry` (`gmStackUI::SetAmount`, as the
//! toggle does) and typing moves it, so OK reads the entry as retail does (`wcstoul`, base 0) and
//! answers with `StackUIDone`, its errors unchanged.

use std::cell::Cell;
use std::sync::Arc;

use crate::compose::NINE_SLICE;
use crate::element::{UiBox, UiTree};
use crate::layout::LayoutDesc;
use crate::modern_art as art;
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part, remap_art};
use crate::mouse::{UiMessage, GLOBAL_MOUSE_MOVE, GLOBAL_MOUSE_UP};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE, MSG_MOUSE_PRESS_TOP};

/// The popover's size and padding.
const SIZE: (i32, i32) = (256, 140);
const PAD: i32 = 10;
/// The slider row: its top, height, and the entry's width beside the slider.
const ROW: (i32, i32) = (52, 24);
const ENTRY: i32 = 64;
/// The buttons' row and size.
const BUTTONS: i32 = 100;
const BUTTON: i32 = 28;
/// The retail dialog's frame and entry box art, and the client's that stand for them.
const ART: [(u32, u32); 2] = [(0x4100_08B6, art::BEVEL), (0x4100_08B7, art::ENTRY_BOX)];
/// `StackableField`'s unnamed children: its drag bar, its static title ("How many?") and the
/// entry's box.
const DRAG_BAR: u32 = 0x89CB_0E70;
const TITLE: u32 = 0x49A3_6817;
const ENTRY_BOX: u32 = 0x2A0F_42F6;

/// The popover's parts beside retail's.
#[derive(Debug, Clone)]
pub(crate) struct ModernStack {
    track: Option<usize>,
    fill: Option<usize>,
    knob: Option<usize>,
    max_label: Option<usize>,
    entry: Option<usize>,
    /// Whether the knob is being dragged.
    dragging: Cell<bool>,
    /// The amount the knob shows.
    shown: Cell<Option<(u32, u32)>>,
}

fn place(tree: &mut UiTree, element: Option<usize>, (x, y, w, h): (i32, i32, i32, i32)) {
    let Some(element) = element else {
        return;
    };
    keep(tree, element);
    tree.resize_to(element, w, h);
    tree.move_to(element, x, y);
}

impl ModernStack {
    /// The dialog `field` laid out as the popover and the slider made in it.
    pub(crate) fn adopt(tree: &mut UiTree, field: usize, layouts: &[&Arc<LayoutDesc>]) -> Self {
        let child = |tree: &UiTree, id: u32| tree.child_recursive(field, id);
        remap_art(tree, field, &ART, true);
        tree.set_int(field, NINE_SLICE, art::BEVEL_BORDER);
        if let Some(entry_box) = child(tree, ENTRY_BOX) {
            tree.set_int(entry_box, NINE_SLICE, art::ENTRY_BORDER);
        }
        place(tree, Some(field), (0, 0, SIZE.0, SIZE.1));
        let width = SIZE.0 - 2 * PAD;
        place(tree, child(tree, DRAG_BAR), (0, 0, SIZE.0, 28));
        let title = child(tree, TITLE);
        place(tree, title, (PAD + 4, 6, width - 4, 18));
        let question = child(tree, tokenize("Label"));
        place(tree, question, (PAD, 30, width, 18));
        let slider = width - ENTRY - 6;
        place(
            tree,
            child(tree, ENTRY_BOX),
            (PAD + slider + 6, ROW.0, ENTRY, ROW.1),
        );
        // The title and the question start at the popover's left, as the design sets them.
        for text in [title, question].into_iter().flatten() {
            if let Some(text) = tree.el_mut(text).text.as_mut() {
                text.justification = Justification::Left;
            }
        }
        let entry = child(tree, tokenize("AmountEntry"));
        place(tree, entry, (4, 2, ENTRY - 8, ROW.1 - 4));
        if let Some(text) = entry.and_then(|e| tree.el_mut(e).text.as_mut()) {
            text.justification = Justification::Right;
        }
        place(
            tree,
            child(tree, tokenize("ErrorText")),
            (PAD, 80, width, 16),
        );
        place(
            tree,
            child(tree, tokenize("AmountToggleButton")),
            (PAD, BUTTONS, BUTTON, BUTTON),
        );
        place(
            tree,
            child(tree, tokenize("StackOkButton")),
            (SIZE.0 - PAD - 2 * BUTTON - 6, BUTTONS, BUTTON, BUTTON),
        );
        place(
            tree,
            child(tree, tokenize("StackCancelButton")),
            (SIZE.0 - PAD - BUTTON, BUTTONS, BUTTON, BUTTON),
        );
        // The slider: the template is the dialog's drag bar (a plain field).
        let template = child(tree, DRAG_BAR);
        let bar = |tree: &mut UiTree, id: &str, image: u32, z: i32| {
            let part = make_part(
                tree,
                field,
                template,
                id,
                image,
                Some(art::SLIDER_BORDER),
                z,
            )?;
            keep(tree, part);
            Some(part)
        };
        let track_y = ROW.0 + (ROW.1 - 6) / 2;
        let track = bar(tree, "ModernStackTrack", art::SLIDER_TRACK, 3);
        place(tree, track, (PAD, track_y, slider, 6));
        let fill = bar(tree, "ModernStackFill", art::SLIDER_FILL, 2);
        place(tree, fill, (PAD, track_y, 6, 6));
        let knob = make_part(
            tree,
            field,
            template,
            "ModernStackKnob",
            art::SLIDER_KNOB,
            None,
            1,
        );
        place(
            tree,
            knob,
            (PAD, ROW.0 + (ROW.1 - art::KNOB) / 2, art::KNOB, art::KNOB),
        );
        // The track and the knob take presses.
        for part in [track, knob].into_iter().flatten() {
            tree.set_bool(part, MOUSE_INVISIBLE, false);
        }
        let max_label = count_label(tree, layouts, field, "ModernStackMax", Justification::Left);
        if let Some(label) = max_label {
            keep(tree, label);
            for (channel, value) in [("red", 0xD9), ("green", 0xC2), ("blue", 0x7A)] {
                let _ = tree.set_float(
                    label,
                    tokenize(&format!("font_color1_{channel}")),
                    value as f32 / 255.0,
                );
            }
        }
        place_label(
            tree,
            max_label,
            UiBox::new(
                PAD + BUTTON + 6,
                BUTTONS,
                PAD + BUTTON + 120,
                BUTTONS + BUTTON - 1,
            ),
        );
        Self {
            track,
            fill,
            knob,
            max_label,
            entry,
            dragging: Cell::new(false),
            shown: Cell::new(None),
        }
    }

    /// The most allowed shown (`max N`) and the knob set for the dialog just shown.
    pub(crate) fn init(&self, tree: &mut UiTree, max: u32) {
        if let Some(label) = self.max_label {
            tree.set_text_with_font(label, &format!("max {max}"), 0, 1);
        }
        self.dragging.set(false);
        self.shown.set(None);
        self.sync(tree, max);
    }

    /// The knob and the filled part set for the entry's amount (as `OK` would read it).
    pub(crate) fn sync(&self, tree: &mut UiTree, max: u32) {
        let text = self
            .entry
            .and_then(|e| tree.get(e))
            .and_then(|e| e.text())
            .unwrap_or_default();
        let Some(amount) = crate::stack::read_uint32(&text) else {
            return;
        };
        let amount = amount.min(max);
        if self.shown.get() == Some((amount, max)) {
            return;
        }
        self.shown.set(Some((amount, max)));
        let (Some(track), Some(knob)) = (self.track, self.knob) else {
            return;
        };
        let t = tree.el(track).rect;
        let share = if max == 0 {
            0.0
        } else {
            f64::from(amount) / f64::from(max)
        };
        let x = t.x0 + (share * f64::from(t.width())).round() as i32;
        let k = tree.el(knob).rect;
        tree.move_to(knob, x - art::KNOB / 2, k.y0);
        if let Some(fill) = self.fill {
            tree.resize_to(fill, (x - t.x0).max(6), t.height());
        }
    }

    /// A press on the track or knob starts a drag, the pointer moving it sets the amount, a
    /// release ends it. The amount the slider sets, if it set one.
    pub(crate) fn message(&self, tree: &UiTree, message: &UiMessage, max: u32) -> Option<u32> {
        match message.message {
            MSG_MOUSE_PRESS_TOP
                if message.element.is_some()
                    && (message.element == self.track || message.element == self.knob) =>
            {
                self.dragging.set(true);
            }
            GLOBAL_MOUSE_MOVE if self.dragging.get() => {}
            GLOBAL_MOUSE_UP => {
                self.dragging.set(false);
                return None;
            }
            _ => return None,
        }
        let track = tree.screen_box(self.track?);
        let x = (tree.pointer().0 - track.x0).clamp(0, track.width());
        let amount = (f64::from(x) / f64::from(track.width().max(1)) * f64::from(max)).round();
        Some(amount as u32)
    }
}
