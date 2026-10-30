//! The collision report a body hands its script: `CCollisionManager::ReportCollision`, called
//! by `CPhysicsObj::ReportCollisions` (and `CPhysicsObj::EndAllCollisions` with both flags set).
//!
//! The manager keeps a table of the server objects a transition touched
//! (`CCollisionManager::AddObjectCollision`, `ObjCollisionType`). An object is reported as
//! beginning when it is first in the table, and as ending once it has not been touched for more
//! than a second in a later check, or at once when the report ends everything. The landscape and
//! water layers, the environment and the contact plane follow the same begin and end rule for
//! their records; the materials a layer names carry the property lists the report lists
//! (`CollideProperty`, see `collide_property`). The motion terrain the landscape and water
//! properties give (`CCollisionManager::SetTerrainType`) is handed to the body after the report.
//! The report carries plain data (`CollisionReport`); `ac2-script` turns it into the script's
//! `CollisionProfile`.

use std::sync::Arc;

use crate::collide_property::{
    mark_all_end, remove_end, terrain_type_value, update_property_array, CollideProperty,
    CollideType, PropertyDescs, TERRAIN_TYPE_PROPERTY,
};
use crate::collision_impact::ImpactReport;
use crate::collision_types::InstanceId;
use crate::physics_material::PhysicsMaterial;
use crate::pushers::Pushers;
use crate::vector::Plane;

/// The time an object stays reported after its last touch (`1.0 <` in `ReportCollision`).
const TOUCH_EXPIRY_SECONDS: f64 = 1.0;

/// `CCollisionManager::m_land_terrain` and the other terrain sentinels: no terrain type.
const NO_TERRAIN: u64 = 0x8000_0001;

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

/// The contact plane a report lists, beginning or ending (`CollisionProfile::HasContact`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactRecord {
    pub plane: Plane,
    pub state: RecordState,
}

/// The materials of the body's current contact, as the world knows them. A layer the body is in
/// names its material when the world has one; `None` is the material id 0 (no properties).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ContactMaterials {
    /// The landscape's physics material under the body (`CCollisionManager::SetOnLandscape`).
    pub landscape: Option<Arc<PhysicsMaterial>>,
    /// The water's physics material (`CCollisionManager::SetInWater`).
    pub water: Option<Arc<PhysicsMaterial>>,
    /// The environment cell's physics material (`CCollisionManager::AddEnvironmentalProperties`).
    pub environment: Option<Arc<PhysicsMaterial>>,
}

/// A server object a transition stopped on, for the owner to tell that object's body
/// (`CPhysicsObj::handle_all_collisions` adds the mover to the other object's collision manager
/// too, and records the contact normal there for the last object that stopped it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Collided {
    /// The object's `InstanceID`.
    pub id: InstanceId,
    /// For the transition's `last_collided_object` when it recorded a collision normal: the
    /// normal and the mover's velocity before the response (`SetCollisionNormal`'s arguments).
    pub impact: Option<(crate::vector::Vector, crate::vector::Vector)>,
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
    /// The environment properties (`m_env_properties`).
    pub environment: Vec<CollideProperty>,
    /// The landscape properties (`m_surface_properties`).
    pub surface: Vec<CollideProperty>,
    /// The water properties (`m_water_properties`).
    pub water: Vec<CollideProperty>,
    /// The properties of the objects touched (`m_object_properties`): each begun or ended with
    /// its object's property list (`CollidePropertyList::BeginPropertyCollision` and
    /// `EndPropertyCollision`).
    pub object_properties: Vec<CollideProperty>,
    /// The objects the report erased from the table (expired, removed or ended with
    /// everything). A body takes them out of the report (`PhysicsBody::take_dropped`), so a report it hands out has none; they are
    /// not part of what the script hears (`CollisionProfile::NonEmpty` ignores them).
    pub dropped: Vec<InstanceId>,
}

