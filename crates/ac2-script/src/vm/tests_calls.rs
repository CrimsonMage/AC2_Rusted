//! Calls, packages, efuns, references and singletons against hand-built bytecode.

use std::sync::Arc;

use super::tests::{run, w};
use super::*;
use crate::bytestream::{ByteStream, PackageInfo, VTableId};
use crate::native::NativeObject;
use crate::reference::{flags as packed, HeapObject, PackedEntry, PackedReference};

const T: u32 = op::TERMINATE;

/// Package 0 with two functions: 0 (the initializer) and 1, at the given word offsets.
fn package_vm(code: Vec<u32>, init_at: u32, method_at: u32, size: u32) -> Vm {
    let mut program = Program::from_parts(
        code,
        vec![vec![0, 1]],
        vec![vec![init_at * 4, method_at * 4]],
    );
    program.package_sizes = vec![size];
    program.string_literals = vec!["hi".encode_utf16().collect()];
    Vm::new(Arc::new(program), Box::new(NoData), EFuns::new())
}

#[test]
fn the_initializer_runs_on_a_fresh_instance_and_methods_reach_this() {
    let code = vec![
        // 0: initializer: this.slot[1] = 7
        w(op::PUSH_FRAME, 0, 0),
        w(op::PUSHV, 2, 7),
        op::LOAD_THIS,
        w(op::RSTORE, 0, 1),
        op::POP,
        op::POP_FRAME,
        // 6: method: this.slot[0] = 42; return 77
        w(op::PUSH_FRAME, 0, 0),
        w(op::PUSHV, 2, 42),
        op::LOAD_THIS,
        w(op::RSTORE, 0, 0),
        op::POP,
        w(op::PUSHV, 2, 77),
        w(op::PUSHV, 2, -2),
        op::STORE,
        op::POP,
        op::POP_FRAME,
    ];
    let mut vm = package_vm(code, 0, 6, 2);
    let this = vm.allocate_package(0, InitState::NoWState, false).unwrap();
    let object = vm.refs.heap(this).unwrap();
    assert_eq!((object.slots[1], object.tags[1]), (7, tag::INT));
    assert_eq!(object.tags[0], tag::UNINIT);

    let mut ctx = Context::new(1, None);
    ctx.push(0, tag::INT).unwrap();
    ctx.push(this, tag::REF).unwrap();
    let status = vm.call_ifun(&mut ctx, 0, FunctionId::new(0, 1, false));
    assert_eq!(status, InterpStatus::Terminated);
    assert_eq!(vm.refs.heap(this).unwrap().slots[0], 42);
    assert_eq!((ctx.sp, ctx.data[0], ctx.tags[0]), (2, 77, tag::INT));

    // DontInit leaves the slots uninitialised.
    let bare = vm.allocate_package(0, InitState::DontInit, false).unwrap();
    assert_eq!(vm.refs.heap(bare).unwrap().tags, vec![tag::UNINIT; 2]);
}

#[test]
fn script_calls_dispatch_through_the_vtable_and_missing_functions_do_nothing() {
    let code = vec![
        // 0: initializer: nothing
        w(op::PUSH_FRAME, 0, 0),
        op::POP_FRAME,
        // 2: method 1: return 5
        w(op::PUSH_FRAME, 0, 0),
        w(op::PUSHV, 2, 5),
        w(op::PUSHV, 2, -2),
        op::STORE,
        op::POP,
        op::POP_FRAME,
        // 8: entry: ret = this.method1(); then a call to a function the vtable lacks
        w(op::PUSH_FRAME, 0, 0),
        w(op::PUSHV, 2, 0),
        op::LOAD_THIS,
        op::CALL,
        FunctionId::new(0, 1, false).0,
        w(op::POPN, 0, 1),
        op::THIS_CALL,
        FunctionId::new(0, 7, false).0,
        T,
    ];
    let mut vm = package_vm(code, 0, 2, 0);
    let this = vm.allocate_package(0, InitState::DontInit, false).unwrap();
    let mut ctx = Context::new(1, None);
    ctx.push(this, tag::REF).unwrap();
    ctx.enter(8).unwrap();
    let status = vm.execute(&mut ctx);
    assert_eq!(status, InterpStatus::Terminated);
    // ret (5), then THIS_CALL's pushed `this` with no call made.
    assert_eq!(&ctx.data[ctx.sp - 2..ctx.sp], &[5, this]);
}

fn double_this(_vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let value = ctx.arg(1).unwrap();
    ctx.set_arg(2, value * 2, tag::INT);
    InterpStatus::Success
}

#[test]
fn efuns_see_this_below_the_frame_and_write_the_return_slot() {
    let code = vec![
        w(op::PUSHV, 2, 0),
        w(op::PUSHV, 2, 21),
        op::CALL_EFUN,
        0x0065_0001,
        w(op::POPN, 0, 1),
        T,
    ];
    let mut vm = tests::vm_with(code.clone(), Vec::new(), Vec::new());
    vm.efuns.register(0x0065_0001, double_this);
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    assert_eq!(vm.execute(&mut ctx), InterpStatus::Terminated);
    assert_eq!((ctx.sp, ctx.data[0], ctx.tags[0]), (1, 42, tag::INT));

    let (vm, _, status) = run(code);
    assert_eq!(status, InterpStatus::Crash);
    assert_eq!(vm.last_fault, Some(Fault::UnboundEFun(0x0065_0001)));
}

