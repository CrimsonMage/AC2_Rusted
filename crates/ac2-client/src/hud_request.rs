//! The HUD panels' requests as gameplay commands: the one place the frameworks' [`HudRequest`]s
//! (raised by `ac2_ui::message_pass` and its siblings) become [`Command`]s. The examination
//! requests open the examination panel ([`PassOut::open_examination`]), and help topics are kept
//! for the frame's `KeyStone::OpenHelp`.

use ac2_ui::{
    CharacterRequest, ConsignmentRequest, ExternalContainerRequest, HudRequest, RadarRequest,
    SelectionCall,
};

use crate::gameplay::Command;
use crate::hud_chat_options::chat_option_commands;
use crate::hud_pets::pet_commands;
use crate::hud_player::{examine, PassOut};
use crate::hud_shop::{catalog_commands, new_sale_commands};
use crate::ui_inventory::item_commands;

/// The panels' requests as the frame's commands, flags and help topics, in order.
pub(crate) fn hud_request_commands(requests: Vec<HudRequest>, out: &mut PassOut) {
    for request in requests {
        match request {
            HudRequest::Dismount => out.commands.push(Command::Dismount),
            HudRequest::DisplaySaleReminders => out.commands.push(Command::DisplaySaleReminders),
            HudRequest::Selection(call) => out.commands.push(match call {
                SelectionCall::Select(id) => Command::Select(id),
                SelectionCall::StopAttacking(target) => Command::StopAttacking(target),
            }),
            HudRequest::Select(id) => out.commands.push(Command::Select(Some(id))),
            HudRequest::SetShowObjectNames(show) => {
                out.commands.push(Command::SetShowObjectNames(show));
            }
            HudRequest::Character(request) => out.commands.push(match request {
                CharacterRequest::Set(setting) => Command::SetGameplayOption(setting),
                CharacterRequest::Upload => Command::UploadGameOptions,
            }),
            HudRequest::ChatOption(request) => {
                out.commands.extend(chat_option_commands(vec![request]));
            }
            HudRequest::Pet(action) => pet_commands(vec![action], &mut out.commands),
            HudRequest::Friends(request) => out.commands.push(Command::Friends(request)),
            HudRequest::Radar(request) => out.commands.push(match request {
                RadarRequest::Select(id) => Command::Select(Some(id)),
                RadarRequest::ShowCoords(show) => Command::SetShowRadarCoords(show),
            }),
            HudRequest::Item(request) => {
                out.open_examination |= item_commands(request, &mut out.commands);
            }
            HudRequest::ExternalContainer(request) => external_container(request, out),
            HudRequest::NewSale(request) => {
                out.commands.extend(new_sale_commands(vec![request]));
            }
            HudRequest::Catalog(request) => {
                out.commands.extend(catalog_commands(vec![request]));
            }
            HudRequest::Consignment(request) => consignment(request, &mut out.commands),
            HudRequest::OpenExamination => out.open_examination = true,
            HudRequest::Examine { kind, data, item } => {
                examine(kind, data, item, &mut out.commands);
                out.open_examination = true;
            }
            HudRequest::OpenHelp(token) => out.help.push(token),
        }
    }
}

/// The requests as commands onto `commands`; whether the examination panel opens is or-ed into
/// `open_examination`. For the passes that keep their own command list.
pub(crate) fn hud_request_into(
    requests: Vec<HudRequest>,
    commands: &mut Vec<Command>,
    open_examination: &mut bool,
) {
    let mut out = PassOut::default();
    hud_request_commands(requests, &mut out);
    commands.extend(out.commands);
    *open_examination |= out.open_examination;
}

/// An external container's close, loot-all and transmute-all (its item requests are
/// [`HudRequest::Item`]).
fn external_container(request: ExternalContainerRequest, out: &mut PassOut) {
    match request {
        ExternalContainerRequest::Item(request) => {
            out.open_examination |= item_commands(request, &mut out.commands);
        }
        ExternalContainerRequest::Close(container) => {
            out.commands.push(Command::SetContainerOpen {
                container,
                open: false,
            });
        }
        ExternalContainerRequest::LootAll(container) => {
            out.commands.push(Command::LootAll(container));
        }
        ExternalContainerRequest::TransmuteAll(container) => {
            out.commands.push(Command::TransmuteAll(container));
        }
    }
}

/// A consignment request the panel left to the player (the new-sale dialog and the examination
/// are handled by `ac2_ui::consignment_requests`).
fn consignment(request: ConsignmentRequest, commands: &mut Vec<Command>) {
    match request {
        ConsignmentRequest::LeaveConsignment => commands.push(Command::LeaveConsignment),
        ConsignmentRequest::Select(item) => commands.push(Command::Select(Some(item))),
        ConsignmentRequest::Collect { target, profits } => {
            commands.push(Command::RequestCollect { target, profits });
        }
        ConsignmentRequest::Restock {
            item,
            quantity,
            target,
        } => commands.push(Command::RequestRestock {
            item,
            quantity,
            target,
        }),
        ConsignmentRequest::NewSale { .. } | ConsignmentRequest::OpenExamination => {}
    }
}
