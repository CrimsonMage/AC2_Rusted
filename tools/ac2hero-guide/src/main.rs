//! The quest guide as a standalone egui tool: browse the baked AC2Hero pages, click a map
//! coordinate to make it the target, and see the arrow and route the client will draw.
//!
//! `ac2hero-guide [--mirror DIR] [--page TITLE] [--stats]`: `--mirror` reads a mirror at run time
//! instead of the baked guide (for checking a parser change without a rebuild); `--page` opens
//! the first page whose title contains TITLE; `--stats` prints counts and exits.

mod app;
mod map_view;
mod navigator;
mod page_view;

use ac2_guide::{html, Guide, Kind};

fn main() -> eframe::Result {
    let mut mirror = None;
    let mut stats = false;
    let mut page = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mirror" => mirror = args.next(),
            "--stats" => stats = true,
            "--page" => page = args.next(),
            _ => {
                eprintln!("usage: ac2hero-guide [--mirror DIR] [--page TITLE] [--stats]");
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
    if stats {
        print_stats(&guide);
        return Ok(());
    }
    if guide.pages.is_empty() {
        eprintln!("the guide is empty: build with AC2HERO_DIR set, or pass --mirror DIR");
    }
    let options = light_options("AC2 quest guide", [1280.0, 800.0]);
    eframe::run_native(
        "ac2hero-guide",
        options,
        Box::new(|cc| {
            let mut app = app::App::new(&cc.egui_ctx, guide);
            if let Some(title) = page {
                app.open_titled(&title);
            }
            Ok(Box::new(app))
        }),
    )
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

fn print_stats(guide: &Guide) {
    for kind in Kind::ALL {
        let pages: Vec<_> = guide.pages.iter().filter(|p| p.kind == kind).collect();
        let with = pages.iter().filter(|p| p.coords().next().is_some()).count();
        let coords: usize = pages.iter().map(|p| p.coords().count()).sum();
        println!(
            "{:9} {:4} pages, {:4} with coordinates, {:5} coordinates",
            kind.label(),
            pages.len(),
            with,
            coords
        );
    }
    let (mut linked, mut missing) = (0, 0);
    for span in guide
        .pages
        .iter()
        .flat_map(|p| &p.lines)
        .flat_map(|l| &l.spans)
    {
        match span {
            ac2_guide::Span::Page { .. } => linked += 1,
            ac2_guide::Span::Missing { .. } => missing += 1,
            _ => {}
        }
    }
    println!("links: {linked} to guide pages, {missing} elsewhere");
    let quests: Vec<_> = guide
        .pages
        .iter()
        .filter_map(|p| p.quest.as_ref())
        .collect();
    let levelled = quests.iter().filter(|q| q.level.is_some()).count();
    let rewards: usize = quests.iter().map(|q| q.rewards.len()).sum();
    let matched = quests
        .iter()
        .flat_map(|q| &q.rewards)
        .filter(|r| r.item.is_some())
        .count();
    println!(
        "quests: {levelled}/{} with a level, {rewards} rewards ({matched} matched to items)",
        quests.len()
    );
    let image_bytes: usize = guide.images.iter().map(|i| i.bytes.len()).sum();
    println!(
        "items: {}, images: {} ({:.1} MB)",
        guide.items.len(),
        guide.images.len(),
        image_bytes as f64 / 1048576.0
    );
    let undecodable: Vec<&str> = guide
        .images
        .iter()
        .filter(|i| image::load_from_memory(&i.bytes).is_err())
        .map(|i| i.path.as_str())
        .collect();
    println!(
        "images that do not decode: {} {:?}",
        undecodable.len(),
        &undecodable[..undecodable.len().min(5)]
    );
    let markers = ac2_guide::atlas::markers(guide);
    for kind in ac2_guide::atlas::MarkerKind::ALL {
        let n = markers.iter().filter(|m| m.kind == kind).count();
        println!("map markers: {:13} {n}", kind.label());
    }
    println!(
        "baked size: {} bytes",
        ac2_guide::guide::encode(guide).len()
    );
}
