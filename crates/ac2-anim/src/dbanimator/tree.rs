//! The `DBAnimator` tree: `AnimAsset`, `AnimCube`, `AMHuman` and `AMSwitchME` nodes loaded from
//! the portal DAT, with `GetPeriod`, `Animate` and `GetHoldLoop` as retail evaluates them.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::ArchiveCursor;
use ac2_db::{ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId};

use super::asset::Ac2AnimAsset;
use super::{Ac2AnimatorContext, Ac2BoneTransform, Ac2TrackerState};
use crate::animation::{load_portal, Ac2AnimMap};

const ANIM_ASSET: u32 = 0x4000_0001;
const ANIM_CUBE: u32 = 0x4000_0002;
const AM_HUMAN: u32 = 0x4000_0003;
const AM_SWITCH: u32 = 0x4000_0008;
const MAX_DEPTH: usize = 12;
const MAX_SLIDERS: usize = 8;
const MAX_TICKS: usize = 64;
/// `AnimCube` drops lattice corners lighter than this.
const MIN_WEIGHT: f32 = 0.0002;
/// `AMSwitchME::SwitchType`.
const SWITCH_MODE: u32 = 0;
const SWITCH_ELEMENT: u32 = 1;
const SWITCH_TERRAIN: u32 = 2;
/// Slider ids `AMHuman` drives.
const SLIDER_FORWARD: u32 = 0x4000_0001;
const SLIDER_SIDESTEP: u32 = 0x4000_0002;
const SLIDER_VERTICAL: u32 = 0x4000_0003;
const SLIDER_TURN: u32 = 0x4000_0004;

/// An `AnimCube` slider axis (`SliderAndTickList`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2CubeSlider {
    pub id: u32,
    /// Ascending, at least one.
    pub ticks: Vec<f32>,
}

/// One `DBAnimator`.
#[derive(Debug, Clone, PartialEq)]
pub enum Ac2DbAnimator {
    Asset(Arc<Ac2AnimAsset>),
    Cube {
        sliders: Arc<[Ac2CubeSlider]>,
        children: Arc<[Option<Arc<Ac2DbAnimator>>]>,
    },
    Human(Option<Arc<Ac2DbAnimator>>),
    Switch {
        kind: u32,
        default: Option<Arc<Ac2DbAnimator>>,
        overrides: Arc<[(u32, Option<Arc<Ac2DbAnimator>>)]>,
    },
    /// An animator this port does not evaluate (`AMLook`, `AMAngle`, ...): no pose, period 0.
    Other,
}

impl Ac2DbAnimator {
    /// `DBAnimator::GetPeriod`.
    pub fn period(&self, ctx: &dyn Ac2AnimatorContext, tracker: Ac2TrackerState) -> f32 {
        match self {
            Self::Asset(asset) => asset.period,
            Self::Cube { sliders, children } => {
                let mut period = 0.0_f32;
                for (weight, child) in cube_weights(sliders, children.len(), ctx)
                    .into_iter()
                    .zip(children.iter())
                {
                    if let (true, Some(child)) = (weight >= MIN_WEIGHT, child) {
                        let child_period = child.period(ctx, tracker);
                        period = (f64::from(child_period) * f64::from(weight) + f64::from(period))
                            as f32;
                    }
                }
                period
            }
            Self::Human(child) => child
                .as_ref()
                .map_or(0.0, |child| child.period(ctx, tracker)),
            Self::Switch { .. } => self
                .switch_target(ctx, tracker)
                .map_or(0.0, |child| child.period(ctx, tracker)),
            Self::Other => 0.0,
        }
    }

    /// `DBAnimator::GetHoldLoop`.
    pub fn hold_loop(
        &self,
        ctx: &dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
    ) -> Option<(f32, f32)> {
        match self {
            Self::Asset(asset) => asset.info.as_ref().and_then(|info| info.hold_loop),
            Self::Switch { .. } => self
                .switch_target(ctx, tracker)
                .and_then(|child| child.hold_loop(ctx, tracker)),
            _ => None,
        }
    }

