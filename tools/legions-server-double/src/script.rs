//! The double's gameplay side: the script calls the client sends (`InterpSEvent`) and its chat
//! client's data (`0x100D3`), decoded with the client's own codecs and answered as a server's
//! `WM_*` handlers and chat server would, from data a test configures.

use std::collections::HashMap;

use ac2_net::protocol::{
    pack_id_list, script_broadcast, script_function, ChatMessage, ChatPacket, ChatRequest,
    ChatResponse, ContainerContents, Equipment, ExaminationProfile, ExaminationRequest,
    InstanceIdWithStamp, PackValue, QualityUpdate, RoomChat, ScriptCall, ScriptMessage,
    ScriptRequest, SelectionInfo, StringInfo,
};

use crate::{State, PLAYER};

/// The player's name in the double's character set.
const PLAYER_NAME: &str = "Explorer";

/// What the double answers script calls with.
#[derive(Debug, Clone, Default)]
pub struct ScriptWorld {
    /// `SelectionInfo` sent back (`WM_Player` 0x2C) when the client targets the object.
    pub selection_infos: HashMap<u64, SelectionInfo>,
    /// The examination profile sent back (`WM_Examination` 0x9) for an object request.
    pub profiles: HashMap<u64, ExaminationProfile>,
    /// The error a use comes back with (`WM_Usage` 0x21), 0 for success.
    pub usage_error: u32,
    /// The message a use comes back with, in place of the client's.
    pub usage_message: Option<StringInfo>,
    /// Containers' contents: packs and items (`WM_Inventory` 0x53).
    pub contents: std::collections::BTreeMap<u64, (Vec<u64>, Vec<u64>)>,
    /// The player's equipment by location (`WM_Inventory` 0x5F).
    pub equipped: Vec<(u32, u64)>,
}

