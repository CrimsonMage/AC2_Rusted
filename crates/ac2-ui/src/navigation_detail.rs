//! `gmNavigationUI`'s detail view: the 128-cell window around a cell (`InitDetailMapFromCellCoords`,
//! `ZoomToCharPos`, `InitDetailMapFromLastClick`), its four tiles (the rows of the window's corners
//! in the region's detail map, placed by the detail branch of `SizeMap`), the tiles' map notes
//! (`AddMapNotes`), the lock to the character and the scroll buttons' pan (`ListenToMessage`).
//! Admin 7148 addresses; the Legions 7160 functions are the same instructions.
//!
//! The window is `MapX`, `MapY` (the cell minus 64, clamped to the land) and `MapW` = `MapH` = 128.
//! Each corner is looked up in the block map as `cell >> 3` (`InqRowFromXY`); a corner without a
//! row leaves its tile blank. The pixel conversions are the detail frame's (scale 250, see
//! [`crate::navigation_map`]).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_world::{DetailMap, DetailMapRow};

use crate::compose::UiPixels;
use crate::element::UiTree;
use crate::navigation::{MapNote, NavigationUi, TileKey, STATIC_MARKER_TYPE};
use crate::navigation_map::{MapFrame, LAND_CELLS};
use crate::token::{tokenize, STATELESS};

/// `gmNavigationUI`'s `MapW` and `MapH` in the detail view: 0x80 cells.
pub const DETAIL_WINDOW: i32 = 0x80;
/// `LandDefs::lblock_shift`: a land block is `1 << 3` cells.
const BLOCK_SHIFT: u32 = 3;
/// The pan step in cells (`ListenToMessage`: 9, or 3 with Shift held).
pub const PAN_STEP: i32 = 9;
pub const PAN_STEP_SHIFT: i32 = 3;
/// `SizeMap`'s per-tile adjustments (`rgAdjustments`, admin 0xa53510: x then y for each tile).
const TILE_ADJUST: [(i32, i32); 4] = [(1, 0), (0, 0), (1, 1), (0, 1)];
/// `AddImageToState(tile, _mapDID, 0, stateless)`: the tile's draw mode.
const TILE_DRAW_MODE: u32 = 0;
/// `AddMapNotes`: a note's `MarkerInfo` is of `Static_MarkerType`.
const NOTE_KIND: u32 = STATIC_MARKER_TYPE;
/// `ListenToMessage`'s element data: the map field's press (`0x40000004`) and the map parent's
/// press that switches to the macro view (`0x40000005`).
pub const MAP_FIELD_PRESS: i32 = 0x4000_0004;
pub const MAP_PARENT_SWITCH: i32 = 0x4000_0005;
/// `ZLevel` of a tile with a row (2), and of a blank tile (`i + 10`).
const TILE_Z: i32 = 2;
const BLANK_Z: i32 = 10;

const BLANK: u32 = tokenize("blank");
const NAVIGATE_UP: u32 = tokenize("NavigateUpButton");
const NAVIGATE_LEFT: u32 = tokenize("NavigateLeftButton");
const NAVIGATE_RIGHT: u32 = tokenize("NavigateRightButton");
const NAVIGATE_DOWN: u32 = tokenize("NavigateDownButton");
/// The lock's tooltip strings (`UpdateLockPositionButtonToolTip`), by lock state and map.
const LOCK_TOOLTIP_ON: u32 = tokenize("ID_Navigation_TT_LockButtonOn");
const LOCK_TOOLTIP_OFF: u32 = tokenize("ID_Navigation_TT_LockButtonOff");
const LOCK_TOOLTIP_ERROR: u32 = tokenize("ID_Navigation_TT_LockButtonError");

/// `LandDefs::land_width` and the window's clamp (`InitDetailMapFromCellCoords`): the window's
/// origin is the cell minus half the window, kept on the land on each axis.
pub fn window_origin(cell: (i32, i32)) -> (i32, i32) {
    let clamp = |center: i32| {
        let mut origin = center - DETAIL_WINDOW / 2;
        if origin < 0 {
            origin = 0;
        }
        if LAND_CELLS < origin + DETAIL_WINDOW {
            origin = LAND_CELLS - DETAIL_WINDOW;
        }
        origin
    };
    (clamp(cell.0), clamp(cell.1))
}

