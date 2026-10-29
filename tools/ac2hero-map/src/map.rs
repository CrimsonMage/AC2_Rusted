//! The world map widget: the terrain (`tiles`), the game's own map notes (`notes`, on by
//! default), the AC2Hero guide's places (off by default) and the player, with the layer toggles
//! and the picked place beside the map. The standalone tool and the world viewer both show it.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_guide::atlas::{self, Marker, MarkerKind};
use ac2_guide::{nav, Guide, MapCoord};
use ac2_world::Ac2WorldSource;
use eframe::egui::{self, RichText, Sense, Vec2};

use crate::canvas::{Canvas, Event, Player, Scene};
use crate::notes::{self, Note, NoteKind};
use crate::panel;
use crate::tiles::Tiles;

/// What a click on the map asks its host for.
pub enum MapAction {
    /// Travel to a map coordinate (the viewer teleports there).
    Travel(MapCoord),
}

/// What the map shows and which place is picked.
pub struct Filter {
    pub layers: Vec<(MarkerKind, bool)>,
    pub levels: (u32, u32),
    pub search: String,
    pub area: String,
}

impl Filter {
    pub fn shows(&self, m: &Marker, guide: &Guide) -> bool {
        let layer = self.layers.iter().any(|&(k, on)| on && k == m.kind);
        let level = m
            .level
            .is_none_or(|l| l >= self.levels.0 && l <= self.levels.1);
        let area = self.area.is_empty() || guide.pages[m.page as usize].area == self.area;
        let search = self.search.trim().to_lowercase();
        let text = search.is_empty()
            || m.title.to_lowercase().contains(&search)
            || m.context.to_lowercase().contains(&search);
        layer && level && area && text
    }
}

pub struct WorldMap {
    guide: Guide,
    markers: Vec<Marker>,
    filter: Filter,
    areas: Vec<String>,
    canvas: Canvas,
    /// The picked guide markers (one place, or a cluster's members).
    picked: Vec<usize>,
    notes: Vec<Note>,
    note_layers: Vec<bool>,
    note_icons: HashMap<u32, egui::TextureHandle>,
    picked_note: Option<usize>,
    /// Keep the player in the middle.
    follow: bool,
    can_travel: bool,
    side_panel: bool,
    bare: bool,
}

impl WorldMap {
    /// The map over `world` (no terrain or notes without it) with `guide`'s places. The terrain
    /// starts drawing on worker threads now.
    pub fn new(ctx: &egui::Context, guide: Guide, world: Option<Arc<Ac2WorldSource>>) -> Self {
        let markers = atlas::markers(&guide);
        let mut areas: Vec<String> = markers
            .iter()
            .map(|m| guide.pages[m.page as usize].area.clone())
            .filter(|a| !a.is_empty())
            .collect();
        areas.sort();
        areas.dedup();
        let (notes, icons) = match &world {
            Some(w) => notes::load(w.resource_store()),
            None => Default::default(),
        };
        let note_icons = icons
            .into_iter()
            .map(|(did, (w, h, rgba))| {
                let image =
                    egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                (
                    did,
                    ctx.load_texture(
                        format!("note icon {did:08X}"),
                        image,
                        egui::TextureOptions::NEAREST,
                    ),
                )
            })
            .collect();
        Self {
            canvas: Canvas::new(&markers, world.map(|w| Tiles::new(ctx, w))),
            guide,
            markers,
            filter: Filter {
                layers: MarkerKind::ALL.iter().map(|&k| (k, false)).collect(),
                levels: (1, 100),
                search: String::new(),
                area: String::new(),
            },
            areas,
            picked: Vec::new(),
            notes,
            note_layers: vec![true; NoteKind::ALL.len()],
            note_icons,
            picked_note: None,
            follow: false,
            can_travel: false,
            side_panel: true,
            bare: false,
        }
    }

    /// Shows `at` at `pixels_per_cell` screen pixels per land cell.
    pub fn set_view(&mut self, at: MapCoord, pixels_per_cell: f32) {
        self.canvas.set_view(at, pixels_per_cell);
    }

    /// Whether the terrain is still being drawn (a screenshot waits for it).
    pub fn busy(&self) -> bool {
        self.canvas.busy()
    }

    /// The map alone: no side panel, toggle, grid or pointer readout (the panel-sized preview).
    pub fn set_bare(&mut self) {
        self.side_panel = false;
        self.bare = true;
        self.canvas.chrome = false;
    }

    /// Whether a right click offers `MapAction::Travel`.
    pub fn set_travel(&mut self, on: bool) {
        self.can_travel = on;
    }

