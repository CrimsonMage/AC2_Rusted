//! Resize grips: `UIElement_Resizebar` (a child of a window, on one of its edges) and the window
//! resize it starts, `UIElement::StartResizing`, `UIElement::MouseResizeElement` and
//! `UIElement::StopResizing`.
//!
//! A left press on an enabled grip starts resizing its parent window (`UIElement_Resizebar::MouseDown`):
//! the grip's `border_left` / `border_right` / `border_top` / `border_bottom` attributes pick the
//! edges, and the corner grips are the two edges together. On each mouse move the window's box
//! moves by the pointer's change since the last move, one edge per dragged side, and each edge is
//! kept within the window's `min_width`, `max_width`, `min_height` and `max_height`; the box is
//! then kept on the display (`MouseResizeElement`). The window is moved and resized to the box,
//! which lays its children out again (`ResizeTo`). A window with `resize_line` shows only an
//! outline while the button is held: its place and size change at the release (`StopResizing`).
//!
//! The window's size is part of its saved place ([`UiTree::window_positions`]), so a resized
//! window opens at its size again (`UIElement::ApplyLocation` resizes when `allow_resize` is set).

use crate::element::{UiBox, UiTree};
use crate::token::{
    BORDER_BOTTOM, BORDER_LEFT, BORDER_RIGHT, BORDER_TOP, BROADCAST_LOC_CHANGE, MAX_HEIGHT,
    MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, RESIZEBAR, RESIZEBAR_DISABLED, RESIZE_LINE, SAVE_LOC,
};
use crate::window_locations::UI_LEFT_BUTTON;

/// `BorderLocation`: the edge or corner a grip drags. The numbers are the enum's values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Border {
    /// `BORDER_UL` (1).
    UpperLeft = 1,
    /// `BORDER_TOP` (2).
    Top = 2,
    /// `BORDER_UR` (3).
    UpperRight = 3,
    /// `BORDER_RIGHT` (4).
    Right = 4,
    /// `BORDER_LR` (5).
    LowerRight = 5,
    /// `BORDER_BOTTOM` (6).
    Bottom = 6,
    /// `BORDER_LL` (7).
    LowerLeft = 7,
    /// `BORDER_LEFT` (8).
    Left = 8,
}

/// One side of a box a border moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Top,
    Right,
    Bottom,
}

impl Border {
    /// The edges the border moves: a corner moves its two sides.
    fn edges(self) -> &'static [Edge] {
        match self {
            Border::UpperLeft => &[Edge::Top, Edge::Left],
            Border::Top => &[Edge::Top],
            Border::UpperRight => &[Edge::Right, Edge::Top],
            Border::Right => &[Edge::Right],
            Border::LowerRight => &[Edge::Bottom, Edge::Right],
            Border::Bottom => &[Edge::Bottom],
            Border::LowerLeft => &[Edge::Left, Edge::Bottom],
            Border::Left => &[Edge::Left],
        }
    }
}

/// `UIElement_Resizebar::MouseDown`'s choice of border from the grip's four attributes. The
/// test reads the right edge first: a grip with `border_right` never considers `border_left`.
/// `None` when no edge is set (the press does nothing).
pub fn border_of(left: bool, right: bool, top: bool, bottom: bool) -> Option<Border> {
    if !right {
        if !left {
            if !top {
                return bottom.then_some(Border::Bottom);
            }
            return Some(Border::Top);
        }
        if !top {
            return Some(if bottom {
                Border::LowerLeft
            } else {
                Border::Left
            });
        }
        return Some(Border::UpperLeft);
    }
    if !top {
        return Some(if bottom {
            Border::LowerRight
        } else {
            Border::Right
        });
    }
    Some(Border::UpperRight)
}

/// A window's min and max size (`min_width`, `max_width`, `min_height`, `max_height`); `None` is
/// an attribute the element does not set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeLimits {
    pub min_width: Option<i32>,
    pub max_width: Option<i32>,
    pub min_height: Option<i32>,
    pub max_height: Option<i32>,
}