    /// `DBAnimator::Animate`: add this animator's pose at `ctime` with `weight` into `out`.
    #[allow(clippy::too_many_arguments)]
    pub fn animate(
        &self,
        ctx: &mut dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
        weight: f32,
        ctime: f32,
        out: &mut [Ac2BoneTransform],
        out_weights: &mut [f32],
        roots: &[bool],
    ) {
        match self {
            Self::Asset(asset) => asset.animate(weight, ctime, out, out_weights, roots),
            Self::Cube { sliders, children } => {
                let weights = cube_weights(sliders, children.len(), &*ctx);
                for (corner, child) in weights.into_iter().zip(children.iter()) {
                    if let (true, Some(child)) = (corner >= MIN_WEIGHT, child) {
                        let w = (f64::from(corner) * f64::from(weight)) as f32;
                        child.animate(ctx, tracker, w, ctime, out, out_weights, roots);
                    }
                }
            }
            Self::Human(child) => {
                drive_human_sliders(ctx);
                if let Some(child) = child {
                    child.animate(ctx, tracker, weight, ctime, out, out_weights, roots);
                }
            }
            Self::Switch { .. } => {
                if let Some(child) = self.switch_target(&*ctx, tracker) {
                    child.animate(ctx, tracker, weight, ctime, out, out_weights, roots);
                }
            }
            Self::Other => {}
        }
    }

    /// The impulse half of `DBAnimator::GetTriggers`: the impulse triggers at or before `ctime`
    /// (its fraction when `normalize` and past one cycle). `AnimCube` asks its weighted
    /// children at a full cycle, so their impulses all count at once.
    pub fn impulses(
        &self,
        ctx: &dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
        ctime: f32,
        normalize: bool,
        out: &mut Vec<super::Ac2ImpulseTrigger>,
    ) {
        match self {
            Self::Asset(asset) => {
                let Some(info) = asset.info.as_ref() else {
                    return;
                };
                let mut time = ctime;
                if normalize && 1.0 < time {
                    time = f64::from(time).fract() as f32;
                }
                out.extend(info.impulses.iter().filter(|impulse| impulse.time <= time));
            }
            Self::Cube { sliders, children } => {
                for (weight, child) in cube_weights(sliders, children.len(), ctx)
                    .into_iter()
                    .zip(children.iter())
                {
                    if let (true, Some(child)) = (weight >= MIN_WEIGHT, child) {
                        child.impulses(ctx, tracker, 1.0, false, out);
                    }
                }
            }
            Self::Human(child) => {
                if let Some(child) = child {
                    child.impulses(ctx, tracker, ctime, true, out);
                }
            }
            Self::Switch { .. } => {
                if let Some(child) = self.switch_target(ctx, tracker) {
                    child.impulses(ctx, tracker, ctime, true, out);
                }
            }
            Self::Other => {}
        }
    }

    /// `AMSwitchME::GetAnimator`: the override for the tracker's mode, element or the terrain,
    /// else the default.
    fn switch_target(
        &self,
        ctx: &dyn Ac2AnimatorContext,
        tracker: Ac2TrackerState,
    ) -> Option<&Arc<Ac2DbAnimator>> {
        let Self::Switch {
            kind,
            default,
            overrides,
        } = self
        else {
            return None;
        };
        let key = match *kind {
            SWITCH_MODE => tracker.mode,
            SWITCH_ELEMENT => tracker.element,
            SWITCH_TERRAIN => ctx.terrain_type(),
            _ => 0,
        };
        match overrides.iter().find(|(k, _)| *k == key) {
            Some((_, target)) => target.as_ref(),
            None => default.as_ref(),
        }
    }
}

