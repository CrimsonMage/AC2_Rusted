//! The quest guide's places on a zoomable world map: quest starts, quest steps, NPCs and guide
//! points as icons, grouped into counted clusters where they crowd, filtered by layer, level and
//! search, with the picked place's details beside the map.
//!
//! `ac2hero-map [--mirror DIR] [--dat-dir DIR] [--snapshot OUT.png]
//! [--screenshot OUT.png [--at COORD] [--zoom PX_PER_CELL]] [--size WxH] [--bare 1]
//! [--player "COORD@HEADING"]`: `--mirror` reads a mirror
//! at run time instead of the baked guide; `--dat-dir` (or `AC2_DAT_DIR`) draws the world from
//! its landscape behind the markers, adding detail as you zoom; `--snapshot` writes the whole
//! world with every marker dotted on it to a PNG and exits (an alignment check); `--screenshot`
//! opens the window at `--at` (a map coordinate) and `--zoom`, waits for the terrain, saves the
//! window to a PNG and exits; `--size` sets the window, `--bare 1` shows the map alone (the
//! panel-sized preview) and `--player` puts the player's arrow on it.

use ac2_guide::atlas::{self, Marker};
use ac2_guide::html;
use ac2_world::map_raster as terrain;
use ac2hero_map::WorldMap;
use eframe::egui;

fn main() -> eframe::Result {
    let (mut mirror, mut snapshot, mut screenshot, mut at, mut zoom) =
        (None, None, None, None, None);
    let (mut size, mut bare, mut player) = ([1400.0, 900.0], false, None);
    let mut dat_dir = std::env::var_os("AC2_DAT_DIR").map(std::path::PathBuf::from);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match (arg.as_str(), args.next()) {
            ("--mirror", Some(v)) => mirror = Some(v),
            ("--dat-dir", Some(v)) => dat_dir = Some(v.into()),
            ("--snapshot", Some(v)) => snapshot = Some(v),
            ("--screenshot", Some(v)) => screenshot = Some(std::path::PathBuf::from(v)),
            ("--at", Some(v)) => at = ac2_guide::coord::find(&v).first().map(|(_, c)| *c),
            ("--zoom", Some(v)) => zoom = v.parse::<f32>().ok(),
            ("--size", Some(v)) => {
                if let Some((w, h)) = v
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                {
                    size = [w, h];
                }
            }
            ("--bare", Some(v)) => bare = v == "1",
            ("--player", Some(v)) => {
                let heading = v
                    .rsplit_once('@')
                    .and_then(|(_, h)| h.parse().ok())
                    .unwrap_or(0.0);
                player = ac2_guide::coord::find(&v)
                    .first()
                    .map(|(_, c)| ac2hero_map::Player { at: *c, heading });
            }
            _ => {
                eprintln!("usage: ac2hero-map [--mirror DIR] [--dat-dir DIR] [--snapshot OUT.png]");
                std::process::exit(2);
            }
        }
    }
    let guide = match mirror {
        Some(dir) => html::read_mirror(dir.as_ref()).unwrap_or_else(|e| {
            eprintln!("{dir}: {e}");
            std::process::exit(1);
        }),
        None => ac2_guide::baked(),
    };
    let markers = atlas::markers(&guide);
    if markers.is_empty() {
        eprintln!("no places: build with AC2HERO_DIR set, or pass --mirror DIR");
    }
    let world = dat_dir
        .as_ref()
        .and_then(|dir| match ac2hero_map::open_world(dir) {
            Ok(world) => world,
            Err(e) => {
                eprintln!(
                    "the landscape could not be read from {}: {e}",
                    dir.display()
                );
                None
            }
        });
    if let Some(out) = snapshot {
        let Some(world) = world else {
            eprintln!("--snapshot needs the landscape: set AC2_DAT_DIR or pass --dat-dir");
            std::process::exit(2);
        };
        if let Err(e) = write_snapshot(&world, &markers, out.as_ref()) {
            eprintln!("{out}: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let options = light_options("AC2 world map", size);
    eframe::run_native(
        "ac2hero-map",
        options,
        Box::new(|cc| {
            let mut map = WorldMap::new(&cc.egui_ctx, guide, world);
            if let Some(at) = at {
                map.set_view(at, zoom.unwrap_or(1.0));
            }
            if bare {
                map.set_bare();
            }
            Ok(Box::new(MapApp {
                map,
                player,
                screenshot,
                asked: false,
                saved: Default::default(),
            }))
        }),
    )
}

/// The world raster with a dot for every marker, as an alignment check.
fn write_snapshot(
    world: &ac2_world::Ac2WorldSource,
    markers: &[Marker],
    out: &std::path::Path,
) -> Result<(), String> {
    let n = terrain::WORLD_CELLS as u32;
    let raster = terrain::world_raster(world, 4);
    let mut img =
        image::RgbaImage::from_raw(n, n, raster).ok_or("the raster does not fit its size")?;
    for m in markers {
        let x = (f64::from(m.at.east) + 100.0) * 10.2;
        let y = f64::from(n) - (f64::from(m.at.north) + 100.0) * 10.2;
        let c = ac2hero_map::colour(m.kind).to_array();
        for dy in -3..=3 {
            for dx in -3..=3 {
                let (px, py) = (x as i64 + dx, y as i64 + dy);
                if px >= 0 && py >= 0 && px < i64::from(n) && py < i64::from(n) {
                    img.put_pixel(px as u32, py as u32, image::Rgba(c));
                }
            }
        }
    }
    img.save(out).map_err(|e| e.to_string())
}

/// Window options that keep the GPU cost low: the integrated GPU unless `WGPU_POWER_PREF` says
/// otherwise (eframe asks for the discrete one), vsync, and egui's repaint-on-input loop.
fn light_options(title: &str, size: [f32; 2]) -> eframe::NativeOptions {
    let mut options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size(size),
        ..Default::default()
    };
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        setup.power_preference = eframe::wgpu::PowerPreference::from_env()
            .unwrap_or(eframe::wgpu::PowerPreference::LowPower);
    }
    options
}

/// The standalone window: the map filling it, and the screenshot it is to save.
struct MapApp {
    map: WorldMap,
    player: Option<ac2hero_map::Player>,
    screenshot: Option<std::path::PathBuf>,
    asked: bool,
    saved: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl eframe::App for MapApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::no_frame().show(ui, |ui| {
            self.map.ui(ui, self.player);
        });
        let Some(out) = self.screenshot.clone() else {
            return;
        };
        if self.saved.load(std::sync::atomic::Ordering::Relaxed) {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        } else if !self.asked && !self.map.busy() {
            self.asked = true;
            let (saved, ctx) = (std::sync::Arc::clone(&self.saved), ui.ctx().clone());
            ui.ctx().request_screenshot(move |image| {
                let [w, h] = image.size;
                let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                match image::RgbaImage::from_raw(w as u32, h as u32, rgba).map(|i| i.save(&out)) {
                    Some(Ok(())) => {}
                    Some(Err(e)) => eprintln!("{}: {e}", out.display()),
                    None => eprintln!("the screenshot's pixels do not fit its size"),
                }
                saved.store(true, std::sync::atomic::Ordering::Relaxed);
                ctx.request_repaint();
            });
        } else {
            ui.ctx().request_repaint();
        }
    }
}
