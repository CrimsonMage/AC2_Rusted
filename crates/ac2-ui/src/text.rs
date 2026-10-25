//! `UIElement_Text`: the text a text element shows, its glyphs (`Glyph`, `GlyphList`) and lines
//! (`GlyphList::Recalculate`), and how `UIElement_Text::PostBlit` draws them into the root's
//! surface (`SurfaceWindow::DrawCharacter`).
//!
//! The font is the `font` DataID attribute, coloured by `font_color_red/green/blue` and outlined
//! by `font_outline` (`UIElement_Text::SetFontNum`); until one is set the element uses the
//! default font (`FONT_GROUP_ENUM` 0x4000000B, `Default_FontID` 0x40000001). The `entry`
//! StringID attribute shows that string of the layout's string table (`UIElement_Text::SetString`).
//! Margins come from `top/bottom/left/right_margin`, clipped to the element
//! (`DetermineMarginValues`); `center` and `right_justify` justify each line
//! (`CalcJustification`); `center_vertically` centres the lines (`KeepInBounds`); `one_line`
//! stops wrapping.
//!
//! Not modelled yet: editing, the cursor, selection, scrolling and scrollbars, the Korean and
//! Japanese fallback fonts (`DetermineFont`), string variables and server string downloads.

use std::sync::Arc;

use ac2_assets::Ac2Font;

use crate::compose::UiSurface;
use crate::element::UiBox;

/// The data text elements draw from, loaded by whoever builds the tree (`StringDownload`).
pub trait UiTextData: Send + Sync {
    /// `FontLocal::GetGlobalOrLocalFont`.
    fn font(&self, did: u32) -> Option<Arc<Ac2Font>>;
    /// `DBObj::GetByEnum(Default_FontID, FONT_GROUP_ENUM, DB_TYPE_FONT)`.
    fn default_font(&self) -> Option<Arc<Ac2Font>>;
    /// `StringInfo::GetString` of string `id` in string table `table`.
    fn string(&self, table: u32, id: u32) -> Option<Vec<u16>>;
}

impl std::fmt::Debug for dyn UiTextData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UiTextData")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Justification {
    Left,
    Center,
    Right,
}

/// `Glyph`.
#[derive(Debug, Clone)]
pub(crate) struct Glyph {
    pub ch: u16,
    pub width: i32,
    pub height: i32,
    pub font: Option<Arc<Ac2Font>>,
    pub color: [f32; 4],
    pub outline: bool,
}

impl Glyph {
    /// `Glyph::Glyph`: a newline is one pixel wide.
    fn new(ch: u16, color: [f32; 4], font: Option<Arc<Ac2Font>>, outline: bool) -> Self {
        let (width, height) = match &font {
            Some(font) => (
                if ch == 10 {
                    1
                } else {
                    font.char_desc(ch).map_or(0, |c| c.width as i32)
                },
                font.max_char_height as i32,
            ),
            None => (0, 0),
        };
        Self {
            ch,
            width,
            height,
            font,
            color,
            outline,
        }
    }
}

/// `GlyphLine`: the first glyph, width and height of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GlyphLine {
    pub index: usize,
    pub width: i32,
    pub height: i32,
}

/// A text element's state.
#[derive(Debug, Clone)]
pub(crate) struct TextState {
    pub font: Option<Arc<Ac2Font>>,
    pub color: [f32; 4],
    pub outline: bool,
    pub justification: Justification,
    pub one_line: bool,
    pub center_vertically: bool,
    /// Requested and effective margins: up, down, left, right.
    pub requested: [i32; 4],
    pub margins: [i32; 4],
    pub paper: (i32, i32),
    pub view: (i32, i32),
    pub glyphs: Vec<Glyph>,
    pub lines: Vec<GlyphLine>,
    pub max_characters: usize,
    pub dirty: bool,
}

impl TextState {
    /// `UIElement_Text::UIElement_Text`.
    pub(crate) fn new(font: Option<Arc<Ac2Font>>) -> Self {
        Self {
            font,
            color: [1.0; 4],
            outline: false,
            justification: Justification::Left,
            one_line: false,
            center_vertically: false,
            requested: [0; 4],
            margins: [0; 4],
            paper: (0, 0),
            view: (0, 0),
            glyphs: Vec::new(),
            lines: Vec::new(),
            max_characters: 0xFFFF,
            dirty: true,
        }
    }

    /// `UIElement_Text::DetermineMarginValues` for a `width` x `height` element.
    pub(crate) fn determine_margins(&mut self, width: i32, height: i32) {
        let [up, down, left, right] = self.requested;
        let up = up.min(height);
        let down = down.min(height - up);
        let left = left.min(width);
        let right = right.min(width - left);
        self.margins = [up, down, left, right];
    }

    /// `UIElement_Text::ClearAllText`.
    pub(crate) fn clear(&mut self) {
        self.glyphs.clear();
        self.dirty = true;
    }

