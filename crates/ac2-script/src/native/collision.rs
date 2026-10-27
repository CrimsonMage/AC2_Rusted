//! The collision natives the impact glue reads and writes: `CollisionParameters` (native 0x1A,
//! the minimums a script sets for its object's collision manager) and `CollisionProfile`
//! (native 0x1B, the report a collision hands the interface's `HandleCollision`).

/// `CollisionParameters`: the two impact minimums (`m_min_reported_impactXY` at +0xA8 and
/// `m_min_reported_impactZ` at +0xAC in retail's native). Zero means the minimum is not set.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CollisionParameters {
    pub min_reported_impact_xy: f32,
    pub min_reported_impact_z: f32,
}

/// `CollisionProfile`: the collision report. An impact report carries the contact normal and
/// the impact speed in state 0 (`CollisionProfile::HasCollided` answers 1 for states 0 and 2);
/// `IsEndCollision` is state 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionProfile {
    /// `m_collision_normal` (`CollisionProfile` +0x5C..0x64).
    pub collision_normal: [f32; 3],
    /// `m_impact_speed` (+0x68).
    pub impact_speed: f32,
    /// `m_collision_normal_state` (+0x54).
    pub state: u32,
}

impl Default for CollisionProfile {
    /// `CollisionProfile::CollisionProfile`: no collision (state 0 with a zero normal and speed).
    fn default() -> Self {
        Self {
            collision_normal: [0.0, 0.0, 1.0],
            impact_speed: 0.0,
            state: 0,
        }
    }
}

impl CollisionProfile {
    /// `CollisionProfile::HasCollided`: the normal and speed, when the report has a collision
    /// (state 0 or 2).
    pub fn has_collided(&self) -> Option<([f32; 3], f32)> {
        matches!(self.state, 0 | 2).then_some((self.collision_normal, self.impact_speed))
    }

    /// `CollisionProfile::IsEndCollision`: state 1.
    pub fn is_end_collision(&self) -> bool {
        self.state == 1
    }

    /// A report of one impact (state 0): the contact normal and the speed the body recorded.
    pub fn impact(collision_normal: [f32; 3], impact_speed: f32) -> Self {
        Self {
            collision_normal,
            impact_speed,
            state: 0,
        }
    }
}
