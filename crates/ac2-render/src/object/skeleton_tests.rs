//! `CPartArray::UpdateBones`, `CPartGroup::RefreshMatrices` and `BlendLitVertices` on synthetic
//! Setups, and the human visual against real data (`AC2_DAT_DIR`).

use std::sync::Arc;

use ac2_anim::{Ac2Bone, Ac2Location, Ac2PlacementFrame, Ac2Setup, Ac2Transform};
use ac2_core::DataId;
use ac2_math::{Mat4, Quat, Vec3};

use super::skeleton::{blend_vertex, fill_matrix, multiply, to_mat4, ObjectSkeleton};
use super::SkeletonGroup;

fn transform(offset: [f32; 3], scale: f32, rotation: Quat) -> Ac2Transform {
    Ac2Transform {
        offset: Vec3::from(offset),
        scale: Vec3::splat(scale),
        rotation,
    }
}

/// Two bones (root, child of root), two meshes, frames 0 and 1, one connection point on bone 1.
fn setup(default: [Ac2Transform; 2], rest: [Ac2Transform; 2]) -> Ac2Setup {
    Ac2Setup {
        meshes: Arc::from([DataId::new(0x0100_0001), DataId::new(0x0100_0002)]),
        bones: Arc::from([
            Ac2Bone {
                parent: None,
                name: 0,
            },
            Ac2Bone {
                parent: Some(0),
                name: 1,
            },
        ]),
        placements: Arc::from([
            Ac2PlacementFrame {
                kind: 0,
                bones: Arc::from(default),
            },
            Ac2PlacementFrame {
                kind: 1,
                bones: Arc::from(rest),
            },
        ]),
        holding_points: Arc::from([]),
        connection_points: Arc::from([Ac2Location {
            id: 7,
            bone: 1,
            transform: transform([0.0, 0.0, 0.5], 1.0, Quat::IDENTITY),
        }]),
        degrade: Default::default(),
        bounds_min: Vec3::ZERO,
        bounds_max: Vec3::ONE,
    }
}

fn close(a: Mat4, b: Mat4) -> bool {
    a.abs_diff_eq(b, 1e-5)
}

