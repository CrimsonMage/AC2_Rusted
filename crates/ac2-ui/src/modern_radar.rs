//! The modern style's radar (AGENTS.md: the HUD's two styles): `gmRadarUI`'s `RadarField` and
//! `HeadingField` laid out as the design's. Retail drew a stone ring (`0x4100336E`) 120 pixels
//! across around the blips, a heading ring with arrow marks (`0x41100000`) and a stone
//! coordinates box (`0x410024BD`) under it, the coordinates button at its left. Here the blips
//! sit on a dark teal disc 92 pixels across in a fine stone ring with the player's arrow at its
//! centre, the heading ring is the four letters around it (still the turning `HeadingField`), the
//! coordinates are a pill under the disc, the Tracker eye and the coordinates button stand in
//! round wells at the disc's lower left and right, and a chip at its upper right gives the
//! range. The blips are retail's (`gmRadarUI::Update` plots them at the field's `RadarCenter`
//! and `RadarRadius`, which this sets to the design's disc, so the scale is the disc's), and so
//! are the buttons' presses.
//!
//! The art is restyled in the gameplay layout ([`restyle`]) before the radar is made, as
//! `BaseField`'s state changes (the coordinates box opening and closing) make its children anew
//! from the layout; their places are set again each update ([`ModernRadar::update`]).

use std::cell::{Cell, RefCell};
use std::sync::Arc;

use crate::compose::{DRAW_OVER, NINE_SLICE};
use crate::element::{UiBox, UiTree};
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc};
use crate::modern_art as art;
use crate::modern_drawer_bar::{count_label, place_label};
use crate::modern_inventory::{keep, make_part, remap_art};
use crate::text::Justification;
use crate::token::{tokenize, MOUSE_INVISIBLE, SIZE_IMAGE_TO_OWNER, STATELESS};

/// `RadarField`'s unnamed children: the ring's image and its second copy, the coordinates' box
/// and the coordinates button's holder.
const RING: u32 = 0x90C9_E06D;
const RING_PIECE: u32 = 0xB4D8_B7B4;
const COORD_BOX: u32 = 0xCDFC_CE86;
const COORD_HOLDER: u32 = 0xCD54_CE86;
/// The retail art the design's stands for: the ring (gone), the heading ring, the coordinates'
/// box; and the small round button the Tracker eye is copied from.
const ART: [(u32, u32); 3] = [
    (0x4100_336E, art::RADAR_CLEAR),
    (0x4110_0000, art::HEADING_RING),
    (0x4100_24BD, art::COORD_PILL),
];
const BUTTON_ART: [(u32, u32); 3] = [
    (0x4100_097B, art::RADAR_WELL),
    (0x4100_097C, art::RADAR_WELL_LIT),
    (0x4100_097D, art::RADAR_WELL_LIT),
];
/// The pill's top, height and room either side of the text; the buttons' top.
const PILL: (i32, i32, i32) = (128, 18, 10);
const BUTTONS_Y: i32 = 92;
/// The radar's distance from the display's right and top edges.
const MARGIN: (i32, i32) = (12, 8);
/// The range chip's top and height.
const CHIP: (i32, i32) = (20, 12);

/// The gameplay layout with the radar's art the design's (`ART`), drawn over, and the
/// coordinates' box in nine pieces. Every element keeps its id and states.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    fn walk(d: &ElementDesc) -> ElementDesc {
        let mut d = d.clone();
        for state in &mut d.states {
            for media in &mut state.media {
                if let MediaDesc::Image {
                    file, draw_mode, ..
                } = media
                {
                    if let Some(&(_, to)) = ART.iter().find(|(from, _)| from == file) {
                        *file = to;
                        *draw_mode = DRAW_OVER;
                    }
                }
            }
            for child in &mut state.children {
                *child = Arc::new(walk(child));
            }
        }
        if d.id == COORD_BOX {
            if let Some(state) = d.states.iter_mut().find(|s| s.id == STATELESS) {
                state.attributes.push(AttributeDesc {
                    id: NINE_SLICE,
                    value: AttributeValue::Int(art::PILL_BORDER),
                });
            }
        }
        d
    }
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        *element = Arc::new(walk(element));
    }
    layout
}

