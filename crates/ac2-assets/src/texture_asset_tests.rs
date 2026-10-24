use std::sync::Arc;

use ac2_core::DataId;
use ac2_core::PixelFormatID;
use ac2_dat::DatFileKind;

use super::{
    texture_asset_from_render_texture, RenderTextureMipPolicy, TextureAsset, TexturePayload,
    TextureShape, TextureWrapMode,
};
use crate::{DecodedTextureAsset, DecodedTextureLevel, DecodedTextureShape, DecodedTextureSurface};
#[test]
fn fidelity_never_drops_the_last_level() {
    assert_eq!(
        RenderTextureMipPolicy {
            environment_texture_detail: 4,
            high_detail_available: true
        }
        .leading_levels_to_drop(1),
        0
    );
}

#[test]
fn checked_repeating_rgba_rejects_malformed_pixels() {
    assert!(TextureAsset::try_repeating(2, 2, Arc::from([0_u8; 15])).is_none());
}
#[test]
fn fidelity_matches_retail_table() {
    assert_eq!(
        RenderTextureMipPolicy::default().leading_levels_to_drop(5),
        0
    );
    assert_eq!(
        RenderTextureMipPolicy {
            environment_texture_detail: 0,
            high_detail_available: false
        }
        .leading_levels_to_drop(5),
        1
    );
    assert_eq!(
        RenderTextureMipPolicy {
            environment_texture_detail: 3,
            high_detail_available: true
        }
        .leading_levels_to_drop(5),
        3
    );
}

fn surface(face: usize, width: u32, height: u32, value: u8) -> DecodedTextureSurface {
    DecodedTextureSurface {
        authored_face_index: face,
        render_surface_file_kind: DatFileKind::Portal,
        render_surface_did: DataId::new(face as u32 + 1),
        width,
        height,
        pixel_format: PixelFormatID::PFID_UNKNOWN,
        rgba: Arc::from(vec![value; width as usize * height as usize * 4]),
        clip_map_transform: false,
        compressed: None,
        resolved_identity: None,
    }
}

#[test]
fn cube_selection_retains_all_faces_and_selected_mips() {
    let texture = DecodedTextureAsset {
        source_kind: crate::TextureSourceKind::AuthoredRenderTexture,
        texture_did: DataId::new(42),
        shape: DecodedTextureShape::Cube,
        raw_texture_type: 4,
        diagnostic_fallback: false,
        levels: Arc::from([
            DecodedTextureLevel {
                source_level_did: DataId::new(1),
                surfaces: Arc::from(
                    (0..6)
                        .map(|face| surface(face, 4, 4, face as u8))
                        .collect::<Vec<_>>(),
                ),
            },
            DecodedTextureLevel {
                source_level_did: DataId::new(2),
                surfaces: Arc::from(
                    (0..6)
                        .map(|face| surface(face, 2, 2, face as u8 + 10))
                        .collect::<Vec<_>>(),
                ),
            },
        ]),
    };
    let asset = texture_asset_from_render_texture(
        &texture,
        RenderTextureMipPolicy {
            environment_texture_detail: 1,
            high_detail_available: true,
        },
        TextureWrapMode::Clamp,
    )
    .unwrap();
    assert_eq!(asset.shape, TextureShape::Cube);
    assert_eq!((asset.width, asset.height, asset.level_count()), (2, 2, 1));
    match asset.payload() {
        TexturePayload::Rgba(levels) => {
            assert_eq!(levels.levels[0].faces.len(), 6);
            assert_eq!(levels.levels.len(), 1);
        }
        _ => panic!("expected RGBA cube"),
    }
}

