//! The frame the renderer reads: [`Client::frame`] (the plain-data [`ClientFrame`]) and the
//! scene camera's frame.

use ac2_world::GameClock;

use crate::client::Client;
use crate::frame::{ClientFrame, PlayerFrame};
use crate::scene::CameraFrame;

impl Client {
    /// The frame for the renderer.
    pub fn frame(&self) -> ClientFrame {
        let server_time = self.session.cur_time();
        let in_world = self.player.as_ref().is_some_and(|p| p.in_world);
        let player = self.player.as_ref().and_then(|p| {
            let body = p.body.as_ref()?;
            let state = body.physics.movement_state();
            let (cell, origin, rotation) =
                ac2_physics::drawn_place(&body.physics, self.render_interpolation);
            Some(PlayerFrame {
                id: p.id,
                cell,
                origin,
                rotation,
                heading_degrees: state.heading_degrees,
                visual: p.visual,
                velocity: state.velocity,
                contact: state.contact,
                pose: ac2_physics::part_poses(&body.physics, self.pose_blend_time()),
                appearance: self
                    .session
                    .objects()
                    .iter()
                    .find(|o| o.id == p.id)
                    .map(|o| o.appearance.clone())
                    .unwrap_or_default(),
            })
        });
        let objects = self.object_frames();
        ClientFrame {
            server_time,
            time_of_day: self.game_clock.as_ref().map(GameClock::present_time_of_day),
            date_time: self.game_clock.as_ref().map(GameClock::date_time),
            cell_id: player.as_ref().map(|p| p.cell),
            in_world,
            blocking_for_cells: self.cells.is_blocking(),
            portal: self.portal_frame(),
            player,
            camera: self.camera_frame(),
            objects,
            blocks: self.cells.resident().collect(),
            selection: self.selection_frame(),
            player_vitals: *self.session.player_script().vitals(),
            inventory: self.inventory_frame(),
            progress: self.progress_frame(),
            radar: self.radar_frame(),
            markers: self.radar.markers.clone(),
            fx_looks: self.fx_objects.looks(),
        }
    }

    /// The scene camera's render position, and the point it looks at: the camera distance
    /// ahead along its view axis (the player's head in third person).
    fn camera_frame(&self) -> Option<CameraFrame> {
        let subject = self.camera_subject(true)?;
        Some(self.scene.frame(&self.camera, subject.scale_z))
    }
}
