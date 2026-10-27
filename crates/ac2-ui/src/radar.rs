//! `gmRadarUI`'s drawing: the blips over `RadarField` and the heading ring (`HeadingField`).
//!
//! `RadarField` is drawn every global loop (`ListenToMessage` 0xffff0005, `UIRegion::ForceDraw`);
//! after each draw (`draw_done`, `Update`) the heading object follows the field (its corner plus
//! `BaseField`'s) and turns to the player's heading in whole degrees, and every object with a
//! look whose offset in the player's frame lies within the range is plotted at
//! `RadarCenter + ftol(offset * RadarRadius / range)` (y up), kept in the radius' square, in its
//! shape (`DrawBlip`: point, fill, cross, X, hollow, L, edges, corners, T, face, each a few
//! `SurfaceWindow::FillArea` pixels) with a box around the selection (`DrawSelected`). The first
//! blip within four pixels of the pointer names `BaseField`'s tooltip.
//!
//! The player's map coordinates (`Position::ToRealMapCoords_Info`, rendered by the client) go
//! into `CoordinatesText` when they change.
//!
//! `CoordBoxButton` opens or closes `BaseField`'s coordinates box and saves the choice
//! (`gmCWInterface::SetShowRadarCoords`); a press on `BaseField` selects the blip under the
//! pointer (`ListenToMessage`); `ReloadOptions` opens or closes the box from the options.

use crate::compose::UiSurface;
use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};
use crate::ui_string::{UiString, UiStrings};

/// What the radar asks of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadarRequest {
    /// `gmCWInterface::SetSelection` of the blip under the pointer.
    Select(u64),
    /// `gmCWInterface::SetShowRadarCoords` (then
    /// `CM_Character::SendNotice_ChangeRadarShowCoordsPreference`).
    ShowCoords(bool),
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// One object with a look, as the client's radar data gives it.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarDot {
    pub id: u64,
    /// The look's `RGBAColor` (r, g, b, a) and blip shape (`0x40000001`..`0x4000000A`).
    pub color: [f32; 4],
    pub shape: u32,
    /// The offset in the player's frame (`Position::localtolocal`), x east, y north.
    pub offset: [f32; 2],
    /// Within the range (`x * x + y * y <= range * range`).
    pub in_range: bool,
    /// The object's name, for the tooltip.
    pub name: String,
}

/// A radar frame: the range where the player stands, the player's heading, the blips and the
/// selection.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadarView {
    pub range: f32,
    pub heading_degrees: Option<f32>,
    pub dots: Vec<RadarDot>,
    pub selection: u64,
    /// The player's map coordinates, rendered.
    pub coordinates: Option<String>,
}

/// `gmRadarUI`'s drawing state.
#[derive(Debug, Clone)]
pub struct RadarUi {
    /// `_radarField`, `_baseField`, `_headingField`.
    field: Option<usize>,
    base: Option<usize>,
    heading: Option<usize>,
    /// `_radarX`, `_radarY`, `_radarRadius`, `_xoffset`, `_yoffset`.
    center: (i32, i32),
    radius: i32,
    offset: (i32, i32),
    /// `_oldHeading`, `_oldTooltip`.
    old_heading: f32,
    old_tooltip: Option<String>,
    /// `_oldPos`.
    old_coordinates: Option<String>,
    /// `_lastUnderMouseID`.
    last_under_mouse: u64,
    /// The pixels the last `Update` plots: (x, y, colour).
    pixels: Vec<(i32, i32, [u8; 4])>,
}

fn int_attr(tree: &UiTree, index: usize, name: &str) -> i32 {
    match tree.get(index).and_then(|e| e.attribute(id(name))) {
        Some(AttributeValue::Int(v)) => *v,
        _ => 0,
    }
}

/// `SurfaceWindow::FillArea`'s colour from an `RGBAColor`.
fn pixel(color: [f32; 4]) -> [u8; 4] {
    color.map(|c| (f64::from(c.clamp(0.0, 1.0)) * 255.0) as u8)
}

