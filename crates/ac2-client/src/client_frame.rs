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
            let state = p.body.as_ref()?.physics.movement_state();
            Some(PlayerFrame {
                id: p.id,
                cell: state.cell_id,
                origin: state.origin,
                rotation: state.rotation,
                heading_degrees: state.heading_degrees,
                visual: p.visual,
                velocity: state.velocity,
                contact: state.contact,
                pose: p
                    .body
                    .as_ref()
                    .map(|b| crate::motion::pose(&b.physics))
                    .unwrap_or_default(),
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
            cell_id: player.as_ref().map(|p| p.cell),
            in_world,
            blocking_for_cells: self.cells.is_blocking(),
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
        }
    }

    /// The scene camera's render position, and the point it looks at: the camera distance
    /// ahead along its view axis (the player's head in third person).
    fn camera_frame(&self) -> Option<CameraFrame> {
        let subject = self.camera_subject(true)?;
        let render = self.scene.render;
        let forward = render.frame.vector_heading();
        let distance =
            f64::from(self.camera.targeting.camera_distance) * f64::from(subject.scale_z);
        let o = render.frame.origin;
        let ahead =
            |origin: f32, axis: f32| (f64::from(origin) + f64::from(axis) * distance) as f32;
        let q = render.frame.q;
        Some(CameraFrame {
            cell: render.cell_id,
            origin: [o.x, o.y, o.z],
            rotation: [q.w, q.x, q.y, q.z],
            target: [
                ahead(o.x, forward.x),
                ahead(o.y, forward.y),
                ahead(o.z, forward.z),
            ],
            fov: self.scene.fov,
            first_person: self.camera.first_person,
        })
    }
}
