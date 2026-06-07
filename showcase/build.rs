//! Walks raylib-sys/raylib/examples + raylib-sys/raygui-examples and emits
//! a phf source-pair registry into $OUT_DIR/source_registry.rs, plus an
//! examples_meta.json sidecar consumed by xtask_build_pages.
//!
//! Hard-errors if a C source has no paired Rust file (or vice versa),
//! treating mismatch as a build-breaking porting bug.
//!
//! raylib examples layout: flat `examples/<category>/<name>.c` files.
//! raygui examples layout: nested `examples/<name>/<name>.c` (with possible
//! sibling helper .c files in the same dir — we only walk the dirname-matching
//! one). Skipping standalone/raygui_standalone.c (a template, not a runnable
//! example) falls out of the dirname-match rule.
//!
//! WS9 Phase 4 (Task 4.1): also resolves the submodule remotes, the pinned
//! submodule SHAs, and the showcase repo origin/HEAD at build time, and
//! bakes per-pair `c_url` / `rust_url` deep links into both the registry
//! source and the `examples_meta.json` sidecar so the Pages gallery and
//! the in-canvas SourceViewer can link straight to the upstream source.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

#[derive(Debug)]
struct Pair {
    name: String,       // e.g. "core_basic_window"
    category: String,   // e.g. "core" or "raygui"
    c_path: PathBuf,    // absolute path to the C source
    rust_path: PathBuf, // absolute path to the Rust port
}

const RAYLIB_EXAMPLES_DIR: &str = "../raylib-sys/raylib/examples";
const RAYGUI_EXAMPLES_DIR: &str = "../raylib-sys/raygui-examples/examples";

// C files in the raylib examples tree we explicitly do not port (templates, etc.).
const EXEMPT_C: &[&str] = &["examples_template.c"];

const RAYLIB_CATEGORIES: &[&str] = &[
    "audio", "core", "models", "others", "shaders", "shapes", "text", "textures",
];

