//! The modern HUD's layout settings, kept per character on this machine (AGENTS.md: the HUD's two
//! styles): the choices the design's windows offer that retail had no setting for (the vitals'
//! layout, each chat window's name, font scale, background, timestamps, fading, line limit and
//! lock). One `key=value` line each in `HudLayout-<character>.txt` beside the window positions;
//! an unknown key is kept as it is, so a newer client's settings survive an older one.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use ac2_prefs::paths;

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
}
