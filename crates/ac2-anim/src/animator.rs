//! Asheron's Call 2 animator trees: a `DBAnimator` (`0x05`) evaluated the way retail
//! `DBAnimator::Animate` walks it. `AnimCube` blends a lattice of child animators
//! by named slider values (`AnimCube::GetAnimatorWeights`), `AMHuman` wraps the
//! locomotion cube whose sliders it drives from the body's velocity, `AMSwitchME`
//! plays its default animator, and `AnimAsset` leaves are keyframe clips. All clips
//! of a blend share one normalized phase (`AnimAsset::Animate` samples `frac(cTime) * period`),
//! and the phase advances by the inverse of the weight-blended period (`CPartArray::GetComboCVel`).

use std::collections::HashMap;
use std::sync::Arc;

use ac2_dat::ArchiveCursor;
use ac2_core::{DataId, QualifiedDataId};
use ac2_math::{Quat, Vec3};
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
};

use crate::animation::{load_portal, Ac2AnimMap, Ac2Animation, Ac2BoneTrack};
use crate::setup::Ac2Transform;

/// Body-local forward speed (`+y`, metres per second over the velocity scale).
pub const AC2_SLIDER_FORWARD_SPEED: u32 = 0x4000_0001;
/// Body-local sidestep speed (`+x`, right).
pub const AC2_SLIDER_SIDESTEP_SPEED: u32 = 0x4000_0002;
/// Turn rate about `+z` in degrees per second (counter-clockwise, a left turn, is positive).
pub const AC2_SLIDER_TURN_RATE: u32 = 0x4000_0004;

const ANIM_ASSET: u32 = 0x4000_0001;
const ANIM_CUBE: u32 = 0x4000_0002;
const AM_HUMAN: u32 = 0x4000_0003;
const AM_SWITCH: u32 = 0x4000_0008;
const MAX_DEPTH: usize = 8;
const MAX_NODES: usize = 512;
const MAX_SLIDERS: usize = 8;
const MAX_TICKS: usize = 64;
/// Retail drops lattice corners lighter than this (`AnimCube::GetAnimatorWeights`).
const MIN_WEIGHT: f32 = 0.0002;

/// A decoded animator tree for one `AnimMap` motion.
#[derive(Debug, Clone)]
pub struct Ac2MotionAnimator {
    root: Node,
}

#[derive(Debug, Clone)]
enum Node {
    Clip(Arc<Ac2Animation>),
    Cube {
        sliders: Arc<[Slider]>,
        children: Arc<[Node]>,
    },
    /// An animator kind this port does not evaluate (`AMLook`, `AMAngle`, ...): contributes nothing.
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
struct Slider {
    id: u32,
    ticks: Vec<f32>,
}

/// Weighted keyframe clips selected by an animator for a set of slider values.
#[derive(Debug, Clone, Default)]
pub struct Ac2Blend {
    clips: Vec<(Arc<Ac2Animation>, f32)>,
}

impl Ac2MotionAnimator {
    /// The clips the tree plays for `sliders` (`(slider id, value)`; unlisted sliders are zero).
    pub fn blend(&self, sliders: &[(u32, f32)]) -> Ac2Blend {
        let mut blend = Ac2Blend::default();
        collect(&self.root, 1.0, sliders, &mut blend);
        blend
    }
}

impl Ac2Blend {
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
    }

    /// The weight-blended clip period (`AnimCube::GetPeriod`); the phase advances by its inverse.
    pub fn period(&self) -> f32 {
        let total: f32 = self.clips.iter().map(|(_, weight)| weight).sum();
        if total <= 0.0 {
            return 0.0;
        }
        self.clips
            .iter()
            .map(|(clip, weight)| clip.period * weight)
            .sum::<f32>()
            / total
    }

