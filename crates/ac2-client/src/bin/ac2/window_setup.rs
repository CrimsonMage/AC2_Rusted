//! The window's and the surface's set-up helpers: the fullscreen mode, the present mode, the
//! depth target, the speakers and the world name in the title.

use ac2_gpu::DEPTH_FORMAT;
use ac2_weenie::SessionEvent;
use winit::window::{Fullscreen, Window};

/// Borderless fullscreen on the window's current monitor (`Windowed = !FullScreen`; retail's
/// exclusive display change has no portable form, see `window_presentation`).
pub(crate) fn borderless() -> Fullscreen {
    Fullscreen::Borderless(None)
}

/// The surface's present mode for vertical sync: FIFO (`PresentationInterval` ONE), else immediate
/// presentation, which falls back to mailbox and FIFO where immediate is not supported.
pub(crate) fn present_mode(vsync: bool) -> wgpu::PresentMode {
    if vsync {
        wgpu::PresentMode::Fifo
    } else {
        wgpu::PresentMode::AutoNoVsync
    }
}

/// `gmClient::RecvNotice_WorldName`: the window title carries the world's name, a modern stand-in
/// for the HUD's display. The HUD scripts read the name through the efun
/// `gTurbineGunUtils::CallEFun_GetWorldName` (bound in `ac2_script`).
fn world_title(info: &ac2_net::protocol::StringInfo) -> Option<String> {
    info.literal
        .as_deref()
        .filter(|name| !name.is_empty())
        .map(|name| format!("Asheron's Call 2 - {name}"))
}

fn show_world_name(window: &Window, info: &ac2_net::protocol::StringInfo) {
    match world_title(info) {
        Some(title) => window.set_title(&title),
        None => eprintln!("ac2: world name is not a literal; the title keeps its default"),
    }
}

/// The world names among a frame's session events (they arrive before character selection too).
pub(crate) fn show_world_names(window: &Window, events: &[SessionEvent]) {
    for event in events {
        if let SessionEvent::WorldName(info) = event {
            show_world_name(window, info);
        }
    }
}

/// The speakers, or silence when there is no output device.
pub(crate) fn speakers() -> Box<dyn ac2_audio::SoundSink> {
    match ac2_audio::SpeakerSink::open() {
        Ok(sink) => Box::new(sink),
        Err(error) => {
            eprintln!("no sound: {error}");
            Box::new(ac2_audio::NullSink)
        }
    }
}

pub(crate) fn depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("ac2 depth"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

#[cfg(test)]
mod tests {
    use super::world_title;
    use ac2_net::protocol::StringInfo;

    #[test]
    fn the_title_carries_a_literal_world_name() {
        assert_eq!(
            world_title(&StringInfo::literal("Darktide")),
            Some("Asheron's Call 2 - Darktide".to_owned())
        );
        assert_eq!(world_title(&StringInfo::literal("")), None);
        assert_eq!(world_title(&StringInfo::default()), None);
    }
}
