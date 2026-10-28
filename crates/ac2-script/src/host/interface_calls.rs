//! The rest of `ggmCWInterface::CallIFun_*` as typed calls on the player's script: the UI's
//! getters of the gameplay options and the selection, the item facts the examination panel asks
//! of the selected object, the chat history, the examination panel's three calls, and the GUN
//! results and events the script hears.
//!
//! Every wrapper is one frame (`ProcessState::CreateContext`, the return slot, the parameters
//! in reverse, the instance) and one `VM::CallIFun`; [`ScriptQueries::query`] builds that frame
//! and reads the return slot back. A getter whose C++ signature names its result `context`
//! (`int *context`) reads the return slot, as the others read `retval`. Each method says which
//! wrapper it reproduces; the ids are the Legions 7160 `FunctionID`s (the same as admin 7148).

use ac2_net::protocol::{ExaminationRequest, NativeObject, StringInfo};

// The four history walkers' ids are passed to [`InterfaceCalls::inq_history_item`].
pub use ids::{
    INQ_NEXT_REPLY_HISTORY_ITEM, INQ_NEXT_RETELL_HISTORY_ITEM, INQ_PREV_REPLY_HISTORY_ITEM,
    INQ_PREV_RETELL_HISTORY_ITEM,
};

use super::queries::ScriptQueries;
use super::{IFunArg, IFunReturn, OutValue};
use crate::native::{ntype, NativeObject as VmNative};

/// The `FunctionID`s of the wrappers [`InterfaceCalls`] sends.
pub mod ids {
    pub const GET_SELECTION: u32 = 0x00B4_0003;
    pub const GET_LAST_SELECTION: u32 = 0x00B4_0004;
    pub const SHOULD_AUTO_TARGET: u32 = super::super::ifun_ids::IFUN_SHOULD_AUTO_TARGET;
    pub const SET_EXAMINATION_REQUEST: u32 = 0x00B4_0066;
    pub const START_EXAMINATION: u32 = 0x00B4_0067;
    pub const STOP_EXAMINATION: u32 = 0x00B4_0068;
    pub const HAS_SOMETHING_SELECTED: u32 = 0x00B4_0069;
    pub const REGISTER_COMMANDS: u32 = 0x00B4_006A;
    pub const UPDATE_OPEN: u32 = 0x00B4_006C;
    pub const INQ_ALIAS: u32 = 0x00B4_006E;
    pub const SET_WHICH_SHORTCUT_SET: u32 = super::super::ifun_ids::IFUN_SET_WHICH_SHORTCUT_SET;
    pub const GET_WHICH_SHORTCUT_SET: u32 = 0x00B4_0072;
    pub const GET_SHOW_SHORTCUT_NUMBERS: u32 = 0x00B4_0074;
    pub const GET_SHOW_DAMAGE_TEXT: u32 = 0x00B4_0076;
    pub const GET_SHOW_DAMAGE_TEXT_OTHER: u32 = 0x00B4_0078;
    pub const GET_DAMAGE_TEXT_RANGE_OTHER: u32 = 0x00B4_007A;
    pub const GET_SHOW_RADAR_COORDS: u32 = 0x00B4_007E;
    pub const GET_RADAR_MASK: u32 = 0x00B4_0080;
    pub const GET_AUTO_TARGET: u32 = 0x00B4_0086;
    pub const GET_ACCEPT_ALLEGIANCE_REQUEST: u32 = 0x00B4_0088;
    pub const GET_ACCEPT_FELLOWSHIP_REQUEST: u32 = 0x00B4_008A;
    pub const GET_ACCEPT_TRADE_REQUEST: u32 = 0x00B4_008C;
    pub const GET_AVAILABLE_EXPERIENCE: u32 = 0x00B4_001E;
    pub const GET_SHORTCUT_LOCKED: u32 = 0x00B4_00FC;
    pub const GET_SHOW_PK_DAMAGE: u32 = 0x00B4_00FE;
    pub const GET_LATENCY_INDICATOR_GRAPH: u32 = 0x00B4_0109;
    pub const GET_SHORTCUT_INSTA_SCROLL: u32 = 0x00B4_010C;
    pub const GET_BYPASS_GOLD_MELTING_CONFIRMATION_DLG: u32 = 0x00B4_00D7;
    pub const GET_SHOW_OBJECT_NAMES: u32 = 0x00B4_00CF;
    pub const GET_SHOW_FELLOWS_ON_MAP: u32 = 0x00B4_013F;
    pub const GET_SHORTCUT_HEIGHT: u32 = 0x00B4_0141;
    pub const GET_SHOW_PET_UI: u32 = 0x00B4_014A;
    pub const IS_FELLOWSHIP_MEMBER: u32 = 0x00B4_00A7;
    pub const DO_ASSISTED_TELL: u32 = 0x00B4_00A9;
    pub const INQ_NEXT_RETELL_HISTORY_ITEM: u32 = 0x00B4_00AB;
    pub const INQ_PREV_RETELL_HISTORY_ITEM: u32 = 0x00B4_00AC;
    pub const RESET_RETELL_HISTORY_INDEX: u32 = 0x00B4_00AD;
    pub const INQ_NEXT_REPLY_HISTORY_ITEM: u32 = 0x00B4_00AE;
    pub const INQ_PREV_REPLY_HISTORY_ITEM: u32 = 0x00B4_00AF;
    pub const RESET_REPLY_HISTORY_INDEX: u32 = 0x00B4_00B0;
    pub const GET_MISSILE_INFORMATION_FROM_WSL: u32 = 0x00B4_00BE;
    pub const GET_FX_AND_BEHAVIOR_INFORMATION_FROM_WSL: u32 = 0x00B4_00BF;
    pub const HEAR_TELL: u32 = 0x00B4_003C;
    pub const RESULT_SEND_TO_USER: u32 = 0x00B4_003D;
    pub const RESULT_SEND_TO_ROOM: u32 = 0x00B4_003E;
    pub const GUN_EVENT_HEAR_FROM_ROOM: u32 = 0x00B4_003F;
    pub const RESULT_IGNORED_ADD: u32 = 0x00B4_0045;
    pub const RESULT_IGNORED_DELETE: u32 = 0x00B4_0046;
    pub const RESULT_CALL_CREATE: u32 = 0x00B4_0048;
    pub const GUN_HANDLE_REAUTHENTICATION: u32 = 0x00B4_00D8;
}

