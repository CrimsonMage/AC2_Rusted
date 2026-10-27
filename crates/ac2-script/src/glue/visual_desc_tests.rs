//! The `gVisualDesc` and `gAppliedAppearanceKey` efuns over the VM (`glue/visual_desc.rs`): each
//! one reads its arguments, edits the description or the applied key, and writes its answer where
//! the efun frame says (`VisualDesc` efun glue, `CallEFun_*`).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_weenie::{Appearance, PartGroupAppearance, GLOBAL_PART_GROUP};

use crate::glue::efun_table;
use crate::host::ScriptHost;
use crate::native::{ntype, AppliedAppearanceKey, NativeObject, RefKey, ScriptHash};
use crate::vm::{tag, Context, InterpStatus, Program, ScriptData, Vm};

const PARENT: u32 = 0x1F00_0023;
const ROOT: u32 = 0x8100_0001;
const TABLE: u32 = 0x2000_0010;
const DB_TYPE_VISUAL: u32 = 0x4000_0021;

/// The DAT records the tests hold: visual descriptions by DID.
struct Records(HashMap<u32, Arc<Appearance>>);

impl ScriptData for Records {
    fn wstate(&self, _did: u32) -> Option<crate::vm::StateRef> {
        None
    }

    fn db_type(&self, did: u32) -> Option<u32> {
        self.0.contains_key(&did).then_some(DB_TYPE_VISUAL)
    }

    fn visual_desc(&self, did: u32) -> Option<Arc<Appearance>> {
        self.0.get(&did).cloned()
    }
}

fn efun(name: &str) -> u32 {
    efun_table()
        .find(|(_, n)| *n == name)
        .map(|(id, _)| id)
        .unwrap_or_else(|| panic!("no efun {name}"))
}

fn vm_with(records: HashMap<u32, Arc<Appearance>>) -> ScriptHost {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    ScriptHost::new(Arc::new(program), Box::new(Records(records)))
}

/// Runs `name` with `args` on `this` and returns its frame slot `ret` (word 2 is the first
/// argument, as the glue reads them).
fn call(vm: &mut Vm, name: &str, args: &[u32], this: u32, ret: usize) -> u32 {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    ctx.push(0, tag::INT).unwrap();
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let fp = ctx.sp;
    assert_eq!(
        vm.run_efun(efun(name), &mut ctx),
        InterpStatus::Success,
        "{name}"
    );
    ctx.data[fp - ret]
}

/// The address of stack slot `slot` as a script passes an out parameter (`SADDR`).
fn saddr(vm: &mut Vm, slot: usize) -> u32 {
    let blob =
        (slot as u32) << crate::vm::refs::flags::OFFSET_SHIFT | crate::vm::refs::flags::KIND_VALUE;
    vm.refs.add(blob, crate::vm::refs::Target::Ref(u32::MAX))
}

/// Like [`call`], but the script's local at stack slot 0 holds `local` and an out argument
/// passed as `saddr(0)` names it.
fn call_with_local(
    vm: &mut Vm,
    name: &str,
    local: u32,
    args: &[u32],
    this: u32,
    ret: usize,
) -> u32 {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    ctx.push(local, tag::REF).unwrap();
    ctx.push(0, tag::INT).unwrap();
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let fp = ctx.sp;
    assert_eq!(
        vm.run_efun(efun(name), &mut ctx),
        InterpStatus::Success,
        "{name}"
    );
    ctx.data[fp - ret]
}

fn visual(vm: &mut Vm, a: Appearance) -> u32 {
    vm.refs
        .create_native(ntype::VISUAL_DESC, NativeObject::VisualDesc(Box::new(a)))
}

fn applied(vm: &mut Vm, key: u32, scalar: f32) -> u32 {
    vm.refs.create_native(
        ntype::APPLIED_APPEARANCE_KEY,
        NativeObject::AppliedKey(AppliedAppearanceKey { key, scalar }),
    )
}

fn appearance(vm: &Vm, r: u32) -> Appearance {
    match vm.native(r) {
        Some(NativeObject::VisualDesc(a)) => (**a).clone(),
        other => panic!("{other:?}"),
    }
}