    /// `UIElement_Text::AddText_Internal` / `GlyphList::AddText` at the end, in the current
    /// font, colour and outline.
    pub(crate) fn add_text(&mut self, text: &[u16]) {
        if self.font.is_none() {
            return;
        }
        for &ch in text {
            if self.glyphs.len() >= self.max_characters {
                break;
            }
            let glyph = Glyph::new(ch, self.color, self.font.clone(), self.outline);
            self.glyphs.push(glyph);
        }
        self.dirty = true;
    }

    /// The text, for re-adding it in a new font (`GetText` / `SetText`).
    pub(crate) fn text(&self) -> Vec<u16> {
        self.glyphs.iter().map(|g| g.ch).collect()
    }

    /// `UIElement_Text::RecalculateGlyphList` for a `width` x `height` element (neither
    /// `resizewidthtotext` nor `resizeheighttotext` is modelled).
    pub(crate) fn recalculate(&mut self, width: i32, height: i32) {
        if !self.dirty {
            return;
        }
        let [up, down, left, right] = self.margins;
        let margin_width = width - right - left;
        let (lines, text_width, text_height) =
            recalculate_lines(&self.glyphs, margin_width, self.one_line);
        self.lines = lines;
        self.paper = (
            (text_width + right + left).max(width),
            (text_height + down + up).max(height),
        );
        self.keep_in_bounds(width, height);
        self.dirty = false;
    }

    /// `UIElement_Text::KeepInBounds`.
    fn keep_in_bounds(&mut self, width: i32, height: i32) {
        let (mut x, mut y) = self.view;
        x = x.max(0);
        if self.paper.0 < x + width {
            x = (self.paper.0 - width).max(0);
        }
        if self.center_vertically {
            let total: i32 = self.lines.iter().map(|l| l.height).sum();
            y = total / 2 - height / 2;
        } else {
            y = y.max(0);
            if self.paper.1 < y + height {
                y = (self.paper.1 - height).max(0);
            }
        }
        self.view = (x, y);
    }
}

/// `LinebreakUtils::CanBreakLineAt`.
fn can_break(prev: Option<u16>, cur: u16) -> bool {
    let Some(prev) = prev else {
        return false;
    };
    if prev == 10 {
        return true;
    }
    let white = |c: u16| matches!(c, 9 | 13 | 32);
    let east_asian = |c: u16| {
        (0x1100..=0x11FF).contains(&c)
            || (0x3000..=0xD7AF).contains(&c)
            || (0xF900..=0xFAFF).contains(&c)
            || (0xFF00..=0xFFDC).contains(&c)
    };
    if !white(cur) && prev != 0x2D && !east_asian(cur) && !east_asian(prev) {
        return false;
    }
    let non_beginning = matches!(
        cur,
        0x21 | 0x29
            | 0x2C
            | 0x2E
            | 0x3F
            | 0x3001
            | 0x3002
            | 0x30FC
            | 0xFF01
            | 0xFF09
            | 0xFF1F
            | 0xFF70
            | 0xFF9E
            | 0xFF9F
    );
    let non_ending = matches!(prev, 0x28 | 0xFF08);
    !non_beginning && !non_ending
}

/// `GlyphList::Recalculate`: the lines, widest line and total height.
pub(crate) fn recalculate_lines(
    glyphs: &[Glyph],
    margin_width: i32,
    one_line: bool,
) -> (Vec<GlyphLine>, i32, i32) {
    let mut lines = Vec::new();
    let (mut width, mut height) = (0, 0);
    let (mut current_width, mut accumulated) = (0, 0);
    let (mut last_wrappable, mut wrappable_height) = (0, 0);
    let mut current_height = 0;
    let mut line_start = 0;
    let mut wrappable = false;
    let mut prev: Option<&Glyph> = None;
    for (index, glyph) in glyphs.iter().enumerate() {
        if !one_line && can_break(prev.map(|g| g.ch), glyph.ch) {
            last_wrappable = index;
            wrappable_height = current_height;
            accumulated = 0;
            wrappable = true;
        }
        current_width += glyph.width;
        accumulated += glyph.width;
        if (wrappable && margin_width < current_width) || prev.is_some_and(|g| g.ch == 10) {
            let line_width = current_width - accumulated;
            lines.push(GlyphLine {
                index: line_start,
                width: line_width,
                height: wrappable_height,
            });
            width = width.max(line_width);
            height += wrappable_height;
            current_height = 0;
            line_start = last_wrappable;
            current_width = accumulated;
            wrappable = false;
        }
        current_height = current_height.max(glyph.height);
        prev = Some(glyph);
    }
    lines.push(GlyphLine {
        index: line_start,
        width: current_width,
        height: current_height,
    });
    width = width.max(current_width);
    height += current_height;
    (lines, width, height)
}

