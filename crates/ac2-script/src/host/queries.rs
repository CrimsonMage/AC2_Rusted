//! The UI's questions to the player's script (`gmCWInterface` -> `ggmCWInterface::CallIFun_*`
//! -> `CPlayer`): the skill panel's and the shortcut bar's, each a query on the VM that reads
//! back the wrapper's return value and out parameters. A query that stops for data (the VM is
//! cache-only) or crashes answers `Err`; its call still finishes when the data lands.

use std::collections::HashMap;

use super::{IFunArg, IFunResult, IFunReturn, OutValue};
use crate::vm::InterpStatus;
use ac2_net::protocol::{
    GameplayOptionsProfile, NativeObject, ShortcutInfo, StringInfo, WordTable,
};

use super::port::ScriptPort;

/// The `ggmCWInterface::CallIFun_*` function ids these queries call.
pub mod ifun {
    pub const USE_SHORTCUT: u32 = 0x00B4_0007;
    /// `ggmCWInterface::CallIFun_DoUIDamage` (`gmCWInterface::DoUIDamage`): attacker, target, skill,
    /// attack result, target and attacker health change, target and attacker PK change.
    pub const DO_UI_DAMAGE: u32 = 0x00B4_00C0;
    /// `ggmCWInterface::CallIFun_SpecialAttackEnded` (`gmCWInterface::SpecialAttackEnded`): the
    /// story's client attack context.
    pub const SPECIAL_ATTACK_ENDED: u32 = 0x00B4_00C2;
    /// `ggmCWInterface::CallIFun_HandlePlayScenesDone` (`gmCWInterface::HandlePlayScenesDone`):
    /// the played `List<gmSceneInfo>`.
    pub const PLAY_SCENES_DONE: u32 = 0x00B4_0090;
    /// `ggmCWInterface::CallIFun_RequestResurrect` (`gmCWInterface::RequestResurrect`): no arguments.
    pub const REQUEST_RESURRECT: u32 = 0x00B4_006B;
    /// `ggmCWInterface::CallIFun_ClearResurrectionRequest` (`gmCWInterface::ClearResurrectionRequest`):
    /// no arguments; the resurrection offer is declined.
    pub const CLEAR_RESURRECTION_REQUEST: u32 = 0x00B4_00F7;
    /// `ggmCWInterface::CallIFun_CanAutoAttackTarget` (`gmCWInterface::CanAutoAttackTarget`): the
    /// target's `InstanceID`, an int back.
    pub const CAN_AUTO_ATTACK_TARGET: u32 = 0x00B4_000A;
    pub const INQ_ROOT_PANEL_FROM_RACE: u32 = 0x00B4_0020;
    pub const CAN_SKILL_BE_TRAINED_BY_ENUM: u32 = 0x00B4_0023;
    pub const CAN_SKILL_BE_UNTRAINED_BY_ENUM: u32 = 0x00B4_0024;
    pub const CAN_SKILL_BE_RAISED_BY_ENUM: u32 = 0x00B4_0025;
    pub const GET_SKILL_EXPERIENCE_NEEDED_TO_LEVEL_BY_ENUM: u32 = 0x00B4_0026;
    pub const INQ_SKILL_LEVEL_BY_ENUM: u32 = 0x00B4_0027;
    pub const GET_SKILL_COST_TO_TRAIN_BY_ENUM: u32 = 0x00B4_0028;
    pub const INQ_SKILL_TIME_LEFT: u32 = 0x00B4_002F;
    pub const INQ_SHORTCUT_INFO: u32 = 0x00B4_006D;
    pub const SET_SHORTCUT: u32 = 0x00B4_006F;
    pub const GET_GAMEPLAY_OPTIONS: u32 = 0x00B4_00DE;
    pub const IS_HERO_SKILL_BY_ENUM: u32 = 0x00B4_012A;
    pub const GET_SKILL_CREDITS: u32 = 0x00B4_001F;
    pub const GET_HERO_SKILL_CREDITS: u32 = 0x00B4_0129;
    pub const GET_SKILL_XP: u32 = 0x00B4_002A;
    pub const IS_SKILL_TRAINED: u32 = 0x00B4_0021;
    pub const IS_SKILL_BEING_UNTRAINED: u32 = 0x00B4_0022;
    pub const IS_SKILL_USABLE_BASED_ON_INVENTORY: u32 = 0x00B4_0031;
    pub const IS_SKILL_HARMFUL: u32 = 0x00B4_00C5;
}

/// `InqSkillLevelByEnum`'s answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillLevel {
    pub raw: u32,
    pub enchanted: u32,
    /// The wrapper's `retval`.
    pub ok: bool,
}

/// `InqSkillTimeLeft`'s answer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkillTime {
    pub total: f32,
    pub left: f32,
    pub ok: bool,
}

