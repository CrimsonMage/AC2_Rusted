//! The screenshot readback and file (`Device::SaveScreenshot`): the presented surface is copied to
//! the CPU before it is presented, the frame is numbered (`capN.jpg`, the first N not yet taken in
//! the AC2 directory) and encoded as JPEG (`RenderSurface::SaveJPG`) by the `image` crate. The
//! read-back stalls the frame; the encode and the file write run on a worker thread, so the frame
//! thread never writes a file. The key that asks for one is the shell's (`bin/ac2/keys.rs`).

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

use crate::Gpu;
use image::codecs::jpeg::JpegEncoder;
use image::ExtendedColorType;

/// `Device::SaveScreenshot`'s bound on the numbering scan.
const MAX_NUMBER: u32 = 100_000;

/// The JPEG quality. Retail's `SaveJPG` sets its quality inside an opaque `jprops` block that the
/// decompile does not expose, so this is a fixed choice.
const JPEG_QUALITY: u8 = 90;

/// The file name of screenshot `number` (`cap%d`, with retail's `.jpg`).
pub fn file_name(number: u32) -> String {
    format!("cap{number}.jpg")
}

/// A frame as JPEG bytes (8-bit RGBA, `width` by `height`, rows tightly packed; alpha is dropped).
pub fn encode_jpeg(width: u32, height: u32, rgba: &[u8]) -> io::Result<Vec<u8>> {
    let expected = width as usize * height as usize * 4;
    if rgba.len() != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "frame is {} bytes, {width}x{height} needs {expected}",
                rgba.len()
            ),
        ));
    }
    let rgb: Vec<u8> = rgba
        .chunks_exact(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY)
        .encode(&rgb, width, height, ExtendedColorType::Rgb8)
        .map_err(io::Error::other)?;
    Ok(out)
}

/// Writes the frame into `dir` as the first `capN.jpg` not yet there. `create_new` claims the name,
/// so two screenshots in a row never pick the same file; the directory is created when missing.
pub fn save_frame(dir: &Path, width: u32, height: u32, rgba: &[u8]) -> io::Result<PathBuf> {
    let bytes = encode_jpeg(width, height, rgba)?;
    std::fs::create_dir_all(dir)?;
    for number in 0..MAX_NUMBER {
        let path = dir.join(file_name(number));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(&bytes)?;
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("no free screenshot name"))
}
/// The surface's pixels as tightly packed RGBA8: the copy's rows are padded to the copy alignment,
/// and a BGRA surface swaps its red and blue.
pub fn unpad_rgba(bytes: &[u8], width: u32, height: u32, padded_row: usize, bgra: bool) -> Vec<u8> {
    let row = width as usize * 4;
    let mut out = Vec::with_capacity(row * height as usize);
    for y in 0..height as usize {
        let start = y * padded_row;
        let Some(source) = bytes.get(start..start + row) else {
            break;
        };
        if bgra {
            out.extend(
                source
                    .chunks_exact(4)
                    .flat_map(|p| [p[2], p[1], p[0], p[3]]),
            );
        } else {
            out.extend_from_slice(source);
        }
    }
    out
}

/// Whether `format` is one of the 8-bit colour layouts the read-back understands, and whether it
/// is BGRA.
fn layout(format: wgpu::TextureFormat) -> Option<bool> {
    use wgpu::TextureFormat::*;
    match format {
        Rgba8Unorm | Rgba8UnormSrgb => Some(false),
        Bgra8Unorm | Bgra8UnormSrgb => Some(true),
        _ => None,
    }
}

/// Copies the presented surface to the CPU as RGBA8 rows (`Device::SaveScreenshot`'s frame).
/// The surface must carry `COPY_SRC`.
pub fn read_surface(
    gpu: &Gpu,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let bgra = layout(format).ok_or_else(|| format!("surface format {format:?} is not 8-bit"))?;
    let padded = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ac2 screenshot read-back"),
        size: u64::from(padded) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    receiver
        .recv()
        .map_err(|_| "read-back callback lost".to_owned())?
        .map_err(|error| format!("read-back buffer does not map: {error:?}"))?;
    let mapped = buffer
        .slice(..)
        .get_mapped_range()
        .map_err(|error| format!("read-back buffer is not mapped: {error:?}"))?;
    Ok(unpad_rgba(&mapped, width, height, padded as usize, bgra))
}

