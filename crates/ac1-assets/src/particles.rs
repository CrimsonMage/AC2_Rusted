//! AC1 particles: a `PhysicsScript` (`0x33`), the timed hooks an object runs (its Setup's
//! `default_script` when it is made), and the `ParticleEmitterInfo` (`0x32`) a `CreateParticle`
//! hook starts on one of the object's parts.

use crate::formats::{Ac1Result, Frame, Reader};

/// `AnimHook` types that start an emitter: `CreateParticle` and `CreateBlockingParticle`.
const CREATE_PARTICLE: u32 = 0x0D;
const CREATE_BLOCKING_PARTICLE: u32 = 0x1A;

/// `CreateParticleHook`: an emitter started on a part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateParticle {
    /// Seconds into the script.
    pub start: f64,
    pub emitter_info: u32,
    pub part: u32,
    /// The emitter's frame in the part's.
    pub offset: Frame,
    pub emitter_id: u32,
}

/// `PhysicsScript::UnPack`: the script's particle starts (its other hooks are read past).
pub fn decode_physics_script(did: u32, bytes: &[u8]) -> Ac1Result<Vec<CreateParticle>> {
    let mut r = Reader::new(did, bytes);
    r.header()?;
    let n = r.u32_count(16, "missing PhysicsScript count")?;
    let mut out = Vec::new();
    for _ in 0..n {
        let start = r.f64("missing PhysicsScriptData start time")?;
        let kind = r.u32("missing AnimHook type")?;
        r.u32("missing AnimHook direction")?;
        if matches!(kind, CREATE_PARTICLE | CREATE_BLOCKING_PARTICLE) {
            out.push(CreateParticle {
                start,
                emitter_info: r.u32("missing CreateParticle emitter info")?,
                part: r.u32("missing CreateParticle part")?,
                offset: r.frame("missing CreateParticle offset")?,
                emitter_id: r.u32("missing CreateParticle emitter id")?,
            });
            r.skip_hook_body(0)?;
        } else {
            r.skip_hook_body(kind)?;
        }
    }
    Ok(out)
}

impl crate::Ac1Dats {
    /// The particle starts of `PhysicsScript` `did`.
    pub fn physics_script(&self, did: u32) -> Ac1Result<Vec<CreateParticle>> {
        self.decode(did, decode_physics_script)
    }

    /// `ParticleEmitterInfo` `did`.
    pub fn emitter_info(&self, did: u32) -> Ac1Result<ParticleEmitterInfo> {
        self.decode(did, ParticleEmitterInfo::decode)
    }
}

/// `ParticleEmitterInfo` (`0x32`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleEmitterInfo {
    /// `EmitterType`: 1 particles per second (`birthrate` seconds apart), 2 per metre moved.
    pub emitter_type: i32,
    /// `ParticleType`: the motion (1 still, 2 local velocity, 3 parabolic local velocity and
    /// global acceleration, ..., 5 swarm, 6 explode, 7 implode, 12 global velocity).
    pub particle_type: i32,
    pub gfx_obj: u32,
    pub hw_gfx_obj: u32,
    pub birthrate: f64,
    pub max_particles: i32,
    pub initial_particles: i32,
    pub total_particles: i32,
    pub total_seconds: f64,
    pub lifespan: f64,
    pub lifespan_rand: f64,
    pub offset_dir: [f32; 3],
    pub min_offset: f32,
    pub max_offset: f32,
    pub a: [f32; 3],
    pub min_a: f32,
    pub max_a: f32,
    pub b: [f32; 3],
    pub min_b: f32,
    pub max_b: f32,
    pub c: [f32; 3],
    pub min_c: f32,
    pub max_c: f32,
    pub start_scale: f32,
    pub final_scale: f32,
    pub scale_rand: f32,
    pub start_trans: f32,
    pub final_trans: f32,
    pub trans_rand: f32,
    pub parent_local: bool,
}

