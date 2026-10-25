//! The character's advancement as the panels read it: experience (the player's qualities), the
//! skill repository the server sends (`WM_Skill` -> `CAgent` 0x1/0x2/0x3/0x7 in the Legions
//! script library), the shortcut bar (the `GameplayOptionsProfile` `WM_Player` 0x27 carries), and
//! the fellowship and allegiance ([`Social`]).
//!
//! Script objects read here: `SkillRepository` (package 0x233: slot 0 skill credits, slot 3
//! hero skill credits, slot 7 the `ARHash` of skills by enum, `SkillRepository` 0x8/0x18/0x11)
//! and `SkillInfo` (0x231: slot 14 the skill's enum, slot 7 its flags, bit 1 trained, slots 5-6
//! its experience; `SkillInfo` 0xF, 0x15, 0x1D). A skill's level comes from the
//! `AdvancementTable` (`SkillSystem` 0x21), which the script VM owns.

use ac2_net::protocol::{
    HeapObject, NativeObject, PackValue, PackedEntry, PackedReference, ScriptHash, ShortcutInfo,
};

use crate::notice::GameplayNotice;
use crate::qualities::WeenieQualities;
use crate::social::Social;

const SKILL_INFO_PACKAGE: u32 = 0x231;
const SKILL_REPOSITORY_PACKAGE: u32 = 0x233;

/// The player's experience and level (qualities long 300 `TotalXP`, long 301 `AvailableXP`,
/// int 0x12E `Level`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Experience {
    pub total_xp: u64,
    pub available_xp: u64,
    pub level: i32,
}

impl Experience {
    pub fn of(q: &WeenieQualities) -> Self {
        Self {
            total_xp: q.long_ints.get(&300).copied().unwrap_or(0) as u64,
            available_xp: q.long_ints.get(&301).copied().unwrap_or(0) as u64,
            level: q.ints.get(&0x12E).copied().unwrap_or(1),
        }
    }
}

/// One `SkillInfo`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Skill {
    /// The skill's enum.
    pub skill: u32,
    pub flags: u32,
    pub xp: u64,
}

impl Skill {
    /// `SkillInfo` 0x15: flag bit 1.
    pub fn trained(&self) -> bool {
        self.flags & 1 != 0
    }

    fn of(object: &HeapObject) -> Option<Self> {
        let s = &object.slots;
        (object.pkg_id == SKILL_INFO_PACKAGE && s.len() == 15).then(|| Self {
            skill: s[14],
            flags: s[7],
            xp: u64::from(s[5]) | u64::from(s[6]) << 32,
        })
    }
}

/// The skill repository: credits and the skills by enum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Skills {
    pub credits: u32,
    pub hero_credits: u32,
    /// In enum order.
    pub skills: Vec<Skill>,
}

/// The shortcut bar (`GameplayOptionsProfile`'s shortcut array, set and height).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shortcuts {
    pub set: u32,
    pub height: u32,
    pub slots: Vec<ShortcutInfo>,
}

/// Everything the advancement, shortcut and social panels read.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    pub skills: Skills,
    pub shortcuts: Shortcuts,
    pub social: Social,
}

fn heap<'a>(graph: &'a PackedReference, id: u32) -> Option<&'a HeapObject> {
    let at = graph.ids.iter().position(|i| *i == id)?;
    match graph.entries.get(at)? {
        PackedEntry::Heap { object, .. } => Some(object),
        _ => None,
    }
}

/// An `ARHash` of skills (`SkillRepository` slot 7): each value names a `SkillInfo` in `graph`.
fn skill_table(graph: &PackedReference, table: u32) -> Option<Vec<Skill>> {
    let hash = match &heap(graph, table)?.native {
        Some(NativeObject::Hash(ScriptHash { values, .. })) => values,
        _ => return None,
    };
    let mut skills: Vec<Skill> = hash
        .iter()
        .filter_map(|v| Skill::of(heap(graph, *v as u32)?))
        .collect();
    skills.sort_by_key(|s| s.skill);
    Some(skills)
}

fn reference(value: &PackValue) -> Option<&PackedReference> {
    match value {
        PackValue::Reference(r) => Some(r),
        _ => None,
    }
}

fn root_id(graph: &PackedReference) -> Option<u32> {
    graph.ids.first().copied()
}

impl Progress {
    fn panel(&self, q: &WeenieQualities) -> GameplayNotice {
        GameplayNotice::UpdateSkillPanel {
            available_xp: Experience::of(q).available_xp,
            skill_credits: self.skills.credits,
            hero_skill_credits: self.skills.hero_credits,
        }
    }

