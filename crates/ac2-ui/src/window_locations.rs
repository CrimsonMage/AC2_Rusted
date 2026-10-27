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
    element: usize,
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
        if self.movement.is_some() {
            return;
        }
        let start_pos = self.position_of(element);
        self.movement = Some(Movement {
            element,
            start_pointer: self.pointer,
            start_pos,
        });
    }

    /// `UIElement::StopMovement`.
    pub(crate) fn stop_movement(&mut self, element: usize) {
        if self.movement.is_some_and(|m| m.element == element) {
            self.movement = None;
        }
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
