//! `UIFlow` / `gmUIFlow`: which main framework the UI shows (its mode) and the switch to the next
//! one. A framework queues the next mode with a message (`QueueUIMode`); on the next UI loop the
//! flow destroys the current framework, makes the new one and, when the message is not empty,
//! hands it to the new framework (`SetErrorMsg`) (`UseNewMode`). `gmUIFlow` registers the modes
//! (intro, disconnected, data patch, keyboard, credits, character selection, avatar selection
//! and modification, gameplay) and turns the login errors and a lost server into a mode and a
//! message (`RecvNotice_CharacterError`, `RecvNotice_ServerDied`).

use crate::pregame::strings::UiStrings;
use crate::token::tokenize;

/// The UI modes (`gmUIFlow::gmUIFlow`'s `Register` calls; `Tokenize` of their names).
pub const MODE_INTRO: u32 = tokenize("intro");
pub const MODE_DISCONNECTED: u32 = tokenize("disconnected");
pub const MODE_DATA_PATCH: u32 = tokenize("datapatch");
pub const MODE_KEYBOARD: u32 = tokenize("keyboard");
pub const MODE_CREDITS: u32 = tokenize("credits");
pub const MODE_CHARACTER_SELECTION: u32 = tokenize("characterselection");
pub const MODE_AVATAR_SELECTION: u32 = tokenize("avatarselection");
pub const MODE_AVATAR_MODIFICATION: u32 = tokenize("avatarmodification");
pub const MODE_GAMEPLAY: u32 = tokenize("play");

/// `charError` values (`ECM_Login::RecvEvt_CharacterError`).
pub mod char_error {
    pub const ACCOUNT_LOGON: u32 = 2;
    pub const SERVER_CRASH: u32 = 3;
    pub const LOGOFF: u32 = 4;
    pub const DELETE: u32 = 5;
    pub const ACCOUNT_IN_USE: u32 = 6;
    pub const ACCOUNT_INVALID: u32 = 7;
    pub const ACCOUNT_DOESNT_EXIST: u32 = 8;
    pub const ENTER_GAME_GENERIC: u32 = 9;
    pub const ENTER_GAME_STRESS_ACCOUNT: u32 = 10;
    pub const ENTER_GAME_CHARACTER_IN_WORLD: u32 = 11;
    pub const ENTER_GAME_PLAYER_ACCOUNT_MISSING: u32 = 12;
    pub const ENTER_GAME_CHARACTER_NOT_OWNED: u32 = 13;
    pub const ENTER_GAME_CHARACTER_IN_WORLD_SERVER: u32 = 14;
    pub const ENTER_GAME_OLD_CHARACTER: u32 = 15;
    pub const ENTER_GAME_CORRUPT_CHARACTER: u32 = 16;
    pub const ENTER_GAME_START_SERVER_DOWN: u32 = 17;
    pub const ENTER_GAME_COULDNT_PLACE_CHARACTER: u32 = 18;
    pub const LOGON_SERVER_FULL: u32 = 19;
    pub const CHARACTER_IS_BOOTED: u32 = 20;
    pub const ENTER_GAME_CHARACTER_LOCKED: u32 = 21;
}

