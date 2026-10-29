//! The world map in the navigation panel (a modern addition that replaces `gmNavigationUI`'s two
//! painted views, kept as an optional background): the land drawn from its own terrain
//! (`world_map_tiles`), finer as it zooms, with the game's map notes, the server's markers, the
//! player and the optional layers (`world_map_layers`) over it.
//!
//! The panel's own elements are moved, not replaced: the map area (`MapParent`) is made taller,
//! the coordinates and the dividers move under it, the time and the date share one line, and the
//! empty box left of the coordinates holds the layers button (a copy of the radar's coordinates
//! button with a town icon). The map's image is drawn on the frame thread from the tiles into the
//! HUD's images under [`MAP_IMAGE`]; notes, markers and labels are elements in the map area. The
//! wheel zooms about the pointer, a drag pans, a press on the player's marker follows the player.
//! The layers list is a sidecar window docked beside the panel (left when the right would leave
//! the screen).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_world::Ac2WorldSource;

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc};
use crate::mouse::{UiMessage, GLOBAL_MOUSE_DOWN, UI_WHEEL_IN, UI_WHEEL_OUT};
use crate::navigation::{MapNote, NavigationUi};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};
use crate::world_map_layers::{
    guide_layers, zone_tint, GuideMarker, NoteKind, WorldNote, Zone, ZONE_CELLS,
};
use crate::world_map_sidecar::{Layer, Sidecar};
use crate::world_map_tiles::{MapTiles, MapWindow};

/// The map area's image in the HUD's images: an id outside the DATs' ranges.
pub const MAP_IMAGE: u32 = 0xF100_0010;
/// The map area's height (retail's is 238, as wide): the time and the date share a line to make
/// the room.
const MAP_HEIGHT: i32 = 289;
/// How much taller the map area is than retail's; everything under it moves down by this.
const GROWN: i32 = MAP_HEIGHT - 238;
/// The height of the row the time and the date share.
const TIME_ROW: i32 = 20;
/// The zoom range in pixels per land cell, and the first view (the land's middle, all of it).
const MIN_ZOOM: f32 = 0.11;
const MAX_ZOOM: f32 = 24.0;
const FIRST_VIEW: MapWindow = MapWindow {
    centre: (1180.0, 1050.0),
    pixels_per_cell: 0.14,
};
/// A wheel notch zooms by this factor.
const WHEEL_ZOOM: f32 = 1.25;
/// How many labels and guide markers are placed at once.
const MAX_LABELS: usize = 30;
const MAX_MARKERS: usize = 160;

/// What the panel shows this frame, from the client.
#[derive(Debug, Clone, Default)]
pub struct WorldMapView {
    /// The player's land position (cells east and north of the south-west corner).
    pub player: Option<(f64, f64)>,
    /// The coordinates text, the time and the date.
    pub coordinates: Option<String>,
    pub time: Option<String>,
    pub date: Option<String>,
    /// The server's markers (`RecvNotice_AddMarker`).
    pub markers: Vec<MapNote>,
}

/// An element the map placed: what it stands for, so it is reused while it stays.
#[derive(Debug, Clone, PartialEq)]
enum Placed {
    Note(usize),
    Server(u64),
    Guide(usize),
    Label(String),
}

pub struct WorldMap {
    /// The gameplay layout, for the copies of its elements (the layers button, the sidecar).
    gameplay: Arc<LayoutDesc>,
    world: Option<Arc<Ac2WorldSource>>,
    tiles: Option<MapTiles>,
    window: MapWindow,
    follow: bool,
    notes: Vec<WorldNote>,
    guide: Option<(Vec<GuideMarker>, Vec<Zone>)>,
    pub(crate) sidecar: Option<Sidecar>,
    layers_button: Option<usize>,
    laid_out: bool,
    /// The elements in the map area, by what they show.
    placed: Vec<(usize, Placed)>,
    /// What the image was last drawn for: the window, the layers and whether tiles arrived.
    drawn: Option<(MapWindow, u64)>,
    tiles_revision: u64,
    /// The wheel notches and the press waiting for the frame; the drag's last pointer.
    wheel: i32,
    press: bool,
    drag: Option<(i32, i32)>,
    classic_image: Option<u32>,
}

