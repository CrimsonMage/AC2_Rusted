//! The content directory's resolution order, on temporary directories.

use std::ffi::OsStr;
use std::path::Path;

use super::{content_beside, resolve_content_dir};

#[test]
fn an_explicit_directory_beats_the_others() {
    let dat = tempfile::tempdir().unwrap();
    let explicit = tempfile::tempdir().unwrap();
    std::fs::create_dir(dat.path().join("Content")).unwrap();
    let env = explicit.path().as_os_str().to_owned();
    let found = resolve_content_dir(Some(explicit.path()), dat.path(), Some(&env));
    assert_eq!(found.as_deref(), Some(explicit.path()));
}

#[test]
fn a_missing_explicit_directory_falls_through() {
    let dat = tempfile::tempdir().unwrap();
    let content = dat.path().join("Content");
    std::fs::create_dir(&content).unwrap();
    let missing = dat.path().join("nowhere");
    let found = resolve_content_dir(Some(&missing), dat.path(), None);
    assert_eq!(found.as_deref(), Some(content.as_path()));
}

#[test]
fn the_folder_beside_the_dats_beats_the_environment() {
    let dat = tempfile::tempdir().unwrap();
    let env_dir = tempfile::tempdir().unwrap();
    let content = dat.path().join("Content");
    std::fs::create_dir(&content).unwrap();
    let env = env_dir.path().as_os_str().to_owned();
    let found = resolve_content_dir(None, dat.path(), Some(&env));
    assert_eq!(found.as_deref(), Some(content.as_path()));
}

#[test]
fn the_folder_beside_the_dats_matches_any_case() {
    let dat = tempfile::tempdir().unwrap();
    let lower = dat.path().join("content");
    std::fs::create_dir(&lower).unwrap();
    assert_eq!(content_beside(dat.path()).as_deref(), Some(lower.as_path()));
    // A file named like the folder is not a content directory.
    let dat = tempfile::tempdir().unwrap();
    std::fs::write(dat.path().join("Content"), b"not a folder").unwrap();
    assert_eq!(content_beside(dat.path()), None);
}

#[test]
fn the_environment_is_the_last_resort() {
    let dat = tempfile::tempdir().unwrap();
    let env_dir = tempfile::tempdir().unwrap();
    let env = env_dir.path().as_os_str().to_owned();
    let found = resolve_content_dir(None, dat.path(), Some(&env));
    assert_eq!(found.as_deref(), Some(env_dir.path()));
    // An empty or missing value is no directory.
    assert_eq!(
        resolve_content_dir(None, dat.path(), Some(OsStr::new(""))),
        None
    );
    let gone = dat.path().join("gone");
    let gone = gone.as_os_str().to_owned();
    assert_eq!(resolve_content_dir(None, dat.path(), Some(&gone)), None);
}

#[test]
fn nothing_found_resolves_to_none() {
    let dat = tempfile::tempdir().unwrap();
    assert_eq!(resolve_content_dir(None, dat.path(), None), None);
    // A DAT directory that does not exist has no folder beside it either.
    assert_eq!(resolve_content_dir(None, Path::new(""), None), None);
}
