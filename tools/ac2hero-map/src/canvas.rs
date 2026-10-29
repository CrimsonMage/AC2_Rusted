//! The map itself: the world's -100..100 map square, drag to pan and wheel to zoom, the shown
//! markers as icons, and markers that crowd one spot grouped into a counted cluster (a click on
//! a cluster zooms into it and picks its members).

use ac2_guide::atlas::{Marker, MarkerKind};
use ac2_guide::{nav, MapCoord};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, Vec2};

use crate::notes::{Note, NoteKind};
use crate::tiles::{self, Tiles};

/// The player on the map: where they stand and their compass heading in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Player {
    pub at: MapCoord,
    pub heading: f32,
}

/// What the canvas draws besides the terrain.
pub struct Scene<'a> {
    pub markers: &'a [Marker],
    pub shown: &'a [usize],
    pub picked: &'a [usize],
    pub notes: &'a [Note],
    /// Whether each `NoteKind::ALL` layer is on.
    pub note_layers: &'a [bool],
    pub note_icons: &'a std::collections::HashMap<u32, egui::TextureHandle>,
    pub picked_note: Option<usize>,
    pub player: Option<Player>,
    /// Whether a right click offers to travel there.
    pub can_travel: bool,
}

/// What a click on the canvas asks for.
pub enum Event {
    Pick(Vec<usize>),
    PickNote(usize),
    Travel(MapCoord),
}

/// Screen size of the cells markers are grouped by.
const CLUSTER_CELL: f32 = 26.0;
/// Zoom (screen points per map unit) from which single markers are labelled.
const LABEL_SCALE: f32 = 14.0;

pub fn colour(kind: MarkerKind) -> Color32 {
    match kind {
        MarkerKind::QuestStart => Color32::from_rgb(240, 180, 40),
        MarkerKind::QuestStep => Color32::from_rgb(230, 120, 50),
        MarkerKind::Npc => Color32::from_rgb(80, 160, 240),
        MarkerKind::GuidePoint => Color32::from_rgb(110, 200, 120),
    }
}

/// Draws a marker's icon at `p`.
pub fn icon(painter: &egui::Painter, p: Pos2, kind: MarkerKind, scale: f32) {
    let c = colour(kind);
    let edge = Stroke::new(1.0, Color32::from_black_alpha(200));
    match kind {
        MarkerKind::QuestStart => {
            let r = 7.0 * scale;
            painter.circle(p, r, c, edge);
            painter.text(
                p,
                Align2::CENTER_CENTER,
                "!",
                FontId::proportional(11.0 * scale),
                Color32::BLACK,
            );
        }
        MarkerKind::Npc => {
            painter.circle(p, 5.0 * scale, c, Stroke::new(1.5, Color32::WHITE));
        }
        MarkerKind::QuestStep => {
            let r = 4.5 * scale;
            painter.add(Shape::convex_polygon(
                vec![
                    p + Vec2::new(0.0, -r),
                    p + Vec2::new(r, 0.0),
                    p + Vec2::new(0.0, r),
                    p + Vec2::new(-r, 0.0),
                ],
                c,
                edge,
            ));
        }
        MarkerKind::GuidePoint => {
            painter.rect(
                Rect::from_center_size(p, Vec2::splat(7.0 * scale)),
                1.0,
                c,
                edge,
                egui::StrokeKind::Middle,
            );
        }
    }
}

/// Land cells per map unit: a map coordinate is a percent of half the land's 2040 cells.
const CELLS_PER_UNIT: f64 = 10.2;

pub struct Canvas {
    terrain: Option<Tiles>,
    /// The coordinate grid and the pointer's coordinates (off in the panel-sized preview).
    pub chrome: bool,
    /// Where the open right-click menu was asked for.
    menu_at: Option<MapCoord>,
    centre: MapCoord,
    /// Screen points per map unit; 0 until the first frame fits the markers.
    scale: f32,
    /// The markers' extent, for the first fit.
    bounds: (MapCoord, MapCoord),
}

