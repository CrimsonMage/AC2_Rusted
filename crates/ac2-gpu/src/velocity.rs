//! The world pass's second colour attachment: the screen-space velocity of the object that owns
//! each pixel, read by the temporal resolve (`taa.wgsl`). A WebGPU pass's pipelines must match
//! its attachments, so every pipeline that draws into [`HDR_FORMAT`] (the world pass and the
//! passes over the same scene target) declares the target; only the object pipelines'
//! depth-writing layers write it. Everything else drawn first (sky, terrain) leaves the clear.

use crate::HDR_FORMAT;

/// The velocity format: the previous minus the current screen position, in uv units.
pub const VELOCITY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

/// What the target holds where no object drew, or where its object has no previous placement
/// (`taa.wgsl` reads anything past 100 as "use the camera's motion"). Exact in half precision.
pub const NO_VELOCITY: f32 = 1000.0;

/// The colour targets of a pipeline drawing into `color`: just `color`, except in the scene
/// target's format, where the velocity attachment follows, written when `write_velocity`.
pub(crate) fn scene_targets(
    color: wgpu::ColorTargetState,
    write_velocity: bool,
) -> Vec<Option<wgpu::ColorTargetState>> {
    let with_velocity = color.format == HDR_FORMAT;
    let mut targets = vec![Some(color)];
    if with_velocity {
        targets.push(Some(wgpu::ColorTargetState {
            format: VELOCITY_FORMAT,
            blend: None,
            write_mask: if write_velocity {
                wgpu::ColorWrites::RED | wgpu::ColorWrites::GREEN
            } else {
                wgpu::ColorWrites::empty()
            },
        }));
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_scene_format_gets_a_velocity_target() {
        let hdr = scene_targets(HDR_FORMAT.into(), false);
        assert_eq!(hdr.len(), 2);
        let velocity = hdr[1].as_ref().unwrap();
        assert_eq!(velocity.format, VELOCITY_FORMAT);
        assert!(velocity.write_mask.is_empty());
        assert_eq!(
            scene_targets(HDR_FORMAT.into(), true)[1]
                .as_ref()
                .unwrap()
                .write_mask
                .bits(),
            3
        );
        assert_eq!(
            scene_targets(wgpu::TextureFormat::Rgba8Unorm.into(), true).len(),
            1
        );
    }

    /// Half floats hold every integer to 2048 exactly; a velocity is a few uv at most.
    const _: () = assert!(NO_VELOCITY == 1000.0 && NO_VELOCITY > 100.0);
}