impl std::fmt::Debug for WorldMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldMap")
            .field("window", &self.window)
            .field("follow", &self.follow)
            .field("notes", &self.notes.len())
            .field("terrain", &self.tiles.is_some())
            .finish()
    }
}

/// A copy leaves the terrain's tiles and workers behind; it starts its own when first shown.
impl Clone for WorldMap {
    fn clone(&self) -> Self {
        Self {
            gameplay: Arc::clone(&self.gameplay),
            world: self.world.clone(),
            tiles: None,
            window: self.window,
            follow: self.follow,
            notes: self.notes.clone(),
            guide: self.guide.clone(),
            sidecar: self.sidecar.clone(),
            layers_button: self.layers_button,
            laid_out: self.laid_out,
            placed: self.placed.clone(),
            drawn: None,
            tiles_revision: 0,
            wheel: 0,
            press: false,
            drag: None,
            classic_image: self.classic_image,
        }
    }
}

impl WorldMap {
    /// The map with the region's notes (`world_map_layers::load_notes`); the landscape comes later
    /// ([`Self::set_world`]).
    pub fn new(gameplay: Arc<LayoutDesc>, notes: Vec<WorldNote>) -> Self {
        Self {
            gameplay,
            world: None,
            tiles: None,
            window: FIRST_VIEW,
            follow: false,
            notes,
            guide: None,
            sidecar: None,
            layers_button: None,
            laid_out: false,
            placed: Vec::new(),
            drawn: None,
            tiles_revision: 0,
            wheel: 0,
            press: false,
            drag: None,
            classic_image: None,
        }
    }

    /// The landscape the terrain is drawn from (a store of its own, with a small cache).
    pub fn set_world(&mut self, world: Arc<Ac2WorldSource>) {
        self.world = Some(world);
    }

    /// The view's window (for tests).
    pub fn window(&self) -> MapWindow {
        self.window
    }

    /// The layers sidecar's state, when made.
    pub fn sidecar(&self) -> Option<&Sidecar> {
        self.sidecar.as_ref()
    }

    /// The panel's messages: the wheel over the map, a press on the map or on the player's
    /// marker, the layers button and the sidecar's rows. Returns whether one was the map's.
    pub fn message(&mut self, tree: &mut UiTree, nav: &NavigationUi, message: &UiMessage) -> bool {
        if !nav.is_shown(tree) {
            return false;
        }
        if let Some(sidecar) = self.sidecar.as_mut() {
            if sidecar.message(tree, message) {
                self.drawn = None;
                return true;
            }
        }
        if message.element.is_some() && message.element == self.layers_button {
            if message.message == MSG_BUTTON_RELEASE_ON {
                if let Some(sidecar) = self.sidecar.as_mut() {
                    let shown = !sidecar.is_shown();
                    sidecar.show(tree, shown);
                }
            }
            return true;
        }
        let Some(area) = nav.map_parent else {
            return false;
        };
        let rect = tree.screen_box(area);
        let (x, y) = tree.pointer();
        let inside = x >= rect.x0 && x <= rect.x1 && y >= rect.y0 && y <= rect.y1;
        let action = message.data as u32;
        if message.element.is_none()
            && message.message == GLOBAL_MOUSE_DOWN
            && matches!(action, UI_WHEEL_IN | UI_WHEEL_OUT)
            && inside
        {
            self.wheel += if action == UI_WHEEL_IN { 1 } else { -1 };
            return true;
        }
        if message.message == MSG_MOUSE_PRESS && inside {
            if message.element.is_some() && message.element == nav.player_marker {
                self.follow = true;
            } else {
                self.press = true;
            }
            return true;
        }
        false
    }

