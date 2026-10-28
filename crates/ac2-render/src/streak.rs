//! `StreakEffect` and its manager (`portal/client/src/render/streak/streakeffect.cpp`,
//! `streakeffectmanager.cpp`): a ribbon of segments that trails two points of an emitter. The
//! emitter's FX starts one (`FXManager::DoStreakFX` -> `CPartArray::CreateStreakEffect`); each
//! frame the emitter places it (`CPartArray::Draw` -> `StreakEffect::SetPosition`), the manager
//! thinks it (`StreakEffect::Think`: a new segment is cut where the two points went, the segments
//! fade over the trail duration, and old ones are dropped), and the ribbon is drawn as a triangle
//! strip of two vertices per segment (`StreakEffect::CreateGeometry`, `Render`).

use std::collections::BTreeMap;

use ac2_core::DataId;
use ac2_math::{Frame, Vec3};

use crate::object::ObjectVertex;
use crate::particles::ParticleGeometry;

/// `StreakEffect::Think`'s cut: a segment goes once it is older than its trail plus this.
const TRAIL_CUT_SECONDS: f64 = 0.1;

/// A streak's make-up (`StreakFX`, as `CPartArray::CreateStreakEffect` passes it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreakSpec {
    /// `m_vLocalLine0` and `m_vLocalLine1`: the two points, in the emitter's space.
    pub start: Vec3,
    pub end: Vec3,
    /// `m_fLifespan`: seconds it lasts (zero: until stopped).
    pub lifespan: f32,
    /// `m_fTrailDuration`: seconds a segment takes to fade out.
    pub trail_duration: f32,
    /// `m_fMinSegmentTime` and `m_fMinSegmentDistance`.
    pub min_segment_time: f32,
    pub min_segment_distance: f32,
    /// `m_cColor` (r, g, b, a).
    pub color: [f32; 4],
    /// `m_fTextureScale`: texture repeats per metre along the ribbon.
    pub texture_scale: f32,
    /// `m_MaterialInstanceID`: the material the ribbon is drawn with.
    pub material: DataId,
}

/// `StreakSegment`: a cut of the two points at client time `created` (streak-relative), its
/// opacity as it fades, and the ribbon's length up to it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Segment {
    pub created: f64,
    pub line0: Vec3,
    pub line1: Vec3,
    pub opacity: f32,
    pub distance: f32,
}

/// The emitter's cell and frame at the last `SetPosition`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Placement {
    cell: u32,
    frame: Frame,
}

/// One streak: its state, its segments (oldest first) and the frame it was last placed in.
#[derive(Debug, Clone, PartialEq)]
pub struct StreakEffect {
    spec: StreakSpec,
    /// `m_fCreationTime`: the client time the streak was made.
    born: f64,
    /// `m_bIsActive`: the streak thinks and cuts segments.
    active: bool,
    /// `m_bPositionValid`: it has been placed since it was last activated.
    placed: bool,
    /// `m_BasePosition.objcell_id`: the cell the streak was first placed in (zero: none).
    base_cell: u32,
    /// `m_Position`: where the emitter stands now.
    current: Placement,
    /// `m_fScale`: the emitter's scale.
    scale: f32,
    /// `m_nSceneStamp`: the frame the streak was last placed in.
    stamp: u64,
    segments: Vec<Segment>,
}

/// `StreakEffect::StreakEffect`: a streak made at `now`; with a lifespan it starts active.
impl StreakEffect {
    pub fn new(spec: StreakSpec, now: f64) -> Self {
        Self {
            spec,
            born: now,
            active: spec.lifespan > 0.0,
            placed: false,
            base_cell: 0,
            current: Placement {
                cell: 0,
                frame: Frame::IDENTITY,
            },
            scale: 1.0,
            stamp: 0,
            segments: Vec::new(),
        }
    }

    /// `StreakEffect::Activate`: it starts again where the emitter stands (its segments restart).
    pub fn activate(&mut self) {
        if !self.active {
            self.placed = false;
            self.active = true;
        }
    }

