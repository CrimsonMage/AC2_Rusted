//! `gmNavigationUI`: the navigation panel (`NavigationField` in the toolbar layout) in its two
//! views.
//!
//! The macro view shows the region's game map (`MapField`, in `MapParent`) fitted into its parent
//! (`SizeMap`), the player's marker over the cell the player stands on (`UpdatePlayerPos`), the
//! coordinates in `CoordinatesText` (`Update`), and the map notes (`AddMarker` / `PlaceMarker`):
//! one element per note, from the toolbar layout's `MarkerTemplate`, shown when its cell lies on
//! the map. A note with `macro_map == false` is hidden in the macro view (`PlaceMarker`:
//! `m_fMacroMap && !info->macro`).
//!
//! The detail view (`navigation_detail.rs`) shows four tiles of the region's detail map around a
//! 128-cell window: the server's map notes (`macro == 0`) and the notes the tiles list show there.
//! The coordinate maps are in [`crate::navigation_map`].

use std::collections::HashMap;
use std::sync::Arc;

use ac2_world::{DetailMap, DetailMapRow};

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::layout::{ElementDesc, LayoutDesc};
use crate::navigation_map::{center_offset, macro_extent, size_macro_map, MapFrame};
use crate::token::tokenize;

/// The navigation field's `MacroXOffset`, `MacroYOffset` and `MacroScale` attributes.
const MACRO_X: u32 = tokenize("MacroXOffset");
const MACRO_Y: u32 = tokenize("MacroYOffset");
const MACRO_SCALE: u32 = tokenize("MacroScale");
/// The `MacroMap` and `DetailMap` states of `ParentField` (`SetupElementsForMapMode`).
const MACRO_MAP_STATE: u32 = tokenize("MacroMap");
const DETAIL_MAP_STATE: u32 = tokenize("DetailMap");
/// `SizeMap`'s image draw mode (`AddImageToState(..., 2, Token_stateless)`).
pub(crate) const MAP_DRAW_MODE: u32 = 2;

/// One map note as the panel places it: its marker kind (a `CMapNoteRow`), the marked object,
/// the land cell it sits on (`gid_to_lcoord`; `None` off the land), its marker image and its
/// text, and whether it belongs to the macro view.
#[derive(Debug, Clone, PartialEq)]
pub struct MapNote {
    pub kind: u32,
    pub id: u64,
    pub land: Option<(i32, i32)>,
    pub image: u32,
    pub message: String,
    pub macro_map: bool,
}

/// The frame's input to the panel: the player's land cell (`None` indoors or off the map), the
/// rendered coordinates (`Position::ToRealMapCoords_Info`), the rendered game date and time
/// (`GameTime::InqCurrentGameDateTimeString`, `None` without a game time) and the map notes in
/// arrival order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NavigationView {
    pub player: Option<(i32, i32)>,
    pub coordinates: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub notes: Vec<MapNote>,
}

#[derive(Debug, Clone)]
pub(crate) struct PlacedNote {
    pub(crate) element: usize,
    pub(crate) note: MapNote,
}

/// What the tiles were last sized for (`SizeMap`'s inputs): the window, the parent's size and
/// each tile's image size. A change sizes them again.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TileKey {
    pub(crate) window: (i32, i32),
    pub(crate) parent: (i32, i32),
    pub(crate) images: [Option<(u32, u32)>; 4],
}

/// `Static_MarkerType`: the marker type of the tiles' map notes (`AddMapNotes`).
pub(crate) const STATIC_MARKER_TYPE: u32 = 3;

