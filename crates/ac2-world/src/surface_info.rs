//! Per-vertex AC2 `SurfaceInfo`: what `MaterialGroup::GetVertexInfo` picks for one landscape vertex
//! from its surface (or road) type, slope and global vertex coordinates. The renderer combines the
//! four corners of a terrain polygon into blended layers (`LScapeMaterialInfo::Create`).

use ac2_core::DataId;

use crate::surface_desc::{MaterialGroup, SurfaceType, VertexColorDesc, WeightedDid};

/// AC2 `SurfaceInfo`. Material DIDs are `RenderMaterial`s (`0x2B......`); alpha DIDs are
/// `RenderTexture` masks (`0x40......`). `None` means the slot is empty (`SurfaceInfo::SetToDefault`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ac2SurfaceInfo {
    pub base_material: Option<DataId>,
    /// Metres one repeat of the base texture covers.
    pub base_tiling: f32,
    pub detail_material: Option<DataId>,
    pub detail_tiling: f32,
    pub bump_material: Option<DataId>,
    pub bump_tiling: f32,
    pub corner_alpha: Option<DataId>,
    pub side_alpha: Option<DataId>,
    pub three_corner_alpha: Option<DataId>,
    pub diagonal_alpha: Option<DataId>,
    /// Linear RGBA vertex tint for the near mesh.
    pub vertex_color: [f32; 4],
    /// Tint for the far landscape mesh.
    pub far_vertex_color: [f32; 4],
}

impl Default for Ac2SurfaceInfo {
    /// `SurfaceInfo::SetToDefault`: every slot empty, white tints.
    fn default() -> Self {
        Self {
            base_material: None,
            base_tiling: 0.0,
            detail_material: None,
            detail_tiling: 0.0,
            bump_material: None,
            bump_tiling: 0.0,
            corner_alpha: None,
            side_alpha: None,
            three_corner_alpha: None,
            diagonal_alpha: None,
            vertex_color: [1.0; 4],
            far_vertex_color: [1.0; 4],
        }
    }
}

impl Ac2SurfaceInfo {
    /// `LScapeMaterialInfo::CornerMatch`: two corners draw the same layer.
    pub fn same_layer(&self, other: &Self) -> bool {
        self.base_material == other.base_material
            && self.detail_material == other.detail_material
            && self.bump_material == other.bump_material
    }
}

/// `LScapeSurfaceType::GetVertexInfo` then `MaterialGroup::GetVertexInfo` for the global vertex
/// (`x`, `y`). Returns `None` when no band covers the slope or the band has no base material,
/// which retail answers with `SurfaceInfo::SetToDefault`.
pub(crate) fn vertex_info(
    surface: &SurfaceType,
    pitch_squared: f32,
    x: i32,
    y: i32,
) -> Option<Ac2SurfaceInfo> {
    let group = surface.group_for_pitch(pitch_squared)?;
    let (base_material, base_tiling) = pick(&group.base, x, y, 0x003C_47F7)?;
    let corner_alpha = pick(&group.corner_alpha, x, y, 0x008E_A4FB)?.0;
    let side_alpha = pick(&group.side_alpha, x, y, 0x02F8_8C21)?.0;
    let mut info = Ac2SurfaceInfo {
        base_material: Some(base_material),
        base_tiling,
        corner_alpha: Some(corner_alpha),
        side_alpha: Some(side_alpha),
        three_corner_alpha: pick(&group.three_corner_alpha, x, y, 0x0096_59F0).map(|p| p.0),
        diagonal_alpha: pick(&group.diagonal_alpha, x, y, 0x0082_3981).map(|p| p.0),
        ..Ac2SurfaceInfo::default()
    };
    (info.vertex_color, info.far_vertex_color) = vertex_color(group, x, y);
    if let Some((did, tiling)) = pick(&group.detail, x, y, 0x0090_083D) {
        info.detail_material = Some(did);
        info.detail_tiling = tiling;
    }
    if let Some((did, tiling)) = pick(&group.bump, x, y, 0x02E1_AB71) {
        info.bump_material = Some(did);
        info.bump_tiling = tiling;
    }
    Some(info)
}

