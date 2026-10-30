//! AC1 animation on an AC2 skeleton. AC1 poses rigid parts in object space (`AnimFrame`); AC2
//! poses a bone tree, each bone relative to its parent. A table pairs AC2 bones (by their bone
//! names) with AC1 parts, and each pair gets a constant correction so the AC1 part's rotation
//! turns the AC2 bone:
//!
//! - the AC2 rest (placement 0, a T-pose for AC2's humans) is first brought to the AC1 reference
//!   pose: each paired bone is swung, in object space, so that it points from its joint to its
//!   child's joint the way the AC1 part points at its child part (a bone without a child takes
//!   its parent's swing; the chest and head take none: they stand upright in both rests);
//! - the correction is the AC1 reference part's rotation undone, then that aligned rest:
//!   `world(bone) = ac1_part(t) * correction`.
//!
//! Every frame the paired bones take those object-space rotations; the AC2 bones between paired
//! ones (spine segments, clavicles, twist bones) hold their rest, so no motion is applied twice;
//! the bones below no pair (fingers, cloth) keep the AC2 animation. The root also moves as the
//! AC1 root part does, scaled from the AC1 hip height to the AC2 one. Bone lengths stay AC2's.

use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_math::{Quat, Vec3};

/// An AC2 bone (its bone name) driven by an AC1 part, and the joints each points at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BonePair {
    pub ac2: u32,
    pub ac1: usize,
    /// The AC2 bone and AC1 part whose joints this one points at; `None` for an end.
    pub ac2_child: Option<u32>,
    pub ac1_child: Option<usize>,
}

const fn pair(ac2: u32, ac1: usize, ac2_child: u32, ac1_child: usize) -> BonePair {
    BonePair {
        ac2,
        ac1,
        ac2_child: if ac2_child == 0 {
            None
        } else {
            Some(ac2_child)
        },
        ac1_child: if ac2_child == 0 {
            None
        } else {
            Some(ac1_child)
        },
    }
}

/// The AC1 human's parts (`0x02000001`: 0 abdomen, 1-4 left leg to toe, 5-8 right leg, 9 chest,
/// 10-12 left arm to hand, 13-15 right arm, 16 head) on the AC2 human skeleton's bones. Sides
/// are matched by where they stand (AC1 part 1 and AC2 bone `0x8100000D` both at -x).
pub const HUMAN: &[BonePair] = &[
    pair(0x8100_0005, 0, 0, 0),
    pair(0x8100_000D, 1, 0x8100_000F, 2),
    pair(0x8100_000F, 2, 0x8100_0011, 3),
    pair(0x8100_0011, 3, 0x8100_0013, 4),
    pair(0x8100_0013, 4, 0, 0),
    pair(0x8100_000C, 5, 0x8100_000E, 6),
    pair(0x8100_000E, 6, 0x8100_0010, 7),
    pair(0x8100_0010, 7, 0x8100_0012, 8),
    pair(0x8100_0012, 8, 0, 0),
    // The chest and head are upright in both rest poses: not swung (a race whose neck juts
    // forward, the Tumerok's, would otherwise throw its head back).
    pair(0x8100_000B, 9, 0, 0),
    pair(0x4000_0005, 16, 0, 0),
    pair(0x8100_0017, 10, 0x8100_0019, 11),
    pair(0x8100_0019, 11, 0x8100_001D, 12),
    pair(0x8100_001D, 12, 0, 0),
    pair(0x8100_0016, 13, 0x8100_0018, 14),
    pair(0x8100_0018, 14, 0x8100_001C, 15),
    pair(0x8100_001C, 15, 0, 0),
];

/// How one AC2 bone is posed.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Drive {
    /// By AC1 part `part`: `world = part * correction`.
    Part { part: usize, correction: Quat },
    /// Held at its rest (it lies between driven bones).
    Rest,
    /// By the AC2 animation.
    Animation,
}

