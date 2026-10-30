//! The selection keys and notices (`gmSelectionUI`): `ac2_ui::select_key_calls` and
//! `ac2_ui::frame_selection_calls` run them over ClientPlayer's `SelectionSource`; this file turns
//! their calls into commands. `pick_targets` answers the 3D selection's potential list for the
//! bin's own frame, which has no ClientPlayer.

use ac2_ui::PotentialTarget;

use crate::client::Client;
use crate::frame::ClientFrame;

/// The potential list of a 3D selection (nearest first), as `Finish3DSelection` reads its first
/// entry: each object's `0x270` selectability and wielder (`0x105`), the player's type (`0x14`,
/// 0 without a player) and the player's id (0 without one).
pub fn pick_targets(
    client: &Client,
    frame: &ClientFrame,
    potential: &[u64],
) -> (Vec<PotentialTarget>, u32, u64) {
    let objects = client.session().objects();
    let targets = potential
        .iter()
        .map(|&id| PotentialTarget {
            id,
            selectable: objects
                .get(id)
                .map(|object| object.qualities.inq_bool(0x270).unwrap_or(0) != 0),
            wielder: objects
                .get(id)
                .and_then(|object| object.qualities.inq_instance_id(0x105))
                .unwrap_or(0),
        })
        .collect();
    let player_type = frame
        .player
        .as_ref()
        .and_then(|player| objects.get(player.id))
        .and_then(|object| object.qualities.inq_int(0x14))
        .unwrap_or(0) as u32;
    let player_id = frame.player.as_ref().map_or(0, |player| player.id);
    (targets, player_type, player_id)
}