/// The radar's parts beside retail's.
#[derive(Debug, Clone)]
pub struct ModernRadar {
    root: usize,
    /// The range chip, made under the root (which `BaseField`'s state changes leave), as are the
    /// disc and the coordinates button's well.
    chip: Option<usize>,
    chip_text: Option<usize>,
    eye: Cell<Option<usize>>,
    /// What the pill and the chip were last sized for: the coordinates and the range's text.
    shown: RefCell<(String, String)>,
}

fn place(tree: &mut UiTree, element: Option<usize>, (x, y, w, h): (i32, i32, i32, i32)) {
    let Some(element) = element else {
        return;
    };
    let at = UiBox::new(x, y, x + w - 1, y + h - 1);
    if tree.el(element).rect != at {
        keep(tree, element);
        tree.resize_to(element, w, h);
        tree.move_to(element, x, y);
    }
}

impl ModernRadar {
    /// The radar laid out as the design's, before `gmRadarUI` reads its geometry. `None` without
    /// the radar.
    pub fn adopt(tree: &mut UiTree, layouts: &[&Arc<LayoutDesc>]) -> Option<Self> {
        let root = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| tree.el(r).id() == tokenize("RadarField"))?;
        let (w, h) = art::RADAR_BOX;
        // Laid out by the display's top right (`MARGIN`), not a saved window, as the drawer under
        // it is.
        let at = tree.el(root).rect;
        if (at.width(), at.height()) != (w, h) {
            tree.resize_to(root, w, h);
        }
        tree.set_overlay_root(root);
        // The blips on the design's disc.
        let (cx, cy) = art::RADAR_CENTER;
        tree.set_int(root, tokenize("RadarCenter_X"), cx);
        tree.set_int(root, tokenize("RadarCenter_Y"), cy);
        tree.set_int(root, tokenize("RadarRadius"), art::RADAR_RADIUS);
        // The parts under the root, behind `BaseField` (a higher z draws further back).
        let template = tree.child_recursive(root, RING);
        let part = |tree: &mut UiTree, id: &str, image: u32, nine: Option<i32>, z: i32| {
            let part = make_part(tree, root, template, id, image, nine, z)?;
            keep(tree, part);
            Some(part)
        };
        let disc = part(tree, "ModernRadarDisc", art::RADAR_DISC, None, 50);
        let side = art::DISC_SIZE;
        place(tree, disc, (cx - side / 2, cy - side / 2, side, side));
        let well = part(tree, "ModernRadarCoordWell", art::RADAR_WELL, None, 50);
        let size = art::RADAR_WELL_SIZE;
        place(tree, well, (w - 2 - size, BUTTONS_Y, size, size));
        let chip = part(
            tree,
            "ModernRadarRange",
            art::RANGE_CHIP,
            Some(art::RANGE_CHIP_BORDER),
            0,
        );
        let chip_text = chip.and_then(|chip| {
            let label = count_label(
                tree,
                layouts,
                chip,
                "ModernRadarRangeText",
                Justification::Center,
            )?;
            keep(tree, label);
            for (channel, value) in [("red", 0x9F), ("green", 0xD2), ("blue", 0xC6)] {
                let _ = tree.set_float(
                    label,
                    tokenize(&format!("font_color1_{channel}")),
                    value as f32 / 255.0,
                );
            }
            Some(label)
        });
        for part in [disc, well].into_iter().flatten() {
            tree.set_bool(part, MOUSE_INVISIBLE, true);
        }
        // The heading ring the radar's width square, its centre the disc's.
        if let Some(heading) = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| tree.el(r).id() == tokenize("HeadingField"))
        {
            let at = tree.el(heading).rect;
            if (at.width(), at.height()) != (w, w) {
                tree.resize_to(heading, w, w);
            }
        }
        let radar = Self {
            root,
            chip,
            chip_text,
            eye: Cell::new(None),
            shown: RefCell::default(),
        };
        radar.ensure(tree);
        Some(radar)
    }

    /// The Tracker eye (made after the radar) in its well at the lower left, its icon centred.
    pub fn place_eye(&self, tree: &mut UiTree, eye: Option<usize>) {
        let Some(eye) = eye else {
            return;
        };
        remap_art(tree, eye, &BUTTON_ART, false);
        self.eye.set(Some(eye));
        self.ensure(tree);
    }

    /// `BaseField`'s children where the design has them (they are made anew when its state
    /// changes).
    fn ensure(&self, tree: &mut UiTree) {
        let find = |tree: &UiTree, id: u32| tree.child_recursive(self.root, id);
        let (w, h) = art::RADAR_BOX;
        let size = art::RADAR_WELL_SIZE;
        // At the display's top right.
        let at = (tree.display.0 - MARGIN.0 - w, MARGIN.1);
        let now = tree.screen_box(self.root);
        if (now.x0, now.y0) != at {
            tree.move_to(self.root, at.0, at.1);
        }
        place(tree, find(tree, tokenize("BaseField")), (0, 0, w, h));
        place(tree, find(tree, RING), (0, 0, w, w));
        if let Some(piece) = find(tree, RING_PIECE) {
            if tree.el(piece).visible {
                tree.set_visible(piece, false);
            }
        }
        let holder = find(tree, COORD_HOLDER);
        place(tree, holder, (w - 2 - size, BUTTONS_Y, size, size));
        if let Some(button) = holder.and_then(|h| tree.child(h, tokenize("CoordBoxButton"))) {
            place(tree, Some(button), (4, 4, 12, 12));
            tree.set_bool(button, SIZE_IMAGE_TO_OWNER, true);
        }
        if let Some(eye) = self.eye.get().filter(|&e| tree.get(e).is_some()) {
            place(tree, Some(eye), (2, BUTTONS_Y, size, size));
            for icon in tree.children(eye).to_vec() {
                place(tree, Some(icon), (4, 4, 12, 12));
                tree.set_bool(icon, SIZE_IMAGE_TO_OWNER, true);
            }
        }
    }

    /// The parts kept where the design has them, the pill sized to the coordinates and centred
    /// under the disc, and the chip to the range (`range` metres), when they change.
    pub fn update(&self, tree: &mut UiTree, range: f32) {
        self.ensure(tree);
        let find = |tree: &UiTree, id: u32| tree.child_recursive(self.root, id);
        let text = find(tree, tokenize("CoordinatesText"));
        let coords = text
            .and_then(|t| tree.get(t).and_then(|e| e.text()))
            .unwrap_or_default();
        let range = format!("{} m", range.round() as i32);
        let (w, _) = art::RADAR_BOX;
        if let (Some(pill), Some(text)) = (find(tree, COORD_BOX), text) {
            let width = tree.text_line_width(text, &coords).unwrap_or(80) + 2 * PILL.2;
            place(tree, Some(pill), ((w - width) / 2, PILL.0, width, PILL.1));
            place(tree, Some(text), (0, 0, width, PILL.1));
            if let Some(t) = tree.el_mut(text).text.as_mut() {
                t.justification = Justification::Center;
            }
        }
        if *self.shown.borrow() == (coords.clone(), range.clone()) {
            return;
        }
        if let (Some(chip), Some(label)) = (self.chip, self.chip_text) {
            tree.set_text_with_font(label, &range, 0, 1);
            let width = tree.text_line_width(label, &range).unwrap_or(30) + 8;
            place(tree, Some(chip), (w + 2 - width, CHIP.0, width, CHIP.1));
            place_label(tree, Some(label), UiBox::new(0, 0, width - 1, CHIP.1 - 1));
        }
        *self.shown.borrow_mut() = (coords, range);
    }
}
