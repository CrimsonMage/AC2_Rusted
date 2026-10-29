//! The route map: the world's -100..100 map square with every NPC (and optionally monster)
//! location, the open page's coordinates joined in reading order, the player and the target.
//! Drag pans, the wheel zooms, a click on a point targets it, a double click opens its page,
//! and a right click sets "you are here".

use ac2_guide::{nav, Guide, Kind, MapCoord};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use crate::app::{page_summary, Target};
use crate::navigator::Navigator;

pub struct MapView {
    /// The map coordinate at the middle of the view.
    centre: MapCoord,
    /// Screen points per map unit; 0 until the first frame fits the map.
    scale: f32,
    show_npcs: bool,
    show_monsters: bool,
    points: Vec<(MapCoord, u32, Kind, String)>,
}

const GOLD: Color32 = Color32::from_rgb(230, 170, 40);

impl MapView {
    pub fn new(guide: &Guide) -> Self {
        let points = guide
            .pages
            .iter()
            .enumerate()
            .filter(|(_, p)| matches!(p.kind, Kind::Npc | Kind::Monster))
            .flat_map(|(i, p)| {
                p.coords()
                    .map(move |(_, c)| (c, i as u32, p.kind, p.title.clone()))
            })
            .collect();
        Self {
            centre: MapCoord {
                north: 0.0,
                east: 0.0,
            },
            scale: 0.0,
            show_npcs: true,
            show_monsters: false,
            points,
        }
    }