/// A retarget of one AC1 body onto one AC2 skeleton.
#[derive(Debug, Clone, PartialEq)]
pub struct Retarget {
    parents: Vec<Option<usize>>,
    rest: Vec<Ac2Transform>,
    drive: Vec<Drive>,
    /// Where each paired bone's AC1 part has its origin, in the bone's frame: for a limb part as
    /// far past the AC2 joint as it is past AC1's joint, scaled by the two segments (AC1's
    /// forearm origin is mid-forearm, not at the elbow), and for a trunk part (root, chest, head: no limb above it) its AC1 height over the
    /// AC1 root, scaled, over the AC2 root (AC2's chest bone sits higher up the spine than
    /// AC1's chest part, whose origin is the waist).
    anchors: Vec<Vec3>,
    /// Each paired bone's AC1 part's scale in its own frame: a trunk part's by AC2's spine over
    /// AC1's (hips to head); a limb part's along its segment by the AC2 segment over AC1's (joint
    /// to joint) and across by the whole limb's ratio; else AC2 over AC1 hip height.
    stretches: Vec<Vec3>,
    /// A paired bone's AC1 part's turn in the bone's frame where it is not `correction⁻¹`: a
    /// trunk part leaned with the AC2 spine.
    turns: Vec<Option<Quat>>,
    /// Each paired bone's AC1 part's length axis in its own frame (towards its child joint, a
    /// trunk part's towards the trunk part above it); `None` for a part with neither.
    axes: Vec<Option<Vec3>>,
    /// The AC1 root part's reference place, and AC2 over AC1 hip height.
    ac1_root: Vec3,
    height: f32,
}

/// The pairs' limbs, each from its first part (one no pair points at) through each part's
/// child to its end (a part without one): shoulder to hand, hip to toe.
fn limb_chains(pairs: &[BonePair]) -> Vec<Vec<BonePair>> {
    let by_part = |part: usize| pairs.iter().find(|p| p.ac1 == part);
    pairs
        .iter()
        .filter(|p| p.ac1_child.is_some() && !pairs.iter().any(|q| q.ac1_child == Some(p.ac1)))
        .map(|start| {
            let mut chain = vec![*start];
            while let Some(next) = chain
                .last()
                .and_then(|p| p.ac1_child)
                .and_then(by_part)
                .filter(|next| chain.len() < pairs.len() && !chain.contains(next))
            {
                chain.push(*next);
            }
            chain
        })
        .collect()
}

/// The shortest turn taking direction `from` to direction `to`.
fn arc(from: Vec3, to: Vec3) -> Quat {
    match (from.try_normalize(), to.try_normalize()) {
        (Some(a), Some(b)) => Quat::from_rotation_arc(a, b),
        _ => Quat::IDENTITY,
    }
}