#[test]
fn fill_matrix_scales_rotation_rows_and_keeps_the_offset() {
    let t = transform(
        [1.0, 2.0, 3.0],
        2.0,
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    let scaled = to_mat4(&fill_matrix(&t, true));
    let expected = Mat4::from_scale_rotation_translation(t.scale, t.rotation, t.offset);
    assert!(close(scaled, expected), "{scaled:?}");
    let unscaled = to_mat4(&fill_matrix(&t, false));
    assert!(close(
        unscaled,
        Mat4::from_rotation_translation(t.rotation, t.offset)
    ));
    // Row vectors: `a` then `b`.
    let a = fill_matrix(&transform([1.0, 0.0, 0.0], 1.0, Quat::IDENTITY), false);
    let b = fill_matrix(&t, true);
    assert!(close(to_mat4(&multiply(&a, &b)), expected * to_mat4(&a)));
}

#[test]
fn rest_pose_deforms_by_identity_and_rigid_parts_ride_unscaled_bones() {
    let turn = Quat::from_rotation_x(0.3);
    let frame = [
        transform([0.0, 0.0, 1.0], 1.0, Quat::IDENTITY),
        transform([0.0, 1.0, 0.0], 2.0, turn),
    ];
    let skinned = SkeletonGroup::new(DataId::new(0x0400_0001), &setup(frame, frame), true, None);
    assert!(skinned.skinned());
    let skeleton = ObjectSkeleton {
        groups: vec![skinned.clone()],
    };
    let pose = &skeleton.pose(None)[0];
    for matrix in pose.blend_matrices().unwrap() {
        assert!(close(matrix, Mat4::IDENTITY), "{matrix:?}");
    }
    // A skinned group draws every part at the group matrix.
    assert_eq!(pose.part_matrix(1), Mat4::IDENTITY);

    let rigid = SkeletonGroup::new(DataId::new(0x0400_0001), &setup(frame, frame), false, None);
    let skeleton = ObjectSkeleton {
        groups: vec![rigid],
    };
    let pose = &skeleton.pose(None)[0];
    assert!(pose.blend_matrices().is_none());
    // Bone 1 = its own rotation and offset, then the root's offset; its scale of 2 is dropped.
    let expected = Mat4::from_translation(Vec3::Z) * Mat4::from_rotation_translation(turn, Vec3::Y);
    assert!(close(pose.part_matrix(1), expected));
}

#[test]
fn animated_root_deforms_by_posed_times_inverse_rest() {
    let rest = [
        transform([0.0, 0.0, 1.0], 1.0, Quat::IDENTITY),
        transform([0.0, 1.0, 0.0], 1.0, Quat::IDENTITY),
    ];
    let group = SkeletonGroup::new(DataId::new(0x0400_0001), &setup(rest, rest), true, None);
    let posed = [
        transform([0.0, 0.0, 1.0], 1.0, Quat::IDENTITY),
        transform([0.0, 1.0, 0.0], 1.5, Quat::from_rotation_z(0.5)),
    ];
    let skeleton = ObjectSkeleton {
        groups: vec![group],
    };
    let pose = &skeleton.pose(Some(&posed))[0];
    let world = |t: &[Ac2Transform; 2]| {
        let root = Mat4::from_scale_rotation_translation(t[0].scale, t[0].rotation, t[0].offset);
        root * Mat4::from_scale_rotation_translation(t[1].scale, t[1].rotation, t[1].offset)
    };
    let expected = world(&posed) * world(&rest).inverse();
    let matrices = pose.blend_matrices().unwrap();
    assert!(close(matrices[1], expected), "{:?}", matrices[1]);

    // A vertex at the child's rest origin, fully on bone 1, follows the posed bone.
    let (p, _) = blend_vertex(
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
        &[(1, 1.0), (0, 0.0), (0, 0.0), (0, 0.0)],
        &matrices,
    );
    assert!(
        Vec3::from(p).abs_diff_eq(Vec3::new(0.0, 1.0, 1.0), 1e-5),
        "{p:?}"
    );
    // A wrong-length bone list leaves the default placement.
    let fallback = &skeleton.pose(Some(&posed[..1]))[0];
    assert!(close(fallback.blend_matrices().unwrap()[1], Mat4::IDENTITY));
}

#[test]
fn child_group_hangs_from_the_parent_bone_and_connection_point() {
    let frame = [
        transform([0.0, 0.0, 1.0], 1.0, Quat::IDENTITY),
        transform([0.0, 1.0, 0.0], 3.0, Quat::IDENTITY),
    ];
    let parent = setup(frame, frame);
    let root = SkeletonGroup::new(DataId::new(0x0400_0001), &parent, false, None);
    let child = SkeletonGroup::new(
        DataId::new(0x0400_0002),
        &parent,
        false,
        Some((0, &parent, 7)),
    );
    let skeleton = ObjectSkeleton {
        groups: vec![root, child],
    };
    let poses = skeleton.pose(None);
    // Parent bone 1 sits at (0, 1, 1) unscaled; the connection point adds 0.5 up; the child's
    // own root bone adds (0, 0, 1).
    let origin = poses[1].part_matrix(0).transform_point3(Vec3::ZERO);
    assert!(
        origin.abs_diff_eq(Vec3::new(0.0, 1.0, 2.5), 1e-5),
        "{origin:?}"
    );
}

/// `CPartGroup::GetHoldingLocation` on a child group returns that group's `m_boneMatrices` folded
/// with its `m_mFinal`: a child bone's model-space position, the same as its part's origin.
#[test]
fn a_child_groups_holding_bone_is_its_model_space_bone() {
    let frame = [
        transform([0.0, 0.0, 1.0], 1.0, Quat::IDENTITY),
        transform([0.0, 1.0, 0.0], 3.0, Quat::IDENTITY),
    ];
    let parent = setup(frame, frame);
    let root = SkeletonGroup::new(DataId::new(0x0400_0001), &parent, false, None);
    let child = SkeletonGroup::new(
        DataId::new(0x0400_0002),
        &parent,
        false,
        Some((0, &parent, 7)),
    );
    let skeleton = ObjectSkeleton {
        groups: vec![root, child],
    };
    let poses = skeleton.pose(None);
    let root_bones = skeleton.root_bones(None);
    let bones = skeleton.group_bones(&poses, &root_bones);
    assert_eq!(bones.len(), 2);
    assert!(bones[0][1].offset.abs_diff_eq(root_bones[1].offset, 1e-5));
    assert!(
        bones[1][0]
            .offset
            .abs_diff_eq(Vec3::new(0.0, 1.0, 2.5), 1e-5),
        "{:?}",
        bones[1][0]
    );
}

#[test]
fn blend_sums_weighted_bones_without_renormalising() {
    let matrices = [
        Mat4::from_translation(Vec3::X),
        Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2),
    ];
    let (p, n) = blend_vertex(
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        &[(0, 0.25), (1, 0.25), (0, 0.0), (1, 0.0)],
        &matrices,
    );
    // 0.25 * (2, 0, 0) + 0.25 * (0, 1, 0); the weights sum to a half and stay that way.
    assert!(
        Vec3::from(p).abs_diff_eq(Vec3::new(0.5, 0.25, 0.0), 1e-6),
        "{p:?}"
    );
    assert!(
        Vec3::from(n).abs_diff_eq(Vec3::new(0.25, 0.25, 0.0), 1e-6),
        "{n:?}"
    );
}

