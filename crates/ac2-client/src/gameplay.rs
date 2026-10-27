//! The UI's gameplay commands and the selection the selection panel shows.
//!
//! Retail's UI calls the player weenie's IFuns (`gmCWInterface::SetSelection`,
//! `SetExaminationRequest`, `StartExamination`, `StopExamination`, `DoEmote`) and its chat
//! commands (`CWInterface::ProcessText`); their notices come back as
//! [`SessionEvent::Notice`] in the frame's events (`CM_*::SendNotice_*`, which retail's UI
//! classes listen to).

use ac2_net::protocol::{ExaminationRequest, SelectionInfo};
use ac2_script::host::{IFunArg, OutValue};
use ac2_weenie::{SessionEvent, NO_SELECTION};

use crate::client::Client;

/// The selection action `gmShortcutUI` raises to auto-target before attacking.
const SELECT_AUTO_TARGET: u32 = 0x4100_0024;

/// A gameplay command from the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// `gmCWInterface::SetSelection`: select an object, or clear the selection.
    Select(Option<u64>),
    /// `gmCWInterface::SetExaminationRequest`.
    Examine(ExaminationRequest),
    /// `gmCWInterface::StartExamination` (the examination panel opened, with its word).
    StartExamination(u32),
    /// `gmCWInterface::StopExamination` (the panel closed).
    StopExamination,
    /// The say command: text to everyone nearby.
    Say(String),
    /// The tell command: text to the named character.
    Tell {
        recipient: String,
        text: String,
    },
    /// `gmCWInterface::DoEmote`, with the emote the emote table named.
    Emote(u32),
    /// `CWInterface::ProcessText`: a line the chat window entered (a `/` command or speech).
    ChatLine(String),
    /// `gmCWInterface::RequestGUN_SendToRoom` (Turbine chat): text to a chat room, with the
    /// script's chat flags.
    SendToRoom {
        room: u32,
        text: String,
        flags: u32,
    },
    /// Use an object (`CUsageSystem::HandleItemUsage`).
    Use(u64),
    /// Use an object on a target (a multi-item use once its target is chosen).
    UseWithTarget {
        item: u64,
        target: u64,
    },
    /// `gmCWInterface::UseSelection`: use the selected object.
    UseSelection,
    /// `gmCWInterface::MoveItem`: move `quantity` of `item` from slot `from_slot` of `from` to
    /// slot `target_slot` (-1: anywhere) of `target`; a drop targets the world, a give another
    /// creature, a split less than the whole stack.
    MoveItem {
        item: u64,
        from: u64,
        from_slot: i32,
        target: u64,
        target_slot: i32,
        quantity: i32,
    },
    /// `gmCWInterface::EquipItem`.
    EquipItem {
        item: u64,
        location: u32,
    },
    /// `gmCWInterface::UnEquipItem`: into slot `target_slot` of the player's pack.
    UnequipItem {
        item: u64,
        target_slot: i32,
    },
    /// `gmCWInterface::UpdateOpen`: open or close a container.
    SetContainerOpen {
        container: u64,
        open: bool,
    },
    /// `gmCWInterface::LootAllFromContainer`.
    LootAll(u64),
    /// `gmCWInterface::TransmuteAllFromContainer`.
    TransmuteAll(u64),
    /// `gmCWInterface::OpenTradeNegotiations`: ask `other` to trade.
    OpenTrade(u64),
    /// `gmCWInterface::CloseTradeNegotiations`.
    CloseTrade,
    /// `gmCWInterface::AcceptTrade`.
    AcceptTrade,
    /// `gmCWInterface::DeclineTrade`.
    DeclineTrade,
    /// `gmCWInterface::OfferItemForTrade`: `item`, `amount` of it.
    OfferItemForTrade {
        item: u64,
        amount: u32,
    },
    /// `gmCWInterface::RevokeItemFromTrade`.
    RevokeItemFromTrade(u64),
    /// `gmCWInterface::RequestEnterStore`: the store's DID. The VM's script sends the request.
    EnterStore(u32),
    /// `gmCWInterface::RequestLeaveStore`.
    LeaveStore,
    /// `gmCWInterface::RequestEnterConsignment` / `RequestLeaveConsignment`.
    EnterConsignment,
    LeaveConsignment,
    /// `gmCWInterface::PurchaseItem`: `quantity` of `item` from store slot `slot`.
    PurchaseItem {
        item: u64,
        quantity: u32,
        slot: u32,
    },
    /// `gmCWInterface::PurchaseConfirmed`: the player's answer to the purchase confirmation.
    PurchaseConfirmed(bool),
    /// `gmCWInterface::RequestSales(index)`: the store's page of sales from `index`.
    RequestSales(u32),
    /// `gmCWInterface::RequestNextSales` / `RequestPrevSales`: the next or previous page.
    NextSales,
    PrevSales,
    /// `gmCWInterface::RequestSell`: `quantity` of `item` at `price`.
    RequestSell {
        item: u64,
        quantity: u32,
        price: u32,
    },
    /// `gmCWInterface::OpenRecipe`: the craft panel opens recipe `recipe` (its ingredient panel;
    /// no request goes out).
    OpenRecipe(u32),
    /// `gmCWInterface::RequestAddIngredient`: `object` goes into slot `ordinal` of `recipe`.
    AddIngredient {
        recipe: u32,
        ordinal: u32,
        object: u64,
        spinner: u32,
    },
    /// `gmCWInterface::RequestRaiseCraftSkill`: raise craft skill `skill`.
    RaiseCraftSkill(u32),
    /// `gmCWInterface::RequestExecuteCraft`: craft `recipe` from `ingredients` (slot ordinal,
    /// object), at `spinner`.
    ExecuteCraft {
        recipe: u32,
        spinner: u32,
        ingredients: Vec<(u32, u64)>,
    },
    /// `gmCWInterface::RequestTrainSkill` / `RequestUntrainSkill` / `RequestRaiseSkill` /
    /// `RequestUntrainSkillCancel` (the skill panel and its confirmation dialogs).
    Skill(ac2_weenie::SkillRequest),
    /// `gmShortcutUI::RequestAttackStateChange` (the bar's attack buttons, key 0x41000013):
    /// start attacking the selection, or stop.
    SetAttacking(bool),
    /// `gmCWInterface::RequestResurrect`: the respawn request (the player's script's IFun
    /// 0xB4006B, which sends `WM_Death` 0x36 and so the server's lifestone teleport).
    RequestResurrect,
    /// `gmCWInterface::ClearResurrectionRequest`: the resurrection offer is declined (the
    /// player's script's IFun 0xB400F7).
    ClearResurrectionRequest,
    /// A fellowship or allegiance panel action (`gmCWInterface::DoFellowship*`,
    /// `SwearAllegiance`, `BreakAllegiance`, `AcceptVassal`, `DeclineVassal`,
    /// `QueryAllegianceProfile`).
    Social(ac2_weenie::SocialRequest),
    /// `gmCWInterface::SetWhichShortcutSet`: the bar shows set `n`.
    SetShortcutSet(u32),
    /// `gmCWInterface::SetShowRadarCoords` (the radar's coordinates box, `gmRadarUI`).
    SetShowRadarCoords(bool),
    /// One of the options page's setters (`gmCharacterUI::Apply`'s `gmCWInterface::Set*`).
    SetGameplayOption(ac2_ui::OptionSetting),
    /// `gmCWInterface::UploadGameOptions`: the options profile to the server.
    UploadGameOptions,
    /// One of the chat options page's setters (`gmCWInterface::SetChatFontSize` and the like).
    SetChatOption(ac2_ui::ChatOptionSetting),
    /// `gmCWInterface::UploadChatOptions`: the chat options to the server.
    UploadChatOptions,
    /// `gmCWInterface::SetShowObjectNames` (`gm3DTextUI::ToggleNames`).
    SetShowObjectNames(bool),
    /// `gmCWInterface::SetShortcut`: the shortcut into slot `index` of the bar.
    SetShortcut {
        index: u32,
        shortcut: ac2_net::protocol::ShortcutInfo,
    },
    /// `gmCWInterface::DragToMoneyBag`: `quantity` of `item` from slot `from_slot` of
    /// `from_container` into the money bag (`gmMoneyBagUI::HandleMoneyBagDrop`).
    DragToMoneyBag {
        item: u64,
        quantity: u32,
        from_slot: i32,
        from_container: u64,
    },
    /// `gmCWInterface::DragFromMoneyBag`: `quantity` of money out to slot `to_slot` of
    /// `to_container`.
    DragFromMoneyBag {
        quantity: u32,
        to_slot: i32,
        to_container: u64,
    },
}