    /// One frame: the layout made once, the zoom and the drag applied, the image drawn when it
    /// changed, and the notes, markers, labels and the player placed.
    pub fn frame(
        &mut self,
        tree: &mut UiTree,
        nav: &mut NavigationUi,
        images: &mut HashMap<u32, UiPixels>,
        view: &WorldMapView,
    ) {
        if !nav.is_shown(tree) {
            return;
        }
        if !self.laid_out {
            self.lay_out(tree, nav);
        }
        if self.tiles.is_none() {
            // The terrain starts drawing the first time the map is seen.
            self.tiles = self.world.clone().map(MapTiles::new);
        }
        let (Some(area), Some(field)) = (nav.map_parent, nav.map_field) else {
            return;
        };
        let area_box = tree.screen_box(area);
        let (w, h) = (area_box.width(), area_box.height());
        self.input(tree, area_box, w, h);
        if self.follow {
            if let Some(player) = view.player {
                self.window.centre = player;
            }
        }
        self.texts(tree, nav, view);
        let wants_guide = self.sidecar.as_ref().is_some_and(Sidecar::wants_guide);
        if wants_guide && self.guide.is_none() {
            self.guide = Some(guide_layers());
        } else if !wants_guide {
            self.guide = None;
        }
        if self.tiles.as_mut().is_some_and(MapTiles::pump) {
            self.tiles_revision += 1;
        }
        let key = (self.window, self.tiles_revision ^ self.layer_bits());
        if self.drawn != Some(key) {
            self.drawn = Some(key);
            let pixels = self.render(images, w, h);
            images.insert(
                MAP_IMAGE,
                UiPixels {
                    width: w as u32,
                    height: h as u32,
                    rgba: pixels.into(),
                },
            );
            // A new image under the same id: set again so the tree redraws it.
            tree.set_image(field, 0, 0);
            tree.set_image(field, MAP_IMAGE, 0);
            tree.move_to(field, 0, 0);
            tree.resize_to(field, w, h);
        }
        self.place(tree, nav, view, w, h);
        if let Some(sidecar) = self.sidecar.as_mut() {
            sidecar.dock(tree, nav.field);
        }
    }

    fn layer_bits(&self) -> u64 {
        self.sidecar.as_ref().map_or(0, Sidecar::bits)
    }

    fn layer(&self, layer: Layer) -> bool {
        self.sidecar
            .as_ref()
            .map_or(layer.default_on(), |s| s.on(layer))
    }

    /// The wheel zooms about the pointer; a held press drags the map.
    fn input(&mut self, tree: &UiTree, area: crate::element::UiBox, w: i32, h: i32) {
        let (px, py) = tree.pointer();
        let local = (f64::from(px - area.x0), f64::from(py - area.y0));
        if self.wheel != 0 {
            let before = self.window.cell_at(w, h, local.0, local.1);
            let zoom = WHEEL_ZOOM.powi(self.wheel);
            self.window.pixels_per_cell =
                (self.window.pixels_per_cell * zoom).clamp(MIN_ZOOM, MAX_ZOOM);
            let after = self.window.cell_at(w, h, local.0, local.1);
            self.window.centre.0 += before.0 - after.0;
            self.window.centre.1 += before.1 - after.1;
            self.wheel = 0;
            self.follow = false;
        }
        if std::mem::take(&mut self.press) {
            self.drag = Some((px, py));
        }
        match self.drag {
            Some((lx, ly)) if tree.drag.left_down => {
                if (lx, ly) != (px, py) {
                    let s = f64::from(self.window.pixels_per_cell);
                    self.window.centre.0 -= f64::from(px - lx) / s;
                    self.window.centre.1 += f64::from(py - ly) / s;
                    self.follow = false;
                }
                self.drag = Some((px, py));
            }
            _ => self.drag = None,
        }
        let world = 2040.0;
        self.window.centre.0 = self.window.centre.0.clamp(0.0, world);
        self.window.centre.1 = self.window.centre.1.clamp(0.0, world);
    }

