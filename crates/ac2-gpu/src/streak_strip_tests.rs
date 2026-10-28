//! A streak's ribbon uploaded as the particle geometry of an object model: the strip the
//! streak manager builds (`StreakEffectManager::RenderStreakEffects`) fills the draw range, and a
//! ribbon with no segments draws nothing. Runs when `AC2_DAT_DIR` is set (for a real model's
//! part) and a GPU adapter exists; it takes one headless device.

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::{Frame, Vec3};
use ac2_render::object::{load_object_model, RenderSettings};
use ac2_render::streak::{StreakEffects, StreakSpec};
use ac2_render::Image;

use super::ObjectRenderer;
use crate::{Gpu, TextureKey, HDR_FORMAT};

const HUMAN_MALE_VISUAL: u32 = 0x1F00_0023;
const MATERIAL: u32 = 0x4000_0032;

fn spec() -> StreakSpec {
    StreakSpec {
        start: Vec3::new(0.0, 0.0, 0.5),
        end: Vec3::new(0.0, 0.0, -0.5),
        lifespan: 0.0,
        trail_duration: 1.0,
        min_segment_time: 0.0,
        min_segment_distance: 0.05,
        color: [1.0, 1.0, 1.0, 1.0],
        texture_scale: 1.0,
        material: DataId::new(MATERIAL),
    }
}

#[test]
fn a_streak_strip_fills_the_particle_draw_range() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let store = ResourceStore::new(dataset);
    let Some(model) = load_object_model(
        &store,
        DataId::new(HUMAN_MALE_VISUAL),
        &RenderSettings::default(),
    )
    .unwrap() else {
        return;
    };
    let mut renderer = ObjectRenderer::new(&gpu, HDR_FORMAT);
    let mut no_images = |_key: TextureKey| -> Option<Image> { None };
    let mut uploaded = renderer.upload(&gpu, &model, &mut no_images);

    // A ribbon of three placements, the emitter moving along x: its strip has segments.
    let mut streaks = StreakEffects::default();
    streaks.create(spec(), 0.0);
    let mut time = 0.0;
    let mut stamp = 0;
    for step in 0..4 {
        time += 0.05;
        stamp += 1;
        let frame = Frame {
            origin: Vec3::new(step as f32, 0.0, 0.0),
            ..Frame::IDENTITY
        };
        if let Some(effect) = streaks.get_mut(1) {
            effect.activate();
            effect.set_position(stamp, 1, frame, 1.0);
        }
        streaks.think(time);
    }
    let geometry = streaks.geometry(stamp);
    let (_, ribbon) = geometry.first().expect("the ribbon draws one material");
    assert!(!ribbon.indices.is_empty(), "the strip has triangles");

    renderer.set_geometry(&gpu, &mut uploaded, &ribbon.vertices, &ribbon.indices);
    assert_eq!(
        uploaded.parts[0].batches[0].range,
        0..ribbon.indices.len() as u32,
        "the strip's triangles are the draw range"
    );

    // A ribbon whose streak was not placed this stamp draws nothing.
    let none = streaks.geometry(stamp + 1);
    assert!(none.is_empty());
    renderer.set_geometry(&gpu, &mut uploaded, &[], &[]);
    assert_eq!(uploaded.parts[0].batches[0].range, 0..0);
}
