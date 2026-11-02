//! The player's entry in the world's object lists.

use ac2_smartbox::body_shadow::sync_body_shadow;

use crate::Client;

impl Client {
    /// `CPhysicsObj::add_to_cells` / `remove_from_cells` for the player: while the body is in the
    /// world with a shape it is listed in the cells it touches (`CObjCell::AddObject`), like any
    /// object, so the creatures that enter the world or walk after it keep clear of it
    /// (`CPhysicsObj::EnterWorld`'s sliding placement, `CTransition::find_placement_position`).
    /// The body's own transitions skip its entry (`CObjCell::find_obj_collisions`).
    pub(crate) fn sync_player_shadow(&mut self) {
        let Some(player) = &mut self.player else {
            return;
        };
        let (id, in_world) = (player.id, player.in_world);
        let Some(body) = &mut player.body else {
            return;
        };
        sync_body_shadow(
            id,
            Some(&mut body.physics),
            body.object.as_ref(),
            in_world,
            false,
            &mut body.shadow,
            &mut self.terrain,
        );
    }
}