/// What the advancement, shortcut, fellowship and allegiance panels read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProgressFrame {
    pub experience: ac2_weenie::Experience,
    pub skills: ac2_weenie::Skills,
    pub shortcuts: ac2_weenie::Shortcuts,
    pub fellowship: Option<ac2_weenie::FellowshipInfo>,
    pub allegiance: Option<ac2_weenie::AllegianceProfile>,
    /// The player is dead until the respawn (`UpdateDeathState`).
    pub dead: bool,
}

/// One container's contents, as the inventory panels show them (ids, in slot order; what each
/// item shows is in [`InventoryFrame::items`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerFrame {
    pub id: u64,
    pub packs: Vec<u64>,
    pub items: Vec<u64>,
    /// `gmCWInterface::GetMaxContainerSize`: how many slots it has.
    pub capacity: i32,
}

/// What `ItemDisplay::Update` and the paper doll show of one item.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemFrame {
    pub id: u64,
    pub stats: ac2_weenie::ItemStats,
    /// `GetPotentiallyPluralizedName` at the item's quantity: the slot's tooltip.
    pub name: ac2_net::protocol::StringInfo,
    /// The object's `VisualDesc` icon layers (layer id, image, shift colour); empty means the
    /// slot shows `stats.icon`.
    pub icon_layers: Vec<(u32, u32, u32)>,
}

