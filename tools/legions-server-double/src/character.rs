//! The double's character as its server would describe it at login: a race (int 290, the
//! player description's), the skill repository and the options profile's shortcut bar (the
//! login message's arguments 5 and 4), so the skill panels and the bar have real data. The
//! skill answers (`WM_Skill` 0x13 train, 0x10 raise) spend the player's credits and XP.

use std::collections::BTreeMap;

use ac2_net::protocol::{
    GameplayOptionsProfile, PackValue, PackedReference, QualityValue, ShortcutInfo, SkillMessage,
    SkillRecord, SkillRepository,
};

/// `Agent` 0x13's race (int 290): Human (enum mapper 0x41000009).
pub const RACE_HUMAN: u32 = 1;
/// Skills of the Legions `MasterSkillList`: Melee Auto Attack, Melee Attack.
pub const MELEE_AUTO_ATTACK: u32 = 6;
pub const MELEE_ATTACK: u32 = 7;

/// What a skill costs the player: the credits its training takes and the XP a raise spends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillCost {
    pub credits: u32,
    pub xp: u64,
}

/// The cost a skill without a configured one has: a training takes one credit, a raise spends
/// ten XP.
pub const DEFAULT_SKILL_COST: SkillCost = SkillCost { credits: 1, xp: 10 };

/// The player's skills (`SkillRepository`, package 0x233) as the double sends them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Character {
    pub credits: u32,
    pub hero_credits: u32,
    /// Skill enum, trained, experience spent.
    pub skills: Vec<(u32, bool, u64)>,
    /// Long qualities 300 (`TotalXP`) and 301 (`AvailableXP`), int 0x12E (`Level`), as the
    /// double last sent them.
    pub total_xp: u64,
    pub available_xp: u64,
    pub level: i32,
    /// Int 3100, the money bag's money, as the double last sent it.
    pub money: i32,
    /// The coin stacks the double has made (the coin item's id counts from them).
    pub coins_made: u32,
    /// The cost of each skill the test configures (see [`DEFAULT_SKILL_COST`]).
    pub skill_costs: BTreeMap<u32, SkillCost>,
    /// The skill being untrained (0 for none; the repository's slot 5).
    pub untraining: u32,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            credits: 2,
            hero_credits: 0,
            skills: vec![(MELEE_AUTO_ATTACK, true, 0), (MELEE_ATTACK, true, 0)],
            total_xp: 0,
            available_xp: 0,
            level: 1,
            money: 0,
            coins_made: 0,
            skill_costs: BTreeMap::new(),
            untraining: 0,
        }
    }
}

impl Character {
    /// Whether the character has `skill` trained.
    pub fn is_trained(&self, skill: u32) -> bool {
        self.skills.iter().any(|(s, t, _)| *s == skill && *t)
    }

    /// The `SkillRepository` graph (`WM_Player` 0x27's argument 5, `WM_Skill` 0xE's), as the
    /// codec in ac2-net packs it.
    pub fn repository(&self) -> PackedReference {
        self.to_repository().graph()
    }

    /// The character's skills as the codec's repository.
    pub fn to_repository(&self) -> SkillRepository {
        SkillRepository {
            credits: self.credits,
            hero_credits: self.hero_credits,
            untraining: self.untraining,
            skills: self
                .skills
                .iter()
                .map(|&(skill, trained, xp)| SkillRecord {
                    skill,
                    flags: u32::from(trained),
                    xp,
                })
                .collect(),
        }
    }

    /// The cost of `skill` (its configured cost, or [`DEFAULT_SKILL_COST`]).
    pub fn skill_cost(&self, skill: u32) -> SkillCost {
        self.skill_costs
            .get(&skill)
            .copied()
            .unwrap_or(DEFAULT_SKILL_COST)
    }

    /// `WM_Skill` 0xE's arguments.
    pub fn repository_call(&self) -> Vec<PackValue> {
        vec![PackValue::Reference(self.repository())]
    }
}

/// The options profile the double's character logs in with: a shortcut bar of ten (set 0,
/// one row) holding Melee Auto Attack, a chat line, then nothing; requests accepted; the radar
/// showing monsters, NPCs, players, portals, lifestones and items (`gmCharacterUI::RefreshFlag`'s
/// mask bits 0x1..0x20; `GameplayOptionsProfile::Clear`'s 0 shows nothing).
pub fn default_options() -> GameplayOptionsProfile {
    let mut shortcuts = vec![ShortcutInfo::Empty(0); 10];
    shortcuts[0] = ShortcutInfo::Enum {
        kind: 0x4000_0001,
        value: MELEE_AUTO_ATTACK,
    };
    shortcuts[1] = ShortcutInfo::Text("/e waves".into());
    GameplayOptionsProfile {
        // The bit field (0x80): allegiance and fellowship requests accepted (0x40, 0x80), and the
        // damage text shown for the player's own hits and for others' (0x4, 0x8).
        pack_bitfield: 0x2 | 0x4 | 0x20 | 0x80 | 0x1000,
        bit_field: 0x4 | 0x8 | 0x40 | 0x80,
        radar_mask: 0x3F,
        shortcuts,
        which_shortcut_set: 0,
        shortcut_height: 1,
        ..GameplayOptionsProfile::default()
    }
}

