//! The options page's handles for the viewer's tests and the frames' messages: the tab opens, a
//! checkbox is ticked, a button pressed, a value read. The page is [`HudView::character`]; its
//! messages go through the tree's dialogs, and its requests wait for the next frameworks call.

use crate::client::Client;
use crate::gameplay::Command;
use crate::hud_view::HudView;

/// The options profile as the options page reads it (`gmCWInterface::Get*`).
pub(crate) fn options_settings(client: &mut Client) -> Option<ac2_ui::OptionsSettings> {
    let p = client.gameplay_options().ok()?;
    Some(ac2_ui::OptionsSettings {
        bits: p.bit_field,
        radar_mask: p.radar_mask,
        shortcut_height: p.shortcut_height,
        damage_text_range_other: p.damage_text_range_other,
    })
}

/// The options page's requests, as the gameplay commands they are.
pub(crate) fn character_commands(page: &mut ac2_ui::CharacterUi, commands: &mut Vec<Command>) {
    for request in page.take_requests() {
        commands.push(match request {
            ac2_ui::CharacterRequest::Set(setting) => Command::SetGameplayOption(setting),
            ac2_ui::CharacterRequest::Upload => Command::UploadGameOptions,
        });
    }
}

impl HudView {
    /// The options page's tab opens (its `opened` message, as the tree sends it).
    pub(crate) fn character_open(&mut self) {
        let (Some((tree, _)), Some(page), Some(dialogs)) = (
            self.hud.as_mut(),
            self.panels.character.as_mut(),
            self.panels.dialogs.as_mut(),
        ) else {
            return;
        };
        page.open_tab(tree, dialogs);
    }

    /// The player ticks or unticks an option's checkbox ([`ac2_ui::CharacterUi::tick`]).
    pub(crate) fn character_tick(&mut self, name: &str, on: bool) {
        let (Some((tree, _)), Some(page), Some(dialogs)) = (
            self.hud.as_mut(),
            self.panels.character.as_mut(),
            self.panels.dialogs.as_mut(),
        ) else {
            return;
        };
        page.tick(tree, dialogs, name, on);
    }

    /// The player presses one of the options page's buttons ([`ac2_ui::CharacterUi::press`]);
    /// the requests wait for the next frameworks call.
    pub(crate) fn character_press(&mut self, element_id: &str) {
        let (Some((tree, _)), Some(page), Some(dialogs)) = (
            self.hud.as_mut(),
            self.panels.character.as_mut(),
            self.panels.dialogs.as_mut(),
        ) else {
            return;
        };
        page.press(tree, dialogs, element_id);
    }

    /// The options page's value of option `name`, as the page shows it.
    pub(crate) fn character_value(&self, name: &str) -> Option<f64> {
        self.panels.character.as_ref()?.value(name)
    }
}