impl Retarget {
    /// The retarget of AC1 parts in reference pose `ac1_ref` (object space) onto `ac2`'s root
    /// skeleton through `pairs`; `None` when a paired bone or part is missing. `joints` are AC1's
    /// limb joints in the reference pose (`Ac1Dats::limb_joints`: part, child part, place;
    /// empty to take the parts' origins), which the parts' fit (`part_on_bone`) maps onto the
    /// AC2 joints.
    pub fn new(
        ac2: &Ac2Setup,
        ac1_ref: &[Ac2Transform],
        pairs: &[BonePair],
        joints: &[(usize, usize, Vec3)],
    ) -> Option<Self> {
        let rest = ac2.rest_locals();
        let world = ac2.world_pose(&rest);
        let parents: Vec<Option<usize>> = ac2.bones.iter().map(|b| b.parent).collect();
        let by_name = |name: u32| ac2.bones.iter().position(|b| b.name == name);
        let mut drive = vec![Drive::Animation; parents.len()];
        let mut swings: Vec<Option<Quat>> = vec![None; parents.len()];
        // The paired bones with a child joint (limbs), and the trunk parts' AC1 origins.
        let mut limb = vec![false; parents.len()];
        let mut trunk: Vec<(usize, Vec3)> = Vec::new();
        // Parents come before children, so a bone's parent swing is known when it is needed.
        let mut order: Vec<(usize, &BonePair)> = pairs
            .iter()
            .map(|p| Some((by_name(p.ac2)?, p)))
            .collect::<Option<_>>()?;
        order.sort_by_key(|(bone, _)| *bone);
        for (bone, pair) in order {
            let part = ac1_ref.get(pair.ac1)?;
            let swing = match (pair.ac2_child.and_then(by_name), pair.ac1_child) {
                (Some(child), Some(ac1_child)) => arc(
                    world[child].offset - world[bone].offset,
                    ac1_ref.get(ac1_child)?.offset - part.offset,
                ),
                _ => {
                    let mut up = parents[bone];
                    loop {
                        match up {
                            Some(p) if swings[p].is_some() => break swings[p].unwrap(),
                            Some(p) => up = parents[p],
                            None => break Quat::IDENTITY,
                        }
                    }
                }
            };
            swings[bone] = Some(swing);
            limb[bone] = pair.ac2_child.is_some();
            let mut up = parents[bone];
            let under_limb = loop {
                match up {
                    Some(p) if limb[p] => break true,
                    Some(p) => up = parents[p],
                    None => break false,
                }
            };
            if !limb[bone] && !under_limb {
                trunk.push((bone, part.offset));
            }
            let aligned = (swing * world[bone].rotation).normalize();
            drive[bone] = Drive::Part {
                part: pair.ac1,
                correction: (part.rotation.inverse() * aligned).normalize(),
            };
        }
        // The undriven bones with a driven bone below them hold their rest.
        for bone in (0..parents.len()).rev() {
            if matches!(drive[bone], Drive::Part { .. } | Drive::Rest) {
                let mut up = parents[bone];
                while let Some(p) = up {
                    if drive[p] == Drive::Animation {
                        drive[p] = Drive::Rest;
                    }
                    up = parents[p];
                }
            }
        }
        let root = parents.iter().position(Option::is_none)?;
        let root_part = match drive[root] {
            Drive::Part { part, .. } => part,
            _ => return None,
        };
        let ac1_root = ac1_ref.get(root_part)?.offset;
        let height = if ac1_root.z > 0.0 {
            world[root].offset.z / ac1_root.z
        } else {
            1.0
        };
        let mut anchors = vec![Vec3::ZERO; parents.len()];
        let mut turns: Vec<Option<Quat>> = vec![None; parents.len()];
        let mut stretches = vec![Vec3::splat(height); parents.len()];
        // AC2's spine over AC1's: the build the limbs' girth follows (hip height, else).
        let mut build = height;
        let mut axes: Vec<Option<Vec3>> = vec![None; parents.len()];

        let is_under = |bone: usize, above: usize| {
            let mut up = parents[bone];
            while let Some(p) = up {
                if p == above {
                    return true;
                }
                up = parents[p];
            }
            false
        };
        for &(bone, origin) in &trunk {
            // A trunk bone is not swung: its aligned place is its rest.
            let target = world[root].offset + (origin - ac1_root) * height;
            anchors[bone] = world[bone].rotation.inverse() * (target - world[bone].offset);
        }
        // A trunk part between the root and another trunk part (the chest, between the hips and
        // the head) goes as far up the AC2 spine as it is up AC1's, leaning as the spine leans
        // there: a hunched spine (the Tumerok's) carries it forward.
        for &(bone, origin) in &trunk {
            let Some(&(top, top_origin)) = trunk
                .iter()
                .filter(|(other, _)| *other != bone && is_under(*other, bone))
                .min_by_key(|(other, _)| *other)
            else {
                continue;
            };
            if bone == root {
                continue;
            }
            let mut path = vec![world[top].offset];
            let mut up = parents[top];
            while let Some(p) = up {
                path.push(world[p].offset);
                if p == root {
                    break;
                }
                up = parents[p];
            }
            path.reverse();
            let ac1_low = (origin - ac1_root).length();
            let ac1_len = ac1_low + (top_origin - origin).length();
            let ac2_len: f32 = path.windows(2).map(|w| (w[1] - w[0]).length()).sum();
            if ac1_len <= 0.0 {
                continue;
            }
            let mut left = ac1_low / ac1_len * ac2_len;
            let mut place = path[0];
            for w in path.windows(2) {
                let step = (w[1] - w[0]).length();
                if left <= step && step > 0.0 {
                    place = w[0] + (w[1] - w[0]) * (left / step);
                    break;
                }
                left -= step;
                place = w[1];
            }
            let rest_turn = world[bone].rotation;
            anchors[bone] = rest_turn.inverse() * (place - world[bone].offset);
            let lean = arc(top_origin - origin, world[top].offset - place);
            let part = ac1_ref.get(match drive[bone] {
                Drive::Part { part, .. } => part,
                _ => continue,
            })?;
            turns[bone] = Some((rest_turn.inverse() * lean * part.rotation).normalize());
            axes[bone] = (part.rotation.inverse() * (top_origin - origin)).try_normalize();
            // The trunk's parts are sized by the spine (AC2's hips to head over AC1's), not the
            // legs: a Tumerok stands tall on its legs over a short, hunched trunk.
            build = ac2_len / ac1_len;
            let spine = Vec3::splat(build);
            for trunk_bone in [root, bone, top] {
                stretches[trunk_bone] = spine;
            }
            // With two limbs starting under it (the arms), the part is fitted to the shoulders
            // too: AC1's waist-to-shoulders line and shoulder line turned onto AC2's, its
            // height scaled by the first, its width by the second, its depth by their mean, so
            // the yoke meets the sleeves (a Tumerok's shoulders sit high and forward).
            let shoulders: Vec<(Vec3, Vec3)> = limb_chains(pairs)
                .iter()
                .filter_map(|chain| {
                    let first = chain.first()?;
                    let start = by_name(first.ac2)?;
                    is_under(start, bone)
                        .then(|| Some((ac1_ref.get(first.ac1)?.offset, world[start].offset)))
                        .flatten()
                })
                .collect();
            if let [(ac1_l, ac2_l), (ac1_r, ac2_r)] = shoulders[..] {
                let up1 = (ac1_l + ac1_r) * 0.5 - origin;
                let up2 = (ac2_l + ac2_r) * 0.5 - place;
                let flat = |v: Vec3, up: Vec3| v - up * v.dot(up);
                let (Some(u1), Some(u2)) = (up1.try_normalize(), up2.try_normalize()) else {
                    continue;
                };
                let tilt = arc(u1, u2);
                let (Some(a1), Some(a2)) = (
                    flat(tilt * (ac1_r - ac1_l), u2).try_normalize(),
                    flat(ac2_r - ac2_l, u2).try_normalize(),
                ) else {
                    continue;
                };
                let turn = (arc(a1, a2) * tilt).normalize();
                turns[bone] = Some((rest_turn.inverse() * turn * part.rotation).normalize());
                let tall = up2.length() / up1.length();
                let wide = (ac2_r - ac2_l).length() / (ac1_r - ac1_l).length();
                let deep = (tall * wide).sqrt();
                let to_part = part.rotation.inverse();
                let local_up = to_part * u1;
                let local_across = to_part * flat(ac1_r - ac1_l, u1).normalize_or_zero();
                let local_depth = local_up.cross(local_across);
                stretches[bone] = tall * local_up * local_up
                    + wide * local_across * local_across
                    + deep * local_depth * local_depth;
            }
        }
        for chain in limb_chains(pairs) {
            let bones: Vec<usize> = chain
                .iter()
                .map(|p| by_name(p.ac2))
                .collect::<Option<_>>()?;
            let parts: Vec<&Ac2Transform> = chain
                .iter()
                .map(|p| ac1_ref.get(p.ac1))
                .collect::<Option<_>>()?;
            let n = chain.len();
            let origins: Vec<Vec3> = parts.iter().map(|t| t.offset).collect();
            // Each part's joint with the part above it (`limb_joints`), else its origin; the
            // first part's is its origin (shoulder, hip).
            let ac1_joints: Vec<Vec3> = (0..n)
                .map(|k| {
                    k.checked_sub(1)
                        .and_then(|above| {
                            joints
                                .iter()
                                .find(|(a, b, _)| *a == chain[above].ac1 && *b == chain[k].ac1)
                        })
                        .map_or(origins[k], |(_, _, at)| *at)
                })
                .collect();
            let ac2_joints: Vec<Vec3> = bones.iter().map(|&b| world[b].offset).collect();
            // Across, a limb's pieces take the body's build (the spine's ratio), not the limb's
            // length: a Tumerok's long legs are not thick ones.
            let across = build;
            for (k, &bone) in bones.iter().enumerate() {
                // The segment the part runs along (the end part: the one before it), mapped
                // joint to joint.
                let segment = k.min(n - 2);
                let ac1_seg = ac1_joints[segment + 1] - ac1_joints[segment];
                let ac2_seg = ac2_joints[segment + 1] - ac2_joints[segment];
                if ac1_seg.length() <= 0.0 {
                    continue;
                }
                let ratio = ac2_seg.length() / ac1_seg.length();
                let past_joint = (origins[k] - ac1_joints[k]).dot(ac1_seg.normalize()) * ratio;
                anchors[bone] =
                    world[bone].rotation.inverse() * (ac2_seg.normalize_or_zero() * past_joint);
                if let Some(axis) = (parts[k].rotation.inverse() * ac1_seg).try_normalize() {
                    // Each axis takes the segment's ratio by how much the limb runs along it.
                    stretches[bone] = Vec3::splat(across) + (ratio - across) * axis.abs();
                    axes[bone] = Some(axis);
                }
            }
        }
        Some(Self {
            parents,
            rest,
            drive,
            anchors,
            turns,
            axes,
            stretches,
            ac1_root,
            height,
        })
    }

