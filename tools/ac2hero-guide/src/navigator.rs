//! The navigator panel: where the player is and faces, the target, the arrow the client will
//! draw in the viewport (relative to the facing), and the page's coordinates as a route.

use ac2_guide::{coord, nav, MapCoord};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};

use crate::app::Target;

pub struct Navigator {
    here_text: String,
    pub here: Option<MapCoord>,
    /// The player's compass heading in degrees (0 north, 90 east).
    pub facing: f64,
    pub target: Option<Target>,
}

impl Default for Navigator {
    fn default() -> Self {
        Self {
            here_text: String::new(),
            here: None,
            facing: 0.0,
            target: None,
        }
    }
}

impl Navigator {
    pub fn set_here(&mut self, at: MapCoord) {
        self.here = Some(at);
        self.here_text = nav::format(at);
    }

    /// Draws the panel; the page "Open page" asks for.
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        route: &[(String, MapCoord)],
        page: Option<u32>,
    ) -> Option<u32> {
        let mut open = None;
        ui.heading("Navigator");
        ui.label("You are at (as the radar shows it):");
        let r = ui.add(egui::TextEdit::singleline(&mut self.here_text).hint_text("12.3N 45.6E"));
        if r.changed() {
            self.here = coord::find(&self.here_text).first().map(|(_, c)| *c);
        }
        if !self.here_text.is_empty() && self.here.is_none() {
            ui.colored_label(Color32::LIGHT_RED, "Not a map coordinate");
        }
        ui.add(egui::Slider::new(&mut self.facing, 0.0..=359.0).text("facing °"));
        ui.separator();

        match self.target.clone() {
            None => {
                ui.label("No target. Click a gold coordinate on a page or a point on the map.");
            }
            Some(t) => {
                ui.label(RichText::new(format!("Target: {}", nav::format(t.at))).strong());
                ui.label(RichText::new(format!("written \"{}\"", t.text)).weak());
                let cell = nav::cell(t.at);
                ui.label(
                    RichText::new(format!("landblock {:04X} · cell {cell:08X}", cell >> 16))
                        .monospace()
                        .weak(),
                );
                ui.horizontal(|ui| {
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(nav::format(t.at));
                    }
                    if ui.button("Clear").clicked() {
                        self.target = None;
                    }
                    if let Some(p) = t.page.filter(|&p| Some(p) != page) {
                        if ui.button("Open page").clicked() {
                            open = Some(p);
                        }
                    }
                });
                match self.here {
                    Some(here) => {
                        let rel = nav::relative_bearing(here, self.facing, t.at);
                        compass(ui, self.facing, rel);
                        ui.label(format!(
                            "{} · bearing {:.0}° · {}",
                            distance_text(nav::distance(here, t.at)),
                            nav::bearing(here, t.at),
                            turn_text(rel),
                        ));
                    }
                    None => {
                        ui.label(RichText::new("Enter where you are to get the arrow.").weak());
                    }
                }
            }
        }
        ui.separator();

        ui.label(RichText::new("Route (this page's coordinates, in order)").strong());
        if route.is_empty() {
            ui.label(RichText::new("This page has no coordinates.").weak());
            return open;
        }
        let at = self
            .target
            .as_ref()
            .and_then(|t| route.iter().position(|(_, c)| *c == t.at));
        ui.horizontal(|ui| {
            let next = at.map_or(0, |i| i + 1);
            if ui
                .add_enabled(next < route.len(), egui::Button::new("Next stop >"))
                .clicked()
            {
                let (text, c) = &route[next];
                self.target = Some(Target {
                    at: *c,
                    text: text.clone(),
                    page,
                });
            }
        });
        let mut previous = self.here;
        let mut total = 0.0;
        egui::ScrollArea::vertical()
            .id_salt("route")
            .show(ui, |ui| {
                for (i, (text, c)) in route.iter().enumerate() {
                    let leg = previous.map(|p| nav::distance(p, *c));
                    total += leg.unwrap_or(0.0);
                    let label = match leg {
                        Some(d) => format!("{}. {text}   (+{})", i + 1, distance_text(d)),
                        None => format!("{}. {text}", i + 1),
                    };
                    if ui.selectable_label(at == Some(i), label).clicked() {
                        self.target = Some(Target {
                            at: *c,
                            text: text.clone(),
                            page,
                        });
                    }
                    previous = Some(*c);
                }
            });
        ui.label(RichText::new(format!("Total {}", distance_text(total))).weak());
        open
    }
}

fn distance_text(metres: f64) -> String {
    if metres < 1000.0 {
        format!("{metres:.0} m")
    } else {
        format!("{:.1} km", metres / 1000.0)
    }
}

fn turn_text(rel: f64) -> String {
    match rel.abs() {
        a if a < 10.0 => "ahead".into(),
        a if a > 170.0 => "behind".into(),
        a => format!("{a:.0}° {}", if rel < 0.0 { "left" } else { "right" }),
    }
}

/// The viewport arrow as the client will draw it: up is the way the player faces, the arrow
/// points at the target, and the ring's N marks north.
fn compass(ui: &mut egui::Ui, facing: f64, rel: f64) {
    let size = 140.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    let p = ui.painter_at(rect);
    let c = rect.center();
    let r = size * 0.45;
    let dir = |deg: f64| {
        let a = deg.to_radians() as f32;
        Vec2::new(a.sin(), -a.cos())
    };
    let ring = ui.visuals().weak_text_color();
    p.circle_stroke(c, r, Stroke::new(1.5, ring));
    p.text(
        c + dir(-facing) * (r - 10.0),
        egui::Align2::CENTER_CENTER,
        "N",
        egui::FontId::proportional(13.0),
        ring,
    );
    let tip = c + dir(rel) * (r - 6.0);
    let tail = c - dir(rel) * (r * 0.35);
    let gold = Color32::from_rgb(230, 170, 40);
    p.line_segment([tail, tip], Stroke::new(5.0, gold));
    let side = Vec2::new(-dir(rel).y, dir(rel).x) * 10.0;
    let back = tip - dir(rel) * 18.0;
    p.add(egui::Shape::convex_polygon(
        vec![tip, back + side, back - side],
        gold,
        Stroke::NONE,
    ));
    p.circle_filled(c, 3.0, ring);
}
