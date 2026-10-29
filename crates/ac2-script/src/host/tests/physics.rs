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
        run_efun(&mut host.vm, 0x0080_0015, &[0x31], 9),
        InterpStatus::Interrupted,
        "DoMode stops the context until the mode is done"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_0017, &[0x32], 9, 3),
        1,
        "SetMode answers 1"
    );
    assert_eq!(
        run_efun(&mut host.vm, 0x0080_001B, &[native], 9),
        InterpStatus::Interrupted,
        "DoBehavior stops the context until the behaviour is done"
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
        host.take_latent_physics(),
        vec![
            LatentPhysics {
                object: 0x55,
                context: 1,
                action: ObjectAction::DoMode(0x31),
            },
            LatentPhysics {
                object: 0x55,
                context: 1,
                action: ObjectAction::DoBehavior(Box::new(params.clone())),
            },
        ]
    );
    assert_eq!(
        host.take_physics(),
        vec![
            (0x55, ObjectAction::SetMode(0x32)),
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

/// `CWPhysicsObject::DoMode` is latent: the efun stops its context, and the body's `ModeDone`
/// resumes it (`Resume_DoMode`), the error written to the out parameter.
#[test]
fn a_latent_mode_resumes_the_context_when_the_body_reports_it_done() {
    use crate::vm::refs::{self, Target};
    use crate::vm::{Program, ReferenceTable};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let this = host
        .vm
        .refs
        .create_native(0, crate::native::NativeObject::new_empty(0));
    host.vm.owners.insert(this, 0x55);
    let object = ReferenceTable::new_instance(0, 1);
    let out = host.vm.refs.add(
        refs::flags::FILLED | refs::flags::KIND_HEAP,
        Target::Heap(Box::new(object)),
    );
    let mut ctx = Context::new(0x61, Some(this));
    ctx.enter(0).unwrap();
    ctx.push(0, tag::INT).unwrap();
    ctx.push(out, tag::REF).unwrap();
    ctx.push(0x31, tag::INT).unwrap();
    ctx.push(this, tag::REF).unwrap();
    assert_eq!(
        host.vm.run_efun(0x0080_0015, &mut ctx),
        InterpStatus::Interrupted
    );
    host.processes.insert(
        0x55,
        super::super::Process {
            instance: this,
            waiting: vec![ctx],
        },
    );
    assert!(
        host.resume_latent(0x55, 0x62, 0).is_none(),
        "another context"
    );
    let outcome = host
        .resume_latent(0x55, 0x61, 0)
        .expect("the context waits for the mode");
    assert_eq!(outcome.status, InterpStatus::Terminated);
    assert_eq!(outcome.context, 0x61);
    assert!(host.resume_latent(0x55, 0x61, 0).is_none(), "resumed once");
}

/// `gCWPhysicsObject::CallEFun_StopAllCameraFX` queues the stop for the weenie's physics object
/// (the client ends the camera's effects when it is the player); an undefined `this` crashes.
#[test]
fn stop_all_camera_fx_queues_the_weenies_object() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    host.vm.owners.insert(9, 0x55);
    assert_eq!(
        run_efun(&mut host.vm, 0x0080_002C, &[], 9),
        InterpStatus::Success
    );
    assert_eq!(host.take_camera_fx_stops(), vec![0x55]);
    assert!(host.take_camera_fx_stops().is_empty());
    assert_ne!(
        run_efun(&mut host.vm, 0x0080_002C, &[], u32::MAX),
        InterpStatus::Success
    );
}
