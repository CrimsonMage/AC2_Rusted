//! `PhysicsBody`'s retail accessors: the `CPhysicsObj` getters and state-bit tests, and the
//! `PHYSICSOBJREP` forwarders that read the body's motion interpreter (`CMotionInterp`) and
//! its physics object. Retail splits them across the object, its representation and the part
//! array; here one body holds all three, so each is one method.

use crate::fpu::ld;
use crate::heading::Heading;
use crate::physics_obj::{PhysicsObj, LANDSCAPE_CONTACT_TS, SLIDING_TS};
use crate::physics_obj_contact::WATER_CONTACT_TS;
use crate::position::Position;
use crate::terrain::Terrain;
use crate::vector::{Plane, Sphere, Vector};

use super::PhysicsBody;

/// `PhysicsState` bits `CPhysicsObj`'s setters and getters test.
const REPORT_COLLISIONS_PS: u32 = 0x4;
const IGNORE_COLLISIONS_PS: u32 = 0x8;
const MISSILE_PS: u32 = 0x20;
const ALIGN_PATH_PS: u32 = 0x80;
const HAS_PHYSICS_BSP_PS: u32 = 0x1000;
const IN_WORKSPACE_PS: u32 = 0x1000_0000;
/// `CPhysicsObj::IsMoving`: the squared speed above which a body counts as moving.
const MOVING_SPEED_SQUARED: f32 = 0.0002;

impl PhysicsBody {
    /// `CPhysicsObj::IsSlidingDown`: in contact but not on walkable ground.
    pub fn is_sliding_down(&self) -> bool {
        self.object.has_contact() && !self.object.on_walkable()
    }

    /// `CPhysicsObj::get_local_velocity`: the world velocity in the body's own axes.
    pub fn local_velocity(&self) -> Vector {
        self.object
            .position
            .frame
            .global_to_local_vec(self.object.velocity)
    }

    /// `CPhysicsObj::GetStandardMotionValues` (`PHYSICSOBJREP::GetStandardMotionValues`): the
    /// peace-on-earth motion values, the static default without a motion table.
    pub fn standard_motion_values(&self) -> ac2_world::Ac2MotionValues {
        self.object.motion.standard_motion_values()
    }

    /// `CPhysicsObj::GetDesiredMotionValues`: the desired mode state's values, else the
    /// element's default mode, else the static default.
    pub fn desired_motion_values(&self) -> ac2_world::Ac2MotionValues {
        let (element, mode) = (self.object.desired_element(), self.object.motion.mode());
        self.object.motion.desired_motion_values(element, mode)
    }

    /// `CPhysicsObj::GetLastDoMotionValues` (`CMotionInterp::GetLastDoMotionValues`): the
    /// do-motion vector last applied.
    pub fn last_do_motion_values(&self) -> Vector {
        self.object.motion.last_do_motion()
    }

    /// The body's own total scale without any parent (`CPhysicsObj::GetTotalScale`'s first
    /// product): the representation's scale (the Setup's visual scale) times `GetScale`, each
    /// component stored as `float`. [`crate::ChildGraph::total_scale`] adds the parent chain.
    pub fn own_total_scale(&self) -> Vector {
        self.object.own_total_scale()
    }

    /// `CPhysicsObj::SetScale(Vector)` / `SetScaleInternal`: the object's scale. The containment
    /// radius follows it (`SetPhysicsContainmentRadius`), and a body with a physics BSP marks
    /// its state (`m_state` bit 29) while its total scale is not one in x or y.
    pub fn set_scale(&mut self, scale: Vector) {
        self.object.set_scale(scale);
    }

