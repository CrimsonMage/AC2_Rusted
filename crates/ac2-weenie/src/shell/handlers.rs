//! The `CPlayer` command handlers the port runs natively, over the chat state those scripts
//! keep (`CPlayer` slots 31, 48, 49, 50 and 54, and the player's fellowship), and the server
//! calls and GUN results that change it.

use ac2_net::protocol::{
    GameplayOptionsProfile, HeapObject, PackValue, PackedEntry, PackedReference, ScriptRequest,
};

use super::lines::*;
use super::{expand_aliases, handler, process_text, ChatCommands, Parsed};
use crate::notice::GameplayNotice;
use crate::tables::StringTableIds;

/// What a chat line does.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellAction {
    /// A script message to the server.
    Send(ScriptRequest),
    /// A line for the chat windows.
    Notice(GameplayNotice),
    /// `gmCWInterface::RequestGUN_SendToRoom`: `text` to chat room `room` with the script's
    /// chat flags; when it fails (no room, empty text) the script shows the chat-server line.
    SendToRoom { room: u32, text: String, flags: u32 },
    /// A registered command the port does not run natively: its `CPlayer` handler, the text
    /// after the command and the tokens (the command's first).
    Command {
        handler: u32,
        text: String,
        args: Vec<String>,
    },
}

/// The player the commands speak for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Speaker<'a> {
    /// `gmEntity` 0xAD's text: the player's name.
    pub name: &'a str,
    /// `Player` 0x5B (instance id quality 0x1C4, the monarch): non-zero in an allegiance.
    pub monarch: u64,
}

/// The chat state the player's scripts keep.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatState {
    commands: ChatCommands,
    /// `GameplayOptionsProfile::m_aliasTable`.
    aliases: Vec<(String, String)>,
    /// `GameplayOptionsProfile::m_filterHash`: chat window -> the channels it shows.
    filters: Vec<(u32, u32)>,
    /// `CPlayer` slot 48: the last /tell's recipient (`CPlayer` 0x3F, read by 0xC0).
    retell: Option<String>,
    /// `CPlayer` slot 49: the last teller with a literal name (`CPlayer` 0x3B, read by 0xB7).
    reply: Option<String>,
    /// `CPlayer` slot 50: the allegiance's chat room (`CPlayer` 0x134).
    allegiance_room: u32,
    /// `CPlayer` slot 31: channel -> chat room (`ChannelData` slot 3; `CPlayer` 0x147).
    channels: Vec<(u32, u32)>,
    /// `Player` slot 28: the player's fellowship, as its chat room (`Fellowship` slot 3).
    fellowship_room: Option<u32>,
    /// `CPlayer` slot 54: the squelch list the GUN service last returned.
    squelched: Vec<String>,
}

/// The `Fellowship` and `ChannelData` packages.
const FELLOWSHIP_PACKAGE: u32 = 0x1F0;
const CHANNEL_DATA_PACKAGE: u32 = 0x2023;

fn heap(graph: &PackedReference, id: u32) -> Option<&HeapObject> {
    let at = graph.ids.iter().position(|i| *i == id)?;
    match graph.entries.get(at)? {
        PackedEntry::Heap { object, .. } => Some(object),
        _ => None,
    }
}

/// The root package instance of a server call's reference argument.
fn root(value: Option<&PackValue>, package: u32) -> Option<(&PackedReference, &HeapObject)> {
    let PackValue::Reference(graph) = value? else {
        return None;
    };
    let object = heap(graph, *graph.ids.first()?)?;
    (object.pkg_id == package).then_some((graph, object))
}

impl ChatState {
    pub fn commands(&self) -> &ChatCommands {
        &self.commands
    }

    pub fn set_commands(&mut self, commands: ChatCommands) {
        self.commands = commands;
    }

    /// The options profile `CPlayer` 0x65 receives: its aliases and chat filters.
    pub fn login_options(&mut self, profile: &GameplayOptionsProfile) {
        self.aliases = profile.aliases.1.clone();
        self.filters = profile.filters.pairs.clone();
    }

    /// `CWInterface::ProcessText` for a line the chat window entered, with
    /// `ChatWindow::ProcessCommand`'s line for an unknown command.
    pub fn process_text(
        &mut self,
        line: &str,
        speaker: Speaker<'_>,
        tables: &StringTableIds,
    ) -> Vec<ShellAction> {
        let line = expand_aliases(line, &self.aliases, &self.commands);
        match process_text(&line, &self.commands) {
            Parsed::Empty | Parsed::Failure => Vec::new(),
            Parsed::Unknown => vec![show_line(
                tables,
                TABLE_CHAT_WINDOW,
                UNKNOWN_COMMAND,
                KIND_ERROR,
            )],
            Parsed::Command {
                handler,
                text,
                args,
            } => self.run(handler, text, args, speaker, tables),
        }
    }

