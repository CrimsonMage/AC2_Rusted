//! `gmBookUI` on the frame: the book's use notice (`CM_Usage::SendNotice_UseBook`, raised by the
//! player's script when a book is used) shows the book, and the panel's messages turn its pages.
//! The page label is rendered from the UI string table through the player.
//!
//! The help button (`KeyStone::OpenHelp(ID_Help_Book)`) opens the help window; the copy button
//! (`Device::SendStringToClipboard`) has no counterpart in the client yet and is dropped here.

use std::cell::RefCell;

use ac2_ui::BookRequest;
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::hud_view::HudView;
use crate::ui_sources::ClientPlayer;

impl HudView {
    /// The book's use notices, then its messages (the page turns, the close button). Runs with
    /// the other panels' frameworks, on the same messages.
    pub(crate) fn book_frameworks(
        &mut self,
        client: &mut Client,
        frame: &ClientFrame,
        events: &[SessionEvent],
    ) {
        let Some(strings) = self.strings.clone() else {
            return;
        };
        if !frame.in_world {
            return;
        }
        let messages = self.player_messages.clone();
        let Some(book) = self.book.as_mut() else {
            return;
        };
        let tree = match self.hud.as_mut() {
            Some((tree, _)) => tree,
            None => return,
        };
        let player = ClientPlayer {
            client: RefCell::new(client),
            frame,
            strings: &strings.tables,
            ui_table: strings.ui_table,
            enums: &strings.enums,
        };
        for event in events {
            if let SessionEvent::Notice(GameplayNotice::UseBook {
                show_controls,
                mode,
                title,
                text,
                image,
            }) = event
            {
                book.recv_use_book(tree, &player, *show_controls, *mode, title, text, *image);
            }
        }
        let mut help = Vec::new();
        for message in &messages {
            // The copy request has no client counterpart yet (see the module doc).
            if let Some(BookRequest::OpenHelp) = book.message(tree, &player, message) {
                help.push(ac2_ui::tokenize("ID_Help_Book"));
            }
        }
        for token in help {
            self.open_help(token);
        }
    }
}
