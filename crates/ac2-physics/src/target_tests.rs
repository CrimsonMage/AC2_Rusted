use super::*;

fn owner(id: u64, x: f32) -> TargetOwner {
    let mut position = Position {
        cell_id: 0x0101_0001,
        ..Position::default()
    };
    position.frame.origin = Vector::new(x, 0.0, 0.0);
    TargetOwner {
        id,
        position,
        velocity: Vector::new(1.0, 0.0, 0.0),
        radius: 0.5,
        height: 2.0,
        scale: Vector::new(1.0, 1.0, 1.0),
    }
}

#[test]
fn set_target_with_no_object_is_a_timed_out_notice() {
    let mut m = TargetManager::new();
    let fx = m.set_target(7, 3, 0, 1.0, 0.5, 10.0);
    assert!(m.target_info().is_none());
    match &fx[..] {
        [TargetEffect::Notice(info)] => {
            assert_eq!(info.context_id, 3);
            assert_eq!(info.status, TargetStatus::TimedOut);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn set_target_replaces_the_old_target_and_asks_to_watch_the_new_one() {
    let mut m = TargetManager::new();
    m.set_target(7, 1, 20, 2.0, 0.5, 1.0);
    let fx = m.set_target(7, 2, 21, 3.0, 0.25, 2.0);
    assert_eq!(
        fx,
        vec![
            TargetEffect::RemoveVoyeur {
                target: 20,
                voyeur: 7
            },
            TargetEffect::AddVoyeur {
                target: 21,
                voyeur: 7,
                radius: 3.0,
                quantum: 0.25
            },
        ]
    );
    assert_eq!(m.target_info().unwrap().last_update_time, 2.0);
}

#[test]
fn a_new_voyeur_gets_the_position_and_a_known_one_only_new_limits() {
    let me = owner(5, 10.0);
    let mut m = TargetManager::new();
    let fx = m.add_voyeur(&me, 9, 2.0, 1.0);
    match &fx[..] {
        [TargetEffect::Update { to: 9, info }] => {
            assert_eq!(info.status, TargetStatus::Ok);
            assert_eq!(info.object_id, 5);
            assert_eq!(info.interpolated_position, me.position);
            assert_eq!(info.object_height, 2.0);
        }
        other => panic!("{other:?}"),
    }
    assert!(m.add_voyeur(&me, 9, 4.0, 2.0).is_empty());
    let v = m.voyeurs().next().unwrap();
    assert_eq!((v.radius, v.quantum), (4.0, 2.0));
}

#[test]
fn targetting_runs_every_half_second_and_updates_voyeurs_that_were_outrun() {
    let mut me = owner(5, 0.0);
    let mut m = TargetManager::new();
    m.add_voyeur(&me, 9, 1.5, 2.0);
    // The voyeur was told x = 0; at a quantum of 2 the body is predicted at x = 2 > radius.
    assert!(m.handle_targetting(&me, 0.2).is_empty());
    let fx = m.handle_targetting(&me, 0.6);
    assert_eq!(fx.len(), 1);
    // Standing still sends nothing more.
    me.velocity = Vector::ZERO;
    me.position.frame.origin.x = 2.0;
    assert!(m.handle_targetting(&me, 1.2).is_empty());
}

#[test]
fn an_unheard_target_times_out_after_ten_seconds() {
    let me = owner(5, 0.0);
    let mut m = TargetManager::new();
    m.set_target(5, 1, 20, 1.0, 0.5, 0.0);
    assert!(m.handle_targetting(&me, 9.0).is_empty());
    let fx = m.handle_targetting(&me, 10.6);
    assert!(matches!(&fx[..], [TargetEffect::Notice(i)] if i.status == TargetStatus::TimedOut));
}

#[test]
fn receive_update_aims_the_heading_at_the_target() {
    let me = owner(5, 0.0);
    let mut m = TargetManager::new();
    m.set_target(5, 1, 20, 1.0, 0.5, 0.0);
    let mut update = TargetInfo {
        object_id: 20,
        status: TargetStatus::Ok,
        ..Default::default()
    };
    update.interpolated_position = owner(20, 0.0).position;
    update.interpolated_position.frame.origin = Vector::new(0.0, 4.0, 0.0);
    let fx = m.receive_update(&me, &update, 3.0);
    assert_eq!(fx.len(), 1);
    let info = m.target_info().unwrap();
    assert_eq!(info.interpolated_heading, Vector::new(0.0, 1.0, 0.0));
    assert_eq!(info.last_update_time, 3.0);
    // Another object's update is ignored.
    update.object_id = 21;
    assert!(m.receive_update(&me, &update, 4.0).is_empty());
    // On top of the body the heading is straight up.
    update.object_id = 20;
    update.interpolated_position = me.position;
    m.receive_update(&me, &update, 5.0);
    assert_eq!(
        m.target_info().unwrap().interpolated_heading,
        Vector::new(0.0, 0.0, 1.0)
    );
}

#[test]
fn leaving_the_world_clears_the_target_and_tells_voyeurs() {
    let me = owner(5, 0.0);
    let mut m = TargetManager::new();
    m.set_target(5, 1, 20, 1.0, 0.5, 0.0);
    m.add_voyeur(&me, 9, 1.0, 1.0);
    let fx = m.handle_leave_world(&me);
    assert_eq!(fx.len(), 2);
    assert!(m.target_info().is_none());
    assert!(
        matches!(&fx[1], TargetEffect::Update { to: 9, info } if info.status == TargetStatus::ExitWorld)
    );
    assert!(m.remove_voyeur(9));
    assert!(!m.remove_voyeur(9));
}
