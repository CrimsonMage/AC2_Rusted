//! `gmDisconnectedUI`: the screen after the connection or the login failed. `DisconnectedField`
//! shows the reason in `ErrorText` (`SetErrorMsg`); `QuitButton` closes the client
//! (`Device::Done`).

use crate::mouse::UiMessage;
use crate::pregame::{Ctx, PregameRequest, LAYOUT_DISCONNECTED};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

/// `gmDisconnectedUI`'s state.
#[derive(Debug, Clone, Default)]
pub(crate) struct DisconnectedUi {
    field: Option<usize>,
    error_text: Option<usize>,
}

impl DisconnectedUi {
    /// `gmDisconnectedUI::gmDisconnectedUI`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let field = ctx.assets.layout(LAYOUT_DISCONNECTED).and_then(|layout| {
            ctx.tree
                .create_root(layout.clone(), tokenize("DisconnectedField"))
        });
        let error_text = field.and_then(|f| ctx.tree.child_recursive(f, tokenize("ErrorText")));
        Self { field, error_text }
    }

    /// The framework's destructor: its root removed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        if let Some(field) = self.field {
            ctx.tree.remove_root(field);
        }
    }

    /// `gmDisconnectedUI::SetErrorMsg`.
    pub(crate) fn set_error_msg(&self, ctx: &mut Ctx<'_>, text: &str) {
        if let Some(error) = self.error_text {
            ctx.tree.set_text(error, text);
        }
    }

    /// `gmDisconnectedUI::ListenToMessage`: quit (registered only when `ErrorText` exists).
    pub(crate) fn message(&self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if self.error_text.is_some()
            && message.message == MSG_BUTTON_RELEASE_ON
            && message.element_id == tokenize("QuitButton")
        {
            ctx.requests.push(PregameRequest::Quit);
        }
    }
}