    /// The map image: the terrain (or the painted maps for the classic background) with the
    /// zone tints over it.
    fn render(&mut self, images: &HashMap<u32, UiPixels>, w: i32, h: i32) -> Vec<u8> {
        let mut pixels = match (self.layer(Layer::Classic), self.classic_image) {
            (true, Some(did)) => classic(images.get(&did), &self.window, w, h),
            _ => match self.tiles.as_mut() {
                Some(tiles) => tiles.render(&self.window, w, h),
                None => vec![255u8; (w * h * 4).max(0) as usize],
            },
        };
        if self.layer(Layer::ZoneLevels) {
            if let Some((_, zones)) = &self.guide {
                tint_zones(&mut pixels, zones, &self.window, w, h);
            }
        }
        pixels
    }

    /// The coordinates, and the time and the date on one line.
    fn texts(&self, tree: &mut UiTree, nav: &NavigationUi, view: &WorldMapView) {
        if let Some(coords) = nav.child(tree, "CoordinatesText") {
            let text = view.coordinates.clone().unwrap_or_default();
            if tree.get(coords).and_then(|e| e.text()) != Some(text.clone()) {
                tree.set_text(coords, &text);
            }
        }
        if let Some(time) = nav.child(tree, "TimeText") {
            let line = match (&view.time, &view.date) {
                (Some(t), Some(d)) => format!("{t} \u{2022} {d}"),
                (Some(t), None) => t.clone(),
                (None, Some(d)) => d.clone(),
                (None, None) => String::new(),
            };
            if tree.get(time).and_then(|e| e.text()) != Some(line.clone()) {
                tree.set_text(time, &line);
            }
        }
    }

    /// The panel's elements moved for the taller map, the painted views' parts hidden, the
    /// layers button and the sidecar made.
    fn lay_out(&mut self, tree: &mut UiTree, nav: &mut NavigationUi) {
        self.laid_out = true;
        let (Some(parent), Some(area)) = (nav.parent, nav.map_parent) else {
            return;
        };
        self.classic_image = nav
            .map_field
            .and_then(|f| tree.get(f)?.image())
            .map(|i| i.file);
        let area_rect = tree.get(area).map(|e| e.rect());
        if let Some(r) = area_rect {
            tree.resize_to(area, r.width(), MAP_HEIGHT);
        }
        for tile in nav.tiles.iter().flatten() {
            tree.set_visible(*tile, false);
        }
        for button in nav
            .overview_buttons
            .iter()
            .chain(nav.scroll.iter())
            .flatten()
        {
            tree.set_visible(*button, false);
        }
        // Everything in the panel under the map moves down by `GROWN`; the date's line goes.
        let children: Vec<usize> = tree.children(parent).to_vec();
        let mut coordinates_row_left = None;
        for child in children {
            if child == area {
                continue;
            }
            let Some(rect) = tree.get(child).map(|e| e.rect()) else {
                continue;
            };
            let below = area_rect.is_some_and(|a| rect.y0 >= a.y1 - 2);
            if !below {
                continue;
            }
            tree.move_to(child, rect.x0, rect.y0 + GROWN);
            let plain = tree.children(child).is_empty()
                && tree.get(child).and_then(|e| e.image()).is_none();
            if rect.x0 == 0 && plain {
                coordinates_row_left = Some(child);
            }
        }
        if let Some(date) = nav.child(tree, "DateText") {
            tree.set_visible(date, false);
        }
        // The time and the date share one short row right under the second divider (retail's
        // two rows sat 8 pixels under it; the row sits 2 under it here).
        if let Some(time) = nav.child(tree, "TimeText") {
            if let Some(group) = tree.get(time).and_then(|e| e.parent_index()) {
                if let Some(r) = tree.get(group).map(|e| e.rect()) {
                    tree.move_to(group, r.x0, r.y0 - 6);
                    tree.resize_to(group, r.width(), TIME_ROW);
                }
                if let Some(r) = tree.get(time).map(|e| e.rect()) {
                    tree.move_to(time, r.x0, 0);
                    tree.resize_to(time, r.width(), TIME_ROW);
                }
            }
        }
        // The layers button in the box left of the coordinates.
        if let Some(holder) = coordinates_row_left {
            self.layers_button = layers_button(tree, &self.gameplay, holder);
        }
        self.sidecar = Sidecar::create(tree, &self.gameplay, &self.notes);
    }

