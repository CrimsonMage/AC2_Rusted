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
    /// The AC1 root part's reference place, and AC2 over AC1 hip height.
    ac1_root: Vec3,
    height: f32,
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
    /// skeleton through `pairs`; `None` when a paired bone or part is missing.
    pub fn new(ac2: &Ac2Setup, ac1_ref: &[Ac2Transform], pairs: &[BonePair]) -> Option<Self> {
        let rest = ac2.rest_locals();
        let world = ac2.world_pose(&rest);
        let parents: Vec<Option<usize>> = ac2.bones.iter().map(|b| b.parent).collect();
        let by_name = |name: u32| ac2.bones.iter().position(|b| b.name == name);
        let mut drive = vec![Drive::Animation; parents.len()];
        let mut swings: Vec<Option<Quat>> = vec![None; parents.len()];
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
        Some(Self {
            parents,
            rest,
            drive,
            ac1_root,
            height,
        })
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
        let retarget = Retarget::new(&skeleton(), &ac1_ref, PAIRS).expect("retarget");
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
}
