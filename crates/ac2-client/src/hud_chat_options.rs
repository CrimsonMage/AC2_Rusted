//! The chat options page's wiring (`gmChatOptionsUI`): the player's chat settings as the page
//! reads them ([`ui_chat_settings`]), and the page's requests as the gameplay commands they are
//! ([`chat_option_commands`]). An Apply sends the reload notice, so the chat windows take the new
//! settings on the next frame ([`ac2_ui::ChatOptionsUi::take_reload`]).

use ac2_ui::{ChatOptionRequest, ChatOptionsUi};

use crate::gameplay::Command;

/// The player's chat settings (`gmCWInterface::GetGameplayOptions`'s chat part) as the UI takes
/// them: the window filters, the font sizes and colours, the popup flags and each window's text
/// type.
pub fn ui_chat_settings(settings: &crate::scripts_queries::ChatSettings) -> ac2_ui::ChatSettings {
    ac2_ui::ChatSettings {
        filters: settings.filters.clone(),
        font_sizes: settings.font_sizes.clone(),
        font_colors: settings.font_colors.clone(),
        popups: settings
            .popups
            .iter()
            .map(|(&kind, &popup)| (kind, popup != 0))
            .collect(),
        window_text_types: settings.window_text_types.clone(),
    }
}

/// The page's requests as gameplay commands (`Command::SetChatOption`, `UploadChatOptions`).
pub fn chat_option_commands(requests: Vec<ChatOptionRequest>) -> Vec<Command> {
    requests
        .into_iter()
        .map(|request| match request {
            ChatOptionRequest::Set(setting) => Command::SetChatOption(setting),
            ChatOptionRequest::Upload => Command::UploadChatOptions,
        })
        .collect()
}

/// The chat options page's requests since the last frame, as commands onto `commands`.
pub(crate) fn chat_options_commands(page: &mut ChatOptionsUi, commands: &mut Vec<Command>) {
    commands.extend(chat_option_commands(page.take_requests()));
}
