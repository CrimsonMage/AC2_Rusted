//! UI layers draw pixel for pixel and turn about their centre (`UISurfaceObject`).

use ac2_gpu::{FrameTarget, Gpu, UiLayer, UiOverlay, COLOR_FORMAT};

const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

fn draw(gpu: &Gpu, rotation: i32) -> Vec<[u8; 4]> {
    let rgba = [RED, GREEN, BLUE, WHITE].concat();
    let mut overlay = UiOverlay::new(gpu, COLOR_FORMAT);
    overlay.set_layers(
        gpu,
        [UiLayer {
            x: 1,
            y: 1,
            width: 2,
            height: 2,
            rotation,
            shown: true,
            opacity: 1.0,
            rgba: &rgba,
        }],
    );
    overlay.prepare(&gpu.queue, 4, 4);
    let target = FrameTarget::new(&gpu.device, 4, 4);
    target.render(&gpu.device, &gpu.queue, [0.0; 3], |pass| overlay.draw(pass));
    let pixels = target.read_rgba(&gpu.device, &gpu.queue);
    // The layer's 2 x 2 at (1, 1).
    [(1, 1), (2, 1), (1, 2), (2, 2)]
        .iter()
        .map(|&(x, y)| {
            let at = (y * 4 + x) * 4;
            [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
        })
        .collect()
}

#[test]
fn a_layer_draws_pixel_for_pixel_and_turns_anticlockwise() {
    let Ok(gpu) = Gpu::headless() else {
        return;
    };
    assert_eq!(draw(&gpu, 0), [RED, GREEN, BLUE, WHITE]);
    // A quarter turn (D3DX RotateZ in clip space, y up): what was on the right comes to the top.
    assert_eq!(draw(&gpu, 90), [GREEN, WHITE, RED, BLUE]);
    assert_eq!(draw(&gpu, 180), [WHITE, BLUE, GREEN, RED]);
}
