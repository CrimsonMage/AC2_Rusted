//! `GlyphList`'s searches over the lines and glyphs of a text, and the cursor moves built on
//! them: the line a position is in (`FindCurrentLine`), the next and previous line, the line at
//! a height (`FindLineFromY`), the next and previous word, a position's pixel offset in its line
//! (`FindPixelsFromPos`), the position nearest a pixel offset in a line
//! (`FindPosFromLineAndPixels`), the position under a point (`DeterminePositionFromXY`), the line ends
//! (`UIElement_Text::MoveBeginEndLine`) and the up and down moves (`MoveUpDown`).

use crate::text::{Justification, TextState};

/// `Glyph::IsWhiteSpace`: tab, carriage return and space (a newline is not white space).
fn is_white(ch: u16) -> bool {
    matches!(ch, 9 | 13 | 32)
}

impl TextState {
    /// `GlyphList::FindCurrentLine`: `(line, first position of the line)` for a position within
    /// the text (`0..=len`); `None` outside it. Text with no lines is one line from `0`.
    pub(crate) fn find_current_line(&self, start: isize) -> Option<(usize, usize)> {
        if start < 0 || start as usize > self.glyphs.len() {
            return None;
        }
        let start = start as usize;
        let count = self.lines.len();
        if count == 0 {
            return Some((0, 0));
        }
        let mut line = 0;
        while line != count - 1 && self.lines[line + 1].index <= start {
            line += 1;
        }
        Some((line, self.lines[line].index))
    }

    /// `GlyphList::FindLineFromY`: the line `y` pixels down the text (`0` at or above the top, the
    /// last line past the bottom).
    pub(crate) fn find_line_from_y(&self, y: i32) -> usize {
        if y < 1 {
            return 0;
        }
        let mut total = 0;
        for (line, l) in self.lines.iter().enumerate() {
            total += l.height;
            if y <= total {
                return line;
            }
        }
        self.lines.len().saturating_sub(1)
    }

    /// `GlyphList::FindNextLine`: the line after the position's, and its first position (the
    /// last line stays).
    pub(crate) fn find_next_line(&self, start: isize) -> Option<(usize, usize)> {
        let (line, _) = self.find_current_line(start)?;
        let last = self.lines.len().checked_sub(1)?;
        let line = (line + 1).min(last);
        Some((line, self.lines[line].index))
    }

    /// `GlyphList::FindPrevLine`: the line before the position's, and its first position (the
    /// first line stays).
    pub(crate) fn find_prev_line(&self, start: isize) -> Option<(usize, usize)> {
        let (line, _) = self.find_current_line(start)?;
        self.lines.first()?;
        let line = line.saturating_sub(1);
        Some((line, self.lines[line].index))
    }

    /// `GlyphList::GetGlyphLineWidth`.
    pub(crate) fn line_width(&self, line: usize) -> Option<i32> {
        self.lines.get(line).map(|l| l.width)
    }

    /// `GlyphList::FindNextWord`: from `start` past the rest of the word and the white space
    /// after it, to the first glyph of the next word (the end of the text when there is none).
    pub(crate) fn find_next_word(&self, start: isize) -> Option<usize> {
        if start < 0 || start as usize > self.glyphs.len() {
            return None;
        }
        let start = start as usize;
        let mut pos = start;
        let mut seen_white = false;
        for glyph in self.glyphs.iter().skip(start) {
            if is_white(glyph.ch) {
                seen_white = true;
            } else if seen_white {
                break;
            }
            pos += 1;
        }
        Some(pos)
    }

    /// `GlyphList::FindPrevWord`: the first glyph of the last word that starts before `start`
    /// (`0` when none does).
    pub(crate) fn find_prev_word(&self, start: isize) -> Option<usize> {
        if start < 0 || start as usize > self.glyphs.len() {
            return None;
        }
        let mut word_start = 0;
        let mut after_white = true;
        for (i, glyph) in self.glyphs.iter().take(start as usize).enumerate() {
            if is_white(glyph.ch) {
                after_white = true;
            } else if after_white {
                after_white = false;
                word_start = i;
            }
        }
        Some(word_start)
    }

