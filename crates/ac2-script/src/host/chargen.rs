//! `gCharacterGenSystem`: the native side's calls into the `CharacterGenSystem` singleton
//! script (`gCharacterGenSystem::CallIFun_*`), which the character creation screens and
//! `gmCharGenResult` use.
//!
//! Each call is `VM::CreateSingletonProcess("CharacterGenSystem")`, a new context, an optional
//! out object with a 0 return slot and a stack address of that object after it, the inputs,
//! the instance, then `VM::CallIFun(instance package, function)`; the return slot and the out
//! object are read back.

use ac2_net::protocol::{GmRaceSexInfo, NativeObject as Wire, StringInfo as WireStringInfo};

use crate::native::{ntype, CharGenResult, NativeObject, Position, ScriptHash, ScriptList};
use crate::vm::refs::{flags, Target};
use crate::vm::{tag, Context, FunctionId, InterpStatus, Vm};
use crate::wire::string_info_to_net;
use ac2_weenie::Appearance;

const SYSTEM: &str = "CharacterGenSystem";

/// The `CharacterGenSystem` IFun ids.
pub mod function {
    pub const INQ_ADMIN_AVATAR_ENTITY_DID: u32 = 0x0096_0001;
    pub const INQ_PLAYER_AVATAR_ENTITY_DID: u32 = 0x0096_0002;
    pub const INQ_RACE_SEX_DATA: u32 = 0x0096_0005;
    pub const INQ_RACE_SEX_INFO_DATA: u32 = 0x0096_0006;
    pub const INQ_PHYS_TYPE_LABEL: u32 = 0x0096_0007;
    pub const INQ_VALID_PHYS_TYPE_VALUES: u32 = 0x0096_0008;
    pub const INQ_VALID_STARTING_AREAS: u32 = 0x0096_0009;
    pub const INQ_STARTING_POSITION: u32 = 0x0096_000A;
    pub const GET_PHYS_OBJ: u32 = 0x0096_000B;
    pub const IS_VALID: u32 = 0x0096_000C;
    pub const SET_CGR_DEFAULTS: u32 = 0x0096_000E;
    pub const RANDOMIZE_CGR: u32 = 0x0096_000F;
    pub const FORMAT_NAME_STRING: u32 = 0x0096_0011;
    pub const INQ_APPLIED_APPEARANCE_VDESC: u32 = 0x0096_0010;
}

/// A value the native side pushes: an int (tag 4) or a reference (tag 1).
#[derive(Debug, Clone, Copy)]
pub(crate) enum Push {
    Int(u32),
    Ref(u32),
}

/// How a system call ended: the status, the return slot, the out slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemCall {
    pub status: InterpStatus,
    pub retval: u32,
    pub out: u32,
}

/// One IFun call into a singleton system script (`gCharacterGenSystem::CallIFun_*`'s frame).
pub(crate) fn call_system(
    vm: &mut Vm,
    package: &str,
    function: u32,
    out: Option<(u32, u8)>,
    args: &[Push],
) -> Option<SystemCall> {
    let pid = vm.program.package_id(package)?;
    let instance = vm.singleton_package(pid);
    let mut ctx = Context::new(0, Some(instance));
    let out_slot = ctx.sp;
    let ret_slot = match out {
        Some((value, t)) => {
            ctx.push(value, t).ok()?;
            ctx.push(0, tag::INT).ok()?;
            let blob = flags::KIND_VALUE | ((out_slot as u32 & 0xFFFF) << flags::OFFSET_SHIFT);
            let address = vm.refs.add(blob, Target::Ref(u32::MAX));
            ctx.push(address, tag::REF).ok()?;
            out_slot + 1
        }
        None => {
            ctx.push(0, tag::INT).ok()?;
            out_slot
        }
    };
    for arg in args {
        match *arg {
            Push::Int(v) => ctx.push(v, tag::BOOL).ok()?,
            Push::Ref(r) => ctx.push(r, tag::REF).ok()?,
        }
    }
    ctx.push(instance, tag::REF).ok()?;
    let pkg = vm.package_id(instance).ok()?;
    vm.last_fault = None;
    let status = vm.call_ifun(&mut ctx, pkg, FunctionId(function));
    Some(SystemCall {
        status,
        retval: ctx.slot(ret_slot).ok()?.0,
        out: ctx.slot(out_slot).ok()?.0,
    })
}

fn native(vm: &mut Vm, t: u32, n: NativeObject) -> u32 {
    vm.refs.create_native(t, n)
}