impl CollisionReport {
    /// `CollisionProfile::NonEmpty`: the report lists something to tell the script.
    pub fn is_empty(&self) -> bool {
        self.impact.is_none()
            && self.contact.is_none()
            && self.objects.is_empty()
            && self.environment.is_empty()
            && self.object_properties.is_empty()
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

/// One entry in the touched-object table (`ObjCollisionType`: `ObjCollisionType::GetID` is `id`,
/// `SetAsBeginCollide` and `SetAsEndCollide` are the [`RecordState`] a report lists it with).
#[derive(Debug, Clone, Copy, PartialEq)]
struct TouchedObject {
    id: InstanceId,
    /// `m_remove_now`: the object left the world (`CCollisionManager::RemoveObjectCollision`);
    /// the next report ends and erases the entry.
    remove_now: bool,
    ethereal: bool,
    /// `m_touched_time`: the check time of the last touch.
    touched: f64,
    /// `m_has_been_reported`.
    reported: bool,
    /// `m_ignore_collisions`: the touched object's `ignore_collisions` when last seen.
    ignore_collisions: bool,
    /// `m_ignore_changed`: set when `ignore_collisions` changed after the entry was reported.
    ignore_changed: bool,
}

/// One entry of the property table (`CollidePropertyList`): the properties of a touched object.
#[derive(Debug, Clone, PartialEq)]
struct ObjectProps {
    /// `m_physobj_id` (the touched object's table key) and `m_inst_id`: its `InstanceID`.
    key: InstanceId,
    touched: f64,
    remove_now: bool,
    /// `m_has_been_reported`.
    reported: bool,
    props: Vec<CollideProperty>,
}

/// Which surface a material layer is: the water or the landscape. The two list and terrain
/// rules differ (`ReportWaterProperties` and `UpdateWaterTerrain`, `ReportSurfaceProperties` and
/// `UpdateLandTerrain`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Water,
    Landscape,
}

/// One material layer: whether the report has begun it, the material it began with
/// (`m_reported_in_water`, `m_reported_*_physics_material`) and its property list
/// (`m_water_properties`, `m_surface_properties`).
#[derive(Debug, Clone, Default)]
struct MaterialLayer {
    reported_in: bool,
    reported_material: u64,
    props: Vec<CollideProperty>,
}

/// The motion terrain the landscape and water properties give (`m_land_terrain`,
/// `m_water_terrain`, `m_prev_terrain`), and the change the body has not taken yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerrainTypes {
    land: u64,
    water: u64,
    prev: u64,
    pending: Option<u64>,
}

impl Default for TerrainTypes {
    fn default() -> Self {
        Self {
            land: NO_TERRAIN,
            water: NO_TERRAIN,
            prev: NO_TERRAIN,
            pending: None,
        }
    }
}

impl TerrainTypes {
    /// `CCollisionManager::SetTerrainType`: a terrain that differs from the last one is handed to
    /// the body (`CPhysicsObj::SetTerrainType`).
    fn set(&mut self, terrain: u64) {
        if terrain != self.prev {
            self.prev = terrain;
            self.pending = Some(terrain);
        }
    }

    /// `CCollisionManager::UpdateLandTerrain`: an ended property clears the land terrain; a
    /// terrain-type value sets it. Without water the land terrain applies.
    fn land_property(&mut self, entry: &CollideProperty) {
        if entry.collide_type == CollideType::End {
            self.land = NO_TERRAIN;
        } else if let Some(value) = terrain_type_value(&entry.property) {
            self.land = value;
        } else {
            return;
        }
        if self.water == NO_TERRAIN {
            let land = self.land;
            self.set(land);
        }
    }

    /// `CCollisionManager::UpdateWaterTerrain`: an ended property clears the water terrain and the
    /// land terrain applies; a terrain-type value sets the water terrain.
    fn water_property(&mut self, entry: &CollideProperty) {
        if entry.collide_type == CollideType::End {
            self.water = NO_TERRAIN;
            let land = self.land;
            self.set(land);
        } else if let Some(value) = terrain_type_value(&entry.property) {
            self.water = value;
            self.set(value);
        }
    }
}

/// `CCollisionManager`: the touched objects, the layers, the environment, the terrain and the
/// contact the report has listed.
#[derive(Debug, Clone)]
pub(crate) struct CollisionManager {
    /// `m_last_collide_check_time`: the time of the latest collision check.
    check_time: f64,
    objects: Vec<TouchedObject>,
    /// `m_prop_collision_table`: the property list of each touched object that has properties.
    object_props: Vec<ObjectProps>,
    water: MaterialLayer,
    landscape: MaterialLayer,
    /// `m_env_properties` and `m_env_properties_updated`.
    environment: Vec<CollideProperty>,
    env_updated: bool,
    /// `m_master_id`: the next id a property begins with.
    master_id: u64,
    terrain: TerrainTypes,
    /// The property table the lists consult (the installed `MasterProperty` in the client).
    pub(crate) descs: PropertyDescs,
    /// The contact plane last reported (`m_reported_has_contact`, with the plane it listed).
    contact: Option<Plane>,
    /// The impact the contact recorded and no report has taken (`m_has_normal`).
    impact: Option<ImpactReport>,
    /// `m_pushers`: the objects pushing the body.
    pub(crate) pushers: Pushers,
}

