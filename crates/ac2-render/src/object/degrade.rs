//! Object level of detail: `CPartArray::Draw`'s distance cull, fade and degrade factor from the
//! Setup's `DegradeInfo`, and `CBaseMesh::DrawMesh`'s choice of a mesh degrade from that factor
//! (`CMeshDegradeList::m_DLIndices`, `CMeshDegradeList::UpdateDetailLevelIndices`).
//!
//! Retail ran these on the x87 unit at 53-bit precision; the expressions are evaluated in `f64`
//! and rounded to `f32` where `CPartArray::Draw` stores a `float` (the scaled distances, the
//! visible distance, the fade length and start, and the factor).

use std::ops::Range;

use ac2_anim::{Ac2DegradeDistances, Ac2DegradeInfo};

/// `ModelDetail` and `MaterialDetail` settings: low, medium, high.
pub const DETAIL_LEVELS: usize = 3;
/// `ModelDetail` high, the setting drawn where degrades are off (the sky, UI scenes).
pub const HIGH_DETAIL: u32 = 2;

/// The render preferences `CPartArray::Draw` and `CBaseMesh::DrawMesh` read
/// (`Render::m_Prefs`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DegradeOptions {
    /// `ModelDetail` (`0x4000004E`): 0 low, 1 medium, 2 high.
    pub model_detail: u32,
    /// `MaterialDetail` (`0x4000004D`).
    pub material_detail: u32,
    /// `SceneryDrawDistance` (`0x40000076`): 0 scales scenery distances by 0.4, 1 by 0.7.
    pub scenery_draw_distance: u32,
    /// `AdaptiveDegrade` (`0x40000064`).
    pub adaptive_degrade: bool,
    /// `AdaptiveDegradeBias` (`0x40000065`), -1 to 1.
    pub adaptive_degrade_bias: f32,
}

impl Default for DegradeOptions {
    /// The registered defaults (`Render::InitRenderingPreferences`): medium model, material and
    /// scenery distance, adaptive degrade off.
    fn default() -> Self {
        Self {
            model_detail: 1,
            material_detail: 1,
            scenery_draw_distance: 1,
            adaptive_degrade: false,
            adaptive_degrade_bias: 0.0,
        }
    }
}

/// How one object draws this frame: the mesh degrade factor, the detail settings, and whether
/// its alpha test is forced off.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawDetail {
    /// `CBaseMesh::DrawMesh`'s factor (after the player's 1.5 scale): 1 the first degrade of the
    /// detail list, 0 the last.
    pub degrade: f32,
    /// `ModelDetail`: which `m_DLIndices` list the degrade is picked from.
    pub model_detail: u32,
    /// `SceneTool::m_bForceDisableAlphaTest`, set by `CPartArray::Draw` past 40 units:
    /// `RenderDeviceD3D::SetupState` drops the alpha test and depth write of a faded layer.
    pub force_disable_alpha_test: bool,
}

impl DrawDetail {
    /// Degrades off: the first degrade of the high detail list.
    pub const FULL: Self = Self {
        degrade: 1.0,
        model_detail: HIGH_DETAIL,
        force_disable_alpha_test: false,
    };
}

impl Default for DrawDetail {
    fn default() -> Self {
        Self::FULL
    }
}

/// `CPartArray::Draw`'s result for a drawn object.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectLod {
    /// The degrade factor handed to every part (`CPhysicsPart::DrawPart`), 0 to 1.
    pub factor: f32,
    /// The fade: multiplied into the object colour's alpha.
    pub alpha: f32,
    /// The material detail setting the parts draw with.
    pub material_detail: u32,
    /// The scaled visible distance (`_pfMinVisibleDistance`'s candidate), at least 20.
    pub visible_distance: f32,
    /// Farther than 40 units (`SceneTool::m_bForceDisableAlphaTest`).
    pub force_disable_alpha_test: bool,
}

impl ObjectLod {
    /// The detail each part's `CBaseMesh::DrawMesh` draws with; `drawing_player` is
    /// `HACK_bDrawingPlayer`.
    pub fn draw_detail(&self, options: &DegradeOptions, drawing_player: bool) -> DrawDetail {
        DrawDetail {
            degrade: mesh_degrade_factor(self.factor, drawing_player),
            model_detail: options.model_detail.min(HIGH_DETAIL),
            force_disable_alpha_test: self.force_disable_alpha_test,
        }
    }
}

