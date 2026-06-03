# WS9 Showcase Finale Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a runnable Rust port of every official raylib 6.0 example (~228 total), each with a built-in side-by-side C↔Rust source viewer rendered in-canvas via raygui, deployed as a GitHub Pages gallery at `raylib-rs.github.io/raylib-rs/`.

**Architecture:** Per-example `[[example]]` Cargo targets (auto-discovered, declared explicitly in `showcase/Cargo.toml`) each with their own `fn main()` visually parallel to the C original. A workspace `build.rs` walks the `raylib-sys/raylib/examples/` + `raylib-sys/raygui-examples/` submodules at compile time and emits a `phf` source registry into `$OUT_DIR`; each example embeds the registry and reads its own (C, Rust) pair by `env!("CARGO_BIN_NAME")` lookup. A `SourceViewer` struct in the new `raylib-showcase` lib crate renders the in-canvas raygui overlay (`F1` to toggle).

**Tech Stack:** Rust 1.85 / edition 2024 / cargo workspaces / `phf` + `phf_codegen` for the registry / raylib-rs's existing safe API + raygui feature / emscripten 3.1.64 for wasm builds / GitHub Actions + Pages.

**Source-of-truth spec:** `docs/superpowers/specs/2026-05-31-ws9-showcase-design.md` (10 locked decisions + 8 sub-decisions + 6 phases).

---

## File Structure

### Created (cumulative across phases)

**P0 — scaffolding:**

| Path | Responsibility |
|---|---|
| `showcase/build.rs` | Walks raylib-sys + raygui-examples submodules, pairs C source with `showcase/examples/<cat>/<name>.rs`, emits `$OUT_DIR/source_registry.rs` (phf map) + `$OUT_DIR/examples_meta.json`. Hard error on mismatch. |
| `showcase/src/lib.rs` | Crate-internal library: re-exports `viewer::SourceViewer`, `registry::{EXAMPLES, ExampleMeta, lookup}`. |
| `showcase/src/viewer.rs` | `SourceViewer` struct: F1 toggle, C/Rust tabs, scroll, manually-laid-out text rendering, thumbnail-capture env-var branch. |
| `showcase/src/registry.rs` | `include!(concat!(env!("OUT_DIR"), "/source_registry.rs"));` + `lookup(name) -> Option<&'static SourcePair>`. |
| `showcase/src/bin/gen_thumbnails.rs` | Subprocess-spawning thumbnail generator. Reads `EXAMPLES`, spawns per-example with thumbnail env vars, writes `target/thumbnails/_manifest.json`. |
| `showcase/src/bin/xtask_wasm_build.rs` | Iterates `EXAMPLES` ∖ `wasm-exclude.toml`, shells out `cargo build --target wasm32-unknown-emscripten --example <name>`, collects + reports failures. |
| `showcase/src/bin/xtask_build_pages.rs` | Reads `examples_meta.json` + thumbnail manifest, populates `index/template.html`, writes `showcase/_site/`. |
| `showcase/wasm-exclude.toml` | `[[exclude]] name = "<n>" reason = "<r>"` entries. Starts empty (no exclusions known until P2). |
| `showcase/thumbnails.toml` | `[[example]] name = "<n>" frames = <N>` and/or `skip = true` overrides. Starts empty. |
| `showcase/examples/core/core_basic_window.rs` | The reference port. P0 smoke test. |
| `showcase/index/template.html` | Pages index template, Tera-style placeholders. |
| `showcase/index/example_shell.html` | Per-example emscripten `--shell-file` template. Wraps the `<canvas>` with gallery chrome. |
| `showcase/index/style.css` | Vanilla CSS for the gallery. |
| `showcase/index/script.js` | Vanilla JS for category filter + per-example navigation. |
| `showcase/.gitignore` | Ignores `target/thumbnails`, `_site`. |
| `.github/workflows/showcase.yml` | New workflow: 3-OS desktop build + Linux wasm build. Gates merges. |
| `.github/workflows/pages.yml` | New workflow: deploys `_site/` to GitHub Pages. |
| `raylib-sys/raygui-examples/` | New git submodule pointing at raysan5/raygui's `examples/` tree. |
| `.gitmodules` | Adds the new submodule entry. |

**P1 — resource vendor:**

| Path | Responsibility |
|---|---|
| `showcase/resources/<cat>/<file>` | Mirrors the upstream resource subtrees, preserving paths so example load paths are unchanged from C. |
| `showcase/resources/README.md` | Per-file license attribution. |
| `showcase/src/bin/xtask_vendor_resources.rs` | One-shot copier; can be re-run on upstream bump. |

**P2 — raylib core ports:**

| Path | Responsibility |
|---|---|
| `showcase/examples/audio/*.rs` (× 11) | One-to-one ports of `raylib-sys/raylib/examples/audio/*.c`. |
| `showcase/examples/core/*.rs` (× 49) | … |
| `showcase/examples/models/*.rs` (× 30) | … |
| `showcase/examples/others/*.rs` (× 3) | … |
| `showcase/examples/shaders/*.rs` (× 35) | … |
| `showcase/examples/shapes/*.rs` (× 41) | … |
| `showcase/examples/text/*.rs` (× 16) | … |
| `showcase/examples/textures/*.rs` (× 32) | … |

(Cargo.toml grows by ~218 `[[example]]` entries; wasm-exclude.toml + thumbnails.toml grow with discovered exclusions/overrides.)

**P3 — raygui ports:**

| Path | Responsibility |
|---|---|
| `showcase/examples/raygui/*.rs` (× ~10-15) | Ports of `raylib-sys/raygui-examples/examples/*.c`. `[[example]]` entries get `required-features = ["raygui"]`. |

**P4 — Pages polish + book cross-links:**

| Path | Responsibility |
|---|---|
| `book/src/appendix-examples.md` | New book appendix enumerating every example with gallery URL. |
| `book/src/SUMMARY.md` | Add the appendix entry. |
| `book/src/**/*.md` (existing chapters) | "See also: showcase examples" footers added per chapter that documents an example-relevant feature. |

**P5 — skill + done-note + status flip:**

| Path | Responsibility |
|---|---|
| `docs/superpowers/skills/raylib-showcase-port-flow.md` | Repo-tracked source of truth for the new skill. |
| `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` | User-trigger mirror. |
| `docs/superpowers/notes/ws9-showcase-complete.md` | Done-note: coverage table, exclusion list, tracked-deferred, CI inventory. |
| `CLAUDE.md` | Status line flipped to `… → WS9 showcase ✅ → final-release ← NEXT`. |
| `CHANGELOG.md` | WS9 entry under the `6.0.0 (unreleased)` heading. |

### Deleted (P0)

| Path | Reason |
|---|---|
| `showcase/src/main.rs` | Closure-launcher architecture is incompatible with the visual-parity rule. |
| `showcase/src/example/` (entire dir) | Same — all 66 existing closure-shaped ports rewritten as `examples/*.rs`. |
| `showcase/original/` (entire dir) | Replaced by direct reference to `raylib-sys/raylib/examples/` (single source of truth). |
| `showcase/Makefile` | Replaced by the new `xtask_wasm_build` + `xtask_build_pages` bins. |

### Modified

| Path | Phase | What changes |
|---|---|---|
| `Cargo.toml` (root) | P0 | Add `"showcase"` to `[workspace] members`. |
| `showcase/Cargo.toml` | P0–P3 | Version bump, `auto-examples = false`, build-deps, dependencies, `[[example]]` entries grow per wave. |
| `.github/workflows/test.yml` | P0 | One-line fold-in: append `+ binary(integration_rgui_icons)` to the raygui-leg `-E` expr on line 89. |

---

## Phase 0 — Scaffolding & End-to-End Smoke

### Task 0.1: Workspace integration + version bump

**Files:**
- Modify: `Cargo.toml`
- Modify: `showcase/Cargo.toml`

- [ ] **Step 1: Add showcase to root workspace**

Edit `Cargo.toml` (root):

```toml
[workspace]
members = ["raylib", "raylib-sys", "showcase"]
resolver = "2"
```

- [ ] **Step 2: Bump showcase version, lock edition, add build deps**

Replace `showcase/Cargo.toml` with:

```toml
[package]
name = "raylib-showcase"
version = "6.0.0-rc.1"
authors = ["raylib-rs team <https://github.com/raylib-rs/raylib-rs>"]
edition = "2024"
rust-version = "1.85"
license = "Zlib"
readme = "../README.md"
repository = "https://github.com/raylib-rs/raylib-rs"
description = "Runnable Rust ports of raylib's official C examples (raylib-rs 6.0 showcase)."

# We declare every [[example]] explicitly so we control the binary name
# (used by SourceViewer::for_current_example for registry lookup) and the
# nested path layout (which mirrors raylib-sys/raylib/examples/<cat>/<name>.c).
autoexamples = false
autobins = false

[features]
default = []
# Mirrors raylib's software_renderer for headless thumbnail generation.
software_renderer = ["raylib/software_renderer"]
# Required for the raygui examples in P3.
raygui = ["raylib/raygui"]

[dependencies]
raylib = { version = "6.0.0-rc.1", path = "../raylib" }
phf = { version = "0.11", features = ["macros"] }
toml = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[build-dependencies]
phf_codegen = "0.11"
walkdir = "2"

# Explicit example entries grow as P2/P3 batches land. P0 ships only the reference.
[[example]]
name = "core_basic_window"
path = "examples/core/core_basic_window.rs"

[[bin]]
name = "gen-thumbnails"
path = "src/bin/gen_thumbnails.rs"

[[bin]]
name = "xtask-wasm-build"
path = "src/bin/xtask_wasm_build.rs"

[[bin]]
name = "xtask-build-pages"
path = "src/bin/xtask_build_pages.rs"

[[bin]]
name = "xtask-vendor-resources"
path = "src/bin/xtask_vendor_resources.rs"
```

- [ ] **Step 3: Verify cargo sees the showcase package**

Run from repo root:

```bash
cargo metadata --no-deps --format-version 1 | python -c "import json,sys; m=json.load(sys.stdin); print(next(p['version'] for p in m['packages'] if p['name']=='raylib-showcase'))"
```

Expected: `6.0.0-rc.1`

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml showcase/Cargo.toml
git commit -m "$(cat <<'EOF'
build(ws9): showcase joins workspace + bumps to 6.0.0-rc.1

Per WS9 spec D1 (per-example [[example]] targets) + S1 (workspace
inclusion) + S2 (version bump). auto-examples = false so we control the
binary name; explicit [[example]] entries land per P2/P3 wave.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.2: Delete the old launcher + closure-shaped ports + vendored originals

**Files:**
- Delete: `showcase/src/main.rs`
- Delete: `showcase/src/example/` (recursive)
- Delete: `showcase/original/` (recursive)
- Delete: `showcase/Makefile`

- [ ] **Step 1: Verify the deletion targets are what we expect**

```bash
ls showcase/src/main.rs showcase/Makefile
find showcase/src/example -type f -name "*.rs" | wc -l
find showcase/original -type d | head -25
```

Expected: `main.rs` and `Makefile` exist; ~66 .rs files in `src/example/`; ~20 subdirs in `original/`.

- [ ] **Step 2: Delete**

```bash
rm showcase/src/main.rs showcase/Makefile
rm -rf showcase/src/example showcase/original
```

- [ ] **Step 3: Verify**

```bash
test ! -e showcase/src/main.rs && echo "main.rs gone"
test ! -e showcase/src/example && echo "src/example gone"
test ! -e showcase/original && echo "original gone"
test ! -e showcase/Makefile && echo "Makefile gone"
```

Expected: four "gone" lines.

- [ ] **Step 4: Commit**

