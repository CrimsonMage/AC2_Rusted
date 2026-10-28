//! The collision natives the impact glue reads and writes: `CollisionParameters` (native 0x1A,
//! the minimums a script sets for its object's collision manager) and `CollisionProfile`
//! (native 0x1B, the report a collision hands the interface's `HandleCollision`), with the
//! `ObjCollisionProfile` (0x4F) and `CollideProperty` (0x19) entries the report lists hold.

/// `CollisionParameters`: the two impact minimums (`m_min_reported_impactXY` at +0xA8 and
/// `m_min_reported_impactZ` at +0xAC in retail's native). Zero means the minimum is not set.
/// The ethereal and physics-type sets the mask efuns build are `sets`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollisionParameters {
    pub min_reported_impact_xy: f32,
    pub min_reported_impact_z: f32,
    pub sets: Box<ac2_physics::CollisionParameters>,
}

/// `ObjCollisionProfile` (native 0x4F): one object a collision report lists
/// (`CollisionProfile::GetObjectA` copies its id, begin flag and ethereal flag).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObjCollisionProfile {
    /// `m_id` (`InstanceID`).
    pub id: u64,
    /// `m_start_collide`: the object's collision began in this report (1) or ended (0).
    pub start_collide: bool,
    /// `m_is_ethereal`.
    pub is_ethereal: bool,
}

/// `CollideProperty` (native 0x19): one property a report lists (object, environment, surface
/// or water): its id, its collide type and its `BaseProperty` (`m_property`, at +0xC).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollideProperty {
    /// `m_id`.
    pub id: u64,
    /// `m_collide_type`.
    pub collide_type: u64,
    /// `m_property`: the property the report carried, read by `gCollideProperty::GetProperty`.
    pub property: ac2_db::BaseProperty,
}

/// `CollisionProfile`: the collision report. An impact report carries the contact normal and
/// the impact speed in state 0 (`CollisionProfile::HasCollided` answers 1 for states 0 and 2);
/// `IsEndCollision` is state 1. The lists and the contact plane are the report's other fields
/// (`CollisionProfile` +0x10..+0x78).
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionProfile {
    /// `m_collision_normal` (`CollisionProfile` +0x5C..0x64).
    pub collision_normal: [f32; 3],
    /// `m_impact_speed` (+0x68).
    pub impact_speed: f32,
    /// `m_collision_normal_state` (+0x54). 3 is no collision.
    pub state: u32,
    /// `m_contact_state` (+0x58). 3 is no contact.
    pub contact_state: u32,
    /// `m_contact_plane` (+0x6C: the normal, +0x78: `d`).
    pub contact_plane: ([f32; 3], f32),
    /// The objects the report lists (`m_objects`, `GetObjectA`).
    pub objects: Vec<ObjCollisionProfile>,
    /// The object properties (`m_object_properties`, `GetObjectProperty`).
    pub object_properties: Vec<CollideProperty>,
    /// The environment properties (`m_env_properties`, `GetEnvProperty`).
    pub env_properties: Vec<CollideProperty>,
    /// The surface properties (`m_surface_properties`, `GetSurfaceProperty`).
    pub surface_properties: Vec<CollideProperty>,
    /// The water properties (`m_water_properties`, `GetWaterProperty`).
    pub water_properties: Vec<CollideProperty>,
}

impl Default for CollisionProfile {
    /// `CollisionProfile::CollisionProfile`: no collision and no contact (states 3), empty lists.
    fn default() -> Self {
        Self {
            collision_normal: [0.0, 0.0, 1.0],
            impact_speed: 0.0,
            state: 3,
            contact_state: 3,
            contact_plane: ([0.0, 0.0, 1.0], 0.0),
            objects: Vec::new(),
            object_properties: Vec::new(),
            env_properties: Vec::new(),
            surface_properties: Vec::new(),
            water_properties: Vec::new(),
        }
    }
}

impl CollisionProfile {
    /// `CollisionProfile` as a physics report fills it (`CCollisionManager::ReportCollision`):
    /// the impact in state 0, the contact plane with its begin or end, and the object and
    /// property lists, each property with its master id and collide type.
    pub fn from_report(report: &ac2_physics::CollisionReport) -> Self {
        use ac2_physics::RecordState;
        let mut profile = Self::default();
        if let Some(impact) = report.impact {
            profile.collision_normal = [impact.normal.x, impact.normal.y, impact.normal.z];
            profile.impact_speed = impact.speed;
            profile.state = 0;
        }
        if let Some(contact) = report.contact {
            let n = contact.plane.n;
            profile.contact_plane = ([n.x, n.y, n.z], contact.plane.d);
            profile.contact_state = match contact.state {
                RecordState::Begin => 0,
                RecordState::End => 1,
            };
        }
        profile.objects = report
            .objects
            .iter()
            .map(|o| ObjCollisionProfile {
                id: o.id,
                start_collide: o.state == RecordState::Begin,
                is_ethereal: o.ethereal,
            })
            .collect();
        let properties = |records: &[ac2_physics::CollideProperty]| {
            records
                .iter()
                .map(|r| CollideProperty {
                    id: r.id,
                    collide_type: r.collide_type.raw(),
                    property: r.property.to_base_property(),
                })
                .collect::<Vec<_>>()
        };
        profile.env_properties = properties(&report.environment);
        profile.surface_properties = properties(&report.surface);
        profile.water_properties = properties(&report.water);
        profile
    }

    /// `CollisionProfile::HasCollided`: the normal and speed, when the report has a collision
    /// (state 0 or 2).
    pub fn has_collided(&self) -> Option<([f32; 3], f32)> {
        matches!(self.state, 0 | 2).then_some((self.collision_normal, self.impact_speed))
    }

    /// `CollisionProfile::IsEndCollision`: state 1.
    pub fn is_end_collision(&self) -> bool {
        self.state == 1
    }

    /// `CollisionProfile::HasContact`: the contact plane, when the contact state is 0 or 2.
    pub fn has_contact(&self) -> Option<([f32; 3], f32)> {
        matches!(self.contact_state, 0 | 2).then_some(self.contact_plane)
    }

    /// `CollisionProfile::IsEndContact`: contact state 1.
    pub fn is_end_contact(&self) -> bool {
        self.contact_state == 1
    }

    /// A report of one impact (state 0): the contact normal and the speed the body recorded.
    pub fn impact(collision_normal: [f32; 3], impact_speed: f32) -> Self {
        Self {
            collision_normal,
            impact_speed,
            state: 0,
            ..Self::default()
        }
    }
}
