//! `UiOverlay::set_layers` reuses each layer's texture and bind group across recomposes while the
//! size matches, and makes new ones when it does not. Runs when a GPU adapter exists.

use super::{UiLayer, UiOverlay};
use crate::{FrameTarget, Gpu, COLOR_FORMAT};

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

fn layer(width: u32, height: u32, rgba: &[u8]) -> UiLayer<'_> {
    UiLayer {
        x: 0,
        y: 0,
        width,
        height,
        rotation: 0,
        shown: true,
        opacity: 1.0,
        rgba,
    }
}

/// The 1 x 1 layer, at (1, 1) so it does not cover the pixel the test reads at (0, 0).
fn inset(rgba: &[u8]) -> UiLayer<'_> {
    UiLayer {
        x: 1,
        y: 1,
        ..layer(1, 1, rgba)
    }
}

#[test]
fn a_recompose_reuses_the_textures_of_same_size_layers() {
    let Ok(gpu) = Gpu::headless() else {
        eprintln!("no GPU adapter; skipping");
        return;
    };
    let big = [RED; 4].concat();
    let small = [GREEN; 1].concat();
    let mut overlay = UiOverlay::new(&gpu, COLOR_FORMAT);

    overlay.set_layers(&gpu, [layer(2, 2, &big), inset(&small)]);
    assert_eq!(
        overlay.textures_created, 2,
        "a new overlay makes each layer's texture"
    );

    // The same sizes again, new pixels: nothing new is made, and the pixels are the new ones.
    let big_again = [BLUE; 4].concat();
    overlay.set_layers(&gpu, [layer(2, 2, &big_again), inset(&small)]);
    assert_eq!(
        overlay.textures_created, 2,
        "same-size layers keep their textures"
    );
    assert_eq!(overlay.layers(), 2);
    overlay.prepare(&gpu.queue, 2, 2);
    let target = FrameTarget::new(&gpu.device, 2, 2);
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| overlay.draw(pass));
    let pixels = target.read_rgba(&gpu.device, &gpu.queue);
    assert_eq!(
        &pixels[0..4],
        &BLUE,
        "the reused texture shows the new pixels"
    );

    // A different size for layer 0 makes one new texture; layer 1 is reused.
    let bigger = [RED; 9].concat();
    overlay.set_layers(&gpu, [layer(3, 3, &bigger), inset(&small)]);
    assert_eq!(
        overlay.textures_created, 3,
        "only the resized layer makes a texture"
    );

    // Fewer layers drop the extra ones.
    overlay.set_layers(&gpu, [layer(3, 3, &bigger)]);
    assert_eq!(overlay.layers(), 1);
    assert_eq!(overlay.textures_created, 3);
}
