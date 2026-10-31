//! Compare two DAT sets record by record: which records a later data release adds, removes or
//! changes, per DAT file and per record type (the DID's high byte).
//!
//! `dat-diff OLD_DIR NEW_DIR [--tsv OUT.tsv]`
//!
//! Each directory holds `portal.dat`, `cell_1.dat`, `local_English.dat` and optionally
//! `highres.dat`. Records are walked in the retail B-tree's order (`ac2-dat`) and compared by
//! their packed bytes; a record whose size and bytes match is unchanged, whatever its iteration.
//! The TSV lists every difference: file, DID, status, old/new packed size, old/new iteration.

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::Hasher;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ac2_dat::{DatDataset, DatEntryMeta, DatFileKind, DatOpenOptions};

#[derive(Default, Clone, Copy)]
struct Counts {
    added: u64,
    removed: u64,
    changed: u64,
    same: u64,
}

fn open(dir: &Path) -> Result<Arc<DatDataset>, String> {
    let mut options = DatOpenOptions::new(dir);
    options.high_res_enabled = dir.join(DatFileKind::HighRes.default_file_name()).is_file();
    DatDataset::open(options).map_err(|e| format!("{}: {e:?}", dir.display()))
}

/// The highest iteration in the file's DDD history (`0xFFFF0001`), as the client reports it.
fn latest_iteration(dataset: &DatDataset, kind: DatFileKind) -> Option<i32> {
    let list = dataset.ddd_iteration(kind).ok()??;
    list.pairs
        .iter()
        .map(|p| p.starting_iteration + p.consecutive_iterations.abs().max(1) - 1)
        .max()
}

fn digest(dataset: &DatDataset, kind: DatFileKind, did: u32) -> Option<u64> {
    let bytes = dataset.read_packed(kind, did).ok()??;
    let mut hasher = DefaultHasher::new();
    hasher.write(&bytes);
    Some(hasher.finish())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (old_dir, new_dir) = match (args.first(), args.get(1)) {
        (Some(a), Some(b)) => (PathBuf::from(a), PathBuf::from(b)),
        _ => {
            eprintln!("usage: dat-diff OLD_DIR NEW_DIR [--tsv OUT.tsv]");
            std::process::exit(2);
        }
    };
    let tsv_path = args.iter().position(|a| a == "--tsv").and_then(|i| args.get(i + 1));
    let (old, new) = match (open(&old_dir), open(&new_dir)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let mut tsv = tsv_path.map(|p| {
        let mut f = std::io::BufWriter::new(std::fs::File::create(p).expect("create tsv"));
        writeln!(f, "file\tdid\tstatus\told_size\tnew_size\told_iteration\tnew_iteration").unwrap();
        f
    });
    for kind in DatFileKind::ALL {
        if old.file_meta(kind).is_none() || new.file_meta(kind).is_none() {
            println!("{}: not in both sets, skipped", kind.default_file_name());
            continue;
        }
        let a: BTreeMap<u32, DatEntryMeta> = old.iter_entries(kind).map(|e| (e.did, e)).collect();
        let b: BTreeMap<u32, DatEntryMeta> = new.iter_entries(kind).map(|e| (e.did, e)).collect();
        let mut total = Counts::default();
        let mut by_type: BTreeMap<u8, Counts> = BTreeMap::new();
        let mut record = |did: u32, status: &str, x: Option<&DatEntryMeta>, y: Option<&DatEntryMeta>| {
            let counts = by_type.entry((did >> 24) as u8).or_default();
            for c in [&mut total, counts] {
                match status {
                    "added" => c.added += 1,
                    "removed" => c.removed += 1,
                    "changed" => c.changed += 1,
                    _ => c.same += 1,
                }
            }
            if status != "same" {
                if let Some(f) = tsv.as_mut() {
                    let size = |m: Option<&DatEntryMeta>| m.map_or(String::from("-"), |m| m.packed_len.to_string());
                    let iter = |m: Option<&DatEntryMeta>| m.map_or(String::from("-"), |m| m.iteration.to_string());
                    writeln!(f, "{}\t{did:08x}\t{status}\t{}\t{}\t{}\t{}", kind.default_file_name(),
                        size(x), size(y), iter(x), iter(y)).unwrap();
                }
            }
        };
        for (did, x) in &a {
            match b.get(did) {
                None => record(*did, "removed", Some(x), None),
                Some(y) => {
                    let same = x.packed_len == y.packed_len
                        && digest(&old, kind, *did) == digest(&new, kind, *did);
                    record(*did, if same { "same" } else { "changed" }, Some(x), Some(y));
                }
            }
        }
        for (did, y) in &b {
            if !a.contains_key(did) {
                record(*did, "added", None, Some(y));
            }
        }
        let iteration = |d: &DatDataset| latest_iteration(d, kind).map_or("?".into(), |i| i.to_string());
        println!(
            "{}: iteration {} -> {}; {} -> {} records; +{} added, -{} removed, ~{} changed, {} unchanged",
            kind.default_file_name(), iteration(&old), iteration(&new), a.len(), b.len(),
            total.added, total.removed, total.changed, total.same
        );
        let mut rows: Vec<_> = by_type.into_iter().filter(|(_, c)| c.added + c.removed + c.changed > 0).collect();
        rows.sort_by_key(|(_, c)| std::cmp::Reverse(c.added + c.removed + c.changed));
        for (ty, c) in rows {
            println!("    type 0x{ty:02x}xxxxxx: +{} -{} ~{} ={}", c.added, c.removed, c.changed, c.same);
        }
    }
}
