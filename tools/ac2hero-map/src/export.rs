//! The map as files another tool can draw (the AC2e content editor): the world overview, every
//! land block's close-up tile, and the game's map notes with their icons.
//!
//! `DIR/manifest.json` describes the layout; `world.png` is the overview, one pixel per land cell
//! (north up); `blocks/XXYY.png` is block (XX, YY) at `DETAIL_SIDE` pixels, written only for
//! blocks with land; `notes.json` lists the notes in map units and `icons/DID.png` their icons.

use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};

use ac2_world::map_raster::{self as terrain, BLOCKS, DETAIL_SIDE, OCEAN, WATER, WORLD_CELLS};
use ac2_world::Ac2WorldSource;

use crate::notes;

/// Land cells per landblock side.
const CELLS: usize = 8;

fn save(path: &Path, width: usize, height: usize, rgba: Vec<u8>) -> Result<(), String> {
    image::RgbaImage::from_raw(width as u32, height as u32, rgba)
        .ok_or("the pixels do not fit the image size")?
        .save(path)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn json_text(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Blocks whose overview pixels are not all open sea (missing blocks or open water).
fn land_blocks(raster: &[u8]) -> Vec<(u32, u32)> {
    let mut blocks = Vec::new();
    for bx in 0..BLOCKS as usize {
        for by in 0..BLOCKS as usize {
            let land = (0..CELLS * CELLS).any(|i| {
                let (cx, cy) = (bx * CELLS + i % CELLS, by * CELLS + i / CELLS);
                let at = ((WORLD_CELLS - 1 - cy) * WORLD_CELLS + cx) * 4;
                raster[at..at + 3] != OCEAN && raster[at..at + 3] != WATER
            });
            if land {
                blocks.push((bx as u32, by as u32));
            }
        }
    }
    blocks
}

/// Writes the export to `dir` with `threads` workers; returns the number of block tiles.
pub fn export(world: &Ac2WorldSource, dir: &Path, threads: usize) -> Result<usize, String> {
    let threads = threads.max(1);
    std::fs::create_dir_all(dir.join("blocks")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir.join("icons")).map_err(|e| e.to_string())?;
    let raster = terrain::world_raster(world, threads);
    let blocks = land_blocks(&raster);
    save(&dir.join("world.png"), WORLD_CELLS, WORLD_CELLS, raster)?;

    let next = AtomicU32::new(0);
    let written = AtomicU32::new(0);
    let failure = std::sync::Mutex::new(None);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed) as usize;
                let Some(&(bx, by)) = blocks.get(index) else {
                    break;
                };
                let Some(tile) = terrain::detail_tile(world, bx, by) else {
                    continue;
                };
                let path = dir.join("blocks").join(format!("{bx:02x}{by:02x}.png"));
                match save(&path, DETAIL_SIDE, DETAIL_SIDE, tile) {
                    Ok(()) => {
                        written.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        *failure.lock().unwrap_or_else(|p| p.into_inner()) = Some(e);
                        break;
                    }
                }
            });
        }
    });
    if let Some(e) = failure.into_inner().unwrap_or_else(|p| p.into_inner()) {
        return Err(e);
    }

    let (notes, icons) = notes::load(world.resource_store());
    let rows: Vec<String> = notes
        .iter()
        .map(|n| {
            format!(
                "{{\"name\":{},\"kind\":{},\"north\":{},\"east\":{},\"icon\":\"{:08x}\"}}",
                json_text(&n.name),
                json_text(n.kind.label()),
                n.at.north,
                n.at.east,
                n.icon
            )
        })
        .collect();
    std::fs::write(
        dir.join("notes.json"),
        format!("[\n{}\n]\n", rows.join(",\n")),
    )
    .map_err(|e| e.to_string())?;
    for (did, (width, height, rgba)) in icons {
        save(
            &dir.join("icons").join(format!("{did:08x}.png")),
            width as usize,
            height as usize,
            rgba,
        )?;
    }
    let count = written.into_inner() as usize;
    let manifest = format!(
        "{{\"format\":\"ac2hero-map-export-v1\",\"world_cells\":{WORLD_CELLS},\"block_cells\":{CELLS},\
\"cell_metres\":20,\"cells_per_unit\":10.2,\"block_pixels\":{DETAIL_SIDE},\"blocks\":{count},\"notes\":{}}}\n",
        notes.len()
    );
    std::fs::write(dir.join("manifest.json"), manifest).map_err(|e| e.to_string())?;
    Ok(count)
}
