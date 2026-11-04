//! When the interior cells' vertices are lit again by the sun (`CEnvCell::DynamicLightVisibleCells`
//! over `CEnvCell::sm_visibleCellTable`, every resident env cell):
//!
//! - `WorldRender::Update` (Legions 0x0054DA80) runs it every 0.1 s (`m_nextEnvLightTick`) as
//!   `(0, 0, -1, -1, 0)`: a cell not yet lit (`m_bDynamicLightingComputed` clear) takes its static
//!   diffuse plus the lights whose range animates, which is the colour it was loaded with here.
//! - `WorldRender::Render` (Legions 0x0054F230), while the viewer is in an env cell
//!   (`m_bCanSeeInside`, set by `GatherIndoorObjects`), runs it as `(force, 1, L0, L1, alpha)`:
//!   a cell not yet lit, or every cell when `force`, takes the static diffuse plus the sun
//!   ([`CellSun::bake`]). `force` is `LScapeRender::UpdateLighting`'s answer, which hands over and
//!   clears `m_bRefreshLighting` on every render, inside or out.
//! - `m_bRefreshLighting` is set when the landscape is made, when the landblocks shift
//!   (`LScapeRender::FlagRefreshDrawLists`) and at each light tick of `WorldRender::UpdateTimeOfDay`
//!   (`LScapeRender::DirtyLighting`), which is checked on its regular tick:
//!   `CSkyDesc::GetLightTickSize(0.5)` and `GetTickSize(0.5)` over `GameTime::GetTimeDilation`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::object::{CellSun, SunPass};

use super::{BlockInfo, BlockKey};

/// `WorldRender::Update`'s `m_nextEnvLightTick` step.
const ENV_LIGHT_TICK: f64 = 0.1;

/// One cell of a resident block that a sun pass can change.
#[derive(Debug, Clone)]
pub(super) struct SunCell {
    /// Its place among the block's interior cells.
    pub(super) index: usize,
    pub(super) sun: Arc<CellSun>,
    /// `m_bDynamicLightingComputed`.
    pub(super) computed: bool,
}

/// New vertex colours for one interior cell of a block (its index among the block's cells).
#[derive(Debug, Clone, PartialEq)]
pub struct CellColors {
    pub block: BlockKey,
    pub cell: usize,
    pub colors: Vec<[f32; 4]>,
}

/// The timers and flag that decide the passes.
#[derive(Debug, Clone)]
pub(super) struct InteriorSun {
    /// `LScapeRender::m_bRefreshLighting`.
    refresh: bool,
    next_env_tick: f64,
    next_tick: f64,
    next_light_tick: f64,
}

impl InteriorSun {
    /// `LScapeRender::LScapeRender` sets the refresh flag; the ticks start due.
    pub(super) fn new() -> Self {
        Self {
            refresh: true,
            next_env_tick: 0.0,
            next_tick: 0.0,
            next_light_tick: 0.0,
        }
    }

    /// `LScapeRender::FlagRefreshDrawLists`: the landblocks shifted.
    pub(super) fn blocks_changed(&mut self) {
        self.refresh = true;
    }

    /// `WorldRender::Update` at client time `now`: the time of day's ticks, with the sky's tick
    /// and light tick sizes (`None` without a sky description, when no light tick is taken),
    /// then the env light tick.
    pub(super) fn update(
        &mut self,
        now: f64,
        ticks: Option<(f64, f64)>,
        blocks: &mut HashMap<BlockKey, BlockInfo>,
    ) {
        // `GameTime::GetTimeDilation`: the scene has no dilation of its own, 1.
        let dilation = 1.0_f32;
        if let Some((tick, light_tick)) = ticks {
            if self.next_tick <= now {
                if self.next_light_tick <= now {
                    self.refresh = true;
                    self.next_light_tick = f64::from((light_tick as f32) / dilation + now as f32);
                }
                self.next_tick = f64::from((tick as f32) / dilation + now as f32);
            }
        }
        if self.next_env_tick <= now {
            for cell in blocks.values_mut().flat_map(|b| b.sun_cells.iter_mut()) {
                cell.computed = true;
            }
            self.next_env_tick = now + ENV_LIGHT_TICK;
        }
    }

    /// `WorldRender::Render`: the refresh flag is taken; while the viewer is `inside`, the cells
    /// not yet lit, or all of them when forced, are lit by `pass`.
    pub(super) fn render(
        &mut self,
        inside: bool,
        pass: &SunPass,
        blocks: &mut HashMap<BlockKey, BlockInfo>,
    ) -> Vec<CellColors> {
        let force = std::mem::take(&mut self.refresh);
        let mut out = Vec::new();
        if !inside {
            return out;
        }
        let mut keys: Vec<BlockKey> = blocks.keys().copied().collect();
        keys.sort_unstable();
        for key in keys {
            let Some(block) = blocks.get_mut(&key) else {
                continue;
            };
            for cell in &mut block.sun_cells {
                if cell.computed && !force {
                    continue;
                }
                cell.computed = true;
                if let Some(colors) = cell.sun.bake_vertex_colors(pass) {
                    out.push(CellColors {
                        block: key,
                        cell: cell.index,
                        colors,
                    });
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_refresh_flag_is_set_at_start_and_by_a_light_tick_and_taken_by_a_render() {
        let mut sun = InteriorSun::new();
        let mut blocks = HashMap::new();
        let pass = SunPass {
            l0: Some(0),
            l1: Some(1),
            alpha: 0.5,
            direction: ac2_math::Vec3::new(0.0, 0.0, -1.0),
            color: [1.0; 3],
            intensity: 1.0,
        };
        sun.update(0.0, Some((3.0, 20.0)), &mut blocks);
        assert!(sun.refresh);
        sun.render(false, &pass, &mut blocks);
        assert!(!sun.refresh, "an outside render takes the flag too");
        // The regular tick at 3 s is before the light tick at 20 s.
        sun.update(3.0, Some((3.0, 20.0)), &mut blocks);
        assert!(!sun.refresh);
        sun.update(21.0, Some((3.0, 20.0)), &mut blocks);
        assert!(sun.refresh);
        sun.blocks_changed();
        sun.render(true, &pass, &mut blocks);
        assert!(!sun.refresh);
    }
}
