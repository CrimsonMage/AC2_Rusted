//! The modern sky's roles keep `LScapeSky::Draw`'s composition: the sky objects draw in list
//! order, each with its authored blend over the ones before it (`CPhysicsObj::EasyDraw` ->
//! `CBaseMesh::DrawMeshDegrade` -> `RenderDeviceD3D::SetupState`). Only a viewer-centred dome's
//! alpha layer becomes the atmosphere's tint (`SkyRole::Veil`); the moon, a flat mesh 738 to
//! 972 m out, keeps `SRCALPHA`, `INVSRCALPHA` and so covers the sky objects drawn before it.
//! The data check runs when `AC2_DAT_DIR` is set.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_render::object::{load_object_model, RenderSettings};

use super::upload::model_bounds;
use crate::object_draw::{sky_dome, SkyRole};

/// The night layer (`SkyObject` 1 of the Legions day) and the moon (`SkyObject` 4).
const NIGHT_LAYER: u32 = 0x0400_00B4;
const MOON: u32 = 0x0400_00B6;

#[test]
fn only_viewer_centred_wide_objects_are_domes() {
    // A hemisphere of radius 360 about the viewer: its vertices average half way up.
    assert!(sky_dome(360.0, Vec3::new(0.0, 0.0, 180.0)));
    // A quad 972 m out (the moon): wide from the origin, but its vertices sit together.
    assert!(!sky_dome(972.0, Vec3::new(0.0, -970.0, 0.0)));
    // Small objects are never domes.
    assert!(!sky_dome(68.0, Vec3::ZERO));
}

#[test]
fn the_moon_keeps_its_alpha_blend_and_the_night_layer_tints() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = Arc::new(ResourceStore::new(Arc::clone(&dataset)));
    let world = ac2_world::Ac2WorldSource::open(Arc::clone(&store))
        .expect("AC2 vocabulary decodes")
        .expect("AC2 dataset recognised");
    let region = world.region().expect("AC2 region decodes");
    let settings = RenderSettings::default();
    let alpha_role = |did: u32| {
        let model = load_object_model(&store, DataId::new(did), &settings)
            .expect("sky model decodes")
            .expect("sky model draws");
        let (center, radius) = model_bounds(&model);
        let pass = model.parts[0].batches[0].material.passes([1.0; 4], 0.0)[0].clone();
        assert_eq!((pass.source_blend, pass.dest_blend), (5, 6), "{did:08X}");
        SkyRole::of(&pass, false, radius, center)
    };
    assert_eq!(alpha_role(NIGHT_LAYER), SkyRole::Veil);
    assert_eq!(alpha_role(MOON), SkyRole::Layer);
    // `CDayDesc::GetSky` lists the night layer before the moon, so the moon draws over it (and
    // over every sky object between them) wherever it is up.
    let mut moon_seen = false;
    for step in 0..48 {
        let objects = region
            .sky_objects(f64::from(step) * 7347.0 / 48.0)
            .expect("the day has sky objects");
        let index = |did: u32| {
            objects
                .iter()
                .position(|o| o.gfx.map(DataId::raw) == Some(did))
        };
        if let (Some(night), Some(moon)) = (index(NIGHT_LAYER), index(MOON)) {
            moon_seen = true;
            assert!(
                night < moon,
                "step {step}: night layer {night}, moon {moon}"
            );
        }
    }
    assert!(moon_seen, "the moon rises during the day");
}