impl Default for CollisionManager {
    fn default() -> Self {
        Self {
            check_time: 0.0,
            objects: Vec::new(),
            object_props: Vec::new(),
            water: MaterialLayer::default(),
            landscape: MaterialLayer::default(),
            environment: Vec::new(),
            env_updated: false,
            master_id: 0,
            terrain: TerrainTypes::default(),
            descs: PropertyDescs::default(),
            contact: None,
            impact: None,
            pushers: Pushers::default(),
        }
    }
}

impl CollisionManager {
    /// Starts a collision check at `now` (`m_last_collide_check_time`).
    pub(crate) fn begin_check(&mut self, now: f64) {
        self.check_time = now;
    }

    /// `CCollisionManager::SetNoCollisionNormal`: the impact the contact recorded is dropped
    /// (`m_has_normal = 0`).
    pub(crate) fn set_no_collision_normal(&mut self) {
        self.impact = None;
    }

    /// Records an impact the contact made (`CCollisionManager::SetCollisionNormal`).
    pub(crate) fn set_impact(&mut self, impact: ImpactReport) {
        self.impact = Some(impact);
    }

    /// The motion terrain the last report changed, for the body to hand to its motion
    /// interpreter (`CPhysicsObj::SetTerrainType`). Taken once.
    pub(crate) fn take_terrain(&mut self) -> Option<u64> {
        self.terrain.pending.take()
    }

    /// `CCollisionManager::AddObjectCollision`: the object is touched in this check. An object
    /// not yet in the table joins it, unreported.
    /// `ignore` is the object's `ignore_collisions` (`CPhysicsObj::ignore_collisions`): an entry
    /// already listed takes a changed value (`ObjCollisionType::UpdateIgnoreCollide`, which marks
    /// `m_ignore_changed` once the entry was reported).
    pub(crate) fn touch(&mut self, id: InstanceId, ethereal: bool, ignore: bool) {
        let now = self.check_time;
        match self.objects.iter_mut().find(|o| o.id == id) {
            Some(entry) => {
                if entry.ignore_collisions != ignore {
                    entry.ignore_collisions = ignore;
                    if entry.reported {
                        entry.ignore_changed = true;
                    }
                }
                entry.touched = now;
                entry.ethereal = ethereal;
            }
            None => self.objects.push(TouchedObject {
                id,
                remove_now: false,
                ethereal,
                touched: now,
                reported: false,
                ignore_collisions: ignore,
                ignore_changed: false,
            }),
        }
    }