fn text(s: &str) -> IFunArg {
    IFunArg::Text(s.to_owned())
}

fn out_text() -> IFunArg {
    IFunArg::Out(OutValue::Native(NativeObject::WString(String::new())))
}

fn out_string(result: &super::IFunResult, n: usize) -> String {
    match result.out_native(n) {
        Some(NativeObject::WString(s)) => s.clone(),
        _ => String::new(),
    }
}

/// `getter!(name, ID, ret, "wrapper", "what it answers")`: a no-argument call that reads the
/// return slot, as `ggmCWInterface::CallIFun_<wrapper>(process, T *context)` does.
macro_rules! getter {
    ($($name:ident => $id:path, $wrapper:literal;)*) => {$(
        #[doc = concat!("`ggmCWInterface::CallIFun_", $wrapper, "`: the return slot's word.")]
        fn $name(&mut self) -> Result<u32, String> {
            Ok(self.query($id, &[], IFunReturn::Word)?.retval as u32)
        }
    )*};
}

/// `request!`: a call with no results the UI reads (the wrapper has no `retval`).
macro_rules! request {
    ($($name:ident => $id:path, $wrapper:literal;)*) => {$(
        #[doc = concat!("`ggmCWInterface::CallIFun_", $wrapper, "`: no arguments, no result.")]
        fn $name(&mut self) -> Result<(), String> {
            self.query($id, &[], IFunReturn::Word).map(|_| ())
        }
    )*};
}

