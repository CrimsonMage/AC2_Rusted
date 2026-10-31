//! The scene's object level of detail: each drawn object's `CPartArray::Draw` cull, fade and
//! degrade, from its root Setup's `DegradeInfo` and the eye distance
//! (`CPhysicsObj::UpdateViewerDistance`).

use std::collections::HashMap;
use std::time::Instant;

use ac2_anim::Ac2DegradeInfo;
use ac2_core::DataId;
use ac2_math::Vec3;

use super::WorldScene;
use crate::object::{
    object_lod, undegraded_lod, DegradeOptions, DrawDetail, ObjectLod, ObjectModel,
};

/// The degrade data of the scene's loaded models and the settings and frame rate it applies.
#[derive(Debug)]
pub(super) struct SceneDegrades {
    pub options: DegradeOptions,
    /// Placed visuals' root Setup `DegradeInfo`.
    visuals: HashMap<DataId, Ac2DegradeInfo>,
    /// World objects' (`WorldScene::request_object` keys).
    objects: HashMap<u64, Ac2DegradeInfo>,
    /// `SceneTool::framesPerSecond`, from the time between laid-out frames.
    frames_per_second: f32,
    last_frame: Option<Instant>,
    /// `SceneTool::m_bDisableDegrades`: the map view draws every object at full detail.
    disabled: bool,
}

impl Default for SceneDegrades {
    fn default() -> Self {
        Self {
            options: DegradeOptions::default(),
            visuals: HashMap::new(),
            objects: HashMap::new(),
            frames_per_second: 60.0,
            last_frame: None,
            disabled: false,
        }
    }
}

/// How a placed object draws this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedDetail {
    pub detail: DrawDetail,
    /// The fade (`CPartArray::Draw`'s colour alpha factor).
    pub alpha: f32,
}

impl PlacedDetail {
    /// Degrades off: full detail, unfaded.
    pub const FULL: Self = Self {
        detail: DrawDetail::FULL,
        alpha: 1.0,
    };
}

impl SceneDegrades {
    pub fn note_visual(&mut self, visual: DataId, model: &ObjectModel) {
        if let Some(info) = model.degrade {
            self.visuals.insert(visual, info);
        }
    }

    pub fn note_object(&mut self, key: u64, model: &ObjectModel) {
        if let Some(info) = model.degrade {
            self.objects.insert(key, info);
        }
    }

    /// Times a laid-out frame for the frame rate.
    pub fn tick(&mut self) {
        let now = Instant::now();
        if let Some(last) = self.last_frame.replace(now) {
            let seconds = now.duration_since(last).as_secs_f64();
            if seconds > 0.0 {
                self.frames_per_second = (1.0 / seconds) as f32;
            }
        }
    }

    fn lod(&self, info: &Ac2DegradeInfo, distance: f32, adaptive: bool) -> Option<ObjectLod> {
        if self.disabled {
            return Some(undegraded_lod(distance, &self.options));
        }
        object_lod(
            distance,
            info,
            &self.options,
            adaptive,
            self.frames_per_second,
        )
    }

    /// The detail of an object drawn while `SceneTool::m_bRenderingToShadowMap` is set:
    /// `CPartArray::Draw` clears `bVar7` there, so no visible-distance cull or fade, factor 1 and
    /// the material detail as set; the Setup's level is still read at `ModelDetail`.
    pub fn shadow_map_detail(&self) -> PlacedDetail {
        let lod = undegraded_lod(0.0, &self.options);
        PlacedDetail {
            detail: lod.draw_detail(&self.options, false),
            alpha: lod.alpha,
        }
    }

    /// The scenery the shadow casters draw (`LScapeRender::RefreshShadowMap`, and the sun cascades
    /// that stand in for it near the eye): every placement at [`Self::shadow_map_detail`], however
    /// far from the eye (`CLandBlock::GatherObjects` skips the in-view test while rendering to the
    /// shadow map, and `CPartArray::Draw` its visible-distance cull).
    pub fn shadow_placements<'a>(
        &self,
        placements: impl Iterator<Item = &'a super::Placement>,
    ) -> Vec<super::FramePlacement> {
        let shadow = self.shadow_map_detail();
        placements
            .map(|p| super::FramePlacement {
                visual: p.visual,
                world: p.world,
                frill_normal: None,
                frill_shade: 1.0,
                detail: shadow.detail,
                alpha: shadow.alpha,
                grass: [0.0; 3],
            })
            .collect()
    }

    /// A placed (client-made, temporary-id) object of `visual` at `origin` seen from `eye`:
    /// `None` when `CPartArray::Draw` culls it. A visual without a Setup draws in full.
    pub fn placed(&self, visual: DataId, origin: Vec3, eye: Vec3) -> Option<PlacedDetail> {
        let Some(info) = self.visuals.get(&visual) else {
            return Some(PlacedDetail::FULL);
        };
        let lod = self.lod(info, origin.distance(eye), false)?;
        Some(PlacedDetail {
            detail: lod.draw_detail(&self.options, false),
            alpha: lod.alpha,
        })
    }

    /// A world object `key` `distance` from the eye.
    pub fn object(
        &self,
        key: u64,
        distance: f32,
        adaptive: bool,
        player: bool,
    ) -> Option<PlacedDetail> {
        let Some(info) = self.objects.get(&key) else {
            return Some(PlacedDetail::FULL);
        };
        let lod = self.lod(info, distance, adaptive)?;
        Some(PlacedDetail {
            detail: lod.draw_detail(&self.options, player),
            alpha: lod.alpha,
        })
    }
}