    /// Where AC1 part `part` rides the AC2 skeleton: its paired bone and the part's frame in
    /// that bone's (`world(bone) = part * correction`, so the part turns as `world(bone) *
    /// correction⁻¹`, its origin at the bone's anchor), its scale the part's stretch (in the
    /// part's own axes); `None` for a part no bone is paired with.
    pub fn part_on_bone(&self, part: usize) -> Option<(usize, Ac2Transform)> {
        self.drive
            .iter()
            .enumerate()
            .find_map(|(bone, drive)| match drive {
                Drive::Part {
                    part: p,
                    correction,
                } if *p == part => Some((
                    bone,
                    Ac2Transform {
                        offset: self.anchors[bone],
                        scale: self.stretches[bone],
                        rotation: self.turns[bone]
                            .unwrap_or_else(|| correction.inverse().normalize()),
                    },
                )),
                _ => None,
            })
    }

    /// The AC1 part each AC2 bone belongs to: a paired bone its part; a bone between paired
    /// ones (spine, clavicle, neck) the part of the nearest paired bone below it, so the neck
    /// goes with the head and a clavicle with its upper arm; any other bone (fingers, twist
    /// bones, a belly, a loincloth) its parent's, unless it hangs from the root behind it (a
    /// tail): `None`.
    pub fn bone_parts(&self) -> Vec<Option<usize>> {
        let count = self.parents.len();
        let paired = |bone: usize| match self.drive[bone] {
            Drive::Part { part, .. } => Some(part),
            _ => None,
        };
        let mut below: Vec<Option<(usize, usize)>> = vec![None; count];
        // Children come after their parents: walking back up, each bone hears of the nearest
        // paired bone below it (the shallowest, then the first).
        for bone in (0..count).rev() {
            let found = paired(bone).map(|part| (0, part)).or(below[bone]);
            if let (Some((depth, part)), Some(parent)) = (found, self.parents[bone]) {
                let candidate = (depth + 1, part);
                if below[parent].is_none_or(|(d, _)| candidate.0 <= d) {
                    below[parent] = Some(candidate);
                }
            }
        }
        let rest = self.rest_world();
        let mut parts: Vec<Option<usize>> = vec![None; count];
        for bone in 0..count {
            // Below no paired bone, a bone goes with what it hangs from (a belly with the spine's
            // chest, a finger with its hand, a loincloth with the hips), except a tail: straight
            // from the root and behind it (AC2 faces +y), it is no part's.
            parts[bone] = paired(bone)
                .or(below[bone].map(|(_, part)| part))
                .or_else(|| {
                    let parent = self.parents[bone]?;
                    let tail = self.parents[parent].is_none()
                        && rest[bone].offset.y < rest[parent].offset.y;
                    if tail {
                        None
                    } else {
                        parts[parent]
                    }
                });
        }
        parts
    }

