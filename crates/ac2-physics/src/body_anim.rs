//! `PhysicsBody`'s animation entry points: the behaviour table, the animation step, the mode,
//! behaviour and slider calls the weenie and the server make (`CPhysicsObj::RecvEvt_*`), and
//! whether the body is the player.

use ac2_anim::{Ac2AnimEvent, Ac2BehaviorParams};

use crate::anim_rep::BodyAnimation;
use crate::moveto::MovementParameters;
use crate::physics_obj::DescSlider;

use super::PhysicsBody;

impl PhysicsBody {
    /// `CPhysicsObj::IsThePlayer` (`m_id == ICPhysicsObj::player_id`). Bodies start as other
    /// objects; the client marks its own. Only the player runs move-tos, jumps standing through
    /// the jump behaviour, locks its actions for behaviours that ask, updates its animation in
    /// full every step, and keeps its client-controlled sliders from the server.
    pub fn set_player(&mut self, player: bool) {
        self.object.anim.set_player(player);
    }

    /// `CPhysicsObj::IsThePlayer`.
    pub fn is_player(&self) -> bool {
        self.object.anim.is_player()
    }

    /// Animate the body with its behaviour table (`PHYSICSOBJREP::SetupModeTracker`): element
    /// and mode changes then wait for the animation's mode transitions, and `update_movement`
    /// starts and stops the movement behaviours.
    pub fn set_animation(&mut self, animation: BodyAnimation) {
        self.object.set_animation(animation);
    }

    /// One animation step (`CPhysicsObj::UpdateAnimation`) of `quantum` seconds, at the steps
    /// an [`AnimationClock`](crate::AnimationClock) gives, at `local_time`
    /// (`Timer::local_time`, the clock's `now`). Bodies past a quarter of their minimum visible
    /// distance refresh their bones and sliders less often; see [`Self::set_view_distance`].
    pub fn update_animation(&mut self, quantum: f32, local_time: f64) {
        self.object.update_animation(quantum, local_time);
    }

    /// `CPhysicsObj::UpdateViewerDistance`: the camera's distance to the body
    /// (`m_fViewDistance`, 1 until set; negative distances are 0).
    pub fn set_view_distance(&mut self, distance: f32) {
        self.object.view_distance = if distance < 0.0 { 0.0 } else { distance };
    }

    /// `m_fMinVisibleDistance`: the distance past which the body is not drawn (what the draw
    /// reports from its degrades; `f32::MAX` until set, so the animation updates fully).
    pub fn set_min_visible_distance(&mut self, distance: f32) {
        self.object.min_visible_distance = distance;
    }

    /// `CPhysicsObj::DoMode`: animate toward `mode`; `context` (0 for none) is told through a
    /// [`Ac2AnimEvent::ModeDone`] when it is reached, replaced or has no path.
    /// `RecvEvt_DoMode` is `do_mode(0, mode)`, as is the `PhysicsDesc` mode at create.
    pub fn do_mode(&mut self, context: u32, mode: u32) -> bool {
        self.object.do_mode(context, mode)
    }

    /// `CPhysicsObj::SetMode` (`RecvEvt_SetMode`): switch to `mode` at once.
    pub fn set_mode(&mut self, mode: u32) -> bool {
        self.object.set_mode(mode)
    }

    /// `CPhysicsObj::DoBehavior` (a weenie or server behaviour; `RecvEvt_DoBehavior` runs a
    /// copy whose context is 1). The params' mode is entered first. Returns whether the
    /// tracker started it.
    pub fn do_behavior(&mut self, params: &Ac2BehaviorParams) -> bool {
        self.object.do_behavior(params)
    }

    /// `CPhysicsObj::StopBehavior` (`RecvEvt_StopBehavior`).
    pub fn stop_behavior(&mut self, behavior: u32) {
        self.object.stop_behavior(behavior);
    }

    /// `CPhysicsObj::ReleaseBehavior`: release a held behaviour, playing `params`' behaviour in
    /// its place when it names one. `RecvEvt_ReleaseBehavior` runs a copy whose context is 1.
    pub fn release_behavior(&mut self, behavior: u32, params: &Ac2BehaviorParams) -> bool {
        self.object.release_behavior(behavior, params)
    }

    /// `CPhysicsObj::StopLayer` (`RecvEvt_StopLayer`).
    pub fn stop_layer(&mut self, layer: u32) {
        self.object.stop_layer(layer);
    }

