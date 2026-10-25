//! `AnimAsset` (`DBAnimator` type 0x40000001) with its keyframe tables as retail keeps them and
//! its optional `AnimInfo` (effect triggers, weenie hooks, impulse triggers, hold loop), and
//! `AnimAsset::Animate`.

use ac2_dat::ArchiveCursor;

use super::{d3dx_slerp, Ac2BoneTransform};

const MAX_BONES: usize = 256;
const MAX_KEYS: usize = 1 << 14;
const MAX_TRIGGERS: usize = 1 << 12;

/// One bone's keyframe tables: `(value, time)` with rotations `(w, x, y, z)`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2AssetBone {
    pub scale: Vec<([f32; 3], f32)>,
    pub offset: Vec<([f32; 3], f32)>,
    pub rotation: Vec<([f32; 4], f32)>,
    pub weight: Vec<(f32, f32)>,
}

/// An `AnimInfo` effect `Trigger`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2AnimTrigger {
    pub fx: u32,
    pub time: f32,
    pub stop: bool,
    pub probability: f32,
    pub intensity: f32,
    pub intensity_var: f32,
    pub play: u32,
}

/// An `AnimInfo` `WeenieHook`: a call into the weenie (and its story manager) when the
/// animation reaches `time`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2WeenieHook {
    pub hook_type: u32,
    pub hook_num: u32,
    pub data: u32,
    /// `mTriggered`, as packed (a hook packed triggered never fires).
    pub triggered: bool,
    pub time: f32,
}

/// An `AnimInfo` `ImpulseTrigger`: a jump, or an impulse velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2ImpulseTrigger {
    pub jump: bool,
    pub impulse: [f32; 3],
    pub num: u32,
    pub time: f32,
}

/// `AnimInfo`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Ac2AnimInfo {
    pub effects: Vec<Ac2AnimTrigger>,
    pub hooks: Vec<Ac2WeenieHook>,
    pub impulses: Vec<Ac2ImpulseTrigger>,
    pub hold_loop: Option<(f32, f32)>,
}

/// `AnimAsset`.
#[derive(Debug, Clone, PartialEq)]
pub struct Ac2AnimAsset {
    pub period: f32,
    pub bones: Vec<Ac2AssetBone>,
    pub info: Option<Ac2AnimInfo>,
}

type Malformed = &'static str;

fn word(cursor: &mut ArchiveCursor<'_>) -> Result<u32, Malformed> {
    cursor.read_u32().map_err(|_| "truncated AnimAsset")
}

fn float(cursor: &mut ArchiveCursor<'_>) -> Result<f32, Malformed> {
    word(cursor).map(f32::from_bits)
}

fn count(cursor: &mut ArchiveCursor<'_>, max: usize, item: usize) -> Result<usize, Malformed> {
    let n = word(cursor)? as usize;
    if n > max || n * item > cursor.remaining() {
        return Err("AnimAsset count out of range");
    }
    Ok(n)
}

fn vec3(cursor: &mut ArchiveCursor<'_>) -> Result<[f32; 3], Malformed> {
    Ok([float(cursor)?, float(cursor)?, float(cursor)?])
}

impl Ac2AnimAsset {
    /// `AnimAsset::StreamPackInternal` after the `DBAnimator` id and type words.
    pub(crate) fn decode(cursor: &mut ArchiveCursor<'_>) -> Result<Self, Malformed> {
        let period = float(cursor)?;
        let bone_count = count(cursor, MAX_BONES, 0)?;
        let mut bones = Vec::with_capacity(bone_count);
        for _ in 0..bone_count {
            let mut bone = Ac2AssetBone::default();
            for _ in 0..count(cursor, MAX_KEYS, 16)? {
                let value = vec3(cursor)?;
                bone.scale.push((value, float(cursor)?));
            }
            for _ in 0..count(cursor, MAX_KEYS, 16)? {
                let value = vec3(cursor)?;
                bone.offset.push((value, float(cursor)?));
            }
            for _ in 0..count(cursor, MAX_KEYS, 20)? {
                let value = [
                    float(cursor)?,
                    float(cursor)?,
                    float(cursor)?,
                    float(cursor)?,
                ];
                bone.rotation.push((value, float(cursor)?));
            }
            for _ in 0..count(cursor, MAX_KEYS, 8)? {
                let value = float(cursor)?;
                bone.weight.push((value, float(cursor)?));
            }
            bones.push(bone);
        }
        let info = if cursor.remaining() >= 4 && word(cursor)? != 0 {
            Some(decode_info(cursor)?)
        } else {
            None
        };
        if cursor.remaining() != 0 {
            return Err("trailing bytes after AnimAsset");
        }
        Ok(Self {
            period,
            bones,
            info,
        })
    }

