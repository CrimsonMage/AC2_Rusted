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
fn tokenize_hashes_the_wide_string_below_its_argument() {
    let mut vm = vm();
    // `Tokenize` of ASCII names gives the layouts' token values.
    let name = vm.new_wstring(w("reference"));
    let (status, rets) = call(&mut vm, 0x006D_0001, 1, &[name], u32::MAX);
    assert_eq!(status, InterpStatus::Success);
    assert_eq!(rets[0], (0xBCE0_5594, INT_RESULT));
    let empty = vm.new_wstring(Vec::new());
    let (_, rets) = call(&mut vm, 0x006D_0001, 1, &[empty], u32::MAX);
    assert_eq!(rets[0].0, 0);
    // Only a wide character's low seven bits survive `(ushort)c << 25`.
    assert_eq!(wpstring::tokenize_wide(&[0x0141]), 0x0141 << 25);
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

#[test]
fn list_glue_follows_the_node_list_and_smart_array_rules() {
    use crate::native::ScriptList;
    use crate::vm::refs::{flags, ReferenceTable, Target};
    let mut vm = vm();
    let empty = |vm: &mut Vm, t: u32| {
        vm.refs
            .create_native(t, NativeObject::List(ScriptList::default()))
    };
    let list = empty(&mut vm, crate::native::ntype::A_LIST);
    for v in [5, 7, 5, 9] {
        call(&mut vm, 0x000A_0006, 1, &[v], list); // push_back
    }
    let (_, r) = call(&mut vm, 0x000A_000E, 1, &[7], list); // push_unique_back
    assert_eq!(r[0].0, 0);
    call(&mut vm, 0x000A_0024, 1, &[5], list); // remove: the last 5
    let items = |vm: &Vm, r: u32| match vm.native(r) {
        Some(NativeObject::List(l)) => l.items.clone(),
        _ => panic!(),
    };
    assert_eq!(items(&vm, list), vec![5, 7, 9]);
    let (_, r) = call(&mut vm, 0x000A_0029, 1, &[9], list); // get_position
    assert_eq!(r[0].0, 2);
    // pop_front into a heap slot through an address reference.
    let holder = vm.refs.add(
        flags::KIND_HEAP | flags::FILLED,
        Target::Heap(Box::new(ReferenceTable::new_instance(0, 1))),
    );
    let out = vm.refs.add(flags::KIND_VALUE, Target::Ref(holder));
    let (_, r) = call(&mut vm, 0x000A_0014, 1, &[out], list);
    assert_eq!(r[0].0, 1);
    assert_eq!(vm.refs.heap(holder).unwrap().slots[0], 5);
    assert_eq!(vm.refs.heap(holder).unwrap().tags[0], tag::BOOL);
    // An array erase moves the last element into the hole.
    let array = empty(&mut vm, crate::native::ntype::A_ARRAY);
    for v in [1, 2, 3, 4] {
        call(&mut vm, 0x0005_0002, 1, &[v], array); // push
    }
    call(&mut vm, 0x0005_0008, 1, &[0], array); // erase(0)
    assert_eq!(items(&vm, array), vec![4, 2, 3]);
    // Iterate with get_begin / data / next / notdone.
    let (_, r) = call(&mut vm, 0x000A_002D, 1, &[], list);
    let iter = r[0].0;
    let mut seen = Vec::new();
    loop {
        let (_, n) = call(&mut vm, 0x000B_0007, 1, &[], iter); // notdone
        if n[0].0 == 0 {
            break;
        }
        let (_, d) = call(&mut vm, 0x000B_0003, 1, &[], iter); // data
        seen.push(d[0].0);
        call(&mut vm, 0x000B_0001, 0, &[], iter); // next
    }
    assert_eq!(seen, vec![7, 9]);
}

#[test]
fn hash_glue_keeps_retail_bucket_order_and_hashes_native_keys_by_value() {
    use crate::native::{ntype, ScriptHash};
    use crate::vm::refs::{flags, ReferenceTable, Target};
    let mut vm = vm();
    let new = |vm: &mut Vm, t: u32| {
        vm.refs
            .create_native(t, NativeObject::Hash(ScriptHash::default()))
    };
    let aa = new(&mut vm, ntype::AA_HASH);
    for (k, v) in [(1, 10), (48, 480), (2, 20)] {
        let (_, r) = call(&mut vm, 0x0001_0001, 1, &[k, v], aa); // add
        assert_eq!(r[0], (1, tag::BOOL));
    }
    let (_, r) = call(&mut vm, 0x0001_0001, 1, &[48, 0], aa);
    assert_eq!(r[0].0, 0);
    let (_, r) = call(&mut vm, 0x0001_0008, 1, &[], aa); // num_buckets
    assert_eq!(r[0].0, 47);
    // find(48, out) into a heap slot.
    let holder = vm.refs.add(
        flags::KIND_HEAP | flags::FILLED,
        Target::Heap(Box::new(ReferenceTable::new_instance(0, 1))),
    );
    let out = vm.refs.add(flags::KIND_VALUE, Target::Ref(holder));
    let (_, r) = call(&mut vm, 0x0001_0004, 1, &[48, out], aa);
    assert_eq!(r[0].0, 1);
    assert_eq!(vm.refs.heap(holder).unwrap().slots[0], 480);
    // Iteration: bucket 1 holds 48 then 1 (head insertion), then bucket 2.
    let (_, r) = call(&mut vm, 0x0001_000A, 1, &[], aa); // get_begin
    let iter = r[0].0;
    let mut keys = Vec::new();
    while call(&mut vm, 0x0002_0005, 1, &[], iter).1[0].0 != 0 {
        keys.push(call(&mut vm, 0x0002_0002, 1, &[], iter).1[0].0); // key
        call(&mut vm, 0x0002_0001, 0, &[], iter); // next
    }
    assert_eq!(keys, vec![48, 1, 2]);
    assert_eq!(
        call(&mut vm, 0x0002_0001, 0, &[], iter).0,
        InterpStatus::Crash
    );
    let (_, r) = call(&mut vm, 0x0001_0011, 1, &[12], aa); // resize(12) -> 23
    assert_eq!(r[0].0, 1);
    assert_eq!(call(&mut vm, 0x0001_0008, 1, &[], aa).1[0].0, 23);
    // A 64-bit key takes two slots, high word first.
    let la = new(&mut vm, ntype::LA_HASH);
    call(&mut vm, 0x0032_0001, 1, &[7, 5, 99], la); // add(0x7_00000005, 99)
    let (_, r) = call(&mut vm, 0x0032_0006, 1, &[7, 5], la); // contains
    assert_eq!(r[0].0, 1);
    let (_, r) = call(&mut vm, 0x0032_000A, 1, &[], la);
    let (_, k) = call(&mut vm, 0x0033_0002, 2, &[], r[0].0); // key: low deeper
    assert_eq!((k[0], k[1].0), ((5, 0x84), 7));
    // N keys: two equal wpstrings are the same key.
    let na = new(&mut vm, ntype::NA_HASH);
    let a = vm.new_wstring(w("sword"));
    let b = vm.new_wstring(w("sword"));
    assert_eq!(call(&mut vm, 0x004B_0001, 1, &[a, 1], na).1[0].0, 1);
    assert_eq!(call(&mut vm, 0x004B_0001, 1, &[b, 2], na).1[0].0, 0);
    assert_eq!(call(&mut vm, 0x004B_0012, 1, &[b], na).1[0].0, 1); // contains
}

#[test]
fn random_selection_tables_choose_without_replacement_and_start_over() {
    use crate::native::{ntype, RandomSelectionTable};
    use crate::vm::refs::{flags, ReferenceTable, Target};
    let mut vm = vm();
    let t = vm.refs.create_native(
        ntype::RANDOM_SELECTION_TABLE_INT,
        NativeObject::RandomSelectionTable(RandomSelectionTable::default()),
    );
    for (object, weight) in [(10, 1), (20, 5), (30, 2), (40, 0)] {
        call(&mut vm, 0x0060_0001, 0, &[object, weight], t); // AddEntry (weight 0 ignored)
    }
    let table = |vm: &Vm| match vm.native(t) {
        Some(NativeObject::RandomSelectionTable(t)) => t.clone(),
        _ => panic!(),
    };
    assert_eq!(table(&vm).unused, vec![(10, 1, 8), (20, 5, 5), (30, 2, 2)]);
    let holder = vm.refs.add(
        flags::KIND_HEAP | flags::FILLED,
        Target::Heap(Box::new(ReferenceTable::new_instance(0, 1))),
    );
    let out = vm.refs.add(flags::KIND_VALUE, Target::Ref(holder));
    let mut chosen = Vec::new();
    while call(&mut vm, 0x0060_0005, 1, &[], t).1[0].0 == 0 {
        let (_, r) = call(&mut vm, 0x0060_0007, 1, &[out], t); // Choose
        assert_eq!(r[0].0, 1);
        chosen.push(vm.refs.heap(holder).unwrap().slots[0]);
        let unused = table(&vm).unused;
        if let Some(root) = unused.first() {
            assert_eq!(root.2, unused.iter().map(|e| e.1).sum::<u32>());
        }
    }
    chosen.sort_unstable();
    assert_eq!(chosen, vec![10, 20, 30]);
    assert_eq!(call(&mut vm, 0x0060_0007, 1, &[out], t).1[0].0, 0);
    call(&mut vm, 0x0060_0006, 0, &[], t); // StartOver
    let unused = table(&vm).unused;
    assert_eq!(unused.len(), 3);
    assert_eq!(unused[0].2, 8);
    let (_, r) = call(&mut vm, 0x005F_0001, 1, &[], t); // RollPercentile
    assert!((1..=100).contains(&r[0].0));
}

#[test]
fn math_vector_and_missile_glue_use_retail_slots() {
    use crate::native::{ntype, MissileParameters};
    let mut vm = vm();
    let d = |x: f64| [(x.to_bits() >> 32) as u32, x.to_bits() as u32];
    // __max(2.5, -1.0): doubles high word first, the result low word tagged 0x84.
    let args = [d(2.5), d(-1.0)].concat();
    let (_, r) = call(&mut vm, 0x0044_0013, 2, &args, u32::MAX);
    assert_eq!(r[0], (2.5f64.to_bits() as u32, 0x84));
    assert_eq!(r[1].0, (2.5f64.to_bits() >> 32) as u32);
    let (_, r) = call(&mut vm, 0x0044_0015, 1, &[3, (-7i32) as u32], u32::MAX); // intMax
    assert_eq!(r[0].0, 3);
    let (_, r) = call(
        &mut vm,
        0x0044_001E,
        2,
        &[u32::MAX, (-5i32) as u32],
        u32::MAX,
    ); // fake_abs64
    assert_eq!((r[0].0, r[1].0), (5, 0));
    // Vectors: Set, Cross, Dot, GetStringW.
    let a = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([0.0; 3]));
    let b = vm
        .refs
        .create_native(ntype::VECTOR, NativeObject::Vector([0.0, 1.0, 0.0]));
    let f = |x: f32| x.to_bits();
    call(&mut vm, 0x0070_0006, 0, &[f(1.0), f(0.0), f(0.0)], a); // Set
    let (_, r) = call(&mut vm, 0x0070_0001, 1, &[b], a); // Cross
    assert!(matches!(vm.native(r[0].0), Some(NativeObject::Vector(v)) if *v == [0.0, 0.0, 1.0]));
    let (_, r) = call(&mut vm, 0x0070_0002, 1, &[b], a); // Dot
    assert_eq!(r[0], (0f32.to_bits(), tag::BOOL));
    use crate::vm::refs::{flags, ReferenceTable, Target};
    let s = vm.new_wstring(Vec::new());
    let mut holder = ReferenceTable::new_instance(0, 1);
    holder.slots[0] = s;
    holder.tags[0] = tag::REF;
    let holder = vm.refs.add(
        flags::KIND_HEAP | flags::FILLED,
        Target::Heap(Box::new(holder)),
    );
    let out = vm.refs.add(flags::KIND_VALUE, Target::Ref(holder));
    call(&mut vm, 0x0070_000F, 0, &[out], a); // GetStringW(out string)
    assert_eq!(vm.wstring(s), Some(w("(1.000, 0.000, 0.000)").as_slice()));
    // A missile setter writes its fields and its group's bit.
    let m = vm.refs.create_native(
        ntype::MISSILE_PARAMETERS,
        NativeObject::MissileParameters(MissileParameters::default()),
    );
    call(&mut vm, 0x0046_0001, 0, &[f(20.0), f(2.0)], m); // SetSpeedLimits
    call(&mut vm, 0x0046_0014, 0, &[1], m); // SetMissileSticky
    let Some(NativeObject::MissileParameters(p)) = vm.native(m) else {
        panic!()
    };
    assert_eq!(p.bitfield, 0x4001);
    assert_eq!(p.field("max_velocity"), Some(f(20.0)));
    assert_eq!(p.field("sticks_upon_collide"), Some(1));
    assert_eq!(p.field("height_above_ground"), Some(f(1.5)));
}