// Fallbacks used if `git` is unavailable or returns an unexpected value
// (e.g. a CI checkout without an `origin` remote). We emit a
// `cargo:warning=` whenever a fallback fires so the deviation is visible
// in the build log.
const FALLBACK_SHOWCASE_REMOTE: &str = "https://github.com/raylib-rs/raylib-rs";
const FALLBACK_SHOWCASE_REF: &str = "6.0-rc";
const FALLBACK_RAYLIB_REMOTE: &str = "https://github.com/raysan5/raylib";
const FALLBACK_RAYGUI_REMOTE: &str = "https://github.com/raysan5/raygui";

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();

    let raylib_root = manifest_dir
        .join(RAYLIB_EXAMPLES_DIR)
        .canonicalize()
        .expect("raylib-sys submodule must be checked out");
    let raygui_root = manifest_dir.join(RAYGUI_EXAMPLES_DIR).canonicalize().ok();

    // Resolve submodule remotes from .gitmodules and pinned SHAs from each
    // submodule's HEAD. Fallbacks keep the build alive on non-submodule
    // checkouts (e.g. a published crate tarball), with a cargo:warning so
    // the deviation is visible.
    let raylib_remote = resolve_submodule_remote(
        &workspace_root,
        "submodule.raylib-sys/raylib.url",
        FALLBACK_RAYLIB_REMOTE,
    );
    let raygui_remote = resolve_submodule_remote(
        &workspace_root,
        "submodule.raylib-sys/raygui-examples.url",
        FALLBACK_RAYGUI_REMOTE,
    );
    let raylib_sha = resolve_git_head(
        &workspace_root.join("raylib-sys").join("raylib"),
        FALLBACK_SHOWCASE_REF,
    );
    let raygui_sha = resolve_git_head(
        &workspace_root.join("raylib-sys").join("raygui-examples"),
        FALLBACK_SHOWCASE_REF,
    );

    // Showcase repo origin + HEAD: same fallback strategy, but the origin
    // resolution also normalizes SSH-form remotes to https-form.
    let showcase_remote = resolve_showcase_remote(&workspace_root);
    let showcase_sha = resolve_git_head(&workspace_root, FALLBACK_SHOWCASE_REF);

    let mut pairs: Vec<Pair> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // raylib core: flat `examples/<category>/<name>.c`
    for entry in fs::read_dir(&raylib_root).expect("read raylib examples dir") {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let category = entry.file_name().to_string_lossy().to_string();
        if !RAYLIB_CATEGORIES.contains(&category.as_str()) {
            continue;
        }
        for cf in fs::read_dir(entry.path()).unwrap() {
            let cf = cf.unwrap();
            let path = cf.path();
            if path.extension().and_then(|s| s.to_str()) != Some("c") {
                continue;
            }
            let fname = path.file_name().unwrap().to_string_lossy().to_string();
            if EXEMPT_C.contains(&fname.as_str()) {
                continue;
            }
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let rust_path = manifest_dir
                .join("examples")
                .join(&category)
                .join(format!("{name}.rs"));
            if !rust_path.exists() {
                errors.push(format!(
                    "C example {category}/{fname} has no Rust port at showcase/examples/{category}/{name}.rs",
                ));
                continue;
            }
            pairs.push(Pair {
                name,
                category: category.clone(),
                c_path: path,
                rust_path,
            });
        }
    }

    // raygui: nested `examples/<name>/<name>.c`. Use directory-name == file-stem
    // to naturally pick the example file and skip helpers and templates.
    if let Some(raygui_root) = raygui_root.as_ref() {
        let raygui_dir_exists = manifest_dir.join("examples").join("raygui").exists();
        for entry in fs::read_dir(raygui_root).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let dirname = entry.file_name().to_string_lossy().to_string();
            // The "<dirname>/<dirname>.c" file is the example entry point.
            let candidate = entry.path().join(format!("{dirname}.c"));
            if !candidate.exists() {
                // No matching entry point — this dir contains only helpers or
                // a template (e.g. standalone/raygui_standalone.c). Skip.
                continue;
            }
            let name = dirname;
            let rust_path = manifest_dir
                .join("examples")
                .join("raygui")
                .join(format!("{name}.rs"));
            if !rust_path.exists() {
                // raygui examples land in P3. Only error if examples/raygui/
                // already exists (the maintainer has started porting raygui
                // examples) — otherwise silently skip until P3 begins.
                if raygui_dir_exists {
                    errors.push(format!(
                        "raygui C example {} has no Rust port at showcase/examples/raygui/{}.rs",
                        candidate.display(),
                        name,
                    ));
                }
                continue;
            }
            pairs.push(Pair {
                name,
                category: "raygui".to_string(),
                c_path: candidate,
                rust_path,
            });
        }
    }

    // Detect orphan Rust files (Rust ports without a paired C source).
    let known_rust: HashSet<PathBuf> = pairs.iter().map(|p| p.rust_path.clone()).collect();
    let examples_root = manifest_dir.join("examples");
    if examples_root.exists() {
        for e in WalkDir::new(&examples_root).into_iter().flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("rs") && !known_rust.contains(p) {
                errors.push(format!(
                    "Orphan Rust example: {} has no paired C source under {} or {}",
                    p.display(),
                    raylib_root.display(),
                    raygui_root
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<raygui-examples submodule not present>".to_string()),
                ));
            }
        }
    }

    // Pairing diagnostics. By default these are cargo warnings so that
    // incremental porting (P0..P2) doesn't block local builds or CI. When
    // `WS9_STRICT_PAIRING=1` is set (intended for the P2.5 close
    // verification onward), pairing mismatches escalate to a hard panic so
    // a forgotten port can never reach a release build silently.
    let strict = env::var("WS9_STRICT_PAIRING").ok().as_deref() == Some("1");
    if !errors.is_empty() {
        for e in &errors {
            if strict {
                eprintln!("build.rs: ERROR: {e}");
            } else {
                println!("cargo:warning=showcase: {e}");
            }
        }
        if strict {
            panic!(
                "build.rs: {} pairing error(s); fix per WS9 spec (WS9_STRICT_PAIRING=1).",
                errors.len(),
            );
        } else {
            println!(
                "cargo:warning=showcase: {} pairing diagnostic(s); set WS9_STRICT_PAIRING=1 to make them fatal.",
                errors.len(),
            );
        }
    }
    // Also re-emit a rerun-if-env-changed so toggling strictness re-runs the build script.
    println!("cargo:rerun-if-env-changed=WS9_STRICT_PAIRING");

    // cargo:rerun-if-changed for every walked file + the TOMLs.
    for p in &pairs {
        println!("cargo:rerun-if-changed={}", p.c_path.display());
        println!("cargo:rerun-if-changed={}", p.rust_path.display());
    }
    // Watch each category directory so additions/removals trigger a rerun even
    // before the new files are in `pairs` (cargo's contract: once any
    // rerun-if-changed is emitted, only listed paths re-trigger builds).
    for cat in RAYLIB_CATEGORIES {
        println!("cargo:rerun-if-changed={}", raylib_root.join(cat).display());
    }
    if let Some(raygui_root) = raygui_root.as_ref() {
        println!("cargo:rerun-if-changed={}", raygui_root.display());
    }
    println!("cargo:rerun-if-changed=wasm-exclude.toml");
    println!("cargo:rerun-if-changed=thumbnails.toml");
    println!("cargo:rerun-if-changed=build.rs");
    // Re-run when submodule pins or the showcase HEAD shift so c_url /
    // rust_url stay in sync with what's actually checked out.
    let gitmodules = workspace_root.join(".gitmodules");
    if gitmodules.exists() {
        println!("cargo:rerun-if-changed={}", gitmodules.display());
    }
    let head_paths = [
        workspace_root.join(".git").join("HEAD"),
        workspace_root
            .join(".git")
            .join("modules")
            .join("raylib-sys")
            .join("raylib")
            .join("HEAD"),
        workspace_root
            .join(".git")
            .join("modules")
            .join("raylib-sys")
            .join("raygui-examples")
            .join("HEAD"),
    ];
    for hp in &head_paths {
        if hp.exists() {
            println!("cargo:rerun-if-changed={}", hp.display());
        }
    }

    // Parse wasm-exclude.toml to populate per-example wasm_excluded flag.
    // A parse failure here would silently drop exclusions and could let
    // examples that should be wasm-excluded slip into the wasm build, so we
    // surface the parse error explicitly via cargo:warning= before falling
    // back to an empty list.
    let exclude_path = manifest_dir.join("wasm-exclude.toml");
    let wasm_excluded: HashSet<String> = if exclude_path.exists() {
        let txt = fs::read_to_string(&exclude_path).unwrap();
        let parsed: WasmExcludeFile = match toml::from_str::<WasmExcludeFile>(&txt) {
            Ok(p) => p,
            Err(e) => {
                println!(
                    "cargo:warning=showcase: failed to parse wasm-exclude.toml ({e}); treating as empty exclusion list",
                );
                WasmExcludeFile::default()
            }
        };
        parsed.exclude.into_iter().map(|e| e.name).collect()
    } else {
        HashSet::new()
    };

    // Emit source_registry.rs using phf_codegen.
    let registry_path = out_dir.join("source_registry.rs");
    let mut map: phf_codegen::Map<String> = phf_codegen::Map::new();
    let mut meta: Vec<MetaEntry> = Vec::new();

    for p in &pairs {
        let (c_url, rust_url) = build_urls(
            p,
            &raylib_remote,
            &raylib_sha,
            &raygui_remote,
            &raygui_sha,
            &showcase_remote,
            &showcase_sha,
        );
        let value = format!(
            "SourcePair {{ c: include_str!(r\"{}\"), rust: include_str!(r\"{}\"), category: \"{}\", c_url: \"{}\", rust_url: \"{}\" }}",
            p.c_path.display(),
            p.rust_path.display(),
            p.category,
            c_url,
            rust_url,
        );
        map.entry(p.name.clone(), &value);
        meta.push(MetaEntry {
            name: p.name.clone(),
            category: p.category.clone(),
            wasm_excluded: wasm_excluded.contains(&p.name),
            c_url,
            rust_url,
        });
    }

    let registry_src = format!(
        r#"// Auto-generated by showcase/build.rs. Do not edit.
pub struct SourcePair {{
    pub c: &'static str,
    pub rust: &'static str,
    pub category: &'static str,
    pub c_url: &'static str,
    pub rust_url: &'static str,
}}

pub static REGISTRY: phf::Map<&'static str, SourcePair> = {};
"#,
        map.build(),
    );
    fs::write(&registry_path, registry_src).unwrap();

    // Emit examples_meta.json (consumed by xtask_build_pages).
    let meta_path = out_dir.join("examples_meta.json");
    fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).unwrap();

    println!(
        "cargo:warning=showcase: {} example pair(s) registered",
        pairs.len(),
    );
}

