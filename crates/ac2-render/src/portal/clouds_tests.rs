//! The tunnel clouds against the DAT's description (`AC2_DAT_DIR`) and the `has_particle` rules
//! of `Render`'s discharge loop and `GeneratePortals`.

use ac2_core::Random;

use super::tunnel::{Tunnel, PORTALS};
use super::{load_clouds_desc, load_portal_models, PortalClouds, PortalSpace};
use crate::particles::CameraBasis;

#[test]
fn generating_a_portal_clears_its_particle_mark() {
    let mut tunnel = Tunnel::new();
    tunnel.generate(0, PORTALS, 0.0);
    for portal in &mut tunnel.portals {
        portal.has_particle = true;
    }
    tunnel.generate(10, 5, 0.0);
    for (index, portal) in tunnel.portals.iter().enumerate() {
        assert_eq!(portal.has_particle, !(10..15).contains(&index), "{index}");
    }
    tunnel.reset();
    assert!(tunnel.portals.iter().all(|p| !p.has_particle));
}

#[test]
fn the_dat_has_a_background_mesh_and_a_cloud_system() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let models = load_portal_models(&store);
    assert!(
        models.background.is_some(),
        "the background mesh (0x40000016)"
    );
    assert!(models.clouds.is_some(), "the clouds' material (0x40000015)");
    assert!(load_clouds_desc(&store).is_some());
}

#[test]
fn a_discharge_marks_the_ten_portals_ahead_once_and_the_clouds_fill_with_particles() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let mut now = 50.0;
    let mut clouds = PortalClouds::new(load_clouds_desc(&store), now);
    let mut tunnel = Tunnel::new();
    tunnel.generate(0, PORTALS, 1234.0);
    let mut random = Random::default();
    // At 0.9 the discharge does not run; above it, portals 40 to 49 ahead are marked.
    clouds.discharge(&mut tunnel, 3, 0.9, now, &mut random);
    assert!(tunnel.portals.iter().all(|p| !p.has_particle));
    clouds.discharge(&mut tunnel, 3, 1.0, now, &mut random);
    let marked: Vec<usize> = (0..PORTALS)
        .filter(|&i| tunnel.portals[i].has_particle)
        .collect();
    assert_eq!(marked, (43..53).collect::<Vec<_>>());
    // Marked portals do not roll again.
    let mut again = Random::default();
    clouds.discharge(&mut tunnel, 3, 1.0, now, &mut again);
    let basis =
        CameraBasis::look_at(ac2_math::Vec3::ZERO, ac2_math::Vec3::new(0.0, 1.0, 0.0)).unwrap();
    let mut drew = false;
    for _ in 0..120 {
        now += 1.0 / 30.0;
        drew |= !clouds.geometry(now, &basis).is_empty();
    }
    assert!(drew, "the discharged particles are drawn");
    assert!(clouds.material().is_some());
}

#[test]
fn the_effect_queues_cloud_particles_while_the_tunnel_runs() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset);
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    let mut now = 10.0;
    effect.set_clouds(load_clouds_desc(&store), now);
    effect.enter(false, now, &mut random);
    let mut peak = 0;
    for _ in 0..(6 * 30) {
        effect.think(now, &mut random);
        effect.render_clouds(now, &mut random);
        peak = peak.max(effect.cloud_geometry().vertices.len());
        now += 1.0 / 30.0;
    }
    assert!(peak > 0, "the tunnel clouds drew no particle");
    // The tunnel leaves: nothing is queued once the tunnels are not drawn.
    effect.exit(true, now);
    effect.think(now + 5.0, &mut random);
    effect.render_clouds(now + 5.0, &mut random);
    assert!(effect.cloud_geometry().is_empty());
}