/// Windows' `MulDiv`: `a * b / c` rounded to the nearest, halves away from zero; `-1` for a zero
/// denominator (the API's failure value).
pub fn mul_div(a: i32, b: i32, c: i32) -> i32 {
    if c == 0 {
        return -1;
    }
    let product = i64::from(a) * i64::from(b);
    let divisor = i64::from(c);
    let negative = (product < 0) != (divisor < 0);
    let magnitude = (product.abs() + divisor.abs() / 2) / divisor.abs();
    (if negative { -magnitude } else { magnitude }) as i32
}

/// `SizeMap`'s detail branch for one tile: the tile's pixel position in its parent (`px - ax * W`,
/// `ay * H + py`), with `px` and `py` the tile's cell edges mapped by `MulDiv` over the parent's
/// size. A tile with no width or no height sits at the origin.
pub fn tile_position(
    tile: usize,
    row: &DetailMapRow,
    window: (i32, i32),
    parent: (i32, i32),
) -> (i32, i32) {
    if row.width == 0 || row.height == 0 {
        return (0, 0);
    }
    let (ax, ay) = TILE_ADJUST[tile];
    let (w, h) = (parent.0, parent.1);
    let wc = (row.width << BLOCK_SHIFT) as i32;
    let hc = (row.height << BLOCK_SHIFT) as i32;
    let ex = ((row.x0 << BLOCK_SHIFT) as i32) + ax * wc;
    let ey = ((row.y0 << BLOCK_SHIFT) as i32) + ay * hc;
    let px = mul_div(ex - window.0, w, wc);
    let py = mul_div(window.1 - ey, h, hc);
    (px - ax * w, ay * h + py)
}

impl NavigationUi {
    /// `gmNavigationUI`'s detail map from the region's `CDetailMapDesc` (loaded on the worker).
    /// The tiles show nothing until the view is zoomed to a cell.
    pub fn set_detail_map(&mut self, map: Arc<DetailMap>) {
        self.detail_map = Some(map);
        self.tile_key = None;
    }

    /// Whether the detail view is up (`m_fMacroMap == 0`).
    pub fn is_detail(&self) -> bool {
        !self.map.macro_map
    }

    /// The row the tile `tile` shows (`m_CurrentRow[tile]`), `None` for a blank tile.
    pub fn tile_row(&self, tile: usize) -> Option<&DetailMapRow> {
        self.tile_rows.get(tile)?.as_ref()
    }

    /// The tile's element.
    pub fn tile(&self, tile: usize) -> Option<usize> {
        *self.tiles.get(tile)?
    }

    /// `m_fLockToCharPos`.
    pub fn is_locked(&self) -> bool {
        self.lock
    }

    /// The string id of the lock button's tooltip (`UpdateLockPositionButtonToolTip`): on or off
    /// while the player stands on the land, the error text otherwise.
    pub fn lock_tooltip_id(&self) -> u32 {
        if self.player.is_none() {
            return LOCK_TOOLTIP_ERROR;
        }
        if self.lock {
            LOCK_TOOLTIP_ON
        } else {
            LOCK_TOOLTIP_OFF
        }
    }

    /// `gmNavigationUI::InitDetailMapFromCellCoords`: the detail view's window around `cell`, the
    /// rows of its four corners on their tiles, their map notes, and the layout's detail state.
    /// Returns whether any corner has a row (the view is placed only then).
    pub fn init_detail_from_cell(
        &mut self,
        tree: &mut UiTree,
        images: &HashMap<u32, UiPixels>,
        cell: (i32, i32),
    ) -> bool {
        let origin = window_origin(cell);
        self.map = MapFrame {
            origin,
            size: (DETAIL_WINDOW, DETAIL_WINDOW),
            macro_map: false,
        };
        self.setup_elements_for_map_mode(tree, false);
        let (x, y) = origin;
        let corners = [
            (x, y + DETAIL_WINDOW),
            (x + DETAIL_WINDOW, y + DETAIL_WINDOW),
            (x, y),
            (x + DETAIL_WINDOW, y),
        ];
        let mut any = false;
        for (tile, corner) in corners.into_iter().enumerate() {
            let row = self.detail_map.as_ref().and_then(|map| {
                map.row_from_block(corner.0 >> BLOCK_SHIFT, corner.1 >> BLOCK_SHIFT)
                    .cloned()
            });
            any |= row.is_some();
            self.set_tile(tree, tile, row);
        }
        if any {
            self.refresh(tree, images);
        }
        any
    }