fn parent_records() -> HashMap<u32, Arc<Appearance>> {
    let mut parent = Appearance {
        parent: 0,
        ..Appearance::default()
    };
    parent.part_groups = vec![PartGroupAppearance {
        key: ROOT,
        appearance: vec![(TABLE, vec![(1, 0.5)])],
        ..PartGroupAppearance::default()
    }];
    parent.root = Some(ROOT);
    HashMap::from([(PARENT, Arc::new(parent))])
}

#[test]
fn clear_empty_and_the_scale_efuns() {
    let mut host = vm_with(HashMap::new());
    let vm = host.vm_mut();
    let d = visual(vm, Appearance::default());
    assert_eq!(call(vm, "gVisualDesc::CallEFun_Empty", &[], d, 2), 1);
    let v = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([1.5, 2.0, 0.25]));
    call(vm, "gVisualDesc::CallEFun_SetScale", &[v], d, 2);
    assert_eq!(appearance(vm, d).scale, [1.5, 2.0, 0.25]);
    // `GetScale` answers a new vector of the scale (`CopyNativeObject(0x70)`).
    let got = call(vm, "gVisualDesc::CallEFun_GetScale", &[], d, 2);
    assert_eq!(
        vm.native(got),
        Some(&NativeObject::Vector([1.5, 2.0, 0.25]))
    );
    // Empty is false once a scale or a parent is set, and Clear returns the unit description.
    // `Empty` looks at the parent, the root and the global group, not the scale; a parent makes
    // it false.
    assert_eq!(call(vm, "gVisualDesc::CallEFun_Empty", &[], d, 2), 1);
    let parented = visual(
        vm,
        Appearance {
            parent: PARENT,
            ..Appearance::default()
        },
    );
    assert_eq!(call(vm, "gVisualDesc::CallEFun_Empty", &[], parented, 2), 0);
    call(vm, "gVisualDesc::CallEFun_Clear", &[], d, 2);
    assert_eq!(appearance(vm, d), Appearance::default());
}

#[test]
fn add_appearance_key_on_the_global_group_and_a_parents_group() {
    let mut host = vm_with(parent_records());
    let vm = host.vm_mut();
    let d = visual(vm, Appearance::default());
    let k = applied(vm, 7, 0.5);
    // The global group takes any table with no parent needed.
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_AddAppearanceKey",
            &[GLOBAL_PART_GROUP, TABLE, k],
            d,
            5
        ),
        1
    );
    assert_eq!(
        appearance(vm, d).global.appearance,
        vec![(TABLE, vec![(7, 0.5)])]
    );
    // A group the description does not hold, with no parent: nothing (retail's `MakeLocal`).
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_AddAppearanceKey",
            &[ROOT, TABLE, k],
            d,
            5
        ),
        0
    );
    // With a parent, the parent's group is copied and the key joins it.
    let p = visual(
        vm,
        Appearance {
            parent: PARENT,
            ..Appearance::default()
        },
    );
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_AddAppearanceKey",
            &[ROOT, TABLE, k],
            p,
            5
        ),
        1
    );
    assert_eq!(
        appearance(vm, p).group(ROOT).unwrap().appearance,
        vec![(TABLE, vec![(7, 0.5)])]
    );
}

#[test]
fn remove_appearance_key_overrides_a_parents_key_with_minus_one() {
    let mut host = vm_with(parent_records());
    let vm = host.vm_mut();
    let p = visual(
        vm,
        Appearance {
            parent: PARENT,
            ..Appearance::default()
        },
    );
    let k = applied(vm, 1, 0.0);
    // The parent applies key 1 of the table, this description does not: an override at -1.
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_RemoveAppearanceKey",
            &[ROOT, TABLE, k],
            p,
            5
        ),
        1
    );
    assert_eq!(appearance(vm, p).applied_scalar(ROOT, TABLE, 1), Some(-1.0));
    // A description without a parent holds nothing to remove.
    let bare = visual(vm, Appearance::default());
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_RemoveAppearanceKey",
            &[ROOT, TABLE, k],
            bare,
            5
        ),
        0
    );
}

