//! Real-data checks (only with `AC2_DAT_DIR`) of child part groups that animate through an
//! `AnimMap` of their own (`CPartGroup::DoAnimation`, `GetComboCVel`, `GetHoldLocations`).

use ac2_core::DataId;
use ac2_db::ResourceStore;

use crate::anim_real_data_tests::run;
use crate::loader_anim::load_body_animation;

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
/// The human visual's child part group and its `AnimMap` 0x03000075.
const HUMAN_CHILD_GROUP: u32 = 0x8100_0009;

#[test]
fn the_human_visual_loads_its_child_group_with_animators() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    let animation = load_body_animation(&store, DataId::new(HUMAN_MALE_VISUAL))
        .expect("animation decodes")
        .expect("human animation");
    let child = animation
        .child_groups
        .iter()
        .find(|group| group.key == HUMAN_CHILD_GROUP)
        .expect("the child group with its own AnimMap");
    assert!(!child.animators.is_empty(), "its map names animators");
    assert!(!child.setup.bones.is_empty());
}

/// A visual whose child part group 0x81000005 has its own `AnimMap` of 217 animators over a
/// six-bone Setup.
const ANIMATED_CHILD_VISUAL: u32 = 0x1F00_0026;
const ANIMATED_CHILD_GROUP: u32 = 0x8100_0005;

fn child_bones(body: &crate::body::PhysicsBody, key: u32) -> Vec<ac2_anim::Ac2Transform> {
    body.child_bone_transforms()
        .into_iter()
        .find(|(group, _)| *group == key)
        .expect("the child group is posed")
        .1
}

#[test]
fn a_child_groups_own_animator_poses_its_bones_over_time() {
    let Some((terrain, mut body)) = crate::anim_real_data_tests::scene_for(ANIMATED_CHILD_VISUAL)
    else {
        return;
    };
    run(&terrain, &mut body, 0.5);
    let rest = child_bones(&body, ANIMATED_CHILD_GROUP);
    assert_eq!(rest.len(), 6);
    body.do_motion([0.0, 1.0, 0.0]);
    let mut poses = Vec::new();
    for _ in 0..16 {
        run(&terrain, &mut body, 0.125);
        poses.push(child_bones(&body, ANIMATED_CHILD_GROUP));
    }
    for bones in &poses {
        for bone in bones {
            assert!(bone.offset.is_finite() && bone.rotation.is_finite() && bone.scale.is_finite());
        }
    }
    assert!(
        poses.iter().any(|bones| *bones != rest),
        "the child's animator moves its bones"
    );
    assert!(
        poses.windows(2).any(|pair| pair[0] != pair[1]),
        "and the pose keeps changing over time"
    );
    let drawn = crate::child_part_poses(&body, Some(100.0));
    assert!(drawn.iter().any(|(key, _)| *key == ANIMATED_CHILD_GROUP));
}
