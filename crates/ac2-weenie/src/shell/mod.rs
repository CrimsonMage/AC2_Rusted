//! The chat command line: `CWInterface::ProcessText` (alias expansion through the options
//! profile), `CPlayerShell::ProcessText` (a `/` command, or the default command for speech)
//! and `ShellCmdTable::ProcessCommand` (tokens, the command's handler, the rest of the line),
//! over the commands the player's script registers (`CPlayer` 0x121 `RegisterCommands`). The
//! handlers the port runs natively are in [`handlers`]; any other registered command comes
//! back to the caller by its handler.
//!
//! The command names live in the DATs (string tables and the emote table), so the caller
//! resolves [`ChatCommands`] on the loader side and hands them to the session.

mod handlers;
mod history;
mod lines;

use std::collections::HashMap;

pub use handlers::{ChatState, ShellAction, Speaker};
pub use history::ChatHistory;
pub(crate) use lines::room_refused;
pub use lines::SQUELCH_WORDS;

/// `CPlayer` handler functions (index in package 0x1AA4) the port runs natively.
pub mod handler {
    /// say, s, c.
    pub const SAY: u32 = 0xDB;
    /// tell, t, send.
    pub const TELL: u32 = 0x131;
    /// retell, rt.
    pub const RETELL: u32 = 0x112;
    /// reply, r.
    pub const REPLY: u32 = 0xC5;
    /// emote, em, me, e: free emote text.
    pub const EMOTE: u32 = 0x5E;
    /// Every emote's name (`CPlayer` 0x121 registers the emote table's names).
    pub const EMOTE_BY_NAME: u32 = 0xCF;
    /// squelch.
    pub const SQUELCH: u32 = 0x94;
    /// f, group, g: the fellowship's room.
    pub const FELLOWSHIP: u32 = 0x103;
    /// a, gu, guild, as, o: the allegiance's room.
    pub const ALLEGIANCE: u32 = 0xE4;
    /// ct, auction: channel 0x100000.
    pub const AUCTION: u32 = 0x14D;
    /// cr, broadcast, br, y, yell, shout: channel 0x200000.
    pub const BROADCAST: u32 = 0x14A;
    /// cg: channel 0x80000.
    pub const CG: u32 = 0x154;
    /// ck: channel 0x400000.
    pub const CK: u32 = 0x1A4;
}

/// The string table enum `CPlayer` 0x121 names its commands from.
pub const COMMAND_TABLE_ENUM: u32 = 0x4100_0007;

