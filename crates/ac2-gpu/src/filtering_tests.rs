use super::*;
use crate::object_state::SamplerKey;

const ALL: wgpu::DownlevelFlags = wgpu::DownlevelFlags::all();

#[test]
fn trilinear_off_gives_a_nearest_mip() {
    let filtering = TextureFiltering::new(false, false, ALL);
    assert_eq!(
        filtering.for_stage(true, true),
        (wgpu::MipmapFilterMode::Nearest, 1)
    );
    let on = TextureFiltering::new(true, false, ALL);
    assert_eq!(
        on.for_stage(true, true),
        (wgpu::MipmapFilterMode::Linear, 1)
    );
}

#[test]
fn anisotropy_needs_linear_min_and_mag() {
    let filtering = TextureFiltering::new(true, true, ALL);
    assert_eq!(
        filtering.for_stage(true, true),
        (wgpu::MipmapFilterMode::Linear, 16)
    );
    assert_eq!(filtering.for_stage(false, true).1, 1);
    assert_eq!(filtering.for_stage(true, false).1, 1);
    assert_eq!(filtering.for_stage(false, false).1, 1);
}

#[test]
fn an_anisotropic_stage_forces_a_linear_mip() {
    let filtering = TextureFiltering::new(false, true, ALL);
    assert_eq!(
        filtering.for_stage(true, true),
        (wgpu::MipmapFilterMode::Linear, 16)
    );
    // A point stage keeps the preference's nearest mip.
    assert_eq!(
        filtering.for_stage(false, false),
        (wgpu::MipmapFilterMode::Nearest, 1)
    );
}

#[test]
fn an_adapter_without_anisotropic_filtering_clamps_to_one() {
    let flags = wgpu::DownlevelFlags::all() - wgpu::DownlevelFlags::ANISOTROPIC_FILTERING;
    let filtering = TextureFiltering::new(true, true, flags);
    assert_eq!(filtering.anisotropy, 1);
    assert_eq!(filtering.for_stage(true, true).1, 1);
}

#[test]
fn stage_sampler_descriptors_follow_the_filtering() {
    let linear = SamplerKey::for_modes(1, 1, 2, 2, TextureFiltering::new(false, true, ALL));
    let d = linear.descriptor();
    assert_eq!(d.anisotropy_clamp, 16);
    assert_eq!(d.mipmap_filter, wgpu::MipmapFilterMode::Linear);
    let point = SamplerKey::for_modes(1, 1, 1, 1, TextureFiltering::new(false, true, ALL));
    let d = point.descriptor();
    assert_eq!(d.anisotropy_clamp, 1);
    assert_eq!(d.mipmap_filter, wgpu::MipmapFilterMode::Nearest);
    assert_eq!(d.min_filter, wgpu::FilterMode::Nearest);
    let plain = SamplerKey::for_modes(1, 1, 2, 2, TextureFiltering::default());
    assert_eq!(plain.descriptor().anisotropy_clamp, 1);
    assert_eq!(
        plain.descriptor().mipmap_filter,
        wgpu::MipmapFilterMode::Linear
    );
}
