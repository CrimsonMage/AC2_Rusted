//! The session's gameplay entry points: the IFuns the native UI calls on the player's weenie
//! (`gmCWInterface::SetSelection`, `StartExamination`, ...), run by the
//! [`PlayerScript`] stand-in, whose messages go out as `InterpSEvent`s.

use ac2_net::protocol::{
    ChatMessage, ChatRequest, ExaminationRequest, GunChatExtraInfo, ScriptMessage,
};

use super::{OnlineSession, SessionEvent};
use crate::player_script::{GameplayNotice, PlayerScript, ScriptOutput};
use crate::qualities::WeenieQualities;
use crate::world::World;

/// `CWInterface::IsCreature`: the weenie type (int quality 0x14) has all of 0x4F.
const CREATURE_TYPE: i32 = 0x4F;

impl OnlineSession {
    /// The player's script state (selection, examination).
    pub fn player_script(&self) -> &PlayerScript {
        &self.script
    }

    /// `gmCWInterface::SetSelection` (IFun `SetSelection`): select an object, or none with
    /// [`crate::NO_SELECTION`]. Returns the notices the script raised.
    pub fn select(&mut self, id: u64) -> Vec<SessionEvent> {
        let combat_target = self.is_combat_target(id);
        let out = self.script.set_selection(id, combat_target);
        self.apply_script(out)
    }

    /// `gmCWInterface::SetExaminationRequest`.
    pub fn examine(&mut self, request: ExaminationRequest) -> Vec<SessionEvent> {
        let out = self.script.examine(request);
        self.apply_script(out)
    }

    /// `gmCWInterface::StartExamination`: the examination panel opened (`context` is the
    /// panel's word, `gmExaminationUI` +0xB8 in Legions).
    pub fn start_examination(&mut self, context: u32) -> Vec<SessionEvent> {
        let out = self.script.start_examination(context);
        self.apply_script(out)
    }

    /// `gmCWInterface::StopExamination`: the examination panel closed.
    pub fn stop_examination(&mut self) -> Vec<SessionEvent> {
        let out = self.script.stop_examination();
        self.apply_script(out)
    }

    /// The say command (`CPlayer` 0xDB): `text` to everyone nearby.
    pub fn say(&mut self, text: &str) {
        if let Some(request) = crate::chat::say(text) {
            self.send(request.to_event());
        }
    }

    /// The tell command (`CCommunicationSystem` 0xB): `text` to the character named
    /// `recipient` on this server.
    pub fn tell(&mut self, recipient: &str, text: &str) {
        if let Some(request) = crate::chat::tell(recipient, text) {
            self.send(request.to_event());
        }
    }

    /// `gmCWInterface::DoEmote` once the emote table named the emote (`CPlayer` 0x15B ->
    /// `CCommunicationSystem` 0x8): play emote `emote`.
    pub fn emote(&mut self, emote: u32) {
        self.send(ac2_net::protocol::ScriptRequest::Emote(emote).to_event());
    }

    /// `gmCWInterface::RequestGUN_SendToRoom` on the Turbine chat path
    /// (`CWBookKeeper::CSendToTurbineRoomByID` -> `IChatClientImpl::SendToRoomByID` ->
    /// `Invoke` -> `ChatInterfaceProvider::SendData`): `text` to chat room `room`, with the
    /// script's chat flags in the `GunChatExtraInfo`. Returns the request's context id, or
    /// `None` for empty text or room 0 (retail sends nothing). The sender's meta letters
    /// (`StringTableMetaLanguage::RetrieveMetaLetters` of its name) are left empty.
    pub fn send_to_room(&mut self, room: u32, text: &str, weenie_flags: u32) -> Option<u32> {
        if text.is_empty() || room == 0 {
            return None;
        }
        self.chat_context = self.chat_context.wrapping_add(1);
        let request = ChatRequest::SendToRoomById {
            context: self.chat_context,
            room,
            text: text.to_owned(),
            extra: GunChatExtraInfo::new(weenie_flags, String::new()).encode(),
        };
        self.send_chat(&request.to_packet().encode());
        Some(self.chat_context)
    }