#[test]
fn rgba_render_texture_preserves_authored_mip_chain_without_generating_levels() {
    let mut high = surface(0, 16, 16, 0);
    high.rgba = Arc::from(
        (0..256)
            .flat_map(|pixel| {
                let value = if (pixel % 16 + pixel / 16) % 2 == 0 {
                    0
                } else {
                    255
                };
                [value, value, value, 255]
            })
            .collect::<Vec<_>>(),
    );
    let one = DecodedTextureAsset {
        source_kind: crate::TextureSourceKind::AuthoredRenderTexture,
        texture_did: DataId::new(42),
        shape: DecodedTextureShape::Texture2d,
        raw_texture_type: 0,
        diagnostic_fallback: false,
        levels: Arc::from([DecodedTextureLevel {
            source_level_did: DataId::new(1),
            surfaces: Arc::from([high.clone()]),
        }]),
    };
    let authored = texture_asset_from_render_texture(
        &one,
        RenderTextureMipPolicy::default(),
        TextureWrapMode::Repeat,
    )
    .unwrap();
    assert_eq!(authored.level_count(), 1);
    let TexturePayload::Rgba(mips) = authored.payload() else {
        panic!("expected RGBA mips");
    };
    assert!(Arc::ptr_eq(&mips.levels[0].faces[0], &high.rgba));
    assert_eq!(authored.mip_tail(4), authored);

    let two = DecodedTextureAsset {
        source_kind: crate::TextureSourceKind::AuthoredRenderTexture,
        levels: Arc::from([
            one.levels[0].clone(),
            DecodedTextureLevel {
                source_level_did: DataId::new(2),
                surfaces: Arc::from([surface(0, 8, 8, 37)]),
            },
        ]),
        ..one
    };
    let preserved = texture_asset_from_render_texture(
        &two,
        RenderTextureMipPolicy::default(),
        TextureWrapMode::Repeat,
    )
    .unwrap();
    let TexturePayload::Rgba(mips) = preserved.payload() else {
        panic!("expected RGBA mips");
    };
    assert_eq!(mips.levels.len(), 2);
    assert!(Arc::ptr_eq(&mips.levels[0].faces[0], &high.rgba));
    assert_eq!(mips.levels[1].faces[0][0], 37);
}

#[test]
fn selected_mips_do_not_keep_dropped_high_resolution_pixels_alive() {
    let high = surface(0, 4, 4, 9);
    let weak = Arc::downgrade(&high.rgba);
    let texture = DecodedTextureAsset {
        source_kind: crate::TextureSourceKind::AuthoredRenderTexture,
        texture_did: DataId::new(42),
        shape: DecodedTextureShape::Texture2d,
        raw_texture_type: 0,
        diagnostic_fallback: false,
        levels: Arc::from([
            DecodedTextureLevel {
                source_level_did: DataId::new(1),
                surfaces: Arc::from([high]),
            },
            DecodedTextureLevel {
                source_level_did: DataId::new(2),
                surfaces: Arc::from([surface(0, 2, 2, 4)]),
            },
        ]),
    };
    let converted = texture_asset_from_render_texture(
        &texture,
        RenderTextureMipPolicy {
            environment_texture_detail: 1,
            high_detail_available: true,
        },
        TextureWrapMode::Repeat,
    )
    .unwrap();
    assert_eq!(converted.base_rgba().unwrap().len(), 16);
    drop(texture);
    assert_eq!(weak.strong_count(), 0);
    assert!(converted.decoded_levels.is_none());
}

#[test]
fn native_conversion_does_not_pin_an_unused_rgba_expansion() {
    let mut face = surface(0, 4, 4, 5);
    let weak_rgba = Arc::downgrade(&face.rgba);
    face.compressed =
        crate::texture_dxt::retain(PixelFormatID::PFID_DXT1, 4, 4, &Arc::from([0_u8; 8]));
    let texture = DecodedTextureAsset {
        source_kind: crate::TextureSourceKind::AuthoredRenderTexture,
        texture_did: DataId::new(42),
        shape: DecodedTextureShape::Texture2d,
        raw_texture_type: 0,
        diagnostic_fallback: false,
        levels: Arc::from([DecodedTextureLevel {
            source_level_did: DataId::new(1),
            surfaces: Arc::from([face]),
        }]),
    };
    let converted = texture_asset_from_render_texture(
        &texture,
        RenderTextureMipPolicy::default(),
        TextureWrapMode::Repeat,
    )
    .unwrap();
    assert!(matches!(
        converted.payload(),
        TexturePayload::BlockCompressed(_)
    ));
    drop(texture);
    assert_eq!(weak_rgba.strong_count(), 0);
    assert!(converted.decoded_levels.is_none());
}
