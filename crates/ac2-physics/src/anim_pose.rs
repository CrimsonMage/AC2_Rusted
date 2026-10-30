//! `CPhysicsObj::UpdateAnimation` and the `DoAnimation` that ends `ModeTracker::Update`: the
//! body's bones posed from its trackers (`CPartGroup::DoAnimation` -> `CPartArray::DoAnimation`
//! for the body's one part group).

use ac2_anim::{
    blend_transform, Ac2AnimEvent, Ac2AnimatorContext, Ac2BoneTransform, Ac2Compositor,
    Ac2ImpulseTrigger, Ac2PoseInput, Ac2SliderInterpolator, Ac2SliderTracker, Ac2Transform,
    Ac2TriggerAction,
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
    /// The global `Random` the animators roll (`AMRandom::Animate`).
    random: &'a mut ac2_core::Random,
    /// The setup's bones (`AnimSetup::IsBoneType` compares their type ids).
    bones: &'a [ac2_anim::Ac2Bone],
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
    fn is_bone_type(&self, bone: usize, ty: u32) -> bool {
        self.bones.get(bone).is_some_and(|b| b.name == ty)
    }
    fn slider_interpolator(&self, slider: u32) -> Ac2SliderInterpolator {
        self.sliders.interpolator(slider)
    }
    fn activate_slider_interpolator(&mut self, slider: u32, start: f32, end: f32, length: f32) {
        self.sliders
            .activate_interpolator(slider, start, end, length);
    }
    fn roll_dice(&mut self, lower: i32, upper: i32) -> i32 {
        self.random.roll_dice(lower, upper)
    }
    fn roll_dice_f32(&mut self, lower: f32, upper: f32) -> f32 {
        self.random.roll_dice_f32(lower, upper)
    }
}

fn array(v: Vector) -> [f32; 3] {
    [v.x, v.y, v.z]
}

/// `CPartArray::DoAnimation` of one part group: the group's animators add the trackers' tracks
/// (`CPartArray::AddAnimation`: each track's reached triggers always, its pose only when
/// refreshing), and the compositor poses the group's bones from them. `None` when not refreshing.
fn pose_group(
    modes: &mut ac2_anim::Ac2ModeTracker,
    table: &ac2_anim::Ac2BehaviorTable,
    animators: &std::collections::HashMap<u32, std::sync::Arc<ac2_anim::Ac2DbAnimator>>,
    setup: &ac2_anim::Ac2Setup,
    ctx: &mut PoseContext<'_>,
    refresh: bool,
) -> Option<Vec<Ac2BoneTransform>> {
    modes.visit_tracks_mut(|_, t| {
        if animators.contains_key(&t.anim_key) && setup.bones.len() == 8 {
            eprintln!("ZZ track {:x} w {} layer", t.anim_key, t.weight)
        }
    });
    let fresh = refresh.then(|| {
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
        // `PHYSICSOBJREP::SetupCompositor`: the table's layer count and tweak layers.
        let input = Ac2PoseInput {
            layers: table.layer_count(),
            tweak_layers: &table.tweak_layers,
            roots: &roots,
            rest: &rest,
        };
        Ac2Compositor::pose(
            modes,
            |key| animators.get(&key).map(|animator| &**animator),
            &mut *ctx,
            &input,
        )
    });
    modes.visit_tracks_mut(|state, track| {
        if let Some(animator) = animators.get(&track.anim_key) {
            animator.triggers(
                &*ctx,
                state,
                track.ctime,
                !track.is_action,
                &mut track.triggers,
            );
        }
    });
    fresh
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

    /// `CPhysicsObj::UpdateAnimation`. Unless animation is frozen: an update frame comes once
    /// the update step has passed since the last one while the body is within its minimum
    /// visible distance (always the first call); it runs the look-at manager, advances the
    /// sliders and the FX scripts by the time since the last update frame and refreshes the
    /// bones (`PHYSICSOBJREP::UpdateFX`, the FX manager's `UseTime`, is the client's own step).
    /// Every call adds the half-step of local acceleration to the animation
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
            // `LookAtManager::Update` runs in the update frame, before the slider tracker.
            self.update_look_at(elapsed);
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
            self.do_animation(refresh, local_time, random);
            self.anim.modes.add_triggers();
            self.sync_mode();
        }
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
        self.set_velocity_culled(v);
        self.anim_velocity = v;
    }

    /// `PHYSICSOBJREP::DoAnimation(refreshBones)`: the bones are posed only when refreshing
    /// (`CPartArray::AddAnimation` adds to the compositor only then); each track's reached
    /// triggers are added to it either way (`DBAnimator::GetTriggers`).
    fn do_animation(&mut self, refresh: bool, local_time: f64, random: &mut ac2_core::Random) {
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
            child_groups,
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
            random,
            bones: &setup.bones,
        };
        let root_bones = pose_group(modes, table, animators, setup, &mut ctx, refresh);
        if let Some(fresh) = root_bones {
            // The refresh before this one is what the drawn pose blends from.
            *prev_bones = std::mem::replace(bones, fresh);
            *prev_bones_at = std::mem::replace(bones_at, local_time);
        }
        // `CPartGroup::DoAnimation` runs the same trackers through each child group, in order.
        for child in child_groups.iter_mut() {
            ctx.bones = &child.setup.bones;
            let fresh = pose_group(
                modes,
                table,
                &child.animators,
                &child.setup,
                &mut ctx,
                refresh,
            );
            if let Some(fresh) = fresh {
                child.prev_bones = std::mem::replace(&mut child.bones, fresh);
                child.prev_bones_at = std::mem::replace(&mut child.bones_at, local_time);
            }
        }
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
        let anim = &self.anim;
        blend_refreshes(
            self.bone_transforms(),
            (&anim.prev_bones, anim.prev_bones_at),
            (&anim.bones, anim.bones_at),
            now,
        )
    }

    /// `CPartArray::m_boneTransforms` of every child part group that animates with an `AnimMap`
    /// of its own (`CPartGroup::DoAnimation` over the children), by group key: the bones as the
    /// last step left them, or the Setup's default placement before the first step.
    pub(crate) fn child_bone_transforms(&self) -> Vec<(u32, Vec<Ac2Transform>)> {
        self.anim
            .child_groups
            .iter()
            .map(|group| (group.key, group.current()))
            .collect()
    }

    /// [`Self::child_bone_transforms`] drawn at `now` between the last two refreshes, as
    /// [`Self::blended_bone_transforms`] draws the root's.
    pub(crate) fn blended_child_bone_transforms(&self, now: f64) -> Vec<(u32, Vec<Ac2Transform>)> {
        self.anim
            .child_groups
            .iter()
            .map(|group| {
                let blended = blend_refreshes(
                    group.current(),
                    (&group.prev_bones, group.prev_bones_at),
                    (&group.bones, group.bones_at),
                    now,
                );
                (group.key, blended)
            })
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

/// `current` unless two refreshes of one skeleton exist, else the pose `now` between them.
fn blend_refreshes(
    current: Vec<Ac2Transform>,
    (prev, prev_at): (&[Ac2BoneTransform], f64),
    (last, last_at): (&[Ac2BoneTransform], f64),
    now: f64,
) -> Vec<Ac2Transform> {
    let span = last_at - prev_at;
    if !(span > 0.0) || prev.is_empty() || prev.len() != last.len() {
        return current;
    }
    let t = ((now - last_at) / span).clamp(0.0, 1.0) as f32;
    prev.iter()
        .zip(last)
        .map(|(from, to)| blend_transform(&from.to_transform(), &to.to_transform(), t))
        .collect()
}
