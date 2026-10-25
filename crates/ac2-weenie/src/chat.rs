//! The chat lines the `CCommunicationSystem` script builds for what the player hears, and the
//! requests its senders make (Legions WLib 0x56000005). Each line is the `StringInfo` the
//! script hands `CM_Communication::SendNotice_DisplayStringInfo`: a string-table entry and its
//! variables, resolved by the UI.

use ac2_net::protocol::{ScriptRequest, StringInfo, StringVariable};

use crate::player_script::GameplayNotice;
use crate::tables::{StringTableIds, TABLE_ENUM_COMMUNICATION, TABLE_ENUM_TELL};

/// "You say ..." (`CCommunicationSystem` 0x6, the speaker is the player).
const STRING_YOU_SAY: u32 = 0xD85E_9379;
/// "... says ..." (another speaker).
const STRING_SAYS: u32 = 0xCF6B_AF4F;
/// The tell strings by the tell's kind 0x40000001-0x40000003 (table `0x41000001`), and any
/// other kind (table `0x41000004`).
const STRING_TELLS: [u32; 3] = [0x99E6_8169, 0xF520_6FA9, 0x555A_8315];
const STRING_TELL_OTHER: u32 = 0x980B_A847;
/// Variables: the message, the speaker's id (`ToIID`), the speaker's name.
const VAR_TEXT: u32 = 0x9B4E_3800;
const VAR_IID: u32 = 0x9310_0000;
const VAR_NAME: u32 = 0x9D06_6C50;
/// Display types (`gmCEntity` 0x1A's second argument).
pub const DISPLAY_SPEECH: u32 = 16;
pub const DISPLAY_TELL: u32 = 32;
/// Or-ed into [`DISPLAY_TELL`] for the three named tell kinds.
const DISPLAY_TELL_KIND: u32 = 0x2_0000;

/// `ToIID`: an instance id as text, `%04X:%04X:%04X:%04X` from the high word down.
pub fn to_iid(id: u64) -> String {
    format!(
        "{:04X}:{:04X}:{:04X}:{:04X}",
        id >> 48,
        (id >> 32) & 0xFFFF,
        (id >> 16) & 0xFFFF,
        id & 0xFFFF
    )
}

fn literal(text: String) -> StringInfo {
    StringInfo {
        literal: Some(text),
        ..StringInfo::default()
    }
}

/// `StringInfo::AddVariable_StringInfo`; `AddVariable_String` stores its text the same way
/// (a nested `StringInfo` with the literal, `SID_StringInfo`).
fn nested(id: u32, value: StringInfo) -> (u32, StringVariable) {
    (id, StringVariable::Nested(Box::new(value)))
}

/// `CCommunicationSystem` 0x6: what someone said nearby (`WM_Communication` 0x10). The
/// session drops squelched speakers first (`CPlayer` 0x11A, `ChatState::has_squelched`).
pub fn hear_speech(
    text: StringInfo,
    speaker_name: StringInfo,
    speaker: u64,
    player: u64,
    tables: &StringTableIds,
) -> GameplayNotice {
    let mut line = StringInfo {
        table_id: tables.did(TABLE_ENUM_COMMUNICATION),
        ..StringInfo::default()
    };
    line.variables.push(nested(VAR_TEXT, text));
    if speaker == player {
        line.string_id = STRING_YOU_SAY;
    } else {
        line.string_id = STRING_SAYS;
        line.variables
            .push(nested(VAR_IID, literal(to_iid(speaker))));
        line.variables.push(nested(VAR_NAME, speaker_name));
    }
    GameplayNotice::DisplayStringInfo {
        kind: DISPLAY_SPEECH,
        text: line,
    }
}

