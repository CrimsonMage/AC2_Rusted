//! Pusher properties: velocity and acceleration fields a nearby object's physics material puts on
//! a body (wind, currents, updrafts). `CCollisionManager::AddPusherProperties` reads the
//! `0x40000040..0x4000004b` vector properties of an object the body touches into one
//! `PusherValues`, keyed by the object; `CCollisionManager::UpdatePusherValues` sums every
//! pusher's fields for the body's element, and `CPhysicsObj::CalcExternalForces` integrates them.

use ac2_db::Ac2PropertyValue;

use crate::fpu::{ld, st, F_EPSILON};
use crate::motion_interp::{ELEMENT_AIR, ELEMENT_GROUND, ELEMENT_SEA_FLOOR, ELEMENT_WATER};
use crate::position::Position;
use crate::vector::Vector;

/// The first and last pusher vector properties: general, general acceleration, ..., water
/// acceleration (twelve in all).
const GENERAL_VELOCITY_WALKS_ON_AIR: u32 = 0x4000_0040;
const GENERAL_ACCELERATION_WALKS_ON_AIR: u32 = 0x4000_0041;
const GENERAL_VELOCITY: u32 = 0x4000_0042;
const GENERAL_ACCELERATION: u32 = 0x4000_0043;
const EARTH_VELOCITY: u32 = 0x4000_0044;
const EARTH_ACCELERATION: u32 = 0x4000_0045;
const AIR_VELOCITY: u32 = 0x4000_0046;
const AIR_ACCELERATION: u32 = 0x4000_0047;
const SEAFLOOR_VELOCITY: u32 = 0x4000_0048;
const SEAFLOOR_ACCELERATION: u32 = 0x4000_0049;
const WATER_VELOCITY: u32 = 0x4000_004a;
const WATER_ACCELERATION: u32 = 0x4000_004b;
/// The bool property that makes the vectors object-local (turned by the object's frame).
const LOCAL_SPACE: u32 = 0x4000_004c;

/// `PusherValues`: what one object pushes with, per element.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PusherValues {
    /// `m_walk_on_air`.
    pub walk_on_air: bool,
    pub general_vel: Vector,
    pub general_acl: Vector,
    pub earth_vel: Vector,
    pub earth_acl: Vector,
    pub air_vel: Vector,
    pub air_acl: Vector,
    pub water_vel: Vector,
    pub water_acl: Vector,
    pub seafloor_vel: Vector,
    pub seafloor_acl: Vector,
}

/// What `UpdatePusherValues` hands `CalcExternalForces`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PusherSum {
    /// `*walkonair`: the last pusher that walks on air sets it.
    pub walk_on_air: bool,
    pub velocity: Vector,
    pub acceleration: Vector,
}

fn add(a: &mut Vector, b: Vector) {
    *a = Vector::new(
        st(ld(a.x) + ld(b.x)),
        st(ld(a.y) + ld(b.y)),
        st(ld(a.z) + ld(b.z)),
    );
}

/// `CCollisionManager::m_pushers`, a `PHashTable<ulong, PusherValues>` keyed by the touched
/// object. Retail iterates its buckets in address order; the order here is insertion order.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pushers {
    entries: Vec<(u64, PusherValues)>,
}

