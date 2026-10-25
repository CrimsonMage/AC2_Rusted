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
    /// Runs the frame's commands; their notices join the frame's events.
    pub(crate) fn commands(&mut self, commands: &[Command]) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        for command in commands {
            events.extend(match command {
                Command::Select(id) => self.session.select(id.unwrap_or(NO_SELECTION)),
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
        }
    }

    /// The selection for [`crate::ClientFrame::selection`].
    pub(crate) fn selection_frame(&self) -> Option<SelectionFrame> {
        let script = self.session.player_script();
        let id = script.selection()?;
        Some(SelectionFrame {
            id,
            name: self.session.objects().get(id).and_then(|o| o.name.clone()),
            info: script
                .selection_info()
                .filter(|(info_id, _)| *info_id == id)
                .map(|(_, info)| info),
        })
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
