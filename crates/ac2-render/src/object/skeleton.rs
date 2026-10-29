//! A visual's `CPartGroup` tree and its bone matrices, built the retail way.
//!
//! `CPartArray::UpdateBones` fills two matrices per bone from `m_boneTransforms`: `m_boneMatrices`
//! from `TransformBase::FillMatrix(t, 0)` (no scale) and `m_deformMatrices` from
//! `FillMatrix(t, 1)`, each multiplied into its parent's. When a part carries matrix weights
//! (`CacheHasMatrixWeights`, state `0x40000000`) every deform matrix is then pre-multiplied by
//! `CharacterSetup::rest_matrix` (`SetRestBones`: the inverse of placement frame 1 with scale).
//! `CPartGroup::RefreshMatrices` hangs a child group from its parent's (unscaled) bone matrix
//! through `m_parentOffsetMatrix` (`SetParentOffset`), and `CPartArray::RefreshMatrices` gives
//! part `i` its bone matrix times the group matrix, or the group matrix alone in a skinned group,
//! whose vertices `RenderVertexBuffer::BlendVertices` blends by the deform matrices instead.
//!
//! Matrices are kept as retail stores them: row-major `float[16]` for row vectors. A row-vector
//! matrix read as column-major is its column-vector transpose, so `Mat4::from_cols_array` turns
//! one into the `glam` matrix that maps points the same way.

use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_core::DataId;
use ac2_math::Mat4;

/// A retail `Matrix4`: row-major, row vectors.
pub(crate) type Matrix4 = [f32; 16];

const IDENTITY: Matrix4 = [
    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
];

/// The placement frame `CPartArray::m_placementID` starts at, and the one `InitRestBones` takes
/// as the rest pose.
const DEFAULT_PLACEMENT: u32 = 0;
const REST_PLACEMENT: u32 = 1;

/// `Matrix4::Inverse` leaves the adjoint unscaled below this determinant.
const MIN_DETERMINANT: f64 = 0.0002;

fn ld(value: f32) -> f64 {
    f64::from(value)
}

fn st(value: f64) -> f32 {
    value as f32
}

/// The column-vector `glam` matrix of a retail row-vector matrix.
pub(crate) fn to_mat4(m: &Matrix4) -> Mat4 {
    Mat4::from_cols_array(m)
}

/// `D3DXMatrixMultiply(out, a, b)`: `a` applied first, each element summed on the x87 stack and
/// stored as a `float`.
pub(crate) fn multiply(a: &Matrix4, b: &Matrix4) -> Matrix4 {
    let mut out = [0.0; 16];
    for row in 0..4 {
        for col in 0..4 {
            let r = row * 4;
            out[r + col] = st(((ld(a[r]) * ld(b[col]) + ld(a[r + 1]) * ld(b[4 + col]))
                + ld(a[r + 2]) * ld(b[8 + col]))
                + ld(a[r + 3]) * ld(b[12 + col]));
        }
    }
    out
}

/// `Quaternion::InqRotationAxes` (right, forward, up), with the spills `Frame::cache_matrix`
/// shows: the doubled `z` and the `w`/`x` cross terms are stored as `float`, `yy`, `yz` and `zz`
/// stay on the x87 stack. The quaternion is not renormalised.
fn rotation_axes(transform: &Ac2Transform) -> [[f32; 3]; 3] {
    let q = transform.rotation;
    let (w, x, y, z) = (ld(q.w), ld(q.x), ld(q.y), ld(q.z));
    let x2 = x + x;
    let y2 = y + y;
    let z2 = ld(st(z + z));
    let wx = ld(st(x2 * w));
    let wy = ld(st(y2 * w));
    let wz = ld(st(z2 * w));
    let xx = ld(st(x2 * x));
    let xy = ld(st(y2 * x));
    let xz = ld(st(z2 * x));
    let yy = y2 * y;
    let yz = z2 * y;
    let zz = z2 * z;
    [
        [st(1.0 - (zz + yy)), st(xy + wz), st(xz - wy)],
        [st(xy - wz), st(1.0 - (zz + xx)), st(wx + yz)],
        [st(xz + wy), st(yz - wx), st(1.0 - (yy + xx))],
    ]
}

