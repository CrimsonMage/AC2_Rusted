use super::*;

/// A Setup level with round distances: visible 100, near 10, ideal 30, far 60, high detail 5,
/// medium detail 10.
fn info() -> Ac2DegradeDistances {
    Ac2DegradeDistances {
        visible: 100.0,
        near: 10.0,
        ideal: 30.0,
        far: 60.0,
        high_detail: 5.0,
        medium_detail: 10.0,
    }
}

#[test]
fn scenery_scale_follows_scenery_draw_distance() {
    let mut options = DegradeOptions::default();
    assert_eq!(degrade_scale(&options, false, 60.0), f64::from(0.7_f32));
    options.scenery_draw_distance = 0;
    assert_eq!(degrade_scale(&options, false, 60.0), f64::from(0.4_f32));
    options.scenery_draw_distance = 2;
    assert_eq!(degrade_scale(&options, false, 60.0), 1.0);
}

#[test]
fn adaptive_scale_follows_the_frame_rate_only_when_on() {
    let mut options = DegradeOptions::default();
    assert_eq!(degrade_scale(&options, true, 5.0), 1.0);
    options.adaptive_degrade = true;
    // Bias 0: 41 frames a second; 20.5 fps is half, times 1.05, times 0.9, plus 0.1.
    let half = degrade_scale(&options, true, 20.5);
    let boost = 2.0 - f64::from(0.95_f32);
    let weight = f64::from(0.9_f32);
    assert_eq!(half, 0.5 * boost * weight + (1.0 - weight));
    assert!((half - 0.5725).abs() < 1e-6);
    // Fast frames saturate at 1 (0.9 + 0.1, in the float constants).
    assert!((degrade_scale(&options, true, 200.0) - 1.0).abs() < 1e-7);
    // The bias is clamped to 1: 81 frames a second for the full scale.
    options.adaptive_degrade_bias = 3.0;
    let at_81 = degrade_scale(&options, true, (81.0 / boost) as f32);
    assert!((at_81 - 1.0).abs() < 1e-6);
}

#[test]
fn objects_past_their_visible_distance_are_culled() {
    assert!(part_array_lod(100.5, &info(), 1.0, 2).is_none());
    // At the visible distance the fade leaves a sliver: the fade length rounds up to a float.
    let edge = part_array_lod(100.0, &info(), 1.0, 2).expect("not past visible");
    assert!(edge.alpha > 0.0 && edge.alpha < 1e-6, "{}", edge.alpha);
    assert!(part_array_lod(99.0, &info(), 1.0, 2).is_some());
    // Scaled by 0.4: visible 40.
    assert!(part_array_lod(41.0, &info(), 0.4, 2).is_none());
    // The visible distance is at least 20.
    let tiny = Ac2DegradeDistances {
        visible: 5.0,
        ..info()
    };
    let lod = part_array_lod(10.0, &tiny, 1.0, 2).expect("drawn inside 20");
    assert_eq!(lod.visible_distance, 20.0);
    assert!(part_array_lod(20.5, &tiny, 1.0, 2).is_none());
}

#[test]
fn the_last_seven_and_a_half_percent_fade_out() {
    // Fade 7.5 (0.075f * 100), from 92.5.
    let fade = f64::from((f64::from(0.075_f32) * 100.0) as f32);
    let start = f64::from((100.0 - fade) as f32);
    assert_eq!(part_array_lod(92.0, &info(), 1.0, 2).unwrap().alpha, 1.0);
    let alpha = part_array_lod(96.25, &info(), 1.0, 2).unwrap().alpha;
    assert_eq!(alpha, (1.0 - (96.25 - start) / fade) as f32);
    assert!((alpha - 0.5).abs() < 1e-5);
}

#[test]
fn the_factor_falls_from_one_at_near_to_zero_at_far() {
    let factor = |d: f32| part_array_lod(d, &info(), 1.0, 2).unwrap().factor;
    assert_eq!(factor(5.0), 1.0, "nearer than near clamps to 1");
    assert_eq!(factor(10.0), 1.0);
    assert_eq!(factor(20.0), 0.75);
    assert_eq!(factor(30.0), 0.5);
    assert_eq!(factor(45.0), 0.25);
    assert_eq!(factor(60.0), 0.0);
    assert_eq!(factor(80.0), 0.0, "past far clamps to 0");
    // Scaled by 0.7: ideal 21, far 42.
    let scaled = part_array_lod(31.5, &info(), f64::from(0.7_f32), 2).unwrap();
    assert!((scaled.factor - 0.25).abs() < 1e-5);
}

