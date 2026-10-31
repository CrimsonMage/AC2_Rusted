//! The modern style's selection marks (AGENTS.md: the HUD's two styles; the design's variant B):
//! `gm3DTextUI`'s reticle and name plates laid out as the design's. The reticle's corners show the
//! drawn gold brackets (`modern_art_target`) unturned, the selection's name and meters sit in a
//! dark glass card, the meters are slim (health 8 pixels, vigor 2 under it), and every other
//! object's name sits in a smaller glass pill. Placement, colours (the radar look's) and what
//! shows when are retail's (`reticle.rs`, `name_plates.rs`).
//!
//! The meters and the pills are restyled in the gameplay layout ([`restyle`]) before the reticle
//! and the plates are made; the card is the client's own root ([`make_card`]).

use std::sync::Arc;

use crate::compose::{DRAW_OVER, NINE_SLICE};
use crate::element::UiTree;
use crate::layout::{AttributeDesc, AttributeValue, ElementDesc, LayoutDesc, MediaDesc, Rect};
use crate::modern_art_target as art;
use crate::token::{tokenize, MOUSE_INVISIBLE};

/// `SelectionStuff`'s meters: health and vigor, and the PK losses behind them (which carry the
/// wells: a higher z draws further back).
const HEALTH: u32 = 0xE2E0_5FCF;
const VIGOR: u32 = 0xB708_1DEA;
const PK_DAMAGE: u32 = 0xC754_B20D;
const PK_VIGOR: u32 = 0x95D1_847D;
/// Vigor's top under health, with a pixel between.
const VIGOR_TOP: i32 = art::HEALTH_HEIGHT + 1;
/// The card's room around the name and the meters: either side, above and below.
pub const CARD_PAD: (i32, i32, i32) = (8, 2, 5);
/// A name pill's room either side of the name and above and below it.
const PILL_MARGINS: (i32, i32) = (6, 1);

/// A layout box from `(x0, y0)` to `(x1, y1)`, inclusive.
fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> Rect {
    Rect {
        x0: x0 as i16,
        y0: y0 as i16,
        x1: x1 as i16,
        y1: y1 as i16,
    }
}

fn set_attribute(state: &mut crate::layout::StateDesc, id: u32, value: AttributeValue) {
    state.attributes.retain(|a| a.id != id);
    state.attributes.push(AttributeDesc { id, value });
}

/// A meter laid out `height` pixels tall from `top`, its well `well` (the PK loss behind carries
/// it) and its fill `fill`.
fn meter(d: &ElementDesc, top: i32, height: i32, well: Option<u32>, fill: u32) -> ElementDesc {
    let mut d = d.clone();
    d.rect = rect(0, top, art::METER_WIDTH - 1, top + height - 1);
    for state in &mut d.states {
        for media in &mut state.media {
            if let (
                MediaDesc::Image {
                    file, draw_mode, ..
                },
                Some(well),
            ) = (&mut *media, well)
            {
                *file = well;
                *draw_mode = DRAW_OVER;
            }
        }
        for child in &mut state.children {
            let mut c = (**child).clone();
            c.rect = rect(0, 0, art::METER_WIDTH - 1, height - 1);
            for s in &mut c.states {
                for media in &mut s.media {
                    if let MediaDesc::Image {
                        file, draw_mode, ..
                    } = media
                    {
                        *file = fill;
                        *draw_mode = DRAW_OVER;
                    }
                }
            }
            *child = Arc::new(c);
        }
    }
    d
}

/// The gameplay layout with `SelectionStuff`'s meters slim and the design's width, and
/// `NameTextTemplate` in its glass pill. Every element keeps its id and states.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        if element.id == tokenize("SelectionStuff") {
            let mut d = (**element).clone();
            d.rect = rect(
                0,
                0,
                art::METER_WIDTH - 1,
                VIGOR_TOP + art::VIGOR_HEIGHT - 1,
            );
            for state in &mut d.states {
                for child in &mut state.children {
                    let restyled = match child.id {
                        HEALTH => meter(child, 0, art::HEALTH_HEIGHT, None, art::HEALTH_FILL),
                        PK_DAMAGE => meter(
                            child,
                            0,
                            art::HEALTH_HEIGHT,
                            Some(art::HEALTH_WELL),
                            art::PK_DAMAGE_FILL,
                        ),
                        VIGOR => meter(child, VIGOR_TOP, art::VIGOR_HEIGHT, None, art::VIGOR_FILL),
                        PK_VIGOR => meter(
                            child,
                            VIGOR_TOP,
                            art::VIGOR_HEIGHT,
                            Some(art::VIGOR_WELL),
                            art::PK_VIGOR_FILL,
                        ),
                        _ => continue,
                    };
                    *child = Arc::new(restyled);
                }
            }
            *element = Arc::new(d);
        } else if element.id == tokenize("NameTextTemplate") {
            let mut d = (**element).clone();
            let (side, up) = PILL_MARGINS;
            for state in &mut d.states {
                state.media.push(MediaDesc::Image {
                    file: art::NAME_PILL,
                    draw_mode: DRAW_OVER,
                    offset: (0, 0),
                });
                set_attribute(
                    state,
                    NINE_SLICE,
                    AttributeValue::Int(art::NAME_PILL_BORDER),
                );
                for (name, value) in [
                    ("left_margin", side),
                    ("right_margin", side),
                    ("top_margin", up),
                    ("bottom_margin", up),
                ] {
                    set_attribute(state, tokenize(name), AttributeValue::Int(value));
                }
            }
            *element = Arc::new(d);
        }
    }
    layout
}

/// The card behind the selection's name and meters: an overlay root of its own, copied from the
/// reticle's corner `template` (a plain field), drawn behind them; hidden.
pub(crate) fn make_card(tree: &mut UiTree, template: usize) -> Option<usize> {
    let layout = Arc::clone(&tree.el(template).layout);
    let mut desc = (*tree.el(template).reference).clone();
    desc.id = tokenize("ModernSelectionCard");
    desc.states.clear();
    let card = tree.create_element(layout, Arc::new(desc), true)?;
    // Behind the name and the meters (z 2500; a higher z goes further back).
    tree.set_z_level(card, tree.el(template).z + 1);
    tree.add_root(card);
    tree.set_overlay_root(card);
    tree.set_image(card, art::SELECTION_CARD, DRAW_OVER);
    tree.set_int(card, NINE_SLICE, art::SELECTION_CARD_BORDER);
    tree.set_bool(card, MOUSE_INVISIBLE, true);
    tree.show_object(card, false);
    Some(card)
}

/// The card around the name's box `name` and, when they show, the meters' box `meters` (each x,
/// y, width, height): its place and size.
pub(crate) fn place_card(
    tree: &mut UiTree,
    card: usize,
    name: (i32, i32, i32, i32),
    meters: Option<(i32, i32, i32, i32)>,
) -> [i32; 4] {
    let (side, up, down) = CARD_PAD;
    let (mut x0, mut y0, mut x1, mut y1) = (name.0, name.1, name.0 + name.2, name.1 + name.3);
    if let Some((x, y, w, h)) = meters {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x + w);
        y1 = y1.max(y + h);
    }
    let (x, y) = (x0 - side, y0 - up);
    let (w, h) = (x1 - x0 + 2 * side, y1 - y0 + up + down);
    let at = tree.el(card).rect;
    if (at.width(), at.height()) != (w, h) {
        tree.resize_to(card, w, h);
    }
    tree.set_object_position(card, x, y);
    tree.show_object(card, true);
    [x, y, x + w - 1, y + h - 1]
}
