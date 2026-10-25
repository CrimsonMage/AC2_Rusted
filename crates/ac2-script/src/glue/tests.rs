//! Pure glue through the efun calling convention.

use std::sync::Arc;

use super::*;
use crate::native::{StringInfo, StringVarValue};
use crate::vm::{NoData, Program};

fn vm() -> Vm {
    let mut efuns = EFuns::new();
    register_pure(&mut efuns);
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    Vm::new(Arc::new(program), Box::new(NoData), efuns)
}

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Calls efun `id` with `returns` zeroed return slots, `args` (first argument first) and
/// `this`; gives back the return slots, deepest first.
fn call(
    vm: &mut Vm,
    id: u32,
    returns: usize,
    args: &[u32],
    this: u32,
) -> (InterpStatus, Vec<(u32, u8)>) {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    for _ in 0..returns {
        ctx.push(0, tag::INT).unwrap();
    }
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let status = vm.run_efun(id, &mut ctx);
    let rets = (0..returns).map(|i| (ctx.data[i], ctx.tags[i])).collect();
    (status, rets)
}

#[test]
fn wcc_concatenation_formats_like_the_crt() {
    let mut vm = vm();
    let hp = vm.new_wstring(w("HP: "));
    let (status, rets) = call(&mut vm, 0x0074_0002, 1, &[hp, -12i32 as u32, 0], u32::MAX);
    assert_eq!(status, InterpStatus::Success);
    assert_eq!(rets[0].1, tag::REF);
    assert_eq!(vm.wstring(rets[0].0).unwrap(), w("HP: -12").as_slice());
    let (_, rets) = call(
        &mut vm,
        0x0074_0004,
        1,
        &[hp, 2.5f32.to_bits(), 1],
        u32::MAX,
    );
    assert_eq!(vm.wstring(rets[0].0).unwrap(), w("2.500000HP: ").as_slice());
    let tail = vm.new_wstring(w("!"));
    let (_, rets) = call(&mut vm, 0x0074_0001, 1, &[hp, tail, 0], u32::MAX);
    assert_eq!(vm.wstring(rets[0].0).unwrap(), w("HP: !").as_slice());
    let (_, rets) = call(&mut vm, 0x0074_0008, 1, &[0xBEEF], u32::MAX);
    assert_eq!(vm.wstring(rets[0].0).unwrap(), w("0x0000BEEF").as_slice());
    // ToIID: the 64-bit argument's low word sits deeper (it is "argument 2" of 2 slots).
    let (_, rets) = call(
        &mut vm,
        0x0074_000A,
        1,
        &[0x0001_0002, 0x0003_0004],
        u32::MAX,
    );
    assert_eq!(
        vm.wstring(rets[0].0).unwrap(),
        w("0001:0002:0003:0004").as_slice()
    );
}

#[test]
fn wpstring_keeps_the_retail_quirks() {
    let mut vm = vm();
    let s = vm.new_wstring(w("Hello"));
    let (_, rets) = call(&mut vm, 0x0076_0003, 1, &[], s);
    assert_eq!(rets[0], (6, INT_RESULT), "length counts the terminator");
    let other = vm.new_wstring(w("hello"));
    let (_, rets) = call(&mut vm, 0x0076_0011, 1, &[other, 0], s);
    assert_eq!(rets[0].0 & 0xFF, 1, "case-insensitive eq");
    let (_, rets) = call(&mut vm, 0x0076_0011, 1, &[other, 1], s);
    assert_eq!(rets[0].0 & 0xFF, 0, "case-sensitive eq");
    let (_, rets) = call(&mut vm, 0x0076_000E, 1, &[1, 99], s);
    assert_eq!(vm.wstring(rets[0].0).unwrap(), w("ello").as_slice());
    let l = vm.new_wstring(w("l"));
    let big_l = vm.new_wstring(w("L"));
    let (_, rets) = call(&mut vm, 0x0076_0020, 1, &[l, big_l], s);
    assert_eq!(rets[0].0, 2);
    assert_eq!(vm.wstring(s).unwrap(), w("HeLLo").as_slice());
    let n = vm.new_wstring(w(" 0x10 "));
    let (_, rets) = call(&mut vm, 0x0076_0017, 1, &[], n);
    assert_eq!(rets[0].0, 16);
    call(&mut vm, 0x0076_000F, 0, &[], n);
    assert_eq!(vm.wstring(n).unwrap(), w("0x10").as_slice());
    // A null `this` crashes, as the retail assert does.
    let (status, _) = call(&mut vm, 0x0076_0003, 1, &[], u32::MAX);
    assert_eq!(status, InterpStatus::Crash);
}

#[test]
fn string_info_glue_builds_the_native() {
    let mut vm = vm();
    let info = vm.refs.create_native(
        crate::native::ntype::STRING_INFO,
        NativeObject::StringInfo(StringInfo::default()),
    );
    let lit = vm.new_wstring(w("a[b]"));
    call(&mut vm, 0x006A_0012, 0, &[lit], info);
    call(&mut vm, 0x006A_000A, 0, &[0x1234], info);
    call(&mut vm, 0x006A_0003, 0, &[7, 5], info);
    call(&mut vm, 0x006A_0003, 0, &[7, 6], info);
    let Some(NativeObject::StringInfo(si)) = vm.native(info) else {
        panic!();
    };
    assert_eq!(si.literal.as_deref(), Some(w("a\\[b\\]").as_slice()));
    assert_eq!(si.string_id, 0x1234);
    assert_eq!(si.variables.len(), 1, "a variable id is replaced");
    assert_eq!(si.variables[0].value, StringVarValue::Int(6));
    let (_, rets) = call(&mut vm, 0x006A_0013, 1, &[], info);
    assert_eq!(rets[0], (1, INT_RESULT));
}

#[test]
fn the_name_table_covers_the_dispatchers() {
    assert_eq!(efun_name(0x0001_0001), Some("gAAHash::CallEFun_add"));
    assert_eq!(
        efun_name(0x0074_0001),
        Some("gwccinternal::CallEFun__WCC_strcat_str")
    );
    assert_eq!(efun_ids().count(), 2112);
}