#[test]
fn equal_degrade_distances_use_the_guard_span() {
    let flat = Ac2DegradeDistances {
        near: 30.0,
        ideal: 30.0,
        far: 30.0,
        ..info()
    };
    // At the ideal distance the far span (0) is replaced by 0.0002.
    assert_eq!(part_array_lod(30.0, &flat, 1.0, 2).unwrap().factor, 0.5);
    assert_eq!(part_array_lod(30.1, &flat, 1.0, 2).unwrap().factor, 0.0);
    assert_eq!(part_array_lod(29.0, &flat, 1.0, 2).unwrap().factor, 1.0);
}

#[test]
fn material_detail_drops_with_distance() {
    // High detail to 15 (5 * 3), medium to 30 (10 * 3), low beyond.
    let material = |d: f32, m: u32| part_array_lod(d, &info(), 1.0, m).unwrap().material_detail;
    assert_eq!(material(15.0, 2), 2);
    assert_eq!(material(16.0, 2), 1);
    assert_eq!(material(30.0, 2), 1);
    assert_eq!(material(31.0, 2), 0);
    assert_eq!(material(10.0, 1), 1);
    assert_eq!(material(31.0, 1), 0);
    assert_eq!(material(10.0, 0), 0);
}

#[test]
fn alpha_test_is_forced_off_past_forty() {
    assert!(
        !part_array_lod(40.0, &info(), 1.0, 2)
            .unwrap()
            .force_disable_alpha_test
    );
    assert!(
        part_array_lod(40.5, &info(), 1.0, 2)
            .unwrap()
            .force_disable_alpha_test
    );
}

#[test]
fn object_lod_reads_the_model_detail_level() {
    let mut degrade = Ac2DegradeInfo::default();
    degrade.levels[0] = Ac2DegradeDistances {
        visible: 30.0,
        ..info()
    };
    degrade.levels[2] = info();
    let mut options = DegradeOptions {
        scenery_draw_distance: 2,
        model_detail: 0,
        ..DegradeOptions::default()
    };
    assert!(object_lod(50.0, &degrade, &options, false, 60.0).is_none());
    options.model_detail = 2;
    assert!(object_lod(50.0, &degrade, &options, false, 60.0).is_some());
}

#[test]
fn the_player_degrades_half_as_fast() {
    assert_eq!(mesh_degrade_factor(0.5, false), 0.5);
    assert_eq!(mesh_degrade_factor(0.5, true), 0.75);
    assert_eq!(mesh_degrade_factor(0.8, true), 1.0);
    assert_eq!(mesh_degrade_factor(-0.1, false), 0.0);
}

#[test]
fn levels_truncate_toward_the_first_degrade() {
    assert_eq!(degrade_level(1.0, 4), 0);
    assert_eq!(degrade_level(0.0, 4), 3);
    // 3 * 0.5 = 1.5 truncates to 1.
    assert_eq!(degrade_level(0.5, 4), 1);
    // 3 * (1 - 0.34) = 1.98 truncates to 1.
    assert_eq!(degrade_level(0.34, 4), 1);
    assert_eq!(degrade_level(0.33, 4), 2);
    assert_eq!(degrade_level(0.0, 1), 0);
    assert_eq!(degrade_level(0.0, 0), 0);
}

#[test]
fn detail_lists_hold_the_degrades_each_setting_admits() {
    // Degrade 0 for medium and high, 1 for every setting, 2 for low only.
    let lists = DegradeLists::new(&[6, 7, 1]);
    assert_eq!(lists.list(0), &[1, 2]);
    assert_eq!(lists.list(1), &[0, 1]);
    assert_eq!(lists.list(2), &[0, 1]);
    assert_eq!(lists.select(2, 1.0), Some(0));
    assert_eq!(lists.select(2, 0.0), Some(1));
    assert_eq!(lists.select(0, 1.0), Some(1));
    assert_eq!(lists.select(0, 0.0), Some(2));
}

#[test]
fn an_empty_detail_list_picks_among_every_degrade() {
    let lists = DegradeLists::new(&[4, 4, 4]);
    assert_eq!(lists.select(0, 1.0), Some(0));
    assert_eq!(lists.select(0, 0.0), Some(2));
    assert_eq!(lists.select(0, 0.5), Some(1));
    assert_eq!(DegradeLists::new(&[]).select(2, 1.0), None);
}

#[test]
fn disabled_degrades_draw_full_detail_with_the_alpha_test_rule_kept() {
    let options = DegradeOptions::default();
    // Far past the 100 unit visible distance: degrading culls it, the switch draws it whole.
    assert!(part_array_lod(500.0, &info(), 1.0, 1).is_none());
    let lod = undegraded_lod(500.0, &options);
    assert_eq!((lod.factor, lod.alpha), (1.0, 1.0));
    assert_eq!(lod.material_detail, options.material_detail);
    assert!(lod.force_disable_alpha_test);
    assert!(!undegraded_lod(40.0, &options).force_disable_alpha_test);
    assert_eq!(lod.draw_detail(&options, true).degrade, 1.0);
}