/// `AnimCube::GetAnimatorWeights`: multilinear weights over the bracketing ticks of each slider
/// (a value on a tick weighs that tick 0.5 + 0.5), the first slider varying fastest in the child
/// index; corners under 0.0002 are dropped.
fn cube_weights(
    sliders: &[Ac2CubeSlider],
    children: usize,
    ctx: &dyn Ac2AnimatorContext,
) -> Vec<f32> {
    let mut weights = vec![0.0_f32; children];
    let brackets: Vec<(usize, usize, f32, f32)> = sliders
        .iter()
        .map(|slider| {
            let ticks = &slider.ticks;
            let (first, last) = (ticks[0], ticks[ticks.len() - 1]);
            let mut value = ctx.slider(slider.id);
            if last < value {
                value = last;
            }
            if value < first {
                value = first;
            }
            let mut index = 0;
            loop {
                let tick = ticks[index];
                if tick == value || index + 1 >= ticks.len() {
                    return (index, index, tick, tick);
                }
                if ticks[index + 1] <= value {
                    index += 1;
                    continue;
                }
                return (index, index + 1, tick, ticks[index + 1]);
            }
        })
        .collect();
    for corner in 0..(1_usize << sliders.len()) {
        let mut weight = 1.0_f32;
        let mut index = 0_usize;
        let mut stride = 1_usize;
        for (bit, (slider, &(lo, hi, lo_value, hi_value))) in
            sliders.iter().zip(&brackets).enumerate()
        {
            let mut f = if lo == hi {
                0.5
            } else {
                (f64::from(ctx.slider(slider.id)) - f64::from(hi_value))
                    / (f64::from(lo_value) - f64::from(hi_value))
            };
            let tick = if corner >> bit & 1 == 0 {
                lo
            } else {
                f = 1.0 - f;
                hi
            };
            weight = (f * f64::from(weight)) as f32;
            index += tick * stride;
            stride *= slider.ticks.len();
        }
        if let Some(slot) = weights.get_mut(index) {
            *slot = (f64::from(weight) + f64::from(*slot)) as f32;
        }
    }
    for weight in &mut weights {
        if *weight < MIN_WEIGHT {
            *weight = 0.0;
        }
    }
    weights
}

/// The slider-driving half of `AMHuman::Animate`: sidestep and forward speed over the scaled
/// body, the vertical speed (at most 10 m/s down) and the turn rate per metre of travel, each
/// moved toward over a time that snaps small or large changes.
fn drive_human_sliders(ctx: &mut dyn Ac2AnimatorContext) {
    let mut velocity = ctx.anim_velocity();
    let effects = ctx.effects_scale();
    let vdesc = ctx.vdesc_scale();
    let scale_x = (f64::from(vdesc[0]) * f64::from(effects[0])) as f32;
    let scale_y = (f64::from(vdesc[1]) * f64::from(effects[1])) as f32;
    for component in &mut velocity {
        if f64::from(*component).abs() < 0.001 {
            *component = 0.0;
        }
    }
    let omega = ctx.omega();
    let turn_rad = (f64::from(omega[2]) * f64::from(0.017_455_053_f32)).abs();
    velocity[0] = if turn_rad < 1.0 {
        ((1.0 - turn_rad) * f64::from(velocity[0])) as f32
    } else {
        0.0
    };
    // Sidestep.
    let (vx, vy) = (f64::from(velocity[0]), f64::from(velocity[1]));
    let side_ext = f64::from(ctx.slider(SLIDER_SIDESTEP)) * f64::from(scale_x);
    let side = f64::from(side_ext as f32);
    let mut time = 0.1_f32;
    if (side_ext > 0.0 && side < 0.05 && vx > 0.0) || (side < 0.0 && side > -0.05 && vx < 0.0) {
        time = 0.05;
    } else if (side - vx).abs() > 0.75 {
        time = 0.3;
    }
    ctx.do_slider(SLIDER_SIDESTEP, (vx / f64::from(scale_x)) as f32, time);
    // Forward.
    let forward = f64::from(ctx.slider(SLIDER_FORWARD)) * f64::from(scale_y);
    let mut time = 0.1_f32;
    if (forward > 0.0 && forward < 0.05 && vy > 0.0)
        || (forward < 0.0 && forward > -0.05 && vy < 0.0)
    {
        time = 0.05;
    }
    if (forward - vy).abs() > 0.75 {
        time = 0.3;
    }
    ctx.do_slider(SLIDER_FORWARD, (vy / f64::from(scale_y)) as f32, time);
    // Vertical.
    let mut vertical = velocity[2];
    if f64::from(vertical) < -10.0 {
        vertical = -10.0;
    }
    let current = f64::from(ctx.slider(SLIDER_VERTICAL));
    let time = if current < 0.0 && f64::from(vertical) > current {
        0.4
    } else {
        0.01
    };
    ctx.do_slider(SLIDER_VERTICAL, vertical, time);
    // Turn.
    let speed = (vy * vy + vx * vx).sqrt();
    let mut turn = omega[2];
    if speed > 0.5 {
        turn = (f64::from(omega[2]) / (speed + speed)) as f32;
    }
    let current = f64::from(ctx.slider(SLIDER_TURN));
    let time = if (current - f64::from(turn)).abs() <= 16.0 {
        0.01
    } else {
        0.1
    };
    ctx.do_slider(SLIDER_TURN, turn, time);
}

