//! The site's pictures as pixels: a picture's size from its header (cheap, for laying a page out),
//! and the picture decoded to RGBA (on a worker; a quest map is a few hundred kilobytes).

use std::io::Cursor;

use image::ImageReader;

fn reader(bytes: &[u8]) -> Option<ImageReader<Cursor<&[u8]>>> {
    ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()
}

/// The picture's width and height, read from its header.
pub fn size(bytes: &[u8]) -> Option<(u32, u32)> {
    reader(bytes)?.into_dimensions().ok()
}

/// The picture as RGBA rows: width, height and pixels.
pub fn decode(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let image = reader(bytes)?.decode().ok()?.into_rgba8();
    let (w, h) = image.dimensions();
    Some((w, h, image.into_raw()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_png_is_sized_and_decoded() {
        // A 2 x 1 PNG, red then blue.
        let mut bytes = Vec::new();
        image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255])
            .unwrap()
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
        assert_eq!(super::size(&bytes), Some((2, 1)));
        let (w, h, rgba) = super::decode(&bytes).unwrap();
        assert_eq!((w, h), (2, 1));
        assert_eq!(&rgba[4..8], &[0, 0, 255, 255]);
        assert_eq!(super::size(b"not a picture"), None);
    }
}