    /// `ZoomToCharPos`: the detail view around the player's last land cell.
    pub fn zoom_to_char(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>) -> bool {
        match self.player_land {
            Some(cell) => self.init_detail_from_cell(tree, images, cell),
            None => false,
        }
    }

    /// `InitDetailMapFromLastClick`: the detail view around the cell under the last macro click.
    pub fn init_detail_from_last_click(
        &mut self,
        tree: &mut UiTree,
        images: &HashMap<u32, UiPixels>,
    ) -> bool {
        match self.map.pixel_to_landscape(self.click) {
            Some(cell) => self.init_detail_from_cell(tree, images, cell),
            None => false,
        }
    }

    /// `InitMacroMap` (the macro view again): the macro window, the lock off, the layout's macro
    /// state, and the tiles' notes hidden with the macro view.
    pub fn init_macro_map(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>) {
        self.map = self.macro_frame;
        self.lock = false;
        self.sized = None;
        self.setup_elements_for_map_mode(tree, true);
        self.refresh(tree, images);
    }

    /// The pending view switch (`ListenToMessage`'s global-loop case: `InitMacroMap` from the
    /// detail view, `InitDetailMapFromLastClick` from the macro view), run once per frame.
    pub(crate) fn run_switch(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>) {
        if self.map.macro_map {
            self.init_detail_from_last_click(tree, images);
        } else {
            self.init_macro_map(tree, images);
        }
        self.switch = false;
    }

    /// The lock to the character (`ListenToMessage`'s `LockPositionButton` case after the button's
    /// state changed): the lock is set, the tooltip and the scroll buttons follow, and locking
    /// zooms to the character. The Legions 7160 layout has no `LockPositionButton`, so the lock is
    /// set through this method; the state is what the button's toggle would give.
    pub fn set_lock(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>, locked: bool) {
        self.lock = locked;
        self.auto_show_scroll_buttons(tree);
        if locked {
            self.zoom_to_char(tree, images);
        }
    }

    /// The panel's pan buttons (`ListenToMessage`): in the detail view without the lock, the
    /// button moves the window's centre by [`PAN_STEP`] cells (or [`PAN_STEP_SHIFT`] with Shift)
    /// and re-centres the window there. The `NavigationButton` shows or hides the panel, and
    /// `NavigationCloseButton` hides it. Returns whether `element` is one of these buttons.
    pub fn button(
        &mut self,
        tree: &mut UiTree,
        images: &HashMap<u32, UiPixels>,
        element: u32,
        shift: bool,
    ) -> bool {
        if element == tokenize("NavigationButton") {
            let shown = self.is_shown(tree);
            self.show(tree, !shown);
            return true;
        }
        if element == tokenize("NavigationCloseButton") {
            self.show(tree, false);
            return true;
        }
        let pan = [NAVIGATE_UP, NAVIGATE_LEFT, NAVIGATE_RIGHT, NAVIGATE_DOWN].contains(&element);
        if !pan {
            return false;
        }
        if self.map.macro_map || self.lock {
            return true;
        }
        let step = if shift { PAN_STEP_SHIFT } else { PAN_STEP };
        let (x, y) = self.map.origin;
        let (cx, cy) = (x + DETAIL_WINDOW / 2, y + DETAIL_WINDOW / 2);
        let target = match element {
            NAVIGATE_RIGHT => (cx + step, cy),
            NAVIGATE_LEFT => (cx - step, cy),
            NAVIGATE_UP => (cx, cy + step),
            _ => (cx, cy - step),
        };
        self.init_detail_from_cell(tree, images, target);
        true
    }

