//! The tunnel's maths against values worked out from the decompile: the ring buffer's indices,
//! the sixteen points, `GeneratePortals`' chain of positions and radii, its rotation product,
//! `UpdateGeometry`'s vertices and the tunnel's advance.

use ac2_core::Random;
use ac2_math::matrix::Matrix4;
use ac2_math::waveform::kind;
use ac2_math::Waveform;

use super::effect::PortalSpace;
use super::tunnel::{fade_color as fade, make_safe_index, Portal, Tunnel, PORTALS};

fn close(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

#[test]
fn the_ring_buffer_wraps_by_150() {
    assert_eq!(make_safe_index(-1), 149);
    assert_eq!(make_safe_index(0), 0);
    assert_eq!(make_safe_index(149), 149);
    assert_eq!(make_safe_index(150), 0);
    assert_eq!(make_safe_index(-151), 149);
}

#[test]
fn the_sixteen_points_are_the_unit_circle_in_eighths_of_a_turn() {
    let tunnel = Tunnel::new();
    let p = tunnel.points();
    assert_eq!((p[0][0], p[0][1], p[0][2]), (1.0, 0.0, 0.0));
    assert!(
        close(p[4][0], 0.0, 1e-6) && close(p[4][2], 1.0, 1e-6),
        "{:?}",
        p[4]
    );
    assert!(
        close(p[8][0], -1.0, 1e-6) && close(p[8][2], 0.0, 1e-6),
        "{:?}",
        p[8]
    );
}

#[test]
fn the_index_buffer_is_sixteen_quads_per_ring_and_closes_the_tube() {
    let tunnel = Tunnel::new();
    let indices = tunnel.index_buffer();
    assert_eq!(indices.len(), PORTALS * 16 * 6);
    // Ring 0's first quad: vertices 0 and 1 of ring 0, 0 and 1 of ring 1 (`17` per ring).
    assert_eq!(&indices[..6], &[0, 1, 17, 1, 18, 17]);
    // The last ring meets ring 0 again: ring 149's quads run to ring 0 (vertices 0 .. 16).
    let last = (PORTALS - 1) * 96;
    assert_eq!(&indices[last..last + 6], &[2533, 2534, 0, 2534, 1, 0]);
}

#[test]
fn generate_chains_the_positions_three_apart_and_the_radii_follow_the_sine() {
    let mut tunnel = Tunnel::new();
    tunnel.reset();
    tunnel.generate(0, PORTALS, 0.0);
    let portals = &tunnel.portals;
    assert_eq!(portals[0].position, 3.0);
    assert_eq!(portals[1].position, 6.0);
    assert_eq!(portals[149].position, 450.0);
    // The radius: 10 + 6 sin(0.01 * position), at position 3.
    let radius = 10.0 + 6.0 * (0.03f64.sin() as f32);
    assert!(
        close(portals[0].radius, radius, 1e-5),
        "{}",
        portals[0].radius
    );
}

#[test]
fn a_ring_is_the_product_of_its_three_perlin_turns_then_the_previous_ring() {
    let random_factor = 1234.5;
    let mut tunnel = Tunnel::new();
    tunnel.reset();
    tunnel.generate(0, 1, random_factor);
    let p = 3.0f64;
    let perlin = |amplitude: f32, arg: f64| {
        Waveform {
            kind: kind::PERLIN,
            base: 0.0,
            amplitude,
            phase: random_factor,
            frequency: 1.0,
            ..Waveform::default()
        }
        .value(arg)
    };
    let x = perlin(0.35, p * 0.005);
    let y = perlin(0.25, p * 0.002);
    let z = perlin(0.35, p * 0.005);
    // mRotZ = (C * B) * A with C = RotateZ(z), B = RotateY(y), A = RotateX(x); the previous ring
    // is the identity, so the world is mRotZ itself.
    let expected = Matrix4::rotate_z(z)
        .multiply(&Matrix4::rotate_y(y))
        .multiply(&Matrix4::rotate_x(x));
    assert_eq!(
        tunnel.portals[0].world,
        expected.multiply(&Matrix4::identity())
    );
    assert!(!tunnel.portals[0].has_particle);
}

#[test]
fn a_ring_with_a_radius_scales_its_points_before_the_matrix() {
    let mut tunnel = Tunnel::new();
    tunnel.portals[0] = Portal {
        position: 3.0,
        radius: 2.0,
        world: Matrix4::identity(),
        has_particle: false,
    };
    let inner = tunnel.update_geometry(0, 1.0);
    // Point 0 is (1, 0, 0): scaled by radius 2 and the inner scale 1.
    let first = inner[0];
    assert_eq!(first.position, [2.0, 0.0, 0.0]);
    // The ring's texture coordinates: u is the portal's position, v is the point's eighth.
    assert_eq!(first.uv, [3.0, 0.0]);
    assert_eq!(inner[4].uv, [3.0, 0.25]);
    assert!(close(inner[4].position[2], 2.0, 1e-6) && close(inner[4].position[0], 0.0, 1e-6));
    // The closing vertex repeats point 0 at v = 1.
    assert_eq!(inner[16].position, first.position);
    assert_eq!(inner[16].uv, [3.0, 1.0]);
    // The outer tunnel is 1.2 times the inner.
    let outer = tunnel.update_geometry(0, 1.2);
    assert!(close(outer[0].position[0], 2.4, 1e-5), "{:?}", outer[0]);
}

#[test]
fn the_ring_colour_is_full_in_the_first_sixteen_and_fades_beyond() {
    // Ring 0: 0.8 + 0.2 * 1 = 1.0, the alpha byte 255.
    assert_eq!(fade(0), 0xFFFF_FFFF);
    // Ring 20: 0.8 (1 - 20/149)^2 = 0.5996..., times 255 = 152.9..., truncated to 152.
    assert_eq!(fade(20), 0x98FF_FFFF);
    // Ring 15: 0.8 + 0.2 (1 - 15/15) = 0.8, times 255 = 204.
    assert_eq!(fade(15), 0xCCFF_FFFF);
}

#[test]
fn the_tunnel_advances_120_units_a_second_and_regenerates_what_it_passes() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    effect.enter(true, 0.0, &mut random);
    effect.think(0.0, &mut random);
    assert_eq!(effect.current_position(), 3.0);
    assert_eq!(effect.last_portal(), 0);
    effect.think(1.0, &mut random);
    // The camera has moved 120 units past the start: 123. Portal 40 is at 123, the first ahead
    // of it is 41 (126), so portal 40 is the one passed.
    assert_eq!(effect.current_position(), 123.0);
    assert_eq!(effect.last_portal(), 40);
    // The passed portals (0..40) were regenerated after the far end (149 at 450): 453 onward.
    assert_eq!(effect.tunnel().portals[0].position, 453.0);
    assert_eq!(effect.tunnel().portals[39].position, 570.0);
    assert_eq!(effect.tunnel().portals[40].position, 123.0);
}

#[test]
fn after_many_wraps_the_portals_ahead_stay_in_order_beyond_the_camera() {
    let mut random = Random::default();
    let mut effect = PortalSpace::new();
    effect.enter(true, 0.0, &mut random);
    effect.think(0.0, &mut random);
    let mut now = 0.0;
    for _ in 0..40 {
        now += 0.25;
        effect.think(now, &mut random);
    }
    let last = effect.last_portal();
    let current = effect.current_position();
    let ahead: Vec<f32> = (1..=20)
        .map(|k| effect.tunnel().portals[make_safe_index((last + k) as i64)].position)
        .collect();
    assert!(ahead.iter().all(|&p| p > current), "{ahead:?} vs {current}");
    assert!(ahead.windows(2).all(|w| w[0] < w[1]), "{ahead:?}");
}