```bash
git add -u showcase/
git commit -m "$(cat <<'EOF'
chore(ws9): delete old closure-launcher + vendored originals

The single-binary gui_list_view_ex launcher (showcase/src/main.rs) and
its closure-shaped run() ports under showcase/src/example/ are
structurally incompatible with the C-Rust visual-parity rule. The
curated vendored copy at showcase/original/ is replaced by direct
reference to raylib-sys/raylib/examples/ (D4). showcase/Makefile is
replaced by the new xtask_wasm_build + xtask_build_pages bins.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.3: Add the raygui-examples submodule

**Files:**
- Modify: `.gitmodules`
- Create: `raylib-sys/raygui-examples/` (submodule)

- [ ] **Step 1: Identify the raygui examples upstream commit**

The raygui hand-patch base (per `binding/raygui.h` and `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`) is upstream commit `4fbd425` on `raysan5/raygui` master. Pin the examples submodule to the same commit so example expectations match the bundled raygui.

- [ ] **Step 2: Add the submodule**

```bash
git submodule add -b master https://github.com/raysan5/raygui raylib-sys/raygui-examples
cd raylib-sys/raygui-examples && git checkout 4fbd425 && cd ../..
```

- [ ] **Step 3: Confirm the examples folder is present**

```bash
ls raylib-sys/raygui-examples/examples/ | head -20
find raylib-sys/raygui-examples/examples -name "*.c" | wc -l
```

Expected: the listing shows raygui's example .c files. Note the final count for later (likely ~10-15).

- [ ] **Step 4: Commit (submodule add + pin)**

```bash
git add .gitmodules raylib-sys/raygui-examples
git commit -m "$(cat <<'EOF'
build(ws9): add raygui-examples submodule

Pins raylib-sys/raygui-examples to raysan5/raygui@4fbd425 (matches the
raygui hand-patch base). Source for the ~10-15 raygui example ports
that land in P3. The build.rs registry walker reads this tree alongside
raylib-sys/raylib/examples/.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.4: Implement `showcase/build.rs` (registry generator)

**Files:**
- Create: `showcase/build.rs`

- [ ] **Step 1: Write the build script**

Create `showcase/build.rs`:

```rust
//! Walks raylib-sys/raylib/examples + raylib-sys/raygui-examples and emits
//! a phf source-pair registry into $OUT_DIR/source_registry.rs, plus an
//! examples_meta.json sidecar consumed by xtask_build_pages.
//!
//! Hard-errors if a C source has no paired Rust file (or vice versa),
//! treating mismatch as a build-breaking porting bug.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let raylib_root = manifest_dir.join(RAYLIB_EXAMPLES_DIR).canonicalize()
        .expect("raylib-sys submodule must be checked out");
    let raygui_root = manifest_dir.join(RAYGUI_EXAMPLES_DIR).canonicalize().ok();

    let mut pairs: Vec<Pair> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Walk raylib core categories.
    for entry in fs::read_dir(&raylib_root).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let category = entry.file_name().to_string_lossy().to_string();
        // Only walk known example categories; skip resources, etc.
        if !matches!(
            category.as_str(),
            "audio" | "core" | "models" | "others" | "shaders" | "shapes" | "text" | "textures"
        ) {
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
            let rust_path = manifest_dir.join("examples").join(&category).join(format!("{}.rs", name));
            if !rust_path.exists() {
                errors.push(format!(
                    "C example {}/{} has no Rust port at showcase/examples/{}/{}.rs",
                    category, fname, category, name,
                ));
                continue;
            }
            pairs.push(Pair { name, category, c_path: path, rust_path });
        }
    }

    // Walk raygui examples (single category).
    if let Some(raygui_root) = raygui_root {
        for cf in fs::read_dir(&raygui_root).unwrap_or_else(|_| fs::read_dir(&raygui_root).unwrap()) {
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
            let rust_path = manifest_dir.join("examples").join("raygui").join(format!("{}.rs", name));
            if !rust_path.exists() {
                // raygui examples are gated behind P3; missing Rust ports are
                // expected until P3 ships. Only error if the user explicitly
                // enabled raygui examples (heuristic: raygui dir exists).
                let raygui_dir = manifest_dir.join("examples").join("raygui");
                if raygui_dir.exists() {
                    errors.push(format!(
                        "raygui C example {} has no Rust port at showcase/examples/raygui/{}.rs",
                        fname, name,
                    ));
                }
                continue;
            }
            pairs.push(Pair {
                name,
                category: "raygui".to_string(),
                c_path: path,
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
                    RAYGUI_EXAMPLES_DIR,
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

    // Emit cargo:rerun-if-changed for every walked file + the two TOMLs.
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
        // Schema: [[exclude]] name = "<n>" reason = "<r>"
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
        // The phf value is a struct literal referencing const-included source strings.
        // To avoid embedding the source inside the codegen string, we use distinct
        // const items per example and reference them by name.
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

    println!("cargo:warning=showcase: {} example pair(s) registered", pairs.len());
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
```

- [ ] **Step 2: Add `serde` to build-deps**

Update `showcase/Cargo.toml` `[build-dependencies]`:

```toml
[build-dependencies]
phf_codegen = "0.11"
walkdir = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
```

- [ ] **Step 3: Verify build.rs panics correctly with no examples**

```bash
cd showcase && cargo build 2>&1 | tail -20
```

Expected: build fails with a clear "C example core/core_basic_window.c has no Rust port at showcase/examples/core/core_basic_window.rs" error (and many similar). This proves the pairing check fires.

- [ ] **Step 4: Commit**

```bash
git add showcase/build.rs showcase/Cargo.toml
git commit -m "$(cat <<'EOF'
build(ws9): showcase build.rs walks submodule examples + emits phf registry

Hard-error pairing check: every C example must have a paired Rust file
under showcase/examples/<cat>/<name>.rs (modulo EXEMPT_C). Orphan Rust
files also error. The registry emits at $OUT_DIR/source_registry.rs as
a phf::Map keyed by example name; sidecar examples_meta.json drives
xtask_build_pages. wasm-exclude.toml is read to populate the
wasm_excluded flag per entry.

Per WS9 spec §6.3 and §8 Flow A.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.5: Implement `src/lib.rs`, `src/registry.rs`, and the TOML schemas

**Files:**
- Create: `showcase/src/lib.rs`
- Create: `showcase/src/registry.rs`
- Create: `showcase/wasm-exclude.toml`
- Create: `showcase/thumbnails.toml`
- Create: `showcase/.gitignore`

- [ ] **Step 1: Create `showcase/src/lib.rs`**

```rust
//! raylib-rs showcase library crate.
//!
//! Each example under `showcase/examples/` is a self-contained `fn main()`
//! that depends on this crate for the source-viewer overlay and the
//! source-pair registry. See `docs/superpowers/specs/2026-05-31-ws9-showcase-design.md`.

pub mod registry;
pub mod viewer;

pub use registry::{ExampleMeta, EXAMPLES, lookup};
pub use viewer::SourceViewer;
```

- [ ] **Step 2: Create `showcase/src/registry.rs`**

```rust
//! Source-pair registry. The build script emits the inner `REGISTRY` map
//! into `$OUT_DIR/source_registry.rs`; this module includes it and exposes
//! a safe lookup API.

include!(concat!(env!("OUT_DIR"), "/source_registry.rs"));

/// Returns the (C, Rust) source pair for an example by its `[[example]] name`.
/// Returns `None` if the example isn't registered (the build script would
/// normally have errored out before we get here, but the API is total).
#[must_use]
pub fn lookup(name: &str) -> Option<&'static SourcePair> {
    REGISTRY.get(name)
}

/// Per-example metadata for the gallery index. The actual `EXAMPLES` slice
/// is populated at build time via `examples_meta.json`; for runtime the
/// registry is sufficient (the slice is only used by `xtask_build_pages`).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExampleMeta {
    pub name: String,
    pub category: String,
    pub wasm_excluded: bool,
}

/// The `EXAMPLES` slice is read at gallery-build time, not at example runtime.
/// `xtask_build_pages` consumes `$OUT_DIR/examples_meta.json` directly; this
/// public re-export is reserved for downstream tooling (post-release).
pub const EXAMPLES: &[ExampleMeta] = &[];
```

- [ ] **Step 3: Create `showcase/wasm-exclude.toml`**

```toml
# WS9 wasm-exclude list. Entries here are excluded from
# `cargo build --target wasm32-unknown-emscripten --example <name>` in CI
# and shown with a "desktop only" badge on the Pages gallery.
#
# Schema:
#   [[exclude]]
#   name = "<example name matching [[example]] name in Cargo.toml>"
#   reason = "one-line reason (LoadDroppedFiles, native dialog, threading, …)"
#
# Starts empty; entries land per P2/P3 wave as broken-on-wasm examples surface.
```

- [ ] **Step 4: Create `showcase/thumbnails.toml`**

```toml
# WS9 thumbnail-generation overrides. The default capture frame is 60
# (~1 second at 60 FPS). Per-example overrides land here.
#
# Schema:
#   [[example]]
#   name = "<example name>"
#   frames = <N>      # capture at frame N (use a higher value to let
#                     # animations play in)
#   skip = true       # skip thumbnail generation entirely (use upstream
#                     # PNG or placeholder)
#
# Starts empty.
```

- [ ] **Step 5: Create `showcase/.gitignore`**

```gitignore
target/
_site/
```

- [ ] **Step 6: Commit**

```bash
git add showcase/src/lib.rs showcase/src/registry.rs showcase/wasm-exclude.toml showcase/thumbnails.toml showcase/.gitignore
git commit -m "$(cat <<'EOF'
feat(ws9): showcase lib crate + registry + TOML schemas

src/lib.rs re-exports SourceViewer + registry surface. src/registry.rs
includes the build-emitted OUT_DIR/source_registry.rs and exposes a
safe lookup() helper. wasm-exclude.toml + thumbnails.toml are stubbed
with documented schemas; entries land in P2/P3 as exclusions/overrides
surface during porting.

Per WS9 spec §6.4 and §7 (Components table).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.6: Implement `src/viewer.rs` (the SourceViewer)

**Files:**
- Create: `showcase/src/viewer.rs`

- [ ] **Step 1: Write the viewer module**

Create `showcase/src/viewer.rs`:

