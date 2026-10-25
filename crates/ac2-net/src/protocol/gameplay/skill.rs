//! The skill messages (`WM_Skill`, package 0x214): the server's answers that fill the player's
//! skill repository, and the repository's records. The answers are forwarders the server reaches
//! (`WM_Skill` 0xD -> `CAgent` 0x7, 0xE -> `CAgent` 0x3, 0x11 -> `CAgent` 0x2, 0x14 -> `CAgent` 0x1),
//! which the `CAgent` functions hand to `SkillRepository` (package 0x233) and `SkillInfo`
//! (package 0x231). The records are per-player heap objects, not DAT WStates: a `SkillInfo` holds
//! its experience (slots 5-6), its flags (slot 7, bit 0: trained) and its enum (slot 14); the
//! repository holds the credits (slot 0), the hero credits (slot 3), the skill being untrained
//! (slot 5) and the `ARHash` of skills by enum (slot 7).
//!
//! The requests are the stubs the IFuns call (`gmCWInterface::RequestTrainSkill` -> 0x2,
//! `RequestUntrainSkill` -> 0x7, `RequestRaiseSkill` -> 0xC, `RequestUntrainSkillCancel` -> 0x1);
//! they are plain [`ScriptCall`](super::ScriptCall)s.

use super::Args;
use crate::protocol::native::{HashKey, NativeObject, ScriptHash};
use crate::protocol::pack::{entry_flags, HeapObject, PackValue, PackedEntry, PackedReference};
use crate::protocol::{ntype, ProtocolError};

/// `SkillRepository`'s package id.
pub const SKILL_REPOSITORY_PACKAGE: u32 = 0x233;
/// `SkillInfo`'s package id.
pub const SKILL_INFO_PACKAGE: u32 = 0x231;
/// The reference id of a message's root value.
const ROOT_ID: u32 = 1;
/// A heap entry as the server packs it.
const HEAP_FLAGS: u32 = entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE;
/// `SkillInfo`'s flag bit for a trained skill (slot 7).
const TRAINED: u32 = 1;
/// The `SkillInfo` slot the experience's low and high words sit in.
const XP_LOW_SLOT: usize = 5;
const XP_HIGH_SLOT: usize = 6;
const FLAGS_SLOT: usize = 7;
const SKILL_SLOT: usize = 14;
/// The `SkillRepository` slots the answers read and write.
const CREDITS_SLOT: usize = 0;
const HERO_CREDITS_SLOT: usize = 3;
const UNTRAINING_SLOT: usize = 5;
const SKILLS_SLOT: usize = 7;
/// A `SkillRepository` has eight slots; a `SkillInfo` fifteen.
const REPOSITORY_SLOTS: usize = 8;
const SKILL_INFO_SLOTS: usize = 15;

/// One `SkillInfo`: the skill's enum, its flags (bit 0: trained) and the experience spent on it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SkillRecord {
    pub skill: u32,
    pub flags: u32,
    pub xp: u64,
}

impl SkillRecord {
    /// A skill the player has trained (flag bit 0).
    pub fn trained(&self) -> bool {
        self.flags & TRAINED != 0
    }

    /// A `SkillInfo` heap object as the repository's constructor leaves it, with the skill's
    /// enum, flags and experience set.
    fn object(&self) -> HeapObject {
        let mut slots = vec![0u32; SKILL_INFO_SLOTS];
        slots[1] = 0xBFF0_0000;
        slots[9] = 0xBFF0_0000;
        slots[XP_LOW_SLOT] = self.xp as u32;
        slots[XP_HIGH_SLOT] = (self.xp >> 32) as u32;
        slots[FLAGS_SLOT] = self.flags;
        slots[SKILL_SLOT] = self.skill;
        HeapObject {
            pkg_state_id: 0,
            pkg_id: SKILL_INFO_PACKAGE,
            native: None,
            slots,
            tags: SKILL_INFO_TAGS.to_vec(),
        }
    }

    /// The record a `SkillInfo` heap object holds, or `None` for another object.
    fn of(object: &HeapObject) -> Option<Self> {
        let s = &object.slots;
        (object.pkg_id == SKILL_INFO_PACKAGE && s.len() == SKILL_INFO_SLOTS).then(|| Self {
            skill: s[SKILL_SLOT],
            flags: s[FLAGS_SLOT],
            xp: u64::from(s[XP_LOW_SLOT]) | u64::from(s[XP_HIGH_SLOT]) << 32,
        })
    }
}

/// The `VM`'s slot tags a `SkillInfo` carries (`tag::INT` 0x2, `REF` 0x1, `UNINIT` 0xCD, the
/// double 0x83, the long 0x82, `TIME` 0xC3).
const SKILL_INFO_TAGS: [u8; SKILL_INFO_SLOTS] = [
    0xC3, 0xCD, 2, 0x83, 0xCD, 0x82, 0xCD, 2, 0xC3, 0xCD, 2, 2, 2, 2, 2,
];

