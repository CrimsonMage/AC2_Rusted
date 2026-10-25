//! `gmGenerationConfirmationUI`: avatar modification's naming dialog (`GenerationConfirmationField`
//! over layout `0x4100000C`). The player names the character in `NameEntry` (cleared the first
//! time it is activated; letters, `'`, `-` and space only, `NameInputFilter`) and creates it
//! (`CreateButton`) or creates and enters with it (`EnterButton`). The result is checked for a
//! name, a race (Legions: one needing the expansion or drudges needs the account's
//! entitlement) and a sex, named, and sent (`Proto_UI::SendCharGenResult`); the server's answer
//! adds the character and returns to character selection with it selected, or enters the world
//! with it, or shows why it was refused (`RecvNotice_CharGenVerification`).

use ac2_net::protocol::CharacterIdentity;

use crate::mouse::UiMessage;
use crate::pregame::chargen::{set_admin, CharGenSystem};
use crate::pregame::flow::{MODE_CHARACTER_SELECTION, MODE_GAMEPLAY};
use crate::pregame::strings::{int_var, text_var};
use crate::pregame::{Ctx, PregameRequest, LAYOUT_GENERATION_CONFIRMATION};
use crate::token::{tokenize, BUTTON_GHOSTED, BUTTON_HIGHLIGHTED, MSG_BUTTON_RELEASE_ON};

/// `UIElement_Text`'s `activated` message.
const MSG_ACTIVATED: u32 = tokenize("activated");
/// `token_clienttype` on the Legions client (`nonadminclient`; the admin build uses
/// `adminclient`, which shows `AdminButton`).
const CLIENT_TYPE: u32 = tokenize("nonadminclient");
/// Races (`m_race` bits) that need the Legions expansion / the drudge entitlement.
const RACE_NEEDS_LEGIONS: u32 = 0x10;
const RACE_NEEDS_DRUDGES: u32 = 0x8;
/// The `EnumMapper` naming the weenie's character generation errors.
const WEENIE_ERRORS: u32 = 0x4000_0035;

/// `gmCharNameCharacterValidator::IsCharacterValid` for English: ASCII letters, `'`, `-` and
/// space (the other languages' extra letters are not modelled).
pub fn is_name_character(ch: u16) -> bool {
    (ch < 0x80 && (ch as u8).is_ascii_alphabetic()) || matches!(ch, 0x27 | 0x2D | 0x20)
}

/// `gmGenerationConfirmationUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct ConfirmationUi {
    field: Option<usize>,
    name_entry: Option<usize>,
    /// The first activation clears the entry (then the message is no longer listened to).
    clear_on_activate: bool,
    /// `_enterGame`.
    enter_game: bool,
}

