//! The modern style's window titles (AGENTS.md: the HUD's two styles): small caps in gold with
//! wide letter spacing, fitted to the title's box. `modern_windows` marks a title element with the
//! client's own `modern_title` attribute (retail has none); every path that sets a text element's
//! text then restyles it here, so titles a framework sets at run time from a string table and
//! those the layout's `entry` names look alike. Retail drew the title in one of its decorative
//! lowercase faces ("sKiLLs") at the layout's size.
//!
//! The DATs carry a small-caps family (`0x4200000A`, `0x42000009`, `0x4200000B`, `0x4200001A`;
//! 20, 18, 16 and 13 pixels high), whose lowercase glyphs are small capitals. A word is set with
//! its first letter capital and the rest lower case, so each reads as a large initial over small
//! capitals; the largest size whose line (the glyphs and the letter spacing, 0.14 em) fits the
//! box is used, and a title that does not fit even in the smallest ends in an ellipsis.

use std::sync::Arc;

use ac2_assets::Ac2Font;

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::token::tokenize;

/// The attribute that marks a title element (a client attribute, `Bool`).
pub(crate) const MODERN_TITLE: u32 = tokenize("modern_title");

/// The small-caps family, largest first.
pub const TITLE_FONTS: [u32; 4] = [0x4200_000A, 0x4200_0009, 0x4200_000B, 0x4200_001A];

/// The letter spacing, in ems (the design's `0.14em`; the font's height stands for its em).
const TRACKING_EM: f64 = 0.14;

/// The extra advance after each glyph of `font`.
pub(crate) fn tracking(font: &Ac2Font) -> i32 {
    (TRACKING_EM * f64::from(font.max_char_height)).round() as i32
}

/// `text` with each word's first letter capital and the rest lower case (words break at spaces
/// and hyphens).
pub(crate) fn small_caps(text: &[u16]) -> Vec<u16> {
    let mut start = true;
    text.iter()
        .map(|&unit| {
            let mapped = char::from_u32(u32::from(unit)).map(|c| {
                let mut converted: Vec<char> = if start {
                    c.to_uppercase().collect()
                } else {
                    c.to_lowercase().collect()
                };
                // A letter that changes into several keeps its own form.
                if converted.len() == 1 {
                    converted.remove(0)
                } else {
                    c
                }
            });
            start = unit == 0x20 || unit == u16::from(b'-');
            mapped.map_or(unit, |c| u16::try_from(u32::from(c)).unwrap_or(unit))
        })
        .collect()
}

/// The width of `text` in `font` with `track` after each glyph but the last.
fn line_width(font: &Ac2Font, text: &[u16], track: i32) -> i32 {
    let glyphs = font.string_width(text) as i32;
    glyphs + track * (text.len() as i32 - 1).max(0)
}

/// How a title is set: the index in `fonts`, the text (cut and ended in an ellipsis when it
/// does not fit) and the letter spacing. `fonts` run largest first; `width` is the room (zero or
/// less when the box has no size yet: the largest font, whole). `None` when no font has every
/// character.
pub(crate) fn fit_title(
    fonts: &[Arc<Ac2Font>],
    text: &[u16],
    width: i32,
) -> Option<(usize, Vec<u16>, i32)> {
    let text = small_caps(text);
    let usable: Vec<usize> = (0..fonts.len())
        .filter(|&i| text.iter().all(|&c| fonts[i].contains(c)))
        .collect();
    let first = *usable.first()?;
    if width <= 0 {
        return Some((first, text, tracking(&fonts[first])));
    }
    for &i in &usable {
        let track = tracking(&fonts[i]);
        if line_width(&fonts[i], &text, track) <= width {
            return Some((i, text, track));
        }
    }
    // None fits: the smallest, cut to leave room for the ellipsis.
    let last = *usable.last()?;
    let font = &fonts[last];
    let track = tracking(font);
    let dots: Vec<u16> = if font.contains(0x2026) {
        vec![0x2026]
    } else {
        vec![u16::from(b'.'); 3]
    };
    let mut kept = text;
    while !kept.is_empty() {
        let mut cut = kept.clone();
        while cut.last() == Some(&0x20) {
            cut.pop();
        }
        cut.extend(&dots);
        if line_width(font, &cut, track) <= width {
            return Some((last, cut, track));
        }
        kept.pop();
    }
    Some((last, dots, track))
}

