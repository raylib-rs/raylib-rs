//! Assembles `showcase/_site/` from the build outputs.
//!
//! P0 version: builds a single-example index referencing
//! `examples/core/core_basic_window.html`. Full categorized index lands in
//! P4 (Task 4.1).

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct ExampleMeta {
    name: String,
    category: String,
    wasm_excluded: bool,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();
    let site = manifest_dir.join("_site");
    let _ = fs::remove_dir_all(&site);
    fs::create_dir_all(site.join("examples")).unwrap();

    // Cargo creates multiple `raylib-showcase-<hash>` build dirs
    // (sometimes one for build-script output, one for run-script). Only
    // some contain the OUT_DIR/examples_meta.json our build.rs emits;
    // iterate and pick the one that actually has the file.
    let meta_path = find_examples_meta(&workspace_root)
        .expect("examples_meta.json not found; run `cargo build -p raylib-showcase --release --examples` first");
    let metas: Vec<ExampleMeta> =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    let index_template = fs::read_to_string(manifest_dir.join("index/template.html")).unwrap();
    let mut index_body = String::new();
    for m in &metas {
        let badge = if m.wasm_excluded {
            " (desktop only)"
        } else {
            ""
        };
        index_body.push_str(&format!(
            r#"<li><a href="examples/{}/{}.html">{} [{}]{}</a></li>"#,
            m.category, m.name, m.name, m.category, badge,
        ));
        index_body.push('\n');
    }
    let index_html = index_template.replace("{{EXAMPLES}}", &index_body);
    fs::write(site.join("index.html"), index_html).unwrap();
    fs::copy(manifest_dir.join("index/style.css"), site.join("style.css")).unwrap();
    fs::copy(manifest_dir.join("index/script.js"), site.join("script.js")).unwrap();

    let wasm_dir = workspace_root
        .join("target")
        .join("wasm32-unknown-emscripten")
        .join("release")
        .join("examples");
    for m in &metas {
        if m.wasm_excluded {
            continue;
        }
        let out_dir = site.join("examples").join(&m.category);
        fs::create_dir_all(&out_dir).unwrap();
        for ext in &["html", "js", "wasm", "data"] {
            let src = wasm_dir.join(format!("{}.{}", m.name, ext));
            if src.exists() {
                let dst = out_dir.join(format!("{}.{}", m.name, ext));
                let _ = fs::copy(src, dst);
            }
        }
    }

    let thumbs_src = workspace_root.join("target").join("thumbnails");
    if thumbs_src.exists() {
        let thumbs_dst = site.join("thumbnails");
        fs::create_dir_all(&thumbs_dst).unwrap();
        for e in fs::read_dir(thumbs_src).unwrap().flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("png") {
                let _ = fs::copy(&p, thumbs_dst.join(p.file_name().unwrap()));
            }
        }
    }

    eprintln!("xtask_build_pages: wrote site to {:?}", site);
}

fn find_examples_meta(workspace_root: &Path) -> Option<PathBuf> {
    for profile in &["release", "debug"] {
        let build_dir = workspace_root.join("target").join(profile).join("build");
        if !build_dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&build_dir).ok()?.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("raylib-showcase-") {
                continue;
            }
            let candidate = entry.path().join("out").join("examples_meta.json");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}
