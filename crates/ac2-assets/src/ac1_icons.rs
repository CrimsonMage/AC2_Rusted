//! Asheron's Call 1's toolbar images, read from the player's own AC1 DATs (`AC1_DAT_DIR`) by the
//! tool that makes the modern HUD's panel icons (`ac2-ui`'s `bake_panel_icons` example; AGENTS.md:
//! the HUD's two styles). AC1's DATs share the retail container
//! (`ac2_dat`'s `Retail` layout), and its toolbar buttons' images are `RenderSurface`s in 24-bit
//! B, G, R order (pixel format `0x14`, `PFID_R8G8B8`).

use std::path::{Path, PathBuf};

/// The AC1 toolbar's Social button (`0x10000197`): its normal and highlight images.
pub const AC1_SOCIAL: [u32; 2] = [0x0600_111F, 0x0600_1121];
/// The AC1 toolbar's Skills and attributes button (`0x10000199`), the character sheet.
pub const AC1_SKILLS: [u32; 2] = [0x0600_1122, 0x0600_1124];

/// `PFID_R8G8B8` (B, G, R bytes) and `PFID_A8R8G8B8` (B, G, R, A).
const PFID_R8G8B8: u32 = 0x14;
const PFID_A8R8G8B8: u32 = 0x15;

/// One decoded AC1 image (RGBA8, rows top-down).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac1Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// The two buttons' images, normal then highlight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac1ToolbarIcons {
    pub social: [Ac1Image; 2],
    pub skills: [Ac1Image; 2],
}

/// The AC1 DAT directory the player names (`AC1_DAT_DIR`), if any.
pub fn ac1_dat_dir() -> Option<PathBuf> {
    std::env::var_os("AC1_DAT_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.join("client_portal.dat").is_file())
}

/// A `RenderSurface` record's image: its id, a word, the width, the height, the pixel format and
/// the byte count, then the pixels. `None` for a format other than 24- or 32-bit colour.
pub fn decode_ac1_surface(bytes: &[u8]) -> Option<Ac1Image> {
    let word = |at: usize| Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?));
    let (width, height, format, length) = (word(8)?, word(12)?, word(16)?, word(20)? as usize);
    let pixels = bytes.get(24..24 + length)?;
    let stride = match format {
        PFID_R8G8B8 => 3,
        PFID_A8R8G8B8 => 4,
        _ => return None,
    };
    if pixels.len() < width as usize * height as usize * stride {
        return None;
    }
    let rgba = pixels
        .chunks_exact(stride)
        .take(width as usize * height as usize)
        .flat_map(|p| [p[2], p[1], p[0], if stride == 4 { p[3] } else { 255 }])
        .collect();
    Some(Ac1Image {
        width,
        height,
        rgba,
    })
}

/// The AC1 images `dids` from the AC1 DATs in `dir`, in order.
pub fn load_ac1_images(dir: &Path, dids: &[u32]) -> Result<Vec<Ac1Image>, String> {
    let mut options = ac2_dat::DatOpenOptions::new(dir);
    options.portal_dat_file_name = "client_portal.dat".into();
    options.cell_dat_file_name = "client_cell_1.dat".into();
    options.local_dat_file_name = "client_local_English.dat".into();
    let dataset = ac2_dat::DatDataset::open(options).map_err(|e| e.to_string())?;
    dids.iter()
        .map(|&did| {
            let bytes = dataset
                .read_packed(ac2_dat::DatFileKind::Portal, did)
                .map_err(|e| e.to_string())?
                .ok_or(format!("AC1 image {did:08X} is not in the DAT"))?;
            decode_ac1_surface(&bytes).ok_or(format!(
                "AC1 image {did:08X}: unsupported format {:?}",
                bytes.get(16..20)
            ))
        })
        .collect()
}

/// The Social and Skills icons from the AC1 DATs in `dir`.
pub fn load_ac1_toolbar_icons(dir: &Path) -> Result<Ac1ToolbarIcons, String> {
    let mut options = ac2_dat::DatOpenOptions::new(dir);
    options.portal_dat_file_name = "client_portal.dat".into();
    options.cell_dat_file_name = "client_cell_1.dat".into();
    options.local_dat_file_name = "client_local_English.dat".into();
    let dataset = ac2_dat::DatDataset::open(options).map_err(|e| e.to_string())?;
    let image = |did: u32| -> Result<Ac1Image, String> {
        let bytes = dataset
            .read_packed(ac2_dat::DatFileKind::Portal, did)
            .map_err(|e| e.to_string())?
            .ok_or(format!("AC1 image {did:08X} is not in the DAT"))?;
        decode_ac1_surface(&bytes).ok_or(format!("AC1 image {did:08X}: unsupported format"))
    };
    Ok(Ac1ToolbarIcons {
        social: [image(AC1_SOCIAL[0])?, image(AC1_SOCIAL[1])?],
        skills: [image(AC1_SKILLS[0])?, image(AC1_SKILLS[1])?],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_24_bit_surface_decodes_from_blue_green_red() {
        let mut bytes = Vec::new();
        for word in [0x0600_0001_u32, 6, 2, 1, PFID_R8G8B8, 6] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&[0x28, 0x42, 0x4D, 1, 2, 3]);
        let image = decode_ac1_surface(&bytes).unwrap();
        assert_eq!((image.width, image.height), (2, 1));
        assert_eq!(image.rgba, vec![0x4D, 0x42, 0x28, 255, 3, 2, 1, 255]);
    }

    #[test]
    fn other_formats_and_short_records_are_refused() {
        let mut bytes = Vec::new();
        for word in [0x0600_0001_u32, 6, 2, 1, 0x65, 2] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0]);
        assert_eq!(decode_ac1_surface(&bytes), None);
        assert_eq!(decode_ac1_surface(&bytes[..10]), None);
    }

    /// The player's AC1 DATs (`AC1_DAT_DIR`), when present: both buttons' images decode at the
    /// toolbar's size.
    #[test]
    fn the_ac1_toolbar_icons_load_from_the_players_dats() {
        let Some(dir) = ac1_dat_dir() else {
            return;
        };
        let icons = load_ac1_toolbar_icons(&dir).unwrap();
        assert_eq!((icons.social[0].width, icons.social[0].height), (35, 27));
        assert_eq!((icons.skills[1].width, icons.skills[1].height), (34, 27));
    }
}
