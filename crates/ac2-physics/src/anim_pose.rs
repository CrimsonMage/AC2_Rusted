//! `CPhysicsObj::UpdateAnimation` and the `DoAnimation` that ends `ModeTracker::Update`: the
//! body's bones posed from its trackers (`CPartGroup::DoAnimation` -> `CPartArray::DoAnimation`
//! for the body's one part group).

use ac2_anim::{
    Ac2AnimEvent, Ac2AnimatorContext, Ac2BoneTransform, Ac2Compositor, Ac2ImpulseTrigger,
    Ac2PoseInput, Ac2SliderTracker, Ac2Transform,
};

use crate::anim_rep::{AnimRep, BodyAnimation};
use crate::fpu::{ld, st};
use crate::physics_obj::PhysicsObj;
use crate::vector::Vector;

/// `PHYSICSOBJREP` as the animators' `AnimationObj` while posing (`AMHuman` sets sliders).
struct PoseContext<'a> {
    sliders: &'a mut Ac2SliderTracker,
    anim_velocity: [f32; 3],
    effects_scale: [f32; 3],
    vdesc_scale: [f32; 3],
    omega: [f32; 3],
    terrain: u32,
}

impl Ac2AnimatorContext for PoseContext<'_> {
    fn slider(&self, slider: u32) -> f32 {
        self.sliders.value(slider)
    }
    fn do_slider(&mut self, slider: u32, pos: f32, time: f32) {
        self.sliders.do_slider(0, slider, pos, time);
    }
    fn anim_velocity(&self) -> [f32; 3] {
        self.anim_velocity
    }
    fn effects_scale(&self) -> [f32; 3] {
        self.effects_scale
    }
    fn vdesc_scale(&self) -> [f32; 3] {
        self.vdesc_scale
    }
    fn omega(&self) -> [f32; 3] {
        self.omega
    }
    fn terrain_type(&self) -> u32 {
        self.terrain
    }
}

fn array(v: Vector) -> [f32; 3] {
    [v.x, v.y, v.z]
}

impl PhysicsObj {
    /// `CPhysicsObj::UpdateAnimation` for an object updated every animation tick (the player;
    /// the distance-degraded step, look-at, FX scripts and triggers are not modelled): the
    /// slider tracker, the half-step of local acceleration added to the animation velocity, then
    /// `ModeTracker::Update` and the pose when the body has a behaviour table.
    pub(crate) fn update_animation(&mut self, quantum: f32) {
        self.anim.update_sliders(quantum);
        let a = self.position.frame.global_to_local_vec(self.acceleration);
        let q = ld(quantum);
        let half = ld(0.5_f32);
        let cz = ld(st(ld(a.z) * half));
        let ax1 = ld(st(ld(a.x) * half * q));
        let by = ld(a.y) * half * q;
        let ax = st(ax1 * q);
        let ay = st(by * q);
        let az = cz * q * q;
        let v = self.anim_velocity;
        self.anim_velocity =
            Vector::new(st(ld(ax) + ld(v.x)), st(ld(ay) + ld(v.y)), st(az + ld(v.z)));
        if self.anim.can_animate() {
            self.anim
                .with_modes(|modes, source| modes.update(quantum, source));
            let impulses = self.do_animation();
            self.sync_mode();
            for impulse in impulses {
                self.fire_impulse(&impulse);
            }
        }
    }

    /// The impulse loop of `CPhysicsObj::UpdateAnimation`: a jump impulse is `Jump`; any other
    /// leaves the ground into the air element at the impulse velocity (`set_local_velocity`,
    /// which retail hands to `set_velocity` unrotated).
    fn fire_impulse(&mut self, impulse: &Ac2ImpulseTrigger) {
        if impulse.jump {
            self.jump();
            return;
        }
        self.transient_state &= !crate::physics_obj::CONTACT_TS;
        self.do_element(crate::motion_interp::ELEMENT_AIR);
        let [x, y, z] = impulse.impulse;
        let v = Vector::new(x, y, z);
        let current = self.velocity;
        let moved = |a: f32, b: f32| (ld(a) - ld(b)).abs() >= ld(crate::fpu::F_EPSILON);
        if moved(v.x, current.x) || moved(v.y, current.y) || moved(v.z, current.z) {
            self.velocity = v;
            self.cull_velocity();
        }
        self.anim_velocity = v;
    }

