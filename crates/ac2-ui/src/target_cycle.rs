//! `gmSelectionUI::Select`'s target cycling: the action keys that pick the next, previous or
//! closest object of a kind (monsters, NPCs, players, items, foes, fellows, pets), and the
//! object the selection moves to.
//!
//! The key picks the direction: `0x41000019`, `1c`, `1f`, `22`, `25` and `5c` step back, `1a`,
//! `1d`, `20`, `23`, `26` and `5d` step forward, `1b`, `1e`, `21`, `24`, `27` and `5e` take the
//! closest. `0x41000022`-`0x41000024` (and `0x41000024`, the auto-target) take foes only
//! (`CanAutoAttackTarget` and not a fellow), `0x41000025`-`0x41000027` fellows only, and
//! `0x4100005C`-`5E` pets of someone else. `TargetTypeVsKey` decides which object types a key
//! takes.
//!
//! A candidate is in range when its planar distance is within the radar range; its distance is
//! the planar distance plus 1.2 times its height (`Position::localtolocal`). Forward (and
//! closest) picks the nearest object farther than the current selection's distance; backward
//! picks the farthest one nearer than it. Nothing in that band wraps round once: a second pass
//! takes the nearest (forward) or farthest (backward) in range (`gmSelectionUI::Select`'s
//! loop). The objects are ordered by distance, then by id (`gmSelectionUI::Farther`).

use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};

/// The target key a global key press carries, when `gmSelectionUI::ListenToMessage` passes it
/// to `Select`.
pub fn select_key_of(message: &UiMessage) -> Option<u32> {
    let key = message.data as u32;
    (message.message == GLOBAL_KEY_PRESS && is_select_key(key)).then_some(key)
}

/// The direction a key steps in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `-1`: the nearest-to-farthest order backwards.
    Previous,
    /// `0`: the closest.
    Closest,
    /// `+1`: forwards.
    Next,
}

/// What a cycling key asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CycleRule {
    pub step: Step,
    /// `local_3c`: fellows only (`0x41000025`-`0x41000027`).
    pub fellows: bool,
    /// `local_64`: foes only (`0x41000022`-`0x41000024`).
    pub foes: bool,
    /// `local_40`: pets of someone else (`0x4100005C`-`0x4100005E`).
    pub pets: bool,
}

/// One object the cycling keys may pick, as `Select` reads it from the object table.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub id: u64,
    /// `CWRepository::GetInt(0x14)`: the weenie type bits `TargetTypeVsKey` tests.
    pub kind: u32,
    /// `CWRepository::GetBool(0x270)`: whether the object can be selected.
    pub selectable: bool,
    /// `InqInstanceID(0x1F9)`: the object's owner, when it has one.
    pub owner: Option<u64>,
    /// `gmCWInterface::IsFellowshipMember` of this object, and of its owner.
    pub fellow: bool,
    pub owner_fellow: bool,
    /// `gmCWInterface::CanAutoAttackTarget`.
    pub can_attack: bool,
    /// The object's offset from the player (`Position::localtolocal`).
    pub offset: [f32; 3],
}

/// Whether `key` is one `gmSelectionUI::ListenToMessage` passes to `Select`: the target keys
/// `0x41000018`-`0x41000028` (but `0x4100002A`, the last attacker), `0x4000001E` (clear) and
/// `0x4100005C`-`0x4100005E`.
pub fn is_select_key(key: u32) -> bool {
    matches!(key, 0x4000_001E | 0x4100_002A)
        || (0x4100_0018..=0x4100_0028).contains(&key)
        || (0x4100_005C..=0x4100_005E).contains(&key)
}

/// The cycling rule of a key, or `None` for the keys `Select` handles otherwise
/// (`0x41000018`, `0x41000028`, `0x4100002A`, `0x4000001E`).
pub fn cycle_rule(key: u32) -> Option<CycleRule> {
    let step = match key {
        0x4100_0019 | 0x4100_001C | 0x4100_001F | 0x4100_0022 | 0x4100_0025 | 0x4100_005C => {
            Step::Previous
        }
        0x4100_001A | 0x4100_001D | 0x4100_0020 | 0x4100_0023 | 0x4100_0026 | 0x4100_005D => {
            Step::Next
        }
        0x4100_001B | 0x4100_001E | 0x4100_0021 | 0x4100_0024 | 0x4100_0027 | 0x4100_005E => {
            Step::Closest
        }
        _ => return None,
    };
    Some(CycleRule {
        step,
        fellows: (0x4100_0025..0x4100_0028).contains(&key),
        foes: (0x4100_0022..0x4100_0025).contains(&key),
        pets: (0x4100_005C..=0x4100_005E).contains(&key),
    })
}

