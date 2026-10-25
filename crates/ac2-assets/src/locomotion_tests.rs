//! The human locomotion cube against real data: its sliders select the movement clips the posed
//! body decodes with.

use ac2_anim::{
    load_ac2_motion_animator, AC2_SLIDER_FORWARD_SPEED, AC2_SLIDER_SIDESTEP_SPEED,
    AC2_SLIDER_TURN_RATE,
};
use ac2_core::DataId;

/// Real data: the human `AnimMap` locomotion motion `0x80000001` is a mode switch over an
/// `AMHuman` cube whose sliders pick the measured walk, run, backward, sidestep and turn clips.
#[test]
fn human_locomotion_cube_selects_movement_clips_by_slider() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let animator = load_ac2_motion_animator(&store, DataId::new(0x0300_0014), 0x8000_0001)
        .expect("locomotion animator decodes")
        .expect("motion 0x80000001 is mapped");
    let single = |sliders: &[(u32, f32)]| {
        let blend = animator.blend(sliders);
        let clips: Vec<_> = blend
            .clips()
            .map(|(clip, weight)| (clip.period, weight))
            .collect();
        assert_eq!(clips.len(), 1, "{sliders:?} selects one clip: {clips:?}");
        assert!((clips[0].1 - 1.0).abs() < 1e-4);
        clips[0].0
    };
    let close = |period: f32, expected: f32| (period - expected).abs() < 0.01;
    // Standing: the ready idle 0x05000029.
    assert!(close(single(&[]), 2.458));
    // Walk forward 0x0500004F, run forward 0x05000050, walk backward 0x05000044.
    assert!(close(single(&[(AC2_SLIDER_FORWARD_SPEED, 1.5)]), 2.3));
    assert!(close(single(&[(AC2_SLIDER_FORWARD_SPEED, 5.0)]), 0.633));
    assert!(close(single(&[(AC2_SLIDER_FORWARD_SPEED, -2.0)]), 1.867));
    // Sidestep left 0x05000041 and right 0x05000049.
    assert!(close(single(&[(AC2_SLIDER_SIDESTEP_SPEED, -1.0)]), 1.3));
    assert!(close(single(&[(AC2_SLIDER_SIDESTEP_SPEED, 1.0)]), 1.3));
    // Turn right 0x05000301 (clockwise, negative) and left 0x05000302.
    assert!(close(single(&[(AC2_SLIDER_TURN_RATE, -90.0)]), 0.875));
    assert!(close(single(&[(AC2_SLIDER_TURN_RATE, 90.0)]), 0.833));
    // Between walk and run both clips play on one phase.
    let mixed = animator.blend(&[(AC2_SLIDER_FORWARD_SPEED, 2.5)]);
    assert_eq!(mixed.clips().count(), 2);
    assert!(mixed.period() > 0.633 && mixed.period() < 2.3);
    let posed = crate::decode_ac2_posed_model(
        &store,
        DataId::new(0x1F00_0023),
        Some(crate::Ac2Pose {
            animation: &mixed.still_frame(0.3),
            time: 0.0,
        }),
    )
    .unwrap()
    .unwrap();
    assert!(posed
        .surface_batches
        .iter()
        .all(|batch| batch.vertices.iter().all(|v| v.position.is_finite())));
}
