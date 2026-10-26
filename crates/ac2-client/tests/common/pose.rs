//! What a drawn player looks like (snapshot tests): the pixels its box covers, and the height
//! of its hand against its root bone, posed and at bind, from the renderer's own Setup.

use ac2_anim::{Ac2Setup, Ac2Transform};
use ac2_client::{local_visual_desc, PlayerFrame};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_math::Vec3;
use ac2_render::object::{load_object_model_with, RenderSettings};
use ac2_render::Camera;

/// A player's height in metres, for its screen box.
pub const PLAYER_HEIGHT: f32 = 1.8;

/// The player's pixels inside its screen box: the fraction that is near white and the mean
/// saturation (0 grey to 1 pure colour).
#[derive(Debug, Clone, Copy)]
pub struct Look {
    pub white: f32,
    pub saturation: f32,
    pub pixels: usize,
}

/// The screen box (x0, y0, x1, y1) of a figure standing at `feet` and `height` tall, in the
/// `size` image, clamped to it.
pub fn screen_box(
    camera: &Camera,
    aspect: f32,
    size: (u32, u32),
    feet: Vec3,
    height: f32,
) -> (u32, u32, u32, u32) {
    let project = |p: Vec3| {
        let c = camera.view_projection(aspect) * p.extend(1.0);
        (
            (c.x / c.w + 1.0) * 0.5 * size.0 as f32,
            (1.0 - c.y / c.w) * 0.5 * size.1 as f32,
        )
    };
    let (fx, fy) = project(feet);
    let (hx, hy) = project(feet + Vec3::new(0.0, 0.0, height));
    let half = ((fy - hy) * 0.35).max(4.0);
    let cx = (fx + hx) / 2.0;
    let x0 = (cx - half).clamp(0.0, size.0 as f32) as u32;
    let x1 = (cx + half).clamp(0.0, size.0 as f32) as u32;
    let y0 = hy.clamp(0.0, size.1 as f32) as u32;
    let y1 = fy.clamp(0.0, size.1 as f32) as u32;
    (x0, y0, x1, y1)
}

/// The look of the pixels in `rect` (x0, y0, x1, y1) of a `width`-pixel RGBA image.
pub fn look(rgba: &[u8], width: u32, rect: (u32, u32, u32, u32)) -> Look {
    let (mut white, mut saturation, mut pixels) = (0usize, 0.0f32, 0usize);
    for y in rect.1..rect.3 {
        for x in rect.0..rect.2 {
            let i = (y as usize * width as usize + x as usize) * 4;
            let [r, g, b] = [rgba[i], rgba[i + 1], rgba[i + 2]];
            let (max, min) = (r.max(g).max(b), r.min(g).min(b));
            pixels += 1;
            if min >= 225 {
                white += 1;
            }
            if max > 0 {
                saturation += f32::from(max - min) / f32::from(max);
            }
        }
    }
    let n = pixels.max(1) as f32;
    Look {
        white: white as f32 / n,
        saturation: saturation / n,
        pixels,
    }
}

/// The height of the player's hand above its root bone, posed and at bind, in metres (model
/// space). The hand is the leaf bone furthest along +x at bind (the right arm), the root is
/// bone 0. `None` when the player has no drawn model yet.
pub fn hand_height(store: &ResourceStore, player: &PlayerFrame) -> Option<(f32, f32)> {
    let visual = player.visual?;
    let local = local_visual_desc(&player.appearance);
    let model = load_object_model_with(
        store,
        DataId::new(visual),
        Some(&local),
        &RenderSettings::default(),
    )
    .ok()??;
    let setup_did = model.skeleton.as_ref()?.groups.first()?.setup;
    let setup = store
        .load_cached_packed::<Ac2Setup>(TypedResourceRef::new(DatFileKind::Portal, setup_did, 0))
        .ok()??;
    let rest = setup.rest_pose();
    let bones: Vec<Ac2Transform> = player.pose.iter().map(|p| p.transform()).collect();
    let posed = setup.world_pose(&bones);
    let parents: Vec<Option<usize>> = setup.bones.iter().map(|b| b.parent).collect();
    let hand = (0..parents.len())
        .filter(|&i| !parents.contains(&Some(i)))
        .max_by(|&a, &b| rest[a].offset.x.total_cmp(&rest[b].offset.x))?;
    Some((
        posed[hand].offset.z - posed[0].offset.z,
        rest[hand].offset.z - rest[0].offset.z,
    ))
}