    /// `CPhysicsObj::SetScale(float)`: the same scale on every axis.
    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.object.set_scale(Vector::new(scale, scale, scale));
    }

    /// `CPhysicsObj::ApplyPropertyCollection` (`ReApplyPropertyCollection` forwards to it): the description's property groups applied to the
    /// body (the blocking mask, the physics material, the graphics state, the game-draw bit),
    /// the properties kept for `InqCurrentProperties`. True when every property applied.
    pub fn apply_property_collection(&mut self, groups: &[ac2_db::Ac2PropertyGroup]) -> bool {
        self.object.apply_property_collection(groups)
    }

    /// `CPhysicsObj::InqCurrentProperties` and `GetObjectProperties` (with `physics_material_id`): the properties `ApplyPropertyCollection` kept.
    pub fn current_properties(&self) -> &[ac2_db::Ac2PropertyGroup] {
        &self.object.property_info
    }

    /// `CPhysicsObj::InqCurrentProperties`: a copy of the cell material's collection
    /// (`m_pCellMaterial->m_collection`, set by `UpdateCellMaterial`); `None` (retail's 0) while
    /// the body has taken no cell material.
    pub fn cell_material(&self) -> Option<std::sync::Arc<crate::PhysicsMaterial>> {
        self.object.cell_material.clone()
    }

    /// `CPhysicsObj::m_physics_material`: the physics material DID the physics property named.
    pub fn physics_material_id(&self) -> u32 {
        self.object.physics_material
    }

    /// `CPhysicsObj::GetGraphicsState`: every bit of `state` is set.
    pub fn graphics_state(&self, state: u32) -> bool {
        self.object.graphics_state & state == state
    }

    /// `CPhysicsObj::SetGraphicsState`.
    pub fn set_graphics_state(&mut self, state: u32, enable: bool) {
        self.object.set_graphics_state(state, enable);
    }

    /// `CPhysicsObj::UpdateGravity`: the body's local gravity from a cell's material (the
    /// `0x80000001` vector property). See `PhysicsObj::update_gravity` for the result.
    pub fn update_gravity(&mut self, material: Option<&crate::PhysicsMaterial>) -> bool {
        self.object.update_gravity(material)
    }

    /// `CPhysicsObj::HandleEnterWorld`'s body part, run when a placement into the world
    /// succeeds: the opacity fades in, the element is picked for the new place and the body's
    /// movement, animation and misc activity begin.
    pub fn handle_enter_world(&mut self, world: &Terrain) {
        self.object.handle_enter_world(world);
    }

    /// `CPhysicsObj::clear_transient_states`, run when the body leaves the world.
    pub fn clear_transient_states(&mut self) {
        self.object.clear_transient_states();
    }

    /// `CPhysicsObj::animation_can_be_inactive`: the body is not in the world's object lists.
    pub fn animation_can_be_inactive(&self) -> bool {
        self.object
            .animation_can_be_inactive(self.object.world_id.is_some())
    }

    /// `CPhysicsObj::movement_can_be_inactive`; `has_parent` is the body's `HasParent` (the
    /// child graph holds the parent link) and `has_missile_manager` its `HasMissileManager`.
    pub fn movement_can_be_inactive(&self, has_parent: bool, has_missile_manager: bool) -> bool {
        self.object.movement_can_be_inactive(
            self.object.world_id.is_some(),
            has_parent,
            has_missile_manager,
        )
    }

    /// `CPhysicsObj::misc_can_be_inactive`; the caller answers `CCollisionManager::IsEmpty` and
    /// `CDetectionManager::IsActive`.
    pub fn misc_can_be_inactive(
        &self,
        collision_manager_empty: bool,
        detection_active: bool,
    ) -> bool {
        self.object.misc_can_be_inactive(
            self.object.world_id.is_some(),
            self.object.state & IN_WORKSPACE_PS != 0,
            collision_manager_empty,
            detection_active,
        )
    }

    #[cfg(test)]
    pub(crate) fn object_mut(&mut self) -> &mut PhysicsObj {
        &mut self.object
    }
}

/// `CPhysicsObj::m_state` bit 29: the object has a physics BSP and a total scale other than one.
const SCALED_PS: u32 = 0x2000_0000;

impl PhysicsObj {
    pub(crate) fn own_total_scale(&self) -> Vector {
        let s = ld(self.shape.scale);
        let one = |c: f32| crate::fpu::st(s * ld(c));
        Vector::new(one(self.scale.x), one(self.scale.y), one(self.scale.z))
    }