    fn help(&self, handler: u32) -> ShellAction {
        show(KIND_INFO, literal(&self.commands.help(handler)))
    }

    fn run(
        &mut self,
        id: u32,
        text: String,
        mut args: Vec<String>,
        speaker: Speaker<'_>,
        tables: &StringTableIds,
    ) -> Vec<ShellAction> {
        let n = args.len();
        let mut out = Vec::new();
        match id {
            // `CPlayer` 0xDB.
            handler::SAY if n <= 1 => out.push(self.help(id)),
            handler::SAY => out.push(ShellAction::Send(ScriptRequest::Say {
                mode: 0,
                text: literal(&text),
            })),
            handler::TELL => self.tell(&args[1..], tables, &mut out),
            // `CPlayer` 0x112 and 0xC5: the remembered name; nothing before the first.
            handler::RETELL | handler::REPLY if n < 2 => out.push(self.help(id)),
            handler::RETELL | handler::REPLY => {
                let (target, missing) = match id {
                    handler::RETELL => (&self.retell, NO_RETELL),
                    _ => (&self.reply, NO_REPLY),
                };
                match target.as_deref() {
                    None => {}
                    Some("") => out.push(show_line(
                        tables,
                        TABLE_COMMUNICATION,
                        missing,
                        KIND_TELL_ERROR,
                    )),
                    Some(name) => tell_to(name, &text, id == handler::RETELL, &mut out),
                }
            }
            // `CPlayer` 0x5E (the mount check, `Player` 0x7A, is not mirrored).
            handler::EMOTE if n <= 1 => out.push(self.help(id)),
            handler::EMOTE => out.push(ShellAction::Send(ScriptRequest::EmoteText(text))),
            // `CPlayer` 0xCF: the command is the emote's name. The emote 0x32B4A467 sets the
            // GUN location instead, which is not mirrored.
            handler::EMOTE_BY_NAME => {
                if let Some(emote) = self.commands.emote(&args[0]) {
                    if emote != 0x32B4_A467 {
                        out.push(ShellAction::Send(ScriptRequest::Emote(emote)));
                    }
                }
            }
            handler::SQUELCH => {
                args.remove(0);
                self.squelch(n, args, speaker, tables, &mut out);
            }
            handler::FELLOWSHIP | handler::ALLEGIANCE | handler::CK if n <= 1 => {
                out.push(self.help(id))
            }
            // `CPlayer` 0x103.
            handler::FELLOWSHIP => match self.fellowship_room {
                None => out.push(show_line(
                    tables,
                    TABLE_FELLOWSHIP,
                    NO_FELLOWSHIP,
                    KIND_ERROR,
                )),
                Some(0) => out.push(show_line(
                    tables,
                    TABLE_CHANNELS,
                    FELLOWSHIP_UNAVAILABLE,
                    KIND_ERROR,
                )),
                Some(room) => out.push(room_message(room, text, 1)),
            },
            // `CPlayer` 0xE4.
            handler::ALLEGIANCE if speaker.monarch == 0 => out.push(show_line(
                tables,
                TABLE_ALLEGIANCE,
                NO_ALLEGIANCE,
                KIND_ERROR,
            )),
            handler::ALLEGIANCE if self.allegiance_room == 0 => out.push(show_line(
                tables,
                TABLE_CHANNELS,
                ALLEGIANCE_UNAVAILABLE,
                KIND_ERROR,
            )),
            handler::ALLEGIANCE => out.push(room_message(self.allegiance_room, text, 1)),
            // `CPlayer` 0x14D, 0x14A, 0x154, 0x1A4 -> 0x14C.
            handler::AUCTION | handler::BROADCAST | handler::CG | handler::CK => {
                let channel = match id {
                    handler::AUCTION => 0x10_0000,
                    handler::BROADCAST => 0x20_0000,
                    handler::CG => 0x8_0000,
                    _ => 0x40_0000,
                };
                if !text.is_empty() {
                    out.push(self.channel(channel, text, tables));
                }
            }
            _ => out.push(ShellAction::Command {
                handler: id,
                text,
                args,
            }),
        }
        out
    }