/// `InqShortcutInfo`'s answer: the shortcut's name and icon DID.
#[derive(Debug, Clone, PartialEq)]
pub struct ShortcutDisplay {
    pub name: StringInfo,
    pub icon: u32,
    pub ok: bool,
}

fn query_word<P: ScriptPort + ?Sized>(
    port: &mut P,
    function: u32,
    arg: u32,
) -> Result<u32, String> {
    Ok(port
        .query(function, &[IFunArg::Int(arg)], IFunReturn::Word)?
        .retval as u32)
}

fn out_word() -> IFunArg {
    IFunArg::Out(OutValue::Word(0))
}

/// The UI's script queries, on any [`ScriptPort`] (the client implements the port).
pub trait ScriptQueries: ScriptPort {
    /// `ggmCWInterface::CallIFun_*` `function` on the player with `args` (out parameters as
    /// `IFunArg::Out`), its return slot shaped `ret`; the output is routed like
    /// [`Client::call_ifun`]'s.
    fn query(
        &mut self,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
    ) -> Result<IFunResult, String> {
        let player = self.player_id().ok_or("no player")?;
        self.query_object(player, function, args, ret)
    }

    /// [`ScriptQueries::query`] on object `id`'s process instead of the player's (the item
    /// queries ask each item's own script): the same routing and the same stop check.
    fn query_object(
        &mut self,
        id: u64,
        function: u32,
        args: &[IFunArg],
        ret: IFunReturn,
    ) -> Result<IFunResult, String> {
        let scripts = self.script_host().ok_or("no script VM")?;
        let result = scripts
            .query(id, function, args, ret)
            .map_err(|e| format!("{e:?}"));
        let outcome = result
            .as_ref()
            .map(|r| r.outcome.clone())
            .map_err(Clone::clone);
        let mut out = Vec::new();
        self.route_outcome(id, None, outcome, &mut out);
        self.push_produced(out);
        let result = result?;
        if result.outcome.status != InterpStatus::Terminated {
            return Err(format!(
                "{id:#x} {function:#010x}: {:?} {:?}",
                result.outcome.status, result.outcome.fault
            ));
        }
        Ok(result)
    }

    /// `gmCWInterface::CanSkillBeTrainedByEnum`: 0, or the `gmWErrorType` why not.
    fn can_skill_be_trained(&mut self, skill: u32) -> Result<u32, String> {
        query_word(self, ifun::CAN_SKILL_BE_TRAINED_BY_ENUM, skill)
    }

    /// `gmCWInterface::CanSkillBeUntrainedByEnum`: 0, or the `gmWErrorType` why not.
    fn can_skill_be_untrained(&mut self, skill: u32) -> Result<u32, String> {
        query_word(self, ifun::CAN_SKILL_BE_UNTRAINED_BY_ENUM, skill)
    }

    /// `gmCWInterface::CanSkillBeRaisedByEnum`: 0, or the `gmWErrorType` why not.
    fn can_skill_be_raised(&mut self, skill: u32) -> Result<u32, String> {
        query_word(self, ifun::CAN_SKILL_BE_RAISED_BY_ENUM, skill)
    }

    /// `gmCWInterface::GetSkillCostToTrainByEnum`.
    fn skill_cost_to_train(&mut self, skill: u32) -> Result<u32, String> {
        query_word(self, ifun::GET_SKILL_COST_TO_TRAIN_BY_ENUM, skill)
    }

    /// `gmCWInterface::IsHeroSkillByEnum`.
    fn is_hero_skill(&mut self, skill: u32) -> Result<bool, String> {
        Ok(query_word(self, ifun::IS_HERO_SKILL_BY_ENUM, skill)? != 0)
    }

    /// `gmCWInterface::CanAutoAttackTarget(target)` (`gmSelectionUI::Select` asks it for each
    /// candidate of the cycling keys): whether the script lets the player auto-attack `target`.
    fn can_auto_attack_target(&mut self, target: u64) -> Result<bool, String> {
        let result = self.query(
            ifun::CAN_AUTO_ATTACK_TARGET,
            &[IFunArg::Long(target)],
            IFunReturn::Word,
        )?;
        Ok(result.retval != 0)
    }

    /// `gmCWInterface::GetSkillCredits`: the player's skill credits.
    fn skill_credits(&mut self) -> Result<u32, String> {
        Ok(self
            .query(ifun::GET_SKILL_CREDITS, &[], IFunReturn::Word)?
            .retval as u32)
    }

    /// `gmCWInterface::GetHeroSkillCredits`: the player's hero skill credits.
    fn hero_skill_credits(&mut self) -> Result<u32, String> {
        Ok(self
            .query(ifun::GET_HERO_SKILL_CREDITS, &[], IFunReturn::Word)?
            .retval as u32)
    }