impl ConfirmationUi {
    /// `gmGenerationConfirmationUI::gmGenerationConfirmationUI`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let mut ui = Self::default();
        let Some(layout) = ctx.assets.layout(LAYOUT_GENERATION_CONFIRMATION) else {
            return ui;
        };
        ui.field = ctx
            .tree
            .create_root(layout.clone(), tokenize("GenerationConfirmationField"));
        let Some(field) = ui.field else {
            return ui;
        };
        if let Some(buttons) = ctx.tree.child_recursive(field, tokenize("ButtonField")) {
            ctx.tree.set_state(buttons, CLIENT_TYPE);
            ui.clear_on_activate = true;
            ui.name_entry = ctx.tree.child_recursive(field, tokenize("NameEntry"));
        }
        ui
    }

    /// The dialog's destructor: its root removed.
    pub(crate) fn destroy(self, ctx: &mut Ctx<'_>) {
        if let Some(field) = self.field {
            ctx.tree.remove_root(field);
        }
    }

    /// The name entry, for the input filter.
    pub(crate) fn name_entry(&self) -> Option<usize> {
        self.name_entry
    }

    fn child(&self, ctx: &Ctx<'_>, name: &str) -> Option<usize> {
        ctx.tree.child_recursive(self.field?, tokenize(name))
    }

    /// `gmGenerationConfirmationUI::GhostButtons`.
    fn ghost_buttons(&self, ctx: &mut Ctx<'_>, ghost: bool) {
        for name in ["CreateButton", "EnterButton", "CancelButton", "AdminButton"] {
            if let Some(button) = self.child(ctx, name) {
                ctx.tree.set_bool(button, BUTTON_GHOSTED, ghost);
            }
        }
    }

    /// `gmGenerationConfirmationUI::SetErrorMsg`.
    fn set_error_msg(&self, ctx: &mut Ctx<'_>, text: &str) {
        if let Some(error) = self.child(ctx, "GenerationConfirmationErrorMsg") {
            ctx.tree.set_text(error, text);
        }
    }

    /// `gmGenerationConfirmationUI::ListenToMessage`.
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        if self.field.is_none() || ctx.data.current().is_none() {
            return;
        }
        let element = message.element_id;
        if message.message == MSG_ACTIVATED && element == tokenize("NameEntry") {
            if self.clear_on_activate {
                self.clear_on_activate = false;
                if let Some(entry) = self.name_entry {
                    ctx.tree.set_text(entry, "");
                }
            }
            return;
        }
        if message.message != MSG_BUTTON_RELEASE_ON {
            return;
        }
        if element == tokenize("AdminButton") {
            let admin = message.element.is_some_and(|b| {
                ctx.tree
                    .get(b)
                    .and_then(|e| e.attribute(BUTTON_HIGHLIGHTED))
                    == Some(&crate::layout::AttributeValue::Bool(true))
            });
            let system: &mut dyn CharGenSystem = ctx.system;
            if let Some(cgr) = ctx.data.current_mut() {
                set_admin(system, cgr, admin);
            }
            return;
        }
        let enter = element == tokenize("EnterButton");
        if !enter && element != tokenize("CreateButton") {
            return;
        }
        let name = self
            .name_entry
            .and_then(|e| ctx.tree.get(e))
            .and_then(|e| e.text())
            .unwrap_or_default();
        let Some(cgr) = ctx.data.current().cloned() else {
            return;
        };
        let entitled = |i: usize| {
            ctx.data
                .characters()
                .and_then(|s| s.legions)
                .is_none_or(|w| w[i] != 0)
        };
        let error = if self.name_entry.is_none() || name.is_empty() {
            "ID_Character_Err_NoName"
        } else if cgr.race == 0 {
            "ID_Character_Err_NoRace"
        } else if cgr.race & RACE_NEEDS_LEGIONS != 0 && !entitled(0) {
            "ID_Character_Err_DoesNotHaveLegionsExpansion"
        } else if cgr.race & RACE_NEEDS_DRUDGES != 0 && !entitled(1) {
            "ID_Character_Err_CanNotMakeDrudges"
        } else if cgr.sex == 0 {
            "ID_Character_Err_NoGender"
        } else {
            if let Some(current) = ctx.data.current_mut() {
                current.name = name;
                let named = current.clone();
                self.ghost_buttons(ctx, true);
                self.enter_game = enter;
                ctx.requests.push(PregameRequest::CreateCharacter(named));
            }
            return;
        };
        let text = ctx.strings.get(error);
        self.set_error_msg(ctx, &text);
    }

    /// `gmGenerationConfirmationUI::RecvNotice_CharGenVerification`.
    pub(crate) fn char_gen_verification(
        &mut self,
        ctx: &mut Ctx<'_>,
        response: u32,
        identity: Option<&CharacterIdentity>,
        weenie_result: u32,
    ) {
        if ctx.data.characters().is_none() {
            return;
        }
        let error = match response {
            1 => {
                let Some(identity) = identity else {
                    return;
                };
                let slot = ctx.data.add_identity(identity.clone());
                if self.enter_game {
                    ctx.requests.push(PregameRequest::EnterWorld(identity.id));
                    ctx.flow.queue_mode(MODE_GAMEPLAY, "");
                } else {
                    ctx.data.set_gen_slot(slot.unwrap_or(0));
                    ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
                }
                return;
            }
            2 => "ID_Character_Err_Pending",
            3 => "ID_Character_Err_NameInUse",
            4 => "ID_Character_Err_NameBanned",
            5 => match weenie_result {
                0 => "ID_Character_Err_Corrupt",
                0xC749 => "ID_Character_Err_Gun_Specific_Error",
                0xC74A => "ID_Character_Err_NameInUse",
                0xC74B | 200 => "ID_Character_Err_NameBanned",
                _ => "ID_Character_Err_Weenie_Specific_Error",
            },
            6 => "ID_Character_Err_DatabaseDown",
            7 => "ID_Character_Err_AdminPriv",
            8 => "ID_Character_Err_NoMoreSlots",
            _ => "ID_Character_Err_Unknown",
        };
        // `EnumMapper::InqString(0x40000035, weenieResult)`: the weenie's error name.
        let weenie_error = if response == 5 {
            ctx.system
                .enum_string(WEENIE_ERRORS, weenie_result)
                .unwrap_or_default()
        } else {
            String::new()
        };
        let text = ctx.strings.render(
            tokenize(error),
            vec![
                int_var("response", response as i32),
                text_var("weenieError", &weenie_error),
            ],
        );
        self.set_error_msg(ctx, &text);
        self.ghost_buttons(ctx, false);
    }
}