#[test]
fn apply_additive_replace_and_subtractive_take_the_incoming_description() {
    let mut host = vm_with(HashMap::new());
    let vm = host.vm_mut();
    let incoming = Appearance {
        scale: [2.0, 1.0, 1.0],
        global: PartGroupAppearance {
            key: GLOBAL_PART_GROUP,
            appearance: vec![(TABLE, vec![(3, 1.0)])],
            ..PartGroupAppearance::default()
        },
        ..Appearance::default()
    };
    let inc = visual(vm, incoming.clone());
    let d = visual(vm, Appearance::default());
    assert_eq!(
        call(vm, "gVisualDesc::CallEFun_ApplyAdditiveVDesc", &[inc], d, 3),
        1
    );
    assert_eq!(appearance(vm, d).scale[0], 2.0);
    assert_eq!(
        appearance(vm, d).global.appearance,
        vec![(TABLE, vec![(3, 1.0)])]
    );
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_ApplySubtractiveVDesc",
            &[inc],
            d,
            3
        ),
        1
    );
    assert!(
        appearance(vm, d).global.appearance.is_empty(),
        "the incoming key left"
    );
    // Replace: the same parent (none) merges the incoming's global keys; key 3 left with the
    // subtractive merge above.
    let other = visual(
        vm,
        Appearance {
            global: PartGroupAppearance {
                key: GLOBAL_PART_GROUP,
                appearance: vec![(TABLE, vec![(4, 1.0)])],
                ..PartGroupAppearance::default()
            },
            ..Appearance::default()
        },
    );
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_ReplaceVisualDesc",
            &[other],
            d,
            3
        ),
        1
    );
    assert_eq!(
        appearance(vm, d).global.appearance,
        vec![(TABLE, vec![(4, 1.0)])]
    );
}

#[test]
fn the_dat_visual_loads_into_the_description_and_a_non_visual_fails() {
    let mut host = vm_with(parent_records());
    let vm = host.vm_mut();
    let out = visual(
        vm,
        Appearance {
            modes: 9,
            ..Appearance::default()
        },
    );
    let did_ok = call(
        vm,
        "gVisualDesc::CallEFun_GetVisualDescFromDB",
        &[PARENT, out],
        out,
        4,
    );
    assert_eq!(did_ok, 1);
    let loaded = appearance(vm, out);
    assert_eq!(loaded.modes, 0, "cleared first");
    assert_eq!(
        loaded.group(ROOT).unwrap().appearance,
        vec![(TABLE, vec![(1, 0.5)])]
    );
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_GetVisualDescFromDB",
            &[0x1F00_0099, out],
            out,
            4
        ),
        0
    );
}

#[test]
fn the_app_info_hash_round_trips_through_a_script_hash() {
    let mut host = vm_with(parent_records());
    let vm = host.vm_mut();
    let source = visual(
        vm,
        Appearance {
            parent: PARENT,
            ..Appearance::default()
        },
    );
    let hash = vm
        .refs
        .create_native(ntype::AA_HASH, NativeObject::Hash(ScriptHash::default()));
    // An empty group answers 0 and leaves the hash empty.
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_InqAppInfoHashForPartGroup",
            &[ROOT, hash],
            source,
            4
        ),
        0
    );
    let mut inq = Appearance {
        parent: PARENT,
        ..Appearance::default()
    };
    inq.part_groups = vec![PartGroupAppearance {
        key: ROOT,
        appearance: vec![(TABLE, vec![(5, 0.25)])],
        ..PartGroupAppearance::default()
    }];
    let with_group = visual(vm, inq);
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_InqAppInfoHashForPartGroup",
            &[ROOT, hash],
            with_group,
            4
        ),
        1
    );
    // The hash now holds the table: its key is the table DID, its value an AppearanceInfo.
    let NativeObject::Hash(h) = vm.native(hash).unwrap().clone() else {
        panic!()
    };
    assert!(h.contains(&RefKey::Value(u64::from(TABLE)), &crate::native::PlainKeys));
    // Adding the hash to another description joins its tables (through the parent's group).
    let target = visual(
        vm,
        Appearance {
            parent: PARENT,
            ..Appearance::default()
        },
    );
    assert_eq!(
        call(
            vm,
            "gVisualDesc::CallEFun_AddAppInfoHashToPartGroup",
            &[ROOT, hash],
            target,
            4
        ),
        1
    );
    assert_eq!(
        appearance(vm, target).applied_scalar(ROOT, TABLE, 5),
        Some(0.25)
    );
}

