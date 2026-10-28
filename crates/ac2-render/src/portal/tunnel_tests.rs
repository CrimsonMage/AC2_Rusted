//! The tunnel's maths against values worked out from the decompile: the ring buffer's indices,
//! the sixteen points, `GeneratePortals`' chain of positions and radii, its rotation product,
//! `UpdateGeometry`'s vertices and the tunnel's advance.

use ac2_core::Random;
use ac2_math::matrix::Matrix4;
use ac2_math::waveform::kind;
use ac2_math::Waveform;

use super::effect::PortalSpace;
use super::tunnel::{
    fade_color as fade, make_safe_index, start_for_visibility, Portal, Tunnel, PORTALS,
};

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
    let mut expected = Matrix4::rotate_z(z)
        .multiply(&Matrix4::rotate_y(y))
        .multiply(&Matrix4::rotate_x(x));
    // The local matrix carries the spacing as its y translation (m[13]).
    expected.m[13] = 3.0;
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
    let inner = tunnel.update_geometry(0, 0, 1.0);
    // Portal 0 is ring step 0, which the buffer fills last: buffer ring 149.
    let ring = 149 * 17;
    // Memory runs backwards through the points: the ring's last vertex is point 0 (v = 0).
    let last = inner[ring + 16];
    // Point 0 is (1, 0, 0): scaled by radius 2 and the inner scale 1.
    assert_eq!(last.position, [2.0, 0.0, 0.0]);
    // The texture coordinates: u is the portal's position, v is the point's eighth.
    assert_eq!(last.uv, [3.0, 0.0]);
    // Memory vertex 4 is point 12 (v = 0.75): (cos 3pi/2, 0, sin 3pi/2) scaled by 2.
    assert_eq!(inner[ring + 4].uv, [3.0, 0.75]);
    assert!(
        close(inner[ring + 4].position[2], -2.0, 1e-6)
            && close(inner[ring + 4].position[0], 0.0, 1e-6)
    );
    // The closing vertex (memory 0) repeats point 0 at v = 1.
    assert_eq!(inner[ring].position, last.position);
    assert_eq!(inner[ring].uv, [3.0, 1.0]);
    // Step 0 is at the start: full colour.
    assert_eq!(last.color, 0xFFFF_FFFF);
    // The outer tunnel is 1.2 times the inner.
    let outer = tunnel.update_geometry(0, 0, 1.2);
    assert!(
        close(outer[ring + 16].position[0], 2.4, 1e-5),
        "{:?}",
        outer[ring + 16]
    );
}

#[test]
fn the_steps_before_the_start_are_transparent_and_the_start_is_full() {
    let tunnel = Tunnel::new();
    let rings = |start: usize| {
        let v = tunnel.update_geometry(0, start, 1.0);
        (0..PORTALS).map(|b| v[b * 17].color).collect::<Vec<u32>>()
    };
    let colors = rings(40);
    // Steps 0..40 are buffer rings 149 down to 110: transparent.
    assert!(colors[110..].iter().all(|&c| c == 0x00FF_FFFF));
    // Step 40 (buffer ring 109) is the start: full, and the next step fades a little.
    assert_eq!(colors[109], 0xFFFF_FFFF);
    assert!(colors[108] < 0xFFFF_FFFF && colors[108] > 0x00FF_FFFF);
}

#[test]
fn the_start_is_the_visibility_run_in_reverse() {
    assert_eq!(start_for_visibility(1.0), 0);
    assert_eq!(start_for_visibility(0.0), 149);
    assert_eq!(start_for_visibility(0.5), 74);
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

#[test]
fn the_ring_steps_start_at_the_current_portal_and_the_start_is_only_a_depth() {
    // `UpdateGeometry(current, start)`: step k is portal current + k; step k is transparent
    // while k < start, whatever portal it is.
    let mut tunnel = Tunnel::new();
    tunnel.portals[7].position = 9.0;
    tunnel.portals[10].position = 12.0;
    let v = tunnel.update_geometry(7, 3, 1.0);
    // Step 0 (buffer ring 149) is portal 7: transparent, before the start of 3 steps.
    let ring = (PORTALS - 1) * 17;
    assert_eq!(v[ring].uv[0], 9.0);
    assert_eq!(v[ring].color, 0x00FF_FFFF);
    // Step 3 (buffer ring 146) is portal 10 and the start: full colour.
    assert_eq!(v[146 * 17].uv[0], 12.0);
    assert_eq!(v[146 * 17].color, 0xFFFF_FFFF);
}
