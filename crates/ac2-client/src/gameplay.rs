//! The UI's gameplay commands and the selection the selection panel shows.
//!
//! Retail's UI calls the player weenie's IFuns (`gmCWInterface::SetSelection`,
//! `SetExaminationRequest`, `StartExamination`, `StopExamination`, `DoEmote`) and its chat
//! commands (`CWInterface::ProcessText`); their notices come back as
//! [`SessionEvent::Notice`] in the frame's events (`CM_*::SendNotice_*`, which retail's UI
//! classes listen to).

use ac2_net::protocol::{ExaminationRequest, SelectionInfo};
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
    Tell { recipient: String, text: String },
    /// `gmCWInterface::DoEmote`, with the emote the emote table named.
    Emote(u32),
    /// `CWInterface::ProcessText`: a line the chat window entered (a `/` command or speech).
    ChatLine(String),
    /// `gmCWInterface::RequestGUN_SendToRoom` (Turbine chat): text to a chat room, with the
    /// script's chat flags.
    SendToRoom { room: u32, text: String, flags: u32 },
    /// Use an object (`CUsageSystem::HandleItemUsage`).
    Use(u64),
    /// Use an object on a target (a multi-item use once its target is chosen).
    UseWithTarget { item: u64, target: u64 },
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
    EquipItem { item: u64, location: u32 },
    /// `gmCWInterface::UnEquipItem`: into slot `target_slot` of the player's pack.
    UnequipItem { item: u64, target_slot: i32 },
    /// `gmCWInterface::UpdateOpen`: open or close a container.
    SetContainerOpen { container: u64, open: bool },
    /// `gmCWInterface::LootAllFromContainer`.
    LootAll(u64),
    /// `gmCWInterface::TransmuteAllFromContainer`.
    TransmuteAll(u64),
    /// `gmCWInterface::RequestTrainSkill` / `RequestUntrainSkill` / `RequestRaiseSkill` /
    /// `RequestUntrainSkillCancel` (the skill panel and its confirmation dialogs).
    Skill(ac2_weenie::SkillRequest),
    /// `gmShortcutUI::RequestAttackStateChange` (the bar's attack buttons, key 0x41000013):
    /// start attacking the selection, or stop.
    SetAttacking(bool),
    /// `gmCWInterface::SetWhichShortcutSet`: the bar shows set `n`.
    SetShortcutSet(u32),
    /// `gmCWInterface::SetShortcut`: the shortcut into slot `index` of the bar.
    SetShortcut {
        index: u32,
        shortcut: ac2_net::protocol::ShortcutInfo,
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
}

/// One container's contents, as the inventory panels show them (ids; names, icons and
/// quantities come from the frame's objects).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerFrame {
    pub id: u64,
    pub packs: Vec<u64>,
    pub items: Vec<u64>,
}

/// The player's inventory and the containers it has open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InventoryFrame {
    /// The player's own pack (the player is its container).
    pub pack: ContainerFrame,
    /// Other listed containers: the side packs and opened containers.
    pub containers: Vec<ContainerFrame>,
    /// Equipment location and item.
    pub equipped: Vec<(u32, u64)>,
    /// The containers open on the player (`gmExternalContainerUI`), in the order they opened.
    pub open: Vec<u64>,
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
                Command::Skill(request) => {
                    self.session.request_skill(*request);
                    Vec::new()
                }
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
                Command::UseSelection => self.session.use_selection(),
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
                Command::LootAll(container) => {
                    self.session.loot_all(*container);
                    Vec::new()
                }
                Command::TransmuteAll(container) => {
                    self.session.transmute_all(*container);
                    Vec::new()
                }
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
        let frame = |(id, c): (u64, &ac2_weenie::Contents)| ContainerFrame {
            id,
            packs: c.packs.clone(),
            items: c.items.clone(),
        };
        InventoryFrame {
            pack: inv
                .contents(player)
                .map(|c| frame((player, c)))
                .unwrap_or(ContainerFrame {
                    id: player,
                    ..ContainerFrame::default()
                }),
            containers: inv
                .containers()
                .filter(|(id, _)| *id != player)
                .map(frame)
                .collect(),
            equipped: inv.equipped().to_vec(),
            open: inv.open_containers().to_vec(),
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

/// A command the scripts refuse before sending (retail logs it): a rejection event.
fn refused(result: Result<(), String>) -> Vec<SessionEvent> {
    result
        .err()
        .map(SessionEvent::Rejected)
        .into_iter()
        .collect()
}