/// The player's inventory and the containers it has open.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InventoryFrame {
    /// The player's own pack (the player is its container).
    pub pack: ContainerFrame,
    /// Other listed containers: the side packs and opened containers.
    pub containers: Vec<ContainerFrame>,
    /// Equipment location and item.
    pub equipped: Vec<(u32, u64)>,
    /// The containers open on the player (`gmExternalContainerUI`), in the order they opened.
    pub open: Vec<u64>,
    /// Every listed or equipped item the session knows, in listing order.
    pub items: Vec<ItemFrame>,
    /// The player's money (`gmMoneyBagUI`; `SendNotice_UpdateMoney` says when it changes).
    pub money: i32,
}

/// The selected object, as `gmSelectionUI` shows it.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectionFrame {
    pub id: u64,
    /// The weenie's name, when its description carries one.
    pub name: Option<String>,
    /// The server's latest `SelectionInfo` for it (health and vigor for a creature).
    pub info: Option<SelectionInfo>,
}

impl Client {
    /// `gmShortcutUI::RequestAttackStateChange`: before starting, `ShouldAutoTarget` may ask
    /// the UI to select a target (`SendNotice_Select(0x41000024)`, handled next frame here,
    /// so the attack goes to the selection as it was); then `RequestAttackStateChange`, on
    /// the script VM when it has the call, else the session's stand-in.
    /// `gmCWInterface::RequestResurrect`: the IFun on the player's script. Its server call and
    /// notices go out with the frame; without the script the request is not made.
    fn request_resurrect(&mut self) -> Vec<SessionEvent> {
        // The call's own failure (no script, a stopped VM) is in the scripts' problem list.
        let _ = self.call_ifun(crate::scripts_queries::ifun::REQUEST_RESURRECT, &[]);
        Vec::new()
    }

    /// `gmCWInterface::ClearResurrectionRequest`: the IFun on the player's script, as
    /// [`Client::request_resurrect`] makes its call.
    fn clear_resurrection_request(&mut self) -> Vec<SessionEvent> {
        let _ = self.call_ifun(
            crate::scripts_queries::ifun::CLEAR_RESURRECTION_REQUEST,
            &[],
        );
        Vec::new()
    }