    /// The blended pose at normalized `phase` (only its fraction is used) as a still animation:
    /// every bone component some clip keys holds one key, so `Ac2Animation::sample` returns the
    /// blend at any time. Components no clip keys stay empty and keep the rest pose.
    pub fn still_frame(&self, phase: f32) -> Ac2Animation {
        let phase = phase.rem_euclid(1.0);
        let bone_count = self
            .clips
            .iter()
            .map(|(clip, _)| clip.bones.len())
            .max()
            .unwrap_or(0);
        let mut bones = vec![Accumulator::default(); bone_count];
        for (clip, weight) in &self.clips {
            let time = phase * clip.period;
            for (bone, track) in bones.iter_mut().zip(clip.bones.iter()) {
                bone.add(track, time, *weight);
            }
        }
        Ac2Animation {
            period: 0.0,
            bones: bones.into_iter().map(Accumulator::finish).collect(),
        }
    }

    /// Local bone transforms of the blend at `phase` over `rest` (used by tests and callers that
    /// hold the Setup's rest locals).
    pub fn sample(&self, phase: f32, rest: &[Ac2Transform]) -> Vec<Ac2Transform> {
        self.still_frame(phase).sample(0.0, rest)
    }

    /// The selected clips and their weights.
    pub fn clips(&self) -> impl Iterator<Item = (&Ac2Animation, f32)> {
        self.clips
            .iter()
            .map(|(clip, weight)| (clip.as_ref(), *weight))
    }

    fn push(&mut self, clip: &Arc<Ac2Animation>, weight: f32) {
        match self
            .clips
            .iter_mut()
            .find(|(known, _)| Arc::ptr_eq(known, clip))
        {
            Some((_, total)) => *total += weight,
            None => self.clips.push((Arc::clone(clip), weight)),
        }
    }
}

/// Retail `TransformBase::AddWeightedTransform` accumulation: weighted sums of offsets and scales
/// and of hemisphere-aligned quaternions, normalized by the summed weight.
#[derive(Debug, Clone, Default)]
struct Accumulator {
    offset: Option<(Vec3, f32)>,
    scale: Option<(Vec3, f32)>,
    rotation: Option<(Quat, f32)>,
}

impl Accumulator {
    fn add(&mut self, track: &Ac2BoneTrack, time: f32, weight: f32) {
        if let Some(value) = sample_keys(&track.offset, time, Vec3::lerp) {
            let (sum, total) = self.offset.get_or_insert((Vec3::ZERO, 0.0));
            *sum += value * weight;
            *total += weight;
        }
        if let Some(value) = sample_keys(&track.scale, time, Vec3::lerp) {
            let (sum, total) = self.scale.get_or_insert((Vec3::ZERO, 0.0));
            *sum += value * weight;
            *total += weight;
        }
        if let Some(value) = sample_keys(&track.rotation, time, Quat::slerp) {
            let (sum, total) = self
                .rotation
                .get_or_insert((Quat::from_xyzw(0.0, 0.0, 0.0, 0.0), 0.0));
            let aligned = if sum.dot(value) < 0.0 { -value } else { value };
            *sum = *sum + aligned * weight;
            *total += weight;
        }
    }

    fn finish(self) -> Ac2BoneTrack {
        let mean = |entry: Option<(Vec3, f32)>| {
            entry
                .filter(|(_, total)| *total > 0.0)
                .map(|(sum, total)| vec![(0.0, sum / total)])
                .unwrap_or_default()
        };
        let rotation = self
            .rotation
            .filter(|(sum, _)| sum.length_squared() > f32::EPSILON)
            .map(|(sum, _)| vec![(0.0, sum.normalize())])
            .unwrap_or_default();
        Ac2BoneTrack {
            offset: mean(self.offset),
            scale: mean(self.scale),
            rotation,
        }
    }
}

fn sample_keys<T: Copy>(keys: &[(f32, T)], time: f32, mix: fn(T, T, f32) -> T) -> Option<T> {
    let first = keys.first()?;
    if keys.len() == 1 || time <= first.0 {
        return Some(first.1);
    }
    for pair in keys.windows(2) {
        let ((t0, a), (t1, b)) = (pair[0], pair[1]);
        if time <= t1 {
            return Some(mix(
                a,
                b,
                ((time - t0) / (t1 - t0).max(f32::EPSILON)).clamp(0.0, 1.0),
            ));
        }
    }
    Some(keys[keys.len() - 1].1)
}