    /// `CCollisionManager::IsEmpty`: neither the object table nor the property table lists
    /// anything.
    pub(crate) fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.object_props.is_empty()
    }

    /// `CCollisionManager::RemoveObjectCollision`: the object `id` left the world; its entry is
    /// marked (`m_remove_now`) and the next report ends it if it was reported, then erases it.
    /// Its property list ends at once, if it had begun, and is erased
    /// (`RemovePropertiesAndErase`); the returned report carries those ended properties and is
    /// `Some` only when it is not empty (`CollisionProfile::NonEmpty`).
    pub(crate) fn remove_object(&mut self, id: InstanceId) -> Option<CollisionReport> {
        if let Some(entry) = self.objects.iter_mut().find(|o| o.id == id) {
            entry.remove_now = true;
        }
        let mut out = CollisionReport::default();
        if let Some(index) = self.object_props.iter().position(|e| e.key == id) {
            let mut entry = self.object_props.remove(index);
            if entry.reported {
                end_object_props(&mut entry, self.descs, &mut out);
            }
            self.pushers.remove(id);
        }
        (!out.is_empty()).then_some(out)
    }

    /// The property part of `CCollisionManager::AddObjectCollision`: the touched object `key`
    /// has `groups` (`GetObjectProperties`). One already listed is only touched again; a new one
    /// gets a `CollidePropertyList` of its inquirable properties (`SetProperties`, each with the
    /// next master id), pushes the body when it has pusher vectors (`AddPusherProperties`, with
    /// `position` for local-space vectors) and joins the table.
    pub(crate) fn touch_properties(
        &mut self,
        key: InstanceId,
        groups: &[ac2_db::Ac2PropertyGroup],
        position: Option<&crate::position::Position>,
    ) {
        if groups.is_empty() {
            return;
        }
        let now = self.check_time;
        if let Some(entry) = self.object_props.iter_mut().find(|e| e.key == key) {
            entry.touched = now;
            return;
        }
        let mut props = Vec::new();
        for property in groups.iter().flat_map(|g| &g.properties) {
            if self.descs.inquirable(property.name) {
                props.push(CollideProperty {
                    id: self.master_id,
                    collide_type: CollideType::Begin,
                    property: property.clone(),
                });
                self.master_id += 1;
            }
        }
        let values: Vec<_> = props.iter().map(|p| p.property.clone()).collect();
        self.pushers.add_properties(key, position, &values);
        self.object_props.push(ObjectProps {
            key,
            touched: now,
            remove_now: false,
            reported: false,
            props,
        });
    }

    /// `CCollisionManager::FindInPropertyList`: whether the object `key` has a property list.
    #[cfg(test)]
    pub(crate) fn find_in_property_list(&self, key: InstanceId) -> bool {
        self.object_props.iter().any(|e| e.key == key)
    }

    /// The ids of the touched objects (`m_collision_table`), in table order.
    pub(crate) fn touched_ids(&self) -> Vec<InstanceId> {
        self.objects.iter().map(|o| o.id).collect()
    }

    /// `ObjCollisionProfile::m_is_ethereal` of the entry for `id`.
    pub(crate) fn entry_ethereal(&self, id: InstanceId) -> Option<bool> {
        self.objects.iter().find(|o| o.id == id).map(|o| o.ethereal)
    }

    /// Sets the entry's `m_is_ethereal` (`CCollisionManager::HasConflict`).
    pub(crate) fn set_entry_ethereal(&mut self, id: InstanceId, ethereal: bool) {
        if let Some(entry) = self.objects.iter_mut().find(|o| o.id == id) {
            entry.ethereal = ethereal;
        }
    }

    /// The entry's `(m_ignore_collisions, m_ignore_changed)`.
    #[cfg(test)]
    pub(crate) fn entry_ignore(&self, id: InstanceId) -> Option<(bool, bool)> {
        self.objects
            .iter()
            .find(|o| o.id == id)
            .map(|o| (o.ignore_collisions, o.ignore_changed))
    }

    /// `CCollisionManager::ReportCollision`: the report at `now`. `end_reporting` ends every
    /// record of the object table that is reported (`force_end`); `end_all` also drops the table,
    /// ends the layers and the environment, and takes no impact. `active` is whether the body
    /// reports this step (the physics-active check, `ICPhysicsObj` vtable slot 0x248, not yet
    /// named): an inactive body begins nothing.
    pub(crate) fn report(
        &mut self,
        now: f64,
        contact: BodyContact,
        materials: &ContactMaterials,
        flags: ReportFlags,
    ) -> CollisionReport {
        let mut out = CollisionReport::default();
        self.report_objects(now, flags, &mut out);
        self.report_object_props(now, flags, &mut out);
        if !flags.active {
            // The impact is discarded, as retail clears `m_has_normal` on every report.
            self.set_no_collision_normal();
            return out;
        }
        if flags.end_all {
            self.set_no_collision_normal();
            if self.water.reported_in {
                end_layer(
                    Kind::Water,
                    &mut self.water,
                    &mut self.terrain,
                    self.descs,
                    &mut out.water,
                );
            }
            self.water.reported_in = false;
            if self.landscape.reported_in {
                end_layer(
                    Kind::Landscape,
                    &mut self.landscape,
                    &mut self.terrain,
                    self.descs,
                    &mut out.surface,
                );
            }
            self.landscape.reported_in = false;
            self.end_environment(&mut out.environment);
            self.end_contact(&mut out);
            return out;
        }
        out.impact = self.impact.take();
        update_material_layer(
            Kind::Water,
            &mut self.water,
            contact.in_water,
            materials.water.as_deref(),
            Layers {
                terrain: &mut self.terrain,
                descs: self.descs,
                next_id: &mut self.master_id,
            },
            &mut out.water,
        );
        update_material_layer(
            Kind::Landscape,
            &mut self.landscape,
            contact.on_landscape,
            materials.landscape.as_deref(),
            Layers {
                terrain: &mut self.terrain,
                descs: self.descs,
                next_id: &mut self.master_id,
            },
            &mut out.surface,
        );
        self.update_environment(
            contact.in_environment,
            materials.environment.as_deref(),
            &mut out.environment,
        );
        self.update_contact(contact.plane, &mut out);
        out
    }

    /// The object table part of `ReportCollision`: an object that expired (untouched for over a
    /// second in a check after its touch), that was removed, or that the report ends, is ended
    /// if reported and dropped when it expired, was removed or the report ends everything; an
    /// unreported object begins. An object that ignores collisions is never listed as beginning
    /// (and ends only when its flag changed after it was reported); a reported entry whose
    /// ignore flag changed is listed as beginning (flag cleared) or ending (flag set) once.
    fn report_objects(&mut self, now: f64, flags: ReportFlags, out: &mut CollisionReport) {
        let check = self.check_time;
        self.objects.retain_mut(|entry| {
            let stale = now - entry.touched > TOUCH_EXPIRY_SECONDS;
            let expired = stale && entry.touched != check;
            let record = |state| ObjectRecord {
                id: entry.id,
                state,
                ethereal: entry.ethereal,
            };
            if expired || entry.remove_now || flags.end_reporting || flags.end_all {
                if entry.reported {
                    if !entry.ignore_collisions || entry.ignore_changed {
                        out.objects.push(record(RecordState::End));
                    }
                    entry.reported = false;
                }
                // Dropped at once when it expired, was removed or the report ends everything.
                if expired || entry.remove_now || flags.end_all {
                    out.dropped.push(entry.id);
                    return false;
                }
                return true;
            }
            if !entry.reported && flags.active {
                if !entry.ignore_collisions {
                    out.objects.push(record(RecordState::Begin));
                }
                entry.reported = true;
            } else if entry.ignore_changed && entry.reported {
                let state = if entry.ignore_collisions {
                    RecordState::End
                } else {
                    RecordState::Begin
                };
                out.objects.push(record(state));
                entry.ignore_changed = false;
            }
            true
        });
    }

    /// The property table part of `ReportCollision`: a list that expired, was removed or is
    /// ended with the report lists its properties as ended if it had begun and is erased (the
    /// object stops pushing, `RemovePropertiesAndErase`); a list not yet reported begins.
    fn report_object_props(&mut self, now: f64, flags: ReportFlags, out: &mut CollisionReport) {
        let check = self.check_time;
        let descs = self.descs;
        let pushers = &mut self.pushers;
        self.object_props.retain_mut(|entry| {
            let stale = now - entry.touched > TOUCH_EXPIRY_SECONDS;
            let expired = stale && entry.touched != check;
            if expired || entry.remove_now || flags.end_reporting || flags.end_all {
                if entry.reported {
                    end_object_props(entry, descs, out);
                }
                if expired || entry.remove_now || flags.end_all {
                    pushers.remove(entry.key);
                    return false;
                }
            } else if !entry.reported && flags.active {
                begin_object_props(entry, descs, out);
            }
            true
        });
    }

    /// `CCollisionManager::AddEnvironmentalProperties` and the environment part of
    /// `ReportCollision`: the cell's material is merged into the list, and the properties the
    /// merge changed are listed. Leaving the environment ends its list.
    fn update_environment(
        &mut self,
        in_environment: bool,
        material: Option<&PhysicsMaterial>,
        out: &mut Vec<CollideProperty>,
    ) {
        if !in_environment {
            for mut entry in self.environment.drain(..) {
                entry.collide_type = CollideType::End;
                out.push(entry);
            }
            self.env_updated = false;
            return;
        }
        mark_all_end(&mut self.environment);
        if let Some(material) = material.filter(|m| m.id != 0) {
            if update_property_array(
                &material.groups,
                &mut self.environment,
                &mut self.master_id,
                self.descs,
            ) {
                self.env_updated = true;
            }
        }
        if remove_end(&mut self.environment) {
            self.env_updated = true;
        }
        if self.env_updated {
            out.extend(
                self.environment
                    .iter()
                    .filter(|entry| entry.collide_type != CollideType::Same)
                    .filter(|entry| self.descs.reported(entry.name()))
                    .cloned(),
            );
            self.env_updated = false;
        }
    }

    /// `CCollisionManager::EndEnvironmentalProperties`: every entry is listed as ended, and the
    /// list is dropped.
    fn end_environment(&mut self, out: &mut Vec<CollideProperty>) {
        for mut entry in self.environment.drain(..) {
            entry.collide_type = CollideType::End;
            out.push(entry);
        }
        self.env_updated = false;
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

/// `CollidePropertyList::BeginPropertyCollision`: each property reported to the weenie is marked
/// as begun and listed.
fn begin_object_props(entry: &mut ObjectProps, descs: PropertyDescs, out: &mut CollisionReport) {
    for prop in &mut entry.props {
        if descs.reported(prop.name()) {
            prop.collide_type = CollideType::Begin;
            out.object_properties.push(prop.clone());
        }
    }
    entry.reported = true;
}

/// `CollidePropertyList::EndPropertyCollision`: each property reported to the weenie is marked as
/// ended and listed.
fn end_object_props(entry: &mut ObjectProps, descs: PropertyDescs, out: &mut CollisionReport) {
    for prop in &mut entry.props {
        if descs.reported(prop.name()) {
            prop.collide_type = CollideType::End;
            out.object_properties.push(prop.clone());
        }
    }
    entry.reported = false;
}

/// The parts of the manager a material layer's report writes besides its own list.
struct Layers<'a> {
    terrain: &'a mut TerrainTypes,
    descs: PropertyDescs,
    next_id: &'a mut u64,
}

