//! The HUD panels' requests as gameplay commands: the one place the frameworks' [`HudRequest`]s
//! (raised by `ac2_ui::message_pass` and its siblings) become [`Command`]s. The examination
//! requests open the examination panel ([`PassOut::open_examination`]), and help topics are kept
//! for the frame's `KeyStone::OpenHelp`.

use ac2_ui::{
    CharacterRequest, ConsignmentRequest, DialogRequest, ExaminationCall, ExternalContainerRequest,
    HudRequest, RadarRequest, SecureTradeRequest, SelectionCall, SkillRequest, SocialCall,
};

use crate::gameplay::Command;
use crate::hud_chat_options::chat_option_commands;
use crate::hud_craft::craft_command;
use crate::hud_ingredient::ingredient_command;
use crate::hud_pets::pet_commands;
use crate::hud_player::{examine, PassOut};
use crate::hud_shop::{catalog_commands, new_sale_commands, shop_commands};
use crate::ui_inventory::item_commands;

/// The panels' requests as the frame's commands, flags and help topics, in order.
pub(crate) fn hud_request_commands(requests: Vec<HudRequest>, out: &mut PassOut) {
    for request in requests {
        match request {
            HudRequest::SaveWindowLocations(saved) => {
                out.commands.push(Command::SaveWindowLocations(saved));
            }
            HudRequest::SetWindowOpacity { window, opacity } => {
                out.commands
                    .push(Command::SetWindowOpacity { window, opacity });
            }
            HudRequest::Dismount => out.commands.push(Command::Dismount),
            HudRequest::DisplaySaleReminders => out.commands.push(Command::DisplaySaleReminders),
            HudRequest::Selection(call) => match call {
                // `gmSelectionUI`: `SetSelection`, then `CWPhysicsObject::LookAtSelection`.
                SelectionCall::Select(id) => {
                    out.commands.push(Command::Select(id));
                    out.commands.push(Command::LookAtSelection(id));
                }
                SelectionCall::StopAttacking(target) => {
                    out.commands.push(Command::StopAttacking(target));
                }
            },
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
            // Run against the Tracker by `HudView::tracker_frame` before these.
            HudRequest::Tracker(_) | HudRequest::PointAt { .. } | HudRequest::TextPress(_) => {}
            HudRequest::Marker(request) => out.commands.push(Command::MapMarker(request)),
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
            HudRequest::Craft(request) => out.commands.push(craft_command(request)),
            HudRequest::Ingredient(request) => out.commands.push(ingredient_command(request)),
            HudRequest::StoryGameplay(shown) => out.story_gameplay.push(shown),
            HudRequest::GameplayShown(shown) => out.death_gameplay.push(shown),
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
            HudRequest::SecureTrade(request) => out.commands.push(secure_trade(request)),
            HudRequest::Shop(request) => out.commands.extend(shop_commands(vec![request])),
            HudRequest::Skill(request) => match request {
                SkillRequest::Examine { kind, data } => {
                    examine(kind, data, 0, &mut out.commands);
                    out.open_examination = true;
                }
                SkillRequest::Train(skill) => {
                    out.commands
                        .push(Command::Skill(ac2_weenie::SkillRequest::Train(skill)));
                }
                SkillRequest::Raise(skill) => {
                    out.commands
                        .push(Command::Skill(ac2_weenie::SkillRequest::Raise(skill)));
                }
                SkillRequest::Untrain(skill) => {
                    out.commands
                        .push(Command::Skill(ac2_weenie::SkillRequest::Untrain(skill)));
                }
                SkillRequest::CancelUntrain(_) => {
                    out.commands
                        .push(Command::Skill(ac2_weenie::SkillRequest::CancelUntrain));
                }
            },
            HudRequest::Examination(call) => out.commands.push(match call {
                ExaminationCall::Start(filter) => Command::StartExamination(filter),
                ExaminationCall::Stop => Command::StopExamination,
            }),
            HudRequest::Social(call) => match call {
                SocialCall::Social(request) => out.commands.push(Command::Social(request)),
                SocialCall::Select(id) => out.commands.push(Command::Select(Some(id))),
                SocialCall::AssistedTell(name) => out.tells.push(name),
            },
            HudRequest::Dialog(request) => out.commands.push(match request {
                DialogRequest::PopupAnswer {
                    object,
                    context,
                    yes,
                } => Command::PopupAnswer {
                    object,
                    context,
                    yes,
                },
                DialogRequest::ClearResurrection => Command::ClearResurrectionRequest,
                DialogRequest::RequestResurrect => Command::RequestResurrect,
            }),
            HudRequest::Quit => {
                // `gmGamePlayUI::RecvNotice_Logoff`: `UploadGameOptions`, then `Device::Done`.
                out.commands.push(Command::UploadGameOptions);
                out.quit = true;
            }
            HudRequest::LogOff => {
                out.commands.push(Command::UploadGameOptions);
                out.commands.push(Command::LogOffCharacter);
            }
        }
    }
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

/// A secure trade's call on the player (`gmCWInterface::*Trade*`).
fn secure_trade(request: SecureTradeRequest) -> Command {
    match request {
        SecureTradeRequest::Open { partner } => Command::OpenTrade(partner),
        SecureTradeRequest::Offer { item, quantity } => Command::OfferItemForTrade {
            item,
            amount: quantity,
        },
        SecureTradeRequest::Revoke { item } => Command::RevokeItemFromTrade(item),
        SecureTradeRequest::Accept => Command::AcceptTrade,
        SecureTradeRequest::Decline => Command::DeclineTrade,
        SecureTradeRequest::Close => Command::CloseTrade,
    }
}
