//! The `/saveui` file beyond client 0930's nine lines (`ui_layout`; `AGENTS.md`): which HUD style
//! wrote it, the other windows' places and, in the modern style, the HUD layout's settings. The
//! lines, in the order they are written:
//!
//! ```text
//! <SOCIA> X:10 Y: 20 W: 300 H: 200     the nine 0930 lines, exactly as 0930 writes them
//! ...
//! <STYLE> modern                       the style that wrote the file (modern or classic)
//! <WIN> 22000001:a05c6b95 X:0 Y: 0 W: 200 H: 90   a window: layout DID:element id (hex), place
//! <SET> chat.1.name=Trade\nTwo         a HUD layout setting: key=value, value escaped
//! ```
//!
//! 0930 reads a line with `sscanf("%s X:%d Y: %d W: %d H: %d")` and keeps the nine tags only, so
//! it skips `<STYLE>`, `<WIN>` and `<SET>` lines (a `<WIN>` line's id word is not `X:`), and the
//! client reads a file 0930 wrote (nine lines) as it always did. A window's `<WIN>` line is keyed
//! by its layout and element ids, the `Tokenize` hash of its element name (the tree keeps no
//! names), and holds the screen place and size as the nine lines do. A setting's value escapes
//! `\` as `\\` and line breaks as `\n` and `\r`, so a name with spaces or `=` round-trips. Any
//! line that is not one of these is skipped.

use crate::hud_style::HudStyle;
use crate::ui_layout::{scan_values, scan_word, ui_layout_parse, ui_layout_to_text, UiLayoutLine};

/// The tag of the style line, a window line and a setting line.
const STYLE_TAG: &str = "<STYLE>";
const WINDOW_TAG: &str = "<WIN>";
const SETTING_TAG: &str = "<SET>";

/// One other window's place: its layout and element ids and its screen box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLayoutWindow {
    pub layout: u32,
    pub element: u32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// What a `/saveui` file holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiLayoutFile {
    /// 0930's nine windows.
    pub nine: Vec<UiLayoutLine>,
    /// The style that wrote the file; none for a file 0930 wrote.
    pub style: Option<HudStyle>,
    /// The other windows' places.
    pub windows: Vec<UiLayoutWindow>,
    /// The HUD layout's settings (`HudLayout::settings`), keys and unescaped values.
    pub settings: Vec<(String, String)>,
}

fn style_word(style: HudStyle) -> &'static str {
    match style {
        HudStyle::Modern => "modern",
        HudStyle::Classic => "classic",
    }
}

/// `\` and line breaks in `value` as the file writes them.
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out
}

/// The inverse of [`escape`]; an unknown escape keeps its character.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

impl UiLayoutFile {
    /// The file's text: the nine lines, the style, the windows, then the settings.
    pub fn to_text(&self) -> String {
        let mut text = ui_layout_to_text(&self.nine);
        if let Some(style) = self.style {
            text.push_str(&format!("{STYLE_TAG} {}\n", style_word(style)));
        }
        for w in &self.windows {
            text.push_str(&format!(
                "{WINDOW_TAG} {:08x}:{:08x} X:{} Y: {} W: {} H: {}\n",
                w.layout, w.element, w.x, w.y, w.width, w.height
            ));
        }
        for (key, value) in &self.settings {
            text.push_str(&format!("{SETTING_TAG} {key}={}\n", escape(value)));
        }
        text
    }

    /// What a file's text holds; lines that are none of these are skipped. A setting repeated
    /// keeps its last value, a style line the last one.
    pub fn parse(text: &str) -> Self {
        let mut file = Self {
            nine: ui_layout_parse(text),
            ..Self::default()
        };
        for line in text.lines() {
            let Some((tag, rest)) = scan_word(line) else {
                continue;
            };
            match tag {
                STYLE_TAG => {
                    if let Some((word, _)) = scan_word(rest) {
                        file.style = HudStyle::parse(word).or(file.style);
                    }
                }
                WINDOW_TAG => file.windows.extend(parse_window(rest)),
                SETTING_TAG => {
                    let Some((key, value)) = rest.trim_start().split_once('=') else {
                        continue;
                    };
                    let key = key.trim_end().to_string();
                    if key.is_empty() {
                        continue;
                    }
                    let value = unescape(value);
                    match file.settings.iter_mut().find(|(k, _)| *k == key) {
                        Some(entry) => entry.1 = value,
                        None => file.settings.push((key, value)),
                    }
                }
                _ => {}
            }
        }
        file
    }

    /// Whether the file holds anything beyond 0930's nine lines.
    pub fn has_extras(&self) -> bool {
        self.style.is_some() || !self.windows.is_empty() || !self.settings.is_empty()
    }
}

/// A window line after its tag: `<layout>:<element> X:%d Y: %d W: %d H: %d`.
fn parse_window(rest: &str) -> Option<UiLayoutWindow> {
    let (ids, rest) = scan_word(rest)?;
    let (layout, element) = ids.split_once(':')?;
    let [x, y, width, height] = scan_values(rest)?;
    Some(UiLayoutWindow {
        layout: u32::from_str_radix(layout, 16).ok()?,
        element: u32::from_str_radix(element, 16).ok()?,
        x,
        y,
        width,
        height,
    })
}
