//! `GunChatExtraInfo` (the chat "extra" blob GUN carries opaquely between two clients) and the
//! two receivers that unpack it, `gmCWInterface::HearTell` and `GUNEvent_HearFromRoom`.
//! The wire form is `ac2_net::GunChatExtraInfo` (`StreamPack`, admin 0x004534E0, Legions
//! 0x004545F0: `[u16 version][u16 misc][u32 weenieFlags][munged, 4-aligned PStringBase<ushort>]`).

use ac2_gun::TagBlob;
use ac2_net::protocol::GunChatExtraInfo;

use super::output::GunOutput;
use super::shared::GunShared;

/// `StringInfo::GetLiteralValue(true)` reads the literal and `UnescapeString`s it: `\q` is `"`,
/// any other `\x` is `x`.
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('q') => out.push('"'),
                Some(other) => out.push(other),
                None => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `StringTableMetaLanguage::RetrieveMetaLetters` (admin 0x00656FD0, Legions 0x00605920):
/// `(body, metaLetters)` of `original`. Without a `[` the body is `original` and the letters
/// are empty. Otherwise the string is split at the first `[`; the letters are what follows the
/// `[` up to the first `]` (everything after the `[` when there is none), and the body is the
/// text before the `[` with trailing white space trimmed.
pub fn retrieve_meta_letters(original: &str) -> (String, String) {
    let Some(open) = original.find('[') else {
        return (original.to_owned(), String::new());
    };
    let body = original[..open].trim_end().to_owned();
    let rest = &original[open + 1..];
    let letters = match rest.find(']') {
        Some(close) => &rest[..close],
        None => rest,
    };
    (body, letters.to_owned())
}

/// `GunChatExtraInfo::GunChatExtraInfo(pWInterface, weenieFlags)` (admin 0x00452580, Legions
/// 0x00453690): version 1, misc flags 0, the weenie flags, and the meta letters from
/// `gmCWInterface::InqName` (virtual `+0x70`, `CWRepository::InqStringInfo(0x100, name)`): when
/// it gives a name, `GetLiteralValue(true)` (the Legions instruction at 0x4536EC pushes 1; the
/// decompile shows `false`) and `RetrieveMetaLetters` of it.
pub fn gun_chat_extra_info(shared: &GunShared, weenie_flags: u32) -> GunChatExtraInfo {
    let name = shared.player_name();
    let letters = if name.is_empty() {
        String::new()
    } else {
        retrieve_meta_letters(&unescape(&name)).1
    };
    GunChatExtraInfo::new(weenie_flags, letters)
}

/// `GunChatExtraInfo::PackIntoBlob` (admin 0x004520F0, Legions 0x00453230): `{size, bytes}`.
/// (Retail's `{0, 0}` on a pack failure cannot occur: the size is the packed length.)
pub fn pack_into_blob(info: &GunChatExtraInfo) -> TagBlob {
    let bytes = info.encode();
    TagBlob {
        cb_size: bytes.len() as u32,
        data: Some(bytes),
    }
}

/// The unpack both receivers start with: a default info (`m_version` 1, flags 0, no letters)
/// and `StreamPack(UNPACK)` over the blob. `None` when the unpack failed (under 8 bytes, a
/// truncated string, or no blob: retail's NULL `tagBLOB*` would crash).
fn unpack(extra: Option<&TagBlob>) -> Option<GunChatExtraInfo> {
    GunChatExtraInfo::decode(extra?.bytes()).ok()
}

/// `name` with `[letters]` appended when the unpack worked and the letters are not empty
/// (`append_n_chars`, `"[" + letters + "]"`), and the weenie flags it gave (0 if it failed).
fn with_metas(name: &str, extra: Option<&TagBlob>) -> (String, u32) {
    match unpack(extra) {
        Some(info) if !info.meta_letters.is_empty() => {
            (format!("{name}[{}]", info.meta_letters), info.weenie_flags)
        }
        Some(info) => (name.to_owned(), info.weenie_flags),
        None => (name.to_owned(), 0),
    }
}

/// `gmCWInterface::HearTell(teller, msg, pExtraData)` (admin 0x004530F0, Legions 0x00454200):
/// `tellerWithMetas` is the teller with `[letters]` appended (the speech is not touched);
/// `CallIFun_HearTell(process, INVALID_IID, StringInfo(teller, escape 0), StringInfo(msg,
/// escape 1), flags)`.
pub fn hear_tell(shared: &GunShared, teller: &str, msg: &str, extra: Option<&TagBlob>) {
    let (teller, flags) = with_metas(teller, extra);
    shared.push(GunOutput::HearTell {
        teller,
        message: msg.to_owned(),
        flags,
    });
}

/// `gmCWInterface::GUNEvent_HearFromRoom(speaker, message, roomID, pExtraData)` (admin
/// 0x00453320, Legions 0x00454430): the same suffix on the speaker (`speakerWithMetas`,
/// escape 0); the message goes as the raw `PStringBase*`.
pub fn gun_event_hear_from_room(
    shared: &GunShared,
    speaker: &str,
    message: &str,
    room: u32,
    extra: Option<&TagBlob>,
) {
    let (speaker, flags) = with_metas(speaker, extra);
    shared.push(GunOutput::EventHearFromRoom {
        speaker,
        message: message.to_owned(),
        room,
        flags,
    });
}