fn collect(node: &Node, weight: f32, sliders: &[(u32, f32)], blend: &mut Ac2Blend) {
    match node {
        Node::Clip(clip) => blend.push(clip, weight),
        Node::Cube {
            sliders: axes,
            children,
        } => {
            for (index, corner) in lattice_weights(axes, sliders) {
                if let Some(child) = children.get(index) {
                    collect(child, weight * corner, sliders, blend);
                }
            }
        }
        Node::Empty => {}
    }
}

/// `AnimCube::GetAnimatorWeights`: multilinear weights over the bracketing ticks of every slider;
/// the first slider varies fastest in the child index.
fn lattice_weights(axes: &[Slider], sliders: &[(u32, f32)]) -> Vec<(usize, f32)> {
    let brackets: Vec<(usize, usize, f32)> = axes
        .iter()
        .map(|axis| {
            let value = sliders
                .iter()
                .find(|(id, _)| *id == axis.id)
                .map_or(0.0, |(_, value)| *value);
            bracket(&axis.ticks, value)
        })
        .collect();
    let mut weights: Vec<(usize, f32)> = Vec::with_capacity(1 << axes.len());
    for corner in 0..(1_usize << axes.len()) {
        let (mut index, mut stride, mut weight) = (0, 1, 1.0);
        for (bit, (axis, (low, high, t))) in axes.iter().zip(&brackets).enumerate() {
            let upper = corner >> bit & 1 == 1;
            index += if upper { high } else { low } * stride;
            weight *= if upper { *t } else { 1.0 - t };
            stride *= axis.ticks.len();
        }
        match weights.iter_mut().find(|(known, _)| *known == index) {
            Some((_, total)) => *total += weight,
            None => weights.push((index, weight)),
        }
    }
    weights.retain(|(_, weight)| *weight >= MIN_WEIGHT);
    weights
}

/// Lower and upper tick index of `value` (clamped to the tick range) and the upper fraction.
fn bracket(ticks: &[f32], value: f32) -> (usize, usize, f32) {
    let last = ticks.len().saturating_sub(1);
    let (Some(first), Some(end)) = (ticks.first(), ticks.last()) else {
        return (0, 0, 0.0);
    };
    let value = value.clamp(*first, *end);
    for index in 0..last {
        let (low, high) = (ticks[index], ticks[index + 1]);
        if value <= high {
            let span = high - low;
            let t = if span > 0.0 {
                (value - low) / span
            } else {
                0.0
            };
            return (index, index + 1, t.clamp(0.0, 1.0));
        }
    }
    (last, last, 0.0)
}

/// Loads the animator tree an `AnimMap` plays for `motion`.
pub fn load_ac2_motion_animator(
    store: &ResourceStore,
    anim_map: DataId,
    motion: u32,
) -> ResourceResult<Option<Ac2MotionAnimator>> {
    let Some(map) = load_portal::<Ac2AnimMap>(store, anim_map)? else {
        return Ok(None);
    };
    let Some(root) = map.animator(motion) else {
        return Ok(None);
    };
    let mut loader = Loader {
        store,
        clips: HashMap::new(),
        nodes: 0,
    };
    let root = loader.node(root, 0)?;
    Ok((!matches!(root, Node::Empty)).then_some(Ac2MotionAnimator { root }))
}

struct Loader<'a> {
    store: &'a ResourceStore,
    clips: HashMap<DataId, Arc<Ac2Animation>>,
    nodes: usize,
}