/// `gmNavigationUI`'s state over its toolbar elements.
#[derive(Debug, Clone)]
pub struct NavigationUi {
    pub(crate) field: usize,
    /// `m_pParentField`: its state picks the macro or detail layout.
    pub(crate) parent: Option<usize>,
    /// `m_pMapParent`: the map field's parent, where the player's marker, the tiles and the
    /// notes sit.
    pub(crate) map_parent: Option<usize>,
    /// `m_pMapField`: the game map's element.
    pub(crate) map_field: Option<usize>,
    /// `m_pPlayerMarker`.
    pub(crate) player_marker: Option<usize>,
    /// The four scroll buttons: up, left, right, down (`AutoShowScrollButtons`).
    pub(crate) scroll: [Option<usize>; 4],
    /// The toolbar layout, for the `MarkerTemplate` and `DetailMapTemplate` elements.
    pub(crate) toolbar: Arc<LayoutDesc>,
    pub(crate) marker_template: Option<Arc<ElementDesc>>,
    /// `m_ppDetailMaps`: the four detail tiles, children of the map parent.
    pub(crate) tiles: [Option<usize>; 4],
    /// `m_CurrentRow`: the row each tile shows; `None` for a blank tile.
    pub(crate) tile_rows: [Option<DetailMapRow>; 4],
    /// The region's detail map (`CRegionDesc::GetCurrentDetailMapTable`), loaded on the worker.
    pub(crate) detail_map: Option<Arc<DetailMap>>,
    /// `MapX`, `MapY`, `MapW`, `MapH` and `m_fMacroMap`: the current view's window.
    pub(crate) map: MapFrame,
    /// The macro view's window (`InitMacroMap`), kept for returning to it.
    pub(crate) macro_frame: MapFrame,
    /// `m_fLockToCharPos`: the detail view follows the character.
    pub(crate) lock: bool,
    /// `m_PlayerBX`, `m_PlayerBY`: the player's last land cell on the land.
    pub(crate) player_land: Option<(i32, i32)>,
    /// The player's land cell this frame (`UpdatePlayerPos`).
    pub(crate) player: Option<(i32, i32)>,
    /// `m_ClickX`, `m_ClickY`: the last click on the macro map, relative to the map field.
    pub(crate) click: (i32, i32),
    /// `m_fSwitch`: a view change waits for the next global loop.
    pub(crate) switch: bool,
    /// The coordinates and notes of the last frame.
    pub(crate) server_notes: Vec<MapNote>,
    /// The size the tiles were last placed for.
    pub(crate) tile_key: Option<TileKey>,
    /// The map size and the parent size the last macro `SizeMap` used (its image's size too).
    pub(crate) sized: Option<((u32, u32), (i32, i32))>,
    /// The notes' elements, in arrival order.
    pub(crate) notes: Vec<PlacedNote>,
    /// `m_pOverviewOstethButton`, `m_pOverviewOmishanButton`, `m_pOverviewArramoraButton` and
    /// `m_pOverviewLinvakMassifButton`: shown in the macro view (`SetupElementsForMapMode`).
    pub(crate) overview_buttons: [Option<usize>; 4],
}