/// `TransformBase::FillMatrix`: `Matrix4::RotationFromQuaternion` (`RotateBasis` of the rotation
/// axes), each axis row times its scale when `include_scale`, and the offset as the last row.
pub(crate) fn fill_matrix(transform: &Ac2Transform, include_scale: bool) -> Matrix4 {
    let axes = rotation_axes(transform);
    let scale = transform.scale.to_array();
    let mut m = IDENTITY;
    for (row, axis) in axes.iter().enumerate() {
        let factor = if include_scale { scale[row] } else { 1.0 };
        for (col, value) in axis.iter().enumerate() {
            m[row * 4 + col] = if include_scale {
                st(ld(*value) * ld(factor))
            } else {
                *value
            };
        }
    }
    m[12..15].copy_from_slice(&transform.offset.to_array());
    m
}

/// `Matrix4::Adjoint`.
fn adjoint(src: &Matrix4) -> Matrix4 {
    let m = src.map(ld);
    let [a1, a2, a3, a4, b1, b2, b3, b4, c1, c2, c3, c4, d1, d2, d3, d4] = m;
    let mut out = [0.0; 16];
    let (e17, e18, e19) = (d4 * c3 - d3 * c4, d4 * c2 - d2 * c4, d3 * c2 - d2 * c3);
    out[0] = st(e19 * b4 + (e17 * b2 - e18 * b3));
    let (e20, e21) = (d4 * c1 - d1 * c4, d3 * c1 - d1 * c3);
    out[4] = st(-(e21 * b4 + (e17 * b1 - e20 * b3)));
    let e22 = d2 * c1 - d1 * c2;
    out[8] = st(e22 * b4 + (e18 * b1 - e20 * b2));
    out[12] = st(-(e22 * b3 + (e19 * b1 - e21 * b2)));
    out[1] = st(-(e19 * a4 + (e17 * a2 - e18 * a3)));
    out[5] = st(e21 * a4 + (e17 * a1 - e20 * a3));
    out[9] = st(-(e22 * a4 + (e18 * a1 - e20 * a2)));
    out[13] = st(e22 * a3 + (e19 * a1 - e21 * a2));
    let (f19, f18, f17) = (d4 * b3 - d3 * b4, d4 * b2 - d2 * b4, d3 * b2 - d2 * b3);
    out[2] = st(f17 * a4 + (f19 * a2 - f18 * a3));
    let (f16, f15) = (d4 * b1 - d1 * b4, d3 * b1 - d1 * b3);
    out[6] = st(-(f15 * a4 + (f19 * a1 - f16 * a3)));
    let f13 = d2 * b1 - d1 * b2;
    out[10] = st(f13 * a4 + (f18 * a1 - f16 * a2));
    out[14] = st(-(f13 * a3 + (f17 * a1 - f15 * a2)));
    let (g15, g14, g13) = (c4 * b3 - c3 * b4, c4 * b2 - c2 * b4, c3 * b2 - c2 * b3);
    out[3] = st(-(g13 * a4 + (g15 * a2 - g14 * a3)));
    let (g8, g7) = (c4 * b1 - c1 * b4, c3 * b1 - c1 * b3);
    out[7] = st(g7 * a4 + (g15 * a1 - g8 * a3));
    let g5 = c2 * b1 - c1 * b2;
    out[11] = st(-(g5 * a4 + (g14 * a1 - g8 * a2)));
    out[15] = st(g5 * a3 + (g13 * a1 - g7 * a2));
    out
}

