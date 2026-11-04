//! The automatic tools' window and file over the client (`ac2_macro::automation`, a modern
//! addition; the tools themselves run in the client, `automation`): the character's settings are
//! read once it is in the world and written whenever they change (a press in the window, or a
//! tool that switched itself off), both on a short-lived worker so the frame never touches a file;
//! the window's presses change them; the window shows them with the player's effects and the
//! skills on the shortcut bar; the tools' lines go to chat.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};

use ac2_macro::automation::settings::{file_name, DIRECTORY};
use ac2_macro::automation::{Change, Settings};
use ac2_net::protocol::ShortcutInfo;
use ac2_ui::automation_ui::AutomationPresentation;

use crate::frame::ClientFrame;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

/// The shortcut type of a skill (`ShortcutInfo` 0x40000001).
const SKILL_SHORTCUT: u32 = 0x4000_0001;
/// The window is refreshed this often while it is open (seconds), for the effect list.
const REFRESH_SECONDS: f64 = 1.0;

/// The character's settings file and the window's state.
#[derive(Default)]
pub(crate) struct AutomationPanel {
    character: Option<u64>,
    loading: Option<Receiver<Option<Vec<u8>>>>,
    /// What the file holds (after the read, or the last write).
    saved: Option<Settings>,
    shown: Option<AutomationPresentation>,
    refreshed: f64,
}

fn settings_path(character: u64) -> PathBuf {
    ac2_prefs::paths::config_dir()
        .join(DIRECTORY)
        .join(file_name(character))
}

fn save(character: u64, bytes: Vec<u8>) {
    let _ = std::thread::Builder::new()
        .name("ac2 automation save".into())
        .spawn(move || {
            let path = settings_path(character);
            let result = path
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| std::fs::write(&path, &bytes));
            if let Err(error) = result {
                eprintln!(
                    "ac2: the automatic tools' settings {}: {error}",
                    path.display()
                );
            }
        });
}

impl HudView {
    /// One frame: the file read and written, the window's changes, the tools' lines, and the
    /// window shown.
    pub(crate) fn automation_frame(
        &mut self,
        player: Option<&ClientPlayer<'_>>,
        frame: &ClientFrame,
        changes: Vec<Change>,
    ) {
        let Some(player) = player else {
            return;
        };
        let character = frame
            .player
            .as_ref()
            .map(|p| p.id)
            .filter(|_| frame.in_world);
        if character != self.automation.character {
            // Another character (or none): its own settings, and the tools off until they load.
            self.automation = AutomationPanel {
                character,
                ..AutomationPanel::default()
            };
            player
                .client
                .borrow_mut()
                .set_automation_settings(Settings::default());
            if let Some(id) = character {
                let (send, receive) = channel();
                let spawned = std::thread::Builder::new()
                    .name("ac2 automation read".into())
                    .spawn(move || {
                        let bytes = std::fs::read(settings_path(id)).ok();
                        let _ = send.send(bytes);
                    });
                if spawned.is_ok() {
                    self.automation.loading = Some(receive);
                }
            }
        }
        let Some(id) = character else {
            return;
        };
        if let Some(loading) = &self.automation.loading {
            if let Ok(bytes) = loading.try_recv() {
                self.automation.loading = None;
                let settings = bytes.map(|b| Settings::decode(&b)).unwrap_or_default();
                player
                    .client
                    .borrow_mut()
                    .set_automation_settings(settings.clone());
                self.automation.saved = Some(settings);
            }
        }
        let Some(saved) = self.automation.saved.clone() else {
            return;
        };
        let mut settings = player.client.borrow().automation_settings();
        let mut pressed = false;
        for change in &changes {
            pressed |= settings.apply(change);
        }
        if pressed {
            player
                .client
                .borrow_mut()
                .set_automation_settings(settings.clone());
        }
        if settings != saved {
            save(id, settings.encode());
            self.automation.saved = Some(settings.clone());
        }
        let lines = player.client.borrow_mut().take_automation_lines();
        for line in lines {
            self.chat_display(ac2_ui::TEXT_TYPE_STANDARD, &line);
        }
        let open = self
            .panels
            .automation
            .as_ref()
            .is_some_and(|ui| ui.is_shown());
        if !open {
            self.automation.shown = None;
            return;
        }
        if !pressed
            && self.automation.shown.is_some()
            && frame.server_time - self.automation.refreshed < REFRESH_SECONDS
        {
            return;
        }
        self.automation.refreshed = frame.server_time;
        let presentation = presentation(player, frame, settings);
        if self.automation.shown.as_ref() != Some(&presentation) {
            if let (Some(ui), Some((tree, _))) =
                (self.panels.automation.as_mut(), self.hud.as_mut())
            {
                ui.present(tree, &presentation);
                self.dirty = true;
            }
            self.automation.shown = Some(presentation);
        }
    }
}

/// The window's content: the settings, the strip's effect names and the bar's skills by name.
fn presentation(
    player: &ClientPlayer<'_>,
    frame: &ClientFrame,
    settings: Settings,
) -> AutomationPresentation {
    let effects: Vec<String> = player
        .client
        .borrow_mut()
        .effect_list()
        .unwrap_or_default()
        .into_iter()
        .map(|n| n.name)
        .filter(|n| !n.is_empty())
        .collect();
    let mut skills: Vec<(u32, String)> = Vec::new();
    for (index, slot) in frame.progress.shortcuts.slots.iter().enumerate() {
        let ShortcutInfo::Enum { kind, value } = slot else {
            continue;
        };
        if *kind != SKILL_SHORTCUT || skills.iter().any(|(s, _)| s == value) {
            continue;
        }
        let shown = player.client.borrow_mut().shortcut_info(index as u32).ok();
        let name = shown
            .filter(|d| d.ok)
            .map(|d| player.render_info(&d.name))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| format!("skill {value:#x}"));
        skills.push((*value, name));
    }
    let mut effects_unique = Vec::new();
    for effect in effects {
        if !effects_unique.contains(&effect) {
            effects_unique.push(effect);
        }
    }
    AutomationPresentation {
        settings,
        effects: effects_unique,
        skills,
    }
}