    /// Draws the map in `ui`, with the player when known; what a click asked the host for.
    pub fn ui(&mut self, ui: &mut egui::Ui, player: Option<Player>) -> Option<MapAction> {
        if self.follow {
            if let Some(p) = player {
                self.canvas.centre_on(p.at);
            }
        }
        let shown: Vec<usize> = (0..self.markers.len())
            .filter(|&i| self.filter.shows(&self.markers[i], &self.guide))
            .collect();
        let mut focus = None;
        if self.side_panel {
            egui::Panel::left("world map layers")
                .resizable(true)
                .default_size(300.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("map side")
                        .show(ui, |ui| {
                            focus = self.side(ui, player, &shown);
                        });
                });
        }
        if let Some(at) = focus {
            self.canvas.focus(at);
        }
        let mut action = None;
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let toggle = if self.side_panel { "<" } else { "> Layers" };
            let scene = Scene {
                markers: &self.markers,
                shown: &shown,
                picked: &self.picked,
                notes: &self.notes,
                note_layers: &self.note_layers,
                note_icons: &self.note_icons,
                picked_note: self.picked_note,
                player,
                can_travel: self.can_travel,
            };
            let top = ui.max_rect().left_top();
            match self.canvas.ui(ui, &scene) {
                Some(Event::Pick(p)) => {
                    self.picked = p;
                    self.picked_note = None;
                }
                Some(Event::PickNote(n)) => {
                    self.picked_note = Some(n);
                    self.picked.clear();
                }
                Some(Event::Travel(at)) => action = Some(MapAction::Travel(at)),
                None => {}
            }
            if !self.bare {
                let button = egui::Area::new(ui.scope_id().with("layers toggle"))
                    .fixed_pos(top + Vec2::new(8.0, 8.0))
                    .show(ui.ctx(), |ui| ui.button(toggle));
                if button.inner.clicked() {
                    self.side_panel = !self.side_panel;
                }
            }
        });
        action
    }

    /// The side panel: player, the map's layers, the guide's layers and the picked place; a
    /// place to centre on.
    fn side(
        &mut self,
        ui: &mut egui::Ui,
        player: Option<Player>,
        shown: &[usize],
    ) -> Option<MapCoord> {
        let mut focus = None;
        ui.heading("World map");
        if let Some(p) = player {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("You: {}", nav::format(p.at))).monospace());
                if ui.small_button("Centre").clicked() {
                    focus = Some(p.at);
                }
            });
            ui.checkbox(&mut self.follow, "Follow me");
        }
        ui.separator();
        ui.label(RichText::new("Map").strong());
        for (i, kind) in NoteKind::ALL.iter().enumerate() {
            let count = self.notes.iter().filter(|n| n.kind == *kind).count();
            if count == 0 {
                continue;
            }
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                let icon = self
                    .notes
                    .iter()
                    .find(|n| n.kind == *kind)
                    .and_then(|n| self.note_icons.get(&n.icon));
                if let Some(icon) = icon {
                    ui.painter().image(
                        icon.id(),
                        rect,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                ui.checkbox(
                    &mut self.note_layers[i],
                    format!("{} ({count})", kind.label()),
                );
            });
        }
        if let Some(n) = self.picked_note.and_then(|n| self.notes.get(n)) {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(&n.name).strong());
                ui.label(
                    RichText::new(format!("{} · {}", n.kind.label(), nav::format(n.at))).weak(),
                );
                ui.horizontal(|ui| {
                    if ui.small_button("Centre").clicked() {
                        focus = Some(n.at);
                    }
                    if ui.small_button("Copy coordinates").clicked() {
                        ui.ctx().copy_text(nav::format(n.at));
                    }
                });
            });
        }
        ui.separator();
        if let Some(i) = panel::show(
            ui,
            &self.guide,
            &self.markers,
            shown,
            &mut self.filter,
            &self.areas,
            &mut self.picked,
        ) {
            focus = Some(self.markers[i].at);
        }
        focus
    }
}

/// The player on the map from a client position: the land cell's block and the offset in it
/// (`origin` in metres), and the compass heading.
pub fn player_at(cell: u32, origin: [f32; 3], heading: f32) -> Player {
    let bx = f64::from((cell >> 24) & 0xFF);
    let by = f64::from((cell >> 16) & 0xFF);
    // Inside a building the origin is the interior's, so place the player at the block's middle.
    let inside = (cell & 0xFFFF) >= 0x100;
    let (ox, oy) = if inside {
        (80.0, 80.0)
    } else {
        (f64::from(origin[0]), f64::from(origin[1]))
    };
    let lx = bx * 8.0 + ox / 20.0;
    let ly = by * 8.0 + oy / 20.0;
    Player {
        at: MapCoord {
            north: (ly / 10.2 - 100.0) as f32,
            east: (lx / 10.2 - 100.0) as f32,
        },
        heading,
    }
}

/// Metres east and north of the world's south-west corner for a map coordinate (the viewer's
/// teleport takes these).
pub fn world_metres(at: MapCoord) -> (f32, f32) {
    let cells = |v: f32| (f64::from(v) + 100.0) * 10.2;
    (
        (cells(at.east) * 20.0) as f32,
        (cells(at.north) * 20.0) as f32,
    )
}

/// The landscape over the DATs under `dir`, with a small record cache (the map sweeps every
/// landblock once, then reads only the ones in view); `Ok(None)` when they hold no AC2 landscape.
pub fn open_world(dir: &std::path::Path) -> Result<Option<Arc<Ac2WorldSource>>, String> {
    const MAP_CACHE_BYTES: usize = 8 * 1024 * 1024;
    let dataset = ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(dir))
        .map_err(|e| format!("{e:?}"))?;
    let store = Arc::new(ac2_db::ResourceStore::with_free_cache_budget(
        dataset,
        MAP_CACHE_BYTES,
    ));
    Ac2WorldSource::open(store).map_err(|e| format!("{e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_round_trip() {
        // Block 0x7F7F, 90 m east and 30 m north into it.
        let p = player_at(0x7F7F_0021, [90.0, 30.0, 0.0], 45.0);
        let (x, y) = world_metres(p.at);
        assert!((x - (127.0 * 160.0 + 90.0)).abs() < 0.05, "{x}");
        assert!((y - (127.0 * 160.0 + 30.0)).abs() < 0.05, "{y}");
    }
}