    /// `StreakEffect::Deactivate`: it stops cutting segments; the trail fades out.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub fn spec(&self) -> &StreakSpec {
        &self.spec
    }

    /// `StreakEffect::SetPosition`: the emitter's cell, frame and scale this frame. The segments
    /// restart only when the streak has no base cell yet, or the emitter enters a cell from none
    /// (a streak keeps its base while either end of the move is in a cell).
    pub fn set_position(&mut self, stamp: u64, cell: u32, frame: Frame, scale: f32) {
        self.current = Placement { cell, frame };
        self.scale = scale;
        self.stamp = stamp;
        if self.placed && (self.base_cell != 0 || cell == 0) {
            return;
        }
        self.segments.clear();
        self.base_cell = cell;
        self.placed = true;
    }

    /// `StreakEffect::Think` at client time `now`: the streak's age expires it when its lifespan
    /// is over; otherwise an active, placed streak cuts its newest segment, and every segment
    /// fades and is dropped past its trail.
    pub fn think(&mut self, now: f64) {
        let age = now - self.born;
        if self.spec.lifespan != 0.0 && self.spec.lifespan < age as f32 {
            self.active = false;
            return;
        }
        if self.active && self.placed {
            self.cut_segment(age);
        }
        self.fade(age);
    }

    /// The world points of the two ends now: the emitter's frame applied to the scaled local
    /// points (`Frame::localtoglobal` after `vWorldLine = fScale * vLocalLine`).
    fn world_line(&self) -> (Vec3, Vec3) {
        let scale = self.scale;
        let frame = self.current.frame;
        (
            frame.local_to_global_point(self.spec.start * scale),
            frame.local_to_global_point(self.spec.end * scale),
        )
    }

    /// Writes the current points into the newest segment. Once the segment is more than the
    /// minimum distance from the one before it (or old enough, `min_segment_time`), it is fixed and
    /// a new segment starts from it; a segment that stays close keeps growing.
    fn cut_segment(&mut self, age: f64) {
        if self.segments.is_empty() {
            self.segments.push(Segment::default());
        }
        let (line0, line1) = self.world_line();
        let last = self.segments.len() - 1;
        self.segments[last] = Segment {
            created: age,
            line0,
            line1,
            opacity: 1.0,
            distance: 0.0,
        };
        if last > 0 {
            let previous = self.segments[last - 1];
            let d0 = squared_distance(line0, previous.line0);
            let d1 = squared_distance(line1, previous.line1);
            let length = d0.max(d1).sqrt();
            self.segments[last].distance = (length + f64::from(previous.distance)) as f32;
            let close = length <= f64::from(self.spec.min_segment_distance);
            let recent = self.spec.min_segment_time == 0.0
                || (age - previous.created) < f64::from(self.spec.min_segment_time);
            if close && recent {
                return;
            }
        }
        let fixed = self.segments[last];
        self.segments.push(fixed);
    }

    /// The fade pass of `StreakEffect::Think`: a segment older than its trail plus 0.1 s goes;
    /// the rest fade to `(1 - age / trail)^2`, clamped at zero before squaring.
    fn fade(&mut self, age: f64) {
        let trail = f64::from(self.spec.trail_duration);
        let mut index = 0;
        while index < self.segments.len() {
            let segment_age = age - self.segments[index].created;
            if segment_age > trail + TRAIL_CUT_SECONDS {
                self.segments.remove(index);
                continue;
            }
            let linear = ((1.0 - segment_age / trail) as f32).max(0.0);
            self.segments[index].opacity = linear * linear;
            index += 1;
        }
    }

    /// `StreakEffect::CreateGeometry` and `Render`: the ribbon as a triangle strip of two vertices
    /// per segment (line1 then line0, the texture's v 0 then 1, its u the segment's distance times
    /// the texture scale, each pair coloured by the segment's opacity and the streak's colour),
    /// appended to `out` as a triangle list. Nothing is drawn unless the streak has two segments
    /// and was placed in frame `stamp` (`m_nSceneStamp == SceneTool::sceneTimestamp`).
    pub fn fill(&self, stamp: u64, out: &mut ParticleGeometry) -> bool {
        if self.segments.len() < 2 || self.stamp != stamp || !self.placed {
            return false;
        }
        let first = out.vertices.len() as u32;
        let [r, g, b, a] = self.spec.color;
        for segment in &self.segments {
            let u = segment.distance * self.spec.texture_scale;
            let colour = [
                channel(f64::from(r)),
                channel(f64::from(g)),
                channel(f64::from(b)),
                channel(f64::from(segment.opacity) * f64::from(a)),
            ];
            out.vertices.push(vertex(segment.line1, [u, 0.0], colour));
            out.vertices.push(vertex(segment.line0, [u, 1.0], colour));
        }
        let count = self.segments.len() as u32 * 2;
        // A strip's triangles alternate winding; the list keeps every triangle facing the same way.
        for k in 0..count - 2 {
            let (lo, hi) = if k % 2 == 0 {
                (first + k, first + k + 1)
            } else {
                (first + k + 1, first + k)
            };
            out.indices.extend([lo, hi, first + k + 2]);
        }
        true
    }
}