/// `gmUIFlow::RecvNotice_CharacterError`'s switch: the mode an error sends the UI to and the
/// UI string it shows. Account and server errors disconnect; the rest return to character
/// selection. Other values (`UNDEF`, `LOGGED_ON`) do nothing.
pub fn character_error_mode(error: u32) -> Option<(u32, &'static str)> {
    use char_error::*;
    let disconnected = |id| Some((MODE_DISCONNECTED, id));
    let selection = |id| Some((MODE_CHARACTER_SELECTION, id));
    match error {
        ACCOUNT_LOGON => disconnected("ID_CHAR_ERROR_ACCOUNT_LOGON"),
        SERVER_CRASH => disconnected("ID_CHAR_ERROR_SERVER_CRASH"),
        LOGOFF => disconnected("ID_CHAR_ERROR_LOGOFF"),
        DELETE => selection("ID_CHAR_ERROR_DELETE"),
        ACCOUNT_IN_USE => disconnected("ID_CHAR_ERROR_ACCOUNT_IN_USE"),
        ACCOUNT_INVALID => disconnected("ID_CHAR_ERROR_ACCOUNT_INVALID"),
        ACCOUNT_DOESNT_EXIST => disconnected("ID_CHAR_ERROR_ACCOUNT_DOESNT_EXIST"),
        ENTER_GAME_GENERIC => selection("ID_CHAR_ERROR_ENTER_GAME_GENERIC"),
        ENTER_GAME_STRESS_ACCOUNT => selection("ID_CHAR_ERROR_ENTER_GAME_STRESS_ACCOUNT"),
        ENTER_GAME_CHARACTER_IN_WORLD => selection("ID_CHAR_ERROR_ENTER_GAME_CHARACTER_IN_WORLD"),
        ENTER_GAME_PLAYER_ACCOUNT_MISSING => {
            selection("ID_CHAR_ERROR_ENTER_GAME_PLAYER_ACCOUNT_MISSING")
        }
        ENTER_GAME_CHARACTER_NOT_OWNED => selection("ID_CHAR_ERROR_ENTER_GAME_CHARACTER_NOT_OWNED"),
        ENTER_GAME_CHARACTER_IN_WORLD_SERVER => {
            selection("ID_CHAR_ERROR_ENTER_GAME_CHARACTER_IN_WORLD_SERVER")
        }
        ENTER_GAME_OLD_CHARACTER => selection("ID_CHAR_ERROR_ENTER_GAME_OLD_CHARACTER"),
        ENTER_GAME_CORRUPT_CHARACTER => selection("ID_CHAR_ERROR_ENTER_GAME_CORRUPT_CHARACTER"),
        ENTER_GAME_START_SERVER_DOWN => selection("ID_CHAR_ERROR_ENTER_GAME_START_SERVER_DOWN"),
        ENTER_GAME_COULDNT_PLACE_CHARACTER => {
            selection("ID_CHAR_ERROR_ENTER_GAME_COULDNT_PLACE_CHARACTER")
        }
        LOGON_SERVER_FULL => disconnected("ID_CHAR_ERROR_LOGON_SERVER_FULL"),
        CHARACTER_IS_BOOTED => disconnected("ID_CHAR_ERROR_CHARACTER_IS_BOOTED"),
        ENTER_GAME_CHARACTER_LOCKED => selection("ID_CHAR_ERROR_ENTER_GAME_LOCKED"),
        _ => None,
    }
}

/// `UIFlow`'s modes: the current one and the one queued for the next UI loop.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiFlow {
    /// `_curMode` (0 before the first switch).
    mode: u32,
    /// `_nextMode` and `_nextText`.
    next: Option<(u32, String)>,
}

impl UiFlow {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current mode (`_curMode`).
    pub fn mode(&self) -> u32 {
        self.mode
    }

    /// The queued mode, if any (`_nextMode`).
    pub fn next_mode(&self) -> Option<u32> {
        self.next.as_ref().map(|(mode, _)| *mode)
    }

    /// `UIFlow::QueueUIMode`: the mode the next UI loop switches to, and the message the new
    /// framework shows; a later call replaces it.
    pub fn queue_mode(&mut self, mode: u32, text: impl Into<String>) {
        self.next = Some((mode, text.into()));
    }

    /// `UIFlow::UseNewMode`'s switch: the queued mode becomes current. The caller destroys the
    /// old framework, makes the new one and, when the message is not empty, calls its
    /// `SetErrorMsg`.
    pub fn use_new_mode(&mut self) -> Option<(u32, String)> {
        let (mode, text) = self.next.take()?;
        self.mode = mode;
        Some((mode, text))
    }

    /// `gmUIFlow::RecvNotice_CharacterError`: the error's mode, with the server's text when it
    /// sent one, else the error's UI string.
    pub fn character_error(&mut self, strings: &UiStrings, error: u32, text: Option<&str>) {
        let Some((mode, id)) = character_error_mode(error) else {
            return;
        };
        let text = match text {
            Some(text) => text.to_owned(),
            None => strings.get(id),
        };
        self.queue_mode(mode, text);
    }

    /// `gmUIFlow::RecvNotice_ServerDied`: unless already (or about to be) disconnected, the
    /// disconnected mode with `ID_NetErr_ConnectionLost`.
    pub fn server_died(&mut self, strings: &UiStrings) {
        if self.mode == MODE_DISCONNECTED || self.next_mode() == Some(MODE_DISCONNECTED) {
            return;
        }
        self.queue_mode(MODE_DISCONNECTED, strings.get("ID_NetErr_ConnectionLost"));
    }
}
