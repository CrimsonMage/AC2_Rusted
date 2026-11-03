//! Bakes the AC2 Vault mirror into the binary: `AC2VAULT_DIR` names the merged
//! `ac2vault.ign.com` tree (`third_party/ac2vault-merged` in the analysis workspace); unset, the
//! guide is empty.

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
#[allow(dead_code)]
#[path = "src/vault.rs"]
mod vault;

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-env-changed=AC2VAULT_DIR");
    println!("cargo:rerun-if-changed=src/coord.rs");
    println!("cargo:rerun-if-changed=src/facts.rs");
    println!("cargo:rerun-if-changed=src/guide.rs");
    println!("cargo:rerun-if-changed=src/html.rs");
    println!("cargo:rerun-if-changed=src/vault.rs");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("guide.bin");
    let guide = match std::env::var_os("AC2VAULT_DIR") {
        Some(dir) => {
            let dir = PathBuf::from(dir);
            println!("cargo:rerun-if-changed={}", dir.display());
            vault::read_vault(&dir)
                .unwrap_or_else(|e| panic!("AC2VAULT_DIR {}: {e}", dir.display()))
        }
        // Without a mirror the guide is empty (every crate that links it builds as usual).
        None => guide::Guide::default(),
    };
    std::fs::write(&out, guide::encode(&guide)).expect("write the baked guide");
}