impl RadarUi {
    /// `gmRadarUI::gmRadarUI`'s part over the HUD's `RadarField` and `HeadingField` roots.
    pub fn create(tree: &UiTree) -> Self {
        let root = |name: &str| {
            tree.roots()
                .iter()
                .copied()
                .find(|&r| tree.get(r).is_some_and(|e| e.id() == id(name)))
        };
        let field = root("RadarField");
        let base = field.and_then(|f| tree.child_recursive(f, id("BaseField")));
        let offset = base.map_or((0, 0), |b| tree.object_origin(b));
        Self {
            field,
            base,
            heading: root("HeadingField"),
            center: field.map_or((0, 0), |f| {
                (
                    int_attr(tree, f, "RadarCenter_X"),
                    int_attr(tree, f, "RadarCenter_Y"),
                )
            }),
            radius: field.map_or(0, |f| int_attr(tree, f, "RadarRadius")),
            offset,
            old_heading: 0.0,
            old_tooltip: None,
            old_coordinates: None,
            last_under_mouse: 0,
            pixels: Vec::new(),
        }
    }

    /// The radar field's root, which [`RadarUi::draw`] paints over.
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `gmRadarUI::Show`'s part: the radar field and the heading ring show or hide with the flag.
    pub fn show(&self, tree: &mut UiTree, shown: bool) {
        for element in [self.field, self.heading].into_iter().flatten() {
            tree.set_visible(element, shown);
        }
    }