    pub(crate) fn set_scale(&mut self, scale: Vector) {
        self.scale = scale;
        if self.state & HAS_PHYSICS_BSP_PS != 0 {
            let total = self.own_total_scale();
            if (1.0 - ld(total.x)).abs() < 0.0002 && (1.0 - ld(total.y)).abs() < 0.0002 {
                self.state &= !SCALED_PS;
            } else {
                self.state |= SCALED_PS;
            }
        }
    }

    /// Sets or clears the `m_state` bit `bit` (`CPhysicsObj::set_movement_frozen`,
    /// `set_animation_frozen`, `set_edge_slide`, `set_missile`).
    pub(crate) fn set_state_bit(&mut self, bit: u32, on: bool) {
        if on {
            self.state |= bit;
        } else {
            self.state &= !bit;
        }
    }
}

impl PhysicsBody {
    /// `CPhysicsObj::GetDepth` and `PHYSICSOBJREP::GetDepth`: how deep in water the body is.
    pub fn depth(&self) -> f32 {
        self.object.depth
    }

    /// `PHYSICSOBJREP::HasWater` (`CPhysicsObj::m_bHasWater`).
    pub fn has_water(&self) -> bool {
        self.object.has_water
    }

    /// `CPhysicsObj::on_ground`: in contact and standing on a walkable surface.
    pub fn on_ground(&self) -> bool {
        self.object.has_contact() && self.object.on_walkable()
    }

    /// `CPhysicsObj::off_ground`: not in contact.
    pub fn off_ground(&self) -> bool {
        !self.object.has_contact()
    }

    /// `PHYSICSOBJREP::HasContact`.
    pub fn has_contact(&self) -> bool {
        self.object.has_contact()
    }

    /// `CPhysicsObj::has_sliding`: `SLIDING_TS`.
    pub fn has_sliding(&self) -> bool {
        self.object.transient_state & SLIDING_TS != 0
    }

    /// `CPhysicsObj::contact_is_water`: `WATER_CONTACT_TS`.
    pub fn contact_is_water(&self) -> bool {
        self.object.transient_state & WATER_CONTACT_TS != 0
    }

    /// `CPhysicsObj::m_transient_state & LANDSCAPE_CONTACT_TS`: the contact is the landscape's.
    pub fn contact_is_landscape(&self) -> bool {
        self.object.transient_state & LANDSCAPE_CONTACT_TS != 0
    }

    /// `PHYSICSOBJREP::GetContactPlane`: the plane the body stands on (its contact plane is
    /// kept as the last one found, so this is also valid in the air).
    pub fn contact_plane(&self) -> Plane {
        self.object.contact_plane
    }

    /// `CPhysicsObj::reports_collisions` (`m_state & REPORT_COLLISIONS_PS`).
    pub fn reports_collisions(&self) -> bool {
        self.object.state & REPORT_COLLISIONS_PS != 0
    }

    /// `CPhysicsObj::ignore_collisions` (`m_state & IGNORE_COLLISIONS_PS`).
    pub fn ignore_collisions(&self) -> bool {
        self.object.state & IGNORE_COLLISIONS_PS != 0
    }

    /// `CPhysicsObj::HasPhysicsBSP` (`m_state & HAS_PHYSICS_BSP_PS`).
    pub fn has_physics_bsp(&self) -> bool {
        self.object.state & HAS_PHYSICS_BSP_PS != 0
    }

    /// `CPhysicsObj::IsInWorkspace` (`m_state` bit 28).
    pub fn is_in_workspace(&self) -> bool {
        self.object.state & IN_WORKSPACE_PS != 0
    }

    /// `CPhysicsObj::SetInWorkspace`.
    pub fn set_in_workspace(&mut self, in_workspace: bool) {
        self.object.set_state_bit(IN_WORKSPACE_PS, in_workspace);
    }

