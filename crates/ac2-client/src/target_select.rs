//! `gmSelectionUI::Select` on the frame: a target key (`gmSelectionUI::ListenToMessage`) or the
//! auto-target notice (`0x41000024`) moves the selection to the object `ac2_ui::select_target`
//! picks from the radar's blips. The types, selectability and owners come from the session, the
//! fellowship from the frame's progress. The clear key (`0x4000001E`) clears it.

use ac2_ui::Candidate;

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;

/// The clear key (`gmSelectionUI::Select` with `0x4000001E`).
const CLEAR_KEY: u32 = 0x4000_001E;

/// The command a target key asks for on this frame: the object it picks, or none to clear.
pub(crate) fn select_command(client: &Client, frame: &ClientFrame, key: u32) -> Option<Command> {
    if key == CLEAR_KEY {
        return Some(Command::Select(None));
    }
    let player = frame.player.as_ref()?.id;
    let selection = frame.selection.as_ref().map(|s| s.id);
    let range = frame.radar.range;
    let candidates = candidates(client, frame);
    let picked = ac2_ui::select_target(key, player, range * range, selection, &candidates)?;
    Some(Command::Select(Some(picked)))
}

/// The objects in the radar's range, as `Select` reads them: `CWRepository`'s type (`0x14`),
/// selectability (`0x270`) and owner (`0x1F9`), and the fellowship.
fn candidates(client: &Client, frame: &ClientFrame) -> Vec<Candidate> {
    let fellowship = frame.progress.fellowship.as_ref();
    let fellow = |id: u64| fellowship.is_some_and(|f| f.fellows.iter().any(|(m, _)| *m == id));
    let objects = client.session().objects();
    frame
        .radar
        .blips
        .iter()
        .filter(|b| b.in_range)
        .filter_map(|blip| {
            let qualities = &objects.get(blip.id)?.qualities;
            let kind = qualities.inq_int(0x14).unwrap_or(0) as u32;
            let owner = qualities.inq_instance_id(0x1F9);
            let is_fellow = fellow(blip.id);
            Some(Candidate {
                id: blip.id,
                kind,
                selectable: qualities.inq_bool(0x270).unwrap_or(0) != 0,
                owner,
                fellow: is_fellow,
                owner_fellow: owner.is_some_and(&fellow),
                // `CanAutoAttackTarget` is not modelled: a creature that is not a fellow.
                can_attack: kind & 0x4F == 0x4F && !is_fellow,
                offset: blip.offset,
            })
        })
        .collect()
}
