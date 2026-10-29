//! `CBaseQualities::m_defaults`: the class defaults a weenie's script reads and adjustments fall
//! back to.

use super::*;

/// A data source holding one class default record (`CBaseQualities::m_defaults`).
struct OneClassDefault(u32, ac2_weenie::WeenieQualities);

impl crate::vm::ScriptData for OneClassDefault {
    fn wstate(&self, _did: u32) -> Option<crate::vm::StateRef> {
        None
    }

    fn class_defaults(&self, did: u32) -> Option<Arc<ac2_weenie::WeenieQualities>> {
        (did == self.0).then(|| Arc::new(self.1.clone()))
    }
}

/// `CBaseQualities::AdjInt` on a quality its table lacks starts from the class default
/// (`m_defaults`, `InqInt`), not from 0; the result is stored in the table. A quality with no
/// default starts from 0.
#[test]
fn adj_on_an_absent_quality_starts_from_the_class_default() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut class = ac2_weenie::WeenieQualities::default();
    class.ints.insert(0x14, 10);
    let mut host = ScriptHost::new(
        Arc::new(program),
        Box::new(OneClassDefault(0x8000_00AB, class)),
    );
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0001);
    host.set_qualities(
        0x5000_0001,
        ac2_weenie::WeenieQualities {
            did: 0x8000_00AB,
            ..Default::default()
        },
    );
    // AdjInt(0x14, 5, 0): the default 10 + 5.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x14, 5, 0], this);
    // AdjInt(0x15, 4, 0): no default, so 0 + 4.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x15, 4, 0], this);
    let q = host.qualities(0x5000_0001).unwrap();
    assert_eq!(
        (q.ints.get(&0x14), q.ints.get(&0x15)),
        (Some(&15), Some(&4))
    );
    // The table holds the value now: the next adjustment works on it, not the default.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x14, 1, 0], this);
    assert_eq!(
        host.qualities(0x5000_0001).unwrap().ints.get(&0x14),
        Some(&16)
    );
}

/// `CWRepository::GetInt` (`CBaseQualities::InqInt` over the default): a quality the weenie's
/// table lacks answers from its class defaults (`m_defaults`) before the caller's default.
#[test]
fn get_on_an_absent_quality_answers_the_class_default() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut class = ac2_weenie::WeenieQualities::default();
    class.ints.insert(0x14, 10);
    let mut host = ScriptHost::new(
        Arc::new(program),
        Box::new(OneClassDefault(0x8000_00AB, class)),
    );
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0001);
    let mut own = ac2_weenie::WeenieQualities {
        did: 0x8000_00AB,
        ..Default::default()
    };
    own.ints.insert(0x16, 3);
    host.set_qualities(0x5000_0001, own);
    let get_int = crate::glue::efun_table()
        .find(|(_, name)| *name == "gCWRepository::CallEFun_GetInt")
        .map(|(id, _)| id)
        .expect("GetInt efun");
    let vm = host.vm_mut();
    assert_eq!(
        efun_result(vm, get_int, &[0x14, 99], this, 4),
        10,
        "class default"
    );
    assert_eq!(
        efun_result(vm, get_int, &[0x16, 99], this, 4),
        3,
        "own table first"
    );
    assert_eq!(
        efun_result(vm, get_int, &[0x15, 99], this, 4),
        99,
        "caller's default"
    );
}
