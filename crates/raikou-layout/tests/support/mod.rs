#![allow(dead_code)]

use std::path::{Path, PathBuf};

use skia_safe::Surface;

#[allow(deprecated)]
pub fn snapshot_png_bytes(surface: &mut Surface) -> Vec<u8> {
    surface
        .image_snapshot()
        .encode_to_data(skia_safe::EncodedImageFormat::PNG)
        .expect("png encode")
        .as_bytes()
        .to_vec()
}

pub fn assert_surface_snapshot(surface: &mut Surface, snapshot_name: &str) {
    let actual = snapshot_png_bytes(surface);
    let snapshot_path = snapshot_path(snapshot_name);
    let expected = std::fs::read(&snapshot_path)
        .unwrap_or_else(|err| panic!("failed to read snapshot {}: {err}", snapshot_path.display()));
    assert_eq!(
        actual,
        expected,
        "snapshot mismatch for {}",
        snapshot_path.display()
    );
}

pub fn write_surface_snapshot(surface: &mut Surface, snapshot_name: &str) {
    let actual = snapshot_png_bytes(surface);
    let snapshot_path = snapshot_path(snapshot_name);
    if let Some(parent) = snapshot_path.parent() {
        std::fs::create_dir_all(parent).expect("create snapshot directory");
    }
    std::fs::write(&snapshot_path, actual).expect("write snapshot");
}

fn snapshot_path(snapshot_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(snapshot_name)
}

pub fn ensure_snapshot_parent(path: &Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
}
