//! `gmDataPatchUI`: the connecting screen after the intro. `DataPatchField` shows the
//! connection's progress in `ErrorText` (`SetErrorMsg`): connecting (`Connect`,
//! `ID_DataPatch_ConnectingGUN`), then connected (`OnNetStatusChange(Net_LoginConnected)`,
//! `ID_DataPatch_Connected`). The character set ends it with character selection
//! (`Update`); `QuitButton` closes the client (`Device::Done`).
//!
//! The client's session connects on its own, so a set that arrived during the intro is taken
//! when the screen is made. Not modelled: the data patch itself (`OnDDDEvent`'s patch progress
//! in `PatchProgress` / `PatchText`, the link rate in `RateText`; the DATs never patch here),
//! the admin start or create character of `Update` (the client's `--character` enters without
//! the screens) and a failed connection's text (`OnConnectFailed`; the session reports it as a
//! lost server).

use crate::mouse::UiMessage;
use crate::pregame::flow::MODE_CHARACTER_SELECTION;
use crate::pregame::{Ctx, PregameRequest, LAYOUT_DATA_PATCH};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON};

/// `gmDataPatchUI`'s state.
#[derive(Debug, Clone, Default)]
pub(crate) struct DataPatchUi {
    field: Option<usize>,
    error_text: Option<usize>,
}

impl DataPatchUi {
    /// `gmDataPatchUI::gmDataPatchUI`: the field, then `Connect`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let field = ctx.assets.layout(LAYOUT_DATA_PATCH).and_then(|layout| {
            ctx.tree
                .create_root(layout.clone(), tokenize("DataPatchField"))
        });
        let error_text = field.and_then(|f| ctx.tree.child_recursive(f, tokenize("ErrorText")));
        let ui = Self { field, error_text };
        let text = ctx.strings.get("ID_DataPatch_ConnectingGUN");
        ui.set_error_msg(ctx, &text);
        if ctx.data.characters().is_some() {
            ui.update(ctx);
        }
        ui
    }

    /// The framework's destructor: its root removed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        if let Some(field) = self.field {
            ctx.tree.remove_root(field);
        }
    }

    /// `gmDataPatchUI::SetErrorMsg`.
    pub(crate) fn set_error_msg(&self, ctx: &mut Ctx<'_>, text: &str) {
        if let Some(error) = self.error_text {
            ctx.tree.set_text(error, text);
        }
    }

    /// `OnNetStatusChange(Net_LoginConnected)`.
    pub(crate) fn connected(&self, ctx: &mut Ctx<'_>) {
        let text = ctx.strings.get("ID_DataPatch_Connected");
        self.set_error_msg(ctx, &text);
    }

    /// `gmDataPatchUI::Update` (the character set arrived): character selection.
    pub(crate) fn update(&self, ctx: &mut Ctx<'_>) {
        ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
    }

    /// `gmDataPatchUI::ListenToMessage`: quit.
    pub(crate) fn message(&self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if message.message == MSG_BUTTON_RELEASE_ON && message.element_id == tokenize("QuitButton")
        {
            ctx.requests.push(PregameRequest::Quit);
        }
    }
}
