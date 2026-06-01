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

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

use walkdir::WalkDir;

#[derive(Debug)]
struct Pair {
    name: String,         // e.g. "core_basic_window"
    category: String,     // e.g. "core" or "raygui"
    c_path: PathBuf,      // absolute path to the C source
    rust_path: PathBuf,   // absolute path to the Rust port
}

const RAYLIB_EXAMPLES_DIR: &str = "../raylib-sys/raylib/examples";
const RAYGUI_EXAMPLES_DIR: &str = "../raylib-sys/raygui-examples/examples";

// C files in the raylib examples tree we explicitly do not port (templates, etc.).
const EXEMPT_C: &[&str] = &[
    "examples_template.c",
];

const RAYLIB_CATEGORIES: &[&str] = &[
    "audio", "core", "models", "others", "shaders", "shapes", "text", "textures",
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let raylib_root = manifest_dir
        .join(RAYLIB_EXAMPLES_DIR)
        .canonicalize()
        .expect("raylib-sys submodule must be checked out");
    let raygui_root = manifest_dir.join(RAYGUI_EXAMPLES_DIR).canonicalize().ok();

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
                .join(format!("{}.rs", name));
            if !rust_path.exists() {
                errors.push(format!(
                    "C example {}/{} has no Rust port at showcase/examples/{}/{}.rs",
                    category, fname, category, name,
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
            let candidate = entry.path().join(format!("{}.c", dirname));
            if !candidate.exists() {
                // No matching entry point — this dir contains only helpers or
                // a template (e.g. standalone/raygui_standalone.c). Skip.
                continue;
            }
            let name = dirname;
            let rust_path = manifest_dir
                .join("examples")
                .join("raygui")
                .join(format!("{}.rs", name));
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

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("build.rs: ERROR: {}", e);
        }
        panic!("build.rs: {} pairing error(s); fix per WS9 spec.", errors.len());
    }

    // cargo:rerun-if-changed for every walked file + the TOMLs.
    for p in &pairs {
        println!("cargo:rerun-if-changed={}", p.c_path.display());
        println!("cargo:rerun-if-changed={}", p.rust_path.display());
    }
    println!("cargo:rerun-if-changed=wasm-exclude.toml");
    println!("cargo:rerun-if-changed=thumbnails.toml");
    println!("cargo:rerun-if-changed=build.rs");

    // Parse wasm-exclude.toml to populate per-example wasm_excluded flag.
    let exclude_path = manifest_dir.join("wasm-exclude.toml");
    let wasm_excluded: HashSet<String> = if exclude_path.exists() {
        let txt = fs::read_to_string(&exclude_path).unwrap();
        let parsed: WasmExcludeFile = toml::from_str(&txt).unwrap_or_default();
        parsed.exclude.into_iter().map(|e| e.name).collect()
    } else {
        HashSet::new()
    };

    // Emit source_registry.rs using phf_codegen.
    let registry_path = out_dir.join("source_registry.rs");
    let mut map: phf_codegen::Map<String> = phf_codegen::Map::new();
    let mut meta: Vec<MetaEntry> = Vec::new();

    for p in &pairs {
        let value = format!(
            "SourcePair {{ c: include_str!(r\"{}\"), rust: include_str!(r\"{}\"), category: \"{}\" }}",
            p.c_path.display(),
            p.rust_path.display(),
            p.category,
        );
        map.entry(p.name.clone(), &value);
        meta.push(MetaEntry {
            name: p.name.clone(),
            category: p.category.clone(),
            wasm_excluded: wasm_excluded.contains(&p.name),
        });
    }

    let registry_src = format!(
        r#"// Auto-generated by showcase/build.rs. Do not edit.
pub struct SourcePair {{
    pub c: &'static str,
    pub rust: &'static str,
    pub category: &'static str,
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
}
