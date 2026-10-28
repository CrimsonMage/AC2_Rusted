//! The collision report a body hands its script: `CCollisionManager::ReportCollision`, called
//! by `CPhysicsObj::ReportCollisions` (and `CPhysicsObj::EndAllCollisions` with both flags set).
//!
//! The manager keeps a table of the server objects a transition touched
//! (`CCollisionManager::AddObjectCollision`, `ObjCollisionType`). An object is reported as
//! beginning when it is first in the table, and as ending once it has not been touched for more
//! than a second in a later check, or at once when the report ends everything. The landscape,
//! water and environment layers, and the contact plane, follow the same begin and end rule. The
//! report carries plain data (`CollisionReport`); `ac2-script` turns it into the script's
//! `CollisionProfile`.

use crate::collision_impact::ImpactReport;
use crate::collision_types::InstanceId;
use crate::vector::Plane;

/// The time an object stays reported after its last touch (`1.0 <` in `ReportCollision`).
const TOUCH_EXPIRY_SECONDS: f64 = 1.0;

/// Whether a record begins or ends in this report (`ObjCollisionType::SetAsBeginCollide` and
/// `SetAsEndCollide`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordState {
    Begin,
    End,
}

/// One object the report lists (`ObjCollisionProfile`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectRecord {
    /// `m_id`, the object's `InstanceID`.
    pub id: InstanceId,
    pub state: RecordState,
    /// `m_is_ethereal` at the time of the report.
    pub ethereal: bool,
}

/// One material the report lists (`CollideProperty`). `id` is the material's physics id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyRecord {
    pub id: u64,
    pub state: RecordState,
}

/// The contact plane a report lists, beginning or ending (`CollisionProfile::HasContact`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactRecord {
    pub plane: Plane,
    pub state: RecordState,
}

/// The materials of the body's current contact, as the world knows them. A layer the body is in
/// records its material only when the world has one; `None` records nothing for that layer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ContactMaterials {
    /// The landscape's physics material under the body (`CCollisionManager::SetOnLandscape`).
    pub landscape: Option<u64>,
    /// The water's physics material (`CCollisionManager::SetInWater`).
    pub water: Option<u64>,
    /// The environment cell's physics material (`CCollisionManager::AddEnvironmentalProperties`).
    pub environment: Option<u64>,
}

/// What one collision report lists.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollisionReport {
    /// The impact the contact recorded (`CollisionProfile`'s normal and speed).
    pub impact: Option<ImpactReport>,
    /// The contact plane beginning or ending.
    pub contact: Option<ContactRecord>,
    /// The objects the body touched (`m_objects`).
    pub objects: Vec<ObjectRecord>,
    /// The environment materials (`m_env_properties`).
    pub environment: Vec<PropertyRecord>,
    /// The landscape materials (`m_surface_properties`).
    pub surface: Vec<PropertyRecord>,
    /// The water materials (`m_water_properties`).
    pub water: Vec<PropertyRecord>,
}

impl CollisionReport {
    /// `CollisionProfile::NonEmpty`: the report lists something to tell the script.
    pub fn is_empty(&self) -> bool {
        self.impact.is_none()
            && self.contact.is_none()
            && self.objects.is_empty()
            && self.environment.is_empty()
            && self.surface.is_empty()
            && self.water.is_empty()
    }
}

/// The body's contact state for one report: which layers it is in and its contact plane.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct BodyContact {
    pub plane: Option<Plane>,
    pub in_water: bool,
    pub on_landscape: bool,
    pub in_environment: bool,
}

/// One entry in the touched-object table (`ObjCollisionType`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct TouchedObject {
    id: InstanceId,
    ethereal: bool,
    /// `m_touched_time`: the check time of the last touch.
    touched: f64,
    /// `m_has_been_reported`.
    reported: bool,
}

/// One material layer: the material the report last listed for it (`m_reported_*_material`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Layer {
    reported: Option<u64>,
}

impl Layer {
    /// Begins or changes the layer's material, ends the previous one, and ends the layer when the
    /// body is no longer in it (`ChangePhysicsMaterial`, `ReportSurfaceProperties`,
    /// `EndOnLandscape`).
    fn update(&mut self, current: Option<u64>, out: &mut Vec<PropertyRecord>) {
        match current {
            Some(material) if self.reported == Some(material) => {}
            Some(material) => {
                if let Some(old) = self.reported {
                    out.push(PropertyRecord {
                        id: old,
                        state: RecordState::End,
                    });
                }
                out.push(PropertyRecord {
                    id: material,
                    state: RecordState::Begin,
                });
                self.reported = Some(material);
            }
            None => self.end(out),
        }
    }

    /// Ends whatever the layer reported (`EndInWater`, `EndOnLandscape`, `EndEnvironmentalProperties`).
    fn end(&mut self, out: &mut Vec<PropertyRecord>) {
        if let Some(old) = self.reported.take() {
            out.push(PropertyRecord {
                id: old,
                state: RecordState::End,
            });
        }
    }
}