impl ParticleEmitterInfo {
    /// `ParticleEmitterInfo::UnPack` (a pack word after the DBObj header, then the fields).
    pub fn decode(did: u32, bytes: &[u8]) -> Ac1Result<Self> {
        let mut r = Reader::new(did, bytes);
        r.header()?;
        r.u32("missing ParticleEmitterInfo pack word")?;
        Ok(Self {
            emitter_type: r.i32("missing emitter type")?,
            particle_type: r.i32("missing particle type")?,
            gfx_obj: r.u32("missing particle GfxObj")?,
            hw_gfx_obj: r.u32("missing particle hardware GfxObj")?,
            birthrate: r.f64("missing birthrate")?,
            max_particles: r.i32("missing max particles")?,
            initial_particles: r.i32("missing initial particles")?,
            total_particles: r.i32("missing total particles")?,
            total_seconds: r.f64("missing total seconds")?,
            lifespan: r.f64("missing lifespan")?,
            lifespan_rand: r.f64("missing lifespan randomness")?,
            offset_dir: r.vec3("missing offset direction")?,
            min_offset: r.f32("missing min offset")?,
            max_offset: r.f32("missing max offset")?,
            a: r.vec3("missing A")?,
            min_a: r.f32("missing min A")?,
            max_a: r.f32("missing max A")?,
            b: r.vec3("missing B")?,
            min_b: r.f32("missing min B")?,
            max_b: r.f32("missing max B")?,
            c: r.vec3("missing C")?,
            min_c: r.f32("missing min C")?,
            max_c: r.f32("missing max C")?,
            start_scale: r.f32("missing start scale")?,
            final_scale: r.f32("missing final scale")?,
            scale_rand: r.f32("missing scale randomness")?,
            start_trans: r.f32("missing start translucency")?,
            final_trans: r.f32("missing final translucency")?,
            trans_rand: r.f32("missing translucency randomness")?,
            parent_local: r.u32("missing parent-local flag")? != 0,
        })
    }
}

use ac2_math::{Quat, Vec3};

/// `ParticleType` values with their own motion.
mod kind {
    pub const STILL: i32 = 1;
    pub const LOCAL_VELOCITY: i32 = 2;
    pub const PARABOLIC_LVGA: i32 = 3;
    pub const PARABOLIC_LVGAGR: i32 = 4;
    pub const SWARM: i32 = 5;
    pub const EXPLODE: i32 = 6;
    pub const IMPLODE: i32 = 7;
    pub const PARABOLIC_LVLA: i32 = 8;
    pub const PARABOLIC_LVLALR: i32 = 9;
    // 10 and 11 (`ParabolicGVGA`, `ParabolicGVGAGR`) take their vectors as given and move as
    // the parabolic types do.
    pub const GLOBAL_VELOCITY: i32 = 12;
}

/// `EmitterType.BirthratePerSec`.
const BIRTHRATE_PER_SEC: i32 = 1;

/// A part's place in the world: what an emitter rides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartFrame {
    pub origin: Vec3,
    pub rotation: Quat,
}

/// One live particle (`Particle`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Particle {
    birth: f64,
    lifespan: f64,
    /// `StartFrame`: the part's frame when it was born.
    start: PartFrame,
    /// `Offset`: the hook's offset and the random offset, in world axes.
    offset: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    scale: (f32, f32),
    trans: (f32, f32),
}

/// A particle to draw: where, how large, how translucent, with which GfxObj.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleDraw {
    pub position: Vec3,
    pub scale: f32,
    /// 0 opaque, 1 invisible.
    pub translucency: f32,
    pub gfx_obj: u32,
}

/// A running emitter (`ParticleEmitter`): the hook that started it and its particles.
#[derive(Debug, Clone)]
pub struct Emitter {
    pub info: ParticleEmitterInfo,
    pub hook: CreateParticle,
    created: f64,
    last_emit: f64,
    emitted: i32,
    stopped: bool,
    particles: Vec<Particle>,
    rng: u64,
}

impl Emitter {
    /// The emitter `hook` starts, created at `now` (`InitEnd` emits the total at once).
    pub fn new(info: ParticleEmitterInfo, hook: CreateParticle, seed: u64, now: f64) -> Self {
        Self {
            info,
            hook,
            created: now,
            last_emit: f64::MIN,
            emitted: 0,
            stopped: false,
            particles: Vec::new(),
            rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1,
        }
    }

    /// The GfxObj each particle draws (the hardware one, else the software one).
    pub fn gfx_obj(&self) -> u32 {
        if self.info.hw_gfx_obj != 0 {
            self.info.hw_gfx_obj
        } else {
            self.info.gfx_obj
        }
    }

