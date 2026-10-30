//! The AC1 objects' particles: each AC1 creature or weapon whose Setup has a `default_script`
//! runs that `PhysicsScript`'s emitters (`ac1_assets::particles::Emitter`, AC1's particle
//! motion) on the part each `CreateParticle` hook names, where the view drew that part the frame
//! before (`ClientView::object_placement`). Every live particle is drawn as an object of its own:
//! its AC1 GfxObj (a quad facing local -y) turned to face the camera, at its scale, faded by its
//! translucency through the frame's tints.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ac1_assets::particles::{CreateParticle, Emitter, PartFrame, ParticleEmitterInfo};
use ac1_assets::{Ac1Dats, Ac1Models};
use ac2_client::view::ClientView;
use ac2_client::{ClientFrame, ObjectFrame};
use ac2_core::DataId;
use ac2_math::{Mat4, Quat, Vec3, Vec4};
use ac2_weenie::Appearance;

/// The particles' object ids: the first, and how many a frame may hold.
const FIRST_PARTICLE: u64 = 0x7000_0000;
const MAX_PARTICLES: u64 = 0x10000;
/// The particles' model visuals (`AC1_VISUAL_BASE` onwards).
const PARTICLE_VISUALS: u32 = 0xC00;

/// A script's particle starts with their emitter infos.
type Script = Arc<[(CreateParticle, ParticleEmitterInfo)]>;

/// The scripts of the AC1 visuals and the emitters running on the drawn objects.
#[derive(Default)]
pub struct Ac1Fx {
    /// Each AC1 visual's default script and the scale its particles draw at.
    scripts: HashMap<u32, (Script, f32)>,
    /// Each particle GfxObj's model visual.
    visuals: HashMap<u32, u32>,
    /// Each drawn object's emitters, by object id.
    running: HashMap<u64, Vec<Emitter>>,
    /// Each AC1 visual's Setup lights, as the scene's object lights.
    lights: HashMap<u32, Vec<ac2_assets::Ac2FxLight>>,
    /// The drawn objects whose lights are on.
    lit: HashSet<u64>,
    seed: u64,
}

impl Ac1Fx {
    /// Reads `script` for objects drawn under `visual`, its particles drawn at `particle_scale`
    /// (a held weapon's held scale), and gives each particle GfxObj it uses a model in `models`.
    /// A script that does not decode is reported and left out.
    pub fn add(
        &mut self,
        dats: &Ac1Dats,
        models: &mut Ac1Models,
        visual: u32,
        script: u32,
        particle_scale: f32,
    ) {
        if script == 0 {
            return;
        }
        let starts = match dats.physics_script(script) {
            Ok(starts) => starts,
            Err(error) => {
                eprintln!("world-viewer: {error}");
                return;
            }
        };
        let mut entries = Vec::new();
        for start in starts {
            let info = match dats.emitter_info(start.emitter_info) {
                Ok(info) => info,
                Err(error) => {
                    eprintln!("world-viewer: {error}");
                    continue;
                }
            };
            let gfx = if info.hw_gfx_obj != 0 {
                info.hw_gfx_obj
            } else {
                info.gfx_obj
            };
            if gfx != 0 && !self.visuals.contains_key(&gfx) {
                let model_visual = ac1_assets::creature::AC1_VISUAL_BASE
                    + PARTICLE_VISUALS
                    + self.visuals.len() as u32;
                match dats.particle_model(gfx, DataId::new(model_visual)) {
                    Ok((model, images)) => {
                        models.insert_model(DataId::new(model_visual), model, images);
                        self.visuals.insert(gfx, model_visual);
                    }
                    Err(error) => {
                        eprintln!("world-viewer: {error}");
                        continue;
                    }
                }
            }
            entries.push((start, info));
        }
        if !entries.is_empty() {
            self.scripts
                .insert(visual, (Arc::from(entries), particle_scale));
        }
    }