    fn request_attack_state(&mut self, attacking: bool) -> Vec<SessionEvent> {
        use crate::scripts::{IFUN_REQUEST_ATTACK_STATE_CHANGE, IFUN_SHOULD_AUTO_TARGET};
        use ac2_script::host::IFunArg;
        if !self.scripts_own(IFUN_REQUEST_ATTACK_STATE_CHANGE) {
            self.session.request_attack_state(attacking);
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut problems = Vec::new();
        if attacking {
            match self.ifun_into(IFUN_SHOULD_AUTO_TARGET, &[], &mut out) {
                Ok(r) if r.retval != 0 => out.push(SessionEvent::Notice(
                    ac2_weenie::GameplayNotice::Select(SELECT_AUTO_TARGET),
                )),
                Ok(_) => {}
                Err(e) => problems.push(e),
            }
        }
        let args = [IFunArg::Int(u32::from(attacking))];
        if let Err(e) = self.ifun_into(IFUN_REQUEST_ATTACK_STATE_CHANGE, &args, &mut out) {
            problems.push(e);
        }
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.problems.extend(problems);
        }
        out
    }

    /// A fellowship or allegiance action: its `ggmCWInterface::CallIFun_*` on the script VM
    /// when it has the call (the arguments in the wrapper's parameter order:
    /// `DoFellowshipCreate(name, social, lootingMethod)`), else the session's stand-in.
    fn social_request(&mut self, request: &ac2_weenie::SocialRequest) -> Vec<SessionEvent> {
        use ac2_script::host::IFunArg;
        use ac2_weenie::SocialRequest as R;
        if !self.scripts_own(request.ifun()) {
            self.session.social_request(request);
            return Vec::new();
        }
        let args = match request {
            R::FellowshipCreate {
                name,
                social,
                looting,
            } => vec![
                IFunArg::Text(name.clone()),
                IFunArg::Int(*social),
                IFunArg::Int(*looting),
            ],
            R::FellowshipRecruit(id)
            | R::FellowshipDismiss(id)
            | R::BreakAllegiance(id)
            | R::QueryAllegianceProfile(id) => vec![IFunArg::Long(*id)],
            R::FellowshipRecruitDecline(context) | R::DeclineVassal(context) => {
                vec![IFunArg::Int(*context)]
            }
            _ => Vec::new(),
        };
        let mut out = Vec::new();
        if let Err(e) = self.ifun_into(request.ifun(), &args, &mut out) {
            if let Some(scripts) = self.scripts.as_mut() {
                scripts.problems.push(e);
            }
        }
        out
    }