impl Pushers {
    /// `PHashTable::GetCount() == 0`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The values `key` pushes with.
    pub fn get(&self, key: u64) -> Option<&PusherValues> {
        self.entries.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    /// `CCollisionManager::RemovePropertiesAndErase`'s pusher part: the object stops pushing.
    pub fn remove(&mut self, key: u64) -> bool {
        match self.entries.iter().position(|(k, _)| *k == key) {
            Some(index) => {
                self.entries.remove(index);
                true
            }
            None => false,
        }
    }

    /// `CCollisionManager::AddPusherProperties`: reads `properties` (the touched object's
    /// `CollidePropertyList`) into one `PusherValues` and adds it under `key`, unless the object
    /// already pushes or no property had a vector of at least `F_EPSILON` on some axis. `object`
    /// is the touched object's position for `LOCAL_SPACE` vectors. Returns whether it was added.
    pub fn add_properties(
        &mut self,
        key: u64,
        object: Option<&Position>,
        properties: &[Ac2PropertyValue],
    ) -> bool {
        let mut local_space = false;
        for property in properties {
            if property.name == LOCAL_SPACE {
                if let Some(word) = property.word() {
                    local_space = word != 0;
                }
            }
        }
        let mut pusher = PusherValues::default();
        let mut found = false;
        for property in properties {
            // `BaseProperty::GetValue(Vector&)`: a vector-typed value only.
            let Some([x, y, z]) = property.vector() else {
                continue;
            };
            let mut v = Vector::new(x, y, z);
            if !(ld(F_EPSILON) <= ld(v.x).abs()
                || ld(F_EPSILON) <= ld(v.y).abs()
                || ld(F_EPSILON) <= ld(v.z).abs())
            {
                continue;
            }
            let name = property.name;
            if !(GENERAL_VELOCITY_WALKS_ON_AIR..=WATER_ACCELERATION).contains(&name) {
                continue;
            }
            // The first four cases set `bVar4` before the conversion; every case but the water
            // velocity does (retail's `0x4000004a` never counts toward adding the pusher).
            if name != WATER_VELOCITY {
                found = true;
            }
            if local_space {
                if let Some(position) = object {
                    v = position.local_to_global_vec(v);
                }
            }
            match name {
                GENERAL_VELOCITY_WALKS_ON_AIR => {
                    pusher.walk_on_air = true;
                    add(&mut pusher.general_vel, v);
                }
                GENERAL_ACCELERATION_WALKS_ON_AIR => {
                    pusher.walk_on_air = true;
                    add(&mut pusher.general_acl, v);
                }
                GENERAL_VELOCITY => add(&mut pusher.general_vel, v),
                GENERAL_ACCELERATION => add(&mut pusher.general_acl, v),
                EARTH_VELOCITY => add(&mut pusher.earth_vel, v),
                EARTH_ACCELERATION => add(&mut pusher.earth_acl, v),
                AIR_VELOCITY => add(&mut pusher.air_vel, v),
                AIR_ACCELERATION => add(&mut pusher.air_acl, v),
                SEAFLOOR_VELOCITY => add(&mut pusher.seafloor_vel, v),
                SEAFLOOR_ACCELERATION => add(&mut pusher.seafloor_acl, v),
                WATER_VELOCITY => add(&mut pusher.water_vel, v),
                WATER_ACCELERATION => add(&mut pusher.water_acl, v),
                _ => {}
            }
        }
        if !found || self.get(key).is_some() {
            return false;
        }
        self.entries.push((key, pusher));
        true
    }

    /// `CCollisionManager::UpdatePusherValues`: for each pusher in order, one that walks on air
    /// replaces the running sums with its general fields (and is then added on top, as retail
    /// does), then the general and the `element`'s fields are added.
    pub fn update_values(&self, element: u32) -> PusherSum {
        let mut sum = PusherSum::default();
        for (_, p) in &self.entries {
            if p.walk_on_air {
                sum.walk_on_air = true;
                sum.velocity = p.general_vel;
                sum.acceleration = p.general_acl;
            }
            add(&mut sum.velocity, p.general_vel);
            add(&mut sum.acceleration, p.general_acl);
            let (vel, acl) = match element {
                ELEMENT_GROUND => (p.earth_vel, p.earth_acl),
                ELEMENT_AIR => (p.air_vel, p.air_acl),
                ELEMENT_WATER => (p.water_vel, p.water_acl),
                ELEMENT_SEA_FLOOR => (p.seafloor_vel, p.seafloor_acl),
                _ => continue,
            };
            add(&mut sum.velocity, vel);
            add(&mut sum.acceleration, acl);
        }
        sum
    }
}

#[cfg(test)]
#[path = "pushers_tests.rs"]
mod tests;