#[test]
fn multi_hash_and_multi_map_glue_walk_in_retail_order() {
    use crate::native::{ScriptMultiHash, ScriptMultiMap};
    let mut vm = vm();
    let mh = vm
        .refs
        .create_native(0x03, NativeObject::MultiHash(ScriptMultiHash::default()));
    for (k, v) in [(5, 1), (198, 2), (5, 3)] {
        call(&mut vm, 0x0003_0001, 0, &[k, v], mh); // add
    }
    assert_eq!(call(&mut vm, 0x0003_0005, 1, &[5], mh).1[0].0, 2); // count
    let iter = call(&mut vm, 0x0003_0009, 1, &[], mh).1[0].0; // get_begin
    let end = call(&mut vm, 0x0003_000A, 1, &[], mh).1[0].0; // get_end
    let mut keys = Vec::new();
    while call(&mut vm, 0x0004_0004, 1, &[end], iter).1[0].0 == 0 {
        keys.push(call(&mut vm, 0x0004_0002, 1, &[], iter).1[0].0); // key
        call(&mut vm, 0x0004_0001, 0, &[], iter); // next
    }
    assert_eq!(keys, vec![198, 5, 5]);
    assert_eq!(call(&mut vm, 0x0003_0002, 1, &[5], mh).1[0].0, 2); // remove
    let mm = vm
        .refs
        .create_native(0x3691, NativeObject::MultiMap(ScriptMultiMap::default()));
    for (k, v) in [(7, 1), (3, 2), (7, 3)] {
        call(&mut vm, 0x3691_0001, 0, &[k, v], mm); // insert
    }
    let it = call(&mut vm, 0x3691_000C, 1, &[3], mm).1[0].0; // upper_bound(3)
    assert_eq!(call(&mut vm, 0x3692_0004, 1, &[], it).1[0].0, 1); // data
    call(&mut vm, 0x3692_0002, 0, &[], it); // next
    assert_eq!(call(&mut vm, 0x3692_0004, 1, &[], it).1[0].0, 3);
    call(&mut vm, 0x3692_0001, 0, &[], it); // prev
    call(&mut vm, 0x3692_0001, 0, &[], it); // prev
    assert_eq!(call(&mut vm, 0x3692_0003, 1, &[], it).1[0].0, 3); // key
    assert_eq!(call(&mut vm, 0x3691_0004, 1, &[7], mm).1[0].0, 2); // count
}

