//! Thumbnail generator. Iterates examples_meta.json + thumbnails.toml,
//! spawns each non-excluded example with thumbnail env vars set, waits,
//! writes `target/thumbnails/_manifest.json` for xtask_build_pages.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ExampleMeta {
    name: String,
    category: String,
    #[allow(dead_code)]
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

#[derive(Deserialize, Default)]
struct ThumbsFile {
    #[serde(default)]
    example: Vec<ThumbsEntry>,
}

#[derive(Deserialize, Clone)]
struct ThumbsEntry {
    name: String,
    frames: Option<usize>,
    #[serde(default)]
    skip: bool,
}

#[derive(Serialize)]
struct ManifestEntry {
    name: String,
    category: String,
    thumbnail: Option<String>,
    reason: Option<String>,
}

const DEFAULT_FRAMES: usize = 60;
const SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();
    let out_thumbs = workspace_root.join("target").join("thumbnails");
    fs::create_dir_all(&out_thumbs).unwrap();

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

    let thumbs_overrides: HashMap<String, ThumbsEntry> = {
        let tpath = manifest_dir.join("thumbnails.toml");
        if tpath.exists() {
            let f: ThumbsFile =
                toml::from_str(&fs::read_to_string(&tpath).unwrap()).unwrap_or_default();
            f.example.into_iter().map(|e| (e.name.clone(), e)).collect()
        } else {
            HashMap::new()
        }
    };

    let mut manifest: Vec<ManifestEntry> = Vec::new();

    for meta in &metas {
        let override_entry = thumbs_overrides.get(&meta.name);
        if override_entry.map(|e| e.skip).unwrap_or(false) {
            manifest.push(ManifestEntry {
                name: meta.name.clone(),
                category: meta.category.clone(),
                thumbnail: None,
                reason: Some("thumbnails.toml: skip = true".into()),
            });
            continue;
        }
        let frames = override_entry
            .and_then(|e| e.frames)
            .unwrap_or(DEFAULT_FRAMES);
        let out = out_thumbs.join(format!("{}_{}.png", meta.category, meta.name));
        let extra_features = required_features
            .get(&meta.name)
            .cloned()
            .unwrap_or_default();
        let result = run_one(&meta.name, frames, &out, &extra_features);
        match result {
            Ok(()) => manifest.push(ManifestEntry {
                name: meta.name.clone(),
                category: meta.category.clone(),
                thumbnail: Some(format!("{}_{}.png", meta.category, meta.name)),
                reason: None,
            }),
            Err(e) => manifest.push(ManifestEntry {
                name: meta.name.clone(),
                category: meta.category.clone(),
                thumbnail: None,
                reason: Some(e),
            }),
        }
    }

    let manifest_path = out_thumbs.join("_manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let ok = manifest.iter().filter(|m| m.thumbnail.is_some()).count();
    let bad = manifest.len() - ok;
    eprintln!(
        "gen_thumbnails: {} ok, {} failed/skipped → {:?}",
        ok, bad, manifest_path
    );
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

fn run_one(name: &str, frames: usize, out: &Path, extra_features: &[String]) -> Result<(), String> {
    let mut features = vec!["software_renderer".to_string()];
    features.extend(extra_features.iter().cloned());
    let features_arg = features.join(",");
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "-p",
        "raylib-showcase",
        "--features",
        &features_arg,
        "--release",
        "--example",
        name,
    ]);
    cmd.env("RAYLIB_SHOWCASE_THUMBNAIL_FRAMES", frames.to_string());
    cmd.env(
        "RAYLIB_SHOWCASE_THUMBNAIL_OUT",
        out.to_string_lossy().to_string(),
    );
    let mut child = cmd.spawn().map_err(|e| format!("spawn: {}", e))?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| format!("try_wait: {}", e))? {
            if status.success() {
                return Ok(());
            }
            return Err(format!("exit code {:?}", status.code()));
        }
        if start.elapsed() > SUBPROCESS_TIMEOUT {
            let _ = child.kill();
            return Err(format!("timeout after {:?}", SUBPROCESS_TIMEOUT));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