    /// Whether the radar is shown (its field is visible; `gmRadarUI::Shown`).
    pub fn shown(&self, tree: &UiTree) -> bool {
        self.field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.visible)
    }

    /// `gmRadarUI::Update` for this frame: the heading ring placed and turned, the blips
    /// plotted, the tooltip named.
    pub fn update(&mut self, tree: &mut UiTree, view: &RadarView) {
        self.pixels.clear();
        let Some(field) = self.field else {
            return;
        };
        if !tree.get(field).is_some_and(|e| e.visible) {
            return;
        }
        if view.coordinates.is_some() && view.coordinates != self.old_coordinates {
            self.old_coordinates = view.coordinates.clone();
            if let Some(text) = tree.child_recursive(field, id("CoordinatesText")) {
                tree.set_text(text, view.coordinates.as_deref().unwrap_or_default());
            }
        }
        let (fx, fy) = tree.get(field).and_then(|e| e.object).unwrap_or((0, 0));
        if let Some(heading) = self.heading {
            tree.set_object_position(heading, self.offset.0 + fx, self.offset.1 + fy);
            if let Some(degrees) = view.heading_degrees {
                if self.old_heading != degrees {
                    self.old_heading = degrees;
                    tree.set_object_rotation(heading, degrees as i32);
                }
            }
        }
        // `radarScale = (float)_radarRadius / range`, stored.
        let scale = f64::from((f64::from(self.radius) / f64::from(view.range)) as f32);
        let (px, py) = tree.pointer();
        let mut tooltip = None;
        self.last_under_mouse = 0;
        for dot in view.dots.iter().filter(|d| d.in_range) {
            let x = self.center.0 + (f64::from(dot.offset[0]) * scale) as i32;
            let y = self.center.1 - (f64::from(dot.offset[1]) * scale) as i32;
            let r = self.radius;
            let (cx, cy) = self.center;
            if x < cx - r || cx + r < x || y < cy - r || cy + r < y {
                continue;
            }
            let (dx, dy) = (px - fx - x, py - fy - y);
            if self.last_under_mouse == 0 && (dx * dx + dy * dy) as u32 <= 16 {
                self.last_under_mouse = dot.id;
                tooltip = Some(dot.name.clone());
            }
            self.draw_blip(dot, x, y, dot.id == view.selection && dot.id != 0);
        }
        if tooltip != self.old_tooltip {
            if let Some(base) = self.base {
                tree.set_tooltip(base, tooltip.clone());
            }
            self.old_tooltip = tooltip;
        }
    }

    /// `gmRadarUI::SetCoordButtonToolTip`.
    fn coord_button_tooltip(&self, tree: &mut UiTree, strings: &dyn UiStrings, shown: bool) {
        let button = self
            .field
            .and_then(|f| tree.child_recursive(f, id("CoordBoxButton")));
        if let Some(button) = button {
            let string = if shown {
                "ID_Radar_TT_CurLoc_Shown"
            } else {
                "ID_Radar_TT_CurLoc_Hidden"
            };
            let text = strings.render_string(&UiString::ui(id(string), Vec::new()));
            tree.set_tooltip(button, Some(text));
        }
    }

    /// `gmRadarUI::ReloadOptions` (and `Init`'s button tooltip): the coordinates box open when
    /// the options show the coordinates.
    pub fn reload_options(&mut self, tree: &mut UiTree, strings: &dyn UiStrings, show: bool) {
        self.old_coordinates = None;
        if let Some(base) = self.base {
            tree.set_state(base, id(if show { "open" } else { "closed" }));
            self.coord_button_tooltip(tree, strings, show);
        }
    }

    /// `gmRadarUI::ListenToMessage`: the coordinates button toggles the box; a press on the
    /// base selects the blip last under the pointer.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn UiStrings,
        message: &UiMessage,
    ) -> Option<RadarRequest> {
        if message.element_id == id("CoordBoxButton") && message.message == MSG_BUTTON_RELEASE_ON {
            let base = self.base?;
            self.old_coordinates = None;
            let show = tree.get(base)?.state() != id("open");
            tree.set_state(base, id(if show { "open" } else { "closed" }));
            self.coord_button_tooltip(tree, strings, show);
            return Some(RadarRequest::ShowCoords(show));
        }
        if message.element_id == id("BaseField")
            && message.message == MSG_MOUSE_PRESS_TOP
            && self.last_under_mouse != 0
        {
            return Some(RadarRequest::Select(self.last_under_mouse));
        }
        None
    }

    /// `SurfaceWindow::FillArea` of the box (x0, y0)..(x1, y1).
    fn fill(&mut self, color: [u8; 4], x0: i32, y0: i32, x1: i32, y1: i32) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                self.pixels.push((x, y, color));
            }
        }
    }

    fn point(&mut self, c: [u8; 4], x: i32, y: i32) {
        self.fill(c, x, y, x, y);
    }

    /// `gmRadarUI::DrawEdges`: above, below, left and right.
    fn edges(&mut self, c: [u8; 4], x: i32, y: i32) {
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            self.point(c, x + dx, y + dy);
        }
    }

    /// `gmRadarUI::DrawCorners`.
    fn corners(&mut self, c: [u8; 4], x: i32, y: i32) {
        for (dx, dy) in [(1, 1), (-1, -1), (-1, 1), (1, -1)] {
            self.point(c, x + dx, y + dy);
        }
    }

    /// `gmRadarUI::DrawBlip`.
    fn draw_blip(&mut self, dot: &RadarDot, x: i32, y: i32, selected: bool) {
        let c = pixel(dot.color);
        match dot.shape {
            0x4000_0001 => self.point(c, x, y),
            0x4000_0002 => {
                self.point(c, x, y);
                self.edges(c, x, y);
                self.corners(c, x, y);
            }
            0x4000_0003 => {
                self.point(c, x, y);
                self.edges(c, x, y);
            }
            0x4000_0004 => {
                self.point(c, x, y);
                self.corners(c, x, y);
            }
            0x4000_0005 => {
                self.edges(c, x, y);
                self.corners(c, x, y);
            }
            0x4000_0006 => {
                // `DrawL`.
                self.point(c, x, y + 1);
                self.fill(c, x - 1, y - 1, x - 1, y + 1);
            }
            0x4000_0007 => self.edges(c, x, y),
            0x4000_0008 => self.corners(c, x, y),
            0x4000_0009 => {
                // `DrawT`.
                self.fill(c, x - 1, y - 1, x + 1, y - 1);
                self.fill(c, x, y - 1, x, y + 1);
            }
            0x4000_000A => {
                // `DrawFace`.
                self.fill(c, x - 1, y + 1, x + 1, y + 1);
                self.point(c, x - 1, y - 1);
                self.point(c, x + 1, y - 1);
            }
            _ => {}
        }
        if selected {
            // `DrawSelected`: a box three pixels out.
            self.fill(c, x - 2, y + 3, x + 2, y + 3);
            self.fill(c, x + 3, y - 2, x + 3, y + 2);
            self.fill(c, x - 2, y - 3, x + 2, y - 3);
            self.fill(c, x - 3, y - 2, x - 3, y + 2);
        }
    }

    /// The pixels the last update plotted, in the radar surface's coordinates.
    pub fn pixels(&self) -> &[(i32, i32, [u8; 4])] {
        &self.pixels
    }

    /// The blips painted over the composed radar surface (`UISurface::PrepareSurface` and the
    /// fills), clipped to it.
    pub fn draw(&self, surface: &mut UiSurface) {
        let (w, h) = (surface.width as i32, surface.height as i32);
        for &(x, y, c) in &self.pixels {
            if 0 <= x && x < w && 0 <= y && y < h {
                let at = (y as usize * w as usize + x as usize) * 4;
                surface.rgba[at..at + 4].copy_from_slice(&c);
            }
        }
    }
}
