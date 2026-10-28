//! `gmSelectionUI::Select` on the frame: a target key (`gmSelectionUI::ListenToMessage`) or the
//! auto-target notice (`0x41000024`) moves the selection to the object `select_target` picks from
//! the radar's blips, and the reselect, last selection, last attacker and clear keys answer from
//! the selection state (`direct_key`). The selectability, deletion and last-attacker notices run
//! their `gmSelectionUI` handlers over the same state. The player answers through
//! `SelectionSource`; the calls come back as `SelectionCall`s for the client to run.

use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::element::UiTree;
use crate::radar_state::RadarBlip;
use crate::selection_notices::{
    being_deleted, direct_key, last_attacker_notice, selectability_notice, set_targeting_pointer,
    SelectionAction, SelectionSnapshot,
};
use crate::target_cycle::{select_target, Candidate};

/// What the selection keys and notices read from the player: the selection state, the auto-target
/// option, the radar, and the objects the session holds.
pub trait SelectionSource {
    /// The selection a key or notice starts from: the player, the selection, the last selection
    /// and the last attacker. `None` without a player in the world.
    fn snapshot(&self) -> Option<SelectionSnapshot>;
    /// The auto-target option (`gmCWInterface::AutoTarget`).
    fn auto_target(&self) -> bool;
    /// The range the radar shows where the player stands.
    fn radar_range(&self) -> f32;
    /// The radar's blips, in the radar's order.
    fn radar_blips(&self) -> &[RadarBlip];
    /// A live object's weenie type, selectability and owner; `None` when the session lacks it.
    fn object(&self, id: u64) -> Option<SelectionObject>;
    /// Whether `id` is a member of the player's fellowship.
    fn is_fellow(&self, id: u64) -> bool;
}

/// The weenie qualities the selection reads from an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionObject {
    /// `CWRepository::GetInt(0x14)`: the weenie type bits.
    pub kind: u32,
    /// `CWRepository::GetBool(0x270)`: whether the object can be selected.
    pub selectable: bool,
    /// `InqInstanceID(0x1F9)`: the object's owner, when it has one.
    pub owner: Option<u64>,
}

/// A selection the frame asks the client to carry out: the session's `SetSelection`, and the
/// `gmCWInterface::StopAttacking` on a target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionCall {
    Select(Option<u64>),
    StopAttacking(u64),
}

/// The commands a target key asks for: `gmSelectionUI::Select` with `key`, from the selection as
/// it stands. `typing` is the input's text mode (the clear key's `ICIDM` check).
pub fn select_key_calls<S: SelectionSource>(
    source: &S,
    key: u32,
    typing: bool,
) -> Vec<SelectionCall> {
    match source.snapshot() {
        Some(snapshot) => key_calls(source, key, &snapshot, typing),
        None => Vec::new(),
    }
}

/// The calls one `Select` key makes from `snapshot`: the direct keys' selection, or the cycling
/// keys' pick (nothing when no candidate fits).
fn key_calls<S: SelectionSource>(
    source: &S,
    key: u32,
    snapshot: &SelectionSnapshot,
    typing: bool,
) -> Vec<SelectionCall> {
    if let Some(actions) = direct_key(key, snapshot, typing) {
        return actions
            .into_iter()
            .filter_map(|action| match action {
                SelectionAction::Select(id) => Some(SelectionCall::Select(id)),
                _ => None,
            })
            .collect();
    }
    let range = source.radar_range();
    let candidates = candidates(source);
    select_target(
        key,
        snapshot.player,
        range * range,
        snapshot.selection,
        &candidates,
    )
    .map(|picked| SelectionCall::Select(Some(picked)))
    .into_iter()
    .collect()
}