    /// `IChatClientImpl::OnNetworkData` for queue-4 chat data.
    pub(super) fn receive_chat(&mut self, data: Vec<u8>) -> Vec<SessionEvent> {
        let notice = match ChatMessage::decode(&data) {
            Ok(ChatMessage::RoomChat(chat)) => GameplayNotice::HearFromRoom {
                room: chat.room,
                speaker: chat.speaker,
                text: chat.text,
                extra: GunChatExtraInfo::decode(&chat.extra).ok(),
            },
            Ok(ChatMessage::Response(response)) => GameplayNotice::SendToRoomResult {
                context: response.context,
                result: response.result,
            },
            Err(_) => return vec![SessionEvent::Chat(data)],
        };
        vec![SessionEvent::Notice(notice)]
    }

    /// `Proto_UI::SendCharGenResult`: create the character the creation screens built
    /// (`gCharacterGenSystem` finishes with it); the answer is
    /// [`SessionEvent::CharGenVerification`].
    pub fn create_character(&mut self, result: ac2_net::protocol::CharGenResult) {
        let account = self.account.clone();
        self.send(ac2_net::protocol::ClientEvent::CharGenResult { account, result });
    }

    /// `Proto_UI::SendDeleteCharacter` (`gmCharacterSelectionUI`); the answer is
    /// [`SessionEvent::CharacterDeleted`].
    pub fn delete_character(&mut self, character: u64) {
        let account = self.account.clone();
        self.send(ac2_net::protocol::ClientEvent::DeleteCharacter { account, character });
    }

    /// `CCombatSystem` 0x1 as far as the client knows it: another object whose weenie is a
    /// creature (`CWInterface::IsCreature`). Retail also asks `CCombatSystem` 0x5 and the
    /// weenie type system.
    fn is_combat_target(&self, id: u64) -> bool {
        Some(id) != self.world.player()
            && self
                .world
                .objects
                .get(id)
                .and_then(|o| o.qualities.ints.get(&0x14))
                .is_some_and(|t| t & CREATURE_TYPE == CREATURE_TYPE)
    }

    /// A known `InterpCEvent` for the player's scripts; `None` when the function is not one
    /// the stand-in handles (the caller reports it as a raw script event).
    pub(super) fn receive_script(
        &mut self,
        function: u32,
        arguments: &[u8],
    ) -> Option<Vec<SessionEvent>> {
        match ScriptMessage::decode(function, arguments)? {
            Ok(message) => {
                let player = self.world.player().unwrap_or_default();
                let out = self.script.receive(message, player);
                Some(self.apply_script(out))
            }
            Err(error) => Some(vec![SessionEvent::Rejected(format!(
                "script function {function:#010x}: {error}"
            ))]),
        }
    }

    /// `GameplayOptionsProfile::SetShowPKDamage` through `CPlayer` 0x1A7: show PK damage and
    /// vigor loss over the vitals (health and vigor are raised again).
    pub fn set_show_pk_damage(&mut self, show: bool) -> Vec<SessionEvent> {
        let out = self
            .script
            .set_show_pk_damage(show, player_qualities(&self.world));
        self.apply_script(out)
    }

    /// `WM_Player` 0x27 -> `CPlayer` 0x65, the vitals part (the rest of its arguments reach the
    /// caller as the raw script event).
    pub(super) fn init_player(&mut self) -> Vec<SessionEvent> {
        let out = self.script.init_player(player_qualities(&self.world));
        self.apply_script(out)
    }

    /// `CWRepository`'s quality-change call into `CPlayer` 0x85 for the player.
    pub(super) fn player_quality_changed(&mut self, table: u32, quality: u32) -> Vec<SessionEvent> {
        let out = self
            .script
            .quality_changed(player_qualities(&self.world), table, quality);
        self.apply_script(out)
    }

    fn apply_script(&mut self, out: ScriptOutput) -> Vec<SessionEvent> {
        for request in out.requests {
            self.send(request.to_event());
        }
        out.notices.into_iter().map(SessionEvent::Notice).collect()
    }
}

/// The player's `CWRepository::m_qualities` (empty before its object exists).
fn player_qualities(world: &World) -> &WeenieQualities {
    static EMPTY: std::sync::OnceLock<WeenieQualities> = std::sync::OnceLock::new();
    world
        .player()
        .and_then(|id| world.objects.get(id))
        .map(|o| &o.qualities)
        .unwrap_or_else(|| EMPTY.get_or_init(WeenieQualities::default))
}
