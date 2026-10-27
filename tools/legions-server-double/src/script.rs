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

use crate::State;

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
    /// Containers' slots, for the segment each listing carries (12, `gmEntity` 0xC's
    /// default, for one not here).
    pub capacities: std::collections::BTreeMap<u64, u32>,
    /// The effects on the player, carried by the login message (`WM_Player` 0x27 argument 6).
    pub effects: Vec<crate::EffectGrant>,
    /// The options profile `WM_Player` 0x27 carries (argument 4: shortcuts and the rest);
    /// `None`: [`crate::default_options`].
    pub options: Option<ac2_net::protocol::GameplayOptionsProfile>,
    /// The skills the login message carries.
    pub character: crate::Character,
    /// The store the server opened (`open_store`): the sales a purchase is priced from.
    pub open_store: Option<ac2_net::protocol::StoreView>,
    /// Purchases made so far: the item each one gives the player is counted from them.
    pub purchases: u64,
    /// Crafts executed so far: the crafted item each one gives the player is counted from them.
    pub crafts: u64,
    /// The owner of each corpse (its container); a corpse opens for its owner only.
    pub corpse_owners: std::collections::BTreeMap<u64, u64>,
    /// The corpses butchered so far (`Death_IsBeingButchered`).
    pub butchered: std::collections::BTreeSet<u64>,
    /// Whether the corpse has no skin to take (an empty butchery profile).
    pub corpse_skinless: bool,
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
        self.script_bytes.push((function, arguments.to_vec()));
        match ScriptRequest::decode(function, arguments) {
            Some(Ok(request)) => {
                self.answer_script(&request);
                self.script_requests.push(request);
            }
            Some(Err(error)) => self
                .script_errors
                .push(format!("function {function:#010x}: {error}")),
            None => match ScriptCall::decode(function, arguments) {
                Some(Ok(call)) => {
                    self.answer_combat(&call);
                    self.answer_call(&call);
                    self.script_calls.push(call);
                }
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
                speaker: self.player(),
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
                        teller: self.player(),
                    });
                }
            }
            // Emotes play on the server; the double has no one to show them to.
            ScriptRequest::Emote(_) | ScriptRequest::EmoteText(_) => {}
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
            // `WM_Inventory` 0x5D: the container's lists, or an error. An unknown container is
            // refused with error 1 (its code is not traced); another character's corpse with
            // `NOT_MINE` (`gmWErrorType::NotMine`, the open answer's error branch in the wlib). A
            // refused container lists nothing (the answer keeps its reference arguments).
            ScriptRequest::OpenContainer(id) => {
                let owned = self.may_loot(*id);
                let listed = self.script_world.contents.get(id).cloned();
                let error = match (&listed, owned) {
                    (None, _) => 1,
                    (Some(_), false) => crate::corpse::NOT_MINE,
                    (Some(_), true) => 0,
                };
                let (packs, items) = match listed {
                    Some(l) if owned => (l.0, l.1),
                    _ => (Vec::new(), Vec::new()),
                };
                self.send_script(&ScriptMessage::OpenContainerDone {
                    error,
                    packs: Some(packs),
                    items: Some(items),
                    container: *id,
                });
            }
            ScriptRequest::CloseContainer(id) => {
                self.send_script(&ScriptMessage::CloseContainerDone {
                    error: 0,
                    container: *id,
                });
            }
            // A take-all of another character's corpse moves nothing (the wlib asks only for a
            // corpse whose lists it has counted, and a refused corpse lists nothing).
            ScriptRequest::LootAll(desc) => {
                if self.may_loot(desc.container()) {
                    let player = self.player();
                    let world = &mut self.script_world;
                    let taken = world
                        .contents
                        .get_mut(&desc.container())
                        .map(|c| std::mem::take(&mut c.1))
                        .unwrap_or_default();
                    if let Some(pack) = world.contents.get_mut(&player) {
                        pack.1.extend(taken);
                    }
                }
                self.send_script(&ScriptMessage::LootAllDone(desc.clone()));
            }
            ScriptRequest::TransmuteAll(desc) => {
                if let Some(c) = self.script_world.contents.get_mut(&desc.container()) {
                    c.1.clear();
                }
                self.send_script(&ScriptMessage::TransmuteAllDone(desc.clone()));
            }
            // `WM_Usage` 0x22: the desc comes back with the outcome the test configured.
            ScriptRequest::OpenTrade(_)
            | ScriptRequest::CloseTrade
            | ScriptRequest::AcceptTrade(_)
            | ScriptRequest::DeclineTrade
            | ScriptRequest::OfferItem { .. }
            | ScriptRequest::RevokeItem(_) => self.answer_trade(request),
            ScriptRequest::LeaveStore(_) => self.answer_leave_store(request),
            ScriptRequest::Purchase(blob) => self.answer_purchase(blob),
            ScriptRequest::CraftAddIngredient { .. } | ScriptRequest::CraftExecute { .. } => {
                self.answer_craft(request)
            }
            ScriptRequest::CraftRaiseSkill(_) => {}
            ScriptRequest::EnterStore { .. } => {}
            // The consignment's enter and leave have no traced answer yet (see the catalog).
            ScriptRequest::EnterConsignment(_) | ScriptRequest::LeaveConsignment(_) => {}
            // The paging, restock, collect and leave-catalog requests have no traced answer yet: the
            // sales and consignment notices come back from the store's own forwarders.
            ScriptRequest::StoreSales { .. }
            | ScriptRequest::NextSales(_)
            | ScriptRequest::PrevSales(_)
            | ScriptRequest::LeaveCatalog(_)
            | ScriptRequest::Restock { .. }
            | ScriptRequest::Collect { .. } => {}
            ScriptRequest::Use(desc) if desc.item == crate::butcher::BUTCHER_KNIFE => {
                self.answer_butcher(desc);
            }
            ScriptRequest::Use(desc) if desc.item == crate::world_objects::GROUND_ITEM => {
                self.answer_pickup(desc);
            }
            ScriptRequest::Use(desc) if desc.item == crate::world_objects::DOOR => {
                self.answer_door(desc);
            }
            ScriptRequest::Use(desc) if desc.item == crate::world_objects::LEVER => {
                self.answer_lever(desc);
            }
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
        let event = script_broadcast(function, arguments);
        // `NetBlob::Fragmentize`: a call longer than one fragment goes out in several.
        if event.len() > ac2_net::FRAGMENT_DATA {
            self.send_fragmented(&event);
        } else {
            self.send_event(&event);
        }
    }

    /// Moves `item` out of whichever container holds it and, when `target` is a known
    /// container, into it at `slot` (-1 or past the end: last); returns the slot it took.
    pub(crate) fn inventory_move(&mut self, item: u64, target: u64, slot: i32) -> i32 {
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
            segments: self.segments_of(container, items.len()),
            packs,
            items,
        });
        if container == self.player() {
            self.send_script(&message);
        } else {
            self.send_event(&message.to_cell(InstanceIdWithStamp {
                id: container,
                instance: 1,
                other: 0,
            }));
        }
    }

    /// `container`'s one segment: its slots and `items` items.
    fn segments_of(&self, container: u64, items: usize) -> ac2_net::protocol::PackedReference {
        let capacity = self.script_world.capacities.get(&container).copied();
        crate::items::segments(capacity.unwrap_or(12), items)
    }

    /// Queues the player's equipment (`WM_Inventory` 0x5F).
    pub fn send_equipment(&mut self) {
        let equipment = Equipment::from_locations(&self.script_world.equipped);
        self.send_script(&ScriptMessage::Equipment(equipment));
    }

    /// Queues the world-name event (`ECM_Login::SendNotice_WorldName`'s wire form) with `name`
    /// as its literal.
    pub fn send_world_name(&mut self, name: &str) {
        let info = ac2_net::protocol::StringInfo {
            literal: Some(name.into()),
            ..ac2_net::protocol::StringInfo::default()
        };
        self.send_event(&info.world_name_event());
    }

    /// Queues a quality update (`ECM_Qualities::SendEvt_Update*_Private` / `_Visual`).
    pub fn send_quality(&mut self, update: &QualityUpdate) {
        self.send_event(&update.encode());
    }

    /// Queues `WM_Player` 0x27 (the player's login state, `CPlayer` 0x65): the skill
    /// repository, the options profile, the equipment and the player's contents from
    /// [`ScriptWorld`]; the double has no other player systems, so the remaining arguments are
    /// zero ints.
    pub fn send_init_player(&mut self) {
        let mut args = vec![PackValue::Int(0); 14];
        // Argument 5: the skill repository (`Agent` 0x6C), which the login's skill panel reads.
        args[4] = PackValue::Reference(self.script_world.character.repository());
        // The player's options profile always comes (the double's character's when the test
        // set none): `CPlayer` 0x65 dereferences it.
        let options = self
            .script_world
            .options
            .clone()
            .unwrap_or_else(crate::default_options);
        args[3] = PackValue::native(ac2_net::protocol::NativeObject::GameplayOptionsProfile(
            Box::new(options),
        ));
        // Argument 6: the effect registry (`CEffectSystem` 0x2; `gmEntity` 0x12B makes one only
        // for a null slot, so it always comes).
        args[5] = PackValue::Reference(crate::effect_registry(&self.script_world.effects));
        // Arguments 7-9: the equipment (`CInventorySystem` 0x6); 10-12: the player's contents
        // (`CInventorySystem` 0x1A).
        let equipment = Equipment::from_locations(&self.script_world.equipped);
        args[6] = equipment.first;
        args[7] = PackValue::Reference(equipment.by_location);
        args[8] = PackValue::Reference(equipment.by_item);
        let (packs, items) = self
            .script_world
            .contents
            .get(&self.player())
            .cloned()
            .unwrap_or_default();
        args[9] = PackValue::Reference(self.segments_of(self.player(), items.len()));
        args[10] = PackValue::Reference(pack_id_list(&packs));
        args[11] = PackValue::Reference(pack_id_list(&items));
        self.send_script_call(script_function::INIT_PLAYER, &args);
    }
}

fn name(text: &str) -> StringInfo {
    StringInfo {
        literal: Some(text.into()),
        ..StringInfo::default()
    }
}
