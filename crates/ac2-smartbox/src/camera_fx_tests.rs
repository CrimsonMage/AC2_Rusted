use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_db::{ResourcePackedDecode, ResourceStore};

use super::*;

const DID: u32 = 0x5500_0001;

fn decode(words: &[u32]) -> ResourceResult<CameraFx> {
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    CameraFx::decode_packed(QualifiedDataId::new(DataId::new(DID), 0), 0, &bytes)
}

#[test]
fn a_record_reads_the_fields_its_bitfield_names_in_stream_order() {
    // Shake, intensity 2, period 3, ramp down 0.5 (streamed before the flash), a red flash.
    let bits = 0x1 | 0x40 | 0x80 | 0x2000 | 0x400;
    let fx = decode(&[
        DID,
        bits,
        0,
        1,
        2.0_f32.to_bits(),
        3.0_f32.to_bits(),
        0.5_f32.to_bits(),
        1,
        0x80FF_0000,
    ])
    .expect("decodes");
    assert!(fx.shake && fx.has_screen_flash && !fx.blur);
    assert_eq!((fx.intensity, fx.period), (2.0, 3.0));
    assert_eq!((fx.ramp_up_time, fx.ramp_down_time), (0.01, 0.5));
    assert_eq!(fx.screen_flash, [1.0, 0.0, 0.0, 128.0 / 255.0]);
}

#[test]
fn a_record_under_another_did_or_with_trailing_bytes_is_refused() {
    assert!(decode(&[DID + 1, 0, 0]).is_err());
    assert!(decode(&[DID, 0, 0, 7]).is_err());
    assert_eq!(decode(&[DID, 0, 0]).expect("empty"), CameraFx::default());
}

#[test]
fn every_camera_fx_in_the_dat_decodes() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    let ids: Vec<u32> = dataset
        .ids_in_range(
            DatFileKind::Portal,
            CAMERA_FX_PREFIX,
            CAMERA_FX_PREFIX | 0x00FF_FFFE,
        )
        .collect();
    assert_eq!(ids.len(), 7, "CameraFX records in portal.dat");
    for did in ids {
        let fx = load_camera_fx(&store, did)
            .unwrap_or_else(|e| panic!("{did:08X} decode error: {e:?}"))
            .unwrap_or_else(|| panic!("{did:08X} missing"));
        assert!(fx.period > 0.0 && fx.intensity.is_finite(), "{did:08X}");
    }
}

fn shake(period: f32, up: f32, down: f32) -> CameraFx {
    CameraFx {
        shake: true,
        period,
        ramp_up_time: up,
        ramp_down_time: down,
        ..CameraFx::default()
    }
}

#[test]
fn a_track_ramps_in_holds_and_ramps_out_then_ends() {
    let mut fx = CameraFxManager::default();
    fx.add(TrackKind::Fov, &shake(4.0, 1.0, 1.0), 1, 20.0);
    fx.use_time(0.5);
    assert_eq!(fx.tracks[0].weight, 0.5);
    assert_eq!(fx.fov_sum, 10.0, "20 at half weight");
    fx.use_time(1.0);
    assert_eq!(fx.fov_sum, 20.0);
    fx.use_time(2.0);
    assert_eq!(fx.tracks[0].weight, 0.5);
    fx.use_time(1.0);
    assert!(fx.tracks.is_empty(), "past its period");
    assert_eq!(fx.fov_sum, 10.0);
}

#[test]
fn the_sums_are_clamped() {
    let mut fx = CameraFxManager::default();
    let instant = shake(10.0, 0.01, 0.01);
    fx.add(TrackKind::Fov, &instant, 1, 400.0);
    fx.add(TrackKind::Blur, &instant, 1, 3.0);
    fx.add(
        TrackKind::ScreenFlash([1.0, 0.5, 0.0, 1.0]),
        &instant,
        1,
        4.0,
    );
    fx.use_time(1.0);
    assert_eq!(fx.fov_sum, 150.0);
    assert_eq!(fx.blur_sum, 1.0);
    assert_eq!(fx.flash_sum, [1.0, 1.0, 0.0, 1.0]);
    fx.stop_all();
    fx.use_time(0.1);
    assert_eq!((fx.fov_sum, fx.blur_sum), (10.0, 0.0));
}

#[test]
fn the_shake_falls_off_with_its_sources_distance() {
    let mut near = CameraFxManager::default();
    near.add(TrackKind::Shake, &shake(10.0, 0.01, 0.01), 7, 1.0);
    let mut far = near.clone();
    let mut roll = |_: f32, _: f32| 3.0;
    near.use_time(0.37);
    far.use_time(0.37);
    let a = near.distortion(&mut |_| Some(0.5), &mut roll);
    let b = far.distortion(&mut |_| Some(4.0), &mut roll);
    assert!(a.x != 0.0, "{a:?}");
    assert!((a.x / 4.0 - b.x).abs() < 1e-6, "{a:?} {b:?}");
    let gone = near.distortion(&mut |_| None, &mut roll);
    assert_eq!(gone, Vector::new(0.0, 0.0, 0.0));
    let mut still = CameraFxManager::default();
    assert_eq!(
        still.distortion(&mut |_| Some(1.0), &mut roll),
        Vector::new(0.0, 0.0, 0.0)
    );
}