/// The worker results that have finished, in the order they were queued; a finished or lost
/// worker leaves the queue.
pub fn finished(pending: &mut Vec<Receiver<io::Result<PathBuf>>>) -> Vec<io::Result<PathBuf>> {
    let mut done = Vec::new();
    pending.retain(|receiver| match receiver.try_recv() {
        Ok(result) => {
            done.push(result);
            false
        }
        Err(TryRecvError::Empty) => true,
        Err(TryRecvError::Disconnected) => false,
    });
    done
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the system temp directory (tests only; the shipped client writes to
    /// the AC2 directory).
    fn scratch_dir(tag: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!(
            "ac2-screenshot-{tag}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    #[test]
    fn names_follow_retail_numbering() {
        assert_eq!(file_name(0), "cap0.jpg");
        assert_eq!(file_name(17), "cap17.jpg");
    }

    #[test]
    fn save_takes_the_first_free_number() {
        let dir = scratch_dir("naming");
        std::fs::write(dir.join("cap0.jpg"), b"taken").unwrap();
        std::fs::write(dir.join("cap1.jpg"), b"taken").unwrap();
        let rgba = vec![9u8; 2 * 2 * 4];
        let first = save_frame(&dir, 2, 2, &rgba).expect("first save");
        let second = save_frame(&dir, 2, 2, &rgba).expect("second save");
        assert_eq!(first, dir.join("cap2.jpg"));
        assert_eq!(second, dir.join("cap3.jpg"));
        // The taken files are not overwritten.
        assert_eq!(std::fs::read(dir.join("cap0.jpg")).unwrap(), b"taken");
        let bytes = std::fs::read(&first).unwrap();
        assert_eq!(&bytes[..3], &[0xFF, 0xD8, 0xFF]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn save_creates_the_directory() {
        let dir = scratch_dir("create").join("AC2");
        let path = save_frame(&dir, 1, 1, &[0, 0, 0, 255]).expect("save");
        assert_eq!(path, dir.join("cap0.jpg"));
        assert!(path.is_file());
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    /// The JPEG decodes to the frame's size, and its colours are the frame's within the lossy
    /// tolerance of the codec (a flat colour survives with only small error).
    #[test]
    fn jpeg_round_trips_size_and_colour() {
        let (width, height) = (64u32, 48u32);
        let rgba: Vec<u8> = (0..width * height)
            .flat_map(|_| [200u8, 100, 50, 255])
            .collect();
        let bytes = encode_jpeg(width, height, &rgba).expect("encode");
        let decoded = image::load_from_memory_with_format(&bytes, image::ImageFormat::Jpeg)
            .expect("decode")
            .to_rgb8();
        assert_eq!(decoded.dimensions(), (width, height));
        for pixel in decoded.pixels() {
            for (got, want) in pixel.0.iter().zip([200i32, 100, 50]) {
                assert!((i32::from(*got) - want).abs() <= 8, "{pixel:?}");
            }
        }
    }

    #[test]
    fn encode_rejects_a_short_frame() {
        assert!(encode_jpeg(2, 2, &[0; 15]).is_err());
    }

    #[test]
    fn unpad_strips_rows_and_swaps_bgra() {
        // Two pixels per row, rows padded to 16 bytes, two rows.
        let mut bytes = vec![0xEE; 32];
        bytes[0..8].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        bytes[16..24].copy_from_slice(&[9, 10, 11, 12, 13, 14, 15, 16]);
        assert_eq!(
            unpad_rgba(&bytes, 2, 2, 16, false),
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
        assert_eq!(
            unpad_rgba(&bytes, 2, 2, 16, true),
            [3, 2, 1, 4, 7, 6, 5, 8, 11, 10, 9, 12, 15, 14, 13, 16]
        );
    }

    #[test]
    fn only_8_bit_colour_surfaces_are_read() {
        assert_eq!(layout(wgpu::TextureFormat::Bgra8Unorm), Some(true));
        assert_eq!(layout(wgpu::TextureFormat::Rgba8UnormSrgb), Some(false));
        assert_eq!(layout(wgpu::TextureFormat::Rgba16Float), None);
    }

    #[test]
    fn finished_drains_only_completed_saves() {
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let (_idle_tx, idle_rx) = std::sync::mpsc::channel::<io::Result<PathBuf>>();
        done_tx.send(Ok(PathBuf::from("a.jpg"))).unwrap();
        let mut pending = vec![done_rx, idle_rx];
        let results = finished(&mut pending);
        assert_eq!(results.len(), 1);
        assert_eq!(pending.len(), 1);
    }
}
