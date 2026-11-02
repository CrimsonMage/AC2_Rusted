//! `/saveui` and `/loadui`: nine HUD windows' places written to and read from a text file. The
//! commands come from the 2013 relaunch client (build 0930) and are added to this client
//! (`AGENTS.md`); the chat side is `ac2_weenie`'s shell.
//!
//! Save is `gmGameplayUI` 0x443030: for each window, in order, the line
//! `"<SOCIA> X:%d Y: %d W: %d H: %d\n"` with `UIRegion::GetScreenX0` / `GetScreenY0` and the box's
//! width and height (`x1 - x0 + 1`, `y1 - y0 + 1`). Load is 0x442D40: each line is read with
//! `sscanf("%s X:%d Y: %d W: %d H: %d")`; a line that does not give 5 values or whose tag is not
//! one of the nine is skipped; the window then takes `UIElement::ResizeTo(W, H)` and
//! `UIElement::MoveTo(X, Y)`, in that order (a window inside a panel is moved to the screen place
//! it was saved at, see [`UiTree::apply_ui_layout`]). Retail did not check that a window exists; a
//! window this tree does not have is skipped. The file name is `"%s%s.txt"` (directory, name,
//! 0x442540), here in the player's AC2 directory (`ui` folder) with the name made safe.
//!
//! The file holds more than these nine lines: the style, the other windows and, in the modern
//! style, the HUD layout (`ui_layout_text` is the format, `ui_layout_hud` the save and load).

use std::path::{Path, PathBuf};

use crate::element::UiTree;
use crate::token::{BROADCAST_LOC_CHANGE, SAVE_LOC};
use crate::tokenize;

/// The nine windows `gmGameplayUI` saves, in order: the line's tag and the element's name.
pub const UI_LAYOUT_WINDOWS: [(&str, &str); 9] = [
    ("<SOCIA>", "SocialField"),
    ("<QUEST>", "QuestField"),
    ("<SKILL>", "SkillField"),
    ("<NAVIG>", "NavigationField"),
    ("<EXAMI>", "ExaminationField"),
    ("<CHAT1>", "ChatWindow1"),
    ("<CHAT2>", "ChatWindow2"),
    ("<CHAT3>", "ChatWindow3"),
    ("<CHAT4>", "ChatWindow4"),
];

/// One window's line: its tag, screen position and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLayoutLine {
    pub tag: &'static str,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// The text of `lines`: `"<TAG> X:%d Y: %d W: %d H: %d\n"` each.
pub fn ui_layout_to_text(lines: &[UiLayoutLine]) -> String {
    let mut text = String::new();
    for l in lines {
        text.push_str(&format!(
            "{} X:{} Y: {} W: {} H: {}\n",
            l.tag, l.x, l.y, l.width, l.height
        ));
    }
    text
}

/// A C `%d` conversion at `rest`: leading white space, an optional sign, digits.
fn scan_int(rest: &str) -> Option<(i32, &str)> {
    let rest = rest.trim_start();
    let digits_at = usize::from(rest.starts_with(['+', '-']));
    let end = rest[digits_at..]
        .find(|c: char| !c.is_ascii_digit())
        .map_or(rest.len(), |n| n + digits_at);
    if end == digits_at {
        return None;
    }
    Some((rest[..end].parse().ok()?, &rest[end..]))
}

/// A literal in a `sscanf` format: white space before it is skipped.
fn scan_literal<'a>(rest: &'a str, literal: &str) -> Option<&'a str> {
    rest.trim_start().strip_prefix(literal)
}

/// One line by `sscanf("%s X:%d Y: %d W: %d H: %d")`: the tag word and four values, when all
/// five conversions succeed.
fn scan_line(line: &str) -> Option<(&str, [i32; 4])> {
    let (tag, rest) = scan_word(line)?;
    Some((tag, scan_values(rest)?))
}

/// A C `%s` conversion: the word after any white space, and what follows it.
pub(crate) fn scan_word(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_start();
    let end = line.find(char::is_whitespace).unwrap_or(line.len());
    let (word, rest) = line.split_at(end);
    (!word.is_empty()).then_some((word, rest))
}