impl Canvas {
    pub fn new(markers: &[Marker], terrain: Option<Tiles>) -> Self {
        let mut lo = MapCoord {
            north: 100.0,
            east: 100.0,
        };
        let mut hi = MapCoord {
            north: -100.0,
            east: -100.0,
        };
        for m in markers {
            lo.north = lo.north.min(m.at.north);
            lo.east = lo.east.min(m.at.east);
            hi.north = hi.north.max(m.at.north);
            hi.east = hi.east.max(m.at.east);
        }
        if markers.is_empty() {
            (lo, hi) = (
                MapCoord {
                    north: -100.0,
                    east: -100.0,
                },
                MapCoord {
                    north: 100.0,
                    east: 100.0,
                },
            );
        }
        Self {
            terrain,
            chrome: true,
            menu_at: None,
            centre: MapCoord {
                north: 0.0,
                east: 0.0,
            },
            scale: 0.0,
            bounds: (lo, hi),
        }
    }

    /// Centres the view on `at` at `pixels_per_cell` screen pixels per land cell.
    pub fn set_view(&mut self, at: MapCoord, pixels_per_cell: f32) {
        self.centre = at;
        self.scale = pixels_per_cell * CELLS_PER_UNIT as f32;
    }

    /// Whether the terrain is still being drawn.
    pub fn busy(&self) -> bool {
        self.terrain.as_ref().is_some_and(Tiles::busy)
    }

    /// Centres the view on `at` at the current zoom.
    pub fn centre_on(&mut self, at: MapCoord) {
        self.centre = at;
    }

    /// Centres the view on `at`, zoomed in to at least street level.
    pub fn focus(&mut self, at: MapCoord) {
        self.centre = at;
        self.scale = self.scale.max(LABEL_SCALE * 1.5);
    }

    /// Draws the map; what a click asked for.
    pub fn ui(&mut self, ui: &mut egui::Ui, scene: &Scene) -> Option<Event> {
        let (markers, shown, picked) = (scene.markers, scene.shown, scene.picked);
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        if self.scale == 0.0 {
            let (lo, hi) = self.bounds;
            self.centre = MapCoord {
                north: (lo.north + hi.north) * 0.5,
                east: (lo.east + hi.east) * 0.5,
            };
            let span =
                Vec2::new(hi.east - lo.east, hi.north - lo.north).max(Vec2::splat(10.0)) * 1.1;
            self.scale = (rect.width() / span.x).min(rect.height() / span.y);
        }
        if response.dragged() {
            let d = response.drag_delta() / self.scale;
            self.centre.east -= d.x;
            self.centre.north += d.y;
        }
        if let (true, Some(pointer)) = (response.hovered(), response.hover_pos()) {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.zoom_at(rect, pointer, (scroll * 0.002).exp());
            }
        }

        match &mut self.terrain {
            Some(tiles) => {
                painter.rect_filled(rect, 0.0, tiles::ocean());
                let view = Viewport {
                    rect,
                    centre: self.centre,
                    scale: self.scale,
                };
                tiles.draw(ui.ctx(), &painter, rect, &view);
                let progress = tiles.progress();
                if progress < 1.0 {
                    painter.text(
                        rect.center_top() + Vec2::new(0.0, 16.0),
                        Align2::CENTER_TOP,
                        format!("Drawing the world… {:.0}%", progress * 100.0),
                        FontId::proportional(14.0),
                        Color32::WHITE,
                    );
                }
            }
            None => {
                painter.rect_filled(rect, 0.0, Color32::from_rgb(22, 30, 38));
            }
        }
        if self.chrome {
            self.grid(&painter, rect);
        }
        let pointer = response.hover_pos();
        let hovered_note = self.draw_notes(&painter, rect, scene, pointer);

