//! Saved window locations: the windows that save their location (`save_loc`) are filed by
//! layout and element id, and restored on the next load (`UIMainFramework::InqLocations` and
//! `ApplyLocations`, `UIElement::InqLocation` / `CheckLocation` / `ApplyLocation`,
//! `UILocationData`). A window's corners are stored as clip-space points of the display
//! (`SceneTool::TransformPixelToClipPoint`), so a saved location means the same place at any
//! size: [`pixel_to_clip`] and [`clip_to_pixel`] are the two conversions.
//!
//! The windows move by their drag bar (`UIElement_Dragbar::MouseDown`, `UIElement::StartMovement`,
//! `MouseMoveElement`, `StopMovement`); a move of a window that saves its location (or broadcasts
//! its location changes) raises `location_changed`, which the client takes with
//! [`UiTree::take_locations_changed`] and answers by saving every window's location.

use ac2_net::protocol::{UiLocation, UiSaveLocations};
use ac2_script::native::{add_ui_location, GameplayOptions};

use crate::element::UiTree;
use crate::token::{ALLOW_RESIZE, BROADCAST_LOC_CHANGE, DRAGBAR, DRAGBAR_DISABLED, SAVE_LOC};

/// The left mouse button (`0x40000004`, `UIElement_Dragbar::MouseDown`'s button).
pub const UI_LEFT_BUTTON: u32 = 0x4000_0004;

/// `UILocationData`: a window's two corners in clip space and whether it shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowLocation {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub shown: bool,
}

/// A window's saved location under its layout and element ids (`UISaveLocations`'s entry).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SavedWindow {
    pub layout: u32,
    pub element: u32,
    pub location: WindowLocation,
}

/// The windows of a `UISaveLocations`, in its order (`UISaveLocations::InqUILocationData`'s
/// entries as the stream holds them).
pub fn saved_windows(ui: &UiSaveLocations) -> Vec<SavedWindow> {
    ui.layouts
        .iter()
        .flat_map(|(layout, _, windows)| {
            windows.iter().map(move |(element, l)| SavedWindow {
                layout: *layout,
                element: *element,
                location: WindowLocation {
                    x0: l.x0,
                    y0: l.y0,
                    x1: l.x1,
                    y1: l.y1,
                    shown: l.shown != 0,
                },
            })
        })
        .collect()
}

/// The `UISaveLocations` the UI's windows make (`UIMainFramework::InqLocations`'s `AddLocation`
/// calls in order). `Err` when two windows claim one place: retail's `AddLocation` fails and the
/// save is not made.
pub fn ui_save_locations(saved: &[SavedWindow]) -> Result<UiSaveLocations, String> {
    let mut ui = GameplayOptions::default().ui_locations;
    for window in saved {
        let l = window.location;
        let location = UiLocation {
            x0: l.x0,
            y0: l.y0,
            x1: l.x1,
            y1: l.y1,
            shown: u32::from(l.shown),
        };
        if !add_ui_location(&mut ui, window.layout, window.element, location) {
            return Err(format!(
                "window {:#x} of layout {:#x} is listed twice",
                window.element, window.layout
            ));
        }
    }
    Ok(ui)
}

/// `SceneTool::TransformPixelToClipPoint`: a pixel of a `width` x `height` display in clip space.
/// Computed in f64 and rounded to f32 where the stored location keeps it.
pub fn pixel_to_clip(x: i32, y: i32, width: i32, height: i32) -> (f32, f32) {
    let (w, h) = (f64::from(width), f64::from(height));
    let (x, y) = (f64::from(x), f64::from(y));
    let cx = (x + x) / w - 1.0 - 1.0 / w;
    let cy = -((y + y) / h - 1.0) - 1.0 / h;
    (cx as f32, cy as f32)
}

/// The rounding margin of a clip point stored as f32 (a twentieth of a pixel).
const CLIP_MARGIN: f64 = 0.05;

/// `SceneTool::TransformClipPointToPixel`: the pixel a clip-space point lands on (truncated, as
/// `_ftol2` does).
pub fn clip_to_pixel(cx: f32, cy: f32, width: i32, height: i32) -> (i32, i32) {
    let (w, h) = (f64::from(width), f64::from(height));
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    // The margin keeps a point stored a hair under a pixel from truncating to the pixel below.
    let x = ((1.0 / w + cx) + 1.0) * w * 0.5 + CLIP_MARGIN;
    let y = (1.0 - (1.0 / h + cy)) * h * 0.5 + CLIP_MARGIN;
    (x.trunc() as i32, y.trunc() as i32)
}

/// An element moved by its drag bar: the start of the drag and where the element was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Movement {
    pub(crate) element: usize,
    start_pointer: (i32, i32),
    start_pos: (i32, i32),
}

impl UiTree {
    /// `UIElement::InqLocation`'s test: the element saves its location.
    fn saves_location(&self, index: usize) -> bool {
        self.bool_attribute(index, SAVE_LOC)
    }