    /// `CPhysicsObj::set_missile` (`MISSILE_PS`).
    pub fn set_missile(&mut self, missile: bool) {
        self.object.set_state_bit(MISSILE_PS, missile);
    }

    /// `CPhysicsObj::align_to_path` (`ALIGN_PATH_PS`).
    pub fn align_to_path(&mut self, align: bool) {
        self.object.set_state_bit(ALIGN_PATH_PS, align);
    }

    /// `CPhysicsObj::m_state`.
    pub fn physics_state(&self) -> u32 {
        self.object.state
    }

    /// `CPhysicsObj::IsMoving`: the squared speed is over `0.0002`.
    pub fn is_moving(&self) -> bool {
        let v = self.object.velocity;
        is_moving_velocity([v.x, v.y, v.z])
    }

    /// `PHYSICSOBJREP::GetOmegaVelocity` (`CPhysicsObj::m_vOmegaVelocity`).
    pub fn omega_velocity(&self) -> Vector {
        self.object.omega
    }

    /// `CPhysicsObj::set_omega_velocity`: the angular velocity is assigned as is.
    pub fn set_omega_velocity(&mut self, omega: Vector) {
        self.object.omega = omega;
    }

    /// `CPhysicsObj::m_vVelocity`.
    pub fn velocity(&self) -> Vector {
        self.object.velocity
    }

    /// `PHYSICSOBJREP::GetAnimVelocity` (`CPhysicsObj::m_vAnimVelocity`).
    pub fn anim_velocity(&self) -> Vector {
        self.object.anim_velocity
    }

    /// `PHYSICSOBJREP::GetAcceleration` (`CMotionInterp::GetAcceleration`, local axes).
    pub fn acceleration(&self) -> Vector {
        self.object.motion.acceleration()
    }

    /// `PHYSICSOBJREP::GetOmegaAcceleration` (`CMotionInterp::GetOmegaAcceleration`).
    pub fn omega_acceleration(&self) -> Vector {
        self.object.motion.omega_acceleration()
    }

    /// `PHYSICSOBJREP::GetDesiredVelocity` (`CMotionInterp::mLocalDesiredVel`).
    pub fn desired_velocity(&self) -> Vector {
        self.object.motion.local_desired_velocity()
    }

    /// `PHYSICSOBJREP::GetTerrainType` (`CMotionInterp::GetTerrainType`).
    pub fn terrain_type(&self) -> u32 {
        self.object.motion.terrain_type()
    }

    /// `PHYSICSOBJREP::GetEffectsScale` (`CPhysicsObj::GetScale`): the object's own scale.
    pub fn effects_scale(&self) -> Vector {
        self.object.scale
    }

    /// `PHYSICSOBJREP::GetWaterMultiplier` (`CMotionInterp::GetWaterMultiplier`).
    pub fn water_multiplier(&self) -> f32 {
        self.object.motion.water_multiplier()
    }

    /// `PHYSICSOBJREP::GetSeaFloorMultiplier` (`CMotionInterp::GetSeaFloorMultiplier`).
    pub fn sea_floor_multiplier(&self) -> f32 {
        self.object.motion.sea_floor_multiplier()
    }

    /// `CPhysicsObj::GetHoverHeight` (`CMotionInterp::GetHoverHeight`): the server's height, else
    /// the motion table's for the desired element.
    pub fn hover_height(&self) -> f32 {
        self.object
            .motion
            .hover_height(self.object.desired_element())
    }

    /// `CPhysicsObj::GetDefaultHoverHeight` (`CMotionInterp::GetDefaultHoverHeight`): the motion
    /// table's height for the desired element, whatever the server set.
    pub fn default_hover_height(&self) -> f32 {
        self.object
            .motion
            .motion_values(self.object.desired_element())
            .hover_height
    }

    /// `CPhysicsObj::GetVelocityScale` (`CMotionInterp::GetVelocityScale`).
    pub fn velocity_scale(&self) -> f32 {
        self.object.motion.velocity_scale()
    }