    /// Runs the frame's commands; their notices join the frame's events.
    pub(crate) fn commands(&mut self, commands: &[Command]) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        for command in commands {
            events.extend(match command {
                Command::Select(id) => {
                    let id = id.unwrap_or(NO_SELECTION);
                    if self.scripts_own(crate::scripts::IFUN_SET_SELECTION) {
                        self.session.mirror_selection(id);
                        self.script_ifun(
                            crate::scripts::IFUN_SET_SELECTION,
                            &[ac2_net::protocol::PackValue::LongInt(id)],
                        )
                    } else {
                        self.session.select(id)
                    }
                }
                Command::Examine(request) => self.session.examine(*request),
                Command::StartExamination(context) => self.session.start_examination(*context),
                Command::StopExamination => self.session.stop_examination(),
                Command::Say(text) => {
                    self.session.say(text);
                    Vec::new()
                }
                Command::Tell { recipient, text } => {
                    self.session.tell(recipient, text);
                    Vec::new()
                }
                Command::ChatLine(line) => self.session.process_text(line),
                Command::SetAttacking(attacking) => self.request_attack_state(*attacking),
                Command::RequestResurrect => self.request_resurrect(),
                Command::ClearResurrectionRequest => self.clear_resurrection_request(),
                Command::Social(request) => self.social_request(request),
                Command::SetShortcutSet(set) => {
                    self.session.set_shortcut_set(*set);
                    if self.scripts_own(crate::scripts::IFUN_SET_WHICH_SHORTCUT_SET) {
                        self.script_ifun(
                            crate::scripts::IFUN_SET_WHICH_SHORTCUT_SET,
                            &[ac2_net::protocol::PackValue::Int(*set)],
                        )
                    } else {
                        Vec::new()
                    }
                }
                Command::SetShowObjectNames(show) => {
                    let ifun = crate::scripts::IFUN_SET_SHOW_OBJECT_NAMES;
                    if self.scripts_own(ifun) {
                        self.script_ifun(
                            ifun,
                            &[ac2_net::protocol::PackValue::Int(u32::from(*show))],
                        )
                    } else {
                        Vec::new()
                    }
                }
                Command::SetShowRadarCoords(show) => {
                    let ifun = crate::scripts::IFUN_SET_SHOW_RADAR_COORDS;
                    if self.scripts_own(ifun) {
                        self.script_ifun(
                            ifun,
                            &[ac2_net::protocol::PackValue::Int(u32::from(*show))],
                        )
                    } else {
                        Vec::new()
                    }
                }
                Command::SetGameplayOption(setting) => {
                    let (ifun, arg) = crate::scripts::option_call(setting);
                    if self.scripts_own(ifun) {
                        self.script_ifun(ifun, &[arg])
                    } else {
                        Vec::new()
                    }
                }
                Command::UploadGameOptions => {
                    let ifun = crate::scripts::IFUN_UPLOAD_GAME_OPTIONS;
                    if self.scripts_own(ifun) {
                        self.script_ifun(ifun, &[])
                    } else {
                        Vec::new()
                    }
                }
                Command::SetChatOption(setting) => {
                    let (ifun, args) = crate::scripts::chat_option_call(setting);
                    if self.scripts_own(ifun) {
                        self.script_ifun(ifun, &args)
                    } else {
                        Vec::new()
                    }
                }
                Command::UploadChatOptions => {
                    let ifun = crate::scripts::IFUN_UPLOAD_CHAT_OPTIONS;
                    if self.scripts_own(ifun) {
                        self.script_ifun(ifun, &[])
                    } else {
                        Vec::new()
                    }
                }
                Command::Skill(request) => self.skill_command(*request),
                Command::Emote(emote) => {
                    self.session.emote(*emote);
                    Vec::new()
                }
                Command::SendToRoom { room, text, flags } => {
                    self.session.send_to_room(*room, text, *flags);
                    Vec::new()
                }
                Command::Use(item) => {
                    self.session.use_object(*item);
                    Vec::new()
                }
                Command::UseWithTarget { item, target } => {
                    self.session.use_with_target(*item, *target);
                    Vec::new()
                }
                Command::UseSelection => {
                    // The IFun on the player's script: `CUsageSystem` 0x9 starts a targeted use
                    // (the targeting pointer, the usage list) and sends `WM_Usage` itself.
                    if self.scripts_own(crate::scripts::IFUN_USE_SELECTION) {
                        self.script_request(crate::scripts::IFUN_USE_SELECTION, &[])
                    } else {
                        self.session.use_selection()
                    }
                }
                Command::MoveItem {
                    item,
                    from,
                    from_slot,
                    target,
                    target_slot,
                    quantity,
                } => {
                    self.session.move_item(
                        *item,
                        *from,
                        *from_slot,
                        *target,
                        *target_slot,
                        *quantity,
                    );
                    Vec::new()
                }
                Command::EquipItem { item, location } => {
                    refused(self.session.equip_item(*item, *location))
                }
                Command::UnequipItem { item, target_slot } => {
                    refused(self.session.unequip_item(*item, *target_slot))
                }
                Command::SetContainerOpen { container, open } => {
                    self.session.set_container_open(*container, *open);
                    Vec::new()
                }
                Command::LootAll(container) => self.script_request(
                    crate::scripts::IFUN_LOOT_ALL_FROM_CONTAINER,
                    &[IFunArg::Long(*container)],
                ),
                Command::TransmuteAll(container) => {
                    self.session.transmute_all(*container);
                    Vec::new()
                }
                Command::OpenTrade(other) => {
                    self.session.open_trade(*other);
                    Vec::new()
                }
                Command::CloseTrade => {
                    self.session.close_trade();
                    Vec::new()
                }
                Command::AcceptTrade => {
                    // `gmCWInterface::AcceptTrade`: its IFun on the player's script, whose
                    // stub packs the accepting instance; else the session's stand-in.
                    if self.scripts_own(crate::scripts::IFUN_ACCEPT_TRADE) {
                        let mut out = Vec::new();
                        if let Err(e) =
                            self.ifun_into(crate::scripts::IFUN_ACCEPT_TRADE, &[], &mut out)
                        {
                            if let Some(scripts) = self.scripts.as_mut() {
                                scripts.problems.push(e);
                            }
                        }
                        out
                    } else {
                        self.session.accept_trade();
                        Vec::new()
                    }
                }
                Command::DeclineTrade => {
                    self.session.decline_trade();
                    Vec::new()
                }
                Command::OfferItemForTrade { item, amount } => {
                    self.session.offer_item_for_trade(*item, *amount);
                    Vec::new()
                }
                Command::RevokeItemFromTrade(item) => {
                    self.session.revoke_item_from_trade(*item);
                    Vec::new()
                }
                Command::EnterStore(store) => self.store_ifun(
                    crate::scripts::IFUN_REQUEST_ENTER_STORE,
                    &[IFunArg::Int(*store)],
                ),
                Command::LeaveStore => {
                    self.store_ifun(crate::scripts::IFUN_REQUEST_LEAVE_STORE, &[])
                }
                Command::EnterConsignment | Command::LeaveConsignment => {
                    self.consignment_command(command)
                }
                Command::PurchaseItem {
                    item,
                    quantity,
                    slot,
                } => self.store_ifun(
                    crate::scripts::IFUN_PURCHASE_ITEM,
                    &[
                        IFunArg::Long(*item),
                        IFunArg::Int(*quantity),
                        IFunArg::Int(*slot),
                    ],
                ),
                Command::PurchaseConfirmed(answer) => self.store_ifun(
                    crate::scripts::IFUN_PURCHASE_CONFIRMED,
                    &[IFunArg::Int(u32::from(*answer))],
                ),
                Command::RequestSales(index) => {
                    self.store_ifun(crate::scripts::IFUN_REQUEST_SALES, &[IFunArg::Int(*index)])
                }
                Command::NextSales => self.store_ifun(crate::scripts::IFUN_REQUEST_NEXT_SALES, &[]),
                Command::PrevSales => self.store_ifun(crate::scripts::IFUN_REQUEST_PREV_SALES, &[]),
                Command::RequestSell {
                    item,
                    quantity,
                    price,
                } => self.store_ifun(
                    crate::scripts::IFUN_REQUEST_SELL,
                    &[
                        IFunArg::Long(*item),
                        IFunArg::Int(*quantity),
                        IFunArg::Int(*price),
                    ],
                ),
                Command::OpenRecipe(_)
                | Command::AddIngredient { .. }
                | Command::ExecuteCraft { .. }
                | Command::RaiseCraftSkill(_) => self.craft_command(command),
                Command::DragToMoneyBag {
                    item,
                    quantity,
                    from_slot,
                    from_container,
                } => money(self.session.drag_to_money_bag(
                    *item,
                    *quantity,
                    *from_slot,
                    *from_container,
                )),
                Command::DragFromMoneyBag {
                    quantity,
                    to_slot,
                    to_container,
                } => money(
                    self.session
                        .drag_from_money_bag(*quantity, *to_slot, *to_container),
                ),
                Command::SetShortcut { index, shortcut } => refused(
                    self.session
                        .set_shortcut(*index, shortcut.clone())
                        .then_some(())
                        .ok_or_else(|| format!("no shortcut slot {index}")),
                ),
            });
        }
        events
    }

    /// The inventory for [`crate::ClientFrame::inventory`].
    pub(crate) fn inventory_frame(&self) -> InventoryFrame {
        let player = self.session.player().unwrap_or_default();
        let inv = self.session.inventory();
        let objects = self.session.objects();
        let stats = |id: u64| {
            objects
                .get(id)
                .map(|o| ac2_weenie::ItemStats::of(&o.qualities))
        };
        let capacity = |id| stats(id).map_or(0, |s| s.capacity);
        let frame = |(id, c): (u64, &ac2_weenie::Contents)| ContainerFrame {
            id,
            packs: c.packs.clone(),
            items: c.items.clone(),
            capacity: capacity(id),
        };
        let pack = inv
            .contents(player)
            .map(|c| frame((player, c)))
            .unwrap_or(ContainerFrame {
                id: player,
                capacity: capacity(player),
                ..ContainerFrame::default()
            });
        let containers: Vec<_> = inv
            .containers()
            .filter(|(id, _)| *id != player)
            .map(frame)
            .collect();
        let equipped = inv.equipped().to_vec();
        let tables = self.session.player_script().string_tables();
        let mut items: Vec<ItemFrame> = Vec::new();
        let listed = std::iter::once(&pack)
            .chain(&containers)
            .flat_map(|c| c.packs.iter().chain(&c.items))
            .chain(equipped.iter().map(|(_, id)| id));
        for &id in listed {
            let (Some(object), false) = (objects.get(id), items.iter().any(|i| i.id == id)) else {
                continue;
            };
            let stats = ac2_weenie::ItemStats::of(&object.qualities);
            items.push(ItemFrame {
                id,
                name: stats.display_name(stats.quantity as u32, tables),
                icon_layers: object.appearance.icon.clone(),
                stats,
            });
        }
        InventoryFrame {
            pack,
            containers,
            equipped,
            open: inv.open_containers().to_vec(),
            items,
            money: self.session.money(),
        }
    }

    /// The panels' state for [`crate::ClientFrame::progress`].
    pub(crate) fn progress_frame(&self) -> ProgressFrame {
        let progress = self.session.progress();
        let experience = self
            .session
            .player()
            .and_then(|id| self.session.objects().get(id))
            .map(|o| ac2_weenie::Experience::of(&o.qualities))
            .unwrap_or_default();
        ProgressFrame {
            experience,
            skills: progress.skills.clone(),
            shortcuts: progress.shortcuts.clone(),
            fellowship: progress.social.fellowship.clone(),
            allegiance: progress.social.allegiance.clone(),
            dead: progress.dead,
        }
    }

    /// The selection for [`crate::ClientFrame::selection`].
    pub(crate) fn selection_frame(&self) -> Option<SelectionFrame> {
        let selection = self.session.progress().selection;
        let id = selection.id?;
        Some(SelectionFrame {
            id,
            name: self.session.objects().get(id).and_then(|o| o.name.clone()),
            info: selection.info,
        })
    }

    /// The frame's notices as the UI sees them (`gmSelectionUI`, the social panels): kept in
    /// the session's panel state, whoever raised them.
    pub(crate) fn notices_seen(&mut self, events: &[SessionEvent]) {
        let notices: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                SessionEvent::Notice(n) => Some(n.clone()),
                _ => None,
            })
            .collect();
        self.session.apply_notices(&notices);
    }
}

