//! `gGameTime` (`portal/shared/src/interp/glue/ggametime.cpp`): the game clock's calendar, read
//! from `GameTime::current_game_time` (the state the client hands the host each frame,
//! [`crate::host::game_time::GameTimeState`]). With no game time every getter answers zero, as
//! retail does when the global is null.
//!
//! `AlterTimeOfDayImmediate` and `TemporalDilation` change the clock, which the client owns: they
//! queue a [`GameTimeRequest`] and answer what `GameTime` does (`TemporalDilationInternal` refuses
//! a dilation of `0.0002` or less). `InqGameDateTimeString` (a time stamp's date) is not bound:
//! no library code calls it.

use super::string_info::{add_var, table_id, STRING_TABLE_ENUM};
use super::wpstring::tokenize_wide;
use super::{out_native, Call, INT_RESULT};
use crate::host::game_time::{GameDate, GameTimeRequest, GameTimeState};
use crate::native::{StringInfo, StringVarValue};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};

/// The string table enum `GameTime::GetDateTimeString` names its strings with.
const GAME_TIME_TABLE: u32 = 0x4000_0002;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x002C_0001, |vm, ctx| {
        state_word(vm, ctx, |s| u32::from(s.night))
    });
    efuns.register(0x002C_0002, get_time_of_day);
    efuns.register(0x002C_0003, |vm, ctx| {
        state_word(vm, ctx, |s| s.day_length.to_bits())
    });
    efuns.register(0x002C_0004, |vm, ctx| {
        state_word(vm, ctx, |s| s.day_of_year as u32)
    });
    efuns.register(0x002C_0005, |vm, ctx| {
        state_word(vm, ctx, |s| s.days_per_year as u32)
    });
    efuns.register(0x002C_0006, |vm, ctx| {
        state_word(vm, ctx, |s| s.year as u32)
    });
    efuns.register(0x002C_0008, inq_current_date_time);
    efuns.register(0x002C_0009, alter_time_of_day);
    efuns.register(0x002C_000A, temporal_dilation);
}

/// A getter with no arguments: the word of the state (0 without game time) in word 2.
fn state_word(vm: &mut Vm, ctx: &mut Context, read: fn(&GameTimeState) -> u32) -> InterpStatus {
    let word = vm.game_time.as_ref().map_or(0, read);
    Call::new(ctx).ret(2, word, INT_RESULT);
    InterpStatus::Success
}

/// `GetTimeOfDay`: `present_time_of_day` widened to a double (words 2 and 3, high word first).
fn get_time_of_day(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = vm
        .game_time
        .as_ref()
        .map_or(0.0, |s| f64::from(s.time_of_day));
    let bits = value.to_bits();
    let high_tag = ctx.arg_tag(2).unwrap_or(tag::UNINIT);
    let mut call = Call::new(ctx);
    call.ret(3, bits as u32, tag::BOOL | tag::LONG);
    call.ret(2, (bits >> 32) as u32, high_tag);
    InterpStatus::Success
}

/// `AlterTimeOfDayImmediate(float)`: 1 when there is a game time, and the clock is asked to move.
fn alter_time_of_day(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let fraction = f32::from_bits(Call::new(ctx).word(2));
    let ok = vm.game_time.is_some();
    if ok {
        vm.game_time_requests
            .push(GameTimeRequest::AlterTimeOfDay(fraction));
    }
    Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

/// `TemporalDilation(float)`: refused (0) for a dilation of `0.0002` or less
/// (`GameTime::TemporalDilationInternal`), or without a game time.
fn temporal_dilation(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let dilation = f32::from_bits(Call::new(ctx).word(2));
    let ok = vm.game_time.is_some() && !(dilation <= 0.0002);
    if ok {
        vm.game_time_requests
            .push(GameTimeRequest::TemporalDilation(dilation));
    }
    Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
    InterpStatus::Success
}

fn token(name: &str) -> u32 {
    let wide: Vec<u16> = name.encode_utf16().collect();
    tokenize_wide(&wide)
}

/// `InqCurrentGameDateTimeString(date StringInfo, time StringInfo)`
/// (`GameTime::GetDateTimeString` at the running clock): `ID_DateString` with `YEAR`, `YEARSPEC`,
/// `DAY` and `SEASON`, and `ID_TimeString` with `TIME`, both of the game-time table; the string
/// ids are `StringInfo::AddVariable_StringID` variables (nested strings of the table).
fn inq_current_date_time(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let call = Call::new(ctx);
    let (date_ref, time_ref) = (call.word(2), call.word(3));
    let Some(state) = vm.game_time else {
        Call::new(ctx).ret(4, 0, INT_RESULT);
        return InterpStatus::Success;
    };
    let did = vm
        .data()
        .enum_did(GAME_TIME_TABLE, STRING_TABLE_ENUM)
        .unwrap_or(0);
    let table = table_id(vm, did);
    let (date, time) = date_time_strings(&state.date, table);
    for (reference, info) in [(date_ref, date), (time_ref, time)] {
        let target = out_native(vm, ctx, reference);
        match vm.native_mut(target) {
            Some(crate::native::NativeObject::StringInfo(out)) => *out = info,
            _ => return vm.bad_this(target),
        }
    }
    Call::new(ctx).ret(4, 1, INT_RESULT);
    InterpStatus::Success
}

/// The two `StringInfo`s `GameTime::GetDateTimeString` makes.
fn date_time_strings(date: &GameDate, table: u32) -> (StringInfo, StringInfo) {
    let name = |id: u32| {
        StringVarValue::StringInfo(Box::new(StringInfo {
            string_id: id,
            table_id: table,
            ..StringInfo::default()
        }))
    };
    let mut date_info = StringInfo {
        string_id: token("ID_DateString"),
        table_id: table,
        ..StringInfo::default()
    };
    let mut time_info = StringInfo {
        string_id: token("ID_TimeString"),
        table_id: table,
        ..StringInfo::default()
    };
    add_var(&mut time_info, token("TIME"), 9, name(date.time));
    add_var(
        &mut date_info,
        token("YEAR"),
        2,
        StringVarValue::Int(date.year),
    );
    add_var(&mut date_info, token("YEARSPEC"), 9, name(date.year_spec));
    add_var(
        &mut date_info,
        token("DAY"),
        2,
        StringVarValue::Int(date.day),
    );
    add_var(&mut date_info, token("SEASON"), 9, name(date.season));
    (date_info, time_info)
}
