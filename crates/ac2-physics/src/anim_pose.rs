//! `CPhysicsObj::UpdateAnimation` and the `DoAnimation` that ends `ModeTracker::Update`: the
//! body's bones posed from its trackers (`CPartGroup::DoAnimation` -> `CPartArray::DoAnimation`
//! for the body's one part group).

use ac2_anim::{
    blend_transform, Ac2AnimEvent, Ac2AnimatorContext, Ac2BoneTransform, Ac2Compositor,
    Ac2ImpulseTrigger, Ac2PoseInput, Ac2SliderTracker, Ac2Transform, Ac2TriggerAction,
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
    /// The animation update step of `CPhysicsObj::UpdateAnimation`: 0 (every call) for the
    /// player and for bodies nearer than a quarter of the minimum visible distance, rising
    /// linearly to 0.25 s at it.
    fn animation_update_step(&self) -> f32 {
        let (view, min_visible) = (ld(self.view_distance), ld(self.min_visible_distance));
        let quarter = st(min_visible * ld(0.25_f32));
        let mut step = 0.0;
        if ld(quarter) < view {
            step = if min_visible <= view {
                0.25
            } else {
                st((view - ld(quarter)) * ld(0.25_f32) / (min_visible - ld(quarter)))
            };
        }
        if self.anim.is_player() {
            step = 0.0;
        }
        step
    }

    /// `CPhysicsObj::UpdateAnimation` (look-at and the FX manager are not modelled). Unless
    /// animation is frozen: an update frame comes once the update step has passed since the last
    /// one while the body is within its minimum visible distance (always the first call); it
    /// advances the sliders and the FX scripts by the time since the last update frame and
    /// refreshes the bones. Every call adds the half-step of local acceleration to the animation
    /// velocity and runs `ModeTracker::Update`, whose `DoAnimation` collects the tracks'
    /// triggers; then the `TriggerUpdate` fires (FX and weenie hooks become events, impulses
    /// move the body), its effect rolls drawn from `random`.
    pub(crate) fn update_animation(
        &mut self,
        quantum: f32,
        local_time: f64,
        random: &mut ac2_core::Random,
    ) {
        if self.is_animation_frozen() {
            return;
        }
        let step = self.animation_update_step();
        let elapsed = st(local_time - self.last_animation_update_time);
        let refresh = (ld(self.view_distance) <= ld(self.min_visible_distance)
            && ld(step) <= ld(elapsed))
            || self.last_animation_update_time == 0.0;
        if refresh {
            self.last_animation_update_time = local_time;
            self.anim.update_sliders(elapsed, self.wind_magnitude);
            self.anim.update_fx_scripts(elapsed);
        }
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
            self.do_animation(refresh, local_time);
            self.anim.modes.add_triggers();
            self.sync_mode();
        }
        // `LookAtManager::Update` (`CPhysicsObj::UpdateAnimation`, before the slider tracker).
        self.update_look_at(quantum);
        for action in self.anim.modes.triggers_mut().fire(random) {
            match action {
                Ac2TriggerAction::Event(event) => self.anim.push_events([event]),
                Ac2TriggerAction::Impulse(impulse) => self.fire_impulse(&impulse),
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

    /// `PHYSICSOBJREP::DoAnimation(refreshBones)`: the bones are posed only when refreshing
    /// (`CPartArray::AddAnimation` adds to the compositor only then); each track's reached
    /// triggers are added to it either way (`DBAnimator::GetTriggers`).
    fn do_animation(&mut self, refresh: bool, local_time: f64) {
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
            bones_at,
            prev_bones,
            prev_bones_at,
            ..
        } = &mut self.anim;
        let (Some(setup), Some(table)) = (setup.as_ref(), table.as_ref()) else {
            return;
        };
        let mut ctx = PoseContext {
            sliders,
            anim_velocity,
            effects_scale,
            vdesc_scale: *vdesc_scale,
            omega,
            terrain,
        };
        if refresh {
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
            let fresh = Ac2Compositor::pose(
                modes,
                |key| animators.get(&key).map(|animator| &**animator),
                &mut ctx,
                &input,
            );
            // The refresh before this one is what the drawn pose blends from.
            *prev_bones = std::mem::replace(bones, fresh);
            *prev_bones_at = std::mem::replace(bones_at, local_time);
        }
        modes.visit_tracks_mut(|state, track| {
            if let Some(animator) = animators.get(&track.anim_key) {
                animator.triggers(
                    &ctx,
                    state,
                    track.ctime,
                    !track.is_action,
                    &mut track.triggers,
                );
            }
        });
        let events = self.anim.sliders.take_events();
        self.anim.push_events(events);
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

    /// The bones at `now` drawn between the last two bone refreshes: `t` runs from 0 at the last
    /// refresh to 1 one refresh interval later (clamped), and the pose is `blend_transform`
    /// from the refresh before to the last one. Falls back to [`Self::bone_transforms`] when
    /// there are not two refreshes of the same skeleton yet.
    pub(crate) fn blended_bone_transforms(&self, now: f64) -> Vec<Ac2Transform> {
        let current = self.bone_transforms();
        let anim = &self.anim;
        let span = anim.bones_at - anim.prev_bones_at;
        if !(span > 0.0) || anim.prev_bones.is_empty() || anim.prev_bones.len() != anim.bones.len()
        {
            return current;
        }
        let t = ((now - anim.bones_at) / span).clamp(0.0, 1.0) as f32;
        anim.prev_bones
            .iter()
            .zip(&anim.bones)
            .map(|(from, to)| blend_transform(&from.to_transform(), &to.to_transform(), t))
            .collect()
    }

    /// Installs the body's behaviour table (`PHYSICSOBJREP::SetupModeTracker`); the body stays
    /// the player or not.
    pub(crate) fn set_animation(&mut self, animation: BodyAnimation) {
        self.anim = AnimRep::new(animation, self.anim.is_player());
        self.sync_mode();
    }

    pub(crate) fn take_animation_events(&mut self) -> Vec<Ac2AnimEvent> {
        self.anim.take_events()
    }
}