```rust
//! In-canvas raygui source viewer.
//!
//! Each `examples/<cat>/<name>.rs` instantiates a `SourceViewer` after init
//! and calls `update` + `draw` inside its main loop. The viewer is invisible
//! by default; F1 toggles a full-screen overlay with C/Rust tabs.
//!
//! Hidden thumbnail-capture branch: when the environment variables
//! `RAYLIB_SHOWCASE_THUMBNAIL_FRAMES` and `RAYLIB_SHOWCASE_THUMBNAIL_OUT`
//! are set (by `gen_thumbnails`), the viewer counts frames and captures
//! the framebuffer at the configured frame, then exits the process.

use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use raylib::prelude::*;

use crate::registry::{lookup, SourcePair};

/// Which source the user is viewing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    C,
    Rust,
}

/// The in-canvas source viewer.
///
/// Constructed once per example main; ticked per frame via `update`/`draw`.
pub struct SourceViewer {
    pair: Option<&'static SourcePair>,
    name: String,
    visible: bool,
    tab: Tab,
    scroll_y: i32,
    line_height: i32,
    // Thumbnail-capture state (None unless env vars are set).
    thumbnail: Option<ThumbnailCapture>,
    frame_counter: usize,
}

struct ThumbnailCapture {
    target_frame: usize,
    out_path: PathBuf,
}

const HINT_TEXT: &str = "F1: view source";
const HINT_FONT_SIZE: i32 = 18;
const PANEL_BG: Color = Color { r: 30, g: 30, b: 38, a: 230 };
const PANEL_FG: Color = Color { r: 220, g: 220, b: 230, a: 255 };
const TAB_BG_ACTIVE: Color = Color { r: 80, g: 80, b: 120, a: 255 };
const TAB_BG_INACTIVE: Color = Color { r: 50, g: 50, b: 60, a: 255 };
const TEXT_FONT_SIZE: i32 = 14;

impl SourceViewer {
    /// Constructs a viewer keyed off the current `[[example]] name`
    /// (resolved at build time as `env!("CARGO_BIN_NAME")`).
    pub fn for_current_example() -> Self {
        // CARGO_BIN_NAME is set for binary AND example targets; for
        // example targets it matches the `[[example]] name`.
        let name = env!("CARGO_BIN_NAME").to_string();
        Self::for_example(&name)
    }

    /// Constructs a viewer for a named example.
    pub fn for_example(name: &str) -> Self {
        let pair = lookup(name);
        let thumbnail = thumbnail_from_env();
        Self {
            pair,
            name: name.to_string(),
            visible: false,
            tab: Tab::C,
            scroll_y: 0,
            line_height: TEXT_FONT_SIZE + 2,
            thumbnail,
            frame_counter: 0,
        }
    }

    /// Per-frame update: input handling + thumbnail-capture step.
    pub fn update(&mut self, rl: &mut RaylibHandle) {
        // Thumbnail-capture path runs invisibly.
        if let Some(t) = &self.thumbnail {
            self.frame_counter += 1;
            if self.frame_counter >= t.target_frame {
                capture_and_exit(rl, &t.out_path);
            }
            return;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            self.visible = !self.visible;
            if self.visible {
                self.scroll_y = 0;
            }
        }
        if !self.visible {
            return;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
            self.tab = match self.tab {
                Tab::C => Tab::Rust,
                Tab::Rust => Tab::C,
            };
            self.scroll_y = 0;
        }
        let lines_per_step = 10;
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_DOWN) {
            self.scroll_y += self.line_height * lines_per_step;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_PAGE_UP) {
            self.scroll_y = (self.scroll_y - self.line_height * lines_per_step).max(0);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            self.scroll_y = 0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_END) {
            // Approximate: scroll to (lines * line_height); clamping happens in draw.
            self.scroll_y = self.lines().count() as i32 * self.line_height;
        }
        // Mouse wheel scroll.
        let wheel = rl.get_mouse_wheel_move();
        if wheel != 0.0 {
            self.scroll_y =
                (self.scroll_y - (wheel * self.line_height as f32 * 3.0) as i32).max(0);
        }
    }

    /// Per-frame draw: closed → hint button; open → full overlay.
    pub fn draw<D: RaylibDraw>(&self, d: &mut D) {
        if self.thumbnail.is_some() {
            return; // Thumbnail path doesn't draw the viewer chrome.
        }
        if !self.visible {
            self.draw_hint(d);
            return;
        }
        self.draw_overlay(d);
    }

    fn draw_hint<D: RaylibDraw>(&self, d: &mut D) {
        let screen_w = d.get_screen_width();
        let screen_h = d.get_screen_height();
        let text_w = measure_text(HINT_TEXT, HINT_FONT_SIZE);
        let pad = 8;
        let x = screen_w - text_w - 2 * pad - 4;
        let y = screen_h - HINT_FONT_SIZE - 2 * pad - 4;
        d.draw_rectangle(
            x,
            y,
            text_w + 2 * pad,
            HINT_FONT_SIZE + 2 * pad,
            Color { r: 0, g: 0, b: 0, a: 160 },
        );
        d.draw_text(HINT_TEXT, x + pad, y + pad, HINT_FONT_SIZE, Color::WHITE);
    }

    fn draw_overlay<D: RaylibDraw>(&self, d: &mut D) {
        let screen_w = d.get_screen_width();
        let screen_h = d.get_screen_height();
        d.draw_rectangle(0, 0, screen_w, screen_h, PANEL_BG);

        // Header: example name + close hint.
        let header = format!("{}  —  F1: close · Tab: swap · PgUp/PgDn: scroll", self.name);
        d.draw_text(&header, 16, 12, 18, PANEL_FG);

        // Tabs.
        let tab_y = 40;
        let tab_w = 100;
        let tab_h = 28;
        let c_bg = if self.tab == Tab::C { TAB_BG_ACTIVE } else { TAB_BG_INACTIVE };
        let r_bg = if self.tab == Tab::Rust { TAB_BG_ACTIVE } else { TAB_BG_INACTIVE };
        d.draw_rectangle(16, tab_y, tab_w, tab_h, c_bg);
        d.draw_text("C", 16 + tab_w / 2 - 8, tab_y + 6, 20, PANEL_FG);
        d.draw_rectangle(16 + tab_w + 4, tab_y, tab_w, tab_h, r_bg);
        d.draw_text("Rust", 16 + tab_w + 4 + tab_w / 2 - 20, tab_y + 6, 20, PANEL_FG);

        // Body: line-by-line text, clipped to the viewport.
        let body_top = tab_y + tab_h + 12;
        let body_bottom = screen_h - 16;
        let body_left = 16;
        let viewport_h = body_bottom - body_top;
        let first_visible_line = (self.scroll_y / self.line_height).max(0);
        let lines_visible = (viewport_h / self.line_height) + 2;
        let mut y = body_top - (self.scroll_y % self.line_height);
        for (i, line) in self.lines().enumerate() {
            let idx = i as i32;
            if idx < first_visible_line {
                continue;
            }
            if idx > first_visible_line + lines_visible {
                break;
            }
            d.draw_text(line, body_left, y, TEXT_FONT_SIZE, PANEL_FG);
            y += self.line_height;
        }
    }

    fn lines(&self) -> std::str::Lines<'static> {
        let source = match (self.pair, self.tab) {
            (Some(p), Tab::C) => p.c,
            (Some(p), Tab::Rust) => p.rust,
            (None, _) => "(source not registered — build.rs walked tree did not include this example)",
        };
        source.lines()
    }
}

fn thumbnail_from_env() -> Option<ThumbnailCapture> {
    let frames = env::var("RAYLIB_SHOWCASE_THUMBNAIL_FRAMES").ok()?;
    let out = env::var("RAYLIB_SHOWCASE_THUMBNAIL_OUT").ok()?;
    let target_frame = frames.parse::<usize>().ok()?;
    Some(ThumbnailCapture {
        target_frame,
        out_path: PathBuf::from(out),
    })
}

/// Captures the current framebuffer and exits the process.
/// Uses raylib's `Image` round-trip; the WS4 software_renderer path
/// supplies the pixels.
fn capture_and_exit(rl: &mut RaylibHandle, out_path: &std::path::Path) {
    use std::process;

    let screen_w = rl.get_screen_width();
    let screen_h = rl.get_screen_height();
    // raylib::load_image_from_screen() copies the current backbuffer to an Image.
    // Available under software_renderer via the same API surface.
    let img = unsafe { Image::load_from_screen(rl) };
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let out_str = out_path.to_string_lossy().to_string();
    let ok = img.export_image(&out_str);
    if !ok {
        eprintln!("gen_thumbnails: export_image({:?}) failed", out_path);
        process::exit(2);
    }
    process::exit(0);
}

// Local atomic counter sentinel for use if multi-window cases emerge later.
#[allow(dead_code)]
static FRAME_COUNTER: AtomicUsize = AtomicUsize::new(0);
```

> **Note for implementer:** the `Image::load_from_screen` API may need an adjustment depending on the safe-API surface in `raylib/src/core/texture.rs`. If `load_from_screen` doesn't exist on `Image` directly, use the equivalent in the safe API (likely `unsafe { ffi::LoadImageFromScreen() }` wrapped in `Image::from_raw`); confirm before implementing. The wrapping is small and self-contained.

- [ ] **Step 2: Verify it compiles with one example present**

Skip until Task 0.11 (the reference example) — `lib.rs` compiles standalone but the viewer can't be exercised without an example main. The compile-clean step happens at 0.11.

- [ ] **Step 3: Commit**

```bash
git add showcase/src/viewer.rs
git commit -m "$(cat <<'EOF'
feat(ws9): SourceViewer — in-canvas raygui source overlay

F1 toggles visibility. When open: full-screen translucent panel with
C/Rust tabs (Tab swaps), PgUp/PgDn + Home/End + mousewheel scroll.
When closed: small "F1: view source" hint in the bottom-right.

Hidden thumbnail-capture branch: when RAYLIB_SHOWCASE_THUMBNAIL_FRAMES
+ RAYLIB_SHOWCASE_THUMBNAIL_OUT env vars are set, the viewer counts
frames and exits the process after capturing the backbuffer at the
configured frame. Zero per-example surface — every example calls
viewer.update anyway.

Per WS9 spec §6.4 and §8 Flow B + Flow D.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.7: Implement the four `src/bin/` tools (skeletons)

**Files:**
- Create: `showcase/src/bin/gen_thumbnails.rs`
- Create: `showcase/src/bin/xtask_wasm_build.rs`
- Create: `showcase/src/bin/xtask_build_pages.rs`
- Create: `showcase/src/bin/xtask_vendor_resources.rs`

- [ ] **Step 1: Create `gen_thumbnails.rs`**

```rust
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
    wasm_excluded: bool,
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
    thumbnail: Option<String>, // relative path under target/thumbnails/, or None for placeholder
    reason: Option<String>,    // populated on failure
}

const DEFAULT_FRAMES: usize = 60;
const SUBPROCESS_TIMEOUT: Duration = Duration::from_secs(30);

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().to_path_buf();
    let out_thumbs = workspace_root.join("target").join("thumbnails");
    fs::create_dir_all(&out_thumbs).unwrap();

    // build.rs emits this; we read it from the showcase package's OUT_DIR.
    // To locate OUT_DIR portably, we depend on the build having succeeded;
    // gen_thumbnails runs after cargo build --examples completes.
    let meta_path = find_examples_meta(&workspace_root)
        .expect("examples_meta.json not found; run `cargo build -p raylib-showcase --examples` first");
    let metas: Vec<ExampleMeta> =
        serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    let thumbs_overrides: HashMap<String, ThumbsEntry> = {
        let tpath = manifest_dir.join("thumbnails.toml");
        if tpath.exists() {
            let f: ThumbsFile = toml::from_str(&fs::read_to_string(&tpath).unwrap())
                .unwrap_or_default();
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
        let result = run_one(&meta.name, frames, &out);
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
    eprintln!("gen_thumbnails: {} ok, {} failed/skipped → {:?}", ok, bad, manifest_path);
}

fn find_examples_meta(workspace_root: &Path) -> Option<PathBuf> {
    // Search target/debug/build/raylib-showcase-*/out/examples_meta.json
    // and target/release/...
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

fn run_one(name: &str, frames: usize, out: &Path) -> Result<(), String> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "-p",
        "raylib-showcase",
        "--features",
        "software_renderer",
        "--release",
        "--example",
        name,
    ]);
    cmd.env("RAYLIB_SHOWCASE_THUMBNAIL_FRAMES", frames.to_string());
    cmd.env("RAYLIB_SHOWCASE_THUMBNAIL_OUT", out.to_string_lossy().to_string());
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
```

- [ ] **Step 2: Create `xtask_wasm_build.rs`**

```rust
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
```

- [ ] **Step 3: Create `xtask_build_pages.rs` (skeleton — single-example version for P0)**

```rust
//! Assembles `showcase/_site/` from the build outputs.
//!
//! P0 version: builds a single-example index referencing
//! `examples/core/core_basic_window.html`. Full categorized index lands in
//! P4 (Task 4.1).