    /// The notes, the server's markers, the guide's places, the labels and the player, placed
    /// in the map area by the view's window; elements no longer needed are deleted.
    fn place(
        &mut self,
        tree: &mut UiTree,
        nav: &NavigationUi,
        view: &WorldMapView,
        w: i32,
        h: i32,
    ) {
        let Some(area) = nav.map_parent else {
            return;
        };
        let ppc = self.window.pixels_per_cell;
        let mut wanted: Vec<(Placed, u32, (f64, f64), Option<String>)> = Vec::new();
        let inside = |p: (f64, f64)| {
            p.0 >= -8.0 && p.1 >= -8.0 && p.0 < f64::from(w) + 8.0 && p.1 < f64::from(h) + 8.0
        };
        let mut labels: Vec<(String, (f64, f64), bool)> = Vec::new();
        for (i, note) in self.notes.iter().enumerate() {
            if !self.layer(Layer::Note(note.kind)) || ppc < note.kind.shown_from() {
                continue;
            }
            let at = self.window.pixel_of(
                w,
                h,
                f64::from(note.land.0) + 0.5,
                f64::from(note.land.1) + 0.5,
            );
            if !inside(at) {
                continue;
            }
            wanted.push((Placed::Note(i), note.icon, at, Some(note.name.clone())));
            if ppc >= note.kind.named_from() && !note.name.is_empty() {
                labels.push((note.name.clone(), at, note.kind == NoteKind::Town));
            }
        }
        for marker in &view.markers {
            let Some(land) = marker.land else { continue };
            let at = self
                .window
                .pixel_of(w, h, f64::from(land.0) + 0.5, f64::from(land.1) + 0.5);
            if inside(at) && marker.image != 0 {
                wanted.push((
                    Placed::Server(marker.id),
                    marker.image,
                    at,
                    Some(marker.message.clone()),
                ));
            }
        }
        if let Some((markers, zones)) = &self.guide {
            let mut count = 0;
            for (i, marker) in markers.iter().enumerate() {
                if count >= MAX_MARKERS || !self.layer(Layer::Guide(marker.layer)) {
                    continue;
                }
                let at = self.window.pixel_of(w, h, marker.land.0, marker.land.1);
                if inside(at) {
                    wanted.push((
                        Placed::Guide(i),
                        marker.layer.icon(),
                        at,
                        Some(marker.tooltip.clone()),
                    ));
                    count += 1;
                }
            }
            if self.layer(Layer::ZoneLevels) {
                for zone in zones {
                    let centre = (
                        f64::from(zone.square.0 * ZONE_CELLS + ZONE_CELLS / 2),
                        f64::from(zone.square.1 * ZONE_CELLS + ZONE_CELLS / 2),
                    );
                    let at = self.window.pixel_of(w, h, centre.0, centre.1);
                    if inside(at) {
                        let text = if zone.low == zone.high {
                            format!("Level {}", zone.low)
                        } else {
                            format!("Levels {}-{}", zone.low, zone.high)
                        };
                        labels.push((text, at, false));
                    }
                }
            }
        }
        // Towns' names first; a name that would cover one already placed is left out.
        labels.sort_by_key(|l| !l.2);
        let mut taken: Vec<(i32, i32, i32, i32)> = Vec::new();
        let label_desc = nav.toolbar.find_element(tokenize("CoordinatesText"));
        for (text, at, _) in labels {
            if taken.len() >= MAX_LABELS {
                break;
            }
            let width = (text.chars().count() as i32) * 7 + 4;
            // Right of the mark, or left of it where the right would run off the map.
            let x = if at.0 as i32 + 7 + width <= w {
                at.0 as i32 + 7
            } else {
                (at.0 as i32 - 7 - width).max(0)
            };
            let rect = (x, at.1 as i32 - 7, width, 14);
            if taken.iter().any(|t| {
                rect.0 < t.0 + t.2
                    && t.0 < rect.0 + rect.2
                    && rect.1 < t.1 + t.3
                    && t.1 < rect.1 + rect.3
            }) {
                continue;
            }
            taken.push(rect);
            wanted.push((
                Placed::Label(text),
                0,
                (f64::from(rect.0), f64::from(rect.1)),
                None,
            ));
        }
        // Reuse the elements that show the same thing; make the rest; delete the leftovers.
        let mut old = std::mem::take(&mut self.placed);
        let mut next = Vec::with_capacity(wanted.len());
        for (what, icon, at, tooltip) in wanted {
            let element = match old.iter().position(|(_, p)| *p == what) {
                Some(i) => Some(old.swap_remove(i).0),
                None => match &what {
                    Placed::Label(text) => label_desc
                        .clone()
                        .and_then(|d| make_label(tree, nav, area, d, text)),
                    _ => make_marker(tree, nav, area, icon, tooltip),
                },
            };
            let Some(element) = element else { continue };
            let (ew, eh) = tree
                .get(element)
                .map_or((0, 0), |e| (e.rect().width(), e.rect().height()));
            let (x, y) = match what {
                Placed::Label(_) => (at.0 as i32, at.1 as i32),
                _ => (at.0 as i32 - ew / 2, at.1 as i32 - eh / 2),
            };
            if tree
                .get(element)
                .is_some_and(|e| (e.rect().x0, e.rect().y0) != (x, y))
            {
                tree.move_to(element, x, y);
            }
            next.push((element, what));
        }
        for (element, _) in old {
            tree.set_parent(element, None);
            tree.delete(element);
        }
        self.placed = next;
        // The player's marker on top of the rest.
        if let Some(marker) = nav.player_marker {
            match view.player {
                Some(player) => {
                    let at = self.window.pixel_of(w, h, player.0, player.1);
                    let (mw, mh) = tree
                        .get(marker)
                        .map_or((0, 0), |e| (e.rect().width(), e.rect().height()));
                    tree.set_visible(marker, true);
                    tree.move_to(marker, at.0 as i32 - mw / 2, at.1 as i32 - mh / 2);
                }
                None => tree.set_visible(marker, false),
            }
        }
    }
}

