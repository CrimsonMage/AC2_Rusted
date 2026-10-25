//! The chat commands the player's script registers (`CPlayer` 0x121 `RegisterCommands`),
//! resolved from the DATs for `OnlineSession::set_chat_commands`: the command names
//! (`CWInterface::AddShellCmdByStringID` over string table enum 0x41000007), one command per
//! emote name (`PrefabEmoteTable`'s `EmoteInfo` names), the usage texts and
//! `ID_DefaultCommand`. Run it on the loader side (it reads string tables and a `WState`).

use ac2_net::protocol::{HashKey, NativeObject, PackedEntry, PackedReference};
use ac2_script::data::DB_TYPE_STRING_TABLE;
use ac2_script::native::StringInfo;
use ac2_script::vm::ScriptData;
use ac2_weenie::{
    ChatCommands, COMMAND_TABLE_ENUM, DEFAULT_COMMAND, EMOTE_TABLE_PACKAGE, HELP_STRINGS,
    REGISTERED_COMMANDS, SQUELCH_WORDS,
};

/// `FileArray::GetDIDFromEnum`'s group for string tables.
const STRING_TABLE_GROUP: u32 = 0x4000_0004;

/// `PStringBase<unsigned short>::whitespace_string`.
const WHITESPACE: [u16; 4] = [0x20, 0x09, 0x0D, 0x0A];

/// `StringTableMetaLanguage::StripMetaLetters`: cut at the first `[` not escaped by a
/// backslash, then drop the trailing whitespace.
fn strip_meta_letters(text: &[u16]) -> Vec<u16> {
    let mut i = 0;
    let mut cut = None;
    while i < text.len() {
        match text[i] {
            0x5C => i += 1,
            0x5B => {
                cut = Some(i);
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let Some(cut) = cut else {
        return text.to_vec();
    };
    let mut out = text[..cut].to_vec();
    while out.last().is_some_and(|c| WHITESPACE.contains(c)) {
        out.pop();
    }
    out
}

/// `StringTableMetaLanguage::UnescapeString`: `\q` is `"`, `\x` is `x`.
fn unescape(text: &[u16]) -> String {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        if text[i] == 0x5C {
            let next = text.get(i + 1).copied().unwrap_or(0);
            out.push(if next == u16::from(b'q') { 0x22 } else { next });
            i += 2;
        } else {
            out.push(text[i]);
            i += 1;
        }
    }
    String::from_utf16_lossy(&out)
}

/// `StringInfo::GetString` (`InqString`: the rendered text without its meta letters,
/// unescaped) of a table string with no variables, or `None` when it does not resolve.
fn get_string(data: &dyn ScriptData, table_id: u32, string_id: u32) -> Option<String> {
    let table = data.string_table(table_id)?;
    let info = StringInfo {
        string_id,
        table_id,
        ..StringInfo::default()
    };
    let text = data.render_string(&table, &info).ok()?;
    Some(unescape(&strip_meta_letters(&text)))
}

/// `StringInfo::SetStringIDandTableEnum` then `GetString`.
fn table_string(data: &dyn ScriptData, table_enum: u32, string_id: u32) -> Option<String> {
    let did = data
        .enum_did(table_enum, STRING_TABLE_GROUP)
        .filter(|did| data.db_type(*did) == Some(DB_TYPE_STRING_TABLE))?;
    get_string(data, did, string_id)
}

/// `PrefabEmoteTable`'s emotes: (`EmoteInfo` slot 4's name, emote id), in packed order.
fn emotes(data: &dyn ScriptData) -> Vec<(String, u32)> {
    let did = ac2_script::wstate::singleton_did(EMOTE_TABLE_PACKAGE);
    let Some(state) = data.wstate(did) else {
        return Vec::new();
    };
    let Ok(graph) = PackedReference::from_bytes(&state.package_state) else {
        return Vec::new();
    };
    let heap = |id: u32| {
        let at = graph.ids.iter().position(|i| *i == id)?;
        match graph.entries.get(at)? {
            PackedEntry::Heap { object, .. } => Some(object),
            _ => None,
        }
    };
    let Some(table) = graph
        .ids
        .first()
        .and_then(|root| heap(*root))
        .and_then(|root| root.slots.first().copied())
        .and_then(heap)
    else {
        return Vec::new();
    };
    let Some(NativeObject::Hash(hash)) = &table.native else {
        return Vec::new();
    };
    hash.keys
        .iter()
        .zip(&hash.values)
        .filter_map(|(key, value)| {
            let HashKey::Value(emote) = key else {
                return None;
            };
            let info = heap(*value as u32)?;
            let Some(NativeObject::StringInfo(name)) = &heap(*info.slots.get(4)?)?.native else {
                return None;
            };
            let name = match &name.literal {
                Some(text) => text.clone(),
                None => get_string(data, name.table_id, name.string_id)?,
            };
            Some((name, *emote as u32))
        })
        .collect()
}

/// The commands `CPlayer` 0x121 registers, resolved through `data`.
pub fn chat_commands(data: &dyn ScriptData) -> ChatCommands {
    let default = table_string(data, DEFAULT_COMMAND.0, DEFAULT_COMMAND.1).unwrap_or_default();
    let mut commands = ChatCommands::new(&default);
    for (id, handler, expandable) in REGISTERED_COMMANDS {
        if let Some(names) = table_string(data, COMMAND_TABLE_ENUM, id) {
            commands.register(&names, handler, expandable);
        }
    }
    for (names, emote) in emotes(data) {
        commands.add_emote(&names, emote);
    }
    for (handler, table_enum, first, second) in HELP_STRINGS {
        let text = |id| table_string(data, table_enum, id).unwrap_or_default();
        commands.set_help(handler, text(first) + &text(second));
    }
    for id in SQUELCH_WORDS {
        if let Some(word) = table_string(data, COMMAND_TABLE_ENUM, id) {
            commands.set_word(id, word);
        }
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registered_commands_and_emotes_resolve() {
        use ac2_weenie::chat_handler::{EMOTE_BY_NAME, SAY, TELL};
        let Some(dataset) = ac2_dat::test_support::env_dataset() else {
            return;
        };
        let store = ac2_db::ResourceStore::new(dataset.clone());
        let data = ac2_script::data::DatScriptData::new(dataset, store);
        let commands = chat_commands(&data);
        // Speech: `ID_DefaultCommand` is a registered command.
        assert_eq!(commands.handler("say"), Some(SAY));
        assert_eq!(commands.handler("T"), Some(TELL));
        assert!(commands.help(TELL).starts_with("usage:"));
        // One command per emote name.
        assert_eq!(emotes(&data).len(), 47);
        assert_eq!(commands.handler("wave"), Some(EMOTE_BY_NAME));
        assert!(commands.emote("WAVE").is_some());
        let mut chat = ac2_weenie::ChatState::default();
        chat.set_commands(commands);
        let tables = ac2_weenie::StringTableIds::default();
        let said = chat.process_text("hello", ac2_weenie::Speaker::default(), &tables);
        assert!(matches!(
            said.as_slice(),
            [ac2_weenie::ShellAction::Send(
                ac2_net::protocol::ScriptRequest::Say { .. }
            )]
        ));
    }
}
