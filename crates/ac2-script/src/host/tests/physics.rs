//! The script's physics efuns on a weenie's physics object (`gCWPhysicsObject`): the queued server
//! events for DoMode, SetMode, DoBehavior, ReleaseBehavior and StopBehavior.

use super::*;

/// `gCWPhysicsObject`'s DoMode, SetMode, DoBehavior, ReleaseBehavior and StopBehavior efuns queue
/// the server event each one is for the weenie's physics object. The behaviour efuns take a
/// `BehaviorParams` native (an argument that is not one crashes the script); an undefined `this`
/// crashes too.
#[test]
fn physics_mode_and_behavior_efuns_queue_their_server_events() {
    use ac2_net::protocol::{BehaviorParams, ObjectAction};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    host.vm.owners.insert(9, 0x55);
    let params = BehaviorParams {
        behavior: 0x41,
        ..BehaviorParams::default()
    };
    let native = host.vm.refs.create_native(
        0,
        crate::native::NativeObject::BehaviorParams(Box::new(params.clone())),
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_0015, &[0x31], 9, 4),
        1,
        "DoMode answers 1 for an owned weenie"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_0017, &[0x32], 9, 3),
        1,
        "SetMode answers 1"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_001B, &[native], 9, 4),
        0,
        "DoBehavior answers 0"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_001E, &[0x41, native], 9, 5),
        0,
        "ReleaseBehavior answers 0"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_0027, &[0x41], 9, 3),
        1,
        "StopBehavior answers 1"
    );
    assert_eq!(
        host.take_physics(),
        vec![
            (0x55, ObjectAction::DoMode(0x31)),
            (0x55, ObjectAction::SetMode(0x32)),
            (0x55, ObjectAction::DoBehavior(Box::new(params.clone()))),
            (
                0x55,
                ObjectAction::ReleaseBehavior {
                    behavior: 0x41,
                    params: Box::new(params),
                }
            ),
            (0x55, ObjectAction::StopBehavior(0x41)),
        ]
    );
    assert!(host.take_physics().is_empty());
    assert_ne!(
        run_efun(&mut host.vm, 0x0080_001B, &[native], u32::MAX),
        InterpStatus::Success,
        "an undefined this crashes the script"
    );
    assert_ne!(
        run_efun(&mut host.vm, 0x0080_001B, &[0x1234], 9),
        InterpStatus::Success,
        "a DoBehavior argument that is not a BehaviorParams crashes the script"
    );
}

/// `gCWPhysicsObject::CallEFun_LookAt(iid)` queues the server's `LookAt` for the weenie's
/// object and answers 1 for an owned weenie. The 64-bit id takes two argument words (the high
/// word nearest `this`); an undefined `this` crashes the script.
#[test]
fn look_at_efun_queues_the_look_for_the_owned_weenie() {
    use ac2_net::protocol::ObjectAction;
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    host.vm.owners.insert(9, 0x55);
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_002A, &[0, 0x41], 9, 4),
        1,
        "LookAt answers 1 for an owned weenie"
    );
    assert_eq!(
        host.take_physics(),
        vec![(0x55, ObjectAction::LookAt(0x41))]
    );
    assert_ne!(
        run_efun(&mut host.vm, 0x0080_002A, &[0, 0x41], u32::MAX),
        InterpStatus::Success,
        "an undefined this crashes the script"
    );
}