/// `UIElement::MouseResizeElement`'s clamp of one moved left edge: at least `min_width` wide and
/// at most `max_width` (the box's right edge `x1` stays).
fn clamp_left(x0: i32, x1: i32, limits: SizeLimits) -> i32 {
    let mut x0 = x0;
    if let Some(min) = limits.min_width {
        x0 = x0.min(x1 - min + 1);
    }
    if let Some(max) = limits.max_width {
        x0 = x0.max(x1 - max + 1);
    }
    x0
}

fn clamp_right(x1: i32, x0: i32, limits: SizeLimits) -> i32 {
    let mut x1 = x1;
    if let Some(min) = limits.min_width {
        x1 = x1.max(x0 + min - 1);
    }
    if let Some(max) = limits.max_width {
        x1 = x1.min(x0 + max - 1);
    }
    x1
}

fn clamp_top(y0: i32, y1: i32, limits: SizeLimits) -> i32 {
    let mut y0 = y0;
    if let Some(min) = limits.min_height {
        y0 = y0.min(y1 - min + 1);
    }
    if let Some(max) = limits.max_height {
        y0 = y0.max(y1 - max + 1);
    }
    y0
}

fn clamp_bottom(y1: i32, y0: i32, limits: SizeLimits) -> i32 {
    let mut y1 = y1;
    if let Some(min) = limits.min_height {
        y1 = y1.max(y0 + min - 1);
    }
    if let Some(max) = limits.max_height {
        y1 = y1.min(y0 + max - 1);
    }
    y1
}

/// `UIElement::MouseResizeElement`'s new box: the `start` box with the border's edges moved by
/// the pointer's change `(dx, dy)`, each clamped by `limits`, then kept on a `display` of
/// `width` x `height` pixels. Every edge moves from the start box, so a corner's two edges and the
/// clamps read the same values retail reads.
pub fn resized_box(
    border: Border,
    start: UiBox,
    (dx, dy): (i32, i32),
    limits: SizeLimits,
    display: (i32, i32),
) -> UiBox {
    let mut b = start;
    for edge in border.edges() {
        match edge {
            Edge::Left => b.x0 = clamp_left(start.x0 + dx, start.x1, limits),
            Edge::Top => b.y0 = clamp_top(start.y0 + dy, start.y1, limits),
            Edge::Right => b.x1 = clamp_right(start.x1 + dx, start.x0, limits),
            Edge::Bottom => b.y1 = clamp_bottom(start.y1 + dy, start.y0, limits),
        }
    }
    // The display's edges (`MouseResizeElement`, last).
    b.x0 = b.x0.max(0);
    b.x1 = b.x1.min(display.0 - 1);
    b.y0 = b.y0.max(0);
    b.y1 = b.y1.min(display.1 - 1);
    b
}

/// A resize in progress (`UIElement::m_resizing`, `m_currentBorder`, the mouse offsets, and the
/// outline's rectangle when the window has `resize_line`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Resizing {
    /// The window being resized (the grip's parent).
    pub(crate) window: usize,
    pub(crate) border: Border,
    /// The pointer at the last move (`m_mouseXOffset`, `m_mouseYOffset`).
    pub(crate) pointer: (i32, i32),
    /// The window's box when the resize started (its change is what the release reports).
    pub(crate) start: UiBox,
    /// The outline's box for a `resize_line` window; applied at the release.
    pub(crate) outline: Option<UiBox>,
}

impl UiTree {
    /// `UIElement_Resizebar::MouseDown`: a left press on an enabled grip starts resizing the
    /// grip's parent by the edges the grip's attributes name, unless that window is moving or
    /// resizing already.
    pub(crate) fn resizebar_mouse_down(&mut self, index: usize, action: u32) {
        if action != UI_LEFT_BUTTON
            || self.el(index).kind() != RESIZEBAR
            || self.bool_attribute(index, RESIZEBAR_DISABLED)
        {
            return;
        }
        let Some(window) = self.el(index).parent else {
            return;
        };
        if self.resizing.is_some()
            || self.movement.is_some_and(|m| m.element == window)
            || self.hud_locked
            || self.el(self.root_of(window)).locked
        {
            return;
        }
        let Some(border) = border_of(
            self.bool_attribute(index, BORDER_LEFT),
            self.bool_attribute(index, BORDER_RIGHT),
            self.bool_attribute(index, BORDER_TOP),
            self.bool_attribute(index, BORDER_BOTTOM),
        ) else {
            return;
        };
        self.start_resizing(window, border);
    }