/// The player's `SkillRepository`: the credits, the hero credits, the skill being untrained (0
/// for none) and the skills by enum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkillRepository {
    pub credits: u32,
    pub hero_credits: u32,
    pub untraining: u32,
    pub skills: Vec<SkillRecord>,
}

impl SkillRepository {
    /// The repository's reference graph as the login and the answers pack it: the repository
    /// (credits, a 64-bit word, the hero credits, two `AAHash`es, the untraining skill), the
    /// `ARHash` of the `SkillInfo`s by enum, then the records.
    pub fn graph(&self) -> PackedReference {
        let infos: Vec<u32> = (0..self.skills.len() as u32).map(|i| 5 + i).collect();
        let mut ids = vec![1, 2, 3, 4];
        ids.extend(&infos);
        let repository = HeapObject {
            pkg_state_id: 0,
            pkg_id: SKILL_REPOSITORY_PACKAGE,
            native: None,
            slots: vec![
                self.credits,
                0,
                0,
                self.hero_credits,
                2,
                self.untraining,
                3,
                4,
            ],
            tags: vec![2, 0x82, 0xCD, 2, 1, 2, 1, 1],
        };
        let mut entries = vec![heap(repository)];
        entries.push(heap(hash(ntype::AA_HASH, Vec::new(), Vec::new())));
        entries.push(heap(hash(ntype::AA_HASH, Vec::new(), Vec::new())));
        entries.push(heap(hash(
            ntype::AR_HASH,
            self.skills
                .iter()
                .map(|s| HashKey::Value(u64::from(s.skill)))
                .collect(),
            infos.iter().map(|i| u64::from(*i)).collect(),
        )));
        entries.extend(self.skills.iter().map(|s| heap(s.object())));
        PackedReference { ids, entries }
    }

    /// The repository a reference graph holds (its root), or `None` for another object.
    pub fn decode(graph: &PackedReference) -> Option<Self> {
        let repo = heap_of(graph, root_id(graph)?)?;
        if repo.pkg_id != SKILL_REPOSITORY_PACKAGE || repo.slots.len() != REPOSITORY_SLOTS {
            return None;
        }
        Some(Self {
            credits: repo.slots[CREDITS_SLOT],
            hero_credits: repo.slots[HERO_CREDITS_SLOT],
            untraining: repo.slots[UNTRAINING_SLOT],
            skills: skill_table(graph, repo.slots[SKILLS_SLOT])?,
        })
    }
}

/// A server answer to the skills (`WM_Skill` forwarders).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillMessage {
    /// `WM_Skill` 0xD (`CAgent` 0x7): one `SkillInfo` added or replaced.
    UpdateSkill(SkillRecord),
    /// `WM_Skill` 0xE (`CAgent` 0x3): the whole repository, with its credits.
    SetRepository(SkillRepository),
    /// `WM_Skill` 0x11 (`CAgent` 0x2): the skill table (an `ARHash` of `SkillInfo`s by enum).
    SetSkills(Vec<SkillRecord>),
    /// `WM_Skill` 0x14 (`CAgent` 0x1): the skill with this enum removed.
    RemoveSkill(u32),
}

impl SkillMessage {
    /// The function id the answer is sent to.
    pub fn function(&self) -> u32 {
        match self {
            Self::UpdateSkill(_) => super::function::SKILL_UPDATE,
            Self::SetRepository(_) => super::function::SKILL_SET_REPOSITORY,
            Self::SetSkills(_) => super::function::SKILL_SET_SKILLS,
            Self::RemoveSkill(_) => super::function::SKILL_REMOVE,
        }
    }

    /// The packed arguments in wire order.
    pub fn arguments(&self) -> Vec<PackValue> {
        match self {
            Self::UpdateSkill(record) => vec![PackValue::Reference(single(record))],
            Self::SetRepository(repository) => vec![PackValue::Reference(repository.graph())],
            Self::SetSkills(skills) => vec![PackValue::Reference(skill_table_graph(skills))],
            Self::RemoveSkill(skill) => vec![PackValue::Int(*skill)],
        }
    }

    /// The answer from its function id and packed arguments, or `None` for another id.
    pub fn decode(function: u32, arguments: &[u8]) -> Option<Result<Self, ProtocolError>> {
        Some(match function {
            super::function::SKILL_UPDATE => decode_with(arguments, |a| {
                let graph = a.reference()?;
                let record = SkillRecord::of(
                    heap_of(&graph, root_id(&graph).ok_or(invalid())?).ok_or_else(invalid)?,
                )
                .ok_or_else(invalid)?;
                Ok(Self::UpdateSkill(record))
            }),
            super::function::SKILL_SET_REPOSITORY => decode_with(arguments, |a| {
                let graph = a.reference()?;
                Ok(Self::SetRepository(
                    SkillRepository::decode(&graph).ok_or_else(invalid)?,
                ))
            }),
            super::function::SKILL_SET_SKILLS => decode_with(arguments, |a| {
                let graph = a.reference()?;
                let table = root_id(&graph).ok_or_else(invalid)?;
                Ok(Self::SetSkills(
                    skill_table(&graph, table).ok_or_else(invalid)?,
                ))
            }),
            super::function::SKILL_REMOVE => {
                decode_with(arguments, |a| Ok(Self::RemoveSkill(a.int()?)))
            }
            _ => return None,
        })
    }
}