#[test]
fn addresses_reach_locals_and_heap_slots_and_strings_are_native() {
    let code = vec![
        // 0: initializer: slot[1] = 8
        w(op::PUSH_FRAME, 0, 0),
        w(op::PUSHV, 2, 8),
        op::LOAD_THIS,
        w(op::RSTORE, 0, 1),
        op::POP,
        op::POP_FRAME,
        // 6: entry
        w(op::PUSH_FRAME, 0, 1),
        w(op::PUSHV, 2, 5),
        w(op::PUSHV, 2, 0),
        op::STORE,
        op::POP,
        w(op::SADDR, 0, 0),
        w(op::RLOAD, 0, 0),
        op::LOAD_THIS,
        w(op::RADDR, 0, 1),
        w(op::RLOAD, 0, 0),
        w(op::THIS_RLOAD, 0, 1),
        op::NEW_STRING,
        0,
        T,
    ];
    let mut vm = package_vm(code, 0, 0, 2);
    let this = vm.allocate_package(0, InitState::NoWState, false).unwrap();
    let mut ctx = Context::new(1, None);
    ctx.push(this, tag::REF).unwrap();
    ctx.enter(6).unwrap();
    assert_eq!(vm.execute(&mut ctx), InterpStatus::Terminated);
    let n = ctx.sp;
    assert_eq!(&ctx.data[n - 4..n - 1], &[5, 8, 8]);
    let literal = ctx.data[n - 1];
    assert_eq!(ctx.tags[n - 1], tag::REF);
    let native = &vm.refs.heap(literal).unwrap().native;
    assert_eq!(
        native,
        &Some(NativeObject::WString("hi".encode_utf16().collect()))
    );
}

struct OneState(Arc<WState>);

impl ScriptData for OneState {
    fn wstate(&self, did: u32) -> Option<StateRef> {
        (did == self.0.did).then(|| Box::new(self.0.clone()) as StateRef)
    }
}

#[test]
fn a_singleton_fills_from_its_wstate_on_first_use() {
    let object = HeapObject {
        pkg_state_id: 0,
        pkg_id: 3,
        native_type: 0,
        native: None,
        slots: vec![99],
        tags: vec![tag::INT],
    };
    let state = WState {
        did: crate::wstate::singleton_did(3),
        package_state: Vec::new(),
        pid: Some(3),
        references: Ok(PackedReference {
            ids: vec![0],
            entries: vec![PackedEntry::Heap {
                flags: packed::KIND_HEAP,
                object,
            }],
        }),
    };
    let program = Program::from_parts(vec![T], Vec::new(), Vec::new());
    let mut vm = Vm::new(
        Arc::new(program),
        Box::new(OneState(Arc::new(state))),
        EFuns::new(),
    );
    let singleton = vm.singleton_package(3);
    assert_eq!(vm.singleton_package(3), singleton, "one entry per WState");
    assert_eq!(
        vm.refs.get(singleton).unwrap().blob & refs::flags::FILLED,
        0
    );
    assert_eq!(vm.package_id(singleton), Ok(3));
    let object = vm.refs.heap(singleton).unwrap();
    assert_eq!(object.slots, vec![99]);
    assert_eq!(object.pkg_state_id, 0x7000_0003);
    // The class-default state also seeds `NEW` instances, without the initializer.
    let instance = vm.allocate_package(3, InitState::UseWState, false).unwrap();
    assert_ne!(instance, singleton);
    assert_eq!(vm.refs.heap(instance).unwrap().slots, vec![99]);
}

#[test]
fn loading_rewrites_virtual_calls_to_static_ones() {
    let mut stream = ByteStream::default();
    let words = [
        op::CALL,
        0x8000_0001,
        op::CALL,
        0x0000_0001,
        w(op::SWITCH, 0, 0),
        0,
        1,
        op::CALL,
        0x8000_0000,
        T,
    ];
    stream.opcodes = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    stream.vtables.func_mapper = vec![vec![VTableId(0), VTableId(1)]];
    stream.vtables.vtables = vec![vec![0x10, 0x20]];
    stream.vtables.package_id_map = vec![0];
    stream.vtables.package_info = vec![PackageInfo::default()];
    let program = Program::from_stream(&stream);
    assert_eq!(&program.code[..4], &[op::CALL_STATIC, 0x20, op::CALL, 1]);
    // The switch's (value, offset) pairs are walked over, not mistaken for calls.
    assert_eq!(&program.code[5..7], &[0, 1]);
    assert_eq!(&program.code[7..9], &[op::CALL, 0x8000_0000]);
}