    /// `UIElement::StartResizing`: the window takes the border and the pointer's offset; a
    /// `resize_line` window starts an outline of its own box.
    pub(crate) fn start_resizing(&mut self, window: usize, border: Border) {
        if self.resizing.is_some() {
            return;
        }
        let start = self.screen_box(window);
        let outline = self.bool_attribute(window, RESIZE_LINE).then_some(start);
        self.resizing = Some(Resizing {
            window,
            border,
            pointer: self.pointer,
            start,
            outline,
        });
    }

    /// `UIElement::MouseResizeElement` on each global mouse move: the window's box moves by the
    /// pointer's change since the last move (from the outline when there is one), and the window
    /// is moved and resized to it, unless the window draws an outline until the release.
    pub(crate) fn resize_mouse_move(&mut self) {
        let Some(r) = self.resizing else {
            return;
        };
        let base = r.outline.unwrap_or_else(|| self.screen_box(r.window));
        let (px, py) = self.pointer;
        let delta = (px - r.pointer.0, py - r.pointer.1);
        let limits = self.size_limits(r.window);
        let next = resized_box(r.border, base, delta, limits, self.display);
        if r.outline.is_some() {
            if let Some(current) = self.resizing.as_mut() {
                current.outline = Some(next);
            }
        } else {
            self.place_window(r.window, next);
        }
        if let Some(current) = self.resizing.as_mut() {
            current.pointer = (px, py);
        }
    }

    /// `UIElement::StopResizing`: the release ends the resize; an outline's box is applied now. A
    /// window whose box changed reports its place (and its location, when it saves one).
    pub(crate) fn end_resizing(&mut self) {
        let Some(r) = self.resizing.take() else {
            return;
        };
        if let Some(outline) = r.outline {
            self.place_window(r.window, outline);
        }
        if self.screen_box(r.window) != r.start {
            self.record_window_positions();
            if self.bool_attribute(r.window, SAVE_LOC)
                || self.bool_attribute(r.window, BROADCAST_LOC_CHANGE)
            {
                self.locations_changed = true;
            }
        }
    }

    /// The window's min and max size attributes (`GetAttribute_Int`).
    fn size_limits(&self, window: usize) -> SizeLimits {
        SizeLimits {
            min_width: self.int_attribute(window, MIN_WIDTH),
            max_width: self.int_attribute(window, MAX_WIDTH),
            min_height: self.int_attribute(window, MIN_HEIGHT),
            max_height: self.int_attribute(window, MAX_HEIGHT),
        }
    }

    /// The window is sized to the box, then placed at its top-left in its parent's (or the
    /// display's) coordinates. The size comes first: `move_to` keeps a root on the display for
    /// the size it has, so a window grown toward the bottom edge would be held above it.
    fn place_window(&mut self, window: usize, b: UiBox) {
        self.resize_to(window, b.width(), b.height());
        let (x, y) = self.to_parent_coords(window, b.x0, b.y0);
        self.move_to(window, x, y);
        self.moved = true;
        self.changed = true;
    }

    /// A screen point in the coordinates `move_to` takes for the element: a root's object origin
    /// is the screen itself, any other element's box is in its parent's.
    fn to_parent_coords(&self, index: usize, x: i32, y: i32) -> (i32, i32) {
        let el = self.el(index);
        match (el.object, el.parent) {
            (None, Some(parent)) => {
                let origin = self.screen_box(parent);
                (x - origin.x0, y - origin.y0)
            }
            _ => (x, y),
        }
    }
}