    /// Draws the map; the page a double click asks to open.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        guide: &Guide,
        route: &[(String, MapCoord)],
        nav: &mut Navigator,
    ) -> Option<u32> {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_npcs, "NPCs");
            ui.checkbox(&mut self.show_monsters, "Monsters");
            if ui.button("Fit world").clicked() {
                self.scale = 0.0;
            }
            if ui.button("Fit route").clicked() {
                self.fit(route.iter().map(|(_, c)| *c).chain(nav.here), ui.available_size());
            }
            ui.label(egui::RichText::new("drag pan · wheel zoom · click target · double-click open · right-click: I am here").weak());
        });
        let (response, painter) =
            ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        if self.scale == 0.0 {
            self.centre = MapCoord {
                north: 0.0,
                east: 0.0,
            };
            self.scale = rect.width().min(rect.height()) / 210.0;
        }
        if response.dragged() {
            let d = response.drag_delta() / self.scale;
            self.centre.east -= d.x;
            self.centre.north += d.y;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                let zoom = (scroll * 0.002).exp();
                if let Some(pointer) = response.hover_pos() {
                    let before = self.to_map(rect, pointer);
                    self.scale = (self.scale * zoom).clamp(0.5, 400.0);
                    let after = self.to_map(rect, pointer);
                    self.centre.east += before.east - after.east;
                    self.centre.north += before.north - after.north;
                }
            }
        }

        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
        self.grid(&painter, rect, ui);

        let mut hits: Vec<(Pos2, Target, Option<u32>, String)> = Vec::new();
        for (c, page, kind, title) in &self.points {
            let shown = match kind {
                Kind::Npc => self.show_npcs,
                _ => self.show_monsters,
            };
            if !shown {
                continue;
            }
            let p = self.to_screen(rect, *c);
            if !rect.contains(p) {
                continue;
            }
            let colour = if *kind == Kind::Npc {
                Color32::from_rgb(90, 160, 230)
            } else {
                Color32::from_rgb(200, 80, 80)
            };
            painter.circle_filled(p, 2.5, colour.gamma_multiply(0.8));
            let target = Target {
                at: *c,
                text: nav::format(*c),
                page: Some(*page),
            };
            hits.push((p, target, Some(*page), title.clone()));
        }

        let screen: Vec<Pos2> = route
            .iter()
            .map(|(_, c)| self.to_screen(rect, *c))
            .collect();
        if screen.len() > 1 {
            painter.add(egui::Shape::dashed_line(
                &screen,
                Stroke::new(2.0, GOLD),
                8.0,
                5.0,
            ));
        }
        for (i, ((text, c), p)) in route.iter().zip(&screen).enumerate() {
            painter.circle(*p, 9.0, ui.visuals().panel_fill, Stroke::new(2.0, GOLD));
            painter.text(
                *p,
                egui::Align2::CENTER_CENTER,
                (i + 1).to_string(),
                egui::FontId::proportional(11.0),
                GOLD,
            );
            hits.push((
                *p,
                Target {
                    at: *c,
                    text: text.clone(),
                    page: None,
                },
                None,
                format!("stop {}: {text}", i + 1),
            ));
        }

        if let Some(here) = nav.here {
            let p = self.to_screen(rect, here);
            let a = nav.facing.to_radians() as f32;
            let fwd = Vec2::new(a.sin(), -a.cos());
            let side = Vec2::new(-fwd.y, fwd.x);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    p + fwd * 11.0,
                    p - fwd * 7.0 + side * 7.0,
                    p - fwd * 7.0 - side * 7.0,
                ],
                Color32::from_rgb(120, 220, 120),
                Stroke::new(1.0, Color32::BLACK),
            ));
            if let Some(t) = &nav.target {
                painter.line_segment(
                    [p, self.to_screen(rect, t.at)],
                    Stroke::new(1.0, Color32::from_rgb(120, 220, 120)),
                );
            }
        }
        if let Some(t) = &nav.target {
            let p = self.to_screen(rect, t.at);
            painter.circle_stroke(p, 14.0, Stroke::new(2.5, Color32::WHITE));
            painter.circle_stroke(p, 18.0, Stroke::new(1.5, GOLD));
        }

        let pointer = response.hover_pos();
        let nearest = pointer.and_then(|m| {
            hits.iter()
                .map(|h| (h.0.distance(m), h))
                .filter(|(d, _)| *d < 10.0)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, h)| h)
        });
        if let Some(m) = pointer {
            let at = self.to_map(rect, m);
            painter.text(
                rect.left_bottom() + Vec2::new(8.0, -8.0),
                egui::Align2::LEFT_BOTTOM,
                nav::format(at),
                egui::FontId::monospace(13.0),
                ui.visuals().text_color(),
            );
        }
        let mut open = None;
        if let Some((_, target, page, title)) = nearest {
            let tip = match page {
                Some(p) => format!(
                    "{title}\n{}\n\n{}",
                    nav::format(target.at),
                    page_summary(&guide.pages[*p as usize])
                ),
                None => title.clone(),
            };
            let response = response.clone().on_hover_text(tip);
            if response.double_clicked() {
                open = *page;
            } else if response.clicked() {
                nav.target = Some(target.clone());
            }
        }
        if response.secondary_clicked() {
            if let Some(m) = pointer {
                nav.set_here(self.to_map(rect, m));
            }
        }
        open
    }

    fn fit(&mut self, coords: impl Iterator<Item = MapCoord>, size: Vec2) {
        let pts: Vec<MapCoord> = coords.collect();
        if pts.is_empty() {
            return;
        }
        let (mut lo, mut hi) = (pts[0], pts[0]);
        for c in &pts {
            lo.east = lo.east.min(c.east);
            lo.north = lo.north.min(c.north);
            hi.east = hi.east.max(c.east);
            hi.north = hi.north.max(c.north);
        }
        self.centre = MapCoord {
            north: (lo.north + hi.north) * 0.5,
            east: (lo.east + hi.east) * 0.5,
        };
        let span = (hi.east - lo.east).max(hi.north - lo.north).max(4.0) * 1.3;
        self.scale = (size.x.min(size.y) / span).clamp(0.5, 400.0);
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

    /// The map's edge and a line every 10 units (every 1 when zoomed in), labelled at the edges.
    fn grid(&self, painter: &egui::Painter, rect: Rect, ui: &egui::Ui) {
        let step = if self.scale > 40.0 {
            1.0
        } else if self.scale > 8.0 {
            5.0
        } else {
            10.0
        };
        let faint = ui.visuals().weak_text_color().gamma_multiply(0.35);
        let label = ui.visuals().weak_text_color();
        let font = egui::FontId::proportional(10.0);
        let mut v = -100.0;
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
            let major = v == 0.0 || v.abs() == 100.0;
            let stroke = Stroke::new(
                if major { 1.5 } else { 0.5 },
                if major { label } else { faint },
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
                    egui::Align2::LEFT_TOP,
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
                    egui::Align2::LEFT_BOTTOM,
                    t,
                    font.clone(),
                    label,
                );
            }
            v += step;
        }
    }
}