#[test]
fn the_total_icon_is_the_parents_layers_with_ours_on_top() {
    let mut host = vm_with(parent_records());
    let vm = host.vm_mut();
    let icon = vm
        .refs
        .create_native(ntype::ICON_DESC, NativeObject::IconDesc(Vec::new()));
    let d = visual(
        vm,
        Appearance {
            parent: PARENT,
            icon: vec![(2, 22, 2)],
            ..Appearance::default()
        },
    );
    // The parent has no icon layers here: ours alone.
    assert_eq!(
        call(vm, "gVisualDesc::CallEFun_GetTotalIconDesc", &[icon], d, 3),
        1
    );
    assert_eq!(
        vm.native(icon),
        Some(&NativeObject::IconDesc(vec![(2, 22, 2)]))
    );
}

/// The out arguments are the script's locals (`SADDR`): each efun reads the native the local holds
/// and writes into that object (`GetReferencePtr` in `gVisualDesc::CallEFun_*`).
#[test]
fn out_arguments_passed_as_local_addresses_write_through() {
    let mut host = vm_with(parent_records());
    let vm = host.vm_mut();
    // GetVisualDescFromDB: the local holds a description that the record overwrites.
    let out = visual(
        vm,
        Appearance {
            modes: 9,
            ..Appearance::default()
        },
    );
    let addr = saddr(vm, 0);
    assert_eq!(
        call_with_local(
            vm,
            "gVisualDesc::CallEFun_GetVisualDescFromDB",
            out,
            &[PARENT, addr],
            out,
            4
        ),
        1
    );
    assert_eq!(
        appearance(vm, out).modes,
        0,
        "the record is loaded into the local's object"
    );
    // InqAppInfoHashForPartGroup: the local holds the hash the group's tables go into.
    let source = visual(
        vm,
        Appearance {
            parent: PARENT,
            part_groups: vec![PartGroupAppearance {
                key: ROOT,
                appearance: vec![(TABLE, vec![(5, 0.25)])],
                ..PartGroupAppearance::default()
            }],
            ..Appearance::default()
        },
    );
    let hash = vm
        .refs
        .create_native(ntype::AA_HASH, NativeObject::Hash(ScriptHash::default()));
    let addr = saddr(vm, 0);
    assert_eq!(
        call_with_local(
            vm,
            "gVisualDesc::CallEFun_InqAppInfoHashForPartGroup",
            hash,
            &[ROOT, addr],
            source,
            4
        ),
        1
    );
    let NativeObject::Hash(h) = vm.native(hash).unwrap().clone() else {
        panic!()
    };
    assert!(h.contains(&RefKey::Value(u64::from(TABLE)), &crate::native::PlainKeys));
    // GetTotalIconDesc (already through SADDR): the local holds the icon.
    let icon = vm
        .refs
        .create_native(ntype::ICON_DESC, NativeObject::IconDesc(Vec::new()));
    let d = visual(
        vm,
        Appearance {
            parent: PARENT,
            icon: vec![(2, 22, 2)],
            ..Appearance::default()
        },
    );
    let addr = saddr(vm, 0);
    assert_eq!(
        call_with_local(
            vm,
            "gVisualDesc::CallEFun_GetTotalIconDesc",
            icon,
            &[addr],
            d,
            3
        ),
        1
    );
    assert_eq!(
        vm.native(icon),
        Some(&NativeObject::IconDesc(vec![(2, 22, 2)]))
    );
}

#[test]
fn the_applied_key_fields_read_and_write() {
    let mut host = vm_with(HashMap::new());
    let vm = host.vm_mut();
    let k = applied(vm, 1, 0.0);
    call(
        vm,
        "gAppliedAppearanceKey::CallEFun_SetAppKey",
        &[0x42],
        k,
        2,
    );
    call(
        vm,
        "gAppliedAppearanceKey::CallEFun_SetMod",
        &[(-1.0f32).to_bits()],
        k,
        2,
    );
    assert_eq!(
        call(vm, "gAppliedAppearanceKey::CallEFun_GetAppKey", &[], k, 2),
        0x42
    );
    assert_eq!(
        call(vm, "gAppliedAppearanceKey::CallEFun_GetMod", &[], k, 2),
        (-1.0f32).to_bits()
    );
}