/// The ` X:%d Y: %d W: %d H: %d` of a line, when all four conversions succeed.
pub(crate) fn scan_values(mut rest: &str) -> Option<[i32; 4]> {
    let mut values = [0; 4];
    for (value, label) in values.iter_mut().zip(["X:", "Y:", "W:", "H:"]) {
        let (number, after) = scan_int(scan_literal(rest, label)?)?;
        *value = number;
        rest = after;
    }
    Some(values)
}

/// The lines of a file's text that name one of the nine windows (the first 8 bytes of the tag
/// are compared with `"<SOCIA>\0"` and so on); any other line is skipped.
pub fn ui_layout_parse(text: &str) -> Vec<UiLayoutLine> {
    text.lines()
        .filter_map(|line| {
            let (tag, [x, y, width, height]) = scan_line(line)?;
            let (tag, _) = UI_LAYOUT_WINDOWS.iter().find(|(t, _)| *t == tag)?;
            Some(UiLayoutLine {
                tag,
                x,
                y,
                width,
                height,
            })
        })
        .collect()
}

/// The file name for `name` (`"%s%s.txt"`): the name made safe so the file stays in the
/// directory. Path separators, drive colons and control characters become `_`, and leading dots
/// are replaced so the name is never `.` or `..`; an empty name becomes `_`.
pub fn ui_layout_file_name(name: &str) -> String {
    let mut safe: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let dots = safe.len() - safe.trim_start_matches('.').len();
    safe.replace_range(..dots, &"_".repeat(dots));
    if safe.is_empty() {
        safe.push('_');
    }
    format!("{safe}.txt")
}

/// The layout file for `name` in `dir`.
pub fn ui_layout_file(dir: &Path, name: &str) -> PathBuf {
    dir.join(ui_layout_file_name(name))
}

impl UiTree {
    /// The window called `name`: a root with that id, or the first descendant of a root with it
    /// (`IUIManager::GetElementByID`'s search by `Tokenize(name)`).
    pub(crate) fn ui_layout_window(&self, name: &str) -> Option<usize> {
        let id = tokenize(name);
        self.roots()
            .iter()
            .find_map(|&r| (self.el(r).id() == id).then_some(r))
            .or_else(|| {
                self.roots()
                    .iter()
                    .find_map(|&r| self.child_recursive(r, id))
            })
    }

    /// `gmGameplayUI` 0x443030: the nine windows' lines, in order; a window the tree does not
    /// have is left out.
    pub fn ui_layout_lines(&self) -> Vec<UiLayoutLine> {
        UI_LAYOUT_WINDOWS
            .iter()
            .filter_map(|&(tag, name)| {
                let window = self.ui_layout_window(name)?;
                let b = self.screen_box(window);
                Some(UiLayoutLine {
                    tag,
                    x: b.x0,
                    y: b.y0,
                    width: b.x1 - b.x0 + 1,
                    height: b.y1 - b.y0 + 1,
                })
            })
            .collect()
    }

    /// `gmGameplayUI` 0x442D40: each line's window is resized, then moved. The new places are
    /// recorded as a finished drag records them ([`UiTree::take_positions_changed`], and the
    /// location upload for a window that saves its location), so the client keeps them.
    pub fn apply_ui_layout(&mut self, lines: &[UiLayoutLine]) {
        let mut any = false;
        for l in lines {
            let Some(&(_, name)) = UI_LAYOUT_WINDOWS.iter().find(|(t, _)| *t == l.tag) else {
                continue;
            };
            let Some(window) = self.ui_layout_window(name) else {
                continue;
            };
            self.resize_to(window, l.width, l.height);
            // `UIRegion::MoveTo` takes the box in the parent's coordinates, and retail passed
            // the saved screen place as it was, which put a window inside a panel (the social,
            // quest, skill, navigation and examination fields here) off by the panel's place.
            // A child is moved to the screen place it was saved at (`AGENTS.md`).
            let (ox, oy) = match self.el(window).parent {
                Some(parent) if self.el(window).object.is_none() => {
                    let p = self.screen_box(parent);
                    (p.x0, p.y0)
                }
                _ => (0, 0),
            };
            self.move_to(window, l.x - ox, l.y - oy);
            if self.bool_attribute(window, SAVE_LOC)
                || self.bool_attribute(window, BROADCAST_LOC_CHANGE)
            {
                self.locations_changed = true;
            }
            any = true;
        }
        if any {
            self.record_window_positions();
        }
    }
}