    /// `PHYSICSOBJREP::DoAnimation(refreshBones)`, then the impulse triggers the tracks reached
    /// (`DBAnimator::GetTriggers` in `CPartArray::AddAnimation`, `BehaviorTracker::AddTriggers`):
    /// the ones not fired yet, now marked fired.
    fn do_animation(&mut self) -> Vec<Ac2ImpulseTrigger> {
        let terrain = self.motion.terrain_type();
        let (anim_velocity, effects_scale, omega) = (
            array(self.anim_velocity),
            array(self.scale),
            array(self.omega),
        );
        let AnimRep {
            modes,
            sliders,
            animators,
            table,
            setup,
            vdesc_scale,
            bones,
            ..
        } = &mut self.anim;
        let (Some(setup), Some(table)) = (setup.as_ref(), table.as_ref()) else {
            return Vec::new();
        };
        let rest: Vec<Ac2BoneTransform> = setup
            .rest_locals()
            .iter()
            .map(Ac2BoneTransform::from_transform)
            .collect();
        let roots: Vec<bool> = setup
            .bones
            .iter()
            .map(|bone| bone.parent.is_none())
            .collect();
        let input = Ac2PoseInput {
            layers: table.layer_count(),
            tweak_layers: &table.tweak_layers,
            roots: &roots,
            rest: &rest,
        };
        let mut ctx = PoseContext {
            sliders,
            anim_velocity,
            effects_scale,
            vdesc_scale: *vdesc_scale,
            omega,
            terrain,
        };
        *bones = Ac2Compositor::pose(
            modes,
            |key| animators.get(&key).map(|animator| &**animator),
            &mut ctx,
            &input,
        );
        let mut pending = Vec::new();
        modes.visit_tracks_mut(|state, track| {
            let Some(animator) = animators.get(&track.anim_key) else {
                return;
            };
            let mut reached = Vec::new();
            animator.impulses(&ctx, state, track.ctime, !track.is_action, &mut reached);
            for impulse in reached {
                if !track.impulses.iter().any(|(known, _)| *known == impulse) {
                    track.impulses.push((impulse, false));
                }
            }
            for (impulse, triggered) in &mut track.impulses {
                if !*triggered {
                    *triggered = true;
                    pending.push(*impulse);
                }
            }
        });
        let events = self.anim.sliders.take_events();
        self.anim.push_events(events);
        pending
    }

    /// The posed bones in model space (`CSetup` hierarchy over the local bone transforms), or
    /// the Setup's rest pose before the first animation step.
    pub(crate) fn posed_parts(&self) -> Vec<Ac2Transform> {
        let Some(setup) = self.anim.setup.as_ref() else {
            return Vec::new();
        };
        if self.anim.bones.len() != setup.bones.len() {
            return setup.rest_pose();
        }
        let locals: Vec<Ac2Transform> = self
            .anim
            .bones
            .iter()
            .map(Ac2BoneTransform::to_transform)
            .collect();
        setup.world_pose(&locals)
    }

    /// `CPartArray::m_boneTransforms`: each bone relative to its parent as the last animation
    /// step left it, or the Setup's default placement before the first step.
    pub(crate) fn bone_transforms(&self) -> Vec<Ac2Transform> {
        let Some(setup) = self.anim.setup.as_ref() else {
            return Vec::new();
        };
        if self.anim.bones.len() != setup.bones.len() {
            return setup.rest_locals();
        }
        self.anim
            .bones
            .iter()
            .map(Ac2BoneTransform::to_transform)
            .collect()
    }

    /// Installs the body's behaviour table (`PHYSICSOBJREP::SetupModeTracker`).
    pub(crate) fn set_animation(&mut self, animation: BodyAnimation, player: bool) {
        self.anim = AnimRep::new(animation, player);
        self.sync_mode();
    }

    pub(crate) fn take_animation_events(&mut self) -> Vec<Ac2AnimEvent> {
        self.anim.take_events()
    }
}
