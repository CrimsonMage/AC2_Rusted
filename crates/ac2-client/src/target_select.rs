//! The selection keys and notices (`gmSelectionUI`): `ac2_ui::select_key_calls` and
//! `ac2_ui::frame_selection_calls` run them over ClientPlayer's `SelectionSource`; this file turns
//! their calls into commands. `pick_targets` answers the 3D selection's potential list for the
//! bin's own frame, which has no ClientPlayer.

use ac2_ui::{PotentialTarget, SelectionCall, UiTree};
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::ui_sources::ClientPlayer;

/// The frame's selection events, in order (see `ac2_ui::frame_selection_calls`).
pub(crate) fn frame_notices(
    player: &ClientPlayer<'_>,
    tree: &mut UiTree,
    events: &[SessionEvent],
    typing: bool,
) -> Vec<Command> {
    commands(ac2_ui::frame_selection_calls(player, tree, events, typing))
}

/// The client's commands for the selection's calls.
fn commands(calls: Vec<SelectionCall>) -> Vec<Command> {
    calls
        .into_iter()
        .map(|call| match call {
            SelectionCall::Select(id) => Command::Select(id),
            SelectionCall::StopAttacking(target) => Command::StopAttacking(target),
        })
        .collect()
}

/// The potential list of a 3D selection (nearest first), as `Finish3DSelection` reads its first
/// entry: each object's `0x270` selectability, and the player's type (`0x14`, 0 without a
/// player).
pub fn pick_targets(
    client: &Client,
    frame: &ClientFrame,
    potential: &[u64],
) -> (Vec<PotentialTarget>, u32) {
    let objects = client.session().objects();
    let targets = potential
        .iter()
        .map(|&id| PotentialTarget {
            id,
            selectable: objects
                .get(id)
                .map(|object| object.qualities.inq_bool(0x270).unwrap_or(0) != 0),
        })
        .collect();
    let player_type = frame
        .player
        .as_ref()
        .and_then(|player| objects.get(player.id))
        .and_then(|object| object.qualities.inq_int(0x14))
        .unwrap_or(0) as u32;
    (targets, player_type)
}
