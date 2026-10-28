//! The scene's object level of detail: each drawn object's `CPartArray::Draw` cull, fade and
//! degrade, from its root Setup's `DegradeInfo` and the eye distance
//! (`CPhysicsObj::UpdateViewerDistance`).

use std::collections::HashMap;
use std::time::Instant;

use ac2_anim::Ac2DegradeInfo;
use ac2_core::DataId;
use ac2_math::Vec3;

use crate::object::{object_lod, DegradeOptions, DrawDetail, ObjectLod, ObjectModel};

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
}

impl Default for SceneDegrades {
    fn default() -> Self {
        Self {
            options: DegradeOptions::default(),
            visuals: HashMap::new(),
            objects: HashMap::new(),
            frames_per_second: 60.0,
            last_frame: None,
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
        object_lod(
            distance,
            info,
            &self.options,
            adaptive,
            self.frames_per_second,
        )
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
