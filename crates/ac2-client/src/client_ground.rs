//! Test access to where a body stands: `CPhysicsObj::GetHeightAboveWalkable` under a body in the
//! client's physics world (live tests assert that bodies stand on their floor).

use crate::client::Client;

/// What the physics world says about a body's footing.
#[derive(Clone, Copy, Debug)]
pub struct Footing {
    /// `CPhysicsObj::GetHeightAboveWalkable`: whether a walkable surface was found under it.
    pub surface_found: bool,
    /// The metres between the body's lowest sphere and that surface (zero in contact).
    pub height: f32,
    /// `CPhysicsObj::on_ground`.
    pub on_ground: bool,
    /// `CPhysicsObj::off_ground`: not in contact (falling or floating).
    pub off_ground: bool,
}

impl Client {
    /// The footing of the player (`id` is the player's id) or of the remote object `id`; none
    /// when that body is not in the client's world.
    pub fn footing(&mut self, id: u64) -> Option<Footing> {
        let terrain = &self.terrain;
        let body = match self.player.as_mut() {
            Some(p) if p.id == id => p.body.as_mut().map(|b| &mut b.physics)?,
            _ => self.remotes.body_mut(id)?,
        };
        let (surface_found, height) = body.get_height_above_walkable(terrain);
        Some(Footing {
            surface_found,
            height,
            on_ground: body.on_ground(),
            off_ground: body.off_ground(),
        })
    }
}