/// The `ggmCWInterface::CallIFun_*` wrappers not covered by [`ScriptQueries`]' own methods, on
/// any [`ScriptQueries`] (the client implements the port).
pub trait InterfaceCalls: ScriptQueries {
    getter! {
        has_something_selected => ids::HAS_SOMETHING_SELECTED, "HasSomethingSelected";
        should_auto_target => ids::SHOULD_AUTO_TARGET, "ShouldAutoTarget";
        auto_target_option => ids::GET_AUTO_TARGET, "GetAutoTarget";
        accept_allegiance_request => ids::GET_ACCEPT_ALLEGIANCE_REQUEST, "GetAcceptAllegianceRequest";
        accept_fellowship_request => ids::GET_ACCEPT_FELLOWSHIP_REQUEST, "GetAcceptFellowshipRequest";
        accept_trade_request => ids::GET_ACCEPT_TRADE_REQUEST, "GetAcceptTradeRequest";
        bypass_gold_melting_confirmation_dlg => ids::GET_BYPASS_GOLD_MELTING_CONFIRMATION_DLG, "GetBypassGoldMeltingConfirmationDlg";
        latency_indicator_graph => ids::GET_LATENCY_INDICATOR_GRAPH, "GetLatencyIndicatorGraph";
        radar_mask => ids::GET_RADAR_MASK, "GetRadarMask";
        shortcut_height => ids::GET_SHORTCUT_HEIGHT, "GetShortcutHeight";
        shortcut_insta_scroll => ids::GET_SHORTCUT_INSTA_SCROLL, "GetShortcutInstaScroll";
        shortcut_locked => ids::GET_SHORTCUT_LOCKED, "GetShortcutLocked";
        show_damage_text => ids::GET_SHOW_DAMAGE_TEXT, "GetShowDamageText";
        show_damage_text_other => ids::GET_SHOW_DAMAGE_TEXT_OTHER, "GetShowDamageTextOther";
        show_fellows_on_map => ids::GET_SHOW_FELLOWS_ON_MAP, "GetShowFellowsOnMap";
        show_object_names_option => ids::GET_SHOW_OBJECT_NAMES, "GetShowObjectNames";
        show_pk_damage => ids::GET_SHOW_PK_DAMAGE, "GetShowPKDamage";
        show_pet_ui => ids::GET_SHOW_PET_UI, "GetShowPetUI";
        show_radar_coords_option => ids::GET_SHOW_RADAR_COORDS, "GetShowRadarCoords";
        show_shortcut_numbers => ids::GET_SHOW_SHORTCUT_NUMBERS, "GetShowShortcutNumbers";
        which_shortcut_set => ids::GET_WHICH_SHORTCUT_SET, "GetWhichShortcutSet";
    }

    /// `ggmCWInterface::CallIFun_GetDamageTextRangeOther`: a `float` in the return slot.
    fn damage_text_range_other(&mut self) -> Result<f32, String> {
        let r = self.query(ids::GET_DAMAGE_TEXT_RANGE_OTHER, &[], IFunReturn::Word)?;
        Ok(f32::from_bits(r.retval as u32))
    }

    /// `ggmCWInterface::CallIFun_GetSelection`: the selected object (an `InstanceID`).
    fn selection(&mut self) -> Result<u64, String> {
        Ok(self
            .query(ids::GET_SELECTION, &[], IFunReturn::Long)?
            .retval)
    }

    /// `ggmCWInterface::CallIFun_GetLastSelection`: the previous selection.
    fn last_selection(&mut self) -> Result<u64, String> {
        Ok(self
            .query(ids::GET_LAST_SELECTION, &[], IFunReturn::Long)?
            .retval)
    }

    /// `ggmCWInterface::CallIFun_GetAvailableExperience`: a `ulong64`.
    fn available_experience(&mut self) -> Result<u64, String> {
        Ok(self
            .query(ids::GET_AVAILABLE_EXPERIENCE, &[], IFunReturn::Long)?
            .retval)
    }

    /// `ggmCWInterface::CallIFun_SetWhichShortcutSet`.
    fn set_which_shortcut_set(&mut self, set: u32) -> Result<(), String> {
        self.query(
            ids::SET_WHICH_SHORTCUT_SET,
            &[IFunArg::Int(set)],
            IFunReturn::Word,
        )
        .map(|_| ())
    }