/// `CPlayer` 0x121's `AddShellCmdByStringID` calls, in order: the string (a list of names
/// split on `;:,`), the handler, and whether an alias may expand it (`final`).
#[rustfmt::skip]
pub const REGISTERED_COMMANDS: [(u32, u32, bool); 70] = [
    (0x282A_FC8C, 0x09D, true), (0xAD52_FD3C, 0x0DB, true), (0x2824_FD4C, 0x131, true),
    (0x2E20_11F4, 0x112, true), (0x2F24_842C, 0x0C5, true), (0xC960_12A6, 0x0E1, true),
    (0x0E60_11F9, 0x094, true), (0x1303_E593, 0x103, true), (0x7B37_CF7E, 0x0E4, true),
    (0x9CAF_E7B8, 0x0FF, true), (0x05DC_979B, 0x02D, true), (0xF684_DEE7, 0x05C, true),
    (0x0B57_5A93, 0x183, true), (0x15B8_E4CD, 0x17F, true), (0x713E_10D6, 0x180, true),
    (0x3CCB_736A, 0x18B, true), (0x3CCD_3989, 0x189, true), (0x7A67_B36F, 0x18D, true),
    (0x391B_509F, 0x18E, true), (0x60BC_E1C6, 0x197, true), (0x6A6F_EFF6, 0x195, true),
    (0x7124_E1C6, 0x196, true), (0xBE2D_A497, 0x14D, true), (0x980C_8889, 0x14A, true),
    (0x6BC5_DB8B, 0x154, true), (0x111E_53A7, 0x150, true), (0x11F6_198E, 0x1A4, true),
    (0x28E8_995D, 0x05E, true), (0x8F77_D9C4, 0x0DE, true), (0x6BD2_FCCD, 0x100, true),
    (0x8A64_55BC, 0x040, true), (0x0962_0F1D, 0x0B5, false), (0xFB4D_9507, 0x0C7, true),
    (0x47D1_438D, 0x0B8, true), (0xE7D1_434F, 0x0D3, true), (0xA828_9989, 0x019, true),
    (0x9B67_A708, 0x123, true), (0x6824_9383, 0x0FC, true), (0xAA22_0F2C, 0x08D, true),
    (0x5F4B_89FA, 0x0BA, true), (0xC4CB_0E4E, 0x07C, true), (0x822B_A23A, 0x0AB, true),
    (0xE51B_765D, 0x050, true), (0xF50D_31C1, 0x096, true), (0x6E30_FCDD, 0x15D, true),
    (0xE9A3_71D7, 0x177, true), (0x4F51_1B89, 0x1C1, true), (0x2E12_FD0C, 0x178, true),
    (0x2F25_6C37, 0x188, true), (0xFEF1_9514, 0x19B, true), (0xEA45_482F, 0x1FE, true),
    (0x2968_FD1E, 0x16C, true), (0xA8A0_FC2D, 0x16A, true), (0xABEB_93DC, 0x167, true),
    (0xAFE7_7D4C, 0x168, true), (0x4EA6_0857, 0x166, true), (0x6FAA_8FF7, 0x171, true),
    (0x2A64_FC4E, 0x170, true), (0x2AD2_FC2E, 0x174, true), (0x9156_09DF, 0x16D, true),
    (0x2F25_0FC4, 0x16B, true), (0x0B65_993E, 0x162, true), (0x8A6B_987D, 0x164, true),
    (0xAF92_FC4D, 0x169, true), (0x0E63_163E, 0x16F, true), (0x8F20_1DC6, 0x16E, true),
    (0x88D2_FD7D, 0x163, true), (0xE643_937F, 0x172, true), (0xF312_FC78, 0x175, true),
    (0x8826_093E, 0x165, true),
];

/// `ID_DefaultCommand` (`Tokenize`) in string table enum 0x40000009: the command
/// `CPlayerShell::ProcessSpeech` puts before a line that is not a command.
pub const DEFAULT_COMMAND: (u32, u32) = (0x4000_0009, 0x8852_3C2E);

/// The usage text each natively run handler shows (`CPlayer` 0xB4 and its kin): the handler,
/// the string table enum, and the two strings it joins.
#[rustfmt::skip]
pub const HELP_STRINGS: [(u32, u32, u32, u32); 9] = [
    (handler::SAY, 0x4100_0006, 0xAD4A_7D3C, 0xAD4B_FD3C),
    (handler::TELL, 0x4100_0006, 0x2824_CC4C, 0x2824_CF4C),
    (handler::RETELL, 0x4100_0006, 0xEA20_11F4, 0xE620_11F4),
    (handler::REPLY, 0x4100_0006, 0x2F24_844E, 0x2F24_8448),
    (handler::EMOTE, 0x4100_0006, 0x28E8_993F, 0x28E8_9939),
    (handler::SQUELCH, 0x4100_0006, 0x0FE8_11F9, 0x0FF0_11F9),
    (handler::FELLOWSHIP, 0x4100_001B, 0x8D1F_1D84, 0x3C3F_8480),
    (handler::ALLEGIANCE, 0x4100_001B, 0xE52B_3769, 0x540B_AE6D),
    (handler::CK, 0x4100_001B, 0x1305_DA11, 0x3393_FE02),
];

/// The emote table singleton (`PrefabEmoteTable`, package 0x397): slot 0 is an `ARHash` of
/// emote id to `EmoteInfo` (package 0x1CC), whose slot 4 is the emote's name.
pub const EMOTE_TABLE_PACKAGE: u32 = 0x397;

