//! Makes the modern HUD's panel button icons (`crates/ac2-ui/assets/hud/`) from the player's DATs:
//! AC2's Skill orb and AC1's toolbar symbols, drawn by `ac2_ui::ac1_buttons`.
//!
//! `cargo run -p ac2-ui --example bake_panel_icons -- AC2_DAT_DIR AC1_DAT_DIR [OUT_DIR]`

use std::path::PathBuf;
use std::sync::Arc;

use ac2_ui::ac1_buttons::{self, Rgba};

fn write_png(path: &PathBuf, image: &Rgba) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(&image.rgba))
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let usage = "bake_panel_icons AC2_DAT_DIR AC1_DAT_DIR [OUT_DIR]";
    let ac2 = PathBuf::from(args.next().ok_or(usage)?);
    let ac1 = PathBuf::from(args.next().ok_or(usage)?);
    let out = args.next().map_or_else(
        || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/hud"),
        PathBuf::from,
    );
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let dataset =
        ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(&ac2)).map_err(|e| e.to_string())?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let base = ac2_ui::load_images(&store, &[ac1_buttons::BASE_ORB])
        .map_err(|e| e.to_string())?
        .remove(&ac1_buttons::BASE_ORB)
        .ok_or("AC2's Skill orb is not in the DATs")?;
    let base = Rgba {
        width: base.width,
        height: base.height,
        rgba: base.rgba.to_vec(),
    };
    let icons = ac2_assets::ac1_icons::load_ac1_images(
        &ac1,
        &ac1_buttons::PANEL_SYMBOLS.map(|(_, did, _, _)| did),
    )?;
    for ((name, did, kind, area), symbol) in ac1_buttons::PANEL_SYMBOLS.iter().zip(icons) {
        let symbol = Rgba {
            width: symbol.width,
            height: symbol.height,
            rgba: symbol.rgba,
        };
        let symbol = match area {
            Some(area) => symbol.crop(*area),
            None => symbol,
        };
        let [normal, hover] = ac1_buttons::icon_file_names(name);
        write_png(
            &out.join(&normal),
            &ac1_buttons::icon(&base, &symbol, *kind, 1.0),
        )?;
        write_png(
            &out.join(&hover),
            &ac1_buttons::icon(&base, &symbol, *kind, ac1_buttons::ROLLOVER_LIGHT),
        )?;
        println!("{name}: AC1 {did:08X} -> {normal}, {hover}");
    }
    Ok(())
}