/// `Matrix4::CalcDeterminant`, stored as a `float`.
fn determinant(m: &Matrix4) -> f32 {
    let m = m.map(ld);
    let [m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15] = m;
    let a = m8 * m13 - m9 * m12;
    let b = m9 * m15 - m13 * m11;
    let c = m8 * m15 - m12 * m11;
    let d = m9 * m14 - m13 * m10;
    let e = m10 * m15 - m14 * m11;
    let f = m8 * m14 - m10 * m12;
    st(((a * m7 + (b * m4 - c * m5)) * m2
        + ((d * m7 + (e * m5 - b * m6)) * m0 - (f * m7 + (e * m4 - c * m6)) * m1))
        - (a * m6 + (d * m4 - f * m5)) * m3)
}

/// `Matrix4::Inverse`: the adjoint over the determinant, or the bare adjoint when the
/// determinant's magnitude is under 0.0002.
fn inverse(src: &Matrix4) -> Matrix4 {
    let adjoint = adjoint(src);
    let det = determinant(src);
    if ld(det).abs() < MIN_DETERMINANT {
        return adjoint;
    }
    let scale = ld(st(1.0 / ld(det)));
    adjoint.map(|value| st(scale * ld(value)))
}

/// `CharacterSetup::GetPlacementFrame(id)`: the bone transforms of that placement frame.
fn placement(setup: &Ac2Setup, id: u32) -> Option<&[Ac2Transform]> {
    setup
        .placements
        .iter()
        .find(|frame| frame.kind == id)
        .map(|frame| &frame.bones[..])
}

/// `CharacterSetup::SetRestBones` over placement frame 1 (`InitRestBones`): each bone's scaled
/// matrix multiplied into its parent's, inverted. `None` when the Setup has no rest frame or its
/// bone count differs.
fn rest_matrices(setup: &Ac2Setup) -> Option<Vec<Matrix4>> {
    let rest = placement(setup, REST_PLACEMENT)?;
    if rest.len() != setup.bones.len() {
        return None;
    }
    let mut world: Vec<Matrix4> = Vec::with_capacity(rest.len());
    for (index, bone) in setup.bones.iter().enumerate() {
        let local = fill_matrix(&rest[index], true);
        world.push(match bone.parent.filter(|parent| *parent < index) {
            Some(parent) => multiply(&local, &world[parent]),
            None => local,
        });
    }
    Some(world.iter().map(inverse).collect())
}

/// One `CPartGroup` of a visual.
#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonGroup {
    /// The drawn group's key (`CPartGroup::m_key`; 0 until the model sets it).
    pub key: u32,
    pub setup: DataId,
    /// Mesh count of the Setup (`m_numParts`).
    pub parts: usize,
    parents: Vec<Option<usize>>,
    /// Each bone's name in this group's Setup (`CharacterSetup::FindBoneType`'s bone types).
    bone_names: Vec<u32>,
    /// `m_boneTransforms` after `SetPlacementFrameID(0)`.
    placement: Vec<Ac2Transform>,
    /// The other placement frames (`CharacterSetup::GetPlacementFrame(id)`), by id, each padded to
    /// the group's bone count.
    frames: Vec<(u32, Vec<Ac2Transform>)>,
    /// `CharacterSetup::rest_matrix` when a part carries matrix weights; `None` for a rigid group.
    rest: Option<Vec<Matrix4>>,
    /// Parent group, the parent bone (`m_parentBoneIndex`, `None` past the parent's bones) and
    /// `m_parentOffsetMatrix` (`FillMatrix` of the connection point with scale).
    attachment: Option<(usize, Option<usize>, Matrix4)>,
}