/// `CPartArray::Draw`'s scale of the Setup's distances. A non-adaptive object (client-made
/// scenery, with a temporary id) takes `SceneryDrawDistance`'s 0.4 or 0.7; an adaptive one
/// (`CPhysicsObj::Draw`: not temporary, `is_mobile`, `is_dynamic`) follows the frame rate when
/// `AdaptiveDegrade` is on (`SceneTool::framesPerSecond`).
pub fn degrade_scale(options: &DegradeOptions, adaptive: bool, frames_per_second: f32) -> f64 {
    if !adaptive {
        return match options.scenery_draw_distance {
            0 => f64::from(0.4_f32),
            1 => f64::from(0.7_f32),
            _ => 1.0,
        };
    }
    if !options.adaptive_degrade {
        return 1.0;
    }
    let bias = f64::from(options.adaptive_degrade_bias.clamp(-1.0, 1.0));
    // The machine code forms 1.05 as `1 - 0.95f + 1` and 0.1 as `1 - 0.9f`.
    let boost = 1.0 - f64::from(0.95_f32) + 1.0;
    let rate = (f64::from(frames_per_second) / (bias * 40.0 + 41.0) * boost).min(1.0);
    let weight = f64::from(0.9_f32);
    rate * weight + (1.0 - weight)
}

/// `CPartArray::Draw` for an object `distance` from the eye (`CPhysicsObj::m_fViewDistance`),
/// with degrades on: `None` when it is past its visible distance or faded out, else its degrade
/// factor, fade, material detail and visible distance.
pub fn part_array_lod(
    distance: f32,
    info: &Ac2DegradeDistances,
    scale: f64,
    material_detail: u32,
) -> Option<ObjectLod> {
    let distance_f = f64::from(distance);
    let scaled = |value: f32| scale * f64::from(value);
    let stored = |value: f64| f64::from(value as f32);
    let visible = stored(scaled(info.visible)).max(20.0);
    if distance_f > visible {
        return None;
    }
    let near = stored(scaled(info.near));
    let ideal = stored(scaled(info.ideal));
    let far = stored(scaled(info.far));
    let high = stored(scaled(info.high_detail) * 3.0);
    let medium = scaled(info.medium_detail) * 3.0;
    let guard = |span: f64| {
        if span == 0.0 {
            f64::from(0.0002_f32)
        } else {
            span
        }
    };
    let factor = if ideal <= distance_f {
        (1.0 - (distance_f - ideal) / guard(far - ideal)) * 0.5
    } else {
        (1.0 - (distance_f - near) / guard(ideal - near)) * 0.5 + 0.5
    };
    let factor = (factor as f32).clamp(0.0, 1.0);
    let mut material = material_detail;
    if material != 0 {
        if distance_f <= medium {
            if high < distance_f {
                material = 1;
            }
        } else {
            material = 0;
        }
    }
    let fade = stored(f64::from(0.075_f32) * visible);
    let start = stored(visible - fade);
    let alpha = if start <= distance_f {
        1.0 - (distance_f - start) / fade
    } else {
        1.0
    };
    // `CPartArray::Draw` draws nothing unless the faded alpha is above zero.
    if alpha.is_nan() || alpha > 0.0 {
        Some(ObjectLod {
            factor,
            alpha: (alpha as f32).min(1.0),
            material_detail: material,
            visible_distance: visible as f32,
            force_disable_alpha_test: distance > 40.0,
        })
    } else {
        None
    }
}

/// `CPartArray::Draw` with `SceneTool::m_bDisableDegrades` set (the map view): no visible-distance
/// cull, no fade, no distance degrade and no distance material detail, so the factor is 1, the
/// alpha 1 and the material detail the `MaterialDetail` setting. The Setup's level is still read
/// at `ModelDetail`, and `SceneTool::m_bForceDisableAlphaTest` is still set past 40 units: the
/// switch only clears `bVar7`, which the 40-unit test does not read.
pub fn undegraded_lod(distance: f32, options: &DegradeOptions) -> ObjectLod {
    ObjectLod {
        factor: 1.0,
        alpha: 1.0,
        material_detail: options.material_detail,
        visible_distance: f32::MAX,
        force_disable_alpha_test: distance > 40.0,
    }
}