fn invalid() -> ProtocolError {
    ProtocolError::Invalid("skill record")
}

/// Reads one answer's values with `read`, refusing any value left over.
fn decode_with(
    arguments: &[u8],
    read: impl FnOnce(&mut Args) -> Result<SkillMessage, ProtocolError>,
) -> Result<SkillMessage, ProtocolError> {
    let mut args = Args::new(arguments)?;
    let message = read(&mut args)?;
    args.finish()?;
    Ok(message)
}

fn heap(object: HeapObject) -> PackedEntry {
    PackedEntry::Heap {
        flags: HEAP_FLAGS,
        object,
    }
}

fn hash(native_type: u32, keys: Vec<HashKey>, values: Vec<u64>) -> HeapObject {
    HeapObject {
        pkg_state_id: 0,
        pkg_id: u32::MAX,
        native: Some(NativeObject::Hash(ScriptHash {
            native_type,
            bucket_index: 0,
            keys,
            values,
        })),
        slots: Vec::new(),
        tags: Vec::new(),
    }
}

/// One record as a message's reference: the record is the root.
fn single(record: &SkillRecord) -> PackedReference {
    PackedReference {
        ids: vec![ROOT_ID],
        entries: vec![heap(record.object())],
    }
}

/// The skill table as a message's reference: the `ARHash` is the root, its records after it.
fn skill_table_graph(skills: &[SkillRecord]) -> PackedReference {
    let infos: Vec<u32> = (0..skills.len() as u32).map(|i| ROOT_ID + 1 + i).collect();
    let mut ids = vec![ROOT_ID];
    ids.extend(&infos);
    let mut entries = vec![heap(hash(
        ntype::AR_HASH,
        skills
            .iter()
            .map(|s| HashKey::Value(u64::from(s.skill)))
            .collect(),
        infos.iter().map(|i| u64::from(*i)).collect(),
    ))];
    entries.extend(skills.iter().map(|s| heap(s.object())));
    PackedReference { ids, entries }
}

fn root_id(graph: &PackedReference) -> Option<u32> {
    graph.ids.first().copied()
}

/// The heap object a reference id names in `graph`.
fn heap_of(graph: &PackedReference, id: u32) -> Option<&HeapObject> {
    let at = graph.ids.iter().position(|i| *i == id)?;
    match graph.entries.get(at)? {
        PackedEntry::Heap { object, .. } => Some(object),
        _ => None,
    }
}

/// The records of the `ARHash` at `table` (each value names a `SkillInfo` in `graph`), by enum.
fn skill_table(graph: &PackedReference, table: u32) -> Option<Vec<SkillRecord>> {
    let values = match &heap_of(graph, table)?.native {
        Some(NativeObject::Hash(ScriptHash { values, .. })) => values,
        _ => return None,
    };
    let mut skills = values
        .iter()
        .map(|v| SkillRecord::of(heap_of(graph, *v as u32)?))
        .collect::<Option<Vec<_>>>()?;
    skills.sort_by_key(|s| s.skill);
    Some(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(skill: u32, xp: u64) -> SkillRecord {
        SkillRecord {
            skill,
            flags: TRAINED,
            xp,
        }
    }

    #[test]
    fn the_repository_round_trips_through_its_graph() {
        let repository = SkillRepository {
            credits: 3,
            hero_credits: 1,
            untraining: 0x41,
            skills: vec![record(0x40, 0), record(0x41, 1 << 33)],
        };
        let graph = repository.graph();
        assert_eq!(SkillRepository::decode(&graph), Some(repository));
    }

    #[test]
    fn the_answers_round_trip_through_their_arguments() {
        let messages = [
            SkillMessage::UpdateSkill(record(0x42, 7)),
            SkillMessage::SetRepository(SkillRepository {
                credits: 0,
                hero_credits: 0,
                untraining: 0,
                skills: vec![record(0x40, 2)],
            }),
            SkillMessage::SetSkills(vec![record(0x40, 0), record(0x41, 0)]),
            SkillMessage::RemoveSkill(0x40),
        ];
        for message in messages {
            let bytes = crate::protocol::pack_values(&message.arguments());
            let decoded = SkillMessage::decode(message.function(), &bytes)
                .expect("a skill answer")
                .expect("its values decode");
            assert_eq!(decoded, message);
        }
        assert!(SkillMessage::decode(0x8214_0002, &[]).is_none());
    }
}