/// `MaterialGroup::GetRandom`: a hash of the vertex and a per-list key, scaled to `[0, max)`.
fn random(x: i32, y: i32, kval: i32, max: f32) -> f32 {
    let inner = x
        .wrapping_mul(y)
        .wrapping_mul(y)
        .wrapping_mul(kval)
        .wrapping_mul(kval)
        .wrapping_mul(0x2BD3_8901)
        .wrapping_add(0x6B3C_5F11);
    let seed = kval
        .wrapping_mul(0x6243_3D7D)
        .wrapping_sub(inner.wrapping_mul(x))
        .wrapping_add(y.wrapping_mul(-0x4A65_06DB));
    unit(seed) * max
}

fn unit(seed: i32) -> f32 {
    seed as u32 as f32 * 2.328_306_4e-10
}

/// `MaterialGroup::GetTexture` / `GetAlpha`: walk the cumulative weights to the first entry past
/// the random draw; the last entry takes any remainder.
fn pick(list: &(Vec<WeightedDid>, f32), x: i32, y: i32, kval: i32) -> Option<(DataId, f32)> {
    let (entries, total) = list;
    let last = entries.last()?;
    let draw = random(x, y, kval, *total);
    let mut sum = 0.0;
    for entry in &entries[..entries.len() - 1] {
        sum += entry.frequency;
        if draw < sum {
            return Some((entry.did, entry.tile_width));
        }
    }
    Some((last.did, last.tile_width))
}

/// `MaterialGroup::GetColor`: pick a vertex-colour description by weight, then jitter its hue,
/// saturation and brightness by three further vertex hashes.
fn vertex_color(group: &MaterialGroup, x: i32, y: i32) -> ([f32; 4], [f32; 4]) {
    let Some(last) = group.vertex_colors.last() else {
        return ([1.0; 4], [1.0; 4]);
    };
    let draw = random(x, y, 0x05DD_39FD, group.vcolor_freq_count);
    let mut sum = 0.0;
    let desc = group.vertex_colors[..group.vertex_colors.len() - 1]
        .iter()
        .find(|desc| {
            sum += desc.frequency;
            draw < sum
        })
        .unwrap_or(last);
    let hash = |mul: i32, scale: i32| {
        let xxyy = x.wrapping_mul(x).wrapping_mul(y).wrapping_mul(y);
        unit(
            xxyy.wrapping_mul(mul)
                .wrapping_add(0x6243_3D7D)
                .wrapping_mul(scale)
                .wrapping_add(x.wrapping_mul(-0x6B3C_5F11))
                .wrapping_add(y.wrapping_mul(-0x4A65_06DB)),
        )
    };
    calc_vertex_color(
        desc,
        hash(-0x4507_51A7, 0x0086_F2A7),
        hash(-0x2EF8_0027, 0x0015_2127),
        hash(-0x6F4A_AF73, 0x0000_2473),
    )
}

/// `LScapeVertexColorDesc::CalcVertexColor`: offsets in HSB space, each shaped by its power and
/// signed by which half of `[0, 1)` its draw fell in. The far colour only takes half the
/// brightness offset.
fn calc_vertex_color(
    desc: &VertexColorDesc,
    hue_draw: f32,
    saturate_draw: f32,
    bright_draw: f32,
) -> ([f32; 4], [f32; 4]) {
    let offset = |(min, max, power): (f32, f32, u32), draw: f32| {
        if min == 0.0 && max == 0.0 {
            return 0.0;
        }
        let t = (draw - 0.5).abs() * 2.0;
        let shaped = t.powi(power.max(1) as i32);
        if draw < 0.5 {
            shaped * min
        } else {
            shaped * max
        }
    };
    if untouched(desc) {
        return (desc.vertex_color, desc.far_vertex_color);
    }
    let hue = offset(desc.hue, hue_draw);
    let saturate = offset(desc.saturate, saturate_draw);
    let bright = offset(desc.bright, bright_draw);
    let (h, s, b) = rgb_to_hsb(desc.vertex_color);
    let near = hsb_to_rgb(h + hue, s + saturate, b + bright);
    let (h, s, b) = rgb_to_hsb(desc.far_vertex_color);
    let far = hsb_to_rgb(h, s, bright * 0.5 + b);
    (near, far)
}