#[test]
fn position_glue_addresses_cells_and_formats_coordinates() {
    use crate::native::ntype;
    let mut vm = vm();
    let new_pos = |vm: &mut Vm| {
        vm.refs
            .create_native(ntype::POSITION, NativeObject::new_empty(ntype::POSITION))
    };
    let a = new_pos(&mut vm);
    // FromCellCoords(1019, 1021): the middle of that land cell.
    assert_eq!(call(&mut vm, 0x005A_0029, 1, &[1019, 1021], a).1[0].0, 1);
    let (_, r) = call(&mut vm, 0x005A_0025, 1, &[], a); // ToCellCoordsString
    assert_eq!(vm.wstring(r[0].0), Some(w("1019 1021").as_slice()));
    let (_, r) = call(&mut vm, 0x005A_0003, 1, &[], a); // GetCellID
    assert_eq!(r[0].0, 0x7F7F_001E);
    let Some(NativeObject::Position(p)) = vm.native(a) else {
        panic!()
    };
    assert_eq!(p.origin[..2], [70.0, 110.0]);
    let (_, r) = call(&mut vm, 0x005A_0013, 1, &[], a); // ToRealMapCoords
    assert_eq!(vm.wstring(r[0].0), Some(w("0.1N, 0.1W").as_slice()));
    // ToWPString / FromWPString round trip.
    let (_, r) = call(&mut vm, 0x005A_0010, 1, &[], a);
    let text = vm.wstring(r[0].0).unwrap().to_vec();
    assert_eq!(
        String::from_utf16_lossy(&text),
        "0x7F7F001E [70.000000 110.000000 0.000000] 1.000000 0.000000 0.000000 0.000000"
    );
    let b = new_pos(&mut vm);
    let s = vm.new_wstring(text);
    assert_eq!(call(&mut vm, 0x005A_000F, 1, &[s], b).1[0].0, 1); // FromWPString
    assert_eq!(call(&mut vm, 0x005A_0017, 1, &[a], b).1[0].0, 1); // equals
    assert_eq!(call(&mut vm, 0x005A_0005, 1, &[a], b).1[0].0, 1); // in_same_block
    assert_eq!(call(&mut vm, 0x005A_0021, 1, &[], b).1[0].0, 1); // IsValid

    // Twenty metres east, one cell over: distance 20.
    call(&mut vm, 0x005A_0029, 1, &[1020, 1021], b);
    let (_, r) = call(&mut vm, 0x005A_000A, 1, &[b], a); // distance
    assert_eq!(f32::from_bits(r[0].0), 20.0);
    let (_, r) = call(
        &mut vm,
        0x005A_0023,
        1,
        &[0x7F7F_001E, 0x7F7F_0026],
        u32::MAX,
    );
    assert_eq!(f32::from_bits(r[0].0), 20.0); // GetCellDistance
}