    /// `gmCWInterface::GetSkillXPByEnum`: the experience spent on `skill` (a `ulong64`).
    fn skill_xp(&mut self, skill: u32) -> Result<u64, String> {
        Ok(self
            .query(ifun::GET_SKILL_XP, &[IFunArg::Int(skill)], IFunReturn::Long)?
            .retval)
    }

    /// `gmCWInterface::IsSkillTrainedByEnum`.
    fn is_skill_trained(&mut self, skill: u32) -> Result<bool, String> {
        Ok(query_word(self, ifun::IS_SKILL_TRAINED, skill)? != 0)
    }

    /// `gmCWInterface::IsSkillBeingUntrainedByEnum`.
    fn is_skill_being_untrained(&mut self, skill: u32) -> Result<bool, String> {
        Ok(query_word(self, ifun::IS_SKILL_BEING_UNTRAINED, skill)? != 0)
    }

    /// `gmCWInterface::IsSkillUsableBasedOnInventoryByEnum`.
    fn is_skill_usable_by_inventory(&mut self, skill: u32) -> Result<bool, String> {
        Ok(query_word(self, ifun::IS_SKILL_USABLE_BASED_ON_INVENTORY, skill)? != 0)
    }

    /// `gmCWInterface::IsSkillHarmful`.
    fn is_skill_harmful(&mut self, skill: u32) -> Result<bool, String> {
        Ok(query_word(self, ifun::IS_SKILL_HARMFUL, skill)? != 0)
    }

    /// `gmCWInterface::UseShortcut(index)`: the script's result.
    fn use_shortcut(&mut self, index: u32) -> Result<u32, String> {
        query_word(self, ifun::USE_SHORTCUT, index)
    }

    /// `gmCWInterface::GetSkillExperienceNeededToLevelByEnum` (a `ulong64`).
    fn skill_experience_to_level(&mut self, skill: u32) -> Result<u64, String> {
        let args = [IFunArg::Int(skill)];
        Ok(self
            .query(
                ifun::GET_SKILL_EXPERIENCE_NEEDED_TO_LEVEL_BY_ENUM,
                &args,
                IFunReturn::Long,
            )?
            .retval)
    }

    /// `gmCWInterface::InqSkillLevelByEnum`: the raw and enchanted levels.
    fn skill_level(&mut self, skill: u32) -> Result<SkillLevel, String> {
        let args = [IFunArg::Int(skill), out_word(), out_word()];
        let r = self.query(ifun::INQ_SKILL_LEVEL_BY_ENUM, &args, IFunReturn::Word)?;
        Ok(SkillLevel {
            raw: r.out_word(0),
            enchanted: r.out_word(1),
            ok: r.retval != 0,
        })
    }

    /// `gmCWInterface::InqSkillTimeLeft`: the skill's timer, total and left.
    fn skill_time_left(&mut self, skill: u32) -> Result<SkillTime, String> {
        let args = [IFunArg::Int(skill), out_word(), out_word()];
        let r = self.query(ifun::INQ_SKILL_TIME_LEFT, &args, IFunReturn::Word)?;
        Ok(SkillTime {
            total: f32::from_bits(r.out_word(0)),
            left: f32::from_bits(r.out_word(1)),
            ok: r.retval != 0,
        })
    }

    /// `gmCWInterface::InqRootPanelFromRace`: the skill tree's root panel for `race`.
    fn root_panel_from_race(&mut self, race: u32) -> Result<Option<u32>, String> {
        let args = [IFunArg::Int(race), out_word()];
        let r = self.query(ifun::INQ_ROOT_PANEL_FROM_RACE, &args, IFunReturn::Word)?;
        Ok((r.retval != 0).then(|| r.out_word(0)))
    }

    /// `gmCWInterface::SetShortcut(index, shortcut)`: the script's result.
    fn set_shortcut(&mut self, index: u32, shortcut: ShortcutInfo) -> Result<u32, String> {
        let args = [
            IFunArg::Int(index),
            IFunArg::Native(NativeObject::ShortcutInfo(shortcut)),
        ];
        Ok(self
            .query(ifun::SET_SHORTCUT, &args, IFunReturn::Word)?
            .retval as u32)
    }

    /// `gmCWInterface::InqShortcutInfo(index)`: the slot's name and icon.
    fn shortcut_info(&mut self, index: u32) -> Result<ShortcutDisplay, String> {
        let args = [
            IFunArg::Int(index),
            IFunArg::Out(OutValue::Native(NativeObject::StringInfo(
                StringInfo::default(),
            ))),
            out_word(),
        ];
        let r = self.query(ifun::INQ_SHORTCUT_INFO, &args, IFunReturn::Word)?;
        let name = match r.out_native(0) {
            Some(NativeObject::StringInfo(info)) => info.clone(),
            _ => StringInfo::default(),
        };
        Ok(ShortcutDisplay {
            name,
            icon: r.out_word(1),
            ok: r.retval != 0,
        })
    }