    /// `UIMainFramework::InqLocations`: the location of every top-level window that saves it.
    pub fn inq_locations(&self) -> Vec<SavedWindow> {
        let (width, height) = self.display;
        self.roots
            .iter()
            .filter(|&&root| self.saves_location(root))
            .map(|&root| {
                let el = self.el(root);
                let screen = self.screen_box(root);
                let (x0, y0) = pixel_to_clip(screen.x0, screen.y0, width, height);
                let (x1, y1) = pixel_to_clip(screen.x1, screen.y1, width, height);
                SavedWindow {
                    layout: el.layout.did,
                    element: el.id(),
                    location: WindowLocation {
                        x0,
                        y0,
                        x1,
                        y1,
                        shown: el.visible,
                    },
                }
            })
            .collect()
    }

    /// `UIMainFramework::ApplyLocations`: each root with a saved location takes it. A saved
    /// window that no longer saves its location aborts the whole apply (`CheckLocation`), and
    /// `false` is returned with nothing moved.
    pub fn apply_locations(&mut self, saved: &[SavedWindow]) -> bool {
        let targets: Vec<(usize, WindowLocation)> = self
            .roots
            .iter()
            .filter_map(|&root| {
                let el = self.el(root);
                saved
                    .iter()
                    .find(|s| s.layout == el.layout.did && s.element == el.id())
                    .map(|s| (root, s.location))
            })
            .collect();
        if targets.iter().any(|&(root, _)| !self.saves_location(root)) {
            return false;
        }
        let (width, height) = self.display;
        for (root, location) in targets {
            // `UIElement::ApplyLocation`: the corners in window pixels, the window moved there,
            // resized when it allows it, then shown or hidden.
            let (x0, y0) = clip_to_pixel(location.x0, location.y0, width, height);
            let (x1, y1) = clip_to_pixel(location.x1, location.y1, width, height);
            self.move_to(root, x0, y0);
            if self.bool_attribute(root, ALLOW_RESIZE) {
                self.resize_to(root, x1 - x0 + 1, y1 - y0 + 1);
            }
            self.set_visible(root, location.shown);
        }
        self.moved = true;
        self.changed = true;
        true
    }

    /// `UIElement::StartMovement`: the element follows the pointer from here. A move in progress
    /// is kept (one drag at a time).
    pub(crate) fn start_movement(&mut self, element: usize) {
        if self.movement.is_some() || self.hud_locked || self.el(self.root_of(element)).locked {
            return;
        }
        let start_pos = self.position_of(element);
        self.movement = Some(Movement {
            element,
            start_pointer: self.pointer,
            start_pos,
        });
    }

    /// The tree drawn again (a change made straight to an element's state, as the modern chat's
    /// line fade makes to the log's glyphs).
    pub(crate) fn mark_changed(&mut self) {
        self.changed = true;
    }

    /// The modern layout's offset of the pinned part `name` from its place (none: `(0, 0)`).
    pub(crate) fn hud_offset(&self, name: &str) -> (i32, i32) {
        self.hud_offsets.get(name).copied().unwrap_or((0, 0))
    }

    /// `root` locked in place or freed (the modern style's lock): a locked window neither moves
    /// (by its drag bar or Alt + drag) nor resizes.
    pub fn set_locked(&mut self, root: usize, locked: bool) {
        self.el_mut(root).locked = locked;
    }

    /// `UIElement::StopMovement`.
    pub(crate) fn stop_movement(&mut self, element: usize) {
        if self.movement.is_some_and(|m| m.element == element) {
            self.end_movement();
        }
    }

    /// The move ends (the button came up). A move that went somewhere raises
    /// `positions_changed`, so the client writes every window's place
    /// ([`UiTree::take_positions_changed`]).
    pub(crate) fn end_movement(&mut self) {
        if let Some(m) = self.movement.take() {
            if self.pointer != m.start_pointer {
                self.record_window_positions();
            }
        }
    }

    /// The window exception (`AGENTS.md`): with Alt held, a left press on a window (the topmost
    /// window root under the pointer, see `window_under_pointer`) moves that window, drag
    /// bar or not. It is decided before the press is offered to the elements, so no control under
    /// the window takes it. Retail moved a window only by its drag bar
    /// (`UIElement_Dragbar::MouseDown`). Returns whether the press was taken for a move.
    pub(crate) fn alt_move_mouse_down(&mut self, action: u32) -> bool {
        if action != UI_LEFT_BUTTON || !self.alt {
            return false;
        }
        let Some(root) = self.window_under_pointer() else {
            return false;
        };
        self.start_movement(root);
        true
    }

