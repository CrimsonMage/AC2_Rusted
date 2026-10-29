//! The navigation panel's frame input (`gmNavigationUI::Update`): the player's land cell, the
//! coordinates the radar renders, and the map notes with their land cells and rendered text.
//! The panel is [`HudView::navigation`]; its tree changes are composed with the frame's
//! `finish_frame`.

use ac2_physics::landdefs::gid_to_lcoord;

use crate::frame::ClientFrame;
use crate::hud_view::HudView;

impl HudView {
    /// `gmNavigationUI::Update` for this frame: the panel follows the player's cell, the
    /// coordinates (the radar's string) and the map notes.
    pub(crate) fn update_navigation(&mut self, frame: &ClientFrame) {
        let (date, time) = match &self.panels.navigation_time {
            Some((_, date, time)) => (Some(date.clone()), Some(time.clone())),
            None => (None, None),
        };
        let view = navigation_view(
            frame,
            &self.panels.navigation_tooltips,
            self.panels.radar_coordinates.clone(),
            date,
            time,
        );
        let Some((tree, images)) = &mut self.hud else {
            return;
        };
        let Some(nav) = &mut self.panels.navigation else {
            return;
        };
        // The world map replaces the painted views (`ac2_ui::world_map`); without it, retail's.
        match self.panels.world_map.as_mut() {
            Some(map) => {
                let map_view = ac2_ui::world_map::WorldMapView {
                    player: frame
                        .player
                        .as_ref()
                        .map(|p| land_position(p.cell, p.origin)),
                    coordinates: view.coordinates.clone(),
                    time: view.time.clone(),
                    date: view.date.clone(),
                    markers: view.notes.clone(),
                };
                if nav.is_shown(tree) {
                    map.frame(tree, nav, images, &map_view);
                }
            }
            None => nav.update(tree, &view, images),
        }
    }

    /// The quest guide's frame (`ac2_ui::guide_window`): its decoded pictures go into the HUD's
    /// images, and a closed guide lets its pictures go.
    pub(crate) fn update_guide(&mut self) {
        if let (Some((tree, images)), Some(guide)) = (&mut self.hud, &mut self.panels.guide) {
            guide.frame(tree, images);
        }
    }

    /// A press on one of the panel's buttons (`gmNavigationUI::ListenToMessage`); `shift` is
    /// the Shift key (the short pan step). Returns whether the panel took the button.
    pub(crate) fn navigation_press(&mut self, button: &str, shift: bool) -> bool {
        match (&mut self.hud, &mut self.panels.navigation) {
            (Some((tree, images)), Some(nav)) => {
                nav.button(tree, images, ac2_ui::tokenize(button), shift)
            }
            _ => false,
        }
    }

    /// A press on the panel's map elements (`MapField`, `MapParentField`, `PlayerMarker`), with
    /// the element's data code and the pixel relative to the map field.
    pub(crate) fn navigation_map_press(
        &mut self,
        element: &str,
        data: i32,
        pixel: (i32, i32),
    ) -> bool {
        match (&mut self.hud, &mut self.panels.navigation) {
            (Some((tree, images)), Some(nav)) => {
                nav.map_press(tree, images, ac2_ui::tokenize(element), data, pixel)
            }
            _ => false,
        }
    }

    /// The lock to the character (`gmNavigationUI::LockPositionButton`'s state, set by the test
    /// handle: the layout has no lock button).
    pub(crate) fn navigation_lock(&mut self, locked: bool) {
        if let (Some((tree, images)), Some(nav)) = (&mut self.hud, &mut self.panels.navigation) {
            nav.set_lock(tree, images, locked);
        }
    }

    /// The world map's window (centre and zoom), when the panel has the world map.
    pub(crate) fn world_map_window(&self) -> Option<ac2_ui::world_map_tiles::MapWindow> {
        self.panels.world_map.as_ref().map(|map| map.window())
    }

    /// The panel and the tree it lives in.
    pub(crate) fn navigation_parts(
        &mut self,
    ) -> Option<(&mut ac2_ui::UiTree, &mut ac2_ui::NavigationUi)> {
        match (&mut self.hud, &mut self.panels.navigation) {
            (Some((tree, _)), Some(nav)) => Some((tree, nav)),
            _ => None,
        }
    }
}

/// A position in land cells east and north of the world's south-west corner (a landblock is 8
/// cells of 20 m; the origin is relative to the cell's landblock).
fn land_position(cell: u32, origin: [f32; 3]) -> (f64, f64) {
    let bx = f64::from((cell >> 24) & 0xFF);
    let by = f64::from((cell >> 16) & 0xFF);
    (
        bx * 8.0 + f64::from(origin[0]) / 20.0,
        by * 8.0 + f64::from(origin[1]) / 20.0,
    )
}

/// The panel's input from the frame. A marker's tooltip is its rendered text (`tooltips`, one
/// per marker, from the player's string tables).
fn navigation_view(
    frame: &ClientFrame,
    tooltips: &[String],
    coordinates: Option<String>,
    date: Option<String>,
    time: Option<String>,
) -> ac2_ui::NavigationView {
    let player = frame.player.as_ref().and_then(|p| gid_to_lcoord(p.cell));
    let notes = frame
        .markers
        .iter()
        .enumerate()
        .map(|(at, m)| ac2_ui::MapNote {
            kind: m.kind,
            id: m.id,
            land: gid_to_lcoord(m.cell),
            image: m.image,
            message: tooltips.get(at).cloned().unwrap_or_default(),
            // `RecvNotice_AddMarker` passes `macro == 0`: the notes show in the detail view.
            macro_map: false,
        })
        .collect();
    ac2_ui::NavigationView {
        player,
        coordinates: if player.is_some() { coordinates } else { None },
        date,
        time,
        notes,
    }
}