    /// `gmCWInterface::GetGameplayOptions`: a copy of the player's options profile (the
    /// shortcut bar reads `InqShortcutArray`, `GetWhichShortcutSet`, `GetShowShortcutNumbers`,
    /// `GetShortcutHeight` and `GetShortcutInstaScroll` from it).
    fn gameplay_options(&mut self) -> Result<GameplayOptionsProfile, String> {
        let args = [IFunArg::Out(OutValue::Native(
            NativeObject::GameplayOptionsProfile(Box::default()),
        ))];
        let r = self.query(ifun::GET_GAMEPLAY_OPTIONS, &args, IFunReturn::Word)?;
        match r.out_native(0) {
            Some(NativeObject::GameplayOptionsProfile(p)) => Ok((**p).clone()),
            _ => Err("no options profile".into()),
        }
    }

    /// `GameplayOptionsProfile::InqShortcutArray` of the player's profile.
    fn shortcut_array(&mut self) -> Result<Vec<ShortcutInfo>, String> {
        Ok(self.gameplay_options()?.shortcuts)
    }

    /// The shortcut bar's settings from the player's profile: `GetWhichShortcutSet`,
    /// `GetShowShortcutNumbers` (bit 0x1), `GetShortcutHeight`, `GetShortcutInstaScroll`
    /// (bit 0x2000).
    /// `gmCWInterface::GetShowObjectNames`: the options profile's names flag (0x200).
    fn show_object_names(&mut self) -> bool {
        self.gameplay_options()
            .is_ok_and(|p| p.bit_field & 0x200 != 0)
    }

    /// `gmCWInterface::GetShowRadarCoords`: the options profile's coordinates flag (0x10).
    fn show_radar_coords(&mut self) -> bool {
        self.gameplay_options()
            .is_ok_and(|p| p.bit_field & 0x10 != 0)
    }

    /// `gmCWInterface::GetAutoTarget`: the options profile's auto-target flag (0x20).
    fn auto_target(&mut self) -> bool {
        self.gameplay_options()
            .is_ok_and(|p| p.bit_field & 0x20 != 0)
    }

    fn shortcut_settings(&mut self) -> Result<ShortcutSettings, String> {
        let p = self.gameplay_options()?;
        Ok(ShortcutSettings {
            which_set: p.which_shortcut_set,
            show_numbers: p.bit_field & 0x1 != 0,
            height: p.shortcut_height,
            insta_scroll: p.bit_field & 0x2000 != 0,
            locked: p.bit_field & 0x800 != 0,
        })
    }

    /// The player's [`ChatSettings`] (`gmCWInterface::GetGameplayOptions`).
    fn chat_settings(&mut self) -> Result<ChatSettings, String> {
        let p = self.gameplay_options()?;
        Ok(ChatSettings {
            filters: words(&p.filters),
            font_sizes: words(&p.chat_font_sizes),
            font_colors: words(&p.chat_font_colors),
            popups: words(&p.chat_popup_flags),
            window_text_types: words(&p.version6_table),
            opacities: p
                .window_opacities
                .pairs
                .iter()
                .map(|&(window, bits)| (window, f32::from_bits(bits)))
                .collect(),
        })
    }
}

impl<P: ScriptPort + ?Sized> ScriptQueries for P {}

/// The chat windows' settings from the player's options profile, for
/// `CM_Communication::SendNotice_ReloadChatSettings`: per chat window (1-4) the text-type
/// filter (`GetFilter`), per text type the font size, colour and popup flag
/// (`GetChatFontSize` / `GetChatFontColor` / `GetChatPopupFlag`), and per window the text
/// type Legions' fifth table keeps (0x9B0054; absent windows answer 0x10).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatSettings {
    pub filters: HashMap<u32, u32>,
    pub font_sizes: HashMap<u32, u32>,
    pub font_colors: HashMap<u32, u32>,
    pub popups: HashMap<u32, u32>,
    pub window_text_types: HashMap<u32, u32>,
    /// `GameplayOptionsProfile::InqWindowOpacity`: each window's opacity (`0x800`, float bits).
    pub opacities: HashMap<u32, f32>,
}

fn words(t: &WordTable) -> HashMap<u32, u32> {
    t.pairs.iter().copied().collect()
}

/// The shortcut bar's settings (`GameplayOptionsProfile`'s getters).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutSettings {
    pub which_set: u32,
    pub show_numbers: bool,
    pub height: u32,
    pub insta_scroll: bool,
    /// `GetShortcutLocked` (0x800).
    pub locked: bool,
}
