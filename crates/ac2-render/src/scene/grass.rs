//! The frame's grass frills: retail's ring (`FrillRender::RefreshFrillMesh`), each drawn
//! `FrillSettings::copies` times jittered over the ground under it, then the client's far ring
//! (`AGENTS.md` exceptions). Each frill keeps the landscape self shadow under where it stands.

use ac2_math::{Mat4, Quat, Vec3};

use super::{BlockKey, FramePlacement, Placement, WorldScene};
use crate::frill::{frill_normal, grass_copy, grass_hash, grass_radii};
use crate::object::DrawDetail;

impl WorldScene {
    /// The placements of the near frills `chosen` (with their copies) and the far frills
    /// `far`, indices into `frills`, sorted by model so each model draws as one instanced batch.
    pub(super) fn frill_placements(
        &self,
        frills: &[(BlockKey, &Placement)],
        chosen: &[usize],
        far: &[usize],
        shadow_bit: Option<u32>,
    ) -> Vec<FramePlacement> {
        let (near, far_radius) = grass_radii(&self.frill);
        let copies = self.frill.copies.max(1);
        let mut out = Vec::with_capacity(chosen.len() * copies as usize + far.len());
        let mut place = |key: BlockKey, p: &Placement, world: Mat4| {
            let origin = world.w_axis.truncate();
            let frill_shade = self.heights.get(&key).map_or(1.0, |heights| {
                heights.self_shadow(origin.x, origin.y, shadow_bit)
            });
            out.push(FramePlacement {
                visual: p.visual,
                world,
                frill_normal: Some(frill_normal(&world)),
                frill_shade,
                detail: DrawDetail::FULL,
                alpha: 1.0,
                grass: [near, far_radius, grass_hash(origin)],
            });
        };
        for &index in chosen {
            let (key, p) = frills[index];
            let origin = p.world.w_axis.truncate();
            let lift = origin.z - ground_or(self, key, origin);
            for copy in 0..copies {
                if copy == 0 {
                    place(key, p, p.world);
                    continue;
                }
                let jitter = grass_copy(origin, copy);
                let (x, y) = (origin.x + jitter.offset.x, origin.y + jitter.offset.y);
                // The ground under the copy, from its own block or whichever resident block it
                // strayed into; a copy off every resident block is left out.
                let ground = self
                    .heights
                    .get(&key)
                    .and_then(|h| h.height_at(x, y))
                    .or_else(|| self.heights.values().find_map(|h| h.height_at(x, y)));
                let Some(z) = ground else {
                    continue;
                };
                let mut world = p.world
                    * Mat4::from_quat(Quat::from_rotation_z(jitter.yaw))
                    * Mat4::from_scale(Vec3::splat(jitter.scale));
                world.w_axis = Vec3::new(x, y, z + lift).extend(1.0);
                place(key, p, world);
            }
        }
        for &index in far {
            let (key, p) = frills[index];
            place(key, p, p.world);
        }
        out.sort_by_key(|p| p.visual);
        out
    }
}

/// The ground height under `origin` on its block, or `origin`'s own height when the block has
/// none, so a frill authored above or below the ground keeps that lift on its copies.
fn ground_or(scene: &WorldScene, key: BlockKey, origin: Vec3) -> f32 {
    scene
        .heights
        .get(&key)
        .and_then(|h| h.height_at(origin.x, origin.y))
        .unwrap_or(origin.z)
}