/// The registered commands, resolved from the DATs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatCommands {
    /// Lowercased name -> (handler, expandable).
    commands: HashMap<String, (u32, bool)>,
    default_command: String,
    help: HashMap<u32, String>,
    /// Words the handlers match sub-commands against, by string id ([`SQUELCH_WORDS`]).
    words: HashMap<u32, String>,
    /// The emote names (a word list) and emote ids, in the table's order.
    emotes: Vec<(String, u32)>,
}

/// `_wcslwr` (C locale: only A-Z).
fn lower(text: &str) -> String {
    text.chars().map(|c| c.to_ascii_lowercase()).collect()
}

/// `IsWordInString`: `words` split on ` ,;` (`PSUtils::FindAllWords`) holds `word`, compared
/// without case (`_wcsicmp`).
fn is_word_in(words: &str, word: &str) -> bool {
    words
        .split([' ', ',', ';'])
        .any(|w| !w.is_empty() && w.eq_ignore_ascii_case(word))
}

/// `PStringBase<unsigned short>::whitespace_string`.
const WHITESPACE: [char; 4] = [' ', '\t', '\r', '\n'];

/// `PStringBase::trim(pre, post, filter)`: drop characters found in `filter` from the ends.
fn trim<'a>(text: &'a str, pre: bool, post: bool, filter: &[char]) -> &'a str {
    let text = if pre {
        text.trim_start_matches(filter)
    } else {
        text
    };
    if post {
        text.trim_end_matches(filter)
    } else {
        text
    }
}

impl ChatCommands {
    /// No commands yet; `default_command` is `ID_DefaultCommand`'s text.
    pub fn new(default_command: &str) -> Self {
        Self {
            default_command: default_command.to_owned(),
            ..Self::default()
        }
    }

    /// `CWInterface::AddShellCmd`: each name in `names` (split on `;:,`, lowercased, trimmed)
    /// runs `handler`; a name registered before keeps its first handler.
    pub fn register(&mut self, names: &str, handler: u32, expandable: bool) {
        for name in names.split([';', ':', ',']) {
            let name = lower(trim(name, true, true, &WHITESPACE));
            if !name.is_empty() {
                self.commands.entry(name).or_insert((handler, expandable));
            }
        }
    }

    /// `CPlayer` 0x121's emote loop: the emote's names become commands for
    /// [`handler::EMOTE_BY_NAME`], and `EmoteTable` 0xD finds the emote by them.
    pub fn add_emote(&mut self, names: &str, emote: u32) {
        self.register(names, handler::EMOTE_BY_NAME, true);
        self.emotes.push((names.to_owned(), emote));
    }

    /// The usage text `handler` shows when it is used wrongly.
    pub fn set_help(&mut self, handler: u32, text: String) {
        self.help.insert(handler, text);
    }

    /// The text of a string a handler matches words against (`CPlayer` 0x101).
    pub fn set_word(&mut self, id: u32, text: String) {
        self.words.insert(id, text);
    }

    /// `CPlayer` 0x101: whether `word` is one of string `id`'s words (`IsWordInString`).
    pub(crate) fn is_word(&self, id: u32, word: &str) -> bool {
        self.words
            .get(&id)
            .is_some_and(|words| is_word_in(words, word))
    }

    /// The handler a command name runs.
    pub fn handler(&self, name: &str) -> Option<u32> {
        self.commands.get(&lower(name)).map(|c| c.0)
    }

    /// The usage text `handler` shows (empty when none was set).
    pub fn help(&self, handler: u32) -> String {
        self.help.get(&handler).cloned().unwrap_or_default()
    }

    /// `EmoteTable` 0xD: the first emote whose names (`IsWordInString`: split on ` ,;`,
    /// compared without case) include `word`.
    pub fn emote(&self, word: &str) -> Option<u32> {
        self.emotes
            .iter()
            .find_map(|(names, emote)| is_word_in(names, word).then_some(*emote))
    }

    /// `ShellCmdTable::ExpandableCommand`: an unregistered command may be an alias.
    fn expandable(&self, cmd: &str) -> bool {
        self.commands.get(&lower(cmd)).is_none_or(|c| c.1)
    }
}