#[test]
fn human_visual_poses_its_skinned_body_from_the_rest_frame() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let human = DataId::new(0x1F00_0023);
    let settings = super::RenderSettings::default();
    let model = super::load_object_model(&store, human, &settings)
        .unwrap()
        .expect("human draws");
    let skeleton = model.skeleton.clone().unwrap();
    assert!(skeleton.groups[0].skinned(), "the body group is skinned");
    eprintln!(
        "human: {} groups, {} parts, {} skinned parts",
        skeleton.groups.len(),
        model.parts.len(),
        model.parts.iter().filter(|p| p.skin.is_some()).count()
    );
    let rest = model.pose(None).unwrap();
    assert_eq!(rest.parts.len(), model.parts.len());
    // Placement frame 0 is the rest frame: every deform matrix is (close to) the identity.
    let worst = rest
        .palette
        .iter()
        .map(|m| (*m - Mat4::IDENTITY).to_cols_array().map(f32::abs))
        .map(|m| m.into_iter().fold(0.0_f32, f32::max))
        .fold(0.0_f32, f32::max);
    assert!(worst < 1e-3, "rest deform off identity by {worst}");

    // The stance clip's first frame through the retail matrices bends the T-pose arms down.
    let animation =
        ac2_anim::load_ac2_motion_animation(&store, DataId::new(0x0300_0014), 0x8000_0012)
            .unwrap()
            .expect("stance clip");
    let setup = store
        .load_cached_packed::<Ac2Setup>(ac2_db::TypedResourceRef::new(
            ac2_dat::DatFileKind::Portal,
            skeleton.groups[0].setup,
            0,
        ))
        .unwrap()
        .unwrap();
    let locals = animation.sample(0.0, &setup.rest_locals());
    let posed = model.pose(Some(&locals)).unwrap();
    let span = |pose: &super::ObjectPose| {
        let (mut min, mut max) = (f32::MAX, f32::MIN);
        for (part, matrix) in model.parts.iter().zip(&pose.parts) {
            let index = model
                .parts
                .iter()
                .position(|p| std::ptr::eq(p, part))
                .unwrap();
            for (i, vertex) in part.vertices.iter().enumerate() {
                let local = match (&part.skin, pose.skins[index]) {
                    (Some(skin), Some(start)) => {
                        blend_vertex(
                            vertex.position,
                            vertex.normal,
                            &skin[i],
                            &pose.palette[start as usize..],
                        )
                        .0
                    }
                    _ => vertex.position,
                };
                let x = matrix.transform_point3(Vec3::from(local)).x;
                min = min.min(x);
                max = max.max(x);
            }
        }
        max - min
    };
    let (bind, stance) = (span(&rest), span(&posed));
    eprintln!("bind span {bind:.2} m, stance span {stance:.2} m");
    assert!(stance < bind * 0.9, "arms leave the bind T-pose");
}

/// A child group with its own animation (`CPartGroup::DoAnimation` over the children) poses at its
/// own bones, found by key; the root and unlisted groups keep their placement, and a wrong bone
/// count is ignored.
#[test]
fn a_child_group_poses_at_its_own_animated_bones() {
    let frame = [Ac2Transform::IDENTITY; 2];
    let root_setup = setup(frame, frame);
    let child_setup = setup(frame, frame);
    let mut child = SkeletonGroup::new(
        DataId::new(0x0400_0002),
        &child_setup,
        false,
        Some((0, &root_setup, 7)),
    );
    child.key = 0x8100_0009;
    let skeleton = ObjectSkeleton {
        groups: vec![
            SkeletonGroup::new(DataId::new(0x0400_0001), &root_setup, false, None),
            child,
        ],
    };
    let at = |x: f32| {
        vec![
            Ac2Transform::IDENTITY,
            transform([x, 0.0, 0.0], 1.0, Quat::IDENTITY),
        ]
    };
    let rest = skeleton.pose_groups(None, &[], 0);
    let moved = skeleton.pose_groups(None, &[(0x8100_0009, at(3.0))], 0);
    assert!(close(moved[0].part_matrix(1), rest[0].part_matrix(1)));
    assert!(
        !close(moved[1].part_matrix(1), rest[1].part_matrix(1)),
        "the child's second part follows its animated bone"
    );
    let other_key = skeleton.pose_groups(None, &[(1, at(3.0))], 0);
    assert!(close(other_key[1].part_matrix(1), rest[1].part_matrix(1)));
    let short = skeleton.pose_groups(None, &[(0x8100_0009, vec![Ac2Transform::IDENTITY])], 0);
    assert!(close(short[1].part_matrix(1), rest[1].part_matrix(1)));
}