    /// `AnimAsset::Animate`: add every bone's pose at `ctime` (cycles; the fraction is the
    /// phase) with its weight-key blend weight times `weight`. A root bone's rotation is turned
    /// into the hemisphere of what is accumulated already.
    pub(crate) fn animate(
        &self,
        weight: f32,
        ctime: f32,
        out: &mut [Ac2BoneTransform],
        out_weights: &mut [f32],
        roots: &[bool],
    ) {
        if weight == 0.0 {
            return;
        }
        let c = f64::from(ctime);
        let mut time = ((c - c.floor()) * f64::from(self.period)) as f32;
        for (index, bone) in self.bones.iter().enumerate() {
            if index >= out.len() || index >= out_weights.len() {
                break;
            }
            if !(0.0 < time) {
                time = 0.0;
            }
            let offset = lerp3(&bone.offset, time).unwrap_or([0.0; 3]);
            let scale = lerp3(&bone.scale, time).unwrap_or([1.0; 3]);
            let mut rotation = slerp_keys(&bone.rotation, time).unwrap_or([1.0, 0.0, 0.0, 0.0]);
            let bone_weight = weight_at(&bone.weight, time);
            let w = (f64::from(bone_weight) * f64::from(weight)) as f32;
            let accumulated = out[index].rotation;
            if roots.get(index).copied().unwrap_or(false) {
                let [qw, qx, qy, qz] = rotation.map(f64::from);
                let [aw, ax, ay, az] = accumulated.map(f64::from);
                if ((qy * ay + qz * az) + qx * ax) + qw * aw < 0.0 {
                    rotation = rotation.map(|v| -v);
                }
            }
            let pose = Ac2BoneTransform {
                offset,
                scale,
                rotation,
            };
            out[index].add_weighted(&pose, w);
            out_weights[index] = (f64::from(w) + f64::from(out_weights[index])) as f32;
        }
    }
}

/// `KeyframeLookupTable::FindLargestLTEQ`: the last key at or before `time` (the first key
/// when none is).
fn largest_at_or_before<T>(keys: &[(T, f32)], time: f32) -> usize {
    keys.iter()
        .rposition(|(_, key_time)| *key_time <= time)
        .unwrap_or(0)
}

/// The key at or before `time` and the fraction toward the next one (0 on the last key).
fn bracket<T>(keys: &[(T, f32)], time: f32) -> Option<(usize, usize, f32)> {
    if keys.is_empty() {
        return None;
    }
    let index = largest_at_or_before(keys, time);
    let next = index + 1;
    if next == keys.len() {
        return Some((index, 0, 0.0));
    }
    let t0 = f64::from(keys[index].1);
    let fraction = (f64::from(time) - t0) / (f64::from(keys[next].1) - t0);
    Some((index, next, fraction as f32))
}

fn lerp3(keys: &[([f32; 3], f32)], time: f32) -> Option<[f32; 3]> {
    let (index, next, fraction) = bracket(keys, time)?;
    let a = keys[index].0;
    if fraction == 0.0 {
        return Some(a);
    }
    let b = keys[next].0;
    let f = f64::from(fraction);
    Some([0, 1, 2].map(|i| ((f64::from(b[i]) - f64::from(a[i])) * f + f64::from(a[i])) as f32))
}

fn slerp_keys(keys: &[([f32; 4], f32)], time: f32) -> Option<[f32; 4]> {
    let (index, next, fraction) = bracket(keys, time)?;
    if fraction == 0.0 {
        return Some(keys[index].0);
    }
    Some(d3dx_slerp(keys[index].0, keys[next].0, fraction))
}

/// The weight keys' blend weight (1 without keys).
fn weight_at(keys: &[(f32, f32)], time: f32) -> f32 {
    let Some((index, next, fraction)) = bracket(keys, time) else {
        return 1.0;
    };
    if fraction == 0.0 {
        return keys[index].0;
    }
    let f = f64::from(fraction);
    (f * f64::from(keys[next].0) + (1.0 - f) * f64::from(keys[index].0)) as f32
}

/// `AnimInfo::StreamPack`.
fn decode_info(cursor: &mut ArchiveCursor<'_>) -> Result<Ac2AnimInfo, Malformed> {
    let mut info = Ac2AnimInfo::default();
    for _ in 0..count(cursor, MAX_TRIGGERS, 28)? {
        info.effects.push(Ac2AnimTrigger {
            fx: word(cursor)?,
            time: float(cursor)?,
            stop: word(cursor)? != 0,
            probability: float(cursor)?,
            intensity: float(cursor)?,
            intensity_var: float(cursor)?,
            play: word(cursor)?,
        });
    }
    for _ in 0..count(cursor, MAX_TRIGGERS, 20)? {
        let hook_type = word(cursor)?;
        let hook_num = word(cursor)?;
        let data = word(cursor)?;
        let triggered = word(cursor)? != 0;
        info.hooks.push(Ac2WeenieHook {
            hook_type,
            hook_num,
            data,
            triggered,
            time: float(cursor)?,
        });
    }
    for _ in 0..count(cursor, MAX_TRIGGERS, 12)? {
        let jump = word(cursor)? != 0;
        let impulse = if jump { [0.0; 3] } else { vec3(cursor)? };
        info.impulses.push(Ac2ImpulseTrigger {
            jump,
            impulse,
            num: word(cursor)?,
            time: float(cursor)?,
        });
    }
    let has_hold = word(cursor)? != 0;
    let (start, end) = (float(cursor)?, float(cursor)?);
    info.hold_loop = has_hold.then_some((start, end));
    Ok(info)
}
