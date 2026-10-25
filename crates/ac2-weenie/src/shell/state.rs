//! What changes the chat state outside the command line: speech and tells heard (the squelch
//! check, the reply name), the server calls that set the rooms, and the GUN service's
//! squelch-list results.

use ac2_net::protocol::{NativeObject, PackValue, ScriptCall, StringInfo};

use super::super::lines::*;
use super::{heap, root, ChatState, CHANNEL_DATA_PACKAGE, FELLOWSHIP_PACKAGE};
use crate::notice::GameplayNotice;
use crate::tables::StringTableIds;

/// `WM_Allegiance` 0x30 -> `CPlayer` 0x134; `WM_Communication` 0x1B -> `CPlayer` 0x147;
/// `WM_Fellowship` 0x2B -> `FellowshipSystem` 0x3D and 0x28 -> `FellowshipSystem` 0x8.
pub(crate) const ALLEGIANCE_ROOM: u32 = 0x8203_0030;
pub(crate) const CHANNEL_TABLE: u32 = 0x8205_001B;
pub(crate) const SET_FELLOWSHIP: u32 = 0x820C_002B;
pub(crate) const CLEAR_FELLOWSHIP: u32 = 0x820C_0028;

fn notice(kind: u32, text: StringInfo) -> GameplayNotice {
    GameplayNotice::DisplayStringInfo { kind, text }
}

impl ChatState {
    /// `CPlayer` 0x11A (`HasSquelched`): a listed name, compared without case; a name with a
    /// `+` never is.
    pub fn has_squelched(&self, name: &str) -> bool {
        !name.contains('+') && self.squelched.iter().any(|s| s.eq_ignore_ascii_case(name))
    }

    /// The squelch list the GUN service last returned.
    pub fn squelched(&self) -> &[String] {
        &self.squelched
    }

    /// `CPlayer` 0xD1 (`CCommunicationSystem` 0x1, a tell heard): a literal teller name
    /// becomes the /reply name (`CPlayer` 0x3B).
    /// Its name also joins the heard history (`CPlayer` 0x10B).
    pub fn heard_tell(&mut self, teller: &StringInfo) {
        if let Some(name) = &teller.literal {
            self.reply = Some(name.clone());
            self.heard.add(name);
        }
    }

    /// The server calls that set the rooms; other calls are not the chat state's.
    pub fn server_call(
        &mut self,
        call: &ScriptCall,
        tables: &StringTableIds,
    ) -> Vec<GameplayNotice> {
        let first = call.arguments.first();
        match call.function {
            ALLEGIANCE_ROOM => match first {
                Some(PackValue::Int(room)) => return self.set_allegiance_room(*room, tables),
                _ => {}
            },
            // `CPlayer` 0x147 keeps the table (its joined/left lines through
            // `ChatChannelControl` are not mirrored).
            CHANNEL_TABLE => {
                let Some(PackValue::Reference(graph)) = first else {
                    return Vec::new();
                };
                let Some(object) = graph.ids.first().and_then(|id| heap(graph, *id)) else {
                    return Vec::new();
                };
                let Some(NativeObject::Hash(hash)) = &object.native else {
                    return Vec::new();
                };
                self.channels = hash
                    .keys
                    .iter()
                    .zip(&hash.values)
                    .filter_map(|(key, value)| {
                        let ac2_net::protocol::HashKey::Value(channel) = key else {
                            return None;
                        };
                        let data = heap(graph, *value as u32)?;
                        (data.pkg_id == CHANNEL_DATA_PACKAGE)
                            .then(|| (*channel as u32, data.slots.get(3).copied().unwrap_or(0)))
                    })
                    .collect();
            }
            // `Player` 0x63 keeps the fellowship; its room is `Fellowship` 0xF (slot 3).
            SET_FELLOWSHIP => {
                if let Some((_, fellowship)) = root(first, FELLOWSHIP_PACKAGE) {
                    self.fellowship_room = Some(fellowship.slots.get(3).copied().unwrap_or(0));
                }
            }
            // `FellowshipSystem` 0x23 clears it.
            CLEAR_FELLOWSHIP => self.fellowship_room = None,
            _ => {}
        }
        Vec::new()
    }

