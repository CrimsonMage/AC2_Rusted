//! `UIElement::SetIconDesc`'s layers: multiplied by their shift colours and blended bottom first.

use std::collections::HashMap;
use std::sync::Arc;

use super::*;

fn image(pixels: &[[u8; 4]]) -> UiPixels {
    UiPixels {
        width: pixels.len() as u32,
        height: 1,
        rgba: Arc::from(pixels.concat()),
    }
}

const WHITE: u32 = 0xFFFF_FFFF;

#[test]
fn a_white_shift_keeps_the_layer_and_a_half_shift_halves_it() {
    let images = HashMap::from([(1, image(&[[200, 100, 50, 255]]))]);
    let plain = composite(
        &[IconLayer {
            image: 1,
            shift_color: WHITE,
        }],
        &images,
    )
    .unwrap();
    assert_eq!(&plain.rgba[..], &[200, 100, 50, 255]);
    // 0x80 is 128/255 of each channel: 200 * 128/255 truncated is 100 (the floats round down).
    let half = composite(
        &[IconLayer {
            image: 1,
            shift_color: 0x8080_8080,
        }],
        &images,
    )
    .unwrap();
    assert_eq!(&half.rgba[..], &[100, 50, 25, 128]);
}

#[test]
fn later_layers_blend_over_the_first_by_their_alpha() {
    let images = HashMap::from([
        (1, image(&[[0, 0, 0, 255], [0, 0, 0, 255]])),
        (2, image(&[[255, 255, 255, 255], [255, 255, 255, 128]])),
    ]);
    let icon = composite(
        &[
            IconLayer {
                image: 1,
                shift_color: WHITE,
            },
            IconLayer {
                image: 2,
                shift_color: WHITE,
            },
        ],
        &images,
    )
    .unwrap();
    // Opaque: copied. Half alpha: 0 - (0 - 255) * 128 / 256 = 127.
    assert_eq!(&icon.rgba[..4], &[255, 255, 255, 255]);
    assert_eq!(&icon.rgba[4..8], &[127, 127, 127, 255]);
}

#[test]
fn a_missing_upper_layer_is_left_out_and_a_missing_base_draws_nothing() {
    let images = HashMap::from([(1, image(&[[9, 9, 9, 255]]))]);
    let layers = [
        IconLayer {
            image: 1,
            shift_color: WHITE,
        },
        IconLayer {
            image: 7,
            shift_color: WHITE,
        },
    ];
    assert_eq!(
        &composite(&layers, &images).unwrap().rgba[..],
        &[9, 9, 9, 255]
    );
    assert!(composite(&layers[1..], &images).is_none());
}

#[test]
fn the_element_asks_for_every_layers_image() {
    let mut tree = UiTree::new(10, 10);
    let layout = Arc::new(crate::layout::LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 10,
        height: 10,
        elements: vec![Arc::new(crate::text_tests::element(
            "Icon",
            0x4DA7_2ECC,
            crate::layout::Rect {
                x0: 0,
                y0: 0,
                x1: 9,
                y1: 9,
            },
            vec![],
            vec![],
        ))],
    });
    let icon = tree
        .create_root(layout, crate::token::tokenize("Icon"))
        .unwrap();
    tree.set_icon_desc(
        icon,
        &[
            IconLayer {
                image: 0x0600_0001,
                shift_color: WHITE,
            },
            IconLayer {
                image: 0x0600_0002,
                shift_color: WHITE,
            },
        ],
    );
    let files = tree.image_files();
    assert!(files.contains(&0x0600_0001) && files.contains(&0x0600_0002));
}

/// `IconDesc::IsValid`: every layer has a layer id and a shift colour, and one has an image.
#[test]
fn an_icon_desc_is_valid_only_with_ids_colours_and_an_image() {
    assert!(icon_desc_valid(&[(1000, 7, WHITE)]));
    assert!(icon_desc_valid(&[(1000, 7, WHITE), (3000, 0, 0xFFDD_6060)]));
    assert!(!icon_desc_valid(&[]), "no image");
    assert!(!icon_desc_valid(&[(1000, 0, WHITE)]), "no image");
    assert!(!icon_desc_valid(&[(1000, 7, 0)]), "a null shift colour");
    assert!(!icon_desc_valid(&[(0, 7, WHITE)]), "no layer id");
}

/// A sale's icon: a valid description is drawn as layers, an invalid one falls back to the icon
/// id in draw mode 2.
#[test]
fn a_sale_icon_takes_its_description_or_its_icon_id() {
    use ac2_net::protocol::IconLayer as Wire;
    let mut tree = UiTree::new(10, 10);
    let layout = Arc::new(crate::layout::LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 10,
        height: 10,
        elements: vec![Arc::new(crate::text_tests::element(
            "Icon",
            0x4DA7_2ECC,
            crate::layout::Rect {
                x0: 0,
                y0: 0,
                x1: 9,
                y1: 9,
            },
            vec![],
            vec![],
        ))],
    });
    let icon = tree
        .create_root(layout, crate::token::tokenize("Icon"))
        .unwrap();
    let valid = [Wire {
        layer: 1000,
        image: 0x0600_0001,
        color: WHITE,
    }];
    tree.set_sale_icon(icon, &valid, 0x0600_0009);
    assert!(tree.get(icon).unwrap().icon_layers.is_some());
    let invalid = [Wire {
        layer: 1000,
        image: 0x0600_0001,
        color: 0,
    }];
    tree.set_sale_icon(icon, &invalid, 0x0600_0009);
    let el = tree.get(icon).unwrap();
    assert!(el.icon_layers.is_none());
    assert_eq!(
        el.image().map(|i| (i.file, i.draw_mode)),
        Some((0x0600_0009, 2))
    );
}
