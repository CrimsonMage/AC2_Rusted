//! The client's frame, offline: what `ac2_client::Client::frame` would hand the view for a
//! player standing in the world, built from the viewer's own state. The view draws it as it
//! draws the client's.

use ac2_client::{ClientFrame, RadarFrame};
use ac2_core::DataId;
use ac2_smartbox::CameraFrame;

use crate::fly::FlyCamera;
use crate::panel::Mode;
use crate::walk::Walk;

/// The radar's range in the viewer: the client reads it from the player's script.
const RADAR_RANGE: f32 = 75.0;
/// The vertical field of view the player camera shows (`CameraManager::m_fov`).
const FLY_FOV: f32 = 60.0;

/// The fly camera as a camera frame: its position relative to its landblock, and the point one
/// metre ahead (also relative to that landblock).
pub fn fly_camera_frame(fly: &FlyCamera, block_size: f32) -> CameraFrame {
    let block = |axis: f32| (axis / block_size).floor().clamp(0.0, 254.0) as u8;
    let (bx, by) = (block(fly.position.x), block(fly.position.y));
    let cell = (u32::from(bx) << 24) | (u32::from(by) << 16) | 1;
    let origin = [
        fly.position.x - f32::from(bx) * block_size,
        fly.position.y - f32::from(by) * block_size,
        fly.position.z,
    ];
    let ahead = fly.forward();
    CameraFrame {
        cell,
        origin,
        rotation: [1.0, 0.0, 0.0, 0.0],
        target: [
            origin[0] + ahead.x,
            origin[1] + ahead.y,
            origin[2] + ahead.z,
        ],
        fov: FLY_FOV,
        first_person: false,
        flash: [0.0; 4],
    }
}

/// One frame for the view: the time of day in `server_time` (the view lights the scene by it),
/// the player or the fly camera, and the radar over the player's heading.
pub fn client_frame(
    mode: Mode,
    walk: &Walk,
    fly: &FlyCamera,
    visual: Option<DataId>,
    time_of_day: f64,
) -> ClientFrame {
    let block_size = walk.block_size();
    let player = match (mode, visual) {
        (Mode::Walk, Some(visual)) => walk.player_frame(visual),
        _ => None,
    };
    let camera = match (mode, walk.camera_frame()) {
        (Mode::Walk, Some(frame)) if player.is_some() => frame,
        _ => fly_camera_frame(fly, block_size),
    };
    let blocks = if player.is_some() {
        walk.resident_blocks()
    } else {
        Vec::new()
    };
    ClientFrame {
        server_time: time_of_day,
        cell_id: Some(camera.cell),
        in_world: true,
        radar: RadarFrame {
            range: RADAR_RANGE,
            heading_degrees: player.as_ref().map(|p| p.heading_degrees),
            ..RadarFrame::default()
        },
        player,
        camera: Some(camera),
        blocks,
        ..ClientFrame::default()
    }
}