/// `ReportWaterProperties` and `ReportSurfaceProperties`: each entry that is not the same
/// updates the terrain when it is the terrain-type property, and is listed when the table reports
/// it to the weenie.
fn list_properties(
    kind: Kind,
    props: &[CollideProperty],
    terrain: &mut TerrainTypes,
    descs: PropertyDescs,
    out: &mut Vec<CollideProperty>,
) {
    for entry in props {
        if entry.collide_type == CollideType::Same {
            continue;
        }
        if entry.name() == TERRAIN_TYPE_PROPERTY {
            match kind {
                Kind::Water => terrain.water_property(entry),
                Kind::Landscape => terrain.land_property(entry),
            }
        }
        if descs.reported(entry.name()) {
            out.push(entry.clone());
        }
    }
}

/// `CCollisionManager::EndInWater` and `EndOnLandscape`: every entry ends and is listed, then the
/// ended entries are dropped.
fn end_layer(
    kind: Kind,
    layer: &mut MaterialLayer,
    terrain: &mut TerrainTypes,
    descs: PropertyDescs,
    out: &mut Vec<CollideProperty>,
) {
    mark_all_end(&mut layer.props);
    list_properties(kind, &layer.props, terrain, descs, out);
    remove_end(&mut layer.props);
}

/// `ChangePhysicsMaterial` followed by the layer's list and `RemoveEndElements`: the layer's
/// properties become the new material's, and the changes are listed.
fn change_material(
    kind: Kind,
    layer: &mut MaterialLayer,
    material: Option<&PhysicsMaterial>,
    layers: Layers<'_>,
    out: &mut Vec<CollideProperty>,
) {
    mark_all_end(&mut layer.props);
    if let Some(material) = material.filter(|m| m.id != 0) {
        update_property_array(
            &material.groups,
            &mut layer.props,
            layers.next_id,
            layers.descs,
        );
    }
    list_properties(kind, &layer.props, layers.terrain, layers.descs, out);
    remove_end(&mut layer.props);
}

/// One layer's part of `ReportCollision`: a layer the body is not in ends if it had begun; one it
/// is in begins with its material, and a new material replaces the old one's properties.
fn update_material_layer(
    kind: Kind,
    layer: &mut MaterialLayer,
    in_layer: bool,
    material: Option<&PhysicsMaterial>,
    layers: Layers<'_>,
    out: &mut Vec<CollideProperty>,
) {
    if !in_layer {
        if layer.reported_in {
            end_layer(kind, layer, layers.terrain, layers.descs, out);
        }
        layer.reported_in = false;
        return;
    }
    let id = material.map_or(0, |m| m.id);
    if !layer.reported_in || id != layer.reported_material {
        change_material(kind, layer, material, layers, out);
        layer.reported_material = id;
    }
    layer.reported_in = true;
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
