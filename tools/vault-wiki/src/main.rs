//! The In-Game Wiki as a standalone application: the AC2 Vault guide in the client's own guide
//! window, in the modern (custom) or classic (default) UI, with nothing else around it.
//!
//! ```text
//! vault-wiki [--dat-dir DIR] [--style modern|classic] [--size WIDTHxHEIGHT] [--page SLUG]
//! vault-wiki ... --snapshot OUT.png   (one frame to a PNG, no window)
//! ```
//! `--dat-dir` defaults to `AC2_DAT_DIR`. The guide is the one `ac2-guide` baked at build time
//! (`AC2VAULT_DIR`). F7 switches the look and keeps the page shown; Escape or the window's close
//! button quits. Share copies the page's `Title [wiki:slug]` to the clipboard; a press on a map
//! coordinate copies it and prints `point-at <north> <east>` on stdout (the line a world builder
//! can follow later).

mod app;
mod wiki;

use std::path::PathBuf;

use ac2_ui::HudStyle;

pub struct Args {
    pub dat_dir: PathBuf,
    pub style: HudStyle,
    pub size: (u32, u32),
    pub page: Option<String>,
    pub snapshot: Option<PathBuf>,
}

fn usage() -> ! {
    eprintln!(
        "usage: vault-wiki [--dat-dir DIR] [--style modern|classic] [--size WIDTHxHEIGHT] \
         [--page SLUG] [--snapshot OUT.png]"
    );
    std::process::exit(2);
}

fn parse_args() -> Args {
    let mut dat_dir = std::env::var_os("AC2_DAT_DIR").map(PathBuf::from);
    let mut style = HudStyle::Modern;
    let mut size = (1100, 760);
    let mut page = None;
    let mut snapshot = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| usage());
        match arg.as_str() {
            "--dat-dir" => dat_dir = Some(PathBuf::from(value())),
            "--style" => style = HudStyle::parse(&value()).unwrap_or_else(|| usage()),
            "--size" => {
                let v = value();
                let (w, h) = v.split_once('x').unwrap_or_else(|| usage());
                size = (
                    w.parse().unwrap_or_else(|_| usage()),
                    h.parse().unwrap_or_else(|_| usage()),
                );
            }
            "--page" => page = Some(value()),
            "--snapshot" => snapshot = Some(PathBuf::from(value())),
            _ => usage(),
        }
    }
    let Some(dat_dir) = dat_dir else {
        eprintln!("vault-wiki: no DATs: pass --dat-dir DIR or set AC2_DAT_DIR");
        std::process::exit(2);
    };
    Args {
        dat_dir,
        style,
        size: (size.0.max(640), size.1.max(480)),
        page,
        snapshot,
    }
}

fn main() {
    let args = parse_args();
    if ac2_guide::baked().pages.is_empty() {
        eprintln!(
            "vault-wiki: this build has no guide: build with \
             AC2VAULT_DIR=../third_party/ac2vault-merged"
        );
        std::process::exit(1);
    }
    let dataset =
        ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::for_retail_install(&args.dat_dir))
            .unwrap_or_else(|e| {
                eprintln!("vault-wiki: {}: {e}", args.dat_dir.display());
                std::process::exit(1);
            });
    let store = std::sync::Arc::new(ac2_db::ResourceStore::new(dataset));
    if let Some(out) = &args.snapshot {
        if let Err(e) = snapshot(&store, &args, out) {
            eprintln!("vault-wiki: {e}");
            std::process::exit(1);
        }
        return;
    }
    let event_loop = winit::event_loop::EventLoop::new().expect("event loop");
    let mut app = app::App::new(store, args);
    event_loop.run_app(&mut app).expect("event loop");
}

/// The window's frame at `args.size` after the guide has loaded (and opened `--page`), the
/// surfaces laid over the background as the overlay draws them, written to `out`.
fn snapshot(
    store: &std::sync::Arc<ac2_db::ResourceStore>,
    args: &Args,
    out: &std::path::Path,
) -> Result<(), String> {
    let (w, h) = args.size;
    let mut wiki = wiki::Wiki::build(store, args.style, w, h, args.page.as_deref())?;
    let modifiers = ac2_ui::Modifiers::default();
    // The guide loads on a worker and the pictures decode on others.
    for i in 0..300 {
        wiki.frame(&[], &[], &[], modifiers, f64::from(i) * 0.02);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let surfaces = wiki.compose().unwrap_or_default();
    let mut rgba = [5u8, 8, 10, 255].repeat((w * h) as usize);
    for s in surfaces.iter().filter(|s| s.shown) {
        for y in 0..s.height as i32 {
            for x in 0..s.width as i32 {
                let (dx, dy) = (s.x + x, s.y + y);
                if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
                    continue;
                }
                let src = ((y as u32 * s.width + x as u32) * 4) as usize;
                let dst = ((dy as u32 * w + dx as u32) * 4) as usize;
                let a = f32::from(s.rgba[src + 3]) / 255.0 * s.opacity;
                for c in 0..3 {
                    let v = f32::from(s.rgba[src + c]) * a + f32::from(rgba[dst + c]) * (1.0 - a);
                    rgba[dst + c] = v.round() as u8;
                }
            }
        }
    }
    let file = std::fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&rgba))
        .map_err(|e| format!("{}: {e}", out.display()))
}