/// A money bag request's notices, or its `gmWErrorType` as a rejection.
fn money(result: Result<Vec<SessionEvent>, u32>) -> Vec<SessionEvent> {
    result.unwrap_or_else(|error| vec![SessionEvent::Rejected(format!("money bag error {error}"))])
}

/// A command the scripts refuse before sending (retail logs it): a rejection event.
fn refused(result: Result<(), String>) -> Vec<SessionEvent> {
    result
        .err()
        .map(SessionEvent::Rejected)
        .into_iter()
        .collect()
}

impl Client {
    /// A store call: its IFun on the player's script, whose stubs send the request. The store has
    /// no session stand-in, so without the VM the call does nothing.
    pub(crate) fn store_ifun(&mut self, function: u32, args: &[IFunArg]) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        if self.scripts_own(function) {
            if let Err(e) = self.ifun_into(function, args, &mut out) {
                if let Some(scripts) = self.scripts.as_mut() {
                    scripts.problems.push(e);
                }
            }
        }
        out
    }

    /// `gmCWInterface::GetMaxPurchaseQuantity` (IFun 0xB4012D): the most of `item` the script
    /// lets the player buy. `None` without the VM, or when the call fails.
    pub fn max_purchase_quantity(&mut self, item: u64) -> Option<u32> {
        let function = crate::scripts::IFUN_GET_MAX_PURCHASE_QUANTITY;
        if !self.scripts_own(function) {
            return None;
        }
        let args = [IFunArg::Long(item), IFunArg::Out(OutValue::Word(0))];
        self.call_ifun(function, &args)
            .ok()
            .map(|result| result.out_word(0))
    }
}