    /// `CPhysicsObj::StopAllWeenieBehaviors` (`RecvEvt_StopAllWeenieBehaviors`).
    pub fn stop_all_weenie_behaviors(&mut self) -> bool {
        self.object.stop_all_weenie_behaviors()
    }

    /// `CPhysicsObj::StopMoveToCancelsBehaviors` (`RecvEvt_StopMoveToCancelsBehaviors`).
    pub fn stop_move_to_cancels_behaviors(&mut self) {
        self.object.stop_move_to_cancels_behaviors();
    }

    /// `CPhysicsObj::DoSlider`: move `slider` to `pos` over `time` seconds; `context` (0 for
    /// none) gets a [`Ac2AnimEvent::SliderDone`].
    pub fn do_slider(&mut self, context: u32, slider: u32, pos: f32, time: f32) -> bool {
        self.object.do_slider(context, slider, pos, time)
    }

    /// `CPhysicsObj::RecvEvt_DoSlider`: `do_slider(0, ..)`, except that the player keeps the
    /// sliders its `CharTemplate` leaves to the client (false then).
    pub fn recv_do_slider(&mut self, slider: u32, value: f32, time: f32) -> bool {
        self.object.recv_do_slider(slider, value, time)
    }

    /// The animation half of `CPhysicsObj::set_description` (a `PhysicsDesc` at create or
    /// update): its mode, behaviours and sliders.
    pub fn set_description_animation(
        &mut self,
        mode: u32,
        behaviors: &[Ac2BehaviorParams],
        sliders: &[DescSlider],
    ) {
        self.object
            .set_description_animation(mode, behaviors, sliders);
    }

    /// `CPhysicsObj::RecvEvt_MoveTo`: for the player only, the behaviours a move-to cancels stop
    /// and the move-to starts (`PHYSICSOBJREP::MoveToEvent`). The move-to stamp is the
    /// caller's. Returns whether it started (false for any other body).
    pub fn recv_move_to(&mut self, params: MovementParameters) -> bool {
        if !self.is_player() {
            return false;
        }
        self.object.stop_move_to_cancels_behaviors();
        self.move_to(params)
    }

    /// `CPhysicsObj::GetSliderCurrentValue`.
    pub fn slider_value(&self, slider: u32) -> f32 {
        self.object.anim.sliders.value(slider)
    }

    /// `CPhysicsObj::GetSliderDesiredValue`.
    pub fn slider_desired_value(&self, slider: u32) -> f32 {
        self.object.anim.sliders.desired(slider)
    }

    /// `PHYSICSOBJREP::m_animcolor` (r, g, b, a in `[0, 1]`): the colour sliders, white
    /// until moved.
    pub fn anim_color(&self) -> [f32; 4] {
        self.object.anim.anim_color
    }

    /// `PHYSICSOBJREP::m_animopacity` in `[0, 1]`: the opacity slider, 1 until moved.
    pub fn anim_opacity(&self) -> f32 {
        self.object.anim.anim_opacity
    }

    /// `CPhysicsObj::GetMode`: the mode the animation has reached.
    pub fn mode(&self) -> u32 {
        self.object.anim.modes.mode()
    }

    /// `PHYSICSOBJREP::GetElement`: the element the animation has reached.
    pub fn current_element(&self) -> u32 {
        self.object.current_element()
    }

    /// `PHYSICSOBJREP::IsDoingBehavior`.
    pub fn is_doing_behavior(&self, behavior: u32) -> bool {
        self.object.anim.modes.is_doing_behavior(behavior)
    }

    /// The body's bones posed by the last refreshed animation step, in model space (relative
    /// to the body's frame, AC axes); the Setup rest pose before the first step, empty without
    /// a Setup. Part `i` of the Setup is drawn with bone `i`'s transform.
    pub fn posed_parts(&self) -> Vec<ac2_anim::Ac2Transform> {
        self.object.posed_parts()
    }

    /// The body's bones relative to their parents (`CPartArray::m_boneTransforms`), what the
    /// renderer builds the bone and deform matrices from (`CPartArray::UpdateBones`); empty
    /// without a Setup.
    pub fn bone_transforms(&self) -> Vec<ac2_anim::Ac2Transform> {
        self.object.bone_transforms()
    }

    /// The behaviour, mode and slider callbacks and the FX the animation raised since the last
    /// call, in order.
    pub fn take_animation_events(&mut self) -> Vec<Ac2AnimEvent> {
        self.object.take_animation_events()
    }
}