/// A marker element (the toolbar's `MarkerTemplate`) in the map area with `icon` and a tooltip.
fn make_marker(
    tree: &mut UiTree,
    nav: &NavigationUi,
    area: usize,
    icon: u32,
    tooltip: Option<String>,
) -> Option<usize> {
    let template = nav.marker_template.clone()?;
    let element = tree.create_element(Arc::clone(&nav.toolbar), template, false)?;
    tree.set_parent(element, Some(area));
    if icon != 0 {
        tree.set_image(element, icon, crate::navigation::MAP_DRAW_MODE);
    }
    tree.set_tooltip(element, tooltip);
    Some(element)
}

/// A name in the map area: a copy of the coordinates text, sized to its words.
fn make_label(
    tree: &mut UiTree,
    nav: &NavigationUi,
    area: usize,
    desc: Arc<ElementDesc>,
    text: &str,
) -> Option<usize> {
    let mut desc = (*desc).clone();
    desc.id = tokenize("WorldMapCaption");
    let element = tree.create_element(Arc::clone(&nav.toolbar), Arc::new(desc), false)?;
    tree.set_parent(element, Some(area));
    let (w, h) = tree
        .text_size_with_margins(element, text)
        .unwrap_or((60, 14));
    tree.resize_to(element, w + 2, h);
    tree.set_text(element, text);
    Some(element)
}

