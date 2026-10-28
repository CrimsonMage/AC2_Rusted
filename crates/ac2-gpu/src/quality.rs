//! The overall graphics quality a first run starts at, from the GPU. A client improvement over
//! retail (`AGENTS.md`): `Render::InitRenderingPreferences` applied only a saved level and
//! otherwise kept the registered defaults (level 3's values), sized for 2002 hardware.

/// `AdapterInfo::vendor` of Apple GPUs (the PCI vendor ID Metal reports).
const APPLE_VENDOR: u32 = 0x106B;
/// Retail's top preset (`Render::SetOverallGraphicsQuality(5)`).
const HIGH: u32 = 5;
/// The preset matching retail's registered defaults (medium model and scenery detail).
const MEDIUM: u32 = 3;

/// The level for a GPU of `device_type` from `vendor`: the top preset on a discrete GPU or Apple
/// silicon, the retail defaults' level on integrated, virtual or software adapters.
pub fn quality_level_for(device_type: wgpu::DeviceType, vendor: u32) -> u32 {
    match device_type {
        wgpu::DeviceType::DiscreteGpu => HIGH,
        wgpu::DeviceType::IntegratedGpu if vendor == APPLE_VENDOR => HIGH,
        _ => MEDIUM,
    }
}

/// [`quality_level_for`] the adapter a window would get (high-performance preference), found
/// without opening a device; the retail defaults' level when there is none.
pub fn suggested_quality_level() -> u32 {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
        apply_limit_buckets: false,
    }));
    adapter.map_or(MEDIUM, |adapter| {
        let info = adapter.get_info();
        quality_level_for(info.device_type, info.vendor)
    })
}

#[cfg(test)]
mod tests {
    use super::quality_level_for;
    use wgpu::DeviceType;

    #[test]
    fn discrete_and_apple_gpus_start_high_and_the_rest_at_the_retail_defaults() {
        assert_eq!(quality_level_for(DeviceType::DiscreteGpu, 0x10DE), 5);
        assert_eq!(quality_level_for(DeviceType::IntegratedGpu, 0x106B), 5);
        assert_eq!(quality_level_for(DeviceType::IntegratedGpu, 0x8086), 3);
        assert_eq!(quality_level_for(DeviceType::VirtualGpu, 0x1AF4), 3);
        assert_eq!(quality_level_for(DeviceType::Cpu, 0), 3);
        assert_eq!(quality_level_for(DeviceType::Other, 0), 3);
    }
}