    /// A roll in `[lo, hi)` (`ThreadSafeRandom.Next`).
    fn roll(&mut self, lo: f32, hi: f32) -> f32 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        let unit =
            (self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32 / (1u32 << 24) as f32;
        lo + (hi - lo) * unit
    }

    /// `ParticleEmitter::UpdateParticles` at `now` with the part at `part`: the dead particles
    /// go, and one is born when the birth rate allows (`ShouldEmitParticle`).
    pub fn update(&mut self, now: f64, part: PartFrame) {
        if self.emitted == 0 && self.info.total_particles > 0 && self.particles.is_empty() {
            for _ in 0..self.info.total_particles.min(self.info.max_particles) {
                self.emit(now, part);
            }
        }
        self.particles.retain(|p| now - p.birth < p.lifespan);
        let info = self.info;
        let under_total = info.total_particles <= 0 || self.emitted < info.total_particles;
        if !self.stopped
            && under_total
            && (self.particles.len() as i32) < info.max_particles
            && info.emitter_type == BIRTHRATE_PER_SEC
            && now - self.last_emit > info.birthrate
        {
            self.emit(now, part);
        }
        // `StopEmitter`: an emitter with a time limit stops after it.
        if info.total_seconds > 0.0 && now - self.created > info.total_seconds {
            self.stopped = true;
        }
    }

    /// `ParticleEmitter::EmitParticle` and `Particle::Init`.
    fn emit(&mut self, now: f64, part: PartFrame) {
        let info = self.info;
        let v = |x: [f32; 3]| Vec3::from(x);
        // `GetRandomOffset`: a random direction off the offset axis, scaled.
        let dir = v(info.offset_dir);
        let r = Vec3::new(
            self.roll(-1.0, 1.0),
            self.roll(-1.0, 1.0),
            self.roll(-1.0, 1.0),
        );
        let across = r - dir * dir.dot(r);
        let random_offset = if across.length_squared() < 1e-8 {
            Vec3::ZERO
        } else {
            across.normalize()
                * ((info.max_offset - info.min_offset) + info.min_offset)
                * self.roll(0.0, 1.0)
        };
        let magnitude = |e: &mut Self, lo: f32, hi: f32| (hi - lo) * e.roll(0.0, 1.0) + lo;
        let a = v(info.a) * magnitude(self, info.min_a, info.max_a);
        let b = v(info.b) * magnitude(self, info.min_b, info.max_b);
        let c = v(info.c) * magnitude(self, info.min_c, info.max_c);
        let rot = part.rotation;
        let offset = rot * (v(self.hook.offset.origin) + random_offset);
        let (mut pa, mut pb, mut pc) = (a, b, c);
        let mut poffset = offset;
        match info.particle_type {
            kind::LOCAL_VELOCITY => pa = rot * a,
            kind::PARABOLIC_LVGA => pa = rot * a,
            kind::PARABOLIC_LVGAGR | kind::SWARM => pa = rot * a,
            kind::EXPLODE => {
                let ra = self.roll(-std::f32::consts::PI, std::f32::consts::PI);
                let po = self.roll(-std::f32::consts::PI, std::f32::consts::PI);
                let rb = po.cos();
                let dir = Vec3::new(
                    ra.cos() * c.x * rb,
                    ra.sin() * c.y * rb,
                    po.sin() * c.z * rb,
                );
                pc = dir.try_normalize().unwrap_or(Vec3::ZERO);
            }
            kind::IMPLODE => {
                poffset = offset * c;
                pc = poffset;
            }
            kind::PARABOLIC_LVLA => {
                pa = rot * a;
                pb = rot * b;
            }
            kind::PARABOLIC_LVLALR => {
                pa = rot * a;
                pb = rot * b;
                pc = rot * c;
            }
            _ => {}
        }
        let spread = |e: &mut Self, base: f32, rand: f32, lo: f32, hi: f32| {
            (base + e.roll(-1.0, 1.0) * rand).clamp(lo, hi)
        };
        let scale = (
            spread(self, info.start_scale, info.scale_rand, 0.1, 10.0),
            spread(self, info.final_scale, info.scale_rand, 0.1, 10.0),
        );
        let trans = (
            spread(self, info.start_trans, info.trans_rand, 0.0, 1.0),
            spread(self, info.final_trans, info.trans_rand, 0.0, 1.0),
        );
        let lifespan =
            (info.lifespan + f64::from(self.roll(-1.0, 1.0)) * info.lifespan_rand).max(0.0);
        self.particles.push(Particle {
            birth: now,
            lifespan,
            start: part,
            offset: poffset,
            a: pa,
            b: pb,
            c: pc,
            scale,
            trans,
        });
        self.emitted += 1;
        self.last_emit = now;
    }

