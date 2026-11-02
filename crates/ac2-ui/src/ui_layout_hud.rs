//! `/saveui` and `/loadui` over the whole HUD (`ui_layout_text` is the file's format): save writes
//! 0930's nine windows, the style, every other window's place and, in the modern style, the HUD
//! layout's settings; load applies what fits the HUD's style. A load keeps what it applied as a
//! drag and an Edit layout preset load keep theirs: the window places are recorded
//! ([`UiTree::take_positions_changed`]) and the settings marked to be written (`layout_dirty`),
//! which the frame's reload pass saves for the character.

use std::io;
use std::path::{Path, PathBuf};

use ac2_prefs::paths;

use crate::element::UiTree;
use crate::hud_style::HudStyle;
use crate::token::{ALLOW_RESIZE, BROADCAST_LOC_CHANGE, SAVE_LOC};
use crate::tokenize;
use crate::ui_layout::{ui_layout_file, UI_LAYOUT_WINDOWS};
use crate::ui_layout_text::{UiLayoutFile, UiLayoutWindow};
use crate::Hud;

/// What a load did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiLayoutLoad {
    /// The file read.
    pub path: PathBuf,
    /// The style the file says it was saved in, when that is not the HUD's and the file holds
    /// more than 0930's nine lines: its style-specific settings were skipped.
    pub skipped_style: Option<HudStyle>,
    /// A file without a style line that holds window or setting lines (written by hand).
    pub skipped_unstyled: bool,
}

impl UiLayoutLoad {
    /// The chat line that says what was skipped, when something was.
    pub fn notice(&self) -> Option<String> {
        match (self.skipped_style, self.skipped_unstyled) {
            (Some(style), _) => Some(format!(
                "Skipped the settings of the {} HUD style in {}; applied the windows that fit.\n",
                match style {
                    HudStyle::Modern => "modern",
                    HudStyle::Classic => "classic",
                },
                file_name(&self.path)
            )),
            (None, true) => Some(format!(
                "Skipped the settings of another HUD style in {}; applied the windows that fit.\n",
                file_name(&self.path)
            )),
            (None, false) => None,
        }
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
}

impl UiTree {
    /// The windows that are saved by `<WIN>` lines: the movable roots, less the nine windows
    /// (they have their own lines) and, in the modern style, the pinned parts (their place is
    /// their offset in the HUD layout).
    fn ui_layout_other_windows(&self, style: HudStyle) -> Vec<usize> {
        let mut skip: Vec<u32> = UI_LAYOUT_WINDOWS.iter().map(|w| tokenize(w.1)).collect();
        if style == HudStyle::Modern {
            skip.extend(crate::modern_layout_edit::pinned_ids());
        }
        self.roots()
            .iter()
            .copied()
            .filter(|&r| self.is_window_root(r) && !skip.contains(&self.el(r).id()))
            .collect()
    }

    /// Every other movable window's place, for the file.
    pub(crate) fn ui_layout_windows(&self, style: HudStyle) -> Vec<UiLayoutWindow> {
        self.ui_layout_other_windows(style)
            .into_iter()
            .map(|r| {
                let b = self.screen_box(r);
                UiLayoutWindow {
                    layout: self.el(r).layout.did,
                    element: self.el(r).id(),
                    x: b.x0,
                    y: b.y0,
                    width: b.x1 - b.x0 + 1,
                    height: b.y1 - b.y0 + 1,
                }
            })
            .collect()
    }

    /// The file's windows that this tree has are resized (when the window allows it) and moved,
    /// as the nine are; one it has not is skipped. The places are recorded as a finished drag
    /// records them.
    pub(crate) fn apply_ui_layout_windows(&mut self, style: HudStyle, lines: &[UiLayoutWindow]) {
        let others = self.ui_layout_other_windows(style);
        let mut any = false;
        for l in lines {
            let Some(&window) = others
                .iter()
                .find(|&&r| self.el(r).layout.did == l.layout && self.el(r).id() == l.element)
            else {
                continue;
            };
            if self.bool_attribute(window, ALLOW_RESIZE) {
                self.resize_to(window, l.width, l.height);
            }
            self.move_to(window, l.x, l.y);
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

impl Hud {
    /// The HUD's whole layout as a `/saveui` file: the settings are those the panels hold now
    /// (as the frame's reload pass writes them), the same an Edit layout preset saves.
    pub fn ui_layout_file(&self, tree: &UiTree) -> UiLayoutFile {
        let settings = if self.style == HudStyle::Modern {
            let mut layout = self.hud_layout.clone();
            if let Some(vitals) = &self.vitals {
                vitals.export_layout(&mut layout);
            }
            if let Some(chat) = &self.chat {
                chat.export_layout(&mut layout);
            }
            if let Some(pets) = &self.pets {
                pets.export_layout(&mut layout);
            }
            crate::modern_layout_edit::export(tree, &mut layout);
            layout
                .settings()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        } else {
            Vec::new()
        };
        UiLayoutFile {
            nine: tree.ui_layout_lines(),
            style: Some(self.style),
            windows: tree.ui_layout_windows(self.style),
            settings,
        }
    }

    /// `/saveui`: writes the layout to the file for `name` in `dir`; returns the file.
    pub fn save_ui_layout(&self, tree: &UiTree, dir: &Path, name: &str) -> io::Result<PathBuf> {
        let path = ui_layout_file(dir, name);
        paths::write_atomic(&path, self.ui_layout_file(tree).to_text().as_bytes())?;
        Ok(path)
    }

    /// `/loadui`: reads the file for `name` in `dir` and applies it. 0930's nine lines always
    /// apply. A file of this HUD's style also sets the other windows and (modern) the HUD layout
    /// as a preset load does: the settings replace the HUD layout's, then each part applies
    /// them, then the windows are placed. A file of the other style (or with no style line)
    /// applies only the windows this HUD also has. A missing file is an error.
    pub fn load_ui_layout(
        &mut self,
        tree: &mut UiTree,
        dir: &Path,
        name: &str,
    ) -> io::Result<UiLayoutLoad> {
        let path = ui_layout_file(dir, name);
        let bytes = std::fs::read(&path)?;
        let file = UiLayoutFile::parse(&String::from_utf8_lossy(&bytes));
        let same = file.style == Some(self.style);
        if same && self.style == HudStyle::Modern && !file.settings.is_empty() {
            self.hud_layout.replace_settings(&file.settings);
            if let Some(vitals) = self.vitals.as_ref() {
                vitals.apply_layout(tree, &self.hud_layout);
            }
            if let Some(chat) = self.chat.as_mut() {
                chat.apply_layout(tree, &self.hud_layout);
            }
            if let Some(pets) = self.pets.as_ref() {
                pets.apply_layout(&self.hud_layout);
            }
            crate::modern_layout_edit::apply(tree, &self.hud_layout);
            self.layout_dirty = true;
        }
        // The panels first: a field inside a panel (the nine's social, navigation and so on) is
        // moved to its saved screen place over the panel's place, and moves with the panel.
        tree.apply_ui_layout_windows(self.style, &file.windows);
        tree.apply_ui_layout(&file.nine);
        let other = file.style.filter(|&s| s != self.style);
        Ok(UiLayoutLoad {
            path,
            skipped_style: other.filter(|_| file.has_extras()),
            skipped_unstyled: file.style.is_none() && file.has_extras(),
        })
    }
}