impl crate::State {
    pub(crate) fn send_private(&mut self, quality: u32, value: ac2_net::protocol::QualityValue) {
        self.send_quality(&ac2_net::protocol::QualityUpdate {
            object: None,
            quality,
            value,
        });
    }

    /// The server grants experience: `TotalXP` (long 300) and `AvailableXP` (long 301) go up
    /// by `amount`, each a private quality update (`CPlayer` 0x85 hears both: the XP bar, the
    /// skill panel).
    pub fn grant_xp(&mut self, amount: u64) {
        use ac2_net::protocol::QualityValue::LongInt;
        let c = &mut self.script_world.character;
        c.total_xp += amount;
        c.available_xp += amount;
        let (total, available) = (c.total_xp as i64, c.available_xp as i64);
        self.send_private(300, LongInt(total));
        self.send_private(301, LongInt(available));
    }

    /// The server raises the player's level (int 0x12E, `CPlayer` 0x85: the XP bar against
    /// the new level's range).
    pub fn set_level(&mut self, level: i32) {
        self.script_world.character.level = level;
        self.send_private(0x12E, ac2_net::protocol::QualityValue::Int(level));
    }

    /// Configures what skill `skill` costs (its training's credits, its raise's XP).
    pub fn set_skill_cost(&mut self, skill: u32, cost: SkillCost) {
        self.script_world.character.skill_costs.insert(skill, cost);
    }

    /// The server's answer to a skill request. Train (`WM_Skill` 0x13) takes the skill's credits and
    /// makes it trained: answered with the repository (0xE, the credits) and the skill (0xD). Raise
    /// (0x10) takes its XP from `AvailableXP` (long 301, a quality update) into the skill's
    /// experience: answered with the skill (0xD). Untrain (0x12) marks the skill as being untrained
    /// (the repository's slot 5), and cancel (0xF) clears it: both answered with the repository.
    /// A request the rules refuse (already trained, not trained, not enough credits or XP, another
    /// untrain under way) gets no answer. The costs are the test's ([`State::set_skill_cost`]): the
    /// double reads no advancement table, and its untrain completes nothing (no timer is run).
    pub(crate) fn answer_skill(&mut self, function: u32, arguments: &[PackValue]) {
        use ac2_net::protocol::script_function as f;
        match (function, arguments) {
            (f::SKILL_CANCEL_UNTRAIN, []) => {
                if self.script_world.character.untraining == 0 {
                    return;
                }
                self.script_world.character.untraining = 0;
                self.send_skill(SkillMessage::SetRepository(self.repository_now()));
            }
            (f::SKILL_UNTRAIN, [PackValue::Int(skill)]) => {
                let c = &mut self.script_world.character;
                if c.untraining != 0 || !c.is_trained(*skill) {
                    return;
                }
                c.untraining = *skill;
                self.send_skill(SkillMessage::SetRepository(self.repository_now()));
            }
            (f::SKILL_TRAIN, [PackValue::Int(skill)]) => {
                let skill = *skill;
                let cost = self.script_world.character.skill_cost(skill);
                let c = &mut self.script_world.character;
                if c.is_trained(skill) || c.credits < cost.credits {
                    return;
                }
                c.credits -= cost.credits;
                match c.skills.iter_mut().find(|(s, _, _)| *s == skill) {
                    Some(entry) => entry.1 = true,
                    None => c.skills.push((skill, true, 0)),
                }
                let record = SkillRecord {
                    skill,
                    flags: 1,
                    xp: c
                        .skills
                        .iter()
                        .find(|(s, _, _)| *s == skill)
                        .map_or(0, |e| e.2),
                };
                self.send_skill(SkillMessage::SetRepository(self.repository_now()));
                self.send_skill(SkillMessage::UpdateSkill(record));
            }
            (f::SKILL_RAISE, [PackValue::Int(skill)]) => {
                let skill = *skill;
                let cost = self.script_world.character.skill_cost(skill);
                let c = &mut self.script_world.character;
                if !c.is_trained(skill) || c.available_xp < cost.xp {
                    return;
                }
                let Some(entry) = c.skills.iter_mut().find(|(s, _, _)| *s == skill) else {
                    return;
                };
                entry.2 += cost.xp;
                let record = SkillRecord {
                    skill,
                    flags: 1,
                    xp: entry.2,
                };
                c.available_xp -= cost.xp;
                let available = c.available_xp as i64;
                self.send_private(301, QualityValue::LongInt(available));
                self.send_skill(SkillMessage::UpdateSkill(record));
            }
            _ => {}
        }
    }

    /// The repository as the double's character is now.
    fn repository_now(&self) -> SkillRepository {
        self.script_world.character.to_repository()
    }

    /// Sends a skill answer and keeps it for the tests.
    fn send_skill(&mut self, message: SkillMessage) {
        self.send_script_call(message.function(), &message.arguments());
        self.skill_answers.push(message);
    }
}
