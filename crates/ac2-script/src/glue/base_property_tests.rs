//! `gBaseProperty` glue through the efun calling convention.

use std::sync::Arc;

use super::*;
use crate::native::{StringInfo, StringVar, StringVarValue};
use crate::vm::refs::{flags, Target};
use crate::vm::{NoData, Program, ReferenceTable};
use ac2_db::{BaseProperty, BasePropertyValue};

fn vm() -> Vm {
    let mut efuns = EFuns::new();
    register_pure(&mut efuns);
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    Vm::new(Arc::new(program), Box::new(NoData), efuns)
}

/// Calls efun `id` with one return slot, `args` and `this`; gives back the return word.
fn call(vm: &mut Vm, id: u32, args: &[u32], this: u32) -> (InterpStatus, u32) {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    ctx.push(0, tag::INT).unwrap();
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let status = vm.run_efun(id, &mut ctx);
    (status, ctx.data[0])
}

/// A one-word heap slot the out argument names.
fn out_slot(vm: &mut Vm) -> u32 {
    let object = ReferenceTable::new_instance(0, 1);
    vm.refs.add(
        flags::FILLED | flags::KIND_HEAP,
        Target::Heap(Box::new(object)),
    )
}

fn slot_word(vm: &Vm, out: u32) -> u32 {
    vm.refs.heap(out).unwrap().slots[0]
}

fn property(value: Option<BasePropertyValue>) -> BaseProperty {
    let mut p = BaseProperty::new(7);
    if let Some(v) = value {
        p.value_type = v.value_type();
        p.value = Some(v);
    }
    p
}

fn property_ref(vm: &mut Vm, p: BaseProperty) -> u32 {
    vm.refs.create_native(
        ntype::BASE_PROPERTY,
        NativeObject::BaseProperty(Box::new(p)),
    )
}

fn id(name: &str) -> u32 {
    efun_table().find(|(_, n)| *n == name).unwrap().0
}

#[test]
fn bool_inq_and_set_follow_the_value_type() {
    let mut vm = vm();
    let p = property_ref(&mut vm, property(Some(BasePropertyValue::Bool(false))));
    let (_, set) = call(&mut vm, id("gBaseProperty::CallEFun_SetBool"), &[1], p);
    assert_eq!(set, 1);
    let out = out_slot(&mut vm);
    let (_, got) = call(&mut vm, id("gBaseProperty::CallEFun_InqBool"), &[out], p);
    assert_eq!(got, 1);
    assert_eq!(slot_word(&vm, out), 1);
    // A wrong-type setter writes nothing and answers 0.
    let (_, wrong) = call(&mut vm, id("gBaseProperty::CallEFun_SetInteger"), &[5], p);
    assert_eq!(wrong, 0);
}

#[test]
fn a_property_without_a_value_answers_zero_and_leaves_the_out_alone() {
    let mut vm = vm();
    let p = property_ref(&mut vm, property(None));
    let out = out_slot(&mut vm);
    let (_, got) = call(&mut vm, id("gBaseProperty::CallEFun_InqInteger"), &[out], p);
    assert_eq!(got, 0);
    assert_eq!(slot_word(&vm, out), 0xCDCD_CDCD);
    let (_, set) = call(&mut vm, id("gBaseProperty::CallEFun_SetInteger"), &[3], p);
    assert_eq!(set, 0);
}

#[test]
fn integer_float_enum_dataid_and_packageid_round_trip() {
    let mut vm = vm();
    let p = property_ref(&mut vm, property(Some(BasePropertyValue::Integer(0))));
    let out = out_slot(&mut vm);
    call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetInteger"),
        &[(-4i32) as u32],
        p,
    );
    let (_, got) = call(&mut vm, id("gBaseProperty::CallEFun_InqInteger"), &[out], p);
    assert_eq!(got, 1);
    assert_eq!(slot_word(&vm, out) as i32, -4);

    let f = property_ref(&mut vm, property(Some(BasePropertyValue::Float(0.0))));
    call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetFloat"),
        &[1.5f32.to_bits()],
        f,
    );
    call(&mut vm, id("gBaseProperty::CallEFun_InqFloat"), &[out], f);
    assert_eq!(f32::from_bits(slot_word(&vm, out)), 1.5);

    let e = property_ref(&mut vm, property(Some(BasePropertyValue::Enum(0))));
    call(&mut vm, id("gBaseProperty::CallEFun_SetEnum"), &[9], e);
    call(&mut vm, id("gBaseProperty::CallEFun_InqEnum"), &[out], e);
    assert_eq!(slot_word(&vm, out), 9);

    let d = property_ref(&mut vm, property(Some(BasePropertyValue::DataFile(0))));
    call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetDataID"),
        &[0x1234_5678],
        d,
    );
    call(&mut vm, id("gBaseProperty::CallEFun_InqDataID"), &[out], d);
    assert_eq!(slot_word(&vm, out), 0x1234_5678);

    let k = property_ref(&mut vm, property(Some(BasePropertyValue::PackageId(0))));
    call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetPackageID"),
        &[42],
        k,
    );
    call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqPackageID"),
        &[out],
        k,
    );
    assert_eq!(slot_word(&vm, out), 42);
}

