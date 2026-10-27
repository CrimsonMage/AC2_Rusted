//! The pregame UI (`ac2_ui::pregame::PregameUi`) joined to the session: the login events become
//! the UI's notices (`gmDataPatchUI::OnNetStatusChange(Net_LoginConnected)`,
//! `UIPersistantData::RecvNotice_CharacterSet`, `gmUIFlow::
//! RecvNotice_CharacterError` / `RecvNotice_ServerDied`, `RecvNotice_CharGenVerification`), and
//! the UI's requests go out through the session (`Proto_UI::SendEnterWorld`,
//! `SendDeleteCharacter`, `SendCharGenResult`). Run the session with
//! `CharacterChoice::Wait` so it stops at the character set.

use ac2_net::protocol::ClientEvent;
use ac2_ui::pregame::{PregameRequest, PregameUi};
use ac2_weenie::{OnlineSession, SessionEvent};

use crate::chargen_data::CharGenCapture;
use crate::client::Notice;

/// What carrying out the UI's requests asks of the program.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PregameOutcome {
    /// `Device::Done`: close the client.
    pub quit: bool,
    /// The screens' music, in the order the screens opened it (`Client::play_pregame_music`).
    pub music: Vec<u32>,
    /// The credits' sounds, in order (`Client::play_credit_sound`).
    pub credit_sounds: Vec<u32>,
    /// The input manager's maps the controls screen saved or restored (`Controls::replace_key_maps`).
    pub key_maps: Option<ac2_ui::pregame::KeyMapSet>,
}

/// One session event into the UI.
pub fn pregame_event(ui: &mut PregameUi, event: &SessionEvent) {
    match event {
        SessionEvent::Connected => ui.connected(),
        SessionEvent::Characters(set) => ui.character_set(set.clone()),
        SessionEvent::CharacterError(error) => ui.character_error(*error, None),
        SessionEvent::CharGenVerification {
            response,
            identity,
            result,
        } => ui.char_gen_verification(*response, Some(identity), *result),
        SessionEvent::Disconnected(_) => ui.server_died(),
        _ => {}
    }
}

/// The client's notices into the UI (`ConnectionLost` is `RecvNotice_ServerDied`).
pub fn pregame_notice(ui: &mut PregameUi, notice: Notice) {
    if notice == Notice::ConnectionLost {
        ui.server_died();
    }
}

/// The UI's requests carried out on the session, without a creation capture.
pub fn pregame_requests(ui: &mut PregameUi, session: &mut OnlineSession) -> PregameOutcome {
    pregame_requests_capturing(ui, session, &mut CharGenCapture::default())
}

/// The UI's requests carried out on the session. A refused entry is reported as the error
/// retail's server would send (`CHAR_ERROR_ENTER_GAME_GENERIC`). A created character is also
/// written to `capture` (`CharGenData::Save`, when the program named a capture file).
pub fn pregame_requests_capturing(
    ui: &mut PregameUi,
    session: &mut OnlineSession,
    capture: &mut CharGenCapture,
) -> PregameOutcome {
    let mut outcome = PregameOutcome::default();
    for request in ui.take_requests() {
        match request {
            PregameRequest::EnterWorld(character) => {
                if let Err(error) = session.enter_world(character) {
                    eprintln!("ac2: enter world: {error}");
                    ui.character_error(ac2_ui::pregame::char_error::ENTER_GAME_GENERIC, None);
                }
            }
            PregameRequest::DeleteCharacter(character) => session.delete_character(character),
            PregameRequest::CreateCharacter(result) => {
                if capture.file().is_some() {
                    let packet = ClientEvent::CharGenResult {
                        account: session.account().to_owned(),
                        result: result.clone(),
                    }
                    .encode()
                    .1;
                    capture.save(&packet, session.account(), &result.name);
                }
                session.create_character(result);
            }
            PregameRequest::RestoreCharacter { character, .. } => {
                eprintln!("ac2: restoring {character:#x} is an admin request; not sent");
            }
            PregameRequest::Music(did) => outcome.music.push(did),
            PregameRequest::CreditSound(did) => outcome.credit_sounds.push(did),
            PregameRequest::Quit => outcome.quit = true,
            PregameRequest::KeyMaps(set) => outcome.key_maps = Some(set),
        }
    }
    outcome
}