        // Group the shown markers by screen cell; a lone marker draws as its icon.
        let mut cells: std::collections::HashMap<(i32, i32), Vec<usize>> = Default::default();
        for &i in shown {
            let p = self.to_screen(rect, markers[i].at);
            if !rect.expand(CLUSTER_CELL).contains(p) {
                continue;
            }
            let key = (
                (p.x / CLUSTER_CELL).floor() as i32,
                (p.y / CLUSTER_CELL).floor() as i32,
            );
            cells.entry(key).or_default().push(i);
        }
        let mut hovered: Option<(f32, Vec<usize>, Pos2)> = None;
        let mut groups: Vec<(Pos2, Vec<usize>)> = cells
            .into_values()
            .map(|members| {
                let sum = members.iter().fold(Vec2::ZERO, |s, &i| {
                    s + self.to_screen(rect, markers[i].at).to_vec2()
                });
                ((sum / members.len() as f32).to_pos2(), members)
            })
            .collect();
        // Quest starts on top of steps and NPCs.
        groups.sort_by_key(|(_, m)| m.iter().map(|&i| rank(markers[i].kind)).max());
        for (p, members) in &groups {
            if members.len() == 1 {
                let m = &markers[members[0]];
                icon(&painter, *p, m.kind, 1.0);
                if self.scale >= LABEL_SCALE && m.kind != MarkerKind::QuestStep {
                    painter.text(
                        *p + Vec2::new(9.0, 0.0),
                        Align2::LEFT_CENTER,
                        &m.title,
                        FontId::proportional(11.0),
                        Color32::from_white_alpha(210),
                    );
                }
            } else {
                let kind = members
                    .iter()
                    .map(|&i| markers[i].kind)
                    .max_by_key(|&k| rank(k))
                    .unwrap_or(MarkerKind::Npc);
                let r = 9.0 + (members.len() as f32).log2() * 2.5;
                painter.circle(
                    *p,
                    r,
                    colour(kind).gamma_multiply(0.85),
                    Stroke::new(1.5, Color32::WHITE),
                );
                painter.text(
                    *p,
                    Align2::CENTER_CENTER,
                    members.len().to_string(),
                    FontId::proportional(11.0),
                    Color32::BLACK,
                );
            }
            if let Some(m) = pointer {
                let d = m.distance(*p);
                let reach = if members.len() == 1 {
                    9.0
                } else {
                    9.0 + (members.len() as f32).log2() * 2.5
                };
                if d <= reach && hovered.as_ref().is_none_or(|h| d < h.0) {
                    hovered = Some((d, members.clone(), *p));
                }
            }
        }
        for &i in picked {
            let p = self.to_screen(rect, markers[i].at);
            if rect.contains(p) {
                painter.circle_stroke(p, 12.0, Stroke::new(2.5, Color32::WHITE));
            }
        }

        if let Some(player) = scene.player {
            let p = self.to_screen(rect, player.at);
            let a = player.heading.to_radians();
            let fwd = Vec2::new(a.sin(), -a.cos());
            let side = Vec2::new(-fwd.y, fwd.x);
            // Smaller in the panel-sized map.
            let k = if self.chrome { 1.0 } else { 0.6 };
            painter.circle_filled(p, 13.0 * k, Color32::from_black_alpha(90));
            painter.add(Shape::convex_polygon(
                vec![
                    p + fwd * 12.0 * k,
                    p - fwd * 7.0 * k + side * 8.0 * k,
                    p - fwd * 3.0 * k,
                    p - fwd * 7.0 * k - side * 8.0 * k,
                ],
                Color32::from_rgb(120, 230, 120),
                Stroke::new(1.5, Color32::BLACK),
            ));
        }