/// `CCommunicationSystem` 0x1: a tell to the player (`WM_Communication` 0x17), shown through
/// `CPlayer` 0x2B. The session runs the squelch check and keeps the /reply name first
/// (`CPlayer` 0x11A, 0xD1); the tell history list (`CPlayer` 0x10B) is not mirrored.
pub fn hear_tell(
    kind: u32,
    text: StringInfo,
    teller_name: StringInfo,
    teller: u64,
    tables: &StringTableIds,
) -> GameplayNotice {
    let (table_id, string_id, display) = match kind {
        0x4000_0001..=0x4000_0003 => (
            tables.did(TABLE_ENUM_TELL),
            STRING_TELLS[(kind - 0x4000_0001) as usize],
            DISPLAY_TELL | DISPLAY_TELL_KIND,
        ),
        _ => (
            tables.did(TABLE_ENUM_COMMUNICATION),
            STRING_TELL_OTHER,
            DISPLAY_TELL,
        ),
    };
    let line = StringInfo {
        string_id,
        table_id,
        literal: None,
        variables: vec![
            nested(VAR_IID, literal(to_iid(teller))),
            nested(VAR_NAME, teller_name),
            nested(VAR_TEXT, text),
        ],
    };
    GameplayNotice::DisplayStringInfo {
        kind: display,
        text: line,
    }
}

/// `CPlayer` 0xDB (the say command): the text as a literal, mode 0. Retail refuses an empty
/// command line with its help text; an empty `text` sends nothing here.
pub fn say(text: &str) -> Option<ScriptRequest> {
    (!text.is_empty()).then(|| ScriptRequest::Say {
        mode: 0,
        text: literal(text.to_owned()),
    })
}

/// `CPlayer` 0x112 -> `CCommunicationSystem` 0xB: a tell, mode 0. A recipient with `@` goes
/// through the chat server instead (`RequestGUN_SendToUser`), which is not mirrored.
pub fn tell(recipient: &str, text: &str) -> Option<ScriptRequest> {
    (!recipient.contains('@') && !recipient.is_empty()).then(|| ScriptRequest::Tell {
        mode: 0,
        text: literal(text.to_owned()),
        recipient: literal(recipient.to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> StringInfo {
        literal(s.into())
    }

    fn tables() -> StringTableIds {
        let mut t = StringTableIds::default();
        t.insert(TABLE_ENUM_TELL, 0x2500_0001);
        t.insert(TABLE_ENUM_COMMUNICATION, 0x2500_0004);
        t
    }

    #[test]
    fn iids_format_as_four_hex_words() {
        assert_eq!(to_iid(0x8000_0002_0000_0042), "8000:0002:0000:0042");
    }

    #[test]
    fn speech_names_the_speaker_unless_it_is_the_player() {
        let GameplayNotice::DisplayStringInfo { kind, text: line } =
            hear_speech(text("hi"), text("Birch"), 9, 1, &tables())
        else {
            panic!();
        };
        assert_eq!(kind, DISPLAY_SPEECH);
        assert_eq!((line.table_id, line.string_id), (0x2500_0004, 0xCF6B_AF4F));
        let ids: Vec<u32> = line.variables.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, [VAR_TEXT, VAR_IID, VAR_NAME]);
        let GameplayNotice::DisplayStringInfo { text: own, .. } =
            hear_speech(text("hi"), text("Explorer"), 1, 1, &tables())
        else {
            panic!();
        };
        assert_eq!(own.string_id, 0xD85E_9379);
        assert_eq!(own.variables.len(), 1);
    }

    #[test]
    fn tells_pick_their_string_by_kind() {
        let GameplayNotice::DisplayStringInfo { kind, text: line } =
            hear_tell(0x4000_0002, text("psst"), text("Birch"), 9, &tables())
        else {
            panic!();
        };
        assert_eq!(kind, 32 | 0x2_0000);
        assert_eq!((line.table_id, line.string_id), (0x2500_0001, 0xF520_6FA9));
        let GameplayNotice::DisplayStringInfo { kind, text: line } = hear_tell(
            0,
            text("psst"),
            text("Birch"),
            9,
            &StringTableIds::default(),
        ) else {
            panic!();
        };
        assert_eq!((kind, line.string_id, line.table_id), (32, 0x980B_A847, 0));
        assert_eq!(say(""), None);
        assert_eq!(tell("Someone@Elsewhere", "hi"), None);
    }
}