/// `CallIFun_InqAdminAvatarEntityDID` / `InqPlayerAvatarEntityDID`: the avatar entity DID.
pub fn avatar_entity_did(vm: &mut Vm, admin: bool) -> Option<u32> {
    let f = if admin {
        function::INQ_ADMIN_AVATAR_ENTITY_DID
    } else {
        function::INQ_PLAYER_AVATAR_ENTITY_DID
    };
    Some(call_system(vm, SYSTEM, f, Some((0, tag::BOOL)), &[])?.out)
}

/// `CallIFun_GetPhysObj(race, sex)`.
pub fn phys_obj(vm: &mut Vm, race: u32, sex: u32) -> Option<u32> {
    let args = [Push::Int(race), Push::Int(sex)];
    Some(call_system(vm, SYSTEM, function::GET_PHYS_OBJ, None, &args)?.retval)
}

/// `CallIFun_InqPhysTypeLabel(race, sex, physType)`: (result, label).
pub fn phys_type_label(
    vm: &mut Vm,
    race: u32,
    sex: u32,
    phys_type: u32,
) -> Option<(u32, WireStringInfo)> {
    let out = native(
        vm,
        ntype::STRING_INFO,
        NativeObject::new_empty(ntype::STRING_INFO),
    );
    let args = [Push::Int(race), Push::Int(sex), Push::Int(phys_type)];
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_PHYS_TYPE_LABEL,
        Some((out, tag::REF)),
        &args,
    )?;
    let label = match vm.native(call.out) {
        Some(NativeObject::StringInfo(info)) => string_info_to_net(info),
        _ => WireStringInfo::default(),
    };
    Some((call.retval, label))
}

/// An `ARHash` out container.
fn ar_hash(vm: &mut Vm) -> u32 {
    native(
        vm,
        ntype::AR_HASH,
        NativeObject::Hash(ScriptHash::default()),
    )
}

fn hash_pairs(vm: &Vm, r: u32) -> Vec<(u32, u32)> {
    match vm.native(r) {
        Some(NativeObject::Hash(h)) => h
            .nodes()
            .filter_map(|n| match n.key {
                crate::native::RefKey::Value(k) => Some((k as u32, n.value as u32)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn list_items(vm: &Vm, r: u32) -> Vec<u32> {
    match vm.native(r) {
        Some(NativeObject::List(l)) => l.items.iter().map(|v| *v as u32).collect(),
        _ => Vec::new(),
    }
}

/// `CallIFun_InqRaceSexData` (`ARHashToRaceSexListHash`): each race's sexes.
pub fn race_sex_data(vm: &mut Vm) -> Option<(u32, Vec<(u32, Vec<u32>)>)> {
    let out = ar_hash(vm);
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_RACE_SEX_DATA,
        Some((out, tag::REF)),
        &[],
    )?;
    let data = hash_pairs(vm, call.out)
        .into_iter()
        .map(|(k, list)| (k, list_items(vm, list)))
        .collect();
    Some((call.retval, data))
}

/// `CallIFun_InqRaceSexInfoData` (`ARHashToRaceSexInfoHash`): per race-sex key its
/// `gmRaceSexInfo`.
pub fn race_sex_info_data(vm: &mut Vm) -> Option<(u32, Vec<(u32, GmRaceSexInfo)>)> {
    let out = ar_hash(vm);
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_RACE_SEX_INFO_DATA,
        Some((out, tag::REF)),
        &[],
    )?;
    let data = hash_pairs(vm, call.out)
        .into_iter()
        .filter_map(|(k, r)| match vm.native(r) {
            Some(NativeObject::Wire(w)) => match &**w {
                Wire::GmRaceSexInfo(info) => Some((k, info.clone())),
                _ => None,
            },
            _ => None,
        })
        .collect();
    Some((call.retval, data))
}

/// `CallIFun_InqStartingPosition(area)`.
pub fn starting_position(vm: &mut Vm, area: u32) -> Option<(u32, Position)> {
    let out = native(
        vm,
        ntype::POSITION,
        NativeObject::new_empty(ntype::POSITION),
    );
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_STARTING_POSITION,
        Some((out, tag::REF)),
        &[Push::Int(area)],
    )?;
    let pos = match vm.native(call.out) {
        Some(NativeObject::Position(p)) => *p,
        _ => return None,
    };
    Some((call.retval, pos))
}

/// `CallIFun_InqAppliedAppearanceVDesc(physMask, cgr, aaVDescOut)`: the appearance the result's
/// physique values apply, written onto `start` (`VisualDesc::VisualDesc(out)` copies the
/// caller's description first). The screens ask for every physique type (`0xFFFFFFFF`).
/// Returns the script's result and the description.
pub fn applied_appearance(
    vm: &mut Vm,
    start: Appearance,
    cgr: &CharGenResult,
    phys_mask: u32,
) -> Option<(u32, Appearance)> {
    let out = native(
        vm,
        ntype::VISUAL_DESC,
        NativeObject::VisualDesc(Box::new(start)),
    );
    let cgr_ref = native(
        vm,
        ntype::CHAR_GEN_RESULT,
        NativeObject::CharGenResult(Box::new(cgr.clone())),
    );
    let args = [Push::Ref(cgr_ref), Push::Int(phys_mask)];
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_APPLIED_APPEARANCE_VDESC,
        Some((out, tag::REF)),
        &args,
    )?;
    let desc = match vm.native(call.out) {
        Some(NativeObject::VisualDesc(desc)) => (**desc).clone(),
        _ => return None,
    };
    Some((call.retval, desc))
}

fn a_list(vm: &mut Vm) -> u32 {
    native(vm, ntype::A_LIST, NativeObject::List(ScriptList::default()))
}

/// `CallIFun_InqValidPhysTypeValues(race, sex, physType)`: the slider's values.
pub fn valid_phys_type_values(
    vm: &mut Vm,
    race: u32,
    sex: u32,
    phys_type: u32,
) -> Option<(u32, Vec<f32>)> {
    let out = a_list(vm);
    let args = [Push::Int(race), Push::Int(sex), Push::Int(phys_type)];
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_VALID_PHYS_TYPE_VALUES,
        Some((out, tag::REF)),
        &args,
    )?;
    let values = list_items(vm, call.out)
        .into_iter()
        .map(f32::from_bits)
        .collect();
    Some((call.retval, values))
}

