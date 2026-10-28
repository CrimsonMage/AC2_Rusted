//! Which efuns the Legions library calls and which of them the host binds.

use std::collections::BTreeMap;

use super::*;
use crate::vm::op;

/// Every efun id the library's code calls (`CALL_EFUN`, `THIS_CALL_EFUN`, `LOAD_CALL_EFUN`,
/// `RLOAD_CALL_EFUN` operands), with the number of call sites.
fn referenced_efuns(program: &Program) -> BTreeMap<u32, usize> {
    let mut found = BTreeMap::new();
    let mut pc = 0;
    while pc < program.code.len() {
        let code = program.code[pc] & 0xFF;
        if matches!(
            code,
            op::CALL_EFUN | op::THIS_CALL_EFUN | op::LOAD_CALL_EFUN | op::RLOAD_CALL_EFUN
        ) {
            if let Some(&efun) = program.code.get(pc + 1) {
                *found.entry(efun).or_insert(0) += 1;
            }
        }
        pc = program.walk(pc);
    }
    found
}

/// The efuns the library calls that no glue binds. The count only goes down.
#[test]
fn every_efun_the_library_calls_is_bound() {
    let Some(host) = legions_host() else {
        return;
    };
    let efuns = host_efuns();
    let referenced = referenced_efuns(&host.vm().program);
    let unbound: Vec<(u32, usize)> = referenced
        .iter()
        .filter(|(id, _)| !efuns.is_bound(**id))
        .map(|(id, n)| (*id, *n))
        .collect();
    let mut by_class: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, n) in &unbound {
        let name = crate::glue::efun_name(*id).unwrap_or("?");
        let class = name.split("::").next().unwrap_or("?").to_string();
        by_class
            .entry(class)
            .or_default()
            .push(format!("{id:08x} {name} x{n}"));
    }
    eprintln!(
        "{} efuns referenced, {} unbound",
        referenced.len(),
        unbound.len()
    );
    for (class, list) in &by_class {
        eprintln!("{class} ({}): {list:?}", list.len());
    }
    if let Ok(path) = std::env::var("AC2_DUMP_BOUND") {
        let names: Vec<String> = crate::glue::efun_table()
            .filter(|(id, _)| efuns.is_bound(*id))
            .map(|(id, n)| format!("{id:08x} {n}"))
            .collect();
        std::fs::write(path, names.join("\n")).unwrap();
    }
    assert!(unbound.len() <= 84, "{} unbound", unbound.len());
}
