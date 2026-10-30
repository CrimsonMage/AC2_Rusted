//! The player's window positions, kept per character. Retail moved a window only by its drag bar
//! and saved only the windows that save their location (`save_loc`) to the server
//! (`UIMainFramework::InqLocations`, `gmCWInterface::SetUISaveLocations`). Here, by the window
//! exception in `AGENTS.md`, every HUD window keeps its place on this machine too, the radar, bars
//! and toolbar included: [`UiTree::window_positions`] lists every window's corners in clip space,
//! the client writes them when a drag ends, and [`UiTree::set_window_positions`] applies them at
//! login and to a window when it is made (`UIMainFramework::ApplyLocations` for each root).
//! The overlay roots are not windows: the world's targeting and name plates, damage text, and the
//! full-display backdrops ([`UiTree::set_overlay_root`]).
//!
//! The file is a line per window, `window <layout> <element> <x0> <y0> <x1> <y1>`, the ids in hex
//! and the corners as clip-space floats (round-trip exact). A `#` line is a comment.

use std::io;
use std::path::{Path, PathBuf};

use ac2_prefs::paths;

use crate::HudStyle;

use crate::element::{UiBox, UiTree};
use crate::token::ALLOW_RESIZE;
use crate::window_locations::{clip_to_pixel, pixel_to_clip};

/// One window's saved corners in clip space, under its layout and element ids, and the display
/// (logical pixels) they were saved on (`None` in a file written before it was kept).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
    pub layout: u32,
    pub element: u32,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub display: Option<(i32, i32)>,
}

/// The file's header comment.
const HEADER: &str =
    "# AC2 window positions: window <layout> <element> <x0> <y0> <x1> <y1> [<width> <height>]";

/// The file's text: a header, then a line per window.
pub fn to_text(positions: &[WindowPosition]) -> String {
    let mut text = String::from(HEADER);
    text.push('\n');
    for p in positions {
        text.push_str(&format!(
            "window {:08x} {:08x} {} {} {} {}",
            p.layout, p.element, p.x0, p.y0, p.x1, p.y1
        ));
        if let Some((w, h)) = p.display {
            text.push_str(&format!(" {w} {h}"));
        }
        text.push('\n');
    }
    text
}

/// The positions in a file's text. A malformed line is an error (the file is not applied).
pub fn parse(text: &str) -> Result<Vec<WindowPosition>, String> {
    let mut out = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        let bad = || format!("line {}: not a window position: {line}", number + 1);
        let (head, display) = match fields.len() {
            7 => (&fields[..], None),
            9 => (&fields[..7], Some((fields[7], fields[8]))),
            _ => return Err(bad()),
        };
        let [tag, layout, element, x0, y0, x1, y1] = head else {
            return Err(bad());
        };
        if *tag != "window" {
            return Err(bad());
        }
        let hex = |s: &str| u32::from_str_radix(s, 16).map_err(|_| bad());
        let float = |s: &str| s.parse::<f32>().map_err(|_| bad());
        let int = |s: &str| s.parse::<i32>().map_err(|_| bad());
        let display = match display {
            Some((w, h)) => Some((int(w)?, int(h)?)),
            None => None,
        };
        out.push(WindowPosition {
            layout: hex(layout)?,
            element: hex(element)?,
            x0: float(x0)?,
            y0: float(y0)?,
            x1: float(x1)?,
            y1: float(y1)?,
            display,
        });
    }
    Ok(out)
}

/// The file of `character`'s window positions in `dir` for the HUD's `style`: each style keeps
/// its own, as the modern style's windows have their own sizes.
pub fn window_positions_file(dir: &Path, character: u64, style: HudStyle) -> PathBuf {
    match style {
        HudStyle::Classic => dir.join(format!("WindowPositions-{character:016x}.txt")),
        HudStyle::Modern => dir.join(format!("WindowPositions-{character:016x}-modern.txt")),
    }
}

