//! `gmSelectionUI::Select` on the frame: a target key (`gmSelectionUI::ListenToMessage`) or the
//! auto-target notice (`0x41000024`) moves the selection to the object `ac2_ui::select_target`
//! picks from the radar's blips, and the reselect, last selection, last attacker and clear keys
//! answer from the selection state (`ac2_ui::direct_key`). The selectability, deletion and
//! last-attacker notices and `SetTargetingPointer` run their `gmSelectionUI` handlers over the
//! same state. The types, selectability and owners come from the session, the fellowship from
//! the frame's progress.

use ac2_ui::{Candidate, PotentialTarget, SelectionAction, SelectionSnapshot, UiTree};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use crate::frame::ClientFrame;
use crate::gameplay::Command;

/// The selection a key or notice starts from: the player, the selection, the last selection and
/// the last attacker. `None` without a player in the world.
fn snapshot(client: &Client, frame: &ClientFrame) -> Option<SelectionSnapshot> {
    let player = frame.player.as_ref()?.id;
    Some(SelectionSnapshot {
        player,
        selection: frame.selection.as_ref().map(|s| s.id),
        last_selection: client.session().player_script().last_selection(),
        last_attacker: client.session().progress().selection.last_attacker,
    })
}

/// The commands a target key asks for: `gmSelectionUI::Select` with `key`, from the selection as
/// it stands. `typing` is the input's text mode (the clear key's `ICIDM` check).
pub(crate) fn select_commands(
    client: &mut Client,
    frame: &ClientFrame,
    key: u32,
    typing: bool,
) -> Vec<Command> {
    match snapshot(client, frame) {
        Some(snapshot) => key_commands(client, frame, key, &snapshot, typing),
        None => Vec::new(),
    }
}

/// The commands one `Select` key makes from `snapshot`: the direct keys' selection, or the
/// cycling keys' pick (nothing when no candidate fits).
fn key_commands(
    client: &mut Client,
    frame: &ClientFrame,
    key: u32,
    snapshot: &SelectionSnapshot,
    typing: bool,
) -> Vec<Command> {
    if let Some(actions) = ac2_ui::direct_key(key, snapshot, typing) {
        return actions
            .into_iter()
            .filter_map(|action| match action {
                SelectionAction::Select(id) => Some(Command::Select(id)),
                _ => None,
            })
            .collect();
    }
    let range = frame.radar.range;
    let candidates = candidates(client, frame);
    ac2_ui::select_target(
        key,
        snapshot.player,
        range * range,
        snapshot.selection,
        &candidates,
    )
    .map(|picked| Command::Select(Some(picked)))
    .into_iter()
    .collect()
}

/// The frame's selection events, in order: `gmSelectionUI::RecvNotice_Select`, the
/// selectability, last attacker and targeting pointer notices, and `RecvNotice_BeingDeleted`
/// for each object the frame removed. `typing` is the input's text mode.
pub(crate) fn frame_notices(
    client: &mut Client,
    frame: &ClientFrame,
    tree: &mut UiTree,
    events: &[SessionEvent],
    typing: bool,
) -> Vec<Command> {
    let mut out = Vec::new();
    for event in events {
        match event {
            SessionEvent::ObjectRemoved(object) => {
                out.extend(object_removed(client, frame, *object, typing));
            }
            SessionEvent::Notice(GameplayNotice::Select(key)) => {
                out.extend(select_commands(client, frame, *key, typing));
            }
            SessionEvent::Notice(notice) => {
                out.extend(selection_notice(client, frame, tree, notice, typing));
            }
            _ => {}
        }
    }
    out
}

/// One selectability, last attacker or targeting pointer notice. Other notices make no commands.
fn selection_notice(
    client: &mut Client,
    frame: &ClientFrame,
    tree: &mut UiTree,
    notice: &GameplayNotice,
    typing: bool,
) -> Vec<Command> {
    match notice {
        GameplayNotice::SetTargetingPointer(on) => {
            ac2_ui::set_targeting_pointer(tree, *on);
            Vec::new()
        }
        GameplayNotice::UpdateSelectability { object, selectable } => {
            let Some(snapshot) = snapshot(client, frame) else {
                return Vec::new();
            };
            let auto = client.auto_target();
            let actions = ac2_ui::selectability_notice(*object, *selectable, &snapshot, auto);
            run(client, frame, &actions, typing)
        }
        GameplayNotice::LastAttacker { attacker, .. } => {
            let Some(snapshot) = snapshot(client, frame) else {
                return Vec::new();
            };
            let auto = client.auto_target();
            let has_selection = snapshot.selection.is_some();
            let actions = ac2_ui::last_attacker_notice(*attacker, auto, has_selection);
            run(client, frame, &actions, typing)
        }
        _ => Vec::new(),
    }
}

/// `gmSelectionUI::RecvNotice_BeingDeleted` for an object the frame removed.
fn object_removed(
    client: &mut Client,
    frame: &ClientFrame,
    object: u64,
    typing: bool,
) -> Vec<Command> {
    let Some(snapshot) = snapshot(client, frame) else {
        return Vec::new();
    };
    let auto = client.auto_target();
    let actions = ac2_ui::being_deleted(object, &snapshot, auto);
    run(client, frame, &actions, typing)
}

/// Carries out a notice's actions in order, the selection as each leaves it. `StopAttacking` is
/// `gmCWInterface::StopAttacking` (IFun 0xB40104) on the script VM.
fn run(
    client: &mut Client,
    frame: &ClientFrame,
    actions: &[SelectionAction],
    typing: bool,
) -> Vec<Command> {
    let Some(mut snapshot) = snapshot(client, frame) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for action in actions {
        match *action {
            SelectionAction::Select(id) => {
                out.push(Command::Select(id));
                snapshot.selection = id;
            }
            SelectionAction::StopAttacking(target) => out.push(Command::StopAttacking(target)),
            SelectionAction::Key(key) => {
                for command in key_commands(client, frame, key, &snapshot, typing) {
                    if let Command::Select(id) = command {
                        snapshot.selection = id;
                    }
                    out.push(command);
                }
            }
            SelectionAction::ClearIfSelected(object) => {
                if snapshot.selection == Some(object) {
                    out.push(Command::Select(None));
                    snapshot.selection = None;
                }
            }
        }
    }
    out
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

/// The objects in the radar's range, as `Select` reads them: `CWRepository`'s type (`0x14`),
/// selectability (`0x270`) and owner (`0x1F9`), and the fellowship.
fn candidates(client: &mut Client, frame: &ClientFrame) -> Vec<Candidate> {
    let fellowship = frame.progress.fellowship.as_ref();
    let fellow = |id: u64| fellowship.is_some_and(|f| f.fellows.iter().any(|(m, _)| *m == id));
    let out: Vec<Candidate> = {
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
                    // `CanAutoAttackTarget` (`Client::can_auto_attack_target`) is not asked here
                    // yet: the script answered no for a creature of the double. Until that is
                    // traced, a creature that is not a fellow.
                    can_attack: kind & 0x4F == 0x4F && !is_fellow,
                    offset: blip.offset,
                })
            })
            .collect()
    };
    out
}