/// `CCollisionManager`: the touched objects, the layers and the contact the report has listed.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CollisionManager {
    /// `m_last_collide_check_time`: the time of the latest collision check.
    check_time: f64,
    objects: Vec<TouchedObject>,
    water: Layer,
    landscape: Layer,
    environment: Layer,
    /// The contact plane last reported (`m_reported_has_contact`, with the plane it listed).
    contact: Option<Plane>,
    /// The impact the contact recorded and no report has taken (`m_has_normal`).
    impact: Option<ImpactReport>,
}

impl CollisionManager {
    /// Starts a collision check at `now` (`m_last_collide_check_time`).
    pub(crate) fn begin_check(&mut self, now: f64) {
        self.check_time = now;
    }

    /// Records an impact the contact made (`CCollisionManager::SetCollisionNormal`).
    pub(crate) fn set_impact(&mut self, impact: ImpactReport) {
        self.impact = Some(impact);
    }

    /// `CCollisionManager::AddObjectCollision`: the object is touched in this check. An object
    /// not yet in the table joins it, unreported.
    pub(crate) fn touch(&mut self, id: InstanceId, ethereal: bool) {
        let now = self.check_time;
        match self.objects.iter_mut().find(|o| o.id == id) {
            Some(entry) => {
                entry.touched = now;
                entry.ethereal = ethereal;
            }
            None => self.objects.push(TouchedObject {
                id,
                ethereal,
                touched: now,
                reported: false,
            }),
        }
    }

    /// `CCollisionManager::ReportCollision`: the report at `now`. `end_reporting` ends every
    /// record that is reported (`force_end`); `end_all` also drops the table and ends the
    /// layers, and takes no impact. `active` is whether the body reports this step
    /// (the physics-active check, `ICPhysicsObj` vtable slot 0x248, not yet named): an inactive
    /// body begins nothing.
    pub(crate) fn report(
        &mut self,
        now: f64,
        contact: BodyContact,
        materials: ContactMaterials,
        flags: ReportFlags,
    ) -> CollisionReport {
        let mut out = CollisionReport::default();
        self.report_objects(now, flags, &mut out);
        if !flags.active {
            // The impact is discarded, as retail clears `m_has_normal` on every report.
            self.impact = None;
            return out;
        }
        if flags.end_all {
            self.impact = None;
            self.water.end(&mut out.water);
            self.landscape.end(&mut out.surface);
            self.environment.end(&mut out.environment);
            self.end_contact(&mut out);
            return out;
        }
        out.impact = self.impact.take();
        self.water.update(
            contact.in_water.then_some(materials.water).flatten(),
            &mut out.water,
        );
        self.landscape.update(
            contact
                .on_landscape
                .then_some(materials.landscape)
                .flatten(),
            &mut out.surface,
        );
        self.environment.update(
            contact
                .in_environment
                .then_some(materials.environment)
                .flatten(),
            &mut out.environment,
        );
        self.update_contact(contact.plane, &mut out);
        out
    }

    /// The object table part of `ReportCollision`: an object that expired (untouched for over a
    /// second in a check after its touch), or that the report ends, is ended if reported and
    /// dropped when it expired or the report ends everything; an unreported object begins.
    fn report_objects(&mut self, now: f64, flags: ReportFlags, out: &mut CollisionReport) {
        let check = self.check_time;
        self.objects.retain_mut(|entry| {
            let stale = now - entry.touched > TOUCH_EXPIRY_SECONDS;
            let expired = stale && entry.touched != check;
            if expired || flags.end_reporting || flags.end_all {
                if entry.reported {
                    out.objects.push(ObjectRecord {
                        id: entry.id,
                        state: RecordState::End,
                        ethereal: entry.ethereal,
                    });
                    entry.reported = false;
                }
                // Dropped at once when it expired or the report ends everything.
                return !(expired || flags.end_all);
            }
            if !entry.reported && flags.active {
                out.objects.push(ObjectRecord {
                    id: entry.id,
                    state: RecordState::Begin,
                    ethereal: entry.ethereal,
                });
                entry.reported = true;
            }
            true
        });
    }

    /// The contact plane: beginning when the body first has contact, ending with the plane it
    /// last listed when contact is lost.
    fn update_contact(&mut self, plane: Option<Plane>, out: &mut CollisionReport) {
        match (plane, self.contact) {
            (Some(now), None) => {
                out.contact = Some(ContactRecord {
                    plane: now,
                    state: RecordState::Begin,
                });
                self.contact = Some(now);
            }
            (None, Some(last)) => {
                out.contact = Some(ContactRecord {
                    plane: last,
                    state: RecordState::End,
                });
                self.contact = None;
            }
            _ => {}
        }
    }

    fn end_contact(&mut self, out: &mut CollisionReport) {
        if let Some(last) = self.contact.take() {
            out.contact = Some(ContactRecord {
                plane: last,
                state: RecordState::End,
            });
        }
    }
}

/// The flags a report runs with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReportFlags {
    /// `force_end`: every reported record ends.
    pub end_reporting: bool,
    /// `EndAllCollisions`: everything ends and the table is dropped.
    pub end_all: bool,
    /// The body reports this step (the physics-active check on the collision manager's object).
    pub active: bool,
}