impl SkeletonGroup {
    /// The group for `setup`, skinned when `weighted` (`CacheHasMatrixWeights`), hung from
    /// `parent` (its group and that group's Setup) at connection point `connection`.
    pub fn new(
        setup_did: DataId,
        setup: &Ac2Setup,
        weighted: bool,
        parent: Option<(usize, &Ac2Setup, u32)>,
    ) -> Self {
        let bones = setup.bones.len();
        let placement = (0..bones)
            .map(|index| {
                placement(setup, DEFAULT_PLACEMENT)
                    .and_then(|frame| frame.get(index))
                    .copied()
                    .unwrap_or(Ac2Transform::IDENTITY)
            })
            .collect();
        let frames = setup
            .placements
            .iter()
            .filter(|frame| frame.kind != DEFAULT_PLACEMENT)
            .map(|frame| {
                let bones = (0..bones)
                    .map(|index| {
                        frame
                            .bones
                            .get(index)
                            .copied()
                            .unwrap_or(Ac2Transform::IDENTITY)
                    })
                    .collect();
                (frame.kind, bones)
            })
            .collect();
        let attachment = parent.map(|(group, parent_setup, connection)| {
            // `CPartGroup::SetParentOffset` takes the parent Setup's connection point only when
            // its bone exists; otherwise the child keeps an identity offset.
            match parent_setup
                .connection_point(connection)
                .filter(|point| point.bone < parent_setup.bones.len())
            {
                Some(point) => (group, Some(point.bone), fill_matrix(&point.transform, true)),
                None => (group, None, IDENTITY),
            }
        });
        Self {
            key: 0,
            setup: setup_did,
            parts: setup.meshes.len(),
            parents: setup.bones.iter().map(|bone| bone.parent).collect(),
            bone_names: setup.bones.iter().map(|bone| bone.name).collect(),
            placement,
            frames,
            rest: weighted.then(|| rest_matrices(setup)).flatten(),
            attachment,
        }
    }

    /// `CPartGroup::SetPlacementFrameID(id)` on this group: the bones of placement frame `id`, or of
    /// frame 0 when the Setup has no frame `id` (`CPartArray::SetPlacementFrameID` falls back to 0).
    fn placed(&self, id: u32) -> &[Ac2Transform] {
        self.frames
            .iter()
            .find(|(kind, _)| *kind == id)
            .map_or(&self.placement[..], |(_, bones)| &bones[..])
    }

    /// Whether the group's parts blend by deform matrices (`m_state & 0x40000000`).
    pub fn skinned(&self) -> bool {
        self.rest.is_some()
    }

    pub fn bone_count(&self) -> usize {
        self.parents.len()
    }

    /// `CPartArray::UpdateBones` over `locals` (`m_boneTransforms`).
    fn update_bones(&self, locals: &[Ac2Transform]) -> GroupPose {
        let count = self.parents.len();
        let mut bones: Vec<Matrix4> = Vec::with_capacity(count);
        let mut deform: Vec<Matrix4> = Vec::with_capacity(count);
        for index in 0..count {
            let local = locals.get(index).unwrap_or(&Ac2Transform::IDENTITY);
            let (bone, scaled) = (fill_matrix(local, false), fill_matrix(local, true));
            match self.parents[index].filter(|parent| *parent < index) {
                None => {
                    bones.push(bone);
                    deform.push(scaled);
                }
                Some(parent) => {
                    bones.push(multiply(&bone, &bones[parent]));
                    deform.push(if self.rest.is_some() {
                        multiply(&scaled, &deform[parent])
                    } else {
                        scaled
                    });
                }
            }
        }
        let deform = self.rest.as_ref().map(|rest| {
            deform
                .iter()
                .zip(rest)
                .map(|(matrix, rest)| multiply(rest, matrix))
                .collect()
        });
        GroupPose {
            bones,
            deform,
            matrix: IDENTITY,
        }
    }
}

/// The posed matrices of one group.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupPose {
    /// `m_boneMatrices` (no scale).
    bones: Vec<Matrix4>,
    /// `m_deformMatrices` with the rest matrices applied, for a skinned group.
    deform: Option<Vec<Matrix4>>,
    /// The group's `mFinal` in `CPartGroup::RefreshMatrices`.
    matrix: Matrix4,
}