    /// `GlyphList::FindPixelsFromPos`: the widths of the glyphs from the start of the position's
    /// line up to the position.
    pub(crate) fn find_pixels_from_pos(&self, start: isize) -> Option<i32> {
        let (_, line_start) = self.find_current_line(start)?;
        Some(
            self.glyphs
                .iter()
                .take(start as usize)
                .skip(line_start)
                .map(|g| g.width)
                .sum(),
        )
    }

    /// `GlyphList::FindPosFromLineAndPixels`: the position in `line` nearest `pixels` from its
    /// left, a glyph counting as passed once more than half of it is behind. Past the end of the
    /// last line it is the end of the text.
    pub(crate) fn find_pos_from_line_and_pixels(&self, line: usize, pixels: i32) -> Option<usize> {
        let len = self.glyphs.len();
        // Text not laid out yet (no lines) is one line over all of it.
        let first = if self.lines.is_empty() && line == 0 {
            0
        } else {
            self.lines.get(line)?.index
        };
        let end = if line + 1 >= self.lines.len() {
            len
        } else {
            self.lines[line + 1].index
        };
        let mut pos = first;
        let mut left = pixels;
        for i in first..end.min(len) {
            let width = self.glyphs[i].width;
            if left as f32 <= width as f32 * 0.5 || i + 1 == end {
                left -= width;
                break;
            }
            left -= width;
            pos += 1;
        }
        if left > 0 && end == len {
            pos = len;
        }
        Some(pos)
    }

    /// `UIElement_Text::MoveBeginEndLine`: the first position of the line (`begin`), or its end.
    /// The end is the position before the next line's first, and the end of the text when the
    /// next line (the line itself if it is the last) is the last: `FindNextLine` overwrites the
    /// line it was asked from, and the last-line test reads that.
    pub(crate) fn move_begin_end_line(&self, begin: bool, start: isize) -> Option<usize> {
        let (_, line_start) = self.find_current_line(start)?;
        if begin {
            return Some(line_start);
        }
        let Some((next, next_start)) = self.find_next_line(start) else {
            return Some(self.glyphs.len());
        };
        if next == self.lines.len() - 1 {
            Some(self.glyphs.len())
        } else {
            Some(next_start.saturating_sub(1))
        }
    }

    /// The left edge of a line of `width` pixels in the text's drawing width
    /// (`UIElement_Text::CalcJustification`).
    pub(crate) fn line_offset(&self, width: i32) -> i32 {
        let draw = self.paper.0 - self.margins[3] - self.margins[2];
        match self.justification {
            Justification::Left => 0,
            Justification::Center => draw / 2 - width / 2,
            Justification::Right => draw - width,
        }
    }

    /// `UIElement_Text::MoveUpDown`: the cursor's position one line up or down at the same pixel
    /// column (the lines' justification taken into account). On the first line going up, or the
    /// last going down, the position stays.
    pub(crate) fn move_up_down(&self, up: bool, cursor: isize) -> Option<usize> {
        let (line, _) = self.find_current_line(cursor)?;
        if (up && line == 0) || (!up && line + 1 >= self.lines.len()) {
            return Some(cursor as usize);
        }
        let pixels = self.find_pixels_from_pos(cursor)?;
        let (new_line, _) = if up {
            self.find_prev_line(cursor)?
        } else {
            self.find_next_line(cursor)?
        };
        let pixels = pixels + self.line_offset(self.line_width(line)?)
            - self.line_offset(self.line_width(new_line)?);
        self.find_pos_from_line_and_pixels(new_line, pixels)
    }
}

impl TextState {
    /// `UIElement_Text::DeterminePositionFromXY`: the position under the element pixel `(x, y)`:
    /// the line at that height (`FindLineFromY`), then the glyph boundary nearest `x` in it, with
    /// the line's justification and the view and margins taken off. Always within the text.
    pub(crate) fn position_at(&self, x: i32, y: i32) -> usize {
        let px = self.view.0 - self.margins[2] + x;
        let py = self.view.1 - self.margins[0] + y;
        let line = self.find_line_from_y(py);
        let width = self
            .lines
            .get(line)
            .map_or_else(|| self.glyphs.iter().map(|g| g.width).sum(), |l| l.width);
        let offset = self.line_offset(width);
        self.find_pos_from_line_and_pixels(line, px - offset)
            .map_or(0, |pos| pos.min(self.glyphs.len()))
    }
}
