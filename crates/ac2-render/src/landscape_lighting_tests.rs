use super::*;

fn rule(combine_type: FogCombineType) -> FogCombine {
    FogCombine {
        combine_type,
        color_mult_first: 0.25,
        color_mult_second: 0.5,
    }
}

fn day() -> Final {
    Final {
        direction: Vec3::new(0.0, 0.6, -0.8),
        dir_bright: 0.5,
        dir_color: [1.0, 0.8, 0.6, 1.0],
        amb_bright: 0.5,
        amb_color: [0.4, 0.4, 0.6, 1.0],
        fog_color: [0.9, 0.9, 1.0, 1.0],
        fog_min: 100.0,
        fog_max: 2000.0,
        fog_sky: 0.1,
    }
}

#[test]
fn combine_follows_each_rule() {
    use FogCombineType::*;
    assert_eq!(combine(0.5, 0.25, &rule(MultiplyColors)), 0.125);
    // `ADD_COLORS`: second times its multiplier plus first times its own.
    assert_eq!(combine(0.4, 0.8, &rule(AddColors)), 0.8 * 0.5 + 0.4 * 0.25);
    assert_eq!(combine(0.4, 0.8, &rule(UseFirstColor)), 0.4);
    assert_eq!(combine(0.4, 0.8, &rule(UseSecondColor)), 0.8);
    assert_eq!(combine(0.4, 0.8, &rule(UseMaximumColor)), 0.8);
    assert_eq!(combine(0.4, 0.8, &rule(UseMinimumColor)), 0.4);
}

#[test]
fn the_day_light_is_scaled_to_its_brightest_channel_when_out_of_range() {
    // In range: direct by the light's length plus ambient by its brightness.
    let [r, g, b, a] = calc_day_light(&day());
    assert!((r - (1.0 * 0.5 + 0.4 * 0.5)).abs() < 1e-6);
    assert!((g - (0.8 * 0.5 + 0.4 * 0.5)).abs() < 1e-6);
    assert!((b - (0.6 * 0.5 + 0.6 * 0.5)).abs() < 1e-6);
    assert_eq!(a, 1.0);
    // Past 1: divided by the largest channel (`RGBAColor::Normalize`).
    let bright = Final {
        dir_bright: 2.0,
        ..day()
    };
    let [r, g, b, _] = calc_day_light(&bright);
    assert!((r - 1.0).abs() < 1e-6);
    assert!(g < 1.0 && b < 1.0);
}

#[test]
fn a_fog_area_takes_its_own_light_and_fog_where_its_rules_say_second() {
    use FogCombineType::*;
    let mut area = LightFogInfo {
        combine_ambient_brightness: rule(UseSecondColor),
        combine_ambient_color: rule(UseSecondColor),
        combine_day_fog: rule(UseSecondColor),
        combine_distance_fog: rule(UseSecondColor),
        ..Default::default()
    };
    area.light.amb_bright = 0.4;
    area.light.amb_color = [0.75, 0.77, 0.77, 1.0];
    area.fog.fog_min = 0.0;
    area.fog.fog_max = 1300.0;
    area.fog.fog_sky = 0.15;
    area.fog.fog_color = [0.32, 0.65, 0.84, 1.0];
    let out = calc_combine_fog(&day(), &area);
    // The direct light keeps the day's (`USE_FIRST_COLOR`).
    assert_eq!(out.light.dir_bright, 0.5);
    assert_eq!(out.light.dir_color, day().dir_color);
    assert_eq!(out.light.amb_bright, 0.4);
    assert_eq!(out.light.amb_color, [0.75, 0.77, 0.77, 1.0]);
    // The range and sky are the area's own, whatever the rules.
    assert_eq!(
        (out.fog.fog_min, out.fog.fog_max, out.fog.fog_sky),
        (0.0, 1300.0, 0.15)
    );
    assert_eq!(out.fog.fog_color, [0.32, 0.65, 0.84, 1.0]);
}

#[test]
fn the_corners_blend_by_where_the_viewer_stands_in_the_block() {
    let corner = |fog_max: f32| {
        let mut info = LightFogInfo::default();
        info.fog.fog_max = fog_max;
        info
    };
    // South-west, south-east, north-west, north-east.
    let corners = [corner(100.0), corner(200.0), corner(300.0), corner(400.0)];
    let at = |x, y| corner_interp(&corners, [x, y]).fog.fog_max;
    assert_eq!(at(0.0, 0.0), 100.0);
    assert_eq!(at(1.0, 0.0), 200.0);
    assert_eq!(at(0.0, 1.0), 300.0);
    assert_eq!(at(1.0, 1.0), 400.0);
    assert_eq!(at(0.5, 0.5), 250.0);
    // Outside the block the position is clamped to it.
    assert_eq!(at(-1.0, 2.0), 300.0);
}

#[test]
fn under_water_the_last_deeper_layer_and_the_one_before_it_blend_by_depth() {
    // Layers whose own fog replaces the day's (`USE_SECOND_COLOR`), keyed by fog distance.
    let layer = |depth: f32, fog_max: f32| WaterLightInfo {
        depth,
        water_light: LightFogInfo {
            combine_distance_fog: rule(FogCombineType::UseSecondColor),
            fog: ac2_world::FogInfo {
                fog_max,
                ..LightFogInfo::default().fog
            },
            ..Default::default()
        },
    };
    let layers = [layer(2.0, 50.0), layer(10.0, 20.0)];
    let fog_at = |depth| {
        calc_water_light(&day(), &layers, depth)
            .expect("layers")
            .fog
            .fog_max
    };
    // Shallower than both: the search keeps the last match (the deeper layer and the one before).
    // At 6 m, halfway from 10 back to 2: (6 - 10) / (2 - 10) = 0.5 of the shallower.
    assert!((fog_at(6.0) - 35.0).abs() < 1e-4, "{}", fog_at(6.0));
    assert!((fog_at(10.0 - 1e-3) - 20.0).abs() < 1e-2);
    // Past the deepest: the last layer alone.
    assert_eq!(fog_at(30.0), 20.0);
    assert!(calc_water_light(&day(), &[], 1.0).is_none());
}