#[test]
fn vector_efuns_copy_through_vector_natives() {
    let mut vm = vm();
    let p = property_ref(
        &mut vm,
        property(Some(BasePropertyValue::Vector(Default::default()))),
    );
    let src = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([1.0, 2.0, 3.0]));
    let (_, set) = call(&mut vm, id("gBaseProperty::CallEFun_SetVector"), &[src], p);
    assert_eq!(set, 1);
    let dst = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([0.0; 3]));
    let (_, got) = call(&mut vm, id("gBaseProperty::CallEFun_InqVector"), &[dst], p);
    assert_eq!(got, 1);
    assert_eq!(vm.native(dst), Some(&NativeObject::Vector([1.0, 2.0, 3.0])));
}

#[test]
fn string_efuns_narrow_on_set_and_widen_on_inq() {
    let mut vm = vm();
    let p = property_ref(
        &mut vm,
        property(Some(BasePropertyValue::String(Vec::new()))),
    );
    let text = vm.new_wstring("abc".encode_utf16().collect());
    let (_, set) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetStringW"),
        &[text],
        p,
    );
    assert_eq!(set, 1);
    let out = vm.new_wstring(Vec::new());
    let (_, got) = call(&mut vm, id("gBaseProperty::CallEFun_InqStringW"), &[out], p);
    assert_eq!(got, 1);
    assert_eq!(vm.wstring(out), Some(&[97u16, 98, 99][..]));
}

#[test]
fn a_bad_this_crashes_the_efun() {
    let mut vm = vm();
    let (status, _) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqBool"),
        &[0],
        u32::MAX,
    );
    assert_ne!(status, InterpStatus::Success);
}

#[test]
fn string_info_efuns_copy_the_string_info_both_ways() {
    let mut vm = vm();
    let p = property_ref(
        &mut vm,
        property(Some(BasePropertyValue::StringInfo(
            ac2_db::StringInfoValue::default(),
        ))),
    );
    // Set from a script StringInfo with a table, a string id and an int variable.
    let src = vm.refs.create_native(
        ntype::STRING_INFO,
        NativeObject::StringInfo(StringInfo {
            string_id: 0x51,
            table_id: 0x4000_0026,
            literal: None,
            variables: vec![StringVar {
                id: 1,
                kind: 2,
                value: StringVarValue::Int(-7),
            }],
        }),
    );
    let (_, set) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetStringInfo"),
        &[src],
        p,
    );
    assert_eq!(set, 1);
    // Inquire into an empty StringInfo native: the same string, table and variable come back.
    let out = vm.refs.create_native(
        ntype::STRING_INFO,
        NativeObject::StringInfo(StringInfo::default()),
    );
    let (_, got) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqStringInfo"),
        &[out],
        p,
    );
    assert_eq!(got, 1);
    let Some(NativeObject::StringInfo(info)) = vm.native(out) else {
        panic!("StringInfo native expected");
    };
    assert_eq!((info.string_id, info.table_id), (0x51, 0x4000_0026));
    assert_eq!(info.variables.len(), 1);
    assert_eq!(info.variables[0].value, StringVarValue::Int(-7));
}

#[test]
fn string_info_efuns_answer_zero_for_other_types() {
    let mut vm = vm();
    let p = property_ref(&mut vm, property(Some(BasePropertyValue::Bool(true))));
    let out = vm.refs.create_native(
        ntype::STRING_INFO,
        NativeObject::StringInfo(StringInfo {
            string_id: 3,
            ..StringInfo::default()
        }),
    );
    let (_, got) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqStringInfo"),
        &[out],
        p,
    );
    assert_eq!(got, 0);
    let (_, set) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetStringInfo"),
        &[out],
        p,
    );
    assert_eq!(set, 0);
    let Some(NativeObject::StringInfo(info)) = vm.native(out) else {
        panic!("StringInfo native expected");
    };
    assert_eq!(info.string_id, 3);
}