    /// The window the pointer is on: the first window root, topmost first (the roots' order), with
    /// a visible element under the pointer, found by the element tree (`UIRegion::NotifyMouseMove`'s
    /// test). An overlay or a transient is skipped, so the window under it is the target. A box is
    /// never enough: a window whose elements are all transparent or hidden under the pointer is not
    /// taken, so a large transparent window cannot steal the press.
    fn window_under_pointer(&self) -> Option<usize> {
        let pointer = self.pointer;
        self.roots.iter().copied().find(|&root| {
            self.is_window_root(root)
                && !self.is_transient_root(root)
                && self.pointer_hits(root, pointer)
        })
    }

    /// Whether an element (or one of its children) is under `pointer` and can be pointed at: shown,
    /// not `mouse_invisible`, and not transparent (`opacity` above zero). A child counts only inside
    /// its parent's place, as the mouse walk reads it.
    fn pointer_hits(&self, index: usize, pointer: (i32, i32)) -> bool {
        let el = self.el(index);
        if !el.visible {
            return false;
        }
        let b = self.screen_box(index);
        let (px, py) = pointer;
        if !((b.x0..=b.x1).contains(&px) && (b.y0..=b.y1).contains(&py)) {
            return false;
        }
        if el.mouse_visible && el.opacity > 0.0 {
            return true;
        }
        el.children
            .iter()
            .any(|&child| self.pointer_hits(child, pointer))
    }

    /// The Alt key's state (`GetKeyState(VK_MENU)`), set from the frame's modifiers.
    pub fn set_alt(&mut self, held: bool) {
        self.alt = held;
    }

    /// A root that is not under the pointer: the tooltip and the dragged copy.
    fn is_transient_root(&self, root: usize) -> bool {
        Some(root) == self.tooltip.element || Some(root) == self.drag.element
    }

    /// `UIElement::MouseMoveElement` on each global mouse move: the moving element goes where
    /// the pointer took it (clamped as `MoveTo` clamps it), and a save-location window raises
    /// `location_changed`.
    pub(crate) fn movement_mouse_move(&mut self) {
        let Some(m) = self.movement else {
            return;
        };
        let (px, py) = self.pointer;
        let x = m.start_pos.0 + (px - m.start_pointer.0);
        let y = m.start_pos.1 + (py - m.start_pointer.1);
        self.move_to(m.element, x, y);
        if self.bool_attribute(m.element, SAVE_LOC)
            || self.bool_attribute(m.element, BROADCAST_LOC_CHANGE)
        {
            self.locations_changed = true;
        }
    }

    /// `UIElement_Dragbar::MouseDown`: a left press on an enabled drag bar starts moving its
    /// parent.
    pub(crate) fn dragbar_mouse_down(&mut self, index: usize, action: u32) {
        if action != UI_LEFT_BUTTON
            || self.el(index).kind() != DRAGBAR
            || self.bool_attribute(index, DRAGBAR_DISABLED)
        {
            return;
        }
        if let Some(parent) = self.el(index).parent {
            self.start_movement(parent);
        }
    }

    /// `UIElement_Dragbar::MouseUp`: the release stops moving the drag bar's parent.
    pub(crate) fn dragbar_mouse_up(&mut self, index: usize, action: u32) {
        if action != UI_LEFT_BUTTON || self.el(index).kind() != DRAGBAR {
            return;
        }
        if let Some(parent) = self.el(index).parent {
            self.stop_movement(parent);
        }
    }

    /// Whether a location change happened since the last call: the client saves every window's
    /// location (`CM_UI::SendNotice_SaveLocationsChanged`).
    pub fn take_locations_changed(&mut self) -> bool {
        std::mem::take(&mut self.locations_changed)
    }

    /// The element's top-left: its surface object's on the display for a root, else its box's in
    /// the parent.
    fn position_of(&self, index: usize) -> (i32, i32) {
        let el = self.el(index);
        match el.object {
            Some(origin) => origin,
            None => (el.rect.x0, el.rect.y0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pixel_round_trips_through_clip_space() {
        for &(w, h) in &[(1024, 768), (1280, 1024), (800, 600)] {
            for &(x, y) in &[(0, 0), (1, 1), (100, 37), (w / 2, h / 2), (w - 1, h - 1)] {
                let (cx, cy) = pixel_to_clip(x, y, w, h);
                let (rx, ry) = clip_to_pixel(cx, cy, w, h);
                assert!((rx - x).abs() <= 1, "x {x} -> {rx} on {w}");
                assert!((ry - y).abs() <= 1, "y {y} -> {ry} on {h}");
            }
        }
    }

    #[test]
    fn the_display_corners_are_the_clip_corners() {
        // `TransformPixelToClipPoint`: the top-left pixel is (-1, 1) less a half pixel.
        let (cx, cy) = pixel_to_clip(0, 0, 1024, 768);
        assert!((cx + 1.0 + 1.0 / 1024.0).abs() < 1e-6, "{cx}");
        assert!((cy - 1.0 + 1.0 / 768.0).abs() < 1e-6, "{cy}");
    }
}