/// `character`'s saved window positions in `dir` for `style`; none when the file is missing or
/// empty (the player has not moved a window yet). A file that does not parse is an error naming
/// the file.
pub fn read_window_positions(
    dir: &Path,
    character: u64,
    style: HudStyle,
) -> Result<Vec<WindowPosition>, String> {
    let path = window_positions_file(dir, character, style);
    let bytes = paths::read_or_empty(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    let text = String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    parse(&text)
}

/// Writes `character`'s window positions into `dir` (atomically; the windows' places when a drag
/// ended).
pub fn write_window_positions(
    dir: &Path,
    character: u64,
    style: HudStyle,
    positions: &[WindowPosition],
) -> io::Result<()> {
    let text = to_text(positions);
    paths::write_atomic(
        &window_positions_file(dir, character, style),
        text.as_bytes(),
    )
}

/// The file that marks `character`'s radar coordinates as hidden in `dir`. The radar shows its
/// coordinates box unless the player turned it off in this client (`AGENTS.md`; retail opened it
/// only from the profile's `PROFILE_SHOW_RADAR_COORDS` bit, which is still kept and uploaded).
pub fn radar_coords_hidden_file(dir: &Path, character: u64) -> PathBuf {
    dir.join(format!("RadarCoordsHidden-{character:016x}.txt"))
}

/// Whether `character` turned the radar coordinates off in this client.
pub fn read_radar_coords_hidden(dir: &Path, character: u64) -> bool {
    radar_coords_hidden_file(dir, character).exists()
}

/// Keeps or clears `character`'s choice to hide the radar coordinates.
pub fn write_radar_coords_hidden(dir: &Path, character: u64, hidden: bool) -> io::Result<()> {
    let path = radar_coords_hidden_file(dir, character);
    if hidden {
        paths::write_atomic(&path, b"hidden\n")
    } else {
        match std::fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }
}

/// One axis of a window saved on a display `from` long, moved to one `to` long: the window keeps
/// its length and its distance from the edge it was nearest (its centre in the first third: the
/// near edge; in the last: the far one; else the middle).
fn anchor_axis(start: i32, end: i32, from: i32, to: i32) -> (i32, i32) {
    let length = end - start;
    let centre = (start + end) as f32 / 2.0;
    let third = from as f32 / 3.0;
    let start = if centre < third {
        start
    } else if centre > 2.0 * third {
        to - (from - start)
    } else {
        to / 2 - (from / 2 - start)
    };
    (start, start + length)
}

/// A saved place (clip space on display `from`) on display `to`, anchored ([`anchor_axis`]).
pub(crate) fn anchored(
    saved: &WindowPosition,
    from: (i32, i32),
    to: (i32, i32),
) -> (i32, i32, i32, i32) {
    let (x0, y0) = clip_to_pixel(saved.x0, saved.y0, from.0, from.1);
    let (x1, y1) = clip_to_pixel(saved.x1, saved.y1, from.0, from.1);
    let (x0, x1) = anchor_axis(x0, x1, from.0, to.0);
    let (y0, y1) = anchor_axis(y0, y1, from.1, to.1);
    (x0, y0, x1, y1)
}

impl UiTree {
    /// Whether `index` is a window the player moves and whose place is kept: a root that is not an
    /// overlay and does not cover the whole display.
    pub(crate) fn is_window_root(&self, index: usize) -> bool {
        self.roots.contains(&index) && !self.el(index).overlay && !self.covers_display(index)
    }

    /// Whether the root's box covers the whole display (a full-screen backdrop, never a window).
    fn covers_display(&self, index: usize) -> bool {
        let (width, height) = self.display;
        let b = self.screen_box(index);
        b.x0 <= 0 && b.y0 <= 0 && b.x1 >= width - 1 && b.y1 >= height - 1
    }

    /// Marks `index` as an overlay root: a world overlay, a transient, or a part that moves with
    /// another window. It does not move by Alt + drag and its place is not kept.
    pub fn set_overlay_root(&mut self, index: usize) {
        self.el_mut(index).overlay = true;
    }

    /// Every window's place (the roots that are windows), in clip space.
    pub fn window_positions(&self) -> Vec<WindowPosition> {
        let (width, height) = self.display;
        self.roots
            .iter()
            .filter(|&&root| self.is_window_root(root))
            .map(|&root| {
                let el = self.el(root);
                let screen = self.screen_box(root);
                let (x0, y0) = pixel_to_clip(screen.x0, screen.y0, width, height);
                let (x1, y1) = pixel_to_clip(screen.x1, screen.y1, width, height);
                WindowPosition {
                    layout: el.layout.did,
                    element: el.id(),
                    x0,
                    y0,
                    x1,
                    y1,
                    display: Some((width, height)),
                }
            })
            .collect()
    }

    /// The player's saved positions: applied to the windows already up now, and to each window
    /// made later (`create_root`). A later call replaces them.
    pub fn set_window_positions(&mut self, saved: &[WindowPosition]) {
        self.saved_positions = saved.to_vec();
        for root in self.roots.clone() {
            self.apply_saved_position(root);
        }
        self.moved = true;
        self.changed = true;
    }

    /// The window `index` back where its layout puts it (as [`Self::create_root`] placed it), its
    /// saved place forgotten (the modern Edit layout mode's Reset to default).
    pub fn reset_window_place(&mut self, index: usize) {
        let (layout, element) = {
            let el = self.el(index);
            (el.layout.did, el.id())
        };
        self.saved_positions
            .retain(|p| p.layout != layout || p.element != element);
        let el = self.el_mut(index);
        el.rect = UiBox::new(0, 0, -1, -1);
        el.object = Some((0, 0));
        self.update_size_and_position(index);
        self.moved = true;
        self.changed = true;
    }

    /// `UIElement::ApplyLocation` for a saved position: the window moves to its corners (clamped
    /// on the display by `move_to`) and is resized when it allows it. The shown state is the
    /// window's own.
    pub(crate) fn apply_saved_position(&mut self, index: usize) {
        if !self.is_window_root(index) {
            return;
        }
        let el = self.el(index);
        let Some(saved) = self
            .saved_positions
            .iter()
            .find(|p| p.layout == el.layout.did && p.element == el.id())
            .copied()
        else {
            return;
        };
        let (width, height) = self.display;
        let (x0, y0, x1, y1) = match saved.display {
            // Saved on another display: kept at its size and its distance from the edges it was
            // nearest (`anchored`), where clip space would scale both with the display.
            Some(on) if on != (width, height) => anchored(&saved, on, (width, height)),
            _ => {
                let (x0, y0) = clip_to_pixel(saved.x0, saved.y0, width, height);
                let (x1, y1) = clip_to_pixel(saved.x1, saved.y1, width, height);
                (x0, y0, x1, y1)
            }
        };
        // The size first: `move_to` clamps the origin on the display for the final size, so a
        // saved window larger than this display still starts on it.
        if self.bool_attribute(index, ALLOW_RESIZE) {
            self.resize_to(index, x1 - x0 + 1, y1 - y0 + 1);
        }
        self.move_to(index, x0, y0);
        self.moved = true;
        self.changed = true;
    }

    /// A drag or resize ended with a window somewhere new: every window's place becomes the
    /// saved places (what the client writes, [`UiTree::take_positions_changed`]), so a display
    /// change puts them back ([`UiTree::set_display`]).
    pub(crate) fn record_window_positions(&mut self) {
        self.saved_positions = self.window_positions();
        self.positions_changed = true;
    }

    /// Whether a window was moved and its drag ended since the last call: the client writes the
    /// positions then ([`UiTree::window_positions`]).
    pub fn take_positions_changed(&mut self) -> bool {
        std::mem::take(&mut self.positions_changed)
    }
}