#[derive(serde::Deserialize, Default)]
struct WasmExcludeFile {
    #[serde(default)]
    exclude: Vec<WasmExcludeEntry>,
}

#[derive(serde::Deserialize)]
struct WasmExcludeEntry {
    name: String,
    #[allow(dead_code)]
    reason: Option<String>,
}

#[derive(serde::Serialize)]
struct MetaEntry {
    name: String,
    category: String,
    wasm_excluded: bool,
    c_url: String,
    rust_url: String,
}

/// Reads `submodule.<name>.url` from the workspace's `.gitmodules`. Strips a
/// trailing `.git` and normalizes SSH-form (`git@github.com:owner/repo`) to
/// https-form. Returns `fallback` (with a cargo:warning) on any failure.
fn resolve_submodule_remote(workspace_root: &Path, key: &str, fallback: &str) -> String {
    let gitmodules = workspace_root.join(".gitmodules");
    let out = Command::new("git")
        .args(["config", "-f"])
        .arg(&gitmodules)
        .arg("--get")
        .arg(key)
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if raw.is_empty() {
                println!(
                    "cargo:warning=showcase: .gitmodules has no value for {key}; using fallback {fallback}",
                );
                normalize_remote(fallback)
            } else {
                normalize_remote(&raw)
            }
        }
        Ok(o) => {
            println!(
                "cargo:warning=showcase: git config -f .gitmodules --get {} exited {:?}; using fallback {}",
                key,
                o.status.code(),
                fallback,
            );
            normalize_remote(fallback)
        }
        Err(e) => {
            println!(
                "cargo:warning=showcase: failed to invoke git for {key} ({e}); using fallback {fallback}",
            );
            normalize_remote(fallback)
        }
    }
}

