//! Bakes the AC2Hero mirror into the binary: `AC2HERO_DIR` names the mirror's
//! `ac2hero.e5i5o.com` folder; unset, the guide is empty.

#[allow(dead_code)]
#[path = "src/coord.rs"]
mod coord;
#[allow(dead_code)]
#[path = "src/facts.rs"]
mod facts;
#[allow(dead_code)]
#[path = "src/guide.rs"]
mod guide;
#[allow(dead_code)]
#[path = "src/html.rs"]
mod html;

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=AC2HERO_DIR");
    println!("cargo:rerun-if-changed=src/coord.rs");
    println!("cargo:rerun-if-changed=src/facts.rs");
    println!("cargo:rerun-if-changed=src/guide.rs");
    println!("cargo:rerun-if-changed=src/html.rs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("ac2hero.bin");
    let guide = match std::env::var_os("AC2HERO_DIR") {
        Some(dir) => {
            let dir = PathBuf::from(dir);
            println!("cargo:rerun-if-changed={}", dir.display());
            html::read_mirror(&dir).unwrap_or_else(|e| panic!("AC2HERO_DIR {}: {e}", dir.display()))
        }
        // Without a mirror the guide is empty (every crate that links it builds as usual).
        None => guide::Guide::default(),
    };
    std::fs::write(&out, guide::encode(&guide)).expect("write the baked guide");
}