/// The frame's selection events, in order: `gmSelectionUI::RecvNotice_Select`, the
/// selectability, last attacker and targeting pointer notices, and `RecvNotice_BeingDeleted`
/// for each object the frame removed. `typing` is the input's text mode.
pub fn frame_selection_calls<S: SelectionSource>(
    source: &S,
    tree: &mut UiTree,
    events: &[SessionEvent],
    typing: bool,
) -> Vec<SelectionCall> {
    let mut out = Vec::new();
    for event in events {
        match event {
            SessionEvent::ObjectRemoved(object) => {
                out.extend(object_removed(source, *object, typing));
            }
            SessionEvent::Notice(GameplayNotice::Select(key)) => {
                out.extend(select_key_calls(source, *key, typing));
            }
            SessionEvent::Notice(notice) => {
                out.extend(selection_notice(source, tree, notice, typing));
            }
            _ => {}
        }
    }
    out
}

/// One selectability, last attacker or targeting pointer notice. Other notices make no calls.
fn selection_notice<S: SelectionSource>(
    source: &S,
    tree: &mut UiTree,
    notice: &GameplayNotice,
    typing: bool,
) -> Vec<SelectionCall> {
    match notice {
        GameplayNotice::SetTargetingPointer(on) => {
            set_targeting_pointer(tree, *on);
            Vec::new()
        }
        GameplayNotice::UpdateSelectability { object, selectable } => {
            let Some(snapshot) = source.snapshot() else {
                return Vec::new();
            };
            let actions =
                selectability_notice(*object, *selectable, &snapshot, source.auto_target());
            run(source, &actions, typing)
        }
        GameplayNotice::LastAttacker { attacker, .. } => {
            let Some(snapshot) = source.snapshot() else {
                return Vec::new();
            };
            let has_selection = snapshot.selection.is_some();
            let actions = last_attacker_notice(*attacker, source.auto_target(), has_selection);
            run(source, &actions, typing)
        }
        _ => Vec::new(),
    }
}

/// `gmSelectionUI::RecvNotice_BeingDeleted` for an object the frame removed.
fn object_removed<S: SelectionSource>(source: &S, object: u64, typing: bool) -> Vec<SelectionCall> {
    let Some(snapshot) = source.snapshot() else {
        return Vec::new();
    };
    let actions = being_deleted(object, &snapshot, source.auto_target());
    run(source, &actions, typing)
}

/// Carries out a notice's actions in order, the selection as each leaves it. `StopAttacking` is
/// `gmCWInterface::StopAttacking` (IFun 0xB40104) on the script VM.
fn run<S: SelectionSource>(
    source: &S,
    actions: &[SelectionAction],
    typing: bool,
) -> Vec<SelectionCall> {
    let Some(mut snapshot) = source.snapshot() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for action in actions {
        match *action {
            SelectionAction::Select(id) => {
                out.push(SelectionCall::Select(id));
                snapshot.selection = id;
            }
            SelectionAction::StopAttacking(target) => {
                out.push(SelectionCall::StopAttacking(target));
            }
            SelectionAction::Key(key) => {
                for call in key_calls(source, key, &snapshot, typing) {
                    if let SelectionCall::Select(id) = call {
                        snapshot.selection = id;
                    }
                    out.push(call);
                }
            }
            SelectionAction::ClearIfSelected(object) => {
                if snapshot.selection == Some(object) {
                    out.push(SelectionCall::Select(None));
                    snapshot.selection = None;
                }
            }
        }
    }
    out
}

/// The objects in the radar's range, as `Select` reads them: `CWRepository`'s type (`0x14`),
/// selectability (`0x270`) and owner (`0x1F9`), and the fellowship.
fn candidates<S: SelectionSource>(source: &S) -> Vec<Candidate> {
    source
        .radar_blips()
        .iter()
        .filter(|blip| blip.in_range)
        .filter_map(|blip| {
            let object = source.object(blip.id)?;
            let is_fellow = source.is_fellow(blip.id);
            Some(Candidate {
                id: blip.id,
                kind: object.kind,
                selectable: object.selectable,
                owner: object.owner,
                fellow: is_fellow,
                owner_fellow: object.owner.is_some_and(|owner| source.is_fellow(owner)),
                // `CanAutoAttackTarget` (`Client::can_auto_attack_target`) is not asked here
                // yet: the script answered no for a creature of the double. Until that is
                // traced, a creature that is not a fellow.
                can_attack: object.kind & 0x4F == 0x4F && !is_fellow,
                offset: blip.offset,
            })
        })
        .collect()
}