    /// A `WM_Skill` call to the player (`function`, its values); `None` for other functions.
    pub fn receive_skill(
        &mut self,
        function: u32,
        values: &[PackValue],
        q: &WeenieQualities,
    ) -> Option<Vec<GameplayNotice>> {
        match (function, values) {
            // 0xE -> `CAgent` 0x3 (`Agent` 0x6C): the whole repository, then the panel.
            (0x8214_000E, [v]) => {
                let graph = reference(v)?;
                let repo = heap(graph, root_id(graph)?)?;
                if repo.pkg_id != SKILL_REPOSITORY_PACKAGE || repo.slots.len() != 8 {
                    return None;
                }
                self.skills = Skills {
                    credits: repo.slots[0],
                    hero_credits: repo.slots[3],
                    skills: skill_table(graph, repo.slots[7]).unwrap_or_default(),
                };
                Some(vec![self.panel(q)])
            }
            // 0x11 -> `CAgent` 0x2 (`SkillRepository` 0x13): the skill table, then the panel.
            (0x8214_0011, [v]) => {
                let graph = reference(v)?;
                self.skills.skills = skill_table(graph, root_id(graph)?)?;
                Some(vec![self.panel(q)])
            }
            // 0xD -> `CAgent` 0x7 (`SkillRepository` 0x9): one skill added or replaced, then
            // `CPlayer` 0x85 (table 0x80) raises its update.
            (0x8214_000D, [v]) => {
                let graph = reference(v)?;
                let skill = Skill::of(heap(graph, root_id(graph)?)?)?;
                let list = &mut self.skills.skills;
                match list.binary_search_by_key(&skill.skill, |s| s.skill) {
                    Ok(at) => list[at] = skill,
                    Err(at) => list.insert(at, skill),
                }
                Some(vec![GameplayNotice::UpdateSkill(skill.skill)])
            }
            // 0x14 -> `CAgent` 0x1 (`SkillRepository` 0x14): one skill removed.
            (0x8214_0014, [PackValue::Int(skill)]) => {
                self.skills.skills.retain(|s| s.skill != *skill);
                Some(vec![GameplayNotice::UpdateSkill(*skill)])
            }
            _ => None,
        }
    }

    /// `CPlayer` 0x85 on long quality 0x12D (`AvailableXP`): the skill panel again.
    pub fn quality_changed(
        &self,
        q: &WeenieQualities,
        table: u32,
        quality: u32,
    ) -> Option<GameplayNotice> {
        (table == 12 && quality == 0x12D).then(|| self.panel(q))
    }

    /// `CPlayer` 0x65's options (`WM_Player` 0x27 argument 4 -> `Player` 0x17, slot 34): the
    /// shortcut bar, then `SendNotice_ReloadOptions`.
    pub fn login_options(&mut self, options: &PackValue) -> Option<GameplayNotice> {
        let Some(NativeObject::GameplayOptionsProfile(p)) = options.as_native() else {
            return None;
        };
        self.shortcuts = Shortcuts {
            set: p.which_shortcut_set,
            height: p.shortcut_height,
            slots: p.shortcuts.clone(),
        };
        Some(GameplayNotice::ReloadOptions)
    }

    /// `GameplayOptionsProfile::SetShortcut`: only an index inside the array takes.
    pub fn set_shortcut(&mut self, index: u32, shortcut: ShortcutInfo) -> bool {
        match self.shortcuts.slots.get_mut(index as usize) {
            Some(slot) => {
                *slot = shortcut;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::{entry_flags, ntype, HashKey};

    fn skill_entry(skill: u32, flags: u32, xp: u64) -> PackedEntry {
        let mut slots = vec![0u32; 15];
        (slots[14], slots[7], slots[5], slots[6]) = (skill, flags, xp as u32, (xp >> 32) as u32);
        PackedEntry::Heap {
            flags: entry_flags::KIND_HEAP | entry_flags::LOADED,
            object: HeapObject {
                pkg_state_id: 0,
                pkg_id: SKILL_INFO_PACKAGE,
                native: None,
                slots,
                tags: vec![2; 15],
            },
        }
    }

    #[test]
    fn the_repository_and_single_skills_fill_the_skill_list() {
        let mut table = PackedReference::native(NativeObject::Hash(ScriptHash {
            native_type: ntype::AR_HASH,
            bucket_index: 0,
            keys: vec![HashKey::Value(0x41), HashKey::Value(0x40)],
            values: vec![2, 3],
        }));
        table.ids.extend([2, 3]);
        table
            .entries
            .extend([skill_entry(0x41, 1, 500), skill_entry(0x40, 0, 0)]);
        let q = WeenieQualities::default();
        let mut p = Progress::default();
        let out = p
            .receive_skill(0x8214_0011, &[PackValue::Reference(table)], &q)
            .unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(
            p.skills.skills.iter().map(|s| s.skill).collect::<Vec<_>>(),
            [0x40, 0x41]
        );
        assert!(p.skills.skills[1].trained() && p.skills.skills[1].xp == 500);
        let one = PackedReference {
            ids: vec![7],
            entries: vec![skill_entry(0x42, 1, 1)],
        };
        assert_eq!(
            p.receive_skill(0x8214_000D, &[PackValue::Reference(one)], &q),
            Some(vec![GameplayNotice::UpdateSkill(0x42)])
        );
        p.receive_skill(0x8214_0014, &[PackValue::Int(0x40)], &q);
        assert_eq!(
            p.skills.skills.iter().map(|s| s.skill).collect::<Vec<_>>(),
            [0x41, 0x42]
        );
        assert!(p.receive_skill(0x8214_0014, &[], &q).is_none());
    }
}