/// `CPlayerShell::ExpandableText`: the command after the first `/` (up to a space) decides;
/// a line with no `/` may be an alias, an empty command may not.
fn expandable_text(text: &str, commands: &ChatCommands) -> bool {
    let Some(at) = text.find('/') else {
        return true;
    };
    let rest = &text[at + 1..];
    let cmd = &rest[..rest.find(' ').unwrap_or(rest.len())];
    !cmd.is_empty() && commands.expandable(cmd)
}

/// `CWInterface::ProcessText`'s alias step (`gmCWInterface::HandleAlias` through
/// `GameplayOptionsProfile::InqAlias`): follow the whole line through the alias table until a
/// line repeats.
pub(crate) fn expand_aliases(
    line: &str,
    aliases: &[(String, String)],
    commands: &ChatCommands,
) -> String {
    if !expandable_text(line, commands) {
        return line.to_owned();
    }
    let mut seen: Vec<&str> = Vec::new();
    let mut current = line;
    loop {
        if seen.contains(&current) {
            return current.to_owned();
        }
        seen.push(current);
        current = aliases
            .iter()
            .find(|(alias, _)| alias == current)
            .map_or(current, |(_, text)| text.as_str());
    }
}

/// What `ShellCmdTable::ProcessCommand` made of a line (`ShellCmdStatus`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Parsed {
    /// `EmptyCmd_ShellCmdStatus`.
    Empty,
    /// `Failure_ShellCmdStatus` (an unclosed quote).
    Failure,
    /// `UnknownCmd_ShellCmdStatus`.
    Unknown,
    /// The command's handler, the line after the command, and every token (the command's
    /// first, as typed).
    Command {
        handler: u32,
        text: String,
        args: Vec<String>,
    },
}

/// `ShellCmdTable::ProcessCommand`'s tokens: `"..."` is one token (the character after the
/// closing quote is skipped), anything else runs to the next space; empty tokens drop.
fn tokens(line: &[char]) -> Option<Vec<String>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < line.len() {
        let token: String = if line[i] == '"' {
            let close = i + 1 + line[i + 1..].iter().position(|c| *c == '"')?;
            let token: String = line[i + 1..close].iter().collect();
            i = close + 1;
            trim(&token, true, true, &['"']).to_owned()
        } else {
            let end = line[i..]
                .iter()
                .position(|c| *c == ' ')
                .map_or(line.len(), |p| i + p);
            let token: String = line[i..end].iter().collect();
            i = end;
            trim(&token, true, true, &WHITESPACE).to_owned()
        };
        if !token.is_empty() {
            out.push(token);
        }
        i += 1;
    }
    Some(out)
}

/// `ShellCmdTable::ProcessCommand`: the first token names the command (lowercased); the
/// text after it is the line with the command's characters trimmed from its front
/// (`trim(true, false, cmd)`), then its whitespace.
fn process_command(line: &str, commands: &ChatCommands) -> Parsed {
    let copy = trim(line, true, true, &WHITESPACE);
    if copy.is_empty() {
        return Parsed::Empty;
    }
    let chars: Vec<char> = copy.chars().collect();
    let Some(args) = tokens(&chars) else {
        return Parsed::Failure;
    };
    let Some(cmd) = args.first() else {
        return Parsed::Unknown;
    };
    let set: Vec<char> = cmd.chars().collect();
    let text = trim(trim(copy, true, false, &set), true, true, &WHITESPACE).to_owned();
    match commands.handler(cmd) {
        Some(handler) => Parsed::Command {
            handler,
            text,
            args,
        },
        None => Parsed::Unknown,
    }
}

/// `CPlayerShell::ProcessText`: a line starting with `/` loses its leading slashes and is a
/// command; any other line is speech (`ProcessSpeech`: the default command, a space, the
/// line).
pub(crate) fn process_text(line: &str, commands: &ChatCommands) -> Parsed {
    let text = trim(line, true, true, &WHITESPACE);
    if text.is_empty() {
        return Parsed::Empty;
    }
    if text.starts_with('/') {
        let command = trim(text, true, false, &['/']);
        if command.is_empty() {
            return Parsed::Empty;
        }
        return process_command(command, commands);
    }
    process_command(&format!("{} {text}", commands.default_command), commands)
}

#[cfg(test)]
mod tests;
