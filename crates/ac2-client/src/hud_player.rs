//! The HUD frameworks that ask the player, run once a frame with the client: the skills panel
//! (`gmSkillTreeUI`) and the shortcut bar (`gmShortcutUI`) take the frame's notices
//! (`RecvNotice_*`) and the tree's messages, ask the player's script and the session through
//! [`ClientPlayer`], and their requests become the player's calls and gameplay commands. The chat
//! takes the player's chat options when they are reloaded.

use std::cell::RefCell;

use ac2_net::protocol::ExaminationRequest;
use ac2_script::host::ScriptNotice;
use ac2_ui::{ChatSettings, ShortcutRequest, SkillRequest};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_view::HudView;
use crate::ui_sources::{shortcut_to_info, ClientPlayer};

/// The player's chat options as the chat windows take them.
fn chat_settings(client: &mut Client) -> Option<ChatSettings> {
    let settings = client.chat_settings().ok()?;
    Some(ChatSettings {
        filters: settings.filters,
        font_sizes: settings.font_sizes,
        font_colors: settings.font_colors,
        popups: settings
            .popups
            .into_iter()
            .map(|(kind, popup)| (kind, popup != 0))
            .collect(),
    })
}

/// `SetSelection(INVALID_IID)` (or the item), `SetExaminationRequest`.
fn examine(kind: u32, data: u32, item: u64, commands: &mut Vec<Command>) {
    let mut request = ExaminationRequest {
        kind,
        ..ExaminationRequest::default()
    };
    match kind {
        0x4000_0002 => request.iid = item,
        0x4000_0007 => request.data_id = data,
        _ => request.data_enum = data,
    }
    commands.push(Command::Select((item != 0).then_some(item)));
    commands.push(Command::Examine(request));
}

impl HudView {
    /// One frame of the frameworks that ask the player; returns the gameplay commands they ask
    /// for. They wait for the HUD, the string tables and the player in the world.
    pub(crate) fn frameworks(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        events: &[SessionEvent],
        script_notices: &[ScriptNotice],
    ) -> Vec<Command> {
        let messages = std::mem::take(&mut self.player_messages);
        let mut commands = Vec::new();
        let Some((strings, ui_table)) = self.strings.clone() else {
            return commands;
        };
        if !frame.in_world {
            return commands;
        }
        let HudView {
            hud,
            skills,
            shortcuts,
            chat,
            player_loaded,
            dirty,
            ..
        } = self;
        let Some((tree, _)) = hud else {
            return commands;
        };
        let reload_chat = !*player_loaded
            || script_notices.iter().any(|n| {
                matches!(
                    n.name,
                    "CM_Communication::SendNotice_ReloadChatSettings"
                        | "CM_Character::SendNotice_ReloadOptions"
                )
            });
        if reload_chat {
            if let (Some(chat), Some(settings)) = (chat.as_mut(), chat_settings(client)) {
                chat.reload_chat_settings(tree, &settings);
            }
        }
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings,
            ui_table,
        };
        let mut reload_shortcuts = !*player_loaded
            || script_notices
                .iter()
                .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
        let mut check_shortcuts = false;
        if !*player_loaded {
            *player_loaded = true;
            if let Some(skills) = skills.as_mut() {
                skills.update_skill_panel(tree, &player);
            }
        }
        for event in events {
            let SessionEvent::Notice(notice) = event else {
                continue;
            };
            match notice {
                GameplayNotice::ExaminationRequestChanged(request) => {
                    if let Some(skills) = skills.as_mut() {
                        skills.examination_request_changed(
                            tree,
                            &player,
                            request.kind,
                            request.data_enum,
                        );
                    }
                }
                GameplayNotice::UpdateSkillPanel { .. } => {
                    if let Some(skills) = skills.as_mut() {
                        skills.update_skill_panel(tree, &player);
                    }
                }
                GameplayNotice::UpdateSkill(skill) => {
                    if let Some(skills) = skills.as_mut() {
                        skills.update_skill(tree, &player, *skill);
                    }
                    check_shortcuts = true;
                }
                GameplayNotice::ReloadShortcuts | GameplayNotice::ReloadOptions => {
                    reload_shortcuts = true;
                }
                GameplayNotice::UpdateInventory { .. } | GameplayNotice::UpdateContainer { .. } => {
                    check_shortcuts = true;
                }
                _ => {}
            }
        }
        if let Some(bar) = shortcuts.as_mut() {
            if reload_shortcuts {
                bar.reload(tree, &player);
            } else if check_shortcuts {
                bar.check_all(tree, &player);
            }
        }
        for message in &messages {
            if let Some(skills) = skills.as_mut() {
                skills.message(tree, &player, message);
            }
            if let Some(bar) = shortcuts.as_mut() {
                bar.message(tree, &player, message);
            }
        }
        let mut open_examination = false;
        if let Some(skills) = skills.as_mut() {
            for request in skills.take_requests() {
                match request {
                    SkillRequest::Examine { kind, data } => {
                        examine(kind, data, 0, &mut commands);
                        open_examination = true;
                    }
                    // The confirmation dialogs are not modelled: confirmed at once.
                    SkillRequest::Train(skill) | SkillRequest::ConfirmTrain(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Train(skill)));
                    }
                    SkillRequest::Raise(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Raise(skill)));
                    }
                    SkillRequest::ConfirmUntrain(skill) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::Untrain(skill)));
                    }
                    SkillRequest::ConfirmCancelUntrain(_) => {
                        commands.push(Command::Skill(ac2_weenie::SkillRequest::CancelUntrain));
                    }
                }
            }
        }
        if let Some(bar) = shortcuts.as_mut() {
            for request in bar.take_requests() {
                let mut client = player.client.borrow_mut();
                let result = match request {
                    ShortcutRequest::Use(index) => client.use_shortcut(index as u32).map(drop),
                    ShortcutRequest::Set(index, shortcut) => client
                        .set_shortcut(index as u32, shortcut_to_info(&shortcut))
                        .map(drop),
                    ShortcutRequest::Examine { kind, data, item } => {
                        examine(kind, data, item, &mut commands);
                        open_examination = true;
                        Ok(())
                    }
                    ShortcutRequest::Attack(_) | ShortcutRequest::WhichSet(_) => Ok(()),
                };
                if let Err(error) = result {
                    eprintln!("ac2: shortcut: {error}");
                }
            }
        }
        if tree.take_changed() {
            *dirty = true;
        }
        if open_examination {
            self.open_examination();
        }
        commands
    }
}