/// `gmSelectionUI::TargetTypeVsKey`: whether an object of `kind` is one `key` takes.
pub fn type_vs_key(kind: u32, key: u32) -> bool {
    if kind & 0x81 == 0x81 || kind & 0x103 == 0x103 {
        return (0x4100_001C..=0x4100_001E).contains(&key);
    }
    if kind & 0x1F == 0x1F {
        return (0x4100_001F..=0x4100_0027).contains(&key);
    }
    if kind & 0x4F == 0x4F {
        return matches!(
            key,
            0x4100_0019
                | 0x4100_001A
                | 0x4100_001B
                | 0x4100_0022
                | 0x4100_0023
                | 0x4100_0024
                | 0x4100_005C
                | 0x4100_005D
                | 0x4100_005E
        );
    }
    false
}

/// `gmSelectionUI::Farther`: whether the object at `b` (distance, id) is nearer than the one at
/// `a`, or as near with the lower id (`b` then beats `a`).
pub fn farther(a: (f32, u64), b: (f32, u64)) -> bool {
    b.0 < a.0 || (a.0 == b.0 && b.1 < a.1)
}

/// `Position::localtolocal`'s planar distance squared, and the distance `Select` measures
/// (planar distance plus 1.2 times the height).
fn distances(offset: [f32; 3]) -> (f32, f32) {
    let planar = offset[0] * offset[0] + offset[1] * offset[1];
    let dist = planar.sqrt() + offset[2].abs() * 1.2;
    (planar, dist)
}

/// `gmSelectionUI::Select` for the cycling key `key`: the object the selection moves to, or
/// `None` when nothing qualifies. `selection` is the object now selected, `range_sq` the radar
/// range squared, and `candidates` the objects it may pick (the player is never one).
pub fn select_target(
    key: u32,
    player: u64,
    range_sq: f32,
    selection: Option<u64>,
    candidates: &[Candidate],
) -> Option<u64> {
    let rule = cycle_rule(key)?;
    let prev = rule.step == Step::Previous;
    let closest = rule.step == Step::Closest;
    // The reference: the selection, when it stands in range (cycling only).
    let reference = (!closest)
        .then(|| {
            let sel = candidates.iter().find(|c| Some(c.id) == selection)?;
            let (planar, dist) = distances(sel.offset);
            (planar <= range_sq).then_some((sel.id, dist))
        })
        .flatten();
    let ref_id = reference.map_or(0, |r| r.0);
    let ref_dist = match reference {
        Some((_, dist)) => dist,
        None if prev => range_sq,
        None => 0.0,
    };
    let mut best_dist = if prev { 0.0 } else { range_sq };
    let mut best_id = 0u64;
    let mut ref_dist_pass = ref_dist;
    for _pass in 0..2 {
        for c in candidates {
            if c.id == player || !c.selectable || !type_vs_key(c.kind, key) {
                continue;
            }
            if c.id == ref_id {
                continue;
            }
            if rule.fellows && !c.fellow {
                continue;
            }
            if rule.foes && (!c.can_attack || c.fellow) {
                continue;
            }
            if rule.pets && c.owner.is_none_or(|o| o == player) {
                continue;
            }
            // `local_64 && owner exists`: the owner is no one's fellow and not the player.
            if rule.foes {
                if let Some(owner) = c.owner {
                    if owner == player || c.owner_fellow {
                        continue;
                    }
                }
            }
            let (planar, dist) = distances(c.offset);
            if planar > range_sq {
                continue;
            }
            let nearer_than_best = farther((dist, c.id), (best_dist, best_id)) == prev;
            let beyond_reference = farther((dist, c.id), (ref_dist_pass, ref_id)) != prev;
            if nearer_than_best && beyond_reference {
                best_dist = dist;
                best_id = c.id;
            }
        }
        if best_id != 0 {
            return Some(best_id);
        }
        if ref_id == 0 {
            return None;
        }
        ref_dist_pass = if prev { range_sq } else { 0.0 };
    }
    None
}
