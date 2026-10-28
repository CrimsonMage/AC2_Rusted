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

/// The DAT-backed VM (the `MasterProperty` names and the type enum read from the local data),
/// or `None` without `AC2_DAT_DIR`.
fn dat_vm() -> Option<Vm> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let mut efuns = EFuns::new();
    register_pure(&mut efuns);
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    Some(Vm::new(
        Arc::new(program),
        Box::new(crate::data::DatScriptData::new(dataset, store)),
        efuns,
    ))
}

/// The wide string a `wpstring` native holds.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn property_name_efuns_name_known_properties_from_the_dat() {
    let Some(mut vm) = dat_vm() else {
        return;
    };
    // `LightType` (0x40000002) and `LightDiffuse` (0x40000008) are in the local MasterProperty.
    let p = property_ref(&mut vm, BaseProperty::new(0x4000_0002));
    let (_, name) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetPropertyName"),
        &[],
        p,
    );
    assert_eq!(name, 0x4000_0002);
    let out = vm.new_wstring(Vec::new());
    let (_, found) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqPropertyNameW"),
        &[out],
        p,
    );
    assert_eq!(found, 1);
    assert_eq!(vm.wstring(out), Some(&wide("LightType")[..]));

    // SetPropertyNameW names a property by its text; its desc gives the type and group.
    let text = vm.new_wstring(wide("LightDiffuse"));
    let (_, set) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetPropertyNameW"),
        &[text],
        p,
    );
    assert_eq!(set, 1);
    let (_, name) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetPropertyName"),
        &[],
        p,
    );
    assert_eq!(name, 0x4000_0008);
    let desc = ac2_db::ac2_property_desc(0x4000_0008).expect("LightDiffuse has a desc");
    let (_, ty) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetPropertyType"),
        &[],
        p,
    );
    assert_eq!(ty, desc.value_type);
    let (_, group) = call(&mut vm, id("gBaseProperty::CallEFun_GetGroupName"), &[], p);
    assert_eq!(group, desc.group);

    // The `0x40000013` type enum maps to the mapper DID 0x23000022, which the Legions DVD does not
    // hold: retail names no type there, so the efun answers 0 with an empty string.
    let type_out = vm.new_wstring(vec![b'x' as u16]);
    let (_, named) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqPropertyTypeW"),
        &[type_out],
        p,
    );
    assert_eq!(named, 0);
    assert_eq!(vm.wstring(type_out), Some(&[][..]));

    // A name no table holds changes nothing and answers 0.
    let unknown = vm.new_wstring(wide("NoSuchPropertyName"));
    let (_, miss) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetPropertyNameW"),
        &[unknown],
        p,
    );
    assert_eq!(miss, 0);
    let (_, name) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetPropertyName"),
        &[],
        p,
    );
    assert_eq!(name, 0x4000_0008);
}

#[test]
fn property_getters_read_the_name_type_and_group_words() {
    let mut vm = vm();
    let mut p = BaseProperty::new(0x1234_5678);
    p.value_type = 0x4000_0003;
    p.group = 0x4000_0002;
    let r = property_ref(&mut vm, p);
    let (_, name) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetPropertyName"),
        &[],
        r,
    );
    assert_eq!(name, 0x1234_5678);
    let (_, ty) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetPropertyType"),
        &[],
        r,
    );
    assert_eq!(ty, 0x4000_0003);
    let (_, group) = call(&mut vm, id("gBaseProperty::CallEFun_GetGroupName"), &[], r);
    assert_eq!(group, 0x4000_0002);
}

#[test]
fn a_property_type_without_its_enum_names_nothing() {
    // No DAT: the `0x40000013` enum is not there, so the type name is empty and the flag 0.
    let mut vm = vm();
    let mut p = BaseProperty::new(1);
    p.value_type = 0x4000_0005;
    let r = property_ref(&mut vm, p);
    let out = vm.new_wstring(vec![b'x' as u16]);
    let (_, found) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqPropertyTypeW"),
        &[out],
        r,
    );
    assert_eq!(found, 0);
    assert_eq!(vm.wstring(out), Some(&[][..]));
}