impl WorldScene {
    /// `SceneTool::m_bDisableDegrades` for the coming frame (the camera's
    /// `CameraManager::disable_degrades`, on in the map view): `CPartArray::Draw` then skips its
    /// visible-distance cull, fade, degrade and distance material detail.
    pub fn set_disable_degrades(&mut self, disabled: bool) {
        self.degrades.disabled = disabled;
    }
}

#[cfg(test)]
mod tests {
    use ac2_anim::Ac2DegradeDistances;

    use super::*;

    fn degrades() -> SceneDegrades {
        let level = Ac2DegradeDistances {
            visible: 100.0,
            near: 10.0,
            ideal: 30.0,
            far: 60.0,
            high_detail: 5.0,
            medium_detail: 10.0,
        };
        let info = Ac2DegradeInfo { levels: [level; 3] };
        let mut degrades = SceneDegrades::default();
        degrades.visuals.insert(DataId::new(7), info);
        degrades.objects.insert(9, info);
        degrades
    }

    #[test]
    fn degrades_on_cull_and_fade_a_far_object() {
        let degrades = degrades();
        let eye = Vec3::ZERO;
        assert!(degrades
            .placed(DataId::new(7), Vec3::new(500.0, 0.0, 0.0), eye)
            .is_none());
        assert!(degrades.object(9, 500.0, false, false).is_none());
        let near = degrades.object(9, 20.0, false, false).unwrap();
        assert_eq!(near.alpha, 1.0);
    }

    #[test]
    fn shadow_map_draws_a_far_object_undegraded_whatever_the_switch() {
        let mut degrades = degrades();
        degrades.options.model_detail = 1;
        for disabled in [false, true] {
            degrades.disabled = disabled;
            // The normal pass culls it; the shadow map's `bVar7` is clear either way.
            assert_eq!(
                degrades
                    .placed(DataId::new(7), Vec3::new(500.0, 0.0, 0.0), Vec3::ZERO)
                    .is_none(),
                !disabled
            );
            let shadow = degrades.shadow_map_detail();
            assert_eq!((shadow.alpha, shadow.detail.degrade), (1.0, 1.0));
            assert_eq!(shadow.detail.model_detail, 1);
            assert!(!shadow.detail.force_disable_alpha_test);
        }
    }

    #[test]
    fn shadow_placements_keep_the_culled_scenery() {
        let degrades = degrades();
        let far = super::super::Placement {
            visual: DataId::new(7),
            world: ac2_math::Mat4::from_translation(Vec3::new(500.0, 0.0, 0.0)),
            frill: false,
        };
        let kept = degrades.shadow_placements([&far].into_iter());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].detail, degrades.shadow_map_detail().detail);
        assert_eq!(kept[0].alpha, 1.0);
    }

    #[test]
    fn disabled_degrades_draw_a_far_object_at_full_detail() {
        let mut degrades = degrades();
        degrades.disabled = true;
        let eye = Vec3::ZERO;
        let placed = degrades
            .placed(DataId::new(7), Vec3::new(500.0, 0.0, 0.0), eye)
            .unwrap();
        assert_eq!(placed.alpha, 1.0);
        assert_eq!(placed.detail.degrade, 1.0);
        // `m_bForceDisableAlphaTest` is set past 40 units whatever the switch.
        assert!(placed.detail.force_disable_alpha_test);
        let object = degrades.object(9, 500.0, true, true).unwrap();
        assert_eq!((object.alpha, object.detail.degrade), (1.0, 1.0));
        degrades.disabled = false;
        assert!(degrades.object(9, 500.0, true, true).is_none());
    }
}