#[test]
fn skill_ui_node_glue_edits_the_wire_node() {
    use crate::native::ntype;
    let mut vm = vm();
    let n = vm.refs.create_native(
        ntype::SKILL_UI_NODE,
        NativeObject::new_empty(ntype::SKILL_UI_NODE),
    );
    call(&mut vm, 0x00AE_0002, 0, &[0x4000_0123], n); // SetEnum
    call(&mut vm, 0x00AE_000C, 0, &[3, 4], n); // SetCoordinates
    assert_eq!(call(&mut vm, 0x00AE_0011, 1, &[9], n).1[0].0, 1); // AddDependency
    assert_eq!(call(&mut vm, 0x00AE_0011, 1, &[9], n).1[0].0, 0);
    assert_eq!(call(&mut vm, 0x00AE_0001, 1, &[], n).1[0].0, 0x4000_0123); // GetEnum
    assert_eq!(call(&mut vm, 0x00AE_000E, 1, &[], n).1[0].0, 4); // GetY
    let (_, r) = call(&mut vm, 0x00AE_0012, 1, &[], n); // GetDependencyList
    assert!(matches!(vm.native(r[0].0), Some(NativeObject::List(l)) if l.items == vec![9]));
    call(&mut vm, 0x00AE_0017, 0, &[], n); // Clear
    assert_eq!(call(&mut vm, 0x00AE_000D, 1, &[], n).1[0].0, u32::MAX); // GetX -1
}