    /// The Setup lights of objects drawn under `visual` (`LIGHTINFO`: its place in the object's
    /// frame, its ARGB colour, its intensity in percent and its reach in metres), on while the
    /// object is drawn.
    pub fn add_lights(&mut self, visual: u32, lights: &[ac1_assets::formats::SetupLight]) {
        let lights: Vec<_> = lights
            .iter()
            .map(|light| {
                let c = light.color;
                let rgb = [(c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF].map(|v| v as f32 / 255.0);
                ac2_assets::Ac2FxLight {
                    offset: Vec3::from(light.offset.origin),
                    source: ac2_assets::Ac2LightSource {
                        range: light.falloff,
                        intensity: light.intensity / 100.0,
                        diffuse: rgb,
                        specular: rgb,
                        ..ac2_assets::Ac2LightSource::default()
                    },
                    lifespan: 0.0,
                    fade_time: 0.0,
                }
            })
            .collect();
        if !lights.is_empty() {
            self.lights.insert(visual, lights);
        }
    }

    /// Runs the emitters of `frame`'s AC1 objects to `now` and adds their particles to `frame`
    /// (objects and tints), facing the camera.
    pub fn frame(
        &mut self,
        view: &mut ClientView,
        frame: &mut ClientFrame,
        now: f64,
        block_size: f32,
    ) {
        self.switch_lights(view, frame);
        frame
            .objects
            .retain(|o| !(FIRST_PARTICLE..FIRST_PARTICLE + MAX_PARTICLES).contains(&o.id));
        let eye = frame
            .camera
            .map(|camera| view.world_point(camera.cell, camera.origin));
        let sources: Vec<(u64, Script, f32)> = frame
            .objects
            .iter()
            .filter_map(|o| {
                let (script, scale) = self.scripts.get(&o.visual?)?;
                Some((o.id, Arc::clone(script), *scale))
            })
            .collect();
        self.running
            .retain(|id, _| sources.iter().any(|(source, _, _)| source == id));
        let mut particles = Vec::new();
        for (id, script, particle_scale) in sources {
            let Some(placement) = view.object_placement(id) else {
                continue;
            };
            // The hooks' offsets are in the parts' unscaled frames; the drawn parts are scaled.
            let scale = placement.scale.x;
            let emitters = self.running.entry(id).or_insert_with(|| {
                script
                    .iter()
                    .map(|(hook, info)| {
                        self.seed += 1;
                        let mut hook = *hook;
                        hook.offset.origin = hook.offset.origin.map(|v| v * scale);
                        Emitter::new(*info, hook, self.seed, now + hook.start)
                    })
                    .collect()
            });
            for emitter in emitters.iter_mut() {
                if now < emitter.hook.start {
                    continue;
                }
                let bone = placement.bones.get(emitter.hook.part as usize);
                let part = PartFrame {
                    origin: placement.frame.origin
                        + placement.frame.orientation
                            * bone.map_or(Vec3::ZERO, |b| b.offset * placement.scale),
                    rotation: placement.frame.orientation
                        * bone.map_or(Quat::IDENTITY, |b| b.rotation),
                };
                emitter.update(now, part);
                particles.extend(emitter.draws(now, part).map(|mut draw| {
                    draw.scale *= particle_scale;
                    draw
                }));
            }
        }
        for (index, particle) in particles.into_iter().enumerate() {
            let id = FIRST_PARTICLE + index as u64;
            if id >= FIRST_PARTICLE + MAX_PARTICLES {
                break;
            }
            let Some(&visual) = self.visuals.get(&particle.gfx_obj) else {
                continue;
            };
            let p = particle.position;
            let bx = (p.x / block_size).floor().clamp(0.0, 254.0);
            let by = (p.y / block_size).floor().clamp(0.0, 254.0);
            let cell = ((bx as u32) << 24) | ((by as u32) << 16) | 1;
            let q = eye.map_or(Quat::IDENTITY, |eye| facing(p, eye));
            frame.fx_looks.tints.insert(
                id,
                [1.0, 1.0, 1.0, (1.0 - particle.translucency).clamp(0.0, 1.0)],
            );
            frame.objects.push(ObjectFrame {
                id,
                visual: Some(visual),
                setup: None,
                name: None,
                stack_name: None,
                cell,
                origin: [p.x - bx * block_size, p.y - by * block_size, p.z],
                rotation: [q.w, q.x, q.y, q.z],
                heading_degrees: None,
                in_world: true,
                lost_cell: None,
                active: true,
                velocity: [0.0; 3],
                contact: false,
                mode: None,
                animated_mode: None,
                motion: [0.0; 3],
                holder: None,
                fx_follow: None,
                container: None,
                wielder: None,
                appearance: Arc::new(Appearance::default()),
                pose: Vec::new(),
                anim_scale: [1.0; 4],
                body_scale: [particle.scale; 3],
                adaptive_degrade: false,
            });
        }
    }
}

impl Ac1Fx {
    /// Turns the Setup lights of `frame`'s AC1 objects on as they appear and off as they go.
    fn switch_lights(&mut self, view: &mut ClientView, frame: &ClientFrame) {
        let drawn: HashMap<u64, u32> = frame
            .objects
            .iter()
            .filter_map(|o| Some((o.id, o.visual?)))
            .filter(|(_, visual)| self.lights.contains_key(visual))
            .collect();
        let gone: Vec<u64> = self
            .lit
            .iter()
            .copied()
            .filter(|id| !drawn.contains_key(id))
            .collect();
        for id in gone {
            view.remove_object_lights(id);
            self.lit.remove(&id);
        }
        for (id, visual) in drawn {
            if self.lit.insert(id) {
                for light in &self.lights[&visual] {
                    view.start_object_light(id, *light);
                }
            }
        }
    }
}

/// The turn that faces a particle's quad (its local -y) towards `eye`, upright.
fn facing(at: Vec3, eye: Vec3) -> Quat {
    let Some(away) = (at - eye).try_normalize() else {
        return Quat::IDENTITY;
    };
    let up = (Vec3::Z - away * away.dot(Vec3::Z))
        .try_normalize()
        .unwrap_or(Vec3::X);
    let right = away.cross(up);
    Quat::from_mat4(&Mat4::from_cols(
        right.extend(0.0),
        away.extend(0.0),
        up.extend(0.0),
        Vec4::W,
    ))
    .normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_particle_quad_faces_the_eye_upright() {
        let q = facing(Vec3::new(0.0, 10.0, 0.0), Vec3::ZERO);
        // Its -y points back at the eye and its z stays up.
        assert!((q * -Vec3::Y - Vec3::new(0.0, -1.0, 0.0)).length() < 1e-5);
        assert!((q * Vec3::Z - Vec3::Z).length() < 1e-5);
        let q = facing(Vec3::new(5.0, 0.0, 0.0), Vec3::ZERO);
        assert!((q * -Vec3::Y - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-5);
    }
}