impl GroupPose {
    /// `CPartArray::RefreshMatrices`: part `index`'s `CPhysicsPart::m_mObject` (bone matrix times
    /// group matrix; the group matrix alone in a skinned group or past the bones).
    pub fn part_matrix(&self, index: usize) -> Mat4 {
        match (&self.deform, self.bones.get(index)) {
            (None, Some(bone)) => to_mat4(&multiply(bone, &self.matrix)),
            _ => to_mat4(&self.matrix),
        }
    }

    /// The deform matrices `CPartArray::Draw` hands to `SceneTool::SetMatrixStack`.
    pub fn blend_matrices(&self) -> Option<Vec<Mat4>> {
        self.deform
            .as_ref()
            .map(|deform| deform.iter().map(to_mat4).collect())
    }
}

/// A visual's part-group tree, parents before children.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjectSkeleton {
    pub groups: Vec<SkeletonGroup>,
}

impl ObjectSkeleton {
    /// `CPartGroup::FindBoneType` over the tree: every group's bones by name, in retail's search
    /// order (a group's own bones, then each child group's search in turn). The first entry for a
    /// name is the bone retail finds. A root bone is its `root_bones` transform; a child group's
    /// bone is its bone matrix times the group matrix (`CPartArray::m_boneMatrices` times
    /// `CPartGroup::m_mFinal`), read back as a transform. `poses` and `root` are as `pose` takes them.
    pub fn named_bones(
        &self,
        poses: &[GroupPose],
        root_bones: &[Ac2Transform],
    ) -> Vec<(u32, Ac2Transform)> {
        let mut found = Vec::new();
        if !self.groups.is_empty() {
            self.find_bones(0, poses, root_bones, &mut found);
        }
        found
    }

    fn find_bones(
        &self,
        group: usize,
        poses: &[GroupPose],
        root_bones: &[Ac2Transform],
        found: &mut Vec<(u32, Ac2Transform)>,
    ) {
        let Some(pose) = poses.get(group) else {
            return;
        };
        for (index, name) in self.groups[group].bone_names.iter().enumerate() {
            let transform = if group == 0 {
                root_bones.get(index).copied()
            } else {
                pose.bones
                    .get(index)
                    .map(|bone| transform_of(&multiply(bone, &pose.matrix)))
            };
            if let Some(transform) = transform {
                found.push((*name, transform));
            }
        }
        for (child, other) in self.groups.iter().enumerate() {
            if other
                .attachment
                .is_some_and(|(parent, _, _)| parent == group)
                && child > group
            {
                self.find_bones(child, poses, root_bones, found);
            }
        }
    }

    /// Every group's model-space bones, the transforms `CPartGroup::GetHoldingLocation` hands back
    /// for a holding location (`CPartArray::m_boneMatrices` of the group that lists it). Group 0
    /// takes `root_bones`; a child group's bone is its bone matrix times `CPartGroup::m_mFinal`,
    /// read back as a transform, as `named_bones` reads it.
    pub fn group_bones(
        &self,
        poses: &[GroupPose],
        root_bones: &[Ac2Transform],
    ) -> Vec<Vec<Ac2Transform>> {
        (0..self.groups.len())
            .map(|index| {
                if index == 0 {
                    return root_bones.to_vec();
                }
                poses.get(index).map_or_else(Vec::new, |pose| {
                    pose.bones
                        .iter()
                        .map(|bone| transform_of(&multiply(bone, &pose.matrix)))
                        .collect()
                })
            })
            .collect()
    }

    /// The root group's model-space bones (`CPartArray::m_boneMatrices`, which carry no scale), as
    /// the transforms `pose` builds them from: each bone's parent-relative local composed up its
    /// chain. `root` is taken as `pose` takes it.
    /// The root group's bone parents (`CSetup`'s), for its skeleton points.
    pub fn root_parents(&self) -> std::sync::Arc<[Option<usize>]> {
        self.groups.first().map_or_else(
            || std::sync::Arc::from(Vec::new()),
            |group| std::sync::Arc::from(group.parents.clone()),
        )
    }