/// Reads the showcase repo's `origin` remote, normalized to https-form +
/// trailing-`.git`-stripped. Returns `FALLBACK_SHOWCASE_REMOTE` (with a
/// cargo:warning) on any failure.
fn resolve_showcase_remote(workspace_root: &Path) -> String {
    let out = Command::new("git")
        .args(["-C"])
        .arg(workspace_root)
        .args(["remote", "get-url", "origin"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if raw.is_empty() {
                println!(
                    "cargo:warning=showcase: `git remote get-url origin` returned empty; using fallback {FALLBACK_SHOWCASE_REMOTE}",
                );
                FALLBACK_SHOWCASE_REMOTE.to_string()
            } else {
                normalize_remote(&raw)
            }
        }
        Ok(o) => {
            println!(
                "cargo:warning=showcase: `git remote get-url origin` exited {:?}; using fallback {}",
                o.status.code(),
                FALLBACK_SHOWCASE_REMOTE,
            );
            FALLBACK_SHOWCASE_REMOTE.to_string()
        }
        Err(e) => {
            println!(
                "cargo:warning=showcase: failed to invoke git remote get-url origin ({e}); using fallback {FALLBACK_SHOWCASE_REMOTE}",
            );
            FALLBACK_SHOWCASE_REMOTE.to_string()
        }
    }
}

/// `git rev-parse HEAD` for a given repo / submodule, with a fallback ref
/// (e.g. the `6.0-rc` branch name) on any failure.
fn resolve_git_head(repo: &Path, fallback: &str) -> String {
    if !repo.exists() {
        println!("cargo:warning=showcase: {repo:?} does not exist; using ref fallback {fallback}",);
        return fallback.to_string();
    }
    let out = Command::new("git")
        .args(["-C"])
        .arg(repo)
        .args(["rev-parse", "HEAD"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if raw.is_empty() {
                println!(
                    "cargo:warning=showcase: `git rev-parse HEAD` in {repo:?} returned empty; using ref fallback {fallback}",
                );
                fallback.to_string()
            } else {
                raw
            }
        }
        Ok(o) => {
            println!(
                "cargo:warning=showcase: `git rev-parse HEAD` in {:?} exited {:?}; using ref fallback {}",
                repo,
                o.status.code(),
                fallback,
            );
            fallback.to_string()
        }
        Err(e) => {
            println!(
                "cargo:warning=showcase: failed to invoke git rev-parse in {repo:?} ({e}); using ref fallback {fallback}",
            );
            fallback.to_string()
        }
    }
}

/// Normalizes a remote URL to https-form and strips a trailing `.git`.
///
/// Accepts:
///   * `https://github.com/owner/repo[.git]`
///   * `http://github.com/owner/repo[.git]`
///   * `git@github.com:owner/repo[.git]`
///   * `ssh://git@github.com/owner/repo[.git]`
fn normalize_remote(raw: &str) -> String {
    let trimmed = raw.trim();
    // SSH short-form: `git@host:owner/repo`
    let https = if let Some(rest) = trimmed.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            format!("https://{host}/{path}")
        } else {
            trimmed.to_string()
        }
    } else if let Some(rest) = trimmed.strip_prefix("ssh://git@") {
        format!("https://{rest}")
    } else {
        trimmed.to_string()
    };
    https.strip_suffix(".git").unwrap_or(&https).to_string()
}

/// Builds the (c_url, rust_url) pair for a single example.
///
/// raylib pairs:   `{raylib_remote}/blob/{raylib_sha}/examples/{category}/{name}.c`
/// raygui pairs:   `{raygui_remote}/blob/{raygui_sha}/examples/{name}/{name}.c`
/// Rust port:      `{showcase_remote}/blob/{showcase_sha}/showcase/examples/{category}/{name}.rs`
fn build_urls(
    p: &Pair,
    raylib_remote: &str,
    raylib_sha: &str,
    raygui_remote: &str,
    raygui_sha: &str,
    showcase_remote: &str,
    showcase_sha: &str,
) -> (String, String) {
    let c_url = if p.category == "raygui" {
        format!(
            "{}/blob/{}/examples/{}/{}.c",
            raygui_remote, raygui_sha, p.name, p.name,
        )
    } else {
        format!(
            "{}/blob/{}/examples/{}/{}.c",
            raylib_remote, raylib_sha, p.category, p.name,
        )
    };
    let rust_url = format!(
        "{}/blob/{}/showcase/examples/{}/{}.rs",
        showcase_remote, showcase_sha, p.category, p.name,
    );
    (c_url, rust_url)
}
