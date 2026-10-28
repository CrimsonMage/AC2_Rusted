use super::*;

fn camera() -> Camera {
    Camera {
        eye: Vec3::new(100.0, 200.0, 50.0),
        target: Vec3::new(100.0, 201.0, 50.0),
        fov_y: 1.0,
        near: 0.5,
        far: 1000.0,
    }
}

#[test]
fn a_light_ahead_is_at_the_centre_wherever_the_camera_stands() {
    let (ndc, in_view) = light_ndc(camera().view_projection(1.5), Vec3::Y);
    assert!(in_view);
    assert!(ndc[0].abs() < 1e-5 && ndc[1].abs() < 1e-5, "{ndc:?}");
    // Up and to the right of the view stays up and right on the screen.
    let (ndc, in_view) = light_ndc(
        camera().view_projection(1.5),
        Vec3::new(0.1, 1.0, 0.1).normalize(),
    );
    assert!(in_view);
    assert!(ndc[0] > 0.0 && ndc[1] > 0.0, "{ndc:?}");
}

#[test]
fn a_light_behind_or_off_the_screen_is_out_of_view() {
    let view_projection = camera().view_projection(1.5);
    let (ndc, in_view) = light_ndc(view_projection, -Vec3::Y);
    assert!(!in_view);
    assert!(ndc.iter().all(|c| c.is_finite()));
    assert!(!light_ndc(view_projection, Vec3::X).1);
}

/// A depth target cleared to `depth` (1: the cleared far depth, the sky; less: something hides
/// the light).
fn depth_view(gpu: &Gpu, depth: f32) -> wgpu::TextureView {
    let view = gpu
        .device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("lens flare test depth"),
            size: wgpu::Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("lens flare test depth"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(depth),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    gpu.queue.submit([encoder.finish()]);
    view
}

/// The first flare's opacity as the last scan left it.
fn opacity(gpu: &Gpu, flares: &LensFlares) -> f32 {
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("lens flare test read-back"),
        size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        flares.opacity[flares.current.get()].as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(1),
            },
        },
        wgpu::Extent3d {
            width: MAX_FLARES as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let mapped = buffer.slice(..).get_mapped_range().expect("mapped");
    half_to_f32(u16::from_le_bytes([mapped[0], mapped[1]]))
}

/// An IEEE half (the opacity texture's red) as f32.
fn half_to_f32(bits: u16) -> f32 {
    let exponent = i32::from((bits >> 10) & 0x1f);
    let mantissa = f32::from(bits & 0x3ff);
    let magnitude = match exponent {
        0 => mantissa * 2f32.powi(-24),
        _ => (1.0 + mantissa / 1024.0) * 2f32.powi(exponent - 15),
    };
    if bits & 0x8000 != 0 {
        -magnitude
    } else {
        magnitude
    }
}

#[test]
fn the_opacity_fades_in_while_the_sky_shows_and_out_while_it_is_hidden() {
    let Ok(gpu) = Gpu::headless() else {
        return;
    };
    let mut flares = LensFlares::new(&gpu, crate::HDR_FORMAT);
    let sun = SkyFlare {
        direction: Vec3::Y,
        flare: ac2_world::LensFlare {
            fade_speed: 4.0,
            points: std::sync::Arc::from(vec![ac2_world::LensFlarePoint {
                axis_pos: -1.0,
                texture: None,
                size: 1.0,
            }]),
        },
    };
    let (sky, hidden) = (depth_view(&gpu, 1.0), depth_view(&gpu, 0.5));
    let step = |flares: &mut LensFlares, time: f64, depth: &wgpu::TextureView, light| {
        let lights = std::slice::from_ref(light);
        flares.prepare(&gpu, &camera(), 1.0, time, lights, &mut |_| None);
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        flares.scan(&gpu, &mut encoder, depth);
        gpu.queue.submit([encoder.finish()]);
        opacity(&gpu, flares)
    };
    // `m_fOpacity` moves by the elapsed time times the fade speed (4 per second), within 0..1.
    let mut seen = Vec::new();
    for (time, depth) in [
        (0.0, &sky),
        (0.1, &sky),
        (0.2, &sky),
        (0.3, &sky),
        (0.4, &hidden),
        (0.5, &hidden),
        (0.6, &hidden),
    ] {
        seen.push(step(&mut flares, time, depth, &sun));
    }
    let expected = [0.0, 0.4, 0.8, 1.0, 0.6, 0.2, 0.0];
    for (got, want) in seen.iter().zip(expected) {
        assert!((got - want).abs() < 2e-3, "{seen:?}");
    }
    // A light behind the camera is out of view: no scan sees it, and it fades.
    let behind = SkyFlare {
        direction: -Vec3::Y,
        ..sun.clone()
    };
    step(&mut flares, 1.0, &sky, &sun);
    let faded = step(&mut flares, 1.1, &sky, &behind);
    assert!((faded - 0.6).abs() < 2e-3, "{faded}");
}