    /// The map's element presses (`ListenToMessage`): the parent's switch to the macro view, the
    /// macro map's click (`pixel` relative to the map field), and the player's marker (which
    /// zooms to the character in the macro view). Returns whether the element was handled.
    pub fn map_press(
        &mut self,
        tree: &mut UiTree,
        images: &HashMap<u32, UiPixels>,
        element: u32,
        data: i32,
        pixel: (i32, i32),
    ) -> bool {
        let map_field = tokenize("MapField");
        let map_parent = tokenize("MapParentField");
        let player_marker = tokenize("PlayerMarker");
        if self.map.macro_map {
            if element == player_marker {
                self.zoom_to_char(tree, images);
                return true;
            }
            if element != map_field {
                return false;
            }
            if data != MAP_FIELD_PRESS {
                return true;
            }
            self.click = pixel;
        } else {
            // The marker's description (`NoteText`) needs the string tables; not shown here.
            if element == player_marker {
                return true;
            }
            if element != map_parent || data != MAP_PARENT_SWITCH {
                return false;
            }
        }
        self.switch = true;
        true
    }

    /// The tile `tile` gets `row`: its image and state (`ListenToMessage`'s `InitDetailMap…`
    /// loop: a row shows the image on the stateless state, a blank tile is hidden).
    fn set_tile(&mut self, tree: &mut UiTree, tile: usize, row: Option<DetailMapRow>) {
        if let Some(element) = self.tiles[tile] {
            match &row {
                Some(row) => {
                    tree.set_state(element, STATELESS);
                    tree.set_z_level(element, TILE_Z);
                    tree.add_image_to_state(
                        element,
                        row.map_did,
                        TILE_DRAW_MODE,
                        crate::token::STATELESS,
                    );
                }
                None => {
                    tree.set_state(element, BLANK);
                    tree.set_z_level(element, tile as i32 + BLANK_Z);
                    tree.clear_image(element);
                }
            }
        }
        self.tile_rows[tile] = row;
    }

    /// `SizeMap`'s detail branch: each tile with a row takes its image's size and its position
    /// (`tile_position`). Repeated only when the window, the parent or an image changes.
    #[allow(clippy::needless_range_loop)] // `tile` indexes four parallel arrays and the position
    pub(crate) fn size_tiles(&mut self, tree: &mut UiTree, images: &HashMap<u32, UiPixels>) {
        let Some(parent) = self.map_parent else {
            return;
        };
        let Some(parent_box) = tree.get(parent).map(|e| e.rect) else {
            return;
        };
        let parent_size = (parent_box.width(), parent_box.height());
        let sizes = std::array::from_fn(|tile| {
            let file = self.tiles[tile]
                .and_then(|e| tree.get(e))
                .and_then(|e| e.image())
                .map(|i| i.file)?;
            images.get(&file).map(|p| (p.width, p.height))
        });
        let key = TileKey {
            window: self.map.origin,
            parent: parent_size,
            images: sizes,
        };
        if self.tile_key.as_ref() == Some(&key) {
            return;
        }
        self.tile_key = Some(key);
        for tile in 0..4 {
            let (Some(element), Some(row), Some((width, height))) =
                (self.tiles[tile], self.tile_rows[tile].as_ref(), sizes[tile])
            else {
                continue;
            };
            tree.resize_to(element, width as i32, height as i32);
            let (x, y) = tile_position(tile, row, self.map.origin, parent_size);
            tree.move_to(element, x, y);
        }
    }

    /// The map notes of the tiles' rows (`AddMapNotes`): each `CMapNoteRow` of a note desc a row
    /// lists, at its land cell, with its icon. They show in the detail view only (`macro == 0`).
    pub(crate) fn tile_notes(&self) -> Vec<MapNote> {
        let Some(map) = self.detail_map.as_ref() else {
            return Vec::new();
        };
        let mut notes = Vec::new();
        for row in self.tile_rows.iter().flatten() {
            for &desc_did in &row.map_notes {
                let Some(desc) = map.map_note(desc_did) else {
                    continue;
                };
                for (index, note) in desc.rows.iter().enumerate() {
                    notes.push(MapNote {
                        kind: NOTE_KIND,
                        id: (u64::from(desc_did) << 32) | index as u64,
                        land: Some((note.x, note.y)),
                        image: note.icon_did,
                        message: String::new(),
                        macro_map: false,
                    });
                }
            }
        }
        notes
    }
}