/// [`part_array_lod`] for an object whose Setup is `info`, at `options`' model detail.
pub fn object_lod(
    distance: f32,
    info: &Ac2DegradeInfo,
    options: &DegradeOptions,
    adaptive: bool,
    frames_per_second: f32,
) -> Option<ObjectLod> {
    let scale = degrade_scale(options, adaptive, frames_per_second);
    part_array_lod(
        distance,
        info.level(options.model_detail),
        scale,
        options.material_detail,
    )
}

/// `CBaseMesh::DrawMesh`'s factor: the part's factor, times 1.5 while drawing the player
/// (`HACK_bDrawingPlayer`), clamped to 0..1 and stored as a `float`.
pub fn mesh_degrade_factor(factor: f32, drawing_player: bool) -> f32 {
    let factor = if drawing_player {
        (f64::from(factor) * 1.5) as f32
    } else {
        factor
    };
    factor.clamp(0.0, 1.0)
}

/// `CBaseMesh::DrawMesh`'s level among `count` admitted degrades for factor `degrade`:
/// `_ftol2((count - 1) * (1 - degrade))`, truncated.
pub fn degrade_level(degrade: f32, count: usize) -> usize {
    let span = count.saturating_sub(1) as f64;
    let level = (span * (1.0 - f64::from(degrade))) as usize;
    level.min(count.saturating_sub(1))
}

/// `CMeshDegradeList::m_DLIndices`: per detail setting, the degrades whose `m_DetailLevels` admit
/// it (`UpdateDetailLevelIndices`: bit 1 low, 2 medium, 4 high), in degrade order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DegradeLists {
    lists: [Vec<u16>; DETAIL_LEVELS],
    /// Every degrade of the mesh.
    count: usize,
}

impl DegradeLists {
    /// The lists of a mesh whose degrades have `detail_levels`, in order.
    pub fn new(detail_levels: &[u32]) -> Self {
        let mut lists: [Vec<u16>; DETAIL_LEVELS] = Default::default();
        for (index, levels) in detail_levels.iter().enumerate() {
            for (bit, list) in lists.iter_mut().enumerate() {
                if levels & (1 << bit) != 0 {
                    list.push(index as u16);
                }
            }
        }
        Self {
            lists,
            count: detail_levels.len(),
        }
    }

    /// The degrades admitted at `detail`.
    pub fn list(&self, detail: u32) -> &[u16] {
        self.lists
            .get(detail as usize)
            .map_or(&[][..], |list| &list[..])
    }

    /// `CBaseMesh::DrawMesh`'s degrade for factor `degrade` at `model_detail`: the level among
    /// that setting's list, or among every degrade when the list is empty. `None` for a mesh
    /// without degrades.
    pub fn select(&self, model_detail: u32, degrade: f32) -> Option<usize> {
        if self.count == 0 {
            return None;
        }
        let list = self.list(model_detail);
        if list.is_empty() {
            return Some(degrade_level(degrade, self.count));
        }
        Some(usize::from(list[degrade_level(degrade, list.len())]))
    }
}

/// A part's degrades: its `m_DLIndices` lists and, per degrade, the run of the part's batches
/// (`ObjectPart::batches`, degrade by degrade) that draw it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartDegrades {
    pub lists: DegradeLists,
    pub batches: Vec<Range<u32>>,
}

impl PartDegrades {
    /// The batches `CBaseMesh::DrawMesh` draws at `detail` (`DrawMeshDegrade` of the chosen
    /// degrade). `total` is the part's batch count; a part without degrade data draws them all.
    pub fn drawn(degrades: Option<&Self>, detail: &DrawDetail, total: usize) -> Range<usize> {
        let Some(degrades) = degrades else {
            return 0..total;
        };
        degrades
            .lists
            .select(detail.model_detail, detail.degrade)
            .and_then(|index| degrades.batches.get(index))
            .map_or(0..0, |range| range.start as usize..range.end as usize)
    }
}

#[cfg(test)]
#[path = "degrade_tests.rs"]
mod tests;