    /// `CPhysicsObj::GetAccelerationScale` (`CMotionInterp::GetAccelerationScale`).
    pub fn acceleration_scale(&self) -> f32 {
        self.object.motion.acceleration_scale()
    }

    /// `CPhysicsObj::GetJumpScale` (`m_fJumpScale`).
    pub fn jump_scale(&self) -> f32 {
        self.object.jump_scale
    }

    /// `PHYSICSOBJREP::IsTurningTo` (`CMotionInterp::IsTurningTo`).
    pub fn is_turning_to(&self) -> bool {
        self.object.motion.is_turning_to()
    }

    /// `CPhysicsObj::get_heading` (`Frame::get_heading` of the body's frame).
    pub fn heading(&self) -> Heading {
        self.object.position.frame.heading()
    }

    /// `CPhysicsObj::get_vector_heading` (`Frame::get_vector_heading`): the forward axis.
    pub fn vector_heading(&self) -> Vector {
        self.object.position.frame.vector_heading()
    }

    /// `CPhysicsObj::calc_heading`: the compass heading from the body toward `target`
    /// (`Position::heading`).
    pub fn calc_heading(&self, target: &Position) -> Heading {
        self.object.position.heading_to(target)
    }

    /// `CPhysicsObj::SetNoCellPosition`: the position and the previous position both become
    /// `position`, without any cell residency.
    pub fn set_no_cell_position(&mut self, position: Position) {
        self.object.position = position;
        self.object.prev_position = position;
    }

    /// `CPhysicsObj::GetRadius` (`PHYSICSOBJREP::GetRadius`): the Setup radius, scaled.
    pub fn radius(&self) -> f32 {
        self.object.radius()
    }

    /// `PHYSICSOBJREP::GetStepUpHeight` (`CPartArray::GetStepUpHeight`), unscaled.
    pub fn step_up_height(&self) -> f32 {
        self.object.shape.step_up_height
    }

    /// `PHYSICSOBJREP::GetStepDownHeight` (`CPartArray::GetStepDownHeight`), unscaled.
    pub fn step_down_height(&self) -> f32 {
        self.object.shape.step_down_height
    }

    /// `CPhysicsObj::SetStepDownHeight` (`CPartArray::SetStepDownHeight`).
    pub fn set_step_down_height(&mut self, step: f32) {
        self.object.shape.step_down_height = step;
    }

    /// `CPhysicsObj::AllowsFreeHeading` (`CPartArray::AllowsFreeHeading`).
    pub fn allows_free_heading(&self) -> bool {
        self.object.shape.allow_free_heading
    }

    /// `CPhysicsObj::GetNumSphere` (`CPartArray::GetNumSphere`).
    pub fn num_sphere(&self) -> usize {
        self.object.shape.spheres.len()
    }

    /// `CPhysicsObj::GetSphere` (`CPartArray::GetSphere`): the Setup's collision spheres.
    pub fn spheres(&self) -> &[Sphere] {
        &self.object.shape.spheres
    }

    /// `CPartArray::GetPhysicsContainmentRadius`: the Setup's containment radius of the
    /// body's spheres, unscaled.
    pub fn physics_containment_radius(&self) -> f32 {
        self.object.setup_containment
    }

    /// `CPhysicsObj::m_fViewDistance` (`CPhysicsObj::GetDistanceToViewer`).
    pub fn distance_to_viewer(&self) -> f32 {
        self.object.view_distance
    }

    /// `CPhysicsObj::GetLookAtX` (`m_fHeadX`).
    pub fn look_at_x(&self) -> f32 {
        self.object.head_x
    }

    /// `CPhysicsObj::GetLookAtZ` (`m_fHeadZ`).
    pub fn look_at_z(&self) -> f32 {
        self.object.head_z
    }

    /// `CPhysicsObj::GetPhysicsCollideMask` (`m_blocking_mask`) as the bits the server sets.
    pub fn physics_collide_mask(&self) -> u64 {
        self.object.blocking.bits()
    }
}