/// A script data source whose only table is the property type enum (`0x40000013`) mapper, with
/// the names of two types.
struct TypeEnum;

impl crate::vm::ScriptData for TypeEnum {
    fn wstate(&self, _did: u32) -> Option<crate::vm::StateRef> {
        None
    }

    fn enum_did(&self, value: u32, group: u32) -> Option<u32> {
        (value == 0x4000_0013 && group == crate::data::EMAPPER_GROUP).then_some(0x2300_0022)
    }

    fn db_type(&self, did: u32) -> Option<u32> {
        (did == 0x2300_0022).then_some(crate::data::DB_TYPE_ENUM_MAPPER)
    }

    fn enum_mapper(
        &self,
        did: u32,
    ) -> Option<Box<dyn std::ops::Deref<Target = crate::data::EnumMapper>>> {
        (did == 0x2300_0022).then(|| {
            let mapper = crate::data::EnumMapper {
                did,
                base: 0,
                names: vec![
                    (0x4000_0003, b"Float".to_vec()),
                    (0x4000_0005, b"Color".to_vec()),
                ],
            };
            Box::new(Box::new(mapper)) as Box<dyn std::ops::Deref<Target = crate::data::EnumMapper>>
        })
    }
}

#[test]
fn property_type_names_come_from_the_type_enum() {
    let mut efuns = EFuns::new();
    register_pure(&mut efuns);
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut vm = Vm::new(Arc::new(program), Box::new(TypeEnum), efuns);
    let mut p = BaseProperty::new(1);
    p.value_type = 0x4000_0005;
    let r = property_ref(&mut vm, p);
    let out = vm.new_wstring(Vec::new());
    let (_, found) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqPropertyTypeW"),
        &[out],
        r,
    );
    assert_eq!(found, 1);
    assert_eq!(vm.wstring(out), Some(&wide("Color")[..]));
    // A type the enum does not name answers 0 and an empty string.
    let mut other = BaseProperty::new(1);
    other.value_type = 0x4000_0009;
    let r = property_ref(&mut vm, other);
    let out = vm.new_wstring(vec![b'x' as u16]);
    let (_, missing) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqPropertyTypeW"),
        &[out],
        r,
    );
    assert_eq!(missing, 0);
    assert_eq!(vm.wstring(out), Some(&[][..]));
}

#[test]
fn collide_property_get_property_copies_the_base_property() {
    let mut vm = vm();
    let mut property = BaseProperty::new(0x4000_0002);
    property.value_type = 0x4000_0002;
    property.group = 0x4000_0003;
    property.value = Some(BasePropertyValue::Integer(-5));
    let collide = vm.refs.create_native(
        ntype::COLLIDE_PROPERTY,
        NativeObject::CollideProperty(crate::native::CollideProperty {
            id: 1,
            collide_type: 0,
            property,
        }),
    );
    let (status, copy) = call(
        &mut vm,
        id("gCollideProperty::CallEFun_GetProperty"),
        &[],
        collide,
    );
    assert_eq!(status, InterpStatus::Success);
    let out = out_slot(&mut vm);
    let (_, got) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_InqInteger"),
        &[out],
        copy,
    );
    assert_eq!(got, 1);
    assert_eq!(slot_word(&vm, out) as i32, -5);
    let (_, group) = call(
        &mut vm,
        id("gBaseProperty::CallEFun_GetGroupName"),
        &[],
        copy,
    );
    assert_eq!(group, 0x4000_0003);
    // The copy is its own property: setting it leaves the report's property alone.
    call(
        &mut vm,
        id("gBaseProperty::CallEFun_SetInteger"),
        &[7],
        copy,
    );
    match vm.native(collide) {
        Some(NativeObject::CollideProperty(c)) => {
            assert_eq!(c.property.value, Some(BasePropertyValue::Integer(-5)));
        }
        other => panic!("CollideProperty expected, got {other:?}"),
    }
    let (status, _) = call(
        &mut vm,
        id("gCollideProperty::CallEFun_GetProperty"),
        &[],
        u32::MAX,
    );
    assert_ne!(status, InterpStatus::Success);
}
