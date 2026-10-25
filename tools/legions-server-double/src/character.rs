//! The double's character as its server would describe it at login: a race (int 290, the
//! player description's), the skill repository and the options profile's shortcut bar (the
//! login message's arguments 5 and 4), so the skill panels and the bar have real data.

use ac2_net::protocol::{
    entry_flags, ntype, GameplayOptionsProfile, HashKey, HeapObject, NativeObject, PackValue,
    PackedEntry, PackedReference, ScriptHash, ShortcutInfo,
};

/// `Agent` 0x13's race (int 290): Human (enum mapper 0x41000009).
pub const RACE_HUMAN: u32 = 1;
/// Skills of the Legions `MasterSkillList`: Melee Auto Attack, Melee Attack.
pub const MELEE_AUTO_ATTACK: u32 = 6;
pub const MELEE_ATTACK: u32 = 7;

/// The player's skills (`SkillRepository`, package 0x233) as the double sends them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub credits: u32,
    pub hero_credits: u32,
    /// Skill enum, trained, experience spent.
    pub skills: Vec<(u32, bool, u64)>,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            credits: 2,
            hero_credits: 0,
            skills: vec![(MELEE_AUTO_ATTACK, true, 0), (MELEE_ATTACK, true, 0)],
        }
    }
}

fn heap(pkg_id: u32, native: Option<NativeObject>, slots: Vec<u32>, tags: Vec<u8>) -> PackedEntry {
    PackedEntry::Heap {
        flags: entry_flags::KIND_HEAP | entry_flags::LOADED | entry_flags::RECURSE,
        object: HeapObject {
            pkg_state_id: 0,
            pkg_id,
            native,
            slots,
            tags,
        },
    }
}

fn hash(native_type: u32, keys: Vec<HashKey>, values: Vec<u64>) -> PackedEntry {
    let native = NativeObject::Hash(ScriptHash {
        native_type,
        bucket_index: 0,
        keys,
        values,
    });
    heap(u32::MAX, Some(native), Vec::new(), Vec::new())
}

impl Character {
    /// The `SkillRepository` graph (`WM_Player` 0x27's argument 5, `WM_Skill` 0xE's), in the
    /// class default's layout:
    /// credits, a 64-bit word, hero credits, two `AAHash`es, the skill being untrained (none),
    /// the `ARHash` of `SkillInfo`s by enum (package 0x231: two doubles at -1, the experience
    /// in slots 5-6, the flags in 7 (bit 1 trained), the enum in 14).
    pub fn repository(&self) -> PackedReference {
        let mut ids = vec![1, 2, 3, 4];
        let infos: Vec<u32> = (0..self.skills.len() as u32).map(|i| 5 + i).collect();
        ids.extend(&infos);
        let mut entries = vec![
            heap(
                0x233,
                None,
                vec![self.credits, 0, 0, self.hero_credits, 2, 0, 3, 4],
                vec![2, 0x82, 0xCD, 2, 1, 2, 1, 1],
            ),
            hash(ntype::AA_HASH, Vec::new(), Vec::new()),
            hash(ntype::AA_HASH, Vec::new(), Vec::new()),
            hash(
                ntype::AR_HASH,
                self.skills
                    .iter()
                    .map(|s| HashKey::Value(u64::from(s.0)))
                    .collect(),
                infos.iter().map(|i| u64::from(*i)).collect(),
            ),
        ];
        for &(skill, trained, xp) in &self.skills {
            let slots = vec![
                0,
                0xBFF0_0000,
                0,
                0,
                0,
                xp as u32,
                (xp >> 32) as u32,
                u32::from(trained),
                0,
                0xBFF0_0000,
                0,
                0,
                0,
                0,
                skill,
            ];
            let tags = vec![
                0xC3, 0xCD, 2, 0x83, 0xCD, 0x82, 0xCD, 2, 0xC3, 0xCD, 2, 2, 2, 2, 2,
            ];
            entries.push(heap(0x231, None, slots, tags));
        }
        PackedReference { ids, entries }
    }

    /// `WM_Skill` 0xE's arguments.
    pub fn repository_call(&self) -> Vec<PackValue> {
        vec![PackValue::Reference(self.repository())]
    }
}

/// The options profile the double's character logs in with: a shortcut bar of ten (set 0,
/// one row) holding Melee Auto Attack, a chat line, then nothing.
pub fn default_options() -> GameplayOptionsProfile {
    let mut shortcuts = vec![ShortcutInfo::Empty(0); 10];
    shortcuts[0] = ShortcutInfo::Enum {
        kind: 0x4000_0001,
        value: MELEE_AUTO_ATTACK,
    };
    shortcuts[1] = ShortcutInfo::Text("/e waves".into());
    GameplayOptionsProfile {
        pack_bitfield: 0x2 | 0x4 | 0x1000,
        shortcuts,
        which_shortcut_set: 0,
        shortcut_height: 1,
        ..GameplayOptionsProfile::default()
    }
}