/// `CallIFun_InqValidStartingAreas(race, classType)`.
pub fn valid_starting_areas(vm: &mut Vm, race: u32, class: u32) -> Option<(u32, Vec<u32>)> {
    let out = a_list(vm);
    let args = [Push::Int(race), Push::Int(class)];
    let call = call_system(
        vm,
        SYSTEM,
        function::INQ_VALID_STARTING_AREAS,
        Some((out, tag::REF)),
        &args,
    )?;
    Some((call.retval, list_items(vm, call.out)))
}

fn cgr_native(vm: &mut Vm, cgr: &CharGenResult) -> u32 {
    let mut copy = CharGenResult::default();
    copy.assign(cgr);
    native(
        vm,
        ntype::CHAR_GEN_RESULT,
        NativeObject::CharGenResult(Box::new(copy)),
    )
}

/// `CallIFun_IsValid(cgr)`: 0 when valid, else an error code.
pub fn is_valid(vm: &mut Vm, cgr: &CharGenResult) -> Option<u32> {
    let r = cgr_native(vm, cgr);
    Some(call_system(vm, SYSTEM, function::IS_VALID, None, &[Push::Ref(r)])?.retval)
}

/// `CallIFun_RandomizeCGR` / `SetCGRDefaults`: the script edits a copy, which is assigned back
/// (`gmCharGenResult::operator=`).
fn edit_cgr(vm: &mut Vm, f: u32, args: &[Push], cgr: &mut CharGenResult) -> Option<u32> {
    let r = cgr_native(vm, cgr);
    let call = call_system(vm, SYSTEM, f, Some((r, tag::REF)), args)?;
    if let Some(NativeObject::CharGenResult(result)) = vm.native(call.out) {
        let result = (**result).clone();
        cgr.assign(&result);
    }
    Some(call.retval)
}

pub fn randomize(vm: &mut Vm, phys_mask: u32, cgr: &mut CharGenResult) -> Option<u32> {
    edit_cgr(vm, function::RANDOMIZE_CGR, &[Push::Int(phys_mask)], cgr)
}

pub fn set_defaults(vm: &mut Vm, cgr: &mut CharGenResult) -> Option<u32> {
    edit_cgr(vm, function::SET_CGR_DEFAULTS, &[], cgr)
}

/// `CallIFun_FormatNameString(nameIn)`: (result, formatted name).
pub fn format_name(vm: &mut Vm, name: &[u16]) -> Option<(u32, Vec<u16>)> {
    let out = vm.new_wstring(Vec::new());
    let input = vm.new_wstring(name.to_vec());
    let call = call_system(
        vm,
        SYSTEM,
        function::FORMAT_NAME_STRING,
        Some((out, tag::REF)),
        &[Push::Ref(input)],
    )?;
    let text = vm
        .wstring(call.out)
        .map(<[u16]>::to_vec)
        .unwrap_or_default();
    Some((call.retval, text))
}