/// The streaks of the scene, keyed by a handle the emitter keeps (`StreakEffectManager`).
#[derive(Debug, Clone, Default)]
pub struct StreakEffects {
    next: u64,
    effects: BTreeMap<u64, StreakEffect>,
}

impl StreakEffects {
    /// `StreakEffectManager::CreateStreakEffect`: a new streak, and its handle.
    pub fn create(&mut self, spec: StreakSpec, now: f64) -> u64 {
        self.next += 1;
        self.effects.insert(self.next, StreakEffect::new(spec, now));
        self.next
    }

    /// `StreakEffectManager::DestroyStreakEffect`.
    pub fn destroy(&mut self, id: u64) {
        self.effects.remove(&id);
    }

    pub fn get(&self, id: u64) -> Option<&StreakEffect> {
        self.effects.get(&id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut StreakEffect> {
        self.effects.get_mut(&id)
    }

    /// `StreakEffectManager::UpdateStreakEffects`: every streak thinks at the client time, which
    /// the manager passes narrowed to `float` (as retail does).
    pub fn think(&mut self, now: f64) {
        let now = f64::from(now as f32);
        for effect in self.effects.values_mut() {
            effect.think(now);
        }
    }

    /// `StreakEffectManager::RenderStreakEffects`: the ribbons placed in frame `stamp`, merged
    /// per material in id order (each streak draws with its own material).
    pub fn geometry(&self, stamp: u64) -> Vec<(DataId, ParticleGeometry)> {
        let mut batches: Vec<(DataId, ParticleGeometry)> = Vec::new();
        for effect in self.effects.values() {
            let material = effect.spec.material;
            let index = match batches.iter().position(|(m, _)| *m == material) {
                Some(index) => index,
                None => {
                    batches.push((material, ParticleGeometry::default()));
                    batches.len() - 1
                }
            };
            effect.fill(stamp, &mut batches[index].1);
        }
        batches.retain(|(_, geometry)| !geometry.is_empty());
        batches
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

fn squared_distance(a: Vec3, b: Vec3) -> f64 {
    let d = [
        f64::from(a.x) - f64::from(b.x),
        f64::from(a.y) - f64::from(b.y),
        f64::from(a.z) - f64::from(b.z),
    ];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// A colour channel as the D3D colour packing keeps it: truncated to a byte (`_ftol2` of
/// `c * 255`), then read back as a fraction.
fn channel(value: f64) -> f32 {
    ((value * 255.0) as i64).clamp(0, 255) as f32 / 255.0
}

fn vertex(position: Vec3, uv: [f32; 2], color: [f32; 4]) -> ObjectVertex {
    ObjectVertex {
        position: [position.x, position.y, position.z],
        normal: [0.0, 0.0, 1.0],
        color,
        uvs: [uv, [0.0; 2], [0.0; 2], [0.0; 2]],
        tangents: [[0.0; 3]; 2],
    }
}

#[cfg(test)]
#[path = "streak_tests.rs"]
mod tests;
