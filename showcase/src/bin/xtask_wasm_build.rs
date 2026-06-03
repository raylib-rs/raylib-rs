//! Builds wasm examples in N batched `cargo build` invocations instead of one
//! per example. Examples are filtered against wasm-exclude.toml, optionally
//! sliced by `--shard <i>/<N>` for matrix-parallel CI, then bucketed by their
//! `required-features` set so a single cargo call covers each bucket.
//!
//! With `--keep-going` cargo continues compiling siblings after a single
//! example fails, and the post-build pass checks every requested example
//! produced its expected `.wasm` artifact under `target/wasm32-unknown-
//! emscripten/release/examples/`.
//!
//! CLI:
//!   xtask-wasm-build                build all wasm-eligible examples
//!   xtask-wasm-build --shard 1/4    build shard 1 of 4 (1-indexed)

use std::collections::BTreeMap;
use std::collections::HashMap;
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

struct Shard {
    index: usize, // 1-indexed
    total: usize,
}

fn parse_shard_arg() -> Option<Shard> {
    let mut args = std::env::args().skip(1);
    let arg = args.next()?;
    let value = if let Some(v) = arg.strip_prefix("--shard=") {
        v.to_string()
    } else if arg == "--shard" {
        args.next().expect("--shard requires a value like 1/4")
    } else {
        panic!("unknown arg: {}", arg);
    };
    let (i, n) = value
        .split_once('/')
        .expect("--shard value must be <index>/<total>, e.g. 1/4");
    let index: usize = i.parse().expect("--shard index must be a positive integer");
    let total: usize = n.parse().expect("--shard total must be a positive integer");
    assert!(
        index >= 1 && index <= total && total >= 1,
        "--shard must satisfy 1 <= index <= total, got {}/{}",
        index,
        total,
    );
    Some(Shard { index, total })
}

fn main() {
    let shard = parse_shard_arg();

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

    let total_metas = metas.len();
    let excluded_count = metas.iter().filter(|m| m.wasm_excluded).count();

    // Deterministic order so shard slicing is stable across runs.
    let mut buildable: Vec<&ExampleMeta> = metas.iter().filter(|m| !m.wasm_excluded).collect();
    buildable.sort_by(|a, b| a.name.cmp(&b.name));

    let slice: &[&ExampleMeta] = if let Some(s) = &shard {
        let len = buildable.len();
        let chunk = len.div_ceil(s.total);
        // Clamp both ends to len so trailing shards on a short list yield an
        // empty slice instead of panicking on an out-of-range start.
        let start = ((s.index - 1) * chunk).min(len);
        let end = (start + chunk).min(len);
        eprintln!(
            "xtask_wasm_build: shard {}/{} -> examples [{}..{}) of {}",
            s.index, s.total, start, end, len,
        );
        &buildable[start..end]
    } else {
        &buildable[..]
    };

    // Bucket the shard's examples by required-features set. BTreeMap keeps
    // bucket ordering stable for log readability.
    let mut buckets: BTreeMap<Vec<String>, Vec<&str>> = BTreeMap::new();
    for m in slice {
        let mut feats = required_features.get(&m.name).cloned().unwrap_or_default();
        feats.sort();
        feats.dedup();
        buckets.entry(feats).or_default().push(m.name.as_str());
    }

    let shell_path = manifest_dir.join("index").join("example_shell.html");
    let existing_cflags = std::env::var("EMCC_CFLAGS").unwrap_or_default();
    let new_cflags = format!("{} --shell-file {}", existing_cflags, shell_path.display());

    let mut bucket_failures: Vec<(Vec<String>, String)> = Vec::new();
    let mut requested_total = 0usize;

    for (feats, names) in &buckets {
        let label = if feats.is_empty() {
            "<no-features>".to_string()
        } else {
            feats.join(",")
        };
        eprintln!(
            "xtask_wasm_build: bucket features=[{}] count={}",
            label,
            names.len(),
        );
        requested_total += names.len();

        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "-p",
            "raylib-showcase",
            "--target",
            "wasm32-unknown-emscripten",
            "--release",
            "--keep-going",
        ]);
        if !feats.is_empty() {
            cmd.args(["--features", &feats.join(",")]);
        }
        for n in names {
            cmd.arg("--example").arg(*n);
        }
        cmd.env("EMCC_CFLAGS", &new_cflags);

        match cmd.status() {
            Ok(s) if s.success() => {}
            Ok(s) => bucket_failures.push((feats.clone(), format!("exit {:?}", s.code()))),
            Err(e) => bucket_failures.push((feats.clone(), format!("spawn: {}", e))),
        }
    }

    // Even with --keep-going, cargo exits non-zero if any target failed. Walk
    // the output dir so we can report per-example success/failure, which the
    // bucket-level exit code alone hides.
    let examples_out = workspace_root
        .join("target")
        .join("wasm32-unknown-emscripten")
        .join("release")
        .join("examples");

    let mut built = 0usize;
    let mut missing: Vec<&str> = Vec::new();
    for names in buckets.values() {
        for n in names {
            let wasm = examples_out.join(format!("{}.wasm", n));
            if wasm.exists() {
                built += 1;
            } else {
                missing.push(*n);
            }
        }
    }

    let shard_suffix = shard
        .as_ref()
        .map(|s| format!(" shard={}/{}", s.index, s.total))
        .unwrap_or_default();
    eprintln!(
        "xtask_wasm_build summary{}: {}/{} built ({} excluded of {} total, {} missing)",
        shard_suffix,
        built,
        requested_total,
        excluded_count,
        total_metas,
        missing.len(),
    );
    for n in &missing {
        eprintln!("  MISSING {}.wasm", n);
    }
    for (feats, reason) in &bucket_failures {
        let label = if feats.is_empty() {
            "<no-features>".to_string()
        } else {
            feats.join(",")
        };
        eprintln!("  BUCKET-FAIL features=[{}]: {}", label, reason);
    }

    if !missing.is_empty() {
        std::process::exit(1);
    }
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