    request! {
        stop_examination => ids::STOP_EXAMINATION, "StopExamination";
        register_commands => ids::REGISTER_COMMANDS, "RegisterCommands";
        reset_retell_history_index => ids::RESET_RETELL_HISTORY_INDEX, "ResetRetellHistoryIndex";
        reset_reply_history_index => ids::RESET_REPLY_HISTORY_INDEX, "ResetReplyHistoryIndex";
        gun_handle_reauthentication => ids::GUN_HANDLE_REAUTHENTICATION, "GUNHandleReauthentication";
    }

    /// `ggmCWInterface::CallIFun_StartExamination`: the wrapper also pushes one word it never
    /// sets (the decompile shows the caller's stack slot), tagged 4; zero here.
    fn start_examination(&mut self) -> Result<(), String> {
        self.query(ids::START_EXAMINATION, &[IFunArg::Int(0)], IFunReturn::Word)
            .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_SetExaminationRequest`: the request as a new native.
    fn set_examination_request(&mut self, request: ExaminationRequest) -> Result<(), String> {
        self.query(
            ids::SET_EXAMINATION_REQUEST,
            &[IFunArg::Native(NativeObject::ExaminationRequest(request))],
            IFunReturn::Word,
        )
        .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_UpdateOpen(updater, open)`: the `retval`.
    fn update_open(&mut self, updater: u64, open: bool) -> Result<u32, String> {
        let args = [IFunArg::Long(updater), IFunArg::Int(u32::from(open))];
        Ok(self
            .query(ids::UPDATE_OPEN, &args, IFunReturn::Word)?
            .retval as u32)
    }

    /// `ggmCWInterface::CallIFun_IsFellowshipMember(who)`.
    fn is_fellowship_member(&mut self, who: u64) -> Result<bool, String> {
        let r = self.query(
            ids::IS_FELLOWSHIP_MEMBER,
            &[IFunArg::Long(who)],
            IFunReturn::Word,
        )?;
        Ok(r.retval != 0)
    }

    /// `ggmCWInterface::CallIFun_DoAssistedTell(name)`.
    fn do_assisted_tell(&mut self, name: &str) -> Result<(), String> {
        self.query(ids::DO_ASSISTED_TELL, &[text(name)], IFunReturn::Word)
            .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_InqAlias(alias)`: the alias' text, when it has one.
    fn inq_alias(&mut self, alias: &str) -> Result<Option<String>, String> {
        let r = self.query(ids::INQ_ALIAS, &[text(alias), out_text()], IFunReturn::Word)?;
        Ok((r.retval != 0).then(|| out_string(&r, 1)))
    }

    /// The four history walkers (`ggmCWInterface::CallIFun_InqNextReplyHistoryItem`,
    /// `InqPrevReplyHistoryItem`, `InqNextRetellHistoryItem`, `InqPrevRetellHistoryItem`): the
    /// name, when the script has one in that direction.
    fn inq_history_item(&mut self, function: u32) -> Result<Option<String>, String> {
        let r = self.query(function, &[out_text()], IFunReturn::Word)?;
        Ok((r.retval != 0).then(|| out_string(&r, 0)))
    }

    /// `ggmCWInterface::CallIFun_GetFXAndBehaviorInformationFromWsl(context, hook, result)`:
    /// the `FXAndBehaviorInfo` the script filled.
    fn fx_and_behavior_info(
        &mut self,
        context: u32,
        hook: u32,
        attack_result: u32,
    ) -> Result<Option<VmNative>, String> {
        let args = [
            IFunArg::Int(context),
            IFunArg::Int(hook),
            IFunArg::Int(attack_result),
            IFunArg::Filled(ntype::FX_AND_BEHAVIOR_INFO),
        ];
        let r = self.query(
            ids::GET_FX_AND_BEHAVIOR_INFORMATION_FROM_WSL,
            &args,
            IFunReturn::Word,
        )?;
        Ok(match r.outs.get(3) {
            Some(Some(OutValue::Vm(n))) => Some(n.clone()),
            _ => None,
        })
    }

    /// `ggmCWInterface::CallIFun_GetMissileInformationFromWSL(skill, hook, attacker, target)`:
    /// the `MissileInfo` the script filled.
    fn missile_info(
        &mut self,
        skill: u32,
        hook: u32,
        attacker: u64,
        target: u64,
    ) -> Result<Option<VmNative>, String> {
        let args = [
            IFunArg::Int(skill),
            IFunArg::Int(hook),
            IFunArg::Long(attacker),
            IFunArg::Long(target),
            IFunArg::Filled(ntype::MISSILE_INFO),
        ];
        let r = self.query(
            ids::GET_MISSILE_INFORMATION_FROM_WSL,
            &args,
            IFunReturn::Word,
        )?;
        Ok(match r.outs.get(4) {
            Some(Some(OutValue::Vm(n))) => Some(n.clone()),
            _ => None,
        })
    }

    /// `ggmCWInterface::CallIFun_HearTell(teller, message)`: a tell heard over GUN.
    fn hear_tell(
        &mut self,
        context: u64,
        teller: StringInfo,
        message: StringInfo,
        chat_flags: u32,
    ) -> Result<(), String> {
        let args = [
            IFunArg::Long(context),
            IFunArg::Native(NativeObject::StringInfo(teller)),
            IFunArg::Native(NativeObject::StringInfo(message)),
            IFunArg::Int(chat_flags),
        ];
        self.query(ids::HEAR_TELL, &args, IFunReturn::Word)
            .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_GUNEvent_HearFromRoom(speaker, message, room, flags)`.
    fn gun_hear_from_room(
        &mut self,
        speaker: StringInfo,
        message: &str,
        room: u32,
        chat_flags: u32,
    ) -> Result<(), String> {
        let args = [
            IFunArg::Native(NativeObject::StringInfo(speaker)),
            text(message),
            IFunArg::Int(room),
            IFunArg::Int(chat_flags),
        ];
        self.query(ids::GUN_EVENT_HEAR_FROM_ROOM, &args, IFunReturn::Word)
            .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_Result_SendToUser(user, message, result)`.
    fn result_send_to_user(
        &mut self,
        user: &str,
        message: &str,
        result: u32,
    ) -> Result<(), String> {
        let args = [text(user), text(message), IFunArg::Int(result)];
        self.query(ids::RESULT_SEND_TO_USER, &args, IFunReturn::Word)
            .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_Result_SendToRoom(text, room, result)`.
    fn result_send_to_room(&mut self, message: &str, room: u32, result: u32) -> Result<(), String> {
        let args = [text(message), IFunArg::Int(room), IFunArg::Int(result)];
        self.query(ids::RESULT_SEND_TO_ROOM, &args, IFunReturn::Word)
            .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_Result_IgnoredAdd(result, name)`.
    fn result_ignored_add(&mut self, result: u32, name: &str) -> Result<(), String> {
        self.query(
            ids::RESULT_IGNORED_ADD,
            &[IFunArg::Int(result), text(name)],
            IFunReturn::Word,
        )
        .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_Result_IgnoredDelete(result, name)`.
    fn result_ignored_delete(&mut self, result: u32, name: &str) -> Result<(), String> {
        self.query(
            ids::RESULT_IGNORED_DELETE,
            &[IFunArg::Int(result), text(name)],
            IFunReturn::Word,
        )
        .map(|_| ())
    }

    /// `ggmCWInterface::CallIFun_Result_CallCreate(result, type, user, description, queue, pos)`.
    fn result_call_create(
        &mut self,
        result: u32,
        call_type: u32,
        user: &str,
        description: &str,
        queue_id: u32,
        queue_pos: u32,
    ) -> Result<(), String> {
        let args = [
            IFunArg::Int(result),
            IFunArg::Int(call_type),
            text(user),
            text(description),
            IFunArg::Int(queue_id),
            IFunArg::Int(queue_pos),
        ];
        self.query(ids::RESULT_CALL_CREATE, &args, IFunReturn::Word)
            .map(|_| ())
    }
}

impl<P: ScriptQueries + ?Sized> InterfaceCalls for P {}
