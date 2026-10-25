//! The skill panel's checks that need the player's script (`gmCWInterface::*ByEnum` ->
//! `ggmCWInterface::CallIFun_*`: a 0 return slot, the skill enum, the instance, then the
//! return slot read back), on the script VM: the `SkillSystem` rules over the
//! `MasterSkillList`, perks and effects. The ones the skill repository answers alone are
//! `ac2_weenie::Skills`' (`is_trained`, `is_being_untrained`, `xp`, the credits).

use ac2_script::host::IFunArg;
use ac2_script::vm::InterpStatus;

use crate::client::Client;

/// The `gmCWInterface` IFuns (`FunctionID` 0xB4xxxx).
const CAN_SKILL_BE_TRAINED: u32 = 0x00B4_0023;
const CAN_SKILL_BE_UNTRAINED: u32 = 0x00B4_0024;
const CAN_SKILL_BE_RAISED: u32 = 0x00B4_0025;
const SKILL_COST_TO_TRAIN: u32 = 0x00B4_0028;
const IS_HERO_SKILL: u32 = 0x00B4_012A;

impl Client {
    /// One `ulong`-returning skill IFun; `None` without a script VM or when the call did not
    /// finish (it stopped for data; retail's wrapper would read 0).
    fn skill_query(&mut self, function: u32, skill: u32) -> Option<u32> {
        let result = self.call_ifun(function, &[IFunArg::Int(skill)]).ok()?;
        (result.outcome.status == InterpStatus::Terminated).then_some(result.retval)
    }

    /// `gmCWInterface::CanSkillBeTrainedByEnum` (`CPlayer` 0xBE -> `SkillSystem` 0x1A): 0, or
    /// the reason (`ac2_weenie::skill_error`).
    pub fn can_skill_be_trained(&mut self, skill: u32) -> Option<u32> {
        self.skill_query(CAN_SKILL_BE_TRAINED, skill)
    }

    /// `gmCWInterface::CanSkillBeUntrainedByEnum` (`CPlayer` 0x6 -> `SkillSystem` 0xB).
    pub fn can_skill_be_untrained(&mut self, skill: u32) -> Option<u32> {
        self.skill_query(CAN_SKILL_BE_UNTRAINED, skill)
    }

    /// `gmCWInterface::CanSkillBeRaisedByEnum` (`CPlayer` 0xE2 -> `SkillSystem` 0x1).
    pub fn can_skill_be_raised(&mut self, skill: u32) -> Option<u32> {
        self.skill_query(CAN_SKILL_BE_RAISED, skill)
    }

    /// `gmCWInterface::GetSkillCostToTrainByEnum` (`CPlayer` 0x132 -> `SkillSystem` 0x17): the
    /// skill credits training costs.
    pub fn skill_cost_to_train(&mut self, skill: u32) -> Option<u32> {
        self.skill_query(SKILL_COST_TO_TRAIN, skill)
    }

    /// `gmCWInterface::IsHeroSkillByEnum` (`CPlayer` 0x1C7 -> `SkillSystem` 0x2B).
    pub fn is_hero_skill(&mut self, skill: u32) -> Option<bool> {
        self.skill_query(IS_HERO_SKILL, skill).map(|r| r != 0)
    }
}