    /// `Particle::Update` for every live particle at `now`, the part at `part` (parent-local
    /// particles ride it; the others keep the frame they were born in).
    pub fn draws(&self, now: f64, part: PartFrame) -> impl Iterator<Item = ParticleDraw> + '_ {
        let gfx_obj = self.gfx_obj();
        self.particles.iter().map(move |p| {
            let frame = if self.info.parent_local {
                part
            } else {
                p.start
            };
            let t = (now - p.birth).max(0.0) as f32;
            let base = frame.origin + p.offset;
            let position = match self.info.particle_type {
                kind::STILL => base,
                kind::LOCAL_VELOCITY | kind::GLOBAL_VELOCITY => base + p.a * t,
                kind::SWARM => {
                    let swarm = base + p.a * t;
                    Vec3::new(
                        (t * p.b.x).cos() * p.c.x + swarm.x,
                        (t * p.b.y).sin() * p.c.y + swarm.y,
                        (t * p.b.z).cos() * p.c.z + swarm.z,
                    )
                }
                kind::EXPLODE => (p.b * t + p.c * p.a.x) * t + base,
                kind::IMPLODE => p.c * (p.a.x * t).cos() + p.b * (t * t) + base,
                _ => base + p.a * t + p.b * (t * t / 2.0),
            };
            let interval = if p.lifespan > 0.0 {
                (f64::from(t) / p.lifespan).min(1.0) as f32
            } else {
                1.0
            };
            ParticleDraw {
                position,
                scale: p.scale.0 + (p.scale.1 - p.scale.0) * interval,
                translucency: p.trans.0 + (p.trans.1 - p.trans.0) * interval,
                gfx_obj,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(particle_type: i32) -> ParticleEmitterInfo {
        ParticleEmitterInfo {
            emitter_type: BIRTHRATE_PER_SEC,
            particle_type,
            gfx_obj: 0,
            hw_gfx_obj: 7,
            birthrate: 0.1,
            max_particles: 3,
            initial_particles: 0,
            total_particles: 0,
            total_seconds: 0.0,
            lifespan: 1.0,
            lifespan_rand: 0.0,
            offset_dir: [0.0, 0.0, 1.0],
            min_offset: 0.0,
            max_offset: 0.0,
            a: [0.0, 0.0, 1.0],
            min_a: 2.0,
            max_a: 2.0,
            b: [0.0; 3],
            min_b: 1.0,
            max_b: 1.0,
            c: [0.0; 3],
            min_c: 1.0,
            max_c: 1.0,
            start_scale: 1.0,
            final_scale: 3.0,
            scale_rand: 0.0,
            start_trans: 0.0,
            final_trans: 1.0,
            trans_rand: 0.0,
            parent_local: false,
        }
    }

    fn hook() -> CreateParticle {
        CreateParticle {
            start: 0.0,
            emitter_info: 0,
            part: 0,
            offset: Frame::IDENTITY,
            emitter_id: 0,
        }
    }

    #[test]
    fn a_global_velocity_particle_rises_grows_and_fades_then_dies() {
        let part = PartFrame {
            origin: Vec3::new(10.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
        };
        let mut emitter = Emitter::new(info(kind::GLOBAL_VELOCITY), hook(), 1, 0.0);
        emitter.update(0.0, part);
        let draws: Vec<_> = emitter.draws(0.5, part).collect();
        assert_eq!(draws.len(), 1);
        assert!((draws[0].position - Vec3::new(10.0, 0.0, 1.0)).length() < 1e-5);
        assert!((draws[0].scale - 2.0).abs() < 1e-5);
        assert!((draws[0].translucency - 0.5).abs() < 1e-5);
        assert_eq!(draws[0].gfx_obj, 7);
        // At most the max, one per birth interval, and each dies at its lifespan.
        for step in 1..40 {
            emitter.update(f64::from(step) * 0.05, part);
        }
        assert!(emitter.draws(2.0, part).count() <= 3);
        emitter.update(10.0, part);
        assert_eq!(emitter.draws(10.0, part).count(), 1);
    }
}