use std::fs;
use std::path::PathBuf;

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

    // Read examples_meta.json.
    let meta_path = workspace_root
        .join("target")
        .join("release")
        .join("build")
        .read_dir()
        .ok()
        .and_then(|d| {
            d.flatten()
                .find(|e| e.file_name().to_string_lossy().starts_with("raylib-showcase-"))
                .map(|e| e.path().join("out").join("examples_meta.json"))
        })
        .expect("examples_meta.json not found; build the showcase first");
    let metas: Vec<ExampleMeta> = serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

    // Copy index template + style + script.
    let index_template = fs::read_to_string(manifest_dir.join("index/template.html")).unwrap();
    let mut index_body = String::new();
    for m in &metas {
        let badge = if m.wasm_excluded { " (desktop only)" } else { "" };
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

    // Copy per-example wasm shells.
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

    // Copy thumbnails (full set lands in P4; in P0 the dir may be empty).
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
```

- [ ] **Step 4: Create `xtask_vendor_resources.rs` (stub for P0; full impl in P1)**

```rust
//! Vendors raylib + raygui example resources into showcase/resources/
//! preserving directory structure. Full implementation lands in P1 (Task 1.1).
//!
//! P0 just panics with a "P1: not implemented" message so the bin entry
//! compiles cleanly; the corresponding bin entry in showcase/Cargo.toml is
//! present so the package builds.

fn main() {
    eprintln!("xtask_vendor_resources: not implemented in P0 (lands in P1).");
    std::process::exit(2);
}
```

- [ ] **Step 5: Commit**

```bash
git add showcase/src/bin/
git commit -m "$(cat <<'EOF'
feat(ws9): scaffolding for the four xtask bins

- gen_thumbnails: subprocess-spawning thumbnail generator (read
  examples_meta.json + thumbnails.toml, spawn per-example with env
  vars, write _manifest.json).
- xtask_wasm_build: iterate examples, run `cargo build --target
  wasm32-unknown-emscripten --example <n>` per non-excluded, summarize.
- xtask_build_pages: P0 single-example skeleton (full categorized index
  lands in P4 Task 4.1).
- xtask_vendor_resources: stub; real impl lands in P1 Task 1.1.

Per WS9 spec §7 Components and §8 Flows D + E.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.8: Port the reference example (`core_basic_window`)

**Files:**
- Create: `showcase/examples/core/core_basic_window.rs`

- [ ] **Step 1: Open the C source for reference**

```bash
cat raylib-sys/raylib/examples/core/core_basic_window.c
```

(Read the C; the Rust port must mirror its blank-line groupings, comment placement, and init/loop/close structure per the visual-parity rule.)

- [ ] **Step 2: Write the Rust port**

Create `showcase/examples/core/core_basic_window.rs`:

```rust
/*******************************************************************************************
*
*   raylib [core] example - Basic window
*
*   Welcome to raylib!
*
*   To test examples, just press F6 and run raylib_compile_execute script
*   NOTE: raylib_compile_execute script must be configured with proper paths
*
*   Example originally created with raylib 1.0, last time updated with raylib 1.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2013-2024 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - basic window")
        .build();
    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        viewer.update(&mut rl);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("Congrats! You created your first window!", 190, 200, 20, Color::LIGHTGRAY);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
```

- [ ] **Step 3: Verify the file passes pairing**

```bash
cargo build -p raylib-showcase --example core_basic_window 2>&1 | tail -10
```

Expected: build succeeds.

- [ ] **Step 4: Run it manually (smoke test of viewer)**

```bash
cargo run -p raylib-showcase --example core_basic_window
```

Expected:
- Window opens showing "Congrats! You created your first window!"
- A bottom-right "F1: view source" hint button is visible.
- Pressing F1 opens a full-screen overlay; the C source is shown by default; Tab swaps to Rust; PgUp/PgDn scrolls; F1 closes.

(Manual verification — the implementer must run this to confirm. If it fails, fix the viewer code before proceeding.)

- [ ] **Step 5: Commit**

```bash
git add showcase/examples/core/core_basic_window.rs
git commit -m "$(cat <<'EOF'
feat(ws9): reference example port — core_basic_window

The first WS9 port, exercising the full pipeline: build.rs registry
pairing, [[example]] target compilation, SourceViewer integration,
F1 overlay toggle. Mirrors raylib-sys/raylib/examples/core/
core_basic_window.c per the showcase-c-rust-port-style visual-parity
rule; two single-line additions for the viewer (init + per-frame
update/draw).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.9: Create the Pages index template + per-example shell

**Files:**
- Create: `showcase/index/template.html`
- Create: `showcase/index/example_shell.html`
- Create: `showcase/index/style.css`
- Create: `showcase/index/script.js`

- [ ] **Step 1: Create `template.html`**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>raylib-rs showcase — 6.0-rc</title>
  <link rel="stylesheet" href="style.css" />
</head>
<body>
  <header>
    <h1>raylib-rs showcase</h1>
    <p class="banner">
      Preview build of raylib-rs 6.0-rc &mdash;
      final docs at <a href="https://raylib-rs.github.io/">raylib-rs.github.io</a>
      after release.
    </p>
  </header>
  <main>
    <ul id="examples">
      {{EXAMPLES}}
    </ul>
  </main>
  <script src="script.js"></script>
</body>
</html>
```

- [ ] **Step 2: Create `example_shell.html` (emscripten `--shell-file`)**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <title>{{{ EXAMPLE_NAME }}}</title>
  <link rel="stylesheet" href="../../style.css" />
</head>
<body>
  <nav><a href="../../index.html">&larr; back to showcase</a></nav>
  <canvas id="canvas" oncontextmenu="event.preventDefault()" tabindex="-1"></canvas>
  <p class="hint">F1 inside the canvas: view source side-by-side.</p>
  <script>
    var Module = {
      canvas: (function() { return document.getElementById('canvas'); })(),
    };
  </script>
  {{{ SCRIPT }}}
</body>
</html>
```

(The `{{{ EXAMPLE_NAME }}}` + `{{{ SCRIPT }}}` placeholders are emscripten's standard ones; emscripten substitutes them at build time when this file is passed via `--shell-file`.)

- [ ] **Step 3: Create `style.css`**

```css
:root { font-family: system-ui, sans-serif; color-scheme: light dark; }
body { margin: 0; padding: 0; }
header { padding: 1rem; border-bottom: 1px solid #888; }
header h1 { margin: 0 0 0.25rem; font-size: 1.5rem; }
.banner { margin: 0; font-size: 0.85rem; opacity: 0.8; }
main { padding: 1rem; }
ul#examples { list-style: none; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 0.5rem; }
ul#examples li { border: 1px solid #888; border-radius: 4px; padding: 0.5rem; }
ul#examples a { text-decoration: none; color: inherit; }
nav { padding: 0.5rem; font-size: 0.9rem; }
canvas { display: block; margin: 1rem auto; background: black; }
.hint { text-align: center; font-size: 0.85rem; opacity: 0.7; }
```

- [ ] **Step 4: Create `script.js` (skeleton — full filter lands in P4)**

```javascript
// WS9 showcase index page script. P0 stub; full category filter +
// search lands in P4 Task 4.1.
console.log("raylib-rs showcase index loaded");
```

- [ ] **Step 5: Commit**

```bash
git add showcase/index/
git commit -m "$(cat <<'EOF'
feat(ws9): Pages index template + per-example emscripten shell

template.html — gallery index with {{EXAMPLES}} placeholder filled by
xtask_build_pages. example_shell.html — emscripten --shell-file
template wrapping the <canvas> with gallery chrome + a "back to
showcase" nav link. style.css + script.js skeleton; full categorized
index + filter land in P4.

Per WS9 spec §6.5.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.10: Create `.github/workflows/showcase.yml`

**Files:**
- Create: `.github/workflows/showcase.yml`

- [ ] **Step 1: Write the workflow**

```yaml
name: showcase

on:
  push:
    branches: [6.0-rc]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  # raylib-sys's emscripten target panics in build.rs without this. Matches
  # web.yml's pinned flag set.
  EMCC_CFLAGS: "-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v5
        with:
          submodules: recursive
      - name: Install Linux build deps
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - name: Install macOS build deps
        if: runner.os == 'macOS'
        run: brew install cmake
      - uses: dtolnay/rust-toolchain@1.85.0
      - name: Build examples (default features)
        run: cargo build -p raylib-showcase --examples
      - name: Build examples (raygui)
        run: cargo build -p raylib-showcase --examples --features raygui
      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest
      - name: Unit tests
        run: cargo nextest run -p raylib-showcase

  wasm-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
        with:
          submodules: recursive
      - name: Install Linux build deps
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake
      - name: Set up emsdk
        uses: mymindstorm/setup-emsdk@v14
        with:
          version: 3.1.64
          actions-cache-folder: emsdk-cache
      - name: Verify emcc on PATH
        run: emcc --version
      - name: Export bindgen sysroot for wasm32-emscripten
        run: echo "BINDGEN_EXTRA_CLANG_ARGS=--sysroot=$EMSDK/upstream/emscripten/cache/sysroot" >> $GITHUB_ENV
      - uses: dtolnay/rust-toolchain@1.85.0
        with:
          targets: wasm32-unknown-emscripten
      - name: Build showcase examples (release) to populate examples_meta.json
        run: cargo build -p raylib-showcase --release --examples
      - name: Iterate wasm builds across examples
        run: cargo run -p raylib-showcase --release --bin xtask-wasm-build
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/showcase.yml
git commit -m "$(cat <<'EOF'
ci(ws9): showcase workflow — 3-OS desktop + Linux wasm

Strict desktop matrix: cargo build -p raylib-showcase --examples
(default + raygui feature legs) + nextest unit tests. Linux wasm leg:
populate examples_meta.json via a release-mode showcase build, then
xtask-wasm-build iterates per-example wasm builds minus
wasm-exclude.toml. EMCC_CFLAGS + BINDGEN_EXTRA_CLANG_ARGS match
web.yml.

Per WS9 spec §10.2.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.11: Create `.github/workflows/pages.yml`

**Files:**
- Create: `.github/workflows/pages.yml`

- [ ] **Step 1: Write the workflow**

```yaml
name: pages

on:
  push:
    branches: [6.0-rc]
  workflow_dispatch:

permissions:
  pages: write
  id-token: write
  contents: read

concurrency:
  group: pages
  cancel-in-progress: false

env:
  CARGO_TERM_COLOR: always
  EMCC_CFLAGS: "-O3 -sUSE_GLFW=3 -sASSERTIONS=1 -sWASM=1 -sASYNCIFY -sGL_ENABLE_GET_PROC_ADDRESS=1"

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v5
        with:
          submodules: recursive
      - name: Install Linux build deps
        run: sudo apt-get update && sudo apt-get install --no-install-recommends -y cmake libasound2-dev libudev-dev libx11-dev libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev libgl1-mesa-dev
      - name: Set up emsdk
        uses: mymindstorm/setup-emsdk@v14
        with:
          version: 3.1.64
          actions-cache-folder: emsdk-cache
      - name: Export bindgen sysroot for wasm32-emscripten
        run: echo "BINDGEN_EXTRA_CLANG_ARGS=--sysroot=$EMSDK/upstream/emscripten/cache/sysroot" >> $GITHUB_ENV
      - uses: dtolnay/rust-toolchain@1.85.0
        with:
          targets: wasm32-unknown-emscripten
      - name: Populate examples_meta.json (release)
        run: cargo build -p raylib-showcase --release --examples
      - name: Build all wasm examples
        run: cargo run -p raylib-showcase --release --bin xtask-wasm-build
      - name: Generate thumbnails (software_renderer)
        run: cargo run -p raylib-showcase --release --features software_renderer --bin gen-thumbnails
      - name: Assemble Pages site
        run: cargo run -p raylib-showcase --release --bin xtask-build-pages
      - uses: actions/upload-pages-artifact@v3
        with:
          path: showcase/_site
      - id: deployment
        uses: actions/deploy-pages@v4
```

- [ ] **Step 2: Configure GitHub Pages on the fork repo**

Manual step (cannot be done from this plan):

```
On https://github.com/Dacode45/ms-raylib-rs/settings/pages:
- Source: GitHub Actions
- Save.
```

(The implementer or owner does this once. After this, pushes to `6.0-rc` deploy automatically.)

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/pages.yml
git commit -m "$(cat <<'EOF'
ci(ws9): pages workflow — wasm build + thumbnail gen + Pages deploy

Triggered on push to 6.0-rc. Pipeline: emsdk setup → showcase release
build (populates examples_meta.json) → xtask-wasm-build → gen-thumbnails
(software_renderer) → xtask-build-pages → upload-pages-artifact →
deploy-pages. URL during RC: raylib-rs.github.io/raylib-rs/. Canonical
URL flip handled by final-release workstream.

Per WS9 spec §10.3 + §8 Flow E.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.12: Fold-in: add `integration_rgui_icons` to `test.yml` raygui leg

**Files:**
- Modify: `.github/workflows/test.yml` (line 89)

- [ ] **Step 1: Edit the raygui leg**

In `.github/workflows/test.yml` line 89, change:

```yaml
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'binary(render_gui)'
```

to:

```yaml
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui -E 'binary(render_gui) + binary(integration_rgui_icons)'
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/test.yml
git commit -m "$(cat <<'EOF'
ci(test): add integration_rgui_icons to software-render raygui leg

Fold-in from the gui-icons workstream's tracked-deferred (the Tier-2
integration test passes locally but wasn't yet in CI). Per WS9 spec
§12 Tracked-deferred.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 0.13: End-to-end smoke verification

- [ ] **Step 1: Verify desktop build green**

```bash
cargo build -p raylib-showcase --examples
```

Expected: build succeeds with one example (`core_basic_window`).

- [ ] **Step 2: Verify nextest green**

```bash
cargo nextest run -p raylib-showcase
```

Expected: 0 tests run (no `#[test]` defined yet), exit 0.

- [ ] **Step 3: Verify the binary smoke-runs**

```bash
cargo run -p raylib-showcase --example core_basic_window
```

Expected: window opens, F1 opens overlay, Tab swaps tabs, ESC closes window.

- [ ] **Step 4: Push to fork; verify CI on GitHub Actions**

```bash
git push fork 6.0-rc
```

Then on `https://github.com/Dacode45/ms-raylib-rs/actions`:
- `check`, `test`, `web`, `sanitizers`, `book` (existing) — still green.
- `showcase` (new) — both `build` and `wasm-build` jobs green.
- `pages` (new) — green; URL output points at `raylib-rs.github.io/raylib-rs/`.
- Visit the URL; verify the single-example index renders and links to `examples/core/core_basic_window.html`.

If any workflow fails, fix it before declaring P0 done.

- [ ] **Step 5: Tag P0 close**

```bash
git tag ws9-p0-complete
git push fork ws9-p0-complete
```

P0 done. Subsequent phases assume this scaffolding is in place.

---

## Phase 1 — Vendor resources

### Task 1.1: Implement `xtask_vendor_resources.rs` (full impl)

**Files:**
- Modify: `showcase/src/bin/xtask_vendor_resources.rs`

- [ ] **Step 1: Replace the stub with the full implementation**

```rust
//! Vendors raylib + raygui example resources into showcase/resources/
//! preserving directory structure.
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

    // raylib categories
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

    // raygui (single category)
    let raygui_src = manifest_dir.join(RAYGUI_EXAMPLES).join("resources");
    if raygui_src.exists() {
        let dest_cat = dest_root.join("raygui");
        let (n, b) = copy_tree(&raygui_src, &dest_cat);
        copied += n;
        bytes += b;
    }

    eprintln!(
        "xtask_vendor_resources: copied {} files, {:.1} MB → {:?}",
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
```

- [ ] **Step 2: Run the vendoring**

```bash
cargo run -p raylib-showcase --bin xtask-vendor-resources 2>&1 | tail -5
```

Expected: summary line reports a non-zero count + size.

- [ ] **Step 3: Verify the tree**

```bash
find showcase/resources -type f | wc -l
du -sh showcase/resources
```

Note the count + size for the done-note.

- [ ] **Step 4: Commit (large diff expected)**

```bash
git add showcase/src/bin/xtask_vendor_resources.rs showcase/resources/
git commit -m "$(cat <<'EOF'
feat(ws9-p1): vendor raylib + raygui example resources

xtask_vendor_resources copies raylib-sys/raylib/examples/<cat>/resources/
+ raylib-sys/raygui-examples/examples/resources/ into showcase/
resources/<cat>/... preserving directory structure so example load
paths match the C originals (visual parity at the path level).
Idempotent — re-run on raylib bumps.

Per WS9 spec §6.1 + S4 + §9 Phase 1.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 1.2: Write the per-file license attribution README

**Files:**
- Create: `showcase/resources/README.md`

- [ ] **Step 1: Build the attribution document**

Create `showcase/resources/README.md`:

```markdown
# Resources

This directory mirrors the resource subtrees vendored from upstream:

- `raylib-sys/raylib/examples/<category>/resources/` → `resources/<category>/`
- `raylib-sys/raygui-examples/examples/resources/`   → `resources/raygui/`

Maintained by `cargo run -p raylib-showcase --bin xtask-vendor-resources`.

## License

All files are vendored under their upstream licenses. raylib's resources are
broadly under the **zlib/libpng** license (the same license raylib itself uses);
raygui's are likewise. A small number of community-contributed assets carry
**CC-BY** or **CC0** attribution requirements; those are listed below.

If you redistribute the showcase, preserve this README and the upstream
LICENSE files referenced.

### Per-file attribution

The implementer should populate this section per file during P1. The general
process:

1. For each file under `showcase/resources/`, check its upstream provenance.
2. If the file is part of raylib's standard examples bundle, no additional
   attribution beyond raylib's zlib/libpng applies.
3. If the file references an external attribution (look in raylib's
   `examples/<cat>/<name>.c` for a `// Resource by: ...` comment), copy
   that comment text here.

```

(The implementer audits the resources tree and fills in the per-file section. For raylib's standard bundle this is usually short — most resources are author-original under zlib/libpng.)

- [ ] **Step 2: Commit**

```bash
git add showcase/resources/README.md
git commit -m "$(cat <<'EOF'
docs(ws9-p1): showcase resources README with license attribution

Captures upstream attribution per WS9 spec D7. Default: zlib/libpng
inherited from raylib. CC-BY/CC0 callouts populated during the P1
audit pass.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 1.3: Verify resource paths in CI

**Files:**
- (none — CI verification only)

- [ ] **Step 1: Push and verify the showcase CI still passes**

```bash
git push fork 6.0-rc
```

Confirm `showcase.yml` `build` job remains green (resources don't affect compilation but tree size growth might affect submodule fetches).

- [ ] **Step 2: Tag P1 close**

```bash
git tag ws9-p1-complete
git push fork ws9-p1-complete
```

---

## Phase 2 — Port raylib core examples

### Task 2.0: The implementer + reviewer brief templates

These prompt templates are dispatched to subagents in Tasks 2.1–2.4. They live in this plan as canonical text so each wave's dispatch is repeatable. **Do not edit during a wave; if the rule changes, write a new task that updates the template.**

#### Implementer brief template (per category)

> **Role:** You are a Rust implementer porting raylib C examples to raylib-rs 6.0 under the visual-parity rule.
>
> **Scope:** Port every `.c` file under `raylib-sys/raylib/examples/<CATEGORY>/` to `showcase/examples/<CATEGORY>/<name>.rs`. Excluded: `examples_template.c` (already in `build.rs` `EXEMPT_C`).
>
> **Visual-parity rule (do not skip — this is the reviewer's primary criterion):** Open the C file alongside the Rust file. Keep blank-line groupings, comment positions, and init/loop/close structure visually parallel. Translate comment text but preserve location. Do not collapse explicit init/update/draw phases into closures or builders. Do not replace index-`for` loops with iterator chains for style alone (if iterators read genuinely better, leave the parallel `for` and add a one-line `// idiomatic: …` comment). Rust-only ergonomics worth a comment: RAII drop vs explicit `Unload*`, `&str` vs `*const c_char`, `Result`/`?` vs raw return-code checks, native math operator overloads vs raymath function calls.
>
> **Per-example workflow:**
> 1. Read `raylib-sys/raylib/examples/<CATEGORY>/<name>.c`.
> 2. Create `showcase/examples/<CATEGORY>/<name>.rs` mirroring the C.
> 3. Add two lines for the viewer:
>    - After init: `let mut viewer = SourceViewer::for_current_example();`
>    - Inside the loop: `viewer.update(&mut rl);` (in Update section) and `viewer.draw(&mut d);` (in Draw section, after the example's own draws).
> 4. Add the `[[example]]` entry to `showcase/Cargo.toml`:
>    ```toml
>    [[example]]
>    name = "<name>"
>    path = "examples/<CATEGORY>/<name>.rs"
>    ```
> 5. Run `cargo build -p raylib-showcase --example <name>`. Must pass.
> 6. Run `cargo build -p raylib-showcase --target wasm32-unknown-emscripten --example <name>`. If it fails:
>    - If the failure is due to a wasm-incompatible raylib API (LoadDroppedFiles, native dialogs, threads, audio recording, etc.), add an entry to `showcase/wasm-exclude.toml`:
>      ```toml
>      [[exclude]]
>      name = "<name>"
>      reason = "<one-line reason>"
>      ```
>    - If the failure is due to your own port code, fix the port code.
> 7. If the default 60-frame thumbnail capture isn't representative (the example doesn't show its visual until later — animations, async loads, etc.), add a `thumbnails.toml` override:
>    ```toml
>    [[example]]
>    name = "<name>"
>    frames = <higher number>
>    ```
>    Default is fine for most examples; only override when you've confirmed by running.
> 8. When all examples in the category are done, commit each example individually with a message like `feat(ws9-p2): port <CATEGORY>/<name>`. Group related commits (e.g., 2-3 examples that share a helper) where natural.
>
> **Resource paths:** raylib examples reference resources via relative paths like `"resources/heart_attack.wav"` (relative to the working directory raylib was launched from). The vendored copy in P1 mirrors these exact paths under `showcase/resources/<CATEGORY>/`. When running examples locally, the cwd should be `showcase/` (`cargo run -p raylib-showcase --example <name>` runs with `showcase/` as cwd). Verify the resource loads when you run the example.
>
> **Safe-API gaps:** If an example needs a raylib API surface that isn't wrapped in `raylib/`, prefer (in order):
> 1. Extend the safe API in a small focused change in `raylib/src/` — fold into your port commit.
> 2. Use `unsafe { raylib::ffi::… }` inline with a `// SAFETY: …` comment explaining soundness.
> 3. Escalate to the reviewer if the surface is large.
>
> **Do not:** edit `build.rs`, `viewer.rs`, `lib.rs`, `registry.rs`, the workflows, or anything outside `showcase/examples/<CATEGORY>/`, `showcase/Cargo.toml`, `showcase/wasm-exclude.toml`, `showcase/thumbnails.toml`, and the small `raylib/src/` extensions noted above.
>
> **Output:** report (a) count ported / total in category, (b) any wasm-exclusions added with reason, (c) any thumbnails.toml overrides added with reason, (d) any safe-API extensions made (file + summary), (e) any examples that you couldn't port and your reasoning.

#### Reviewer brief template (per category)

> **Role:** You are a reviewer for WS9 raylib core example ports under the visual-parity rule. The previous subagent ported the category `<CATEGORY>`.
>
> **What to check per example file:**
>
> 1. **Visual parity (the primary criterion):** Open `raylib-sys/raylib/examples/<CATEGORY>/<name>.c` and `showcase/examples/<CATEGORY>/<name>.rs` side by side.
>    - Blank-line groupings match.
>    - Comment positions match (translated text, same location).
>    - Init → loop → close structure is parallel.
>    - No collapse of phases into closures/builders.
>    - For-loops that were index-based in C are still index-based in Rust (or have a `// idiomatic: …` comment if iterators were chosen).
> 2. **Two viewer wiring additions** are present and correctly placed (after init + inside the loop).
> 3. `[[example]]` entry in `showcase/Cargo.toml` matches `name` + `path`.
> 4. `cargo build -p raylib-showcase --example <name>` is clean.
> 5. `cargo build -p raylib-showcase --target wasm32-unknown-emscripten --example <name>` is clean *or* the example is in `wasm-exclude.toml` with a reason.
> 6. No edits outside the allowed paths.
>
> **What to do with violations:**
>
> - Visual-parity violation → flag with file:line + suggested fix; do NOT fix yourself (kick back to implementer).
> - Build failure → flag; the implementer dispatches a fix.
> - Soundness concern in `unsafe { … }` blocks → flag with the specific concern; do NOT fix yourself.
> - Style nit that doesn't affect parity → note but accept.
>
> **Output:** per-example accept/reject. If reject, the implementer fixes and the reviewer re-checks just the fixed files.

---

### Task 2.1: Wave 1 — `audio` (11) + `others` (3)

- [ ] **Step 1: Dispatch implementer for `audio`**

Subagent: `general-purpose`, prompt = implementer brief with `<CATEGORY>` = `audio`. Run in background.

- [ ] **Step 2: Dispatch implementer for `others`**

Subagent: `general-purpose`, prompt = implementer brief with `<CATEGORY>` = `others`. Run in background.

- [ ] **Step 3: Wait for both, then dispatch reviewers**

When both implementers report done, dispatch reviewer subagent for each with the reviewer brief template. Run in background.

- [ ] **Step 4: Process review results**

For each rejection, dispatch implementer fix subagent with the specific feedback. Re-review fixed files. Loop until clean.

- [ ] **Step 5: Verify locally**

```bash
cargo build -p raylib-showcase --examples
cargo run -p raylib-showcase --release --bin xtask-wasm-build
```

Expected: both green.

- [ ] **Step 6: Push and verify CI**

```bash
git push fork 6.0-rc
```

Verify `showcase.yml` `build` and `wasm-build` jobs green for the new examples.

- [ ] **Step 7: Tag W1 close**

```bash
git tag ws9-p2-w1-complete
git push fork ws9-p2-w1-complete
```

---

### Task 2.2: Wave 2 — `core` (49) + `text` (16)

Same shape as Task 2.1, with `<CATEGORY>` = `core` and `text`. `core` is the largest single category (49 examples); the implementer may split this into two sub-batches (e.g., `core_2*` + `core_3*` + a few stragglers in batch A; `core_basic*`, `core_input*`, `core_random*`, etc. in batch B) if context gets unwieldy. Note `core_basic_window` is already ported in P0; the implementer should skip it (not re-port) and confirm it's still in `Cargo.toml` with the correct path.

- [ ] **Step 1: Dispatch implementer for `core` (warn about size)**
- [ ] **Step 2: Dispatch implementer for `text`**
- [ ] **Step 3: Wait, dispatch reviewers**
- [ ] **Step 4: Process review results, loop until clean**
- [ ] **Step 5: Local verify (`cargo build --examples` + xtask-wasm-build)**
- [ ] **Step 6: Push, verify CI**
- [ ] **Step 7: Tag `ws9-p2-w2-complete`**

---

### Task 2.3: Wave 3 — `models` (30) + `shaders` (35)

Same shape; `<CATEGORY>` = `models` and `shaders`. The implementer for `shaders` may split into shader-language-specific sub-batches (`shaders_basic_lighting`, `shaders_*`, fragment vs vertex, etc.) if useful. Shaders often have GLSL files alongside; verify those load correctly via the vendored `showcase/resources/shaders/`.

- [ ] **Step 1–7: Same as Task 2.1.** Tag close as `ws9-p2-w3-complete`.

---

### Task 2.4: Wave 4 — `shapes` (41) + `textures` (32)

Same shape; `<CATEGORY>` = `shapes` and `textures`. Both are above the 30-example split threshold; the implementer is encouraged to sub-batch.

- [ ] **Step 1–7: Same as Task 2.1.** Tag close as `ws9-p2-w4-complete`.

---

### Task 2.5: P2 close verification

- [ ] **Step 1: Verify all 218 examples build**

```bash
cargo build -p raylib-showcase --examples 2>&1 | tail -5
```

Expected: clean.

- [ ] **Step 2: Verify wasm builds (minus exclusions)**

```bash
cargo run -p raylib-showcase --release --bin xtask-wasm-build 2>&1 | tail -10
```

Expected: summary line shows `N/218 built (M excluded, 0 failed)`. Compare against `wasm-exclude.toml` count to verify exclusions are intentional.

- [ ] **Step 3: Count check**

```bash
find showcase/examples -name "*.rs" -not -path "*/raygui/*" | wc -l
```

Expected: 218 (the raylib core count).

- [ ] **Step 4: Spot-check 5 examples end-to-end**

Pick 5 examples across categories (e.g., `audio_module_playing`, `core_2d_camera`, `models_animation`, `shaders_basic_lighting`, `textures_bunnymark`). For each:

```bash
cargo run -p raylib-showcase --example <name>
```

Verify: window opens, the example renders as expected, F1 overlay works (shows the correct C and Rust sources).

- [ ] **Step 5: Push + verify CI**

```bash
git push fork 6.0-rc
```

`showcase.yml` green.

- [ ] **Step 6: Tag P2 close**

```bash
git tag ws9-p2-complete
git push fork ws9-p2-complete
```

---

## Phase 3 — Port raygui examples

### Task 3.1: Enable the raygui registry walker

The `build.rs` already walks `raylib-sys/raygui-examples/examples/`, but the orphan check is suppressed until any Rust port exists. To trigger the walker, create the `showcase/examples/raygui/` directory first.

- [ ] **Step 1: Create the directory**

```bash
mkdir -p showcase/examples/raygui
```

This causes `build.rs` to start expecting Rust ports for every raygui C example. Any unported raygui example will error the build — so the next step must port them all in one batch.

- [ ] **Step 2: Verify the build now errors with raygui pairing complaints**

```bash
cargo build -p raylib-showcase --examples 2>&1 | grep "raygui" | head -5
```

Expected: errors per missing raygui port.

(Don't commit this directory creation as a separate commit; it lands with Task 3.2's ports.)

---

### Task 3.2: Dispatch single implementer + reviewer batch for raygui

- [ ] **Step 1: Dispatch implementer**

Subagent prompt = the implementer brief from Task 2.0, with:
- `<CATEGORY>` = `raygui`
- C source root = `raylib-sys/raygui-examples/examples/` (not the raylib one)
- Each `[[example]]` entry gets `required-features = ["raygui"]`:
  ```toml
  [[example]]
  name = "controls_test_suite"
  path = "examples/raygui/controls_test_suite.rs"
  required-features = ["raygui"]
  ```
- Apply the `rgui-feature-gate-rule` memory: every example main and any helper that calls `ffi::Gui*` must be gated. Since `required-features` already gates the `[[example]]`, examples don't need internal `#[cfg]` — but any helper added to `raylib-showcase` lib (rare) does.

- [ ] **Step 2: Dispatch reviewer**

Same as Task 2.0 reviewer brief, with the additional check:
- Every raygui `[[example]]` entry has `required-features = ["raygui"]`.

- [ ] **Step 3: Loop until clean**

- [ ] **Step 4: Verify local builds**

```bash
cargo build -p raylib-showcase --examples --features raygui
cargo run -p raylib-showcase --release --bin xtask-wasm-build
```

Both clean.

- [ ] **Step 5: Push, verify CI**

```bash
git push fork 6.0-rc
```

- [ ] **Step 6: Tag P3 close**

```bash
git tag ws9-p3-complete
git push fork ws9-p3-complete
```

---

## Phase 4 — Pages polish + book cross-links

### Task 4.1: Full categorized index in `xtask_build_pages.rs`

**Files:**
- Modify: `showcase/src/bin/xtask_build_pages.rs`
- Modify: `showcase/index/template.html`
- Modify: `showcase/index/style.css`
- Modify: `showcase/index/script.js`

- [ ] **Step 1: Update `template.html` for category sections**

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>raylib-rs showcase — 6.0-rc</title>
  <link rel="stylesheet" href="style.css" />
</head>
<body>
  <header>
    <h1>raylib-rs showcase</h1>
    <p class="banner">
      Preview build of raylib-rs 6.0-rc &mdash;
      final docs at <a href="https://raylib-rs.github.io/">raylib-rs.github.io</a>
      after release.
    </p>
    <input id="filter" type="search" placeholder="filter by name…" autocomplete="off" />
    <p id="status"></p>
  </header>
  <main>
    {{CATEGORIES}}
  </main>
  <script src="script.js"></script>
</body>
</html>
```

- [ ] **Step 2: Update `xtask_build_pages.rs` to emit per-category sections + thumbnail integration**

Replace the body of `main` after the metadata read with:

```rust
    // Group by category.
    let mut by_cat: std::collections::BTreeMap<String, Vec<&ExampleMeta>> =
        std::collections::BTreeMap::new();
    for m in &metas {
        by_cat.entry(m.category.clone()).or_default().push(m);
    }

    // Load thumbnail manifest.
    let thumbs_dir = workspace_root.join("target").join("thumbnails");
    let manifest_path = thumbs_dir.join("_manifest.json");
    let manifest: Vec<ManifestEntry> = if manifest_path.exists() {
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap()
    } else {
        Vec::new()
    };
    let thumb_lookup: std::collections::HashMap<String, Option<String>> =
        manifest.iter().map(|m| (m.name.clone(), m.thumbnail.clone())).collect();

    let mut categories_html = String::new();
    for (cat, examples) in &by_cat {
        categories_html.push_str(&format!("<section class=\"category\" id=\"cat-{}\"><h2>{}</h2>\n<div class=\"grid\">\n", cat, cat));
        for m in examples {
            let badge = if m.wasm_excluded { "<span class=\"badge\">desktop only</span>" } else { "" };
            let thumb = thumb_lookup.get(&m.name).and_then(|t| t.as_ref());
            let thumb_html = match thumb {
                Some(t) => format!("<img loading=\"lazy\" src=\"thumbnails/{}\" alt=\"\" />", t),
                None => String::from("<div class=\"placeholder\"></div>"),
            };
            let href = if m.wasm_excluded {
                format!("examples/{}/{}.html#desktop-only", cat, m.name)
            } else {
                format!("examples/{}/{}.html", cat, m.name)
            };
            categories_html.push_str(&format!(
                "<a class=\"tile\" href=\"{}\" data-name=\"{}\"><div class=\"thumb\">{}</div><p>{}{}</p></a>\n",
                href, m.name, thumb_html, m.name, badge,
            ));
        }
        categories_html.push_str("</div></section>\n");
    }

    let index_template = fs::read_to_string(manifest_dir.join("index/template.html")).unwrap();
    let index_html = index_template.replace("{{CATEGORIES}}", &categories_html);
    fs::write(site.join("index.html"), index_html).unwrap();
    fs::copy(manifest_dir.join("index/style.css"), site.join("style.css")).unwrap();
    fs::copy(manifest_dir.join("index/script.js"), site.join("script.js")).unwrap();

    // Per-example shells (same as P0 code, unchanged).
    let wasm_dir = workspace_root
        .join("target")
        .join("wasm32-unknown-emscripten")
        .join("release")
        .join("examples");
    for m in &metas {
        let out_dir = site.join("examples").join(&m.category);
        fs::create_dir_all(&out_dir).unwrap();
        if m.wasm_excluded {
            // Write a "desktop only" placeholder page.
            let pair_lookup = format!(
                r#"<p>This example is desktop-only. Run locally:</p>
<pre><code>cargo run -p raylib-showcase --example {}</code></pre>"#,
                m.name,
            );
            let html = format!(
                "<!DOCTYPE html><html><head><title>{} (desktop only)</title><link rel=\"stylesheet\" href=\"../../style.css\"></head><body><nav><a href=\"../../index.html\">&larr; back</a></nav><h1>{}</h1>{}</body></html>",
                m.name, m.name, pair_lookup,
            );
            fs::write(out_dir.join(format!("{}.html", m.name)), html).unwrap();
            continue;
        }
        for ext in &["html", "js", "wasm", "data"] {
            let src = wasm_dir.join(format!("{}.{}", m.name, ext));
            if src.exists() {
                let dst = out_dir.join(format!("{}.{}", m.name, ext));
                let _ = fs::copy(src, dst);
            }
        }
    }

    // Copy thumbnails dir.
    if thumbs_dir.exists() {
        let thumbs_dst = site.join("thumbnails");
        fs::create_dir_all(&thumbs_dst).unwrap();
        for e in fs::read_dir(&thumbs_dir).unwrap().flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("png") {
                let _ = fs::copy(&p, thumbs_dst.join(p.file_name().unwrap()));
            }
        }
    }
```

Also add the `ManifestEntry` struct above `main`:

```rust
#[derive(Deserialize, Clone)]
struct ManifestEntry {
    name: String,
    #[allow(dead_code)]
    category: String,
    thumbnail: Option<String>,
    #[allow(dead_code)]
    reason: Option<String>,
}
```

- [ ] **Step 3: Update `style.css`**

```css
:root { font-family: system-ui, sans-serif; color-scheme: light dark; }
body { margin: 0; padding: 0; }
header { padding: 1rem; border-bottom: 1px solid #888; }
header h1 { margin: 0 0 0.25rem; font-size: 1.5rem; }
.banner { margin: 0 0 0.5rem; font-size: 0.85rem; opacity: 0.8; }
#filter { width: 100%; padding: 0.4rem 0.6rem; font-size: 1rem; box-sizing: border-box; }
#status { margin: 0.25rem 0 0; font-size: 0.8rem; opacity: 0.7; min-height: 1em; }
main { padding: 1rem; }
.category h2 { margin: 1.5rem 0 0.5rem; font-size: 1.1rem; text-transform: lowercase; opacity: 0.7; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 0.75rem; }
.tile { display: flex; flex-direction: column; border: 1px solid #888; border-radius: 4px; padding: 0.5rem; text-decoration: none; color: inherit; }
.tile .thumb { aspect-ratio: 16 / 10; overflow: hidden; background: #222; border-radius: 2px; }
.tile .thumb img { width: 100%; height: 100%; object-fit: cover; }
.tile .placeholder { width: 100%; height: 100%; background: linear-gradient(135deg, #444, #222); }
.tile p { margin: 0.4rem 0 0; font-size: 0.85rem; font-family: monospace; word-break: break-all; }
.badge { display: inline-block; margin-left: 0.4rem; padding: 0 0.4rem; background: #555; color: #fff; font-size: 0.7rem; border-radius: 2px; }
.hidden { display: none; }
nav { padding: 0.5rem; font-size: 0.9rem; }
canvas { display: block; margin: 1rem auto; background: black; }
.hint { text-align: center; font-size: 0.85rem; opacity: 0.7; }
```

- [ ] **Step 4: Update `script.js` for the filter**

```javascript
// WS9 showcase index page — category filter.
(function() {
  const filter = document.getElementById('filter');
  const status = document.getElementById('status');
  const tiles = Array.from(document.querySelectorAll('.tile'));
  if (!filter) return;
  filter.addEventListener('input', () => {
    const q = filter.value.trim().toLowerCase();
    let shown = 0;
    for (const t of tiles) {
      const name = t.dataset.name || '';
      const hit = q === '' || name.toLowerCase().includes(q);
      t.classList.toggle('hidden', !hit);
      if (hit) shown++;
    }
    status.textContent = q ? `${shown} match` + (shown === 1 ? '' : 'es') : '';
  });
})();
```

- [ ] **Step 5: Verify locally**

```bash
cargo build -p raylib-showcase --release --examples
cargo run -p raylib-showcase --release --bin xtask-build-pages
ls showcase/_site/
```

Expected: `index.html`, `style.css`, `script.js`, `examples/<cat>/...html`, optionally `thumbnails/`. Open `showcase/_site/index.html` in a browser; the categorized list renders.

- [ ] **Step 6: Commit**

```bash
git add showcase/src/bin/xtask_build_pages.rs showcase/index/
git commit -m "$(cat <<'EOF'
feat(ws9-p4): full categorized Pages index + thumbnail integration

xtask_build_pages now emits per-category sections with thumbnail tiles
(from gen-thumbnails' _manifest.json), desktop-only placeholder pages
for wasm-excluded examples, and a vanilla-JS name filter. style.css
grows a grid layout + badge styles; script.js adds the filter handler.

Per WS9 spec §6.5 + §9 Phase 4.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4.2: Hook the per-example shell into emscripten builds

**Files:**
- Modify: `showcase/src/bin/xtask_wasm_build.rs`

- [ ] **Step 1: Pass the shell file to emscripten**

The cleanest way to use a custom shell is to set the `EMCC_CFLAGS` env var per-call. Update the `cargo build` command in `xtask_wasm_build.rs` `run`:

```rust
    for meta in &metas {
        if meta.wasm_excluded {
            continue;
        }
        let shell_path = manifest_dir.join("index/example_shell.html");
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
        let existing_cflags = std::env::var("EMCC_CFLAGS").unwrap_or_default();
        let new_cflags = format!(
            "{} --shell-file {}",
            existing_cflags,
            shell_path.display(),
        );
        cmd.env("EMCC_CFLAGS", new_cflags);
        // ... rest unchanged
    }
```

(Add `let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));` near the top of `main` if not already there.)

- [ ] **Step 2: Verify locally**

```bash
cargo run -p raylib-showcase --release --bin xtask-wasm-build
```

Then inspect one of the generated HTML files:

```bash
grep -l "back to showcase" target/wasm32-unknown-emscripten/release/examples/*.html | head -1
```

Expected: at least one match. Confirms the custom shell is in effect.

- [ ] **Step 3: Commit**

```bash
git add showcase/src/bin/xtask_wasm_build.rs
git commit -m "$(cat <<'EOF'
feat(ws9-p4): pass example_shell.html via EMCC_CFLAGS --shell-file

Wraps each per-example emscripten output with the gallery's chrome
(back nav + F1 hint). Inherits the workflow's EMCC_CFLAGS and appends
the --shell-file flag.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4.3: Book — per-chapter "See also" footers

**Files:**
- Modify: `book/src/**/*.md` (selective; chapters that document raylib feature areas)

- [ ] **Step 1: Identify chapters and their relevant examples**

The book's 28 chapters cover specific raylib feature areas. For each chapter that documents a feature (drawing, input, audio, models, shaders, textures, etc.), gather the matching example names from `showcase/examples/<cat>/`.

A subagent batch is appropriate here: dispatch a single implementer with the prompt:

> Walk `book/src/` chapters. For each chapter that documents a raylib feature area (drawing, input, audio, models, shaders, textures, raygui, …), append a "See also" footer near the end of the chapter listing the most relevant 3-6 showcase examples by name with links of the form `[<name>](https://raylib-rs.github.io/raylib-rs/examples/<cat>/<name>.html)`. Don't touch chapters about getting-started, architecture, or anything that isn't a feature area. Don't link more than 6 examples per chapter — pick the most representative.
>
> Footer format:
> ```markdown
> ## See also
>
> Showcase examples that demonstrate this chapter's feature area:
> - [<name1>](<url1>)
> - [<name2>](<url2>)
> - ...
> ```

- [ ] **Step 2: Dispatch the subagent**

Use `general-purpose` subagent.

- [ ] **Step 3: Verify book builds clean**

```bash
mdbook build book/
```

Expected: book builds; no broken-link errors.

- [ ] **Step 4: Commit**

```bash
git add book/src/
git commit -m "$(cat <<'EOF'
docs(ws9-p4): book — per-chapter "See also" showcase footers

Each feature-area chapter gets a "See also" footer linking up to 6
showcase examples on the RC gallery. URLs target dacode45.github.io/
raylib-rs/ during RC; URL flip handled by final-release.

Per WS9 spec D9 + §9 Phase 4.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4.4: Book — Examples appendix

**Files:**
- Create: `book/src/appendix-examples.md`
- Modify: `book/src/SUMMARY.md`

- [ ] **Step 1: Generate the appendix from the registry**

A small one-shot script (or a subagent with the registry as input) generates the appendix. Subagent prompt:

> Read `showcase/Cargo.toml` and extract every `[[example]] name = "..."` entry. Group by category (the parent directory under `examples/`). Write `book/src/appendix-examples.md` enumerating every example with a link to its gallery page. Format:
>
> ```markdown
> # Appendix: Showcase Examples
>
> The raylib-rs showcase ships a Rust port of every official raylib 6.0
> example. The full gallery is at
> [raylib-rs.github.io/raylib-rs](https://raylib-rs.github.io/raylib-rs/)
> during 6.0-rc; URL flips to canonical after release.
>
> Each example is runnable locally via `cargo run -p raylib-showcase --example <name>`.
> Press F1 to view the C and Rust sources side-by-side.
>
> ## audio
>
> - [audio_module_playing](https://raylib-rs.github.io/raylib-rs/examples/audio/audio_module_playing.html)
> - ...
>
> ## core
> - ...
> ```

- [ ] **Step 2: Add the entry to `book/src/SUMMARY.md`**

Append to `book/src/SUMMARY.md`:

```markdown

# Appendix

- [Showcase Examples](appendix-examples.md)
```

- [ ] **Step 3: Verify book builds clean**

```bash
mdbook build book/
```

- [ ] **Step 4: Commit**

```bash
git add book/src/appendix-examples.md book/src/SUMMARY.md
git commit -m "$(cat <<'EOF'
docs(ws9-p4): book — Examples appendix

Enumerates every showcase example by category with gallery links.
Added to SUMMARY.md as an Appendix section.

Per WS9 spec §9 Phase 4 done bar item 4.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4.5: P4 close verification

- [ ] **Step 1: Push to fork; verify Pages deploys**

```bash
git push fork 6.0-rc
```

Wait for `pages.yml` to complete. Visit `https://raylib-rs.github.io/raylib-rs/`. Verify:
- Categorized index renders with all categories.
- Filter input works.
- A few tiles have thumbnails; the rest have placeholders.
- Clicking a tile lands on the per-example page; the canvas loads.
- F1 inside the canvas toggles the source viewer (smoke test in browser).
- "Desktop only" tiles link to a placeholder page.

- [ ] **Step 2: Verify `book.yml` is still green**

Check on Actions tab.

- [ ] **Step 3: Tag P4 close**

```bash
git tag ws9-p4-complete
git push fork ws9-p4-complete
```

---

## Phase 5 — Skill, done-note, status flip

### Task 5.1: Write the repo-tracked skill mirror

**Files:**
- Create: `docs/superpowers/skills/raylib-showcase-port-flow.md`

- [ ] **Step 1: Write the skill**

```markdown
---
name: raylib-showcase-port-flow
description: Use when bumping the raylib-sys submodule to a new raylib upstream tag to detect new/removed/renamed example files and port new ones into the showcase under the visual-parity rule. Covers wasm-exclude triage, thumbnail overrides, and CI verification.
---

# raylib-showcase-port-flow

Use this skill **after** bumping `raylib-sys/raylib` (or `raylib-sys/raygui-examples`) to a new upstream tag/commit. It detects what changed in the examples tree and walks you through porting new examples into `showcase/examples/` correctly.

## When to use

- After `git submodule update --remote raylib-sys/raylib` (or `raygui-examples`).
- Before merging the submodule bump to `unstable` or `6.0-rc`.
- When `cargo build -p raylib-showcase --examples` starts failing with pairing-check errors after a submodule bump.

## Step 1 — Detect the diff

From the repo root:

```bash
# Identify the previous and new tags/commits.
cd raylib-sys/raylib && git log --oneline -2 && cd ../..

# Diff the examples tree.
git -C raylib-sys/raylib diff <prev-tag>..<new-tag> --name-status -- examples/
```

Look for:
- `A` lines → new C examples that need a Rust port.
- `D` lines → removed C examples whose Rust port + `[[example]]` entry + thumbnail/exclude entries must be deleted.
- `R` lines → renames; rename the Rust file + `[[example]]` entry.
- `M` lines on `.c` files → the C source content changed; **re-validate the existing Rust port** for visual-parity drift (the spacing/structure may have shifted).

## Step 2 — For each addition

1. Open the C source: `raylib-sys/raylib/examples/<cat>/<name>.c`.
2. Create `showcase/examples/<cat>/<name>.rs` from the `core_basic_window.rs` template:
   - Header comment block (translated from the C — same lines, same structure).
   - `use raylib::prelude::*; use raylib_showcase::SourceViewer;`
   - `fn main() { ... }` with init / loop / close visually parallel to the C.
   - **Two single-line additions** for the viewer:
     - After init: `let mut viewer = SourceViewer::for_current_example();`
     - In loop: `viewer.update(&mut rl);` (Update section) and `viewer.draw(&mut d);` (Draw section, after the example's own draws).
3. Apply the visual-parity rule (see `showcase-c-rust-port-style` memory + spec §14 Appendix). The reviewer's primary criterion.
4. Add the `[[example]]` entry to `showcase/Cargo.toml`:
   ```toml
   [[example]]
   name = "<name>"
   path = "examples/<cat>/<name>.rs"
   # add `required-features = ["raygui"]` if it's a raygui example
   ```
5. Verify: `cargo build -p raylib-showcase --example <name>` is clean.
6. Verify wasm: `cargo build -p raylib-showcase --target wasm32-unknown-emscripten --example <name>`. If it fails:
   - Look at the build error. If it's due to a raylib API that doesn't work under emscripten (anything touching file dialogs, threads, recording, drop files, etc.), add to `showcase/wasm-exclude.toml`:
     ```toml
     [[exclude]]
     name = "<name>"
     reason = "<one-line reason>"
     ```
   - Otherwise fix the port code.
7. Run the example: `cargo run -p raylib-showcase --example <name>`. Watch for:
   - Does the example show its visual within the first ~60 frames? If not, add a `thumbnails.toml` override with `frames = <higher number>`.
   - Does it use any resource files? If yes, verify they're in `showcase/resources/<cat>/...`; if not, run `cargo run -p raylib-showcase --bin xtask-vendor-resources` to refresh.
8. Commit: `feat(showcase): port <cat>/<name>` with a brief body explaining any wasm-exclude or thumbnails.toml entries.

## Step 3 — For each removal

```bash
git rm showcase/examples/<cat>/<name>.rs
# Edit showcase/Cargo.toml to remove the [[example]] entry.
# Edit showcase/wasm-exclude.toml + showcase/thumbnails.toml to remove any
#   matching entries.
cargo build -p raylib-showcase --examples
git commit -m "chore(showcase): drop <cat>/<name> (removed upstream)"
```

## Step 4 — For each rename

```bash
git mv showcase/examples/<cat>/<old>.rs showcase/examples/<cat>/<new>.rs
# Edit showcase/Cargo.toml — update both `name` and `path`.
# Update any wasm-exclude.toml / thumbnails.toml entries.
cargo build -p raylib-showcase --examples
git commit -m "chore(showcase): rename <cat>/<old> → <cat>/<new>"
```

## Step 5 — For each modification

When the upstream C source changed (`M` lines on `.c`), the visual-parity rule means **the existing Rust port may now diverge from the C structure**. Open both files side by side and patch the Rust port to match the new C shape:
- Did C add a new variable or call? Add the parallel line in Rust at the same position.
- Did C reorder phases? Mirror the new order.
- Did C remove a comment? Remove the translated comment from the Rust port too.

If the C change introduced a new API surface that's not yet wrapped in raylib-rs, follow the same fallback chain as Step 2: extend the safe API in a small PR, or `unsafe { ffi::… }` with a `// SAFETY:` comment.

## Step 6 — raygui submodule bump

If you also bumped `raylib-sys/raygui-examples/`, the raygui pin and the raygui hand-patch base (in `binding/raygui.h`) **must match** the new commit, or example expectations diverge from the bundled raygui. Re-pin both to the same commit before continuing.

## Step 7 — Run the full verification

```bash
cargo build -p raylib-showcase --examples
cargo build -p raylib-showcase --examples --features raygui
cargo run -p raylib-showcase --release --bin xtask-wasm-build
cargo nextest run -p raylib-showcase
```

All green. Push and watch `showcase.yml` + `pages.yml` complete.

## Quick reference

- Visual-parity rule: spec §14 Appendix + `showcase-c-rust-port-style` memory.
- raygui feature-gate rule: `rgui-feature-gate-rule` memory.
- F1 keybinding: the viewer reserves F1 globally. If a ported example natively uses F1, remap to F2 in the port and add a header-comment note.
- Resource paths: vendored under `showcase/resources/<cat>/...` matching the C-side path. Re-run `xtask-vendor-resources` after a raylib bump to refresh.
- Reference example: `showcase/examples/core/core_basic_window.rs` — the canonical port shape.
```

- [ ] **Step 2: Commit**

```bash
git add docs/superpowers/skills/raylib-showcase-port-flow.md
git commit -m "$(cat <<'EOF'
docs(ws9-p5): skill — raylib-showcase-port-flow

Reusable workflow skill for detecting + porting new examples on every
future raylib upstream bump. Mirrors to ~/.claude/skills/ in Task 5.2;
this is the repo-tracked source of truth.

Per WS9 spec done-criteria item 8.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5.2: Mirror the skill to `~/.claude/skills/`

**Files:**
- Create: `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` (outside repo)

- [ ] **Step 1: Create the directory + mirror the file**

```bash
mkdir -p ~/.claude/skills/raylib-showcase-port-flow
cp docs/superpowers/skills/raylib-showcase-port-flow.md ~/.claude/skills/raylib-showcase-port-flow/SKILL.md
```

- [ ] **Step 2: Verify the skill is discoverable**

In a new conversation, asking the assistant to "use the raylib-showcase-port-flow skill" should surface it via the Skill tool. (Manual verification by the owner; cannot be done from this plan directly.)

- [ ] **Step 3: No commit (outside repo)**

The mirror lives outside the repo. The repo-tracked version (Task 5.1) is the source of truth.

---

### Task 5.3: Write the done-note

**Files:**
- Create: `docs/superpowers/notes/ws9-showcase-complete.md`

- [ ] **Step 1: Gather the final stats**

Before writing, collect:

```bash
# Total examples ported
find showcase/examples -name "*.rs" | wc -l

# Per-category counts
for d in audio core models others shaders shapes text textures raygui; do
  printf "%-12s %3s\n" "$d" "$(find showcase/examples/$d -name '*.rs' 2>/dev/null | wc -l)"
done

# Wasm exclusions count
grep -c '^\[\[exclude\]\]' showcase/wasm-exclude.toml

# Thumbnail overrides count
grep -c '^\[\[example\]\]' showcase/thumbnails.toml

# Resource size
du -sh showcase/resources

# CI workflow runs (latest, on fork/6.0-rc)
gh run list --branch 6.0-rc --limit 7 --json name,conclusion,databaseId
```

- [ ] **Step 2: Write the done-note**

Create `docs/superpowers/notes/ws9-showcase-complete.md`:

```markdown
# WS9 — Showcase Finale: complete

**Status:** DONE on branch `6.0-rc` (pushed to `fork`). Eighth and final pre-release workstream. Spec: `docs/superpowers/specs/2026-05-31-ws9-showcase-design.md`. Plan: `docs/superpowers/plans/2026-06-01-ws9-showcase.md`.

## What shipped

(Populate from the Step-1 gather. Example:)

| Statistic | Before | After |
|-----------|--------|-------|
| Rust example ports | 66 (closure-shaped, broken under 6.0) | <N> (cargo [[example]] targets, visually parallel to C) |
| Categories covered | 10 partial | 9 complete (audio, core, models, others, shaders, shapes, text, textures, raygui) |
| Wasm exclusions | n/a | <M> with one-line reasons each |
| Thumbnail overrides | n/a | <K> per-example overrides |
| Vendored resources | 0 | <files>, ~<MB> MB |
| CI workflows green on fork | 5/5 | 7/7 (added `showcase`, `pages`) |
| Pages site URL | — | https://raylib-rs.github.io/raylib-rs/ |
| mdBook chapters touched | — | <N> chapters got "See also" footers + new appendix |
| New skill | — | `raylib-showcase-port-flow` at both `~/.claude/skills/` and `docs/superpowers/skills/` |

## Decisions executed

(List D1–D10 with ✅ marks; reference the spec for context.)

## Tracked-deferred (carries forward to final-release + post-release)

(Copy the spec's §12 list, removing anything that landed in WS9.)

## Lessons learned

(Implementer fills in during P5: surprises, near-misses, the few wasm-exclusions that revealed deeper issues, etc.)

## CI inventory

(Populate from the Step-1 `gh run list` output.)

## Next workstream

**final-release.** Bump `6.0.0-rc.1 → 6.0.0`, flip CHANGELOG `(unreleased)` → ISO date, PR `Dacode45:6.0-rc` → `raylib-rs:unstable`, tag `v6.0.0`, manually trigger `release-sys.yml` then `release-safe.yml` (dry-run first), then GitHub release. The Pages URL flip from `raylib-rs.github.io/raylib-rs/` to `raylib-rs.github.io/` (or canonical) is part of final-release.
```

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/notes/ws9-showcase-complete.md
git commit -m "$(cat <<'EOF'
docs(ws9-p5): done-note for WS9 showcase finale

Final coverage table + decisions + tracked-deferred carry-forwards +
CI inventory + lessons. Hands off to the final-release workstream.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5.4: Update `CHANGELOG.md`

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add the WS9 entry under the `6.0.0 (unreleased)` heading**

Open `CHANGELOG.md` and locate the `6.0.0 (unreleased)` section. Add (or extend existing) entries:

```markdown
## 6.0.0 (unreleased)

...existing entries...

### Showcase (WS9)

- The `showcase/` crate is rewritten end-to-end as a gallery of ~228 `[[example]]` targets, one Rust port per official raylib 6.0 example, all visually parallel to their C originals.
- Each example carries an in-canvas raygui source viewer (`F1`) showing the C and Rust sources side-by-side.
- New `showcase.yml` + `pages.yml` CI workflows: strict desktop build matrix + wasm-after-exclusions + GitHub Pages deploy.
- Gallery deployed to <https://raylib-rs.github.io/raylib-rs/> during 6.0-rc; canonical URL handled by final-release.
- mdBook gains per-chapter "See also" footers and a new Examples appendix.
- New skill `raylib-showcase-port-flow` documents the bump-detection + porting workflow for future raylib upgrades.
- Removed: the closure-shaped showcase launcher (`showcase/src/main.rs`) and the vendored `showcase/original/` (replaced by direct reference to the `raylib-sys/raylib/examples/` submodule).
- New submodule: `raylib-sys/raygui-examples/` (pinned to raysan5/raygui@4fbd425).
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "$(cat <<'EOF'
docs(changelog): WS9 showcase finale entry under 6.0.0 (unreleased)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5.5: Flip `CLAUDE.md` status line + final CI verification

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Locate and update the status line**

In `CLAUDE.md`, find the line that ends with `→ safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ (...) → WS9 showcase ← NEXT`. Replace `WS9 showcase ← NEXT` with `WS9 showcase ✅ (...) → final-release ← NEXT`, where `(...)` is a brief note matching the prior workstream summaries.

Concretely, change:

```
safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ (see `docs/superpowers/notes/ws-gui-icons-safe-abstractions-complete.md`) → **WS9 showcase ← NEXT** → GitHub Pages (finale) → final-release.
```

to:

```
safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ → WS9 showcase ✅ — **~228 ported examples + in-canvas C/Rust source viewer + Pages gallery at raylib-rs.github.io/raylib-rs/ + reusable port-flow skill (see `docs/superpowers/notes/ws9-showcase-complete.md`)** → **final-release ← NEXT**.
```

- [ ] **Step 2: Push everything and watch CI**

```bash
git push fork 6.0-rc
```

Verify all 7 workflows green on the fork:
- `check`, `test`, `web`, `sanitizers`, `book` (existing)
- `showcase`, `pages` (new)

Visit `https://raylib-rs.github.io/raylib-rs/` and click through 3-5 examples to confirm the deploy is correct.

- [ ] **Step 3: Final commit + tag**

```bash
git add CLAUDE.md
git commit -m "$(cat <<'EOF'
docs(ws9): flip CLAUDE.md status line — WS9 ✅ → final-release ← NEXT

Closes WS9. Eighth and final pre-release workstream of the raylib-rs
6.0 upgrade. Hands off to the final-release workstream.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc

git tag ws9-complete
git push fork ws9-complete
```

WS9 done. The next workstream (final-release) is unblocked.

---

## Self-Review

**Spec coverage check** (against spec §13 done criteria):

| # | Spec criterion | Implementing task(s) |
|---|---|---|
| 1 | Every raylib 6.0 example has a Rust port | P2 (Tasks 2.1–2.4) + P3 (Task 3.2) |
| 2 | Visual-parity per the rule | Implementer brief + reviewer brief in Task 2.0; reviewer pass each wave |
| 3 | Source viewer per example via F1 | Task 0.6 (viewer) + Task 0.8 (wired in reference example) + propagation through P2/P3 |
| 4 | `cargo build --examples` green on 3-OS | Task 0.10 (showcase.yml `build` job); verified per wave |
| 5 | `cargo build --target wasm…` minus wasm-exclude green | Task 0.10 (showcase.yml `wasm-build` job); verified per wave |
| 6 | Pages site renders the gallery | Task 0.11 (pages.yml) + Task 4.1 (full index) + Task 4.5 (verification) |
| 7 | mdBook footers + appendix | Task 4.3 + Task 4.4 |
| 8 | New skill at both paths | Task 5.1 (repo) + Task 5.2 (mirror) |
| 9 | Done-note | Task 5.3 |
| 10 | CLAUDE.md status line flipped | Task 5.5 |
| 11 | All 7 workflows green on fork | Task 0.13 (P0), 1.3 (P1), each P2/P3 wave close, Task 4.5 (P4), Task 5.5 (P5) |

All 11 covered.

**Placeholder scan:** Searched for "TBD", "TODO", "implement later", "fill in details", "appropriate error handling". The plan contains "TODO" only in the existing test.yml fold-in target (an existing comment block being preserved, not introduced by this plan), and "fill in" only in Task 5.3's gather-stats instructions for the done-note (intentional — the implementer literally fills in the numbers after running the gather commands). No problematic placeholders.

**Type consistency:** `SourceViewer::for_current_example()` (Task 0.6) is called in every example (Task 0.8 + P2/P3). `SourcePair { c, rust, category }` matches between `build.rs` (Task 0.4), `registry.rs` (Task 0.5), and `viewer.rs` (Task 0.6). `ExampleMeta { name, category, wasm_excluded }` matches between `build.rs` emits and `gen_thumbnails`/`xtask_wasm_build`/`xtask_build_pages` consumers. `EMCC_CFLAGS` value is consistent across `showcase.yml`, `pages.yml`, and the `xtask_wasm_build` --shell-file extension. Task 4.2 references a `manifest_dir` that needs to be declared in `main` if not already; noted in-line.

One known imperfection flagged inline: `Image::load_from_screen` in `viewer.rs` (Task 0.6, Step 1 note) may not exist on the safe `Image` directly. Resolution: the implementer checks the actual surface and either uses the existing API or wraps `unsafe { ffi::LoadImageFromScreen() }` with a `// SAFETY: …` comment. Documented as an inline note in Task 0.6 so the implementer doesn't proceed blind.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-01-ws9-showcase.md`.

Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration. Best fit for a workstream this size (50-100 commits across ~40 tasks).
2. **Inline Execution** — Execute tasks in this session using executing-plans, batch execution with checkpoints. Better fit if the owner wants finer-grained control of each commit.

Which approach?