        if let Some(m) = pointer.filter(|_| self.chrome) {
            painter.text(
                rect.left_bottom() + Vec2::new(10.0, -10.0),
                Align2::LEFT_BOTTOM,
                nav::format(self.to_map(rect, m)),
                FontId::monospace(13.0),
                Color32::from_white_alpha(220),
            );
        }
        if response.secondary_clicked() {
            self.menu_at = pointer.map(|m| self.to_map(rect, m));
        }
        let mut event = None;
        if scene.can_travel {
            if let Some(at) = self.menu_at {
                response.context_menu(|ui| {
                    ui.label(nav::format(at));
                    if ui.button("Travel here").clicked() {
                        event = Some(Event::Travel(at));
                        ui.close();
                    }
                });
            }
        }
        match (hovered, hovered_note) {
            (Some((_, members, p)), _) => {
                let tip = tooltip(markers, &members);
                let response = response.on_hover_text(tip);
                if response.clicked() {
                    if members.len() > 1 {
                        self.zoom_at(rect, p, 2.5);
                    }
                    event = Some(Event::Pick(members));
                }
            }
            (None, Some(n)) => {
                let note = &scene.notes[n];
                let response =
                    response.on_hover_text(format!("{}\n{}", note.name, nav::format(note.at)));
                if response.clicked() {
                    event = Some(Event::PickNote(n));
                }
            }
            (None, None) if response.clicked() => event = Some(Event::Pick(Vec::new())),
            (None, None) => {}
        }
        event
    }

    /// Draws the map notes whose layer is on and whose kind the zoom reaches, names where the
    /// zoom allows; the note under the pointer.
    fn draw_notes(
        &self,
        painter: &egui::Painter,
        rect: Rect,
        scene: &Scene,
        pointer: Option<Pos2>,
    ) -> Option<usize> {
        let ppc = self.scale / CELLS_PER_UNIT as f32;
        let mut hovered: Option<(f32, usize)> = None;
        // The panel-sized map draws the 9 x 9 icons at their own size.
        let size = Vec2::splat(match (self.chrome, ppc >= 2.0) {
            (true, true) => 18.0,
            (true, false) => 14.0,
            (false, true) => 11.0,
            (false, false) => 9.0,
        });
        let (town_font, note_font) = if self.chrome {
            (14.0, 11.0)
        } else {
            (10.0, 9.0)
        };
        // Names already written; a name that would cover one is left out (towns go first).
        let mut written: Vec<Rect> = Vec::new();
        let mut names: Vec<(Pos2, &str, bool)> = Vec::new();
        // Towns last, so they draw on top.
        let mut order: Vec<usize> = (0..scene.notes.len()).collect();
        order.sort_by_key(|&i| {
            std::cmp::Reverse(NoteKind::ALL.iter().position(|k| *k == scene.notes[i].kind))
        });
        for i in order {
            let note = &scene.notes[i];
            let layer = NoteKind::ALL
                .iter()
                .position(|k| *k == note.kind)
                .unwrap_or(0);
            if !scene.note_layers.get(layer).copied().unwrap_or(false)
                || ppc < note.kind.shown_from()
            {
                continue;
            }
            let p = self.to_screen(rect, note.at);
            if !rect.expand(40.0).contains(p) {
                continue;
            }
            let r = Rect::from_center_size(p, size);
            match scene.note_icons.get(&note.icon) {
                Some(icon) => {
                    painter.rect_filled(r.expand(1.0), 2.0, Color32::from_black_alpha(140));
                    painter.image(
                        icon.id(),
                        r,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                None => {
                    painter.circle(
                        p,
                        size.x * 0.4,
                        Color32::from_rgb(240, 220, 160),
                        Stroke::new(1.0, Color32::BLACK),
                    );
                }
            }
            if scene.picked_note == Some(i) {
                painter.circle_stroke(p, size.x * 0.8, Stroke::new(2.5, Color32::WHITE));
            }
            if ppc >= note.kind.named_from() && !note.name.is_empty() {
                names.push((
                    p + Vec2::new(size.x * 0.5 + 4.0, 0.0),
                    &note.name,
                    note.kind == NoteKind::Town,
                ));
            }
            if let Some(m) = pointer {
                let d = m.distance(p);
                if d <= size.x * 0.6 && hovered.is_none_or(|h| d < h.0) {
                    hovered = Some((d, i));
                }
            }
        }
        // Towns' names first, then the rest, each only where it covers no other.
        names.sort_by_key(|n| !n.2);
        for (at, name, town) in names {
            let (font, colour) = if town {
                (FontId::proportional(town_font), Color32::WHITE)
            } else {
                (
                    FontId::proportional(note_font),
                    Color32::from_white_alpha(225),
                )
            };
            let galley = painter.layout_no_wrap(name.to_string(), font.clone(), colour);
            let r = Rect::from_min_size(at - Vec2::new(0.0, galley.size().y * 0.5), galley.size())
                .expand(2.0);
            if written.iter().any(|w| w.intersects(r)) {
                continue;
            }
            written.push(r);
            for d in [
                Vec2::new(1.0, 1.0),
                Vec2::new(-1.0, 1.0),
                Vec2::new(1.0, -1.0),
                Vec2::new(-1.0, -1.0),
            ] {
                painter.text(
                    at + d,
                    Align2::LEFT_CENTER,
                    name,
                    font.clone(),
                    Color32::from_black_alpha(200),
                );
            }
            painter.text(at, Align2::LEFT_CENTER, name, font, colour);
        }
        hovered.map(|(_, i)| i)
    }

    fn zoom_at(&mut self, rect: Rect, at: Pos2, zoom: f32) {
        let before = self.to_map(rect, at);
        self.scale = (self.scale * zoom).clamp(1.0, 600.0);
        let after = self.to_map(rect, at);
        self.centre.east += before.east - after.east;
        self.centre.north += before.north - after.north;
    }

    fn to_screen(&self, rect: Rect, c: MapCoord) -> Pos2 {
        rect.center()
            + Vec2::new(c.east - self.centre.east, self.centre.north - c.north) * self.scale
    }

    fn to_map(&self, rect: Rect, p: Pos2) -> MapCoord {
        let d = (p - rect.center()) / self.scale;
        MapCoord {
            north: self.centre.north - d.y,
            east: self.centre.east + d.x,
        }
    }

    /// The map's edge and a line every 10, 5 or 1 units by zoom, labelled at the view's edges.
    fn grid(&self, painter: &egui::Painter, rect: Rect) {
        let step = if self.scale > 60.0 {
            1.0
        } else if self.scale > 12.0 {
            5.0
        } else {
            10.0
        };
        let faint = Color32::from_white_alpha(18);
        let strong = Color32::from_white_alpha(60);
        let label = Color32::from_white_alpha(110);
        let font = FontId::proportional(10.0);
        let mut v = -100.0f32;
        while v <= 100.0 {
            let x = self
                .to_screen(
                    rect,
                    MapCoord {
                        north: 0.0,
                        east: v,
                    },
                )
                .x;
            let y = self
                .to_screen(
                    rect,
                    MapCoord {
                        north: v,
                        east: 0.0,
                    },
                )
                .y;
            let major = v == 0.0 || v.abs() == 100.0 || (step < 10.0 && v % 10.0 == 0.0);
            let stroke = Stroke::new(
                if major { 1.0 } else { 0.5 },
                if major { strong } else { faint },
            );
            if (rect.left()..=rect.right()).contains(&x) {
                painter.line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    stroke,
                );
                let t = if v < 0.0 {
                    format!("{}W", -v)
                } else {
                    format!("{v}E")
                };
                painter.text(
                    Pos2::new(x + 2.0, rect.top() + 2.0),
                    Align2::LEFT_TOP,
                    t,
                    font.clone(),
                    label,
                );
            }
            if (rect.top()..=rect.bottom()).contains(&y) {
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    stroke,
                );
                let t = if v < 0.0 {
                    format!("{}S", -v)
                } else {
                    format!("{v}N")
                };
                painter.text(
                    Pos2::new(rect.left() + 2.0, y - 2.0),
                    Align2::LEFT_BOTTOM,
                    t,
                    font.clone(),
                    label,
                );
            }
            v += step;
        }
    }
}

