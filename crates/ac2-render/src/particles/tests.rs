//! Real-data checks (`AC2_DAT_DIR`): every particle description runs without panicking and
//! produces finite geometry; invariants of the data, no recorded values.

use std::sync::Arc;

use ac2_assets::Ac2ParticleSystemDesc;
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};
use ac2_math::{Frame, Vec3};

use super::{CameraBasis, NearbyRepulsor, ParticleGeometry, ParticleSystem};

#[test]
fn every_description_runs_and_fills_finite_quads() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x3900_0000, 0x39FF_FFFF)
        .collect();
    assert!(!ids.is_empty());
    let basis = CameraBasis::look_at(Vec3::new(-6.0, -6.0, 3.0), Vec3::new(0.0, 0.0, 1.0))
        .expect("a basis");
    let mut ran = 0;
    let mut drew = 0;
    for did in ids {
        let Ok(Some(desc)) = store.load_cached_packed::<Ac2ParticleSystemDesc>(
            TypedResourceRef::new(DatFileKind::Portal, DataId::new(did), 0),
        ) else {
            continue;
        };
        let mut system = ParticleSystem::new(
            Arc::new((*desc).clone()),
            Frame::IDENTITY,
            Vec3::ONE,
            1.0,
            did,
            100.0,
        );
        if system.needs_repulsors() {
            continue;
        }
        let mut geometry = ParticleGeometry::default();
        let mut now = 100.0;
        for _ in 0..150 {
            now += 1.0 / 30.0;
            system.think(now);
            geometry = ParticleGeometry::default();
            system.fill(&basis, now, 1.0, &mut geometry);
        }
        ran += 1;
        for vertex in &geometry.vertices {
            assert!(
                vertex.position.iter().all(|v| v.is_finite())
                    && vertex.color.iter().all(|v| (0.0..=1.0).contains(v)),
                "{did:08X}: {vertex:?}"
            );
        }
        assert!(geometry
            .indices
            .iter()
            .all(|i| (*i as usize) < geometry.vertices.len()));
        drew += usize::from(!geometry.is_empty());
    }
    assert!(ran > 50, "{ran} descriptions ran");
    assert!(
        drew * 3 > ran,
        "{drew} of {ran} descriptions drew particles"
    );
}

#[test]
fn a_camera_basis_is_orthonormal_and_right_handed() {
    let basis = CameraBasis::look_at(Vec3::ZERO, Vec3::new(0.0, 5.0, 0.0)).expect("a basis");
    assert!((basis.right - Vec3::X).length() < 1e-6, "{basis:?}");
    assert!((basis.up - Vec3::Z).length() < 1e-6, "{basis:?}");
    assert!(CameraBasis::look_at(Vec3::ZERO, Vec3::ZERO).is_none());
}

/// Every description with a repulse emitter runs under a repulsor and an attractor passing through
/// it, with finite particles; invariants of the data, no recorded values.
#[test]
fn every_repulse_description_runs_under_a_repulsor() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let ids: Vec<u32> = dataset
        .ids_in_range(DatFileKind::Portal, 0x3900_0000, 0x39FF_FFFF)
        .collect();
    let basis = CameraBasis::look_at(Vec3::new(-6.0, -6.0, 3.0), Vec3::new(0.0, 0.0, 1.0))
        .expect("a basis");
    let mut ran = 0;
    for did in ids {
        let Ok(Some(desc)) = store.load_cached_packed::<Ac2ParticleSystemDesc>(
            TypedResourceRef::new(DatFileKind::Portal, DataId::new(did), 0),
        ) else {
            continue;
        };
        let mut system = ParticleSystem::new(
            Arc::new((*desc).clone()),
            Frame::IDENTITY,
            Vec3::ONE,
            1.0,
            did,
            100.0,
        );
        if !system.needs_repulsors() {
            continue;
        }
        let mut now = 100.0;
        for step in 0..150 {
            now += 1.0 / 30.0;
            let repulsors = [NearbyRepulsor {
                position: Vec3::new(step as f32 * 0.02, 0.0, 0.5),
                range: 3.0,
                acceleration: 8.0,
                attractor: step % 2 == 0,
            }];
            system.think_with(now, &repulsors);
        }
        let mut geometry = ParticleGeometry::default();
        system.fill(&basis, now, 1.0, &mut geometry);
        ran += 1;
        assert!(
            geometry
                .vertices
                .iter()
                .all(|v| v.position.iter().all(|c| c.is_finite())),
            "{did:08X}"
        );
    }
    assert!(ran > 0, "the data has repulse systems");
}