impl NavigationUi {
    /// `gmNavigationUI::gmNavigationUI` and `InitMacroMap`: the navigation field's children, the
    /// macro view's layout, the four detail tiles (`DetailMapTemplate`), and the map frame from the
    /// field's `MacroXOffset`, `MacroYOffset` and `MacroScale` (`1.0` when absent). The map image
    /// is set later by [`Self::set_game_map`].
    pub fn create(tree: &mut UiTree, toolbar: &Arc<LayoutDesc>) -> Option<Self> {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, tokenize("NavigationField")))?;
        let child = |tree: &UiTree, name: &str| tree.child_recursive(field, tokenize(name));
        let parent_field = child(tree, "ParentField");
        let map_field = child(tree, "MapField");
        let map_parent = map_field.and_then(|m| tree.get(m)?.parent_index());
        let int = |name: u32| match tree.get(field).and_then(|e| e.attribute(name)) {
            Some(crate::layout::AttributeValue::Int(v)) => *v,
            _ => 0,
        };
        let scale = match tree.get(field).and_then(|e| e.attribute(MACRO_SCALE)) {
            Some(crate::layout::AttributeValue::Float(v)) => *v,
            _ => 1.0,
        };
        let extent = macro_extent(scale);
        let scroll = [
            "NavigateUpButton",
            "NavigateLeftButton",
            "NavigateRightButton",
            "NavigateDownButton",
        ]
        .map(|name| child(tree, name));
        let overview_buttons = [
            "NavigationZoomToOverviewOstethButton",
            "NavigationZoomToOverviewOmishanButton",
            "NavigationZoomToOverviewArramoraButton",
            "NavigationZoomToOverviewLinvakMassifButton",
        ]
        .map(|name| child(tree, name));
        let macro_frame = MapFrame {
            origin: (int(MACRO_X), int(MACRO_Y)),
            size: (extent, extent),
            macro_map: true,
        };
        let mut panel = Self {
            field,
            parent: parent_field,
            map_parent,
            map_field,
            player_marker: child(tree, "PlayerMarker"),
            scroll,
            toolbar: Arc::clone(toolbar),
            marker_template: toolbar.element(tokenize("MarkerTemplate")).cloned(),
            tiles: [None; 4],
            tile_rows: [None, None, None, None],
            detail_map: None,
            map: macro_frame,
            macro_frame,
            lock: false,
            player_land: None,
            player: None,
            click: (0, 0),
            switch: false,
            server_notes: Vec::new(),
            tile_key: None,
            sized: None,
            notes: Vec::new(),
            overview_buttons,
        };
        panel.create_tiles(tree);
        panel.setup_elements_for_map_mode(tree, true);
        Some(panel)
    }

    /// `gmNavigationUI::gmNavigationUI`'s four tiles (`CreateElement(DetailMapTemplate)`), children
    /// of the map parent.
    fn create_tiles(&mut self, tree: &mut UiTree) {
        let (Some(template), Some(parent)) = (
            self.toolbar.element(tokenize("DetailMapTemplate")).cloned(),
            self.map_parent,
        ) else {
            return;
        };
        for slot in 0..4 {
            let Some(tile) =
                tree.create_element(Arc::clone(&self.toolbar), template.clone(), false)
            else {
                continue;
            };
            tree.set_parent(tile, Some(parent));
            self.tiles[slot] = Some(tile);
        }
    }

    /// The map image of the region (`gmNavigationUI::gmNavigationUI`'s
    /// `AddImageToState(MapField, CRegionDesc::GetGameMap(), 2)`).
    pub fn set_game_map(&mut self, tree: &mut UiTree, did: u32) {
        if let (Some(map), true) = (self.map_field, did != 0) {
            tree.set_image(map, did, MAP_DRAW_MODE);
        }
    }

    /// The map's window on the land (`MapX`, `MapY`, `MapW`, `MapH`, the macro flag).
    pub fn map_frame(&self) -> MapFrame {
        self.map
    }

    /// The panel's root element (`NavigationField`).
    pub fn field(&self) -> usize {
        self.field
    }

    /// The scroll buttons: up, left, right, down.
    pub fn scroll_buttons(&self) -> [Option<usize>; 4] {
        self.scroll
    }

    /// The player's marker element.
    pub fn player_marker(&self) -> Option<usize> {
        self.player_marker
    }

    /// The map field's element.
    pub fn map_field(&self) -> Option<usize> {
        self.map_field
    }

    /// The note elements, in arrival order.
    pub fn note_elements(&self) -> Vec<usize> {
        self.notes.iter().map(|n| n.element).collect()
    }

    /// A child of the panel by its name. Texts are looked up each time they are used: the layout's
    /// detail state leaves `DateText` and `TimeText` out of the tree (`ParentField`'s children).
    pub fn child(&self, tree: &UiTree, name: &str) -> Option<usize> {
        tree.child_recursive(self.field, tokenize(name))
    }

    /// `gmNavigationUI::SetupElementsForMapMode`'s parts: the map field and the coordinates
    /// shown in the macro view, the layout state of `ParentField`, the player's marker moved into
    /// the map parent, and the scroll buttons.
    pub(crate) fn setup_elements_for_map_mode(&mut self, tree: &mut UiTree, macro_map: bool) {
        self.flush_marker_type(tree, STATIC_MARKER_TYPE);
        if let Some(map) = self.map_field {
            tree.set_visible(map, macro_map);
            // `UIElement::ClearTooltip(m_pMapField)`.
            tree.set_tooltip(map, None);
        }
        if let Some(coords) = self.child(tree, "CoordinatesText") {
            tree.set_visible(coords, macro_map);
        }
        for button in self.overview_buttons.iter().flatten() {
            set_visible_if(tree, *button, macro_map);
        }
        // `UIElement::ClearTooltip` on each detail tile (`m_ppDetailMaps`).
        for tile in self.tiles.iter().flatten() {
            tree.set_tooltip(*tile, None);
        }
        if let Some(parent) = self.parent {
            tree.set_state(
                parent,
                if macro_map {
                    MACRO_MAP_STATE
                } else {
                    DETAIL_MAP_STATE
                },
            );
        }
        if let (Some(marker), Some(map_parent)) = (self.player_marker, self.map_parent) {
            tree.set_parent(marker, Some(map_parent));
        }
        self.auto_show_scroll_buttons(tree);
    }

    /// `gmNavigationUI::FlushMarkers`: the notes of one marker type (`MarkerInfo`'s type, the key
    /// of `m_markers`) are deleted and dropped; the other types stay as they were.
    pub(crate) fn flush_marker_type(&mut self, tree: &mut UiTree, kind: u32) {
        let (gone, kept): (Vec<PlacedNote>, Vec<PlacedNote>) = std::mem::take(&mut self.notes)
            .into_iter()
            .partition(|placed| placed.note.kind == kind);
        for placed in gone {
            tree.set_parent(placed.element, None);
            tree.delete(placed.element);
        }
        self.notes = kept;
    }

    /// `gmNavigationUI::AutoShowScrollButtons`: the four scroll buttons show while the panel is
    /// shown and the detail view is up without the lock to the character.
    pub(crate) fn auto_show_scroll_buttons(&self, tree: &mut UiTree) {
        let shown =
            tree.get(self.field).is_some_and(|e| e.visible) && !self.map.macro_map && !self.lock;
        for button in self.scroll.iter().flatten() {
            set_visible_if(tree, *button, shown);
        }
    }

    /// `gmNavigationUI::SizeMap` in the macro view once the map's image has a size: the field
    /// placed and sized in its parent (`MoveTo`, then `ResizeTo`). Repeated only when the image or
    /// the parent changes.
    pub(crate) fn size_map(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>) {
        let (Some(map), Some(parent)) = (self.map_field, self.map_parent) else {
            return;
        };
        let Some(file) = tree.get(map).and_then(|e| e.image()).map(|i| i.file) else {
            return;
        };
        let Some(pixels) = images.get(&file) else {
            return;
        };
        let Some(parent_box) = tree.get(parent).map(|e| e.rect) else {
            return;
        };
        let parent_size = (parent_box.width(), parent_box.height());
        let key = ((pixels.width, pixels.height), parent_size);
        if self.sized == Some(key) {
            return;
        }
        self.sized = Some(key);
        if let Some(place) = size_macro_map(parent_size, (pixels.width, pixels.height)) {
            tree.move_to(map, place.origin.0, place.origin.1);
            tree.resize_to(map, place.size.0, place.size.1);
        }
    }

    /// `gmNavigationUI::UpdatePlayerPos`: the marker's centre at the player's pixel on the map
    /// when the player stands on the land, else at the marker's own width and height (off the
    /// map, as retail parks it).
    pub(crate) fn update_player_pos(&self, tree: &mut UiTree, player: Option<(i32, i32)>) {
        let (Some(marker), Some(map)) = (self.player_marker, self.map_field) else {
            return;
        };
        let Some(marker_box) = tree.get(marker).map(|e| e.rect) else {
            return;
        };
        let size = (marker_box.width(), marker_box.height());
        let field = field_size(tree, map);
        let (x, y) = match player {
            Some(land) => {
                let (pixel, _) = self.map.landscape_to_pixel(field, land);
                (pixel[0] as i32, pixel[1] as i32)
            }
            None => (-size.0, -size.1),
        };
        let (cx, cy) = center_offset(size);
        move_if(tree, marker, x - cx, y - cy);
    }

    /// `gmNavigationUI::PlaceMarker`: the note's element is shown at its cell's pixel, centred,
    /// when it belongs to the view and its cell is on the map; else hidden.
    pub(crate) fn place_note(&self, tree: &mut UiTree, placed: &PlacedNote) {
        let element = placed.element;
        let shown = !self.map.macro_map || placed.note.macro_map;
        let placed_at = shown
            .then_some(placed.note.land)
            .flatten()
            .map(|land| {
                let field = self.map_field.map_or((0, 0), |m| field_size(tree, m));
                self.map.landscape_to_pixel(field, land)
            })
            .filter(|(_, inside)| *inside)
            .map(|(pixel, _)| pixel);
        let Some(pixel) = placed_at else {
            set_visible_if(tree, element, false);
            return;
        };
        let Some(marker_box) = tree.get(element).map(|e| e.rect) else {
            return;
        };
        let (cx, cy) = center_offset((marker_box.width(), marker_box.height()));
        set_visible_if(tree, element, true);
        move_if(tree, element, pixel[0] as i32 - cx, pixel[1] as i32 - cy);
    }

    /// Makes the notes of `notes` the panel's elements: the ones already shown are kept, new ones
    /// get a `MarkerTemplate` element under the map parent with their image and text, and the
    /// ones gone are deleted (`AddMarker`, `FlushMarkers`).
    pub(crate) fn reconcile_notes(&mut self, tree: &mut UiTree, notes: &[MapNote]) {
        let mut old = std::mem::take(&mut self.notes);
        let mut next = Vec::with_capacity(notes.len());
        for note in notes {
            if let Some(at) = old.iter().position(|p| p.note == *note) {
                next.push(old.swap_remove(at));
                continue;
            }
            let (Some(template), Some(parent)) = (self.marker_template.clone(), self.map_parent)
            else {
                continue;
            };
            let Some(element) = tree.create_element(Arc::clone(&self.toolbar), template, false)
            else {
                continue;
            };
            tree.set_parent(element, Some(parent));
            if note.image != 0 {
                tree.set_image(element, note.image, MAP_DRAW_MODE);
            }
            tree.set_tooltip(element, Some(note.message.clone()));
            next.push(PlacedNote {
                element,
                note: note.clone(),
            });
        }
        for gone in old {
            tree.set_parent(gone.element, None);
            tree.delete(gone.element);
        }
        self.notes = next;
    }

    /// `gmNavigationUI::Update`: while the panel is shown, the coordinates text, the player's
    /// marker, the map (in its view) and the notes follow the frame. A player who moves to another
    /// land cell re-centres the detail view when it is locked (`OnPlayerBlockChanged`).
    pub fn update(
        &mut self,
        tree: &mut UiTree,
        view: &NavigationView,
        images: &HashMap<u32, UiPixels>,
    ) {
        if !tree.get(self.field).is_some_and(|e| e.visible) {
            return;
        }
        if self.switch {
            self.run_switch(tree, images);
        }
        let coordinates = self.child(tree, "CoordinatesText");
        set_text_for(
            tree,
            coordinates,
            view.coordinates.as_deref(),
            view.player.is_some(),
        );
        if self.map.macro_map {
            let date = self.child(tree, "DateText");
            let time = self.child(tree, "TimeText");
            set_text_if_some(tree, date, view.date.as_deref());
            set_text_if_some(tree, time, view.time.as_deref());
        }
        self.player = view.player;
        self.server_notes = view.notes.clone();
        if let Some(land) = view.player {
            if self.player_land != Some(land) {
                self.player_land = Some(land);
                if self.lock && !self.map.macro_map {
                    self.init_detail_from_cell(tree, images, land);
                    return;
                }
            }
        }
        self.refresh(tree, images);
        self.read_marker_info(tree);
    }

    /// `gmNavigationUI::ReadMarkerInfo`: the hovered note's tooltip is the text of `NoteText`
    /// (retail sets it on the marker's message; here it follows the pointer each frame).
    fn read_marker_info(&self, tree: &mut UiTree) {
        let Some(owner) = tree.tooltip_owner() else {
            return;
        };
        if !self.note_elements().contains(&owner) {
            return;
        }
        let (Some(text), Some(note)) = (tree.tooltip_text(owner), self.child(tree, "NoteText"))
        else {
            return;
        };
        tree.set_text(note, &text);
    }

    /// `gmNavigationUI::Show`: the panel shown or hidden (`NavigationButton` and the close button).
    pub fn show(&self, tree: &mut UiTree, shown: bool) {
        set_visible_if(tree, self.field, shown);
    }

    /// Whether the panel is shown.
    pub fn is_shown(&self, tree: &UiTree) -> bool {
        tree.get(self.field).is_some_and(|e| e.visible)
    }

    /// The frame's placement: the view's map (`SizeMap`, macro or detail), the notes (the
    /// server's and the tiles'), the player's marker and the scroll buttons.
    pub(crate) fn refresh(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>) {
        if self.map.macro_map {
            self.size_map(tree, images);
        } else {
            self.size_tiles(tree, images);
        }
        let mut notes = self.server_notes.clone();
        // `AddMapNotes` runs only for the detail tiles; the macro view has none of them.
        if !self.map.macro_map {
            notes.extend(self.tile_notes());
        }
        self.reconcile_notes(tree, &notes);
        for placed in &self.notes {
            self.place_note(tree, placed);
        }
        self.update_player_pos(tree, self.player);
        self.auto_show_scroll_buttons(tree);
    }
}