    /// `CPlayer` 0x134: the allegiance's room, and the line saying the channel was joined (a
    /// new non-zero room) or left (zero after a room).
    fn set_allegiance_room(&mut self, room: u32, tables: &StringTableIds) -> Vec<GameplayNotice> {
        let old = std::mem::replace(&mut self.allegiance_room, room);
        let id = match (old, room) {
            (_, 0) if old != 0 => CHANNEL_LEFT,
            (old, new) if new != 0 && new != old => CHANNEL_JOINED,
            _ => return Vec::new(),
        };
        let name = line(tables, TABLE_CHANNELS, ALLEGIANCE_CHANNEL);
        let text = with(line(tables, TABLE_COMMUNICATION, id), VAR_CHANNEL, name);
        vec![notice(KIND_INFO, text)]
    }

    /// `gmCWInterface::Result_IgnoredAdd` (`CPlayer` 0x32): the GUN service's answer to a
    /// squelch; on success the name joins the list.
    pub fn ignored_added(
        &mut self,
        status: u32,
        name: &str,
        tables: &StringTableIds,
    ) -> GameplayNotice {
        self.ignored_result(status, name, &ADD_ERRORS, SQUELCHED, tables)
    }

    /// `gmCWInterface::Result_IgnoredDelete` (`CPlayer` 0x5D): on success the first entry
    /// matching the name (without case) leaves the list.
    pub fn ignored_deleted(
        &mut self,
        status: u32,
        name: &str,
        tables: &StringTableIds,
    ) -> GameplayNotice {
        self.ignored_result(status, name, &DELETE_ERRORS, UNSQUELCHED, tables)
    }

    fn ignored_result(
        &mut self,
        status: u32,
        name: &str,
        errors: &[(u32, u32)],
        done: u32,
        tables: &StringTableIds,
    ) -> GameplayNotice {
        let error = errors.iter().find(|(s, _)| *s == status).map(|(_, id)| *id);
        let text = with(
            line(tables, TABLE_COMMUNICATION, error.unwrap_or(done)),
            VAR_PLAYER,
            literal(name),
        );
        if error.is_some() {
            return notice(KIND_ERROR, text);
        }
        if done == SQUELCHED {
            self.squelched.push(name.to_owned());
        } else if let Some(at) = self
            .squelched
            .iter()
            .position(|s| s.eq_ignore_ascii_case(name))
        {
            self.squelched.remove(at);
        }
        notice(KIND_INFO, text)
    }

    /// `gmCWInterface::Result_GetIgnoredList` (`CPlayer` 0x7): the list replaces the kept one
    /// (unless it failed and is not shown); shown, it is listed one name per line, and the
    /// listing pops the names from the very list it kept, emptying it, as retail's does.
    pub fn ignored_list(
        &mut self,
        status: u32,
        show: bool,
        list: Vec<String>,
        tables: &StringTableIds,
    ) -> Vec<GameplayNotice> {
        if !show {
            if status == 0 {
                self.squelched = list;
            }
            return Vec::new();
        }
        self.squelched = list;
        let at = |id| notice_line(tables, id);
        if status != 0 {
            return vec![notice(
                KIND_ERROR,
                line(tables, TABLE_COMMUNICATION, SQUELCH_LIST_ERROR),
            )];
        }
        if self.squelched.is_empty() {
            return vec![at(SQUELCH_LIST_EMPTY)];
        }
        let mut out = vec![at(SQUELCH_LIST)];
        out.extend(
            std::mem::take(&mut self.squelched)
                .into_iter()
                .map(|name| notice(KIND_INFO, literal(&format!("{name}\n")))),
        );
        out
    }
}

fn notice_line(tables: &StringTableIds, id: u32) -> GameplayNotice {
    notice(KIND_INFO, line(tables, TABLE_COMMUNICATION, id))
}
