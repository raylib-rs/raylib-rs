//! Iterates examples_meta.json minus wasm-exclude.toml; runs
//! `cargo build --target wasm32-unknown-emscripten --example <name>`
//! for each. Collects failures and prints a summary; exits non-zero on
//! any failure.
//!
//! Reads `required-features` per example from `showcase/Cargo.toml` so
//! raygui-gated examples build with `--features raygui` (otherwise cargo
//! refuses to build them).

use std::collections::HashMap;
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

#[derive(Deserialize)]
struct CargoToml {
    #[serde(rename = "example", default)]
    examples: Vec<CargoExample>,
}

#[derive(Deserialize)]
struct CargoExample {
    name: String,
    #[serde(rename = "required-features", default)]
    required_features: Vec<String>,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();

    let meta_path = find_examples_meta(&workspace_root).expect(
        "examples_meta.json not found; run `cargo build -p raylib-showcase --examples` first",
    );
    let metas: Vec<ExampleMeta> =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    let cargo_toml: CargoToml =
        toml::from_str(&fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap()).unwrap();
    let required_features: HashMap<String, Vec<String>> = cargo_toml
        .examples
        .into_iter()
        .filter(|e| !e.required_features.is_empty())
        .map(|e| (e.name, e.required_features))
        .collect();

    let total = metas.len();
    let mut excluded: HashSet<String> = HashSet::new();
    for m in &metas {
        if m.wasm_excluded {
            excluded.insert(m.name.clone());
        }
    }

    let mut failures: Vec<(String, String)> = Vec::new();
    let mut built = 0usize;

    let shell_path = manifest_dir.join("index").join("example_shell.html");

    for meta in &metas {
        if meta.wasm_excluded {
            eprintln!("xtask_wasm_build: skipping {} (wasm-excluded)", meta.name);
            continue;
        }
        eprintln!(
            "xtask_wasm_build: building {} for wasm32-unknown-emscripten",
            meta.name
        );
        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "-p",
            "raylib-showcase",
            "--target",
            "wasm32-unknown-emscripten",
            "--release",
            "--example",
            &meta.name,
        ]);
        if let Some(feats) = required_features.get(&meta.name) {
            cmd.args(["--features", &feats.join(",")]);
        }
        let existing_cflags = std::env::var("EMCC_CFLAGS").unwrap_or_default();
        let new_cflags = format!("{} --shell-file {}", existing_cflags, shell_path.display(),);
        cmd.env("EMCC_CFLAGS", new_cflags);
        let status = cmd.status();
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