/// `CPhysicsObj::IsMoving` over a body's velocity (`m_velocityVector`, as a frame carries it): the
/// squared speed is over `0.0002`.
pub fn is_moving_velocity(v: [f32; 3]) -> bool {
    let (x, y, z) = (ld(v[0]), ld(v[1]), ld(v[2]));
    ld(MOVING_SPEED_SQUARED) < z * z + y * y + x * x
}

#[cfg(test)]
mod tests {
    use crate::test_support::body_at;
    use crate::vector::Vector;

    #[test]
    fn contact_and_velocity_queries_follow_the_object() {
        let mut body = body_at(70.0, 30.0, 50.0);
        assert!(!body.is_sliding_down());
        body.object_mut().transient_state |= crate::physics_obj::CONTACT_TS;
        assert!(body.is_sliding_down(), "contact without walkable ground");
        body.object_mut().transient_state |= crate::physics_obj::ON_WALKABLE_TS;
        assert!(!body.is_sliding_down());
        body.set_velocity([0.0, 3.0, 0.0]);
        let local = body.local_velocity();
        assert!((local.x * local.x + local.y * local.y + local.z * local.z - 9.0).abs() < 1e-3);
        assert_eq!(body.last_do_motion_values(), Vector::ZERO);
        assert_eq!(body.standard_motion_values(), body.desired_motion_values());
    }

    #[test]
    fn scale_sets_the_total_scale_and_marks_a_scaled_bsp_body() {
        let mut body = body_at(70.0, 30.0, 50.0);
        body.set_uniform_scale(2.0);
        assert_eq!(body.own_total_scale(), Vector::new(2.0, 2.0, 2.0));
        assert_eq!(body.effects_scale(), Vector::new(2.0, 2.0, 2.0));
        assert_eq!(body.physics_state() & super::SCALED_PS, 0, "no physics BSP");
        body.object_mut().state |= super::HAS_PHYSICS_BSP_PS;
        body.set_uniform_scale(1.0001);
        assert_eq!(body.physics_state() & super::SCALED_PS, 0);
        body.set_scale(Vector::new(1.0, 1.0, 3.0));
        assert_eq!(
            body.physics_state() & super::SCALED_PS,
            0,
            "only x and y are compared"
        );
        body.set_scale(Vector::new(1.0, 3.0, 1.0));
        assert_ne!(body.physics_state() & super::SCALED_PS, 0);
    }

    #[test]
    fn state_bits_and_motion_getters_follow_the_object() {
        let mut body = body_at(70.0, 30.0, 50.0);
        assert!(body.reports_collisions());
        assert!(!body.ignore_collisions());
        assert!(!body.is_in_workspace());
        body.set_in_workspace(true);
        assert!(body.is_in_workspace());
        body.set_in_workspace(false);
        assert!(!body.is_in_workspace());
        body.set_missile(true);
        body.align_to_path(true);
        assert_eq!(body.physics_state() & 0xa0, 0xa0);
        body.set_missile(false);
        assert_eq!(body.physics_state() & 0xa0, 0x80);
        assert!(!body.is_moving());
        body.set_velocity([0.0, 0.0, 0.1]);
        assert!(body.is_moving());
        body.set_velocity([0.0, 0.0, 0.01]);
        assert!(!body.is_moving());
        body.set_omega_velocity(Vector::new(1.0, 2.0, 3.0));
        assert_eq!(body.omega_velocity(), Vector::new(1.0, 2.0, 3.0));
        assert_eq!(body.off_ground(), !body.has_contact());
        assert_eq!(body.velocity_scale(), 1.0);
        assert_eq!(body.jump_scale(), 1.0);
    }

    #[test]
    fn no_cell_position_sets_both_positions() {
        let mut body = body_at(70.0, 30.0, 50.0);
        let mut target = body.position();
        target.frame.origin.x += 5.0;
        body.set_no_cell_position(target);
        assert_eq!(body.position(), target);
        assert_eq!(body.previous_position(), target);
        assert!(body.calc_heading(&target).degrees() == 0.0);
    }
}
