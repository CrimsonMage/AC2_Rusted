//! A physics body's entry in the world's object lists (`CObjCell::AddObject` through
//! `CPhysicsObj::add_to_cells`), for every body that is in the world with a collision shape: a
//! remote object, and the player. Retail's player is a `CPhysicsObj` like any other, listed in
//! its cells, so the objects that enter the world or move after it (a creature's
//! `EnterWorld` with `SLIDE_SPF`, `CTransition::find_placement_position`) test the player's
//! spheres and keep clear of them.

use std::sync::Arc;

use ac2_physics::{ObjectShape, PhysicsBody, StaticObject, Terrain, Vector};

/// A body's collision shape as a world object: the Setup's shape and its root scale.
pub type BodyObject = (Arc<ObjectShape>, Vector);

/// Brings the body's entry in the world's object lists up to date: listed at its position while
/// it is `in_world` with a shape, moved when it moved, listed again when its scale changed, and
/// taken out otherwise. `guid` names the entry (a collision report lists it) and `shadow` is the
/// body's entry in the lists, which the body also keeps as its world id so a transition does not
/// test its own mover.
pub fn sync_body_shadow(
    guid: u64,
    mut body: Option<&mut PhysicsBody>,
    object: Option<&BodyObject>,
    in_world: bool,
    ethereal: bool,
    shadow: &mut Option<u32>,
    terrain: &mut Terrain,
) {
    // The entry's blocking mask is the body's (`CPhysicsObj::IsBlocking` reads m_blocking_mask),
    // so it is re-read on every sync: a body's bits can change between syncs.
    // The Setup's root scale times the body's own (`CPhysicsObj::SetScale`, which a script's
    // `CWRepository::SetScale` changes at run time): a new scale lists the object again.
    let wanted = match (body.as_deref(), object, in_world) {
        (Some(body), Some((shape, scale)), true) => {
            let own = body.effects_scale();
            let times = |a: f32, b: f32| (f64::from(a) * f64::from(b)) as f32;
            Some((
                body.position(),
                Arc::clone(shape),
                Vector::new(
                    times(scale.x, own.x),
                    times(scale.y, own.y),
                    times(scale.z, own.z),
                ),
                body.is_mobile(),
                body.blocking_bits(),
            ))
        }
        _ => None,
    };
    if let (Some((_, _, scale, _, _)), Some(id)) = (&wanted, *shadow) {
        if terrain
            .object_scale(id)
            .is_some_and(|listed| listed != *scale)
        {
            terrain.remove_object(id);
            *shadow = None;
        }
    }
    match (wanted, *shadow) {
        (Some((position, shape, scale, mobile, blocking)), None) => {
            // The object is named by its guid, so a collision report lists it.
            let mut entry = StaticObject::dynamic(position, scale, shape, mobile)
                .with_blocking_mask(blocking)
                .with_instance_id(guid);
            entry.set_ethereal(ethereal);
            let id = terrain.insert_object(entry);
            if let Some(body) = body.as_deref_mut() {
                terrain.set_object_ignores_collisions(id, body.ignores_collisions());
                terrain.set_object_collision(
                    id,
                    body.collision_state_ptr(),
                    body.current_properties(),
                );
                body.set_world_id(Some(id));
            }
            *shadow = Some(id);
        }
        (Some((position, _, _, _, blocking)), Some(id)) => {
            terrain.move_object(id, position);
            terrain.set_object_blocking(id, blocking);
            if let Some(body) = body.as_deref() {
                terrain.set_object_ignores_collisions(id, body.ignores_collisions());
                terrain.set_object_collision(
                    id,
                    body.collision_state_ptr(),
                    body.current_properties(),
                );
            }
        }
        (None, Some(id)) => {
            terrain.remove_object(id);
            *shadow = None;
            if let Some(body) = body {
                body.set_world_id(None);
            }
        }
        (None, None) => {}
    }
}