fn untouched(desc: &VertexColorDesc) -> bool {
    [desc.hue, desc.saturate, desc.bright]
        .iter()
        .all(|(min, max, _)| *min == 0.0 && *max == 0.0)
}

/// `RGBAColor::RGBtoHSB`: hue in `[0, 6)`, saturation and brightness in `[0, 1]`.
fn rgb_to_hsb([r, g, b, _]: [f32; 4]) -> (f32, f32, f32) {
    if r == b && r == g {
        return (0.0, 0.0, r);
    }
    if g <= r {
        if r < b {
            return ((r - g) / (b - g) + 4.0, 1.0 - g / b, b);
        }
        if b <= g {
            return ((g - b) / (r - b), 1.0 - b / r, r);
        }
        return (6.0 - (b - g) / (r - g), 1.0 - g / r, r);
    }
    if g < b {
        return (4.0 - (g - r) / (b - r), 1.0 - r / b, b);
    }
    if b <= r {
        return (2.0 - (r - b) / (g - b), 1.0 - b / g, g);
    }
    ((b - r) / (g - r) + 2.0, 1.0 - r / g, g)
}

/// `RGBAColor::HSBtoRGB`, clamped to `[0, 1]` (`CapLow`, `CapHigh`); alpha is always 1.
fn hsb_to_rgb(hue: f32, saturate: f32, bright: f32) -> [f32; 4] {
    let rgb = if saturate > 0.0 && bright > 0.0 {
        let low = (1.0 - saturate) * bright;
        let hue = hue.rem_euclid(6.0);
        let ramp = |t: f32| t * (bright - low) + low;
        match hue {
            h if h < 1.0 => [bright, ramp(h), low],
            h if h < 2.0 => [ramp(2.0 - h), bright, low],
            h if h < 3.0 => [low, bright, ramp(h - 2.0)],
            h if h < 4.0 => [low, ramp(4.0 - h), bright],
            h if h < 5.0 => [ramp(h - 4.0), low, bright],
            h => [bright, low, ramp(6.0 - h)],
        }
    } else {
        [bright; 3]
    };
    let [r, g, b] = rgb.map(|c| c.clamp(0.0, 1.0));
    [r, g, b, 1.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsb_round_trips_retail_hue_sectors() {
        for rgb in [
            [0.8, 0.2, 0.1],
            [0.1, 0.7, 0.3],
            [0.2, 0.3, 0.9],
            [0.9, 0.1, 0.6],
        ] {
            let (h, s, b) = rgb_to_hsb([rgb[0], rgb[1], rgb[2], 1.0]);
            assert!((0.0..6.0).contains(&h));
            let back = hsb_to_rgb(h, s, b);
            for (a, e) in back.iter().zip(rgb) {
                assert!((a - e).abs() < 1e-5, "{rgb:?} -> {back:?}");
            }
        }
    }

    #[test]
    fn weighted_pick_walks_cumulative_frequencies() {
        let entry = |did, frequency| WeightedDid {
            did: DataId::new(did),
            frequency,
            tile_width: 20.0,
        };
        let list = (vec![entry(1, 1.0), entry(2, 1.0)], 2.0);
        let mut seen = [0; 2];
        for x in 0..64 {
            for y in 0..64 {
                let (did, _) = pick(&list, x, y, 0x003C_47F7).unwrap();
                seen[did.raw() as usize - 1] += 1;
            }
        }
        assert!(seen.iter().all(|n| *n > 1000), "{seen:?}");
    }
}
