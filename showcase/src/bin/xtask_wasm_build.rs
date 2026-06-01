//! Iterates examples_meta.json minus wasm-exclude.toml; runs
//! `cargo build --target wasm32-unknown-emscripten --example <name>`
//! for each. Collects failures and prints a summary; exits non-zero on
//! any failure.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct ExampleMeta {
    name: String,
    #[allow(dead_code)]
    category: String,
    wasm_excluded: bool,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();

    let meta_path = find_examples_meta(&workspace_root)
        .expect("examples_meta.json not found; run `cargo build -p raylib-showcase --examples` first");
    let metas: Vec<ExampleMeta> =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    let total = metas.len();
    let mut excluded: HashSet<String> = HashSet::new();
    for m in &metas {
        if m.wasm_excluded {
            excluded.insert(m.name.clone());
        }
    }

    let mut failures: Vec<(String, String)> = Vec::new();
    let mut built = 0usize;

    for meta in &metas {
        if meta.wasm_excluded {
            eprintln!("xtask_wasm_build: skipping {} (wasm-excluded)", meta.name);
            continue;
        }
        eprintln!("xtask_wasm_build: building {} for wasm32-unknown-emscripten", meta.name);
        let status = Command::new("cargo")
            .args([
                "build",
                "-p",
                "raylib-showcase",
                "--target",
                "wasm32-unknown-emscripten",
                "--release",
                "--example",
                &meta.name,
            ])
            .status();
        match status {
            Ok(s) if s.success() => built += 1,
            Ok(s) => failures.push((meta.name.clone(), format!("exit {:?}", s.code()))),
            Err(e) => failures.push((meta.name.clone(), format!("spawn: {}", e))),
        }
    }

    eprintln!(
        "xtask_wasm_build summary: {}/{} built ({} excluded, {} failed)",
        built,
        total - excluded.len(),
        excluded.len(),
        failures.len()
    );
    for (name, reason) in &failures {
        eprintln!("  FAIL {}: {}", name, reason);
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
}

fn find_examples_meta(workspace_root: &Path) -> Option<PathBuf> {
    for profile in &["debug", "release"] {
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