impl Loader<'_> {
    fn node(&mut self, did: DataId, depth: usize) -> ResourceResult<Node> {
        self.nodes += 1;
        if depth > MAX_DEPTH || self.nodes > MAX_NODES {
            return Ok(Node::Empty);
        }
        let Some(header) = load_portal::<Ac2AnimatorNode>(self.store, did)? else {
            return Ok(Node::Empty);
        };
        Ok(match &*header {
            Ac2AnimatorNode::Asset => match self.clips.get(&did) {
                Some(clip) => Node::Clip(Arc::clone(clip)),
                None => match load_portal::<Ac2Animation>(self.store, did)? {
                    Some(clip) => {
                        let clip = Arc::new((*clip).clone());
                        self.clips.insert(did, Arc::clone(&clip));
                        Node::Clip(clip)
                    }
                    None => Node::Empty,
                },
            },
            Ac2AnimatorNode::Forward(child) => self.node(*child, depth + 1)?,
            Ac2AnimatorNode::Cube { sliders, children } => {
                let children = children
                    .iter()
                    .map(|child| self.node(*child, depth + 1))
                    .collect::<ResourceResult<Arc<[_]>>>()?;
                Node::Cube {
                    sliders: Arc::from(sliders.as_slice()),
                    children,
                }
            }
            Ac2AnimatorNode::Other => Node::Empty,
        })
    }
}

/// One `DBAnimator` record: its `Animator::Create` type word and the fields this port evaluates.
#[derive(Debug)]
enum Ac2AnimatorNode {
    Asset,
    /// `AMHuman::dbAnimatorID` or `AMSwitchME::mDefault`.
    Forward(DataId),
    /// `AnimCube::StreamPack`: `PSmartArray<SliderAndTickList>` then the child `DBAnimator` IDs.
    Cube {
        sliders: Vec<Slider>,
        children: Vec<DataId>,
    },
    Other,
}

impl ResourcePackedDecode for Ac2AnimatorNode {
    const TYPE_NAME: &'static str = "AC2 DBAnimator node";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(
        qdid: QualifiedDataId,
        _pack_version: u16,
        bytes: &[u8],
    ) -> ResourceResult<Self> {
        let malformed = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut cursor = ArchiveCursor::new(bytes);
        if cursor.read_u32()? != qdid.data_id().raw() {
            return Err(malformed("embedded DID did not match DAT identity"));
        }
        Ok(match cursor.read_u32()? {
            ANIM_ASSET => Self::Asset,
            AM_HUMAN | AM_SWITCH => Self::Forward(DataId::new(cursor.read_u32()?)),
            ANIM_CUBE => {
                let count = cursor.read_u32_count()?;
                if count > MAX_SLIDERS {
                    return Err(malformed("slider count"));
                }
                let mut sliders = Vec::with_capacity(count);
                let mut lattice = 1_usize;
                for _ in 0..count {
                    let id = cursor.read_u32()?;
                    let ticks = cursor.read_u32_count()?;
                    if ticks == 0 || ticks > MAX_TICKS {
                        return Err(malformed("tick count"));
                    }
                    let ticks = (0..ticks)
                        .map(|_| cursor.read_f32())
                        .collect::<Result<Vec<_>, _>>()?;
                    if ticks.windows(2).any(|pair| !(pair[0] <= pair[1])) {
                        return Err(malformed("ticks are not ascending"));
                    }
                    lattice = lattice.saturating_mul(ticks.len());
                    sliders.push(Slider { id, ticks });
                }
                let children = cursor.read_u32_count()?;
                if children != lattice || children * 4 > cursor.remaining() {
                    return Err(malformed("child count does not fill the slider lattice"));
                }
                let children = (0..children)
                    .map(|_| cursor.read_u32().map(DataId::new))
                    .collect::<Result<Vec<_>, _>>()?;
                Self::Cube { sliders, children }
            }
            _ => Self::Other,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + match self {
                Self::Cube { sliders, children } => {
                    sliders
                        .iter()
                        .map(|slider| 8 + slider.ticks.len() * 4)
                        .sum::<usize>()
                        + children.len() * 4
                }
                _ => 0,
            }
    }
}

#[cfg(test)]
#[path = "animator_tests.rs"]
mod tests;
