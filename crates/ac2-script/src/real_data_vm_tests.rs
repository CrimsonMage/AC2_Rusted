//! The interpreter over the Legions library (`AC2_DAT_DIR`).

use std::collections::BTreeMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;

use crate::vm::{
    tag, Context, EFuns, Fault, FunctionId, InitState, InterpStatus, Program, StateRef,
    StateSource, Vm,
};
use crate::{load_wlib, load_wstate, LEGIONS_WLIB_DID};

/// WStates from the DAT cache (the store decodes them on first use).
struct StoreStates(ResourceStore);

impl StateSource for StoreStates {
    fn wstate(&self, did: u32) -> Option<StateRef> {
        load_wstate(&self.0, DataId::new(did))
            .ok()
            .flatten()
            .map(|state| Box::new(state) as StateRef)
    }
}

fn library_vm() -> Option<Vm> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ResourceStore::new(dataset);
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .expect("the library");
    let program = Program::from_stream(&wlib.stream);
    Some(Vm::new(
        Arc::new(program),
        Box::new(StoreStates(store)),
        EFuns::new(),
    ))
}

/// Every package's initializer, on a fresh instance, with no efun bound: the interpreter must
/// never meet an opcode, jump or stack state it cannot handle; initializers that reach an
/// efun stop there (`UnboundEFun`), the rest run to completion.
#[test]
fn every_package_initializer_runs_or_stops_at_an_unbound_efun() {
    let Some(mut vm) = library_vm() else {
        return;
    };
    let packages: Vec<(String, u32)> = vm
        .program
        .package_names
        .iter()
        .map(|(n, id)| (String::from_utf8_lossy(n).into_owned(), *id))
        .collect();
    let mut outcomes: BTreeMap<String, usize> = BTreeMap::new();
    let mut efuns: BTreeMap<u32, usize> = BTreeMap::new();
    let mut odd = Vec::new();
    for (name, pid) in &packages {
        let instance = vm
            .allocate_package(*pid, InitState::DontInit, false)
            .expect("fresh instance");
        let mut ctx = Context::new(0, None);
        ctx.push(0, tag::INT).unwrap();
        ctx.push(instance, tag::REF).unwrap();
        vm.last_fault = None;
        let status = vm.call_ifun(&mut ctx, *pid, FunctionId::new(*pid, 0, false));
        let key = match (&status, &vm.last_fault) {
            (InterpStatus::Crash, Some(Fault::UnboundEFun(id))) => {
                *efuns.entry(*id).or_default() += 1;
                "stopped at an unbound efun".to_string()
            }
            (InterpStatus::Crash, Some(fault)) => {
                if odd.len() < 20 {
                    odd.push(format!("{name} ({pid}): {fault:?}"));
                }
                format!("crash: {fault:?}")
                    .split('(')
                    .next()
                    .unwrap_or_default()
                    .to_string()
            }
            (status, _) => format!("{status:?}"),
        };
        *outcomes.entry(key).or_default() += 1;
    }
    let mut top: Vec<_> = efuns.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("{} packages: {outcomes:?}", packages.len());
    eprintln!(
        "most-needed efuns: {:x?}",
        top.iter().take(15).collect::<Vec<_>>()
    );
    eprintln!("other crashes: {odd:#?}");
    eprintln!("references: {}", vm.refs.entries.len());
    // With no glue bound, only the initializers that never call an efun complete (570 of
    // 9,874 in Legions); binding the pure glue raises this.
    assert!(outcomes.get("Terminated").copied().unwrap_or(0) >= 500);
    for bad in ["crash: UnknownOpcode", "crash: BadJump", "crash: Stack"] {
        assert!(!outcomes.contains_key(bad), "{bad}: {odd:#?}");
    }
}