/// `RGBAColor::GetColor32` as RGBA bytes: each channel `_ftol2(c * 255)`.
fn color_bytes(color: [f32; 4]) -> [u8; 4] {
    color.map(|c| (f64::from(c) * 255.0) as i64 as u8)
}

impl UiSurface {
    /// `SurfaceWindow::DrawCharacter` into `window` (surface coordinates, inclusive) at window
    /// position (`x`, `y`); returns the advance.
    #[allow(clippy::too_many_arguments)]
    fn draw_character(
        &mut self,
        window: UiBox,
        x: i32,
        y: i32,
        font: &Ac2Font,
        ch: u16,
        color: [f32; 4],
        blend: bool,
    ) -> i32 {
        let Some(desc) = font.char_desc(ch) else {
            return 0;
        };
        let (glyph_width, height) = (desc.width as i32, font.max_char_height as i32);
        let (window_width, window_height) = (window.width(), window.height());
        if x > window_width || y > window_height || x < -glyph_width || y < -height {
            return glyph_width;
        }
        let first = x.max(0);
        let last = (x + glyph_width - 2).min(window_width - 1);
        let pixel = color_bytes(color);
        for row in 0..height {
            let sy = y + row;
            if sy < 0 {
                continue;
            }
            if sy >= window_height {
                break;
            }
            for sx in first..=last {
                let coverage = font.coverage(desc.offset + (sx - x) as u32, row as u32);
                if coverage == 0 {
                    continue;
                }
                let (tx, ty) = (window.x0 + sx, window.y0 + sy);
                if tx < 0 || ty < 0 || tx >= self.width as i32 || ty >= self.height as i32 {
                    continue;
                }
                let at = (ty as usize * self.width as usize + tx as usize) * 4;
                let dest = &mut self.rgba[at..at + 4];
                if !font.antialiased || !blend || coverage == 255 {
                    dest.copy_from_slice(&pixel);
                    continue;
                }
                // Coverage blend in float, alpha accumulated at 1.25x, clamped (`CapHigh`).
                let a = f32::from(coverage) * 0.003_921_569;
                let d = dest
                    .iter()
                    .map(|&v| f32::from(v) * 0.003_921_569)
                    .collect::<Vec<_>>();
                let inv = 1.0 - a;
                let mixed = [
                    color[0] * a + d[0] * inv,
                    color[1] * a + d[1] * inv,
                    color[2] * a + inv * d[2],
                    color[3] * a * 1.25 + d[3],
                ];
                dest.copy_from_slice(&color_bytes(mixed.map(|c| c.clamp(0.0, 1.0))));
            }
        }
        glyph_width
    }

    /// `UIElement_Text::PostBlit` for an element whose box is `me` and dirty part `dirty`
    /// (surface coordinates).
    pub(crate) fn draw_text(&mut self, text: &TextState, me: UiBox, dirty: UiBox) {
        let [up, down, left, right] = text.margins;
        let width = me.width();
        let drawable = text.paper.0 - right - left;
        if drawable <= 0 || text.paper.1 - down - up <= 0 || text.lines.is_empty() {
            return;
        }
        // Not editable and without a horizontal scrollbar: justify the whole paper.
        let view_x = -justify(width, text.paper.0 - right - left, text.justification);
        let window = dirty;
        let origin_x = left - view_x - (dirty.x0 - me.x0);
        let mut y = up - text.view.1 - (dirty.y0 - me.y0);
        let mut x = justify(drawable, text.lines[0].width, text.justification) + origin_x;
        let mut line_height = text.lines[0].height;
        let mut next_line = 1;
        for (index, glyph) in text.glyphs.iter().enumerate() {
            let Some(font) = &glyph.font else {
                continue;
            };
            if next_line < text.lines.len() && text.lines[next_line].index <= index {
                y += line_height;
                x = justify(drawable, text.lines[next_line].width, text.justification) + origin_x;
                line_height = text.lines[next_line].height;
                next_line += 1;
            }
            if glyph.ch == 10 {
                continue;
            }
            if glyph.outline {
                for dy in -1..2 {
                    for dx in -1..2 {
                        if dx != 0 || dy != 0 {
                            let black = [0.0, 0.0, 0.0, 1.0];
                            self.draw_character(
                                window,
                                x + dx,
                                y + dy,
                                font,
                                glyph.ch,
                                black,
                                false,
                            );
                        }
                    }
                }
            }
            x += self.draw_character(window, x, y, font, glyph.ch, glyph.color, true);
        }
    }
}

/// `UIElement_Text::CalcJustification`.
fn justify(draw_width: i32, text_width: i32, justification: Justification) -> i32 {
    match justification {
        Justification::Left => 0,
        Justification::Center => draw_width / 2 - text_width / 2,
        Justification::Right => draw_width - text_width,
    }
}