    /// `CPlayer` 0x131: the name runs to the first token ending in `,` (or is the first
    /// token when none does); the message is the rest.
    fn tell(&mut self, words: &[String], tables: &StringTableIds, out: &mut Vec<ShellAction>) {
        if words.len() <= 1 {
            out.push(self.help(handler::TELL));
            return;
        }
        let comma = words.iter().position(|w| w.ends_with(','));
        let taken = comma.map_or(words.len(), |at| at + 1);
        let spaced: String = words[..taken].iter().map(|w| format!(" {w}")).collect();
        if spaced.chars().count() < 3 {
            let error = line(tables, TABLE_COMMUNICATION, BAD_NAME);
            out.push(show(
                KIND_TELL_ERROR,
                with(error, VAR_TELL_NAME, literal(&spaced)),
            ));
            out.push(self.help(handler::TELL));
            return;
        }
        let (name, message) = match comma {
            Some(_) => {
                let name: String = spaced.chars().skip(1).collect();
                (name[..name.len() - 1].to_owned(), words[taken..].join(" "))
            }
            None => (words[0].clone(), words[1..].join(" ")),
        };
        if message.is_empty() {
            let error = line(tables, TABLE_COMMUNICATION, NO_MESSAGE);
            out.push(show(
                KIND_TELL_ERROR,
                with(error, VAR_PLAYER, literal(&name)),
            ));
            out.push(self.help(handler::TELL));
            return;
        }
        tell_to(&name, &message, true, out);
        self.retell = Some(name);
    }

    /// `CPlayer` 0x14C: a channel's room, or why there is none.
    fn channel(&self, channel: u32, text: String, tables: &StringTableIds) -> ShellAction {
        if let Some((_, room)) = self.channels.iter().find(|(c, _)| *c == channel) {
            return room_message(*room, text, 0);
        }
        let name = CHANNEL_NAMES
            .iter()
            .find(|(c, _)| *c == channel)
            .map(|(_, id)| line(tables, TABLE_COMMUNICATION, *id))
            .unwrap_or_default();
        // `Player` 0x7F: one of the five chat windows' filters shows the channel.
        let listens = (0..=4).any(|window| {
            self.filters
                .iter()
                .any(|(w, mask)| *w == window && mask & channel != 0)
        });
        let id = if listens {
            CHANNEL_UNAVAILABLE
        } else {
            NOT_SUBSCRIBED
        };
        let error = line(tables, TABLE_COMMUNICATION, id);
        show(KIND_ERROR, with(error, VAR_CHANNEL, name))
    }

    /// `CPlayer` 0x94: add, remove or list through the GUN service. The client has no GUN
    /// connection (`TurbineGun::S_pTurbineGun` is null), so each request fails with the
    /// service line after its "Squelching ..." line.
    fn squelch(
        &self,
        n: usize,
        mut args: Vec<String>,
        speaker: Speaker<'_>,
        tables: &StringTableIds,
        out: &mut Vec<ShellAction>,
    ) {
        let word = |i: usize, w: &str| self.commands.is_word(SQUELCH_WORDS[i], w);
        let service = show_line(tables, TABLE_COMMUNICATION, NO_SQUELCH_SERVICE, KIND_ERROR);
        let sub = if n < 2 { String::new() } else { args.remove(0) };
        if n >= 2 && (word(0, &sub) || word(1, &sub)) {
            let add = word(0, &sub);
            if args.is_empty() {
                let id = if add { SQUELCH_WHO } else { UNSQUELCH_WHO };
                out.push(show_line(tables, TABLE_COMMUNICATION, id, KIND_ERROR));
                return;
            }
            let name = args.join(" ");
            if add && name.eq_ignore_ascii_case(speaker.name) {
                out.push(show_line(
                    tables,
                    TABLE_COMMUNICATION,
                    SQUELCH_SELF,
                    KIND_ERROR,
                ));
                return;
            }
            let id = if add { SQUELCHING } else { UNSQUELCHING };
            let notice = with(
                line(tables, TABLE_COMMUNICATION, id),
                VAR_PLAYER,
                literal(&name),
            );
            out.push(show(KIND_INFO, notice));
            out.push(service);
        } else if n >= 2 && word(2, &sub) {
            out.push(service);
        } else {
            out.push(self.help(handler::SQUELCH));
        }
    }
}

/// `gmCEntity` 0x11 (`CCommunicationSystem` 0xB): a tell, mode 0. A name with `@` goes to
/// the GUN service instead (`gmCEntity` 0x24), which the client does not have; /reply never
/// checks.
fn tell_to(name: &str, text: &str, gun: bool, out: &mut Vec<ShellAction>) {
    if gun && name.contains('@') {
        return;
    }
    out.push(ShellAction::Send(ScriptRequest::Tell {
        mode: 0,
        text: literal(text),
        recipient: literal(name),
    }));
}

fn room_message(room: u32, text: String, flags: u32) -> ShellAction {
    ShellAction::SendToRoom { room, text, flags }
}

#[path = "state.rs"]
mod state;