    pub fn root_bones(&self, root: Option<&[Ac2Transform]>) -> Vec<Ac2Transform> {
        self.root_bones_placed(root, DEFAULT_PLACEMENT)
    }

    /// [`Self::root_bones`] with the root group at placement frame `placement`.
    pub fn root_bones_placed(
        &self,
        root: Option<&[Ac2Transform]>,
        placement: u32,
    ) -> Vec<Ac2Transform> {
        let Some(group) = self.groups.first() else {
            return Vec::new();
        };
        let locals = match root {
            Some(root) if root.len() == group.bone_count() => root,
            _ => group.placed(placement),
        };
        let mut model: Vec<Ac2Transform> = Vec::with_capacity(locals.len());
        for (index, local) in locals.iter().enumerate() {
            let local = Ac2Transform {
                scale: ac2_math::Vec3::ONE,
                ..*local
            };
            let bone = match group.parents[index].filter(|parent| *parent < index) {
                Some(parent) => model[parent].compose(&local),
                None => local,
            };
            model.push(bone);
        }
        model
    }

    /// Every group's matrices with the root group's bones at `root` (parent-relative bone
    /// transforms, as the animation leaves `m_boneTransforms`) and the rest at placement frame 0;
    /// `root` of the wrong length is ignored. `CPartGroup::UpdateBones` then `RefreshMatrices`
    /// from the identity (`CheckRefreshMatrices`).
    pub fn pose(&self, root: Option<&[Ac2Transform]>) -> Vec<GroupPose> {
        self.pose_placed(root, DEFAULT_PLACEMENT)
    }

    /// [`Self::pose`] with every group at placement frame `placement` (`CPartGroup::SetPlacementFrameID`
    /// over the whole tree) where the root has no animation.
    pub fn pose_placed(&self, root: Option<&[Ac2Transform]>, placement: u32) -> Vec<GroupPose> {
        let mut poses: Vec<GroupPose> = Vec::with_capacity(self.groups.len());
        for (index, group) in self.groups.iter().enumerate() {
            let locals = match root {
                Some(root) if index == 0 && root.len() == group.bone_count() => root,
                _ => group.placed(placement),
            };
            let mut pose = group.update_bones(locals);
            pose.matrix = match group.attachment {
                Some((parent, bone, offset)) if parent < poses.len() => {
                    let parent = &poses[parent];
                    let local = match bone.and_then(|bone| parent.bones.get(bone)) {
                        Some(bone) => multiply(&offset, bone),
                        None => offset,
                    };
                    multiply(&local, &parent.matrix)
                }
                _ => IDENTITY,
            };
            poses.push(pose);
        }
        poses
    }
}

/// A row-vector matrix read back as the transform it was filled from (rotation, offset, scale).
fn transform_of(matrix: &Matrix4) -> Ac2Transform {
    let (scale, rotation, offset) = to_mat4(matrix).to_scale_rotation_translation();
    Ac2Transform {
        offset,
        rotation,
        scale,
    }
}