    /// AC1 part `part`'s length axis in its own frame (`axes`); `None` for a part with none.
    pub fn part_axis(&self, part: usize) -> Option<Vec3> {
        self.drive
            .iter()
            .enumerate()
            .find_map(|(bone, drive)| match drive {
                Drive::Part { part: p, .. } if *p == part => self.axes[bone],
                _ => None,
            })
    }

    /// The AC2 skeleton's rest pose (placement 0), in object space.
    pub fn rest_world(&self) -> Vec<Ac2Transform> {
        let mut world: Vec<Ac2Transform> = Vec::with_capacity(self.rest.len());
        for (bone, local) in self.rest.iter().enumerate() {
            let placed = match self.parents[bone] {
                Some(parent) => world[parent].compose(local),
                None => *local,
            };
            world.push(placed);
        }
        world
    }

    /// AC2 over AC1 hip height: how much larger the AC2 body stands.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// The AC2 bones' parent-relative transforms for AC1 parts at `ac1` (object space), the
    /// undriven ones from `animation` (the AC2 pose; its rest where it is short).
    pub fn pose(&self, ac1: &[Ac2Transform], animation: &[Ac2Transform]) -> Vec<Ac2Transform> {
        let count = self.parents.len();
        let mut locals = Vec::with_capacity(count);
        let mut world: Vec<Ac2Transform> = Vec::with_capacity(count);
        for bone in 0..count {
            let rest = self.rest[bone];
            let mut local = match self.drive[bone] {
                Drive::Animation => animation.get(bone).copied().unwrap_or(rest),
                _ => rest,
            };
            if let Drive::Part { part, correction } = self.drive[bone] {
                if let Some(frame) = ac1.get(part) {
                    let target = (frame.rotation * correction).normalize();
                    match self.parents[bone] {
                        Some(parent) => {
                            local.rotation =
                                (world[parent].rotation.inverse() * target).normalize();
                        }
                        None => {
                            local.rotation = target;
                            local.offset =
                                rest.offset + (frame.offset - self.ac1_root) * self.height;
                        }
                    }
                }
            }
            let placed = match self.parents[bone] {
                Some(parent) => world[parent].compose(&local),
                None => local,
            };
            world.push(placed);
            locals.push(local);
        }
        locals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_anim::{Ac2Bone, Ac2DegradeInfo, Ac2PlacementFrame};
    use std::sync::Arc;

    fn at(x: f32, y: f32, z: f32) -> Ac2Transform {
        Ac2Transform {
            offset: Vec3::new(x, y, z),
            ..Ac2Transform::IDENTITY
        }
    }

    /// A root, an arm held out along +x (T-pose) and its hand.
    fn skeleton() -> Ac2Setup {
        let bones = [(None, 1), (Some(0), 2), (Some(1), 3)];
        Ac2Setup {
            meshes: Arc::from([]),
            bones: Arc::from(
                bones
                    .iter()
                    .map(|&(parent, name)| Ac2Bone { parent, name })
                    .collect::<Vec<_>>(),
            ),
            placements: Arc::from(vec![Ac2PlacementFrame {
                kind: 0,
                bones: Arc::from(vec![
                    at(0.0, 0.0, 1.0),
                    at(0.2, 0.0, 0.5),
                    at(0.6, 0.0, 0.0),
                ]),
            }]),
            holding_points: Arc::from([]),
            connection_points: Arc::from([]),
            degrade: Ac2DegradeInfo::default(),
            bounds_min: Vec3::ZERO,
            bounds_max: Vec3::ZERO,
        }
    }

    const PAIRS: &[BonePair] = &[pair(1, 0, 0, 0), pair(2, 1, 3, 2), pair(3, 2, 0, 0)];

    #[test]
    fn the_reference_pose_swings_the_t_pose_arm_down_and_ac1_motion_turns_it() {
        // AC1: a root, an arm hanging down -z and its hand below.
        let ac1_ref = [at(0.0, 0.0, 0.9), at(0.2, 0.0, 1.4), at(0.2, 0.0, 0.8)];
        let retarget = Retarget::new(&skeleton(), &ac1_ref, PAIRS, &[]).expect("retarget");
        let place = |locals: &[Ac2Transform]| skeleton().world_pose(locals);
        // In the reference pose the AC2 arm hangs down too: the hand straight below the shoulder.
        let world = place(&retarget.pose(&ac1_ref, &[]));
        let arm = world[2].offset - world[1].offset;
        assert!((arm.normalize() - -Vec3::Z).length() < 1e-4, "{arm:?}");
        assert!((arm.length() - 0.6).abs() < 1e-4, "AC2's own bone length");
        // The AC1 arm swings forward (+y): a quarter turn about +x for the arm and hand.
        let turn = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let mut moved = ac1_ref;
        moved[1].rotation = turn;
        moved[2].rotation = turn;
        let world = place(&retarget.pose(&moved, &[]));
        let arm = world[2].offset - world[1].offset;
        assert!((arm.normalize() - Vec3::Y).length() < 1e-4, "{arm:?}");
        // The root rises as the AC1 root does, scaled by the hip heights (1.0 / 0.9).
        let mut raised = ac1_ref;
        raised[0].offset.z += 0.09;
        let world = place(&retarget.pose(&raised, &[]));
        assert!((world[0].offset.z - 1.1).abs() < 1e-4);
    }

    #[test]
    fn a_part_on_its_bone_turns_with_the_bone_as_the_ac1_part_did() {
        let ac1_ref = [at(0.0, 0.0, 0.9), at(0.2, 0.0, 1.4), at(0.2, 0.0, 0.8)];
        let retarget = Retarget::new(&skeleton(), &ac1_ref, PAIRS, &[]).expect("retarget");
        let turn = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let mut moved = ac1_ref;
        moved[1].rotation = turn;
        let world = skeleton().world_pose(&retarget.pose(&moved, &[]));
        let (bone, in_bone) = retarget.part_on_bone(1).expect("arm part");
        assert_eq!(bone, 1, "bone named 2");
        // The bone's turn undone by the correction is the AC1 part's own, at the joint.
        let part = world[bone].rotation * in_bone.rotation;
        assert!(part.angle_between(turn) < 1e-4, "{part:?}");
        assert_eq!(in_bone.offset, Vec3::ZERO);
        // AC2's arm (0.6) over AC1's (0.6) along; across, the build: with no chest to measure a
        // spine by, the hip heights (1.0 / 0.9).
        let h = 1.0 / 0.9;
        assert!(
            (in_bone.scale - Vec3::new(h, h, 1.0)).length() < 1e-4,
            "{:?}",
            in_bone.scale
        );
        // The root part (a trunk part) sits at the root: its AC1 height over itself is none.
        let (root, in_root) = retarget.part_on_bone(0).expect("root part");
        assert_eq!(root, 0, "bone named 1");
        assert!(in_root.offset.length() < 1e-5, "{:?}", in_root.offset);
        assert!(retarget.part_on_bone(7).is_none());
        // Every bone of the test skeleton is paired: each belongs to its own part.
        assert_eq!(retarget.bone_parts(), vec![Some(0), Some(1), Some(2)]);
        assert!((retarget.height() - 1.0 / 0.9).abs() < 1e-5);
    }
}
