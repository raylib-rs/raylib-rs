//! Vendors raylib + raygui example resources into showcase/resources/
//! preserving directory structure so example load paths match the C
//! originals (path-level visual parity per WS9 spec D7).
//!
//! Run via:  cargo run -p raylib-showcase --bin xtask-vendor-resources
//!
//! Idempotent — re-run on raylib bumps to refresh.

use std::fs;
use std::path::{Path, PathBuf};

const RAYLIB_EXAMPLES: &str = "../raylib-sys/raylib/examples";
const RAYGUI_EXAMPLES: &str = "../raylib-sys/raygui-examples/examples";

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dest_root = manifest_dir.join("resources");
    fs::create_dir_all(&dest_root).unwrap();

    let mut copied = 0usize;
    let mut bytes = 0u64;

    // raylib categories: each category dir may contain a `resources/` subtree.
    let raylib_src = manifest_dir.join(RAYLIB_EXAMPLES);
    for cat in fs::read_dir(&raylib_src).unwrap().flatten() {
        if !cat.file_type().unwrap().is_dir() {
            continue;
        }
        let cat_name = cat.file_name().to_string_lossy().to_string();
        let res_src = cat.path().join("resources");
        if !res_src.exists() {
            continue;
        }
        let dest_cat = dest_root.join(&cat_name);
        let (n, b) = copy_tree(&res_src, &dest_cat);
        copied += n;
        bytes += b;
    }

    // raygui examples: nested per-program dirs, each may have its own
    // resources/ subtree. Plus a possible top-level resources/.
    let raygui_root = manifest_dir.join(RAYGUI_EXAMPLES);
    if raygui_root.exists() {
        let dest_raygui = dest_root.join("raygui");
        let top_res = raygui_root.join("resources");
        if top_res.exists() {
            let (n, b) = copy_tree(&top_res, &dest_raygui);
            copied += n;
            bytes += b;
        }
        for entry in fs::read_dir(&raygui_root).unwrap().flatten() {
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let res_src = entry.path().join("resources");
            if !res_src.exists() {
                continue;
            }
            let dest_sub = dest_raygui.join(entry.file_name());
            let (n, b) = copy_tree(&res_src, &dest_sub);
            copied += n;
            bytes += b;
        }
    }

    eprintln!(
        "xtask_vendor_resources: copied {} files, {:.1} MB -> {:?}",
        copied,
        bytes as f64 / 1_048_576.0,
        dest_root,
    );
}

fn copy_tree(src: &Path, dst: &Path) -> (usize, u64) {
    fs::create_dir_all(dst).unwrap();
    let mut count = 0usize;
    let mut bytes = 0u64;
    for e in fs::read_dir(src).unwrap().flatten() {
        let p = e.path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let dest = dst.join(&name);
        if p.is_dir() {
            let (n, b) = copy_tree(&p, &dest);
            count += n;
            bytes += b;
        } else {
            let meta = fs::metadata(&p).unwrap();
            bytes += meta.len();
            fs::copy(&p, &dest).unwrap();
            count += 1;
        }
    }
    (count, bytes)
}