/// Which icon wins a cluster's colour and draws on top.
fn rank(kind: MarkerKind) -> u8 {
    match kind {
        MarkerKind::QuestStart => 3,
        MarkerKind::Npc => 2,
        MarkerKind::GuidePoint => 1,
        MarkerKind::QuestStep => 0,
    }
}

fn tooltip(markers: &[Marker], members: &[usize]) -> String {
    const MAX: usize = 12;
    let mut lines: Vec<String> = members
        .iter()
        .take(MAX)
        .map(|&i| {
            let m = &markers[i];
            let level = m.level.map_or(String::new(), |l| format!(" (level {l})"));
            match m.kind {
                MarkerKind::QuestStart => format!("! {}{level}: {}", m.title, m.context),
                MarkerKind::QuestStep => format!("{}{level}: {}", m.title, m.context),
                MarkerKind::Npc => format!("{}: {}", m.title, m.context),
                MarkerKind::GuidePoint => format!("{}: {}", m.title, m.context),
            }
        })
        .collect();
    if members.len() > MAX {
        lines.push(format!(
            "… and {} more (click to zoom in)",
            members.len() - MAX
        ));
    }
    lines.join("\n")
}

/// The canvas's mapping for the terrain tiles: land cells (x east, y north) to the screen.
struct Viewport {
    rect: Rect,
    centre: MapCoord,
    scale: f32,
}

