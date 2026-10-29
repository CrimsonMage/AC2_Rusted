//! `--crowd N`: N extra objects standing on a grid around the player, each wearing the next
//! race and sex model of the race table. They are remote objects in the frame, idle (no pose,
//! no motion), so the view draws them through the path it draws the client's remotes by.

use std::sync::Arc;

use ac2_client::{ObjectFrame, PlayerFrame};
use ac2_physics::{Frame, Heading, Quaternion, Vector};
use ac2_weenie::Appearance;

use crate::races::PlayerModel;

/// The first crowd object's id; the player is 1.
const FIRST_ID: u64 = 0x100;
/// The grid's spacing in metres.
const SPACING: f32 = 3.0;
/// Each object's fixed heading step, so the crowd does not face one way.
const HEADING_STEP: f32 = 37.0;

/// `count` objects on a square grid centred on `player`, cycling through `models`. The grid's
/// side is even, so the player stands between its middle rows. `ground` gives the terrain under
/// an origin (relative to the player's landblock); an object without terrain stands at the
/// player's height.
pub fn objects(
    models: &[PlayerModel],
    count: u32,
    player: &PlayerFrame,
    ground: impl Fn([f32; 3]) -> Option<f32>,
) -> Vec<ObjectFrame> {
    if models.is_empty() {
        return Vec::new();
    }
    let mut side = (f64::from(count).sqrt().ceil() as u32).max(2);
    side += side % 2;
    let half = side as f32 / 2.0 - 0.5;
    (0..count)
        .map(|index| {
            let visual = models[index as usize % models.len()].visual.raw();
            let x = player.origin[0] + ((index % side) as f32 - half) * SPACING;
            let y = player.origin[1] + ((index / side) as f32 - half) * SPACING;
            let z = ground([x, y, player.origin[2]]).unwrap_or(player.origin[2]);
            let heading = (index as f32 * HEADING_STEP) % 360.0;
            let mut frame = Frame::new(Vector::new(0.0, 0.0, 0.0), Quaternion::IDENTITY);
            frame.set_heading(Heading::from_degrees(heading));
            let q = frame.q;
            ObjectFrame {
                id: FIRST_ID + u64::from(index),
                visual: Some(visual),
                setup: None,
                name: None,
                stack_name: None,
                cell: player.cell,
                origin: [x, y, z],
                rotation: [q.w, q.x, q.y, q.z],
                heading_degrees: Some(heading),
                in_world: true,
                lost_cell: None,
                active: true,
                velocity: [0.0; 3],
                contact: true,
                mode: None,
                animated_mode: None,
                motion: [0.0; 3],
                holder: None,
                fx_follow: None,
                container: None,
                wielder: None,
                appearance: Arc::new(Appearance {
                    parent: visual,
                    ..Appearance::default()
                }),
                pose: Vec::new(),
                anim_scale: [1.0; 4],
                body_scale: [1.0; 3],
                adaptive_degrade: true,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_core::DataId;

    fn model(visual: u32) -> PlayerModel {
        PlayerModel {
            race: 0x1,
            sex: 0x1000,
            phys_obj: 0x4700_0001,
            visual: DataId::new(visual),
        }
    }

    #[test]
    fn the_crowd_cycles_the_models_on_a_grid_around_the_player() {
        let models = [model(0x1F00_0001), model(0x1F00_0002), model(0x1F00_0003)];
        let player = PlayerFrame {
            id: 1,
            cell: 0x7F7F_0001,
            origin: [80.0, 90.0, 12.0],
            rotation: [1.0, 0.0, 0.0, 0.0],
            heading_degrees: 0.0,
            visual: Some(0x1F00_0001),
            velocity: [0.0; 3],
            contact: true,
            pose: Vec::new(),
            appearance: Arc::default(),
            hidden: false,
        };
        let crowd = objects(&models, 16, &player, |_| None);
        assert_eq!(crowd.len(), 16);
        let mut ids: Vec<u64> = crowd.iter().map(|o| o.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 16);
        assert!(crowd.iter().all(|o| o.in_world && o.holder.is_none()));
        for (index, object) in crowd.iter().enumerate() {
            assert_eq!(object.visual, Some(models[index % 3].visual.raw()));
            assert_eq!(object.origin[2], 12.0);
            let offset = (object.origin[0] - 80.0).hypot(object.origin[1] - 90.0);
            assert!(offset > 0.0 && offset < 10.0, "offset {offset}");
        }
        // The grid is centred: its objects average to the player's position.
        let mean = crowd.iter().fold([0.0_f32; 2], |acc, o| {
            [acc[0] + o.origin[0] / 16.0, acc[1] + o.origin[1] / 16.0]
        });
        assert!((mean[0] - 80.0).abs() < 1e-3 && (mean[1] - 90.0).abs() < 1e-3);
        assert!(objects(&[], 10, &player, |_| None).is_empty());
    }
}
