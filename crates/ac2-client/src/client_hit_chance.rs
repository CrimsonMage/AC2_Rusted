//! Hit chance on name plates (a modern addition, AGENTS.md). The session keeps the hit chance of
//! every creature profile it receives (`ac2_weenie::OnlineSession::hit_chance`, the retail
//! profile's "Chance to hit this target: " row). With `ShowHitChanceOnNameplates` on, the frame
//! carries them and a creature's name plate shows its percentage after its name
//! (`hud_player::name_looks`). With `ExamineNearbyForHitChance` on, the client asks the server for
//! the profile of one nearby creature at most once a second: a mobile object other than the
//! player within 30 m on the ground plane, in front of the camera, whose value is unknown or more
//! than 60 seconds old, the nearest first. The request is the examination panel's own for an
//! object; the selection, the player's examination request and the panel are left alone
//! (`OnlineSession::examine_for_hit_chance`).

use std::collections::BTreeMap;

use crate::client::Client;
use crate::frame::ClientFrame;

/// How far a creature may be for its examination, metres on the ground plane.
pub const EXAMINE_RANGE: f32 = 30.0;
/// How old a hit chance may be before it is asked for again, seconds.
pub const MAX_AGE: f64 = 60.0;
/// The shortest time between two of the client's own requests, seconds.
pub const REQUEST_INTERVAL: f64 = 1.0;
/// The widest display the view cone allows for (the horizontal field of view is the vertical
/// one widened by this aspect).
const VIEW_ASPECT: f32 = 16.0 / 9.0;

/// A cell's point in the world: its landblock's corner plus the origin (AC axes).
fn global(cell: u32, origin: [f32; 3]) -> [f32; 2] {
    let x = ((cell >> 24) & 0xFF) as f32 * 192.0;
    let y = ((cell >> 16) & 0xFF) as f32 * 192.0;
    [x + origin[0], y + origin[1]]
}

/// Whether `point` (world, ground plane) lies in the camera's horizontal view cone.
fn in_view(camera: &crate::CameraFrame, point: [f32; 2]) -> bool {
    let eye = global(camera.cell, camera.origin);
    let [w, x, y, z] = camera.rotation;
    // The camera's local +y (the view direction) rotated into the world, on the ground plane.
    let forward = [2.0 * (x * y - w * z), 1.0 - 2.0 * (x * x + z * z)];
    let to = [point[0] - eye[0], point[1] - eye[1]];
    let (f, t) = (forward[0].hypot(forward[1]), to[0].hypot(to[1]));
    if f <= f32::EPSILON || t <= f32::EPSILON {
        return true;
    }
    let cos = (forward[0] * to[0] + forward[1] * to[1]) / (f * t);
    let half = ((camera.fov.to_radians() * 0.5).tan() * VIEW_ASPECT).atan();
    cos >= half.cos()
}

impl Client {
    /// The hit chances the frame carries: all known ones while the player shows them.
    pub(crate) fn hit_chance_frame(&self) -> BTreeMap<u64, u32> {
        let show = self
            .prefs()
            .is_some_and(|p| ac2_ui::hit_chance_pref::show_hit_chance(p.bank()));
        if !show {
            return BTreeMap::new();
        }
        self.session.hit_chances().collect()
    }

    /// `ExamineNearbyForHitChance`: one examination request for the nearest nearby creature in
    /// view whose hit chance is unknown or stale, at most once a second. Call once per frame with
    /// the frame. Whether a request went out.
    pub fn examine_nearby_for_hit_chance(&mut self, frame: &ClientFrame) -> bool {
        let on = self
            .prefs()
            .is_some_and(|p| ac2_ui::hit_chance_pref::examine_nearby(p.bank()));
        let now = self.session.cur_time();
        if !on || now - self.hit_chance_asked < REQUEST_INTERVAL {
            return false;
        }
        let (Some(player), Some(camera)) = (frame.player.as_ref(), frame.camera.as_ref()) else {
            return false;
        };
        let here = global(player.cell, player.origin);
        let nearest = frame
            .objects
            .iter()
            .filter(|o| o.id != player.id && o.in_world && o.adaptive_degrade)
            .filter(|o| o.container.is_none() && o.wielder.is_none() && o.holder.is_none())
            .filter_map(|o| {
                let at = global(o.cell, o.origin);
                let distance = (at[0] - here[0]).hypot(at[1] - here[1]);
                (distance <= EXAMINE_RANGE && in_view(camera, at)).then_some((distance, o.id))
            })
            .filter(|(_, id)| self.session.hit_chance_stale(*id, MAX_AGE))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((_, id)) = nearest else {
            return false;
        };
        if !self.session.examine_for_hit_chance(id) {
            return false;
        }
        self.hit_chance_asked = now;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(rotation: [f32; 4]) -> crate::CameraFrame {
        crate::CameraFrame {
            cell: 0x7D64_0001,
            origin: [100.0, 100.0, 0.0],
            rotation,
            target: [0.0; 3],
            fov: 60.0,
            first_person: false,
            flash: [0.0; 4],
            disable_degrades: false,
        }
    }

    #[test]
    fn the_view_cone_is_the_cameras_heading() {
        // Identity: the camera looks along +y.
        let looking_north = camera([1.0, 0.0, 0.0, 0.0]);
        let base = global(0x7D64_0001, [100.0, 100.0, 0.0]);
        assert!(in_view(&looking_north, [base[0], base[1] + 20.0]));
        assert!(in_view(&looking_north, [base[0] + 10.0, base[1] + 20.0]));
        assert!(!in_view(&looking_north, [base[0], base[1] - 20.0]));
        assert!(!in_view(&looking_north, [base[0] + 20.0, base[1]]));
        // A half turn about z looks along -y.
        let looking_south = camera([0.0, 0.0, 0.0, 1.0]);
        assert!(in_view(&looking_south, [base[0], base[1] - 20.0]));
        assert!(!in_view(&looking_south, [base[0], base[1] + 20.0]));
    }

    #[test]
    fn points_across_landblocks_are_in_one_space() {
        let a = global(0x7D64_0001, [191.0, 5.0, 0.0]);
        let b = global(0x7E64_0001, [1.0, 5.0, 0.0]);
        assert!(((b[0] - a[0]) - 2.0).abs() < 1e-3);
        assert_eq!(a[1], b[1]);
    }
}