impl UiTree {
    /// Restyles a marked title element's text (`MODERN_TITLE`): small capitals in the largest
    /// size of the family that fits the element's box, spaced out. Anything else is left alone.
    pub(crate) fn style_title(&mut self, index: usize) {
        let el = self.el(index);
        if !matches!(
            el.attributes.get(&MODERN_TITLE),
            Some(AttributeValue::Bool(true))
        ) {
            return;
        }
        let Some(state) = el.text.as_ref() else {
            return;
        };
        let units = state.text();
        if units.is_empty() {
            return;
        }
        // The gold the window's title colour names (a title is set in colour 0).
        let channel =
            |name: &str| self.float_attribute(index, tokenize(&format!("font_color_{name}")));
        let color = match (channel("red"), channel("green"), channel("blue")) {
            (Some(r), Some(g), Some(b)) => Some([r, g, b, 1.0]),
            _ => None,
        };
        let room = el.rect.width() - state.margins[2] - state.margins[3];
        let Some(data) = self.text_data.clone() else {
            return;
        };
        let fonts: Vec<Arc<Ac2Font>> = TITLE_FONTS.iter().filter_map(|&d| data.font(d)).collect();
        let Some((chosen, text, track)) = fit_title(&fonts, &units, room) else {
            return;
        };
        let font = fonts[chosen].clone();
        if let Some(state) = self.el_mut(index).text.as_mut() {
            state.clear();
            state.font = Some(font);
            if let Some(color) = color {
                state.color = color;
            }
            state.add_text(&text);
            state.tracking = track;
            let last = state.glyphs.len().saturating_sub(1);
            for glyph in state.glyphs.iter_mut().take(last) {
                glyph.width += track;
            }
            self.changed = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_assets::Ac2FontChar;

    /// A font of `height` whose every glyph in `chars` is `width` wide.
    fn font(height: u32, width: u32, chars: &str) -> Arc<Ac2Font> {
        let list: Vec<Ac2FontChar> = chars
            .chars()
            .enumerate()
            .map(|(i, c)| Ac2FontChar {
                width,
                unicode: c as u16,
                offset: i as u32 * width,
            })
            .collect();
        let total = width * list.len() as u32;
        Arc::new(Ac2Font::new(
            [height, width, width, total],
            true,
            list,
            vec![0; (total * height) as usize],
        ))
    }

    fn u(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    fn s(v: &[u16]) -> String {
        String::from_utf16_lossy(v)
    }

    const CHARS: &str = "Skills Quest-ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.";

    #[test]
    fn words_get_a_capital_and_small_caps() {
        assert_eq!(s(&small_caps(&u("skills"))), "Skills");
        assert_eq!(s(&small_caps(&u("QUEST LOG"))), "Quest Log");
        assert_eq!(s(&small_caps(&u("Inventory-pack"))), "Inventory-Pack");
    }

    #[test]
    fn tracking_is_fourteen_hundredths_of_the_size() {
        assert_eq!(tracking(&font(20, 8, CHARS)), 3);
        assert_eq!(tracking(&font(13, 8, CHARS)), 2);
    }

    #[test]
    fn the_largest_size_that_fits_is_used() {
        // 6 glyphs: 8 wide + 3 spacing in the big one, 6 + 2 in the small one.
        let fonts = [font(20, 8, CHARS), font(13, 6, CHARS)];
        let (i, text, track) = fit_title(&fonts, &u("skills"), 100).unwrap();
        assert_eq!((i, s(&text), track), (0, "Skills".to_string(), 3));
        // Big: 6 * 8 + 5 * 3 = 63; small: 6 * 6 + 5 * 2 = 46.
        let (i, text, track) = fit_title(&fonts, &u("skills"), 50).unwrap();
        assert_eq!((i, s(&text), track), (1, "Skills".to_string(), 2));
        // No room known yet: the largest.
        assert_eq!(fit_title(&fonts, &u("skills"), 0).unwrap().0, 0);
    }

    #[test]
    fn a_title_too_long_for_the_smallest_ends_in_an_ellipsis() {
        let fonts = [font(20, 8, CHARS), font(13, 6, CHARS)];
        let (i, text, track) = fit_title(&fonts, &u("skills quest"), 40).unwrap();
        assert_eq!(i, 1);
        assert!(s(&text).ends_with("..."), "{}", s(&text));
        assert!(line_width(&fonts[1], &text, track) <= 40);
        assert!(s(&text).starts_with("Sk"));
    }

    #[test]
    fn a_font_without_the_characters_is_skipped() {
        let fonts = [font(20, 8, "Skil"), font(13, 6, CHARS)];
        assert_eq!(fit_title(&fonts, &u("skills"), 0).unwrap().0, 1);
        assert!(fit_title(&fonts[..1], &u("skills"), 0).is_none());
    }
}
