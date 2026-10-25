//! `gmIntroUI`: the first screen (`gmUIFlow::gmUIFlow` queues it). `IntroField` shows its
//! media, two pictures four seconds each, and then broadcasts `intro_stop`; that message, a key
//! press or a mouse button let go (`UIMESSAGE_GLOBAL_KEYPRESS`, `UIMESSAGE_GLOBAL_MOUSEUP`)
//! close it into the data patch (connecting) screen (`Close`).
//!
//! The field's `music` plays when it opens (`gmIntroUI::gmIntroUI`, `Channel_Main`, the previous
//! screen's music fading over two seconds: [`PregameRequest::Music`]).

use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS, GLOBAL_MOUSE_UP};
use crate::pregame::flow::MODE_DATA_PATCH;
use crate::pregame::{screen_music, Ctx, PregameRequest, LAYOUT_INTRO};
use crate::token::tokenize;

/// `gmIntroUI`'s state.
#[derive(Debug, Clone, Default)]
pub(crate) struct IntroUi {
    field: Option<usize>,
}

impl IntroUi {
    /// `gmIntroUI::gmIntroUI`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let field = ctx
            .assets
            .layout(LAYOUT_INTRO)
            .and_then(|layout| ctx.tree.create_root(layout.clone(), tokenize("IntroField")));
        if let Some(did) = screen_music(ctx.tree, field) {
            ctx.requests.push(PregameRequest::Music(did));
        }
        Self { field }
    }

    /// The framework's destructor: its root removed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        if let Some(field) = self.field {
            ctx.tree.remove_root(field);
        }
    }

    /// `gmIntroUI::Close`: on to the data patch screen.
    fn close(&self, ctx: &mut Ctx<'_>) {
        ctx.flow.queue_mode(MODE_DATA_PATCH, "");
    }

    /// Both `gmIntroUI::ListenToMessage`s: `IntroField`'s `intro_stop`, a key press or a mouse
    /// button let go.
    pub(crate) fn message(&self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        let stop = message.element_id == tokenize("IntroField")
            && message.message == tokenize("intro_stop");
        let global = message.element.is_none()
            && matches!(message.message, GLOBAL_KEY_PRESS | GLOBAL_MOUSE_UP);
        if stop || global {
            self.close(ctx);
        }
    }
}