impl tiles::View for Viewport {
    fn cell_to_screen(&self, x: f64, y: f64) -> Pos2 {
        let east = (x / CELLS_PER_UNIT - 100.0) as f32;
        let north = (y / CELLS_PER_UNIT - 100.0) as f32;
        self.rect.center()
            + Vec2::new(east - self.centre.east, self.centre.north - north) * self.scale
    }

    fn screen_to_cell(&self, p: Pos2) -> (f64, f64) {
        let d = (p - self.rect.center()) / self.scale;
        let east = f64::from(self.centre.east + d.x);
        let north = f64::from(self.centre.north - d.y);
        (
            (east + 100.0) * CELLS_PER_UNIT,
            (north + 100.0) * CELLS_PER_UNIT,
        )
    }

    fn pixels_per_cell(&self) -> f32 {
        self.scale / CELLS_PER_UNIT as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tiles::View;

    /// The terrain's land-cell mapping puts a cell where the markers' map-coordinate mapping
    /// puts the same place.
    #[test]
    fn terrain_and_markers_share_the_mapping() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(800.0, 600.0));
        let mut canvas = Canvas::new(&[], None);
        canvas.centre = MapCoord {
            north: 14.3,
            east: 3.3,
        };
        canvas.scale = 37.0;
        let view = Viewport {
            rect,
            centre: canvas.centre,
            scale: canvas.scale,
        };
        for at in [
            MapCoord {
                north: 14.3,
                east: 3.3,
            },
            MapCoord {
                north: -51.8,
                east: 39.0,
            },
        ] {
            let cell = (
                (f64::from(at.east) + 100.0) * 10.2,
                (f64::from(at.north) + 100.0) * 10.2,
            );
            let a = view.cell_to_screen(cell.0, cell.1);
            let b = canvas.to_screen(rect, at);
            assert!((a - b).length() < 0.01, "{a:?} {b:?}");
            let back = view.screen_to_cell(a);
            assert!((back.0 - cell.0).abs() < 0.01 && (back.1 - cell.1).abs() < 0.01);
        }
        assert!((view.pixels_per_cell() - 37.0 / 10.2).abs() < 1e-5);
    }
}