impl State {
    /// An `InterpSEvent` (0x10053) or its encrypted twin (0x100D0) arrived: decode it, record
    /// it, answer it.
    pub(crate) fn script_event(&mut self, data: &[u8]) {
        let word = |at: usize| {
            data.get(at..at + 4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        };
        let (Some(function), Some(size)) = (word(4), word(8)) else {
            self.script_errors
                .push(format!("short script event {data:02x?}"));
            return;
        };
        let arguments = &data[12..];
        if arguments.len() != size as usize {
            self.script_errors.push(format!(
                "PackBuffer size {size} for {} bytes",
                arguments.len()
            ));
            return;
        }
        match ScriptRequest::decode(function, arguments) {
            Some(Ok(request)) => {
                self.answer_script(&request);
                self.script_requests.push(request);
            }
            Some(Err(error)) => self
                .script_errors
                .push(format!("function {function:#010x}: {error}")),
            None => match ScriptCall::decode(function, arguments) {
                Some(Ok(call)) => self.script_calls.push(call),
                Some(Err(error)) => self
                    .script_errors
                    .push(format!("function {function:#010x}: {error}")),
                None => self.script_unknown.push((function, arguments.to_vec())),
            },
        }
    }

    fn answer_script(&mut self, request: &ScriptRequest) {
        match request {
            ScriptRequest::SetTarget(id) => {
                if let Some(info) = self.script_world.selection_infos.get(id).copied() {
                    self.send_script(&ScriptMessage::UpdateSelectionInfo { info, id: *id });
                }
            }
            ScriptRequest::Examine(request) => {
                let object = matches!(
                    request.kind,
                    ExaminationRequest::OBJECT | ExaminationRequest::OBJECT_EXTENDED
                );
                if let Some(profile) = object
                    .then(|| self.script_world.profiles.get(&request.iid).cloned())
                    .flatten()
                {
                    let profile = ExaminationProfile {
                        request: *request,
                        ..profile
                    };
                    self.send_script(&ScriptMessage::ExaminationProfile(profile));
                }
            }
            // A server repeats speech to everyone in range, the speaker included.
            ScriptRequest::Say { text, .. } => self.send_script(&ScriptMessage::HearSpeech {
                kind: 0,
                text: text.clone(),
                speaker_name: name(PLAYER_NAME),
                speaker: PLAYER,
            }),
            // The only character the double knows is the player: a tell to it comes back.
            ScriptRequest::Tell {
                text, recipient, ..
            } => {
                if recipient.literal.as_deref() == Some(PLAYER_NAME) {
                    self.send_script(&ScriptMessage::HearTell {
                        kind: 0,
                        text: text.clone(),
                        teller_name: name(PLAYER_NAME),
                        teller: PLAYER,
                    });
                }
            }
            ScriptRequest::Emote(_) => {}
            ScriptRequest::MoveItem(desc) | ScriptRequest::ReorganizeContents(desc) => {
                let mut desc = desc.clone();
                let slot = self.inventory_move(desc.item(), desc.target(), desc.target_slot());
                desc.set_status(0);
                desc.set_actual_target_slot(slot);
                let done = match request {
                    ScriptRequest::MoveItem(_) => ScriptMessage::MoveItemDone(desc),
                    _ => ScriptMessage::ReorganizeContentsDone(desc),
                };
                self.send_script(&done);
            }
            ScriptRequest::EquipItem(desc) => {
                self.inventory_move(desc.item(), 0, -1);
                let world = &mut self.script_world;
                world
                    .equipped
                    .retain(|(l, i)| *l != desc.location() && *i != desc.item());
                world.equipped.push((desc.location(), desc.item()));
                self.send_script(&ScriptMessage::EquipItemDone(desc.clone()));
            }
            ScriptRequest::UnequipItem(desc) => {
                self.script_world
                    .equipped
                    .retain(|(_, i)| *i != desc.item());
                self.inventory_move(desc.item(), desc.container(), desc.slot());
                self.send_script(&ScriptMessage::UnequipItemDone(desc.clone()));
            }
            ScriptRequest::OpenContainer(id) => {
                if self.script_world.contents.contains_key(id) {
                    self.send_contents(*id);
                }
            }
            ScriptRequest::CloseContainer(_) => {}
            // `WM_Usage` 0x22: the desc comes back with the outcome the test configured.
            ScriptRequest::Use(desc) => {
                let mut desc = desc.clone();
                desc.error = self.script_world.usage_error;
                if let Some(message) = &self.script_world.usage_message {
                    desc.message = message.clone();
                }
                self.send_script(&ScriptMessage::UsageDone(desc));
            }
        }
    }

    /// Chat data from the client's chat client (`CChatEvent`, 0x100D3 on queue 4): record the
    /// request, answer it (result 0) and, as the chat server does for a room's members, send
    /// the message back to the player.
    pub(crate) fn chat_event(&mut self, data: &[u8]) {
        let Some(size) = data
            .get(4..8)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        else {
            self.script_errors.push("short chat event".into());
            return;
        };
        let request =
            ChatPacket::decode(&data[8..]).and_then(|packet| ChatRequest::from_packet(&packet));
        let request = match request {
            Ok(request) if size as usize == data.len() - 8 => request,
            other => {
                self.script_errors.push(format!("chat data: {other:?}"));
                return;
            }
        };
        if let ChatRequest::SendToRoomById {
            context,
            room,
            text,
            extra,
        } = &request
        {
            self.send_chat(&ChatMessage::Response(ChatResponse {
                context: *context,
                response_id: 0,
                method: ac2_net::protocol::METHOD_SEND_TO_ROOM_BY_ID,
                result: 0,
            }));
            self.send_chat(&ChatMessage::RoomChat(RoomChat {
                room: *room,
                speaker: PLAYER_NAME.into(),
                text: text.clone(),
                extra: extra.clone(),
            }));
        }
        self.chat_requests.push(request);
    }

    /// Queues chat-server data for the player (`0x100D3` on queue 4).
    pub fn send_chat(&mut self, message: &ChatMessage) {
        let packet = message.to_packet().encode();
        let mut data = 0x0001_00D3u32.to_le_bytes().to_vec();
        data.extend((packet.len() as u32).to_le_bytes());
        data.extend(packet);
        self.next_object_blob += 1;
        let id = 0x2300_0000_0000_0000 | u64::from(self.next_object_blob) << 32 | 0x10;
        self.reliable_blob(id, 4, &data);
    }

    /// Queues a script call to the player (`InterpCEvent_Broadcast`, 0x10066).
    pub fn send_script(&mut self, message: &ScriptMessage) {
        self.send_event(&message.to_broadcast());
    }

    /// Queues a script call the client's codecs have no message for, from its packed values.
    pub fn send_script_call(&mut self, function: u32, arguments: &[PackValue]) {
        self.send_event(&script_broadcast(function, arguments));
    }

    /// Moves `item` out of whichever container holds it and, when `target` is a known
    /// container, into it at `slot` (-1 or past the end: last); returns the slot it took.
    fn inventory_move(&mut self, item: u64, target: u64, slot: i32) -> i32 {
        let world = &mut self.script_world;
        let mut is_pack = false;
        for (packs, items) in world.contents.values_mut() {
            is_pack |= packs.contains(&item);
            packs.retain(|i| *i != item);
            items.retain(|i| *i != item);
        }
        let Some((packs, items)) = world.contents.get_mut(&target) else {
            return -1;
        };
        let list = if is_pack { packs } else { items };
        let at = usize::try_from(slot).map_or(list.len(), |s| s.min(list.len()));
        list.insert(at, item);
        at as i32
    }

    /// Queues `container`'s contents (`WM_Inventory` 0x53): to the player as a broadcast, to
    /// another container as a cell event for that object.
    pub fn send_contents(&mut self, container: u64) {
        let (packs, items) = self
            .script_world
            .contents
            .get(&container)
            .cloned()
            .unwrap_or_default();
        let message = ScriptMessage::ContainerContents(ContainerContents {
            segments: pack_id_list(&[]),
            packs,
            items,
        });
        if container == PLAYER {
            self.send_script(&message);
        } else {
            self.send_event(&message.to_cell(InstanceIdWithStamp {
                id: container,
                instance: 1,
                other: 0,
            }));
        }
    }

    /// Queues the player's equipment (`WM_Inventory` 0x5F).
    pub fn send_equipment(&mut self) {
        let equipment = Equipment::from_locations(&self.script_world.equipped);
        self.send_script(&ScriptMessage::Equipment(equipment));
    }

    /// Queues a quality update (`ECM_Qualities::SendEvt_Update*_Private` / `_Visual`).
    pub fn send_quality(&mut self, update: &QualityUpdate) {
        self.send_event(&update.encode());
    }

    /// Queues `WM_Player` 0x27 (the player's login state, `CPlayer` 0x65). The double has no
    /// player systems to pack, so its fourteen arguments are zero ints.
    pub fn send_init_player(&mut self) {
        self.send_script_call(script_function::INIT_PLAYER, &vec![PackValue::Int(0); 14]);
    }
}

fn name(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}
