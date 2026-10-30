//! The modern HUD's layout settings, kept per character on this machine (AGENTS.md: the HUD's two
//! styles): the choices the design's windows offer that retail had no setting for (the vitals'
//! layout, each chat window's name, font scale, background, timestamps, fading, line limit and
//! lock). One `key=value` line each in `HudLayout-<character>.txt` beside the window positions;
//! an unknown key is kept as it is, so a newer client's settings survive an older one. Named
//! layouts (the design's presets, saved and loaded from the Edit layout mode) are copies of the
//! other settings and the chat windows' places under `preset.<name>.`.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use ac2_prefs::paths;

/// The saved layouts' key prefix, and a saved window place's key within one.
const PRESET: &str = "preset.";
const WINDOW: &str = "window.";
/// The key every saved layout has, so one saved from no settings still exists.
const MARK: &str = "saved";

/// The settings: keys to values, both plain text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HudLayout {
    values: BTreeMap<String, String>,
}

impl HudLayout {
    /// The value of `key`, if set.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    /// Whether `key` is set to `true` (`default` when unset).
    pub fn flag(&self, key: &str, default: bool) -> bool {
        self.get(key).map_or(default, |v| v == "true")
    }

    /// The number `key` holds (`default` when unset or not a number).
    pub fn number(&self, key: &str, default: i64) -> i64 {
        self.get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    /// `key` set to `value`; whether it changed.
    pub fn set(&mut self, key: &str, value: impl ToString) -> bool {
        let value = value.to_string();
        if self.get(key) == Some(value.as_str()) {
            return false;
        }
        self.values.insert(key.to_string(), value);
        true
    }

    /// The saved layouts' names, in name order.
    pub fn presets(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .values
            .keys()
            .filter_map(|k| k.strip_prefix(PRESET)?.split_once('.'))
            .map(|(name, _)| name.to_string())
            .collect();
        names.dedup();
        names
    }

    /// The settings (not the saved layouts) and the windows' places `windows` (window position
    /// lines) saved as the layout `name` (its `.`, `=` and line breaks made spaces), replacing one
    /// of that name. The name used, or `None` for an empty one.
    pub fn save_preset(&mut self, name: &str, windows: &[String]) -> Option<String> {
        let name: String = name
            .trim()
            .chars()
            .map(|c| {
                if matches!(c, '.' | '=' | '\n' | '\r') {
                    ' '
                } else {
                    c
                }
            })
            .collect();
        if name.is_empty() {
            return None;
        }
        let prefix = format!("{PRESET}{name}.");
        self.values.retain(|k, _| !k.starts_with(&prefix));
        let current: Vec<(String, String)> = self
            .values
            .iter()
            .filter(|(k, _)| !k.starts_with(PRESET))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (k, v) in current {
            self.values.insert(format!("{prefix}{k}"), v);
        }
        for (i, line) in windows.iter().enumerate() {
            self.values
                .insert(format!("{prefix}{WINDOW}{i}"), line.clone());
        }
        self.values.insert(format!("{prefix}{MARK}"), "true".into());
        Some(name)
    }

    /// The layout `name` made the settings (the saved layouts kept); the windows' places it
    /// holds (window position lines), or `None` when there is no such layout.
    pub fn load_preset(&mut self, name: &str) -> Option<Vec<String>> {
        let prefix = format!("{PRESET}{name}.");
        let saved: Vec<(String, String)> = self
            .values
            .iter()
            .filter_map(|(k, v)| Some((k.strip_prefix(&prefix)?.to_string(), v.clone())))
            .collect();
        if saved.is_empty() {
            return None;
        }
        self.values.retain(|k, _| k.starts_with(PRESET));
        let mut windows = Vec::new();
        for (k, v) in saved {
            if k.starts_with(WINDOW) {
                windows.push(v);
            } else if k != MARK {
                self.values.insert(k, v);
            }
        }
        Some(windows)
    }

    /// The settings as the file's text: `key=value` lines in key order.
    pub fn to_text(&self) -> String {
        self.values
            .iter()
            .map(|(k, v)| format!("{k}={v}\n"))
            .collect()
    }

    /// The settings a file's text holds (lines without `=` are skipped; a value may hold `=`).
    pub fn parse(text: &str) -> Self {
        let values = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .map(|(k, v)| (k.trim().to_string(), v.to_string()))
            .filter(|(k, _)| !k.is_empty())
            .collect();
        Self { values }
    }
}

/// The file holding `character`'s HUD layout in `dir`.
pub fn hud_layout_file(dir: &Path, character: u64) -> PathBuf {
    dir.join(format!("HudLayout-{character:016x}.txt"))
}

/// `character`'s HUD layout in `dir`; empty when the file is missing.
pub fn read_hud_layout(dir: &Path, character: u64) -> Result<HudLayout, String> {
    let path = hud_layout_file(dir, character);
    let bytes = paths::read_or_empty(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let text = String::from_utf8(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(HudLayout::parse(&text))
}

/// Writes `character`'s HUD layout into `dir` (atomically).
pub fn write_hud_layout(dir: &Path, character: u64, layout: &HudLayout) -> io::Result<()> {
    paths::write_atomic(
        &hud_layout_file(dir, character),
        layout.to_text().as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::HudLayout;

    #[test]
    fn settings_round_trip_through_the_files_text() {
        let mut layout = HudLayout::default();
        assert!(layout.set("vitals.layout", "stacked"));
        assert!(!layout.set("vitals.layout", "stacked"));
        layout.set("chat.1.name", "Trade = gold");
        layout.set("chat.1.timestamps", true);
        layout.set("chat.1.lines", 200);
        let back = HudLayout::parse(&layout.to_text());
        assert_eq!(back, layout);
        assert_eq!(back.get("chat.1.name"), Some("Trade = gold"));
        assert!(back.flag("chat.1.timestamps", false));
        assert!(back.flag("chat.2.timestamps", true));
        assert_eq!(back.number("chat.1.lines", 0), 200);
        assert_eq!(back.number("chat.2.lines", 500), 500);
    }

    #[test]
    fn a_saved_layout_loads_back_with_its_windows() {
        let mut layout = HudLayout::default();
        layout.set("vitals.layout", "stacked");
        layout.set("offset.radar", "10,20");
        let windows = vec!["window 22000001 a05c6b95 -1 1 0 0 1365 768".to_string()];
        assert_eq!(
            layout.save_preset(" Raid.Night ", &windows).as_deref(),
            Some("Raid Night")
        );
        layout.set("vitals.layout", "side");
        layout.set("offset.strip", "0,-30");
        assert_eq!(layout.presets(), ["Raid Night"]);
        assert_eq!(layout.load_preset("Raid Night"), Some(windows));
        assert_eq!(layout.get("vitals.layout"), Some("stacked"));
        assert_eq!(layout.get("offset.radar"), Some("10,20"));
        assert_eq!(layout.get("offset.strip"), None);
        assert_eq!(layout.presets(), ["Raid Night"]);
        assert_eq!(layout.load_preset("none"), None);
        // Kept through the file's text.
        assert_eq!(HudLayout::parse(&layout.to_text()), layout);
        assert_eq!(layout.save_preset("  ", &[]), None);
        // A layout saved from no settings still exists and loads.
        let mut empty = HudLayout::default();
        empty.save_preset("Bare", &[]);
        assert_eq!(empty.presets(), ["Bare"]);
        assert_eq!(empty.load_preset("Bare"), Some(Vec::new()));
        assert_eq!(empty.get("saved"), None);
    }
}
