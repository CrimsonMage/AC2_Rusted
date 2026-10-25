//! Character selection: the character set the server sends at logon (and again when it
//! changes), and entering the world with one of its characters (`gmCharacterSelectionUI` ->
//! `Proto_UI::SendEnterWorld`), chosen up front or on request.

use std::time::Duration;

use ac2_net::protocol::{CharacterSet, ClientEvent};

use super::{CharacterChoice, OnlineSession, SessionEvent, FRAME};
use crate::world::WorldState;

impl OnlineSession {
    /// The latest character set the server sent (`ECM_Login::RecvEvt_CharacterSet`).
    pub fn characters(&self) -> Option<&CharacterSet> {
        self.characters.as_ref()
    }

    /// `gmCharacterSelectionUI` entering the world with `character` (`Proto_UI::SendEnterWorld`):
    /// refused while in (or entering) the world, or for a character not in the latest set.
    pub fn enter_world(&mut self, character: u64) -> Result<(), String> {
        if self.world.state() != WorldState::NotInWorld {
            return Err("already in the world".into());
        }
        let listed = self
            .characters
            .as_ref()
            .is_some_and(|set| set.characters.iter().any(|c| c.id == character));
        if !listed {
            return Err(format!(
                "character {character:#018x} is not in the character set"
            ));
        }
        self.world.start_entering_world();
        self.client.net_mut().enter_world();
        let account = self.account.clone();
        self.send(ClientEvent::EnterWorld { character, account });
        self.pending.push(SessionEvent::Entering(character));
        Ok(())
    }

    /// Runs frames until the server sent a character set, or fails after `timeout`. The other
    /// events it sees come back from the next [`OnlineSession::poll`].
    pub fn wait_for_characters(&mut self, timeout: Duration) -> Result<CharacterSet, String> {
        let deadline = self.client.elapsed() + timeout.as_secs_f64();
        let mut seen = Vec::new();
        while self.client.elapsed() < deadline {
            let mut found = None;
            for event in self.poll() {
                match event {
                    SessionEvent::Characters(set) if found.is_none() => found = Some(set),
                    SessionEvent::Disconnected(reason) => return Err(reason),
                    other => seen.push(other),
                }
            }
            if let Some(set) = found {
                self.pending.extend(seen);
                return Ok(set);
            }
            self.client.sleep(FRAME);
        }
        Err("timed out waiting for the character set".into())
    }

    /// A character set arrived: kept, reported, and, unless the session waits for
    /// [`OnlineSession::enter_world`], the configured character enters the world.
    pub(super) fn choose_character(&mut self, set: CharacterSet, out: &mut Vec<SessionEvent>) {
        let chosen = set
            .characters
            .iter()
            .find(|c| match &self.choice {
                CharacterChoice::First => true,
                CharacterChoice::Name(name) => c.name.eq_ignore_ascii_case(name),
                CharacterChoice::Id(id) => c.id == *id,
                CharacterChoice::Wait => false,
            })
            .map(|c| c.id);
        self.characters = Some(set.clone());
        out.push(SessionEvent::Characters(set));
        if self.world.state() != WorldState::NotInWorld || self.choice == CharacterChoice::Wait {
            return;
        }
        match chosen {
            Some(character) => {
                self.world.start_entering_world();
                self.client.net_mut().enter_world();
                let account = self.account.clone();
                self.send(ClientEvent::EnterWorld { character, account });
                out.push(SessionEvent::Entering(character));
            }
            None => {
                self.closed = true;
                out.push(SessionEvent::Rejected(format!(
                    "no character matching {:?} on this account",
                    self.choice
                )));
            }
        }
    }
}