/// `BlendLitVertices`: a skinned vertex's position and normal under `matrices` (column-vector
/// deform matrices). Each influence with a non-zero weight adds its transformed point divided by
/// the homogeneous `w` and scaled by the weight; the normal takes the rotation rows with `w`
/// offset by one. Neither the weights nor the normal are renormalised.
pub fn blend_vertex(
    position: [f32; 3],
    normal: [f32; 3],
    influences: &[(u8, f32); 4],
    matrices: &[Mat4],
) -> ([f32; 3], [f32; 3]) {
    let (mut p, mut n) = ([0.0_f32; 3], [0.0_f32; 3]);
    let point = ac2_math::Vec4::new(position[0], position[1], position[2], 1.0);
    let direction = ac2_math::Vec4::new(normal[0], normal[1], normal[2], 0.0);
    for (bone, weight) in influences {
        if *weight == 0.0 {
            continue;
        }
        let Some(matrix) = matrices.get(usize::from(*bone)) else {
            continue;
        };
        let h = *matrix * point;
        let r = *matrix * direction;
        let (pw, nw) = (weight / h.w, weight / (r.w + 1.0));
        for axis in 0..3 {
            p[axis] += h[axis] * pw;
            n[axis] += r[axis] * nw;
        }
    }
    (p, n)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ac2_anim::{Ac2Bone, Ac2PlacementFrame};
    use ac2_math::{Quat, Vec3};

    use super::*;

    fn bone(parent: Option<usize>) -> Ac2Bone {
        Ac2Bone { parent, name: 1 }
    }

    /// `CPartArray::UpdateBones`: a child bone's model-space origin is its parent's rotated
    /// offset plus its own, and the bone matrices and `root_bones` agree on it.
    #[test]
    fn root_bones_compose_the_chain_the_matrices_pose() {
        let setup = Ac2Setup {
            meshes: Arc::from(Vec::new()),
            bones: Arc::from(vec![bone(None), bone(Some(0))]),
            placements: Arc::from(Vec::new()),
            holding_points: Arc::from(Vec::new()),
            connection_points: Arc::from(Vec::new()),
            degrade: Default::default(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        };
        let skeleton = ObjectSkeleton {
            groups: vec![SkeletonGroup::new(DataId::new(1), &setup, false, None)],
        };
        let root = [
            Ac2Transform {
                offset: Vec3::new(1.0, 2.0, 3.0),
                scale: Vec3::ONE,
                rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            },
            Ac2Transform {
                offset: Vec3::new(0.0, 4.0, 0.0),
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            },
        ];
        let bones = skeleton.root_bones(Some(&root));
        let posed = skeleton.pose(Some(&root));
        let matrices = &posed[0].bones;
        for (index, bone) in bones.iter().enumerate() {
            let m = matrices[index];
            let origin = Vec3::new(m[12], m[13], m[14]);
            assert!(
                origin.distance(bone.offset) < 1e-4,
                "bone {index}: matrix origin {origin:?} vs chain {:?}",
                bone.offset
            );
        }
        // The child sits at the root's rotated (0, 4, 0): (-4, 0, 0) after a quarter turn about z.
        assert!(bones[1].offset.distance(Vec3::new(-3.0, 2.0, 3.0)) < 1e-4);
    }

    fn frame(kind: u32, x: f32) -> Ac2PlacementFrame {
        Ac2PlacementFrame {
            kind,
            bones: Arc::from(vec![Ac2Transform {
                offset: Vec3::new(x, 0.0, 0.0),
                scale: Vec3::ONE,
                rotation: Quat::IDENTITY,
            }]),
        }
    }

    /// `CPartArray::SetPlacementFrameID`: a held object's placement frame poses the rest bones
    /// (no animation), a frame the Setup lacks falls back to frame 0.
    #[test]
    fn a_placement_frame_poses_the_rest_bones_and_a_missing_one_falls_back_to_frame_0() {
        let setup = Ac2Setup {
            meshes: Arc::from(Vec::new()),
            bones: Arc::from(vec![bone(None)]),
            placements: Arc::from(vec![frame(0, 1.0), frame(5, 7.0)]),
            holding_points: Arc::from(Vec::new()),
            connection_points: Arc::from(Vec::new()),
            degrade: Default::default(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        };
        let skeleton = ObjectSkeleton {
            groups: vec![SkeletonGroup::new(DataId::new(1), &setup, false, None)],
        };
        let x = |id: u32| skeleton.root_bones_placed(None, id)[0].offset.x;
        assert_eq!(x(0), 1.0);
        assert_eq!(x(5), 7.0);
        assert_eq!(x(9), 1.0, "a frame the Setup lacks is frame 0");
        let posed = skeleton.pose_placed(None, 5);
        assert!((posed[0].bones[0][12] - 7.0).abs() < 1e-5);
    }
}