/// Loads the animator tree `anim_map` plays for `anim_key` (`AnimMap::GetAnimator`).
pub fn load_ac2_db_animator(
    store: &ResourceStore,
    anim_map: DataId,
    anim_key: u32,
) -> ResourceResult<Option<Arc<Ac2DbAnimator>>> {
    let Some(map) = load_portal::<Ac2AnimMap>(store, anim_map)? else {
        return Ok(None);
    };
    let Some(root) = map.animator(anim_key) else {
        return Ok(None);
    };
    let mut loader = Loader {
        store,
        cache: HashMap::new(),
    };
    loader.node(root, 0)
}

struct Loader<'a> {
    store: &'a ResourceStore,
    cache: HashMap<DataId, Option<Arc<Ac2DbAnimator>>>,
}

impl Loader<'_> {
    fn node(&mut self, did: DataId, depth: usize) -> ResourceResult<Option<Arc<Ac2DbAnimator>>> {
        if did.is_null() || depth > MAX_DEPTH {
            return Ok(None);
        }
        if let Some(cached) = self.cache.get(&did) {
            return Ok(cached.clone());
        }
        let Some(record) = load_portal::<AnimatorRecord>(self.store, did)? else {
            self.cache.insert(did, None);
            return Ok(None);
        };
        let node = match &*record {
            AnimatorRecord::Asset(asset) => Ac2DbAnimator::Asset(Arc::clone(asset)),
            AnimatorRecord::Cube { sliders, children } => {
                let children = children
                    .iter()
                    .map(|&child| self.node(child, depth + 1))
                    .collect::<ResourceResult<Arc<[_]>>>()?;
                Ac2DbAnimator::Cube {
                    sliders: Arc::from(sliders.as_slice()),
                    children,
                }
            }
            AnimatorRecord::Human(child) => Ac2DbAnimator::Human(self.node(*child, depth + 1)?),
            AnimatorRecord::Switch {
                kind,
                default,
                overrides,
            } => {
                let default = self.node(*default, depth + 1)?;
                let overrides = overrides
                    .iter()
                    .map(|&(key, child)| Ok((key, self.node(child, depth + 1)?)))
                    .collect::<ResourceResult<Arc<[_]>>>()?;
                Ac2DbAnimator::Switch {
                    kind: *kind,
                    default,
                    overrides,
                }
            }
            AnimatorRecord::Other => Ac2DbAnimator::Other,
        };
        let node = Some(Arc::new(node));
        self.cache.insert(did, node.clone());
        Ok(node)
    }
}

/// One `DBAnimator` record (`Animator::Create` type word, then the type's `StreamPack`).
#[derive(Debug)]
enum AnimatorRecord {
    Asset(Arc<Ac2AnimAsset>),
    Cube {
        sliders: Vec<Ac2CubeSlider>,
        children: Vec<DataId>,
    },
    Human(DataId),
    Switch {
        kind: u32,
        default: DataId,
        overrides: Vec<(u32, DataId)>,
    },
    Other,
}

impl ResourcePackedDecode for AnimatorRecord {
    const TYPE_NAME: &'static str = "AC2 DBAnimator";
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
        let record = match cursor.read_u32()? {
            ANIM_ASSET => Self::Asset(Arc::new(
                Ac2AnimAsset::decode(&mut cursor).map_err(malformed)?,
            )),
            AM_HUMAN => Self::Human(DataId::new(cursor.read_u32()?)),
            AM_SWITCH => {
                let default = DataId::new(cursor.read_u32()?);
                let kind = cursor.read_u32()?;
                let count = (cursor.read_u32()? & 0xFFFF) as usize;
                if count * 8 > cursor.remaining() {
                    return Err(malformed("switch override count"));
                }
                let overrides = (0..count)
                    .map(|_| Ok((cursor.read_u32()?, DataId::new(cursor.read_u32()?))))
                    .collect::<ResourceResult<Vec<_>>>()?;
                Self::Switch {
                    kind,
                    default,
                    overrides,
                }
            }
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
                    lattice = lattice.saturating_mul(ticks.len());
                    sliders.push(Ac2CubeSlider { id, ticks });
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
            _ => return Ok(Self::Other),
        };
        if cursor.remaining() != 0 {
            return Err(malformed("trailing bytes after DBAnimator"));
        }
        Ok(record)
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