/// The layers button: a copy of the radar's coordinates button in `holder`, with a town icon.
fn layers_button(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>, holder: usize) -> Option<usize> {
    let template = gameplay.find_element(tokenize("CoordBoxButton"))?;
    let mut desc: ElementDesc = (*template).clone();
    desc.id = tokenize("WorldMapLayersOrb");
    let layout = Arc::clone(gameplay);
    let button = tree.create_element(Arc::clone(&layout), Arc::new(desc), false)?;
    tree.set_parent(button, Some(holder));
    let size = (
        i32::from(template.rect.x1) - i32::from(template.rect.x0) + 1,
        i32::from(template.rect.y1) - i32::from(template.rect.y0) + 1,
    );
    tree.move_to(button, 6, 5);
    tree.resize_to(button, size.0, size.1);
    let mut icon: ElementDesc = (*template).clone();
    icon.id = tokenize("WorldMapLayersArt");
    icon.states.clear();
    if let Some(art) = tree.create_element(layout, Arc::new(icon), false) {
        tree.set_parent(art, Some(button));
        tree.move_to(art, 2, 2);
        tree.resize_to(art, size.0 - 4, size.1 - 4);
        tree.set_image(art, 0x4100_512C, 2);
        tree.set_bool(art, crate::token::SIZE_IMAGE_TO_OWNER, true);
    }
    tree.set_tooltip(button, Some("Map layers".to_owned()));
    Some(button)
}

/// The painted game map (`CRegionDesc::GetGameMap`) under the window, placed as the retail macro
/// view places it (`MacroXOffset` 321, `MacroYOffset` 195, 1720 cells across).
fn classic(image: Option<&UiPixels>, window: &MapWindow, w: i32, h: i32) -> Vec<u8> {
    const ORIGIN: (f64, f64) = (321.0, 195.0);
    const SIZE: f64 = 1720.0;
    let mut out = vec![255u8; (w * h * 4).max(0) as usize];
    for py in 0..h {
        for px in 0..w {
            let o = ((py * w + px) * 4) as usize;
            let (cx, cy) = window.cell_at(w, h, f64::from(px) + 0.5, f64::from(py) + 0.5);
            let rgb = image.and_then(|img| {
                let u = (cx - ORIGIN.0) / SIZE;
                let v = 1.0 - (cy - ORIGIN.1) / SIZE;
                if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
                    return None;
                }
                let x = (u * f64::from(img.width)) as u32;
                let y = (v * f64::from(img.height)) as u32;
                let s = ((y * img.width + x) * 4) as usize;
                Some([img.rgba[s], img.rgba[s + 1], img.rgba[s + 2]])
            });
            out[o..o + 3].copy_from_slice(&rgb.unwrap_or([96, 84, 60]));
        }
    }
    out
}

/// The zone squares' tints over the map, by their typical level.
fn tint_zones(pixels: &mut [u8], zones: &[Zone], window: &MapWindow, w: i32, h: i32) {
    let tints: HashMap<(i32, i32), [u8; 3]> = zones
        .iter()
        .map(|z| (z.square, zone_tint(z.median)))
        .collect();
    for py in 0..h {
        for px in 0..w {
            let (cx, cy) = window.cell_at(w, h, f64::from(px) + 0.5, f64::from(py) + 0.5);
            let square = (
                (cx as i32).div_euclid(ZONE_CELLS),
                (cy as i32).div_euclid(ZONE_CELLS),
            );
            if let Some(tint) = tints.get(&square) {
                let o = ((py * w + px) * 4) as usize;
                for c in 0..3 {
                    pixels[o + c] =
                        ((u16::from(pixels[o + c]) * 3 + u16::from(tint[c]) * 2) / 5) as u8;
                }
            }
        }
    }
}

impl Layer {
    /// Whether the layer is on before the player changes it: the game's notes are, the guide's
    /// layers, the zone levels and the classic background are not.
    pub fn default_on(self) -> bool {
        matches!(self, Layer::Note(_))
    }
}

#[cfg(test)]
#[path = "world_map_tests.rs"]
mod tests;