/// `Update`'s coordinates text: the text while the player is on the land, cleared otherwise
/// (`UIElement_Text::ClearAllText`), and only set when it changed.
fn set_text_for(tree: &mut UiTree, element: Option<usize>, text: Option<&str>, on_land: bool) {
    let Some(element) = element else {
        return;
    };
    match text {
        Some(text) if on_land => {
            if tree.get(element).and_then(|e| e.text()).as_deref() != Some(text) {
                tree.set_text(element, text);
            }
        }
        _ => {
            if tree
                .get(element)
                .and_then(|e| e.text())
                .is_some_and(|t| !t.is_empty())
            {
                tree.clear_text(element);
            }
        }
    }
}

/// `UpdateMacroMap`'s date and time: set when the string changed (`StringInfo` compare), and left
/// alone without a game time (`InqCurrentGameDateTimeString` fails).
fn set_text_if_some(tree: &mut UiTree, element: Option<usize>, text: Option<&str>) {
    let (Some(element), Some(text)) = (element, text) else {
        return;
    };
    if tree.get(element).and_then(|e| e.text()).as_deref() != Some(text) {
        tree.set_text(element, text);
    }
}

/// `UIElement::SetVisible` when the visibility changes (a set marks the tree changed).
pub(crate) fn set_visible_if(tree: &mut UiTree, index: usize, visible: bool) {
    if tree.get(index).is_some_and(|e| e.visible != visible) {
        tree.set_visible(index, visible);
    }
}

/// `UIElement::MoveTo` when the element is elsewhere (a move marks the tree moved).
pub(crate) fn move_if(tree: &mut UiTree, index: usize, x: i32, y: i32) {
    if tree
        .get(index)
        .is_some_and(|e| (e.rect.x0, e.rect.y0) != (x, y))
    {
        tree.move_to(index, x, y);
    }
}

/// `MapField`'s width and height (`UIElement::GetWidth` / `GetHeight`), the field the pixel
/// conversion is checked against.
pub(crate) fn field_size(tree: &UiTree, map: usize) -> (i32, i32) {
    tree.get(map)
        .map_or((0, 0), |e| (e.rect.width(), e.rect.height()))
}
