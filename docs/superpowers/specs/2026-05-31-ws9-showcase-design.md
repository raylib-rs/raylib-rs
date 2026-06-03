# WS9 — Showcase Finale: Design Spec

**Status:** draft (brainstorm-approved 2026-05-31; awaiting user spec review)
**Workstream:** WS9 — the eighth and final pre-release workstream of the raylib-rs 6.0 upgrade
**Branch:** `6.0-rc` (on `fork = Dacode45/ms-raylib-rs`); no canonical merge until final-release
**Prerequisite workstreams:** WS0 → WS8 ✅; pre-WS9 queue (pixel-pointers → gui-icons) ✅
**Next workstream:** final-release (crates.io publish, canonical merge, GitHub release)

---

## 1. Summary

Ship a runnable Rust port of every official raylib 6.0 example (218 raylib core + ~10-15 raygui = ~228-233 total), each with a built-in side-by-side C↔Rust source viewer rendered in-canvas via raygui, deployed as a GitHub Pages gallery at `dacode45.github.io/raylib-rs/`. The showcase is the publishable artifact for raylib-rs 6.0 — the landing surface that demonstrates the binding is alive, current, and at upstream parity. The source viewer is the central UX differentiator: newcomers can see how each raylib API call translates to its Rust equivalent without leaving the example.

At workstream completion, a new reusable skill at `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` documents how to detect new upstream raylib examples on future raylib version bumps and how to properly port them — making future bumps cheap.

## 2. Context

The pre-WS9 queue is fully shipped (`pixel-pointers` → `hashes` → `mixed-audio` → `raylib-test` → UBSAN → rustdoc-rewrite → gui-icons + PR #296). raylib-rs 6.0 is feature-complete on the safe-crate side. What remains before final-release is the publishable demonstration: a gallery of working examples that prove the binding compiles, runs, and shows what idiomatic raylib-rs code looks like.

The owner directive at WS9 kickoff added a new requirement on top of the roadmap's "port all examples" target: **every example must carry both its original C source and its Rust port at runtime, with a UI toggle for side-by-side viewing.** The viewer is the central pedagogical hook of the gallery ("see exactly how this looks in raylib C vs raylib-rs"). A `build.rs` step bundles each pair into the example.

A new visual-parity rule (captured in the `showcase-c-rust-port-style` memory) was locked at kickoff: Rust ports must mirror the C source's spacing, blank-line groupings, comment positions, and init/loop/close structure line-by-line. Rust-only ergonomics (RAII drop, `&str` vs `*const c_char`, `Result`/`?`, native math operators) are surfaced via inline comments rather than refactored into idiomatic Rust shapes that diverge from the C. This rule directly drives the per-example `[[example]]` architecture — the existing closure-launcher in `showcase/src/main.rs` is structurally incompatible with the C's `InitWindow → while !WindowShouldClose → CloseWindow` shape, so the launcher gets deleted.

## 3. Inventory at WS9 start

Verified 2026-05-31 on branch `6.0-rc` HEAD `e3e9ed9`:

| Surface | Count | Notes |
|---|---|---|
| Vendored C examples (`showcase/original/`) | 125 | Curated 5.x-era subset; **deleted in WS9 P0** in favor of the raylib-sys submodule as source of truth. |
| Upstream raylib C examples (`raylib-sys/raylib/examples/`) | 218 | Tracks raylib 6.0 tag. 8 categories: `audio` (11), `core` (49), `models` (30), `others` (3), `shaders` (35), `shapes` (41), `text` (16), `textures` (32), `+ examples_template.c` (1 excluded). |
| raygui upstream examples (raysan5/raygui) | ~10-15 | Added in P0 as new submodule at `raylib-sys/raygui-examples/`. |
| Existing Rust ports (`showcase/src/example/`) | 66 | All closure-shaped (`run(rl, thread) -> Box<dyn FnMut>`); incompatible with visual-parity rule; **rewritten in P2** under the new `examples/` architecture. |
| Existing ports wired into launcher | 56 | Old `showcase/src/main.rs` `gui_list_view_ex` launcher; **deleted in P0**. |
| Showcase Cargo.toml version | 5.5.1 | Bumped to `6.0.0-rc.1` in P0. |
| Showcase in root workspace? | No | Added to `Cargo.toml` `members` in P0. |
| Existing CI workflows green on `fork/6.0-rc` | 5/5 | `check`, `test`, `web`, `sanitizers`, `book`. |

## 4. Locked decisions

The 10 brainstorm decisions:

| # | Decision | Choice |
|---|---|---|
| D1 | Architecture | Per-example `[[example]]` targets (`cargo examples/`, **not** `[[bin]]`). Each example a self-contained `fn main()` with explicit `init → loop → close`. |
| D2 | Porting scope | Full upstream 6.0 parity: all 218 raylib core examples + the ~10-15 raygui examples = **~228-233 total**. |
| D3 | Source-viewer UX | In-canvas raygui overlay. Toggle via `F1`; tab buttons swap C/Rust; PgUp/PgDn scroll. Uniform across desktop + wasm. |
| D4 | Source pairing | `showcase/build.rs` walks the raylib-sys submodule + raygui-examples submodule, emits `$OUT_DIR/source_registry.rs` (a `phf` map keyed by example name) embedded into every binary. **`showcase/original/` is deleted; submodule is single source of truth.** |
| D5 | Wasm scope | Best-effort + explicit `showcase/wasm-exclude.toml` list. Excluded examples ship desktop + appear on the Pages site with a "desktop only" badge. |
| D6 | raygui examples | **Include.** Add ~10-15 raygui upstream examples to the porting set. |
| D7 | Asset handling | **Vendor every upstream resource** to `showcase/resources/<cat>/...` preserving directory structure so example load paths match the C originals (path-level visual parity). License attribution captured in `showcase/resources/README.md`. |
| D8 | Pages deploy | Deploy from `fork/6.0-rc` to `dacode45.github.io/raylib-rs/` during RC. At final-release, deploy target flips to canonical (handled by the final-release workstream). |
| D9 | Book integration | Cross-link only. Per-chapter "See also: showcase examples" footers + new `book/src/appendix-examples.md`. No live wasm canvases in chapters (deferred). |
| D10 | CI gating | Strict desktop (3-OS matrix, gates merges) + strict wasm-after-exclusions (gates merges). From Phase 0. |

Sub-decisions locked by recommendation in this spec (not raised as separate brainstorm questions):

| Sub# | Decision | Choice |
|---|---|---|
| S1 | Workspace inclusion | Add `"showcase"` to root `Cargo.toml` `members`. |
| S2 | Version bump | `5.5.1 → 6.0.0-rc.1`. |
| S3 | raygui originals vendoring | New submodule at `raylib-sys/raygui-examples/` pinned to a stable raysan5/raygui commit (alongside the existing raygui hand-patch base; CI check that both pins match). |
| S4 | Resource location | `showcase/resources/<cat>/...` mirroring upstream resource subtrees. |
| S5 | Thumbnail strategy | Vendored upstream `<name>.png` where present; otherwise generated by `showcase/src/bin/gen_thumbnails.rs` running each non-excluded example under `software_renderer` for a configurable frame count (default 60, per-example override in `showcase/thumbnails.toml`). Capture logic lives inside `SourceViewer::update` gated on `RAYLIB_SHOWCASE_THUMBNAIL_FRAMES` env var. |
| S6 | Visual-parity rule | Human-enforced via reviewer pass in P2/P3 (no static checker). Documented in the new skill (P5 artifact). |
| S7 | Registry encoding | `phf::Map` via `phf_codegen` build-time codegen. Fallback to `&[(&str, SourcePair)]` + binary search if `phf_codegen` benchmark too slow (unlikely at 228 entries). |
| S8 | Skill mirror | Skill content lives at `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` (user-trigger location) **and** at `docs/superpowers/skills/raylib-showcase-port-flow.md` (repo-tracked source of truth). |

## 5. Surface & scope

**WS9 owns:**
- `showcase/` — full rewrite (delete `src/main.rs`, delete `src/example/`, delete `showcase/original/`; add `build.rs`, `src/lib.rs`, `src/viewer.rs`, `src/registry.rs`, `src/bin/gen_thumbnails.rs`, `src/bin/xtask-wasm-build.rs`, `src/bin/xtask-build-pages.rs`, `examples/<cat>/<name>.rs` × ~228, `wasm-exclude.toml`, `thumbnails.toml`, `index/template.html`, `index/style.css`, `index/script.js`, `resources/<cat>/...`).
- `raylib-sys/raygui-examples/` — new submodule.
- `.github/workflows/showcase.yml` — new.
- `.github/workflows/pages.yml` — new.
- `book/src/**/*.md` — per-chapter footer additions + new `appendix-examples.md`.
- Root `Cargo.toml` — `members += ["showcase"]`.
- `CHANGELOG.md` — WS9 entry under the `6.0.0 (unreleased)` heading.
- `CLAUDE.md` — status line flip at P5.
- `docs/superpowers/notes/ws9-showcase-complete.md` — P5 done-note.
- `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` + `docs/superpowers/skills/raylib-showcase-port-flow.md` — P5 skill artifact.

**Out of scope:**
- Any change to `raylib/` (the safe crate) beyond bug fixes surfaced by porting.
- Any new raygui safe-API work beyond what porting requires (e.g., a missing helper for a single example).
- Any change to `raylib-sys/` beyond vendoring raygui examples + the resources tree.
- crates.io publish (final-release workstream).
- Canonical merge (final-release workstream).
- Pages URL flip from fork → canonical (final-release workstream).
- bevy-raylib crate (post-release).
- Live wasm canvases in mdBook chapters.
- Per-example screenshot regression tests beyond the 5-example thumbnail smoke.
- HTML-chrome source viewer for web-only.
- macOS / Windows UBSAN coverage.
- SR doctest leg re-enable in `test.yml` (blocks on pre-existing failures unrelated to showcase).

## 6. Architecture

### 6.1 Directory layout (target state)

```
showcase/
  Cargo.toml                  # 6.0.0-rc.1; auto-examples = false; explicit [[example]] entries × ~228
  build.rs                    # walks raylib-sys/raylib/examples + raygui-examples; emits OUT_DIR/source_registry.rs + OUT_DIR/examples_meta.json
  wasm-exclude.toml           # curated list of examples that can't wasm-build, with one-line reason each
  thumbnails.toml             # optional per-example frame-count overrides + skip flags
  resources/                  # vendored upstream resources, preserving directory structure
    audio/, core/, models/, shaders/, shapes/, text/, textures/, others/, raygui/
    README.md                 # license attribution per file
  examples/                   # one Rust file per upstream C example, nested by category
    audio/audio_module_playing.rs
    core/core_basic_window.rs
    core/core_2d_camera.rs
    ...
    raygui/controls_test_suite.rs
    ...
  src/
    lib.rs                    # crate-internal library: viewer, registry re-exports, harness helpers
    viewer.rs                 # in-canvas raygui SourceViewer overlay
    registry.rs               # thin re-export of OUT_DIR/source_registry.rs
    bin/
      gen_thumbnails.rs       # subprocess-spawning thumbnail generator
      xtask-wasm-build.rs     # iterates examples × wasm-exclude.toml, runs `cargo build --target wasm32-unknown-emscripten --example <name>`
      xtask-build-pages.rs    # reads examples_meta.json + _manifest.json, populates template.html, assembles _site/
  index/
    template.html             # Pages index page template (Tera-style placeholders)
    style.css                 # gallery styling (vanilla CSS, no framework)
    script.js                 # category filter + per-example launch (vanilla JS)
```

### 6.2 Per-example file shape

Each `examples/<cat>/<name>.rs` is a self-contained, single-file program visually parallel to its C counterpart:

```rust
//! raylib [core] example - basic window
//! Ported from raylib-sys/raylib/examples/core/core_basic_window.c
//! Visual-parity rule applies — see showcase-c-rust-port-style.

use raylib::prelude::*;
use raylib_showcase::viewer::SourceViewer;

fn main() {
    // Initialization
    //----------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - basic window")
        .build();
    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        // Update
        viewer.update(&mut rl);

        // Draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text("Congrats! You created your first window!", 190, 200, 20, Color::LIGHTGRAY);
        viewer.draw(&mut d);
    }
    // CloseWindow is handled by RAII drop of `rl`.
}
```

Two single-line additions versus the C:
1. `let mut viewer = SourceViewer::for_current_example();` after init.
2. `viewer.update(&mut rl)` + `viewer.draw(&mut d)` inside the loop.

`SourceViewer::for_current_example()` resolves `env!("CARGO_BIN_NAME")` (= the `[[example]] name`) against the registry; no per-example wiring.

### 6.3 `build.rs` registry

The build script:

1. Walks `raylib-sys/raylib/examples/<cat>/*.c` + `raylib-sys/raygui-examples/examples/*.c`.
2. For each C source, locates the paired `showcase/examples/<cat>/<name>.rs` (or `examples/raygui/<name>.rs`).
3. Emits `$OUT_DIR/source_registry.rs` containing a `phf::Map<&'static str, SourcePair>` keyed by example name, with `SourcePair { c: &'static str, rust: &'static str, category: &'static str }`. Both strings are `include_str!`-loaded at codegen time.
4. Emits `$OUT_DIR/examples_meta.json` (read by `xtask-build-pages`, not at example runtime) with `[{name, category, wasm_excluded}]`.
5. Emits `cargo:rerun-if-changed=` for every walked C/Rust file, plus `wasm-exclude.toml` and `thumbnails.toml`.
6. **Hard error** on mismatch: every C source must have a paired Rust file (or be on a documented exception list, e.g. `examples_template.c`); every Rust example must have a paired C source (no orphans). This is the primary correctness check.

### 6.4 Source-viewer overlay (`src/viewer.rs`)

Public API:

```rust
pub struct SourceViewer { /* visible, scroll_y, active_tab (C|Rust), pair: &'static SourcePair */ }

impl SourceViewer {
    pub fn for_current_example() -> Self;
    pub fn for_example(name: &str) -> Self;
    pub fn update(&mut self, rl: &mut RaylibHandle);
    pub fn draw<D: RaylibDraw>(&self, d: &mut D);
}
```

**Behavior:**

- Off by default. `F1` toggles visibility. When closed: a small "📄 F1: view source" hint button is drawn in the bottom-right corner.
- When visible: full-screen translucent panel; two raygui tab buttons (`C` | `Rust`); manually-laid-out scrollable text region (since raygui has no native multi-line scrollable widget — we render line-by-line via `d.draw_text` with viewport clipping and a `scroll_y` offset).
- `PgUp`/`PgDn` scroll by ~10 lines; `Home`/`End` jump to top/bottom; `Tab` swaps the C/Rust tab.
- Monospace font (raylib's default is fine; fallback to LiberationMono if needed).
- Selectable: no. Editable: no. Text is read-only.

**Hidden thumbnail-capture branch:** inside `update`, if `RAYLIB_SHOWCASE_THUMBNAIL_FRAMES` env var is set, the viewer increments a frame counter; once it reaches the configured value, it captures the framebuffer using the WS4 `software_renderer` readback pattern (BGRA, Y-flipped → RGBA), encodes a PNG to the path in `RAYLIB_SHOWCASE_THUMBNAIL_OUT`, then calls `std::process::exit(0)`. Zero per-example surface for thumbnails — every example calls `viewer.update` anyway.

### 6.5 Pages site

`xtask-build-pages` reads `examples_meta.json` (registry metadata) + `_manifest.json` (thumbnail results), populates `index/template.html` with a categorized table of examples, writes per-example shell pages by wrapping each emscripten-generated `<name>.html` with the gallery's chrome (header, breadcrumb, source-viewer toggle hint, link back to index). Output to `showcase/_site/`. `actions/deploy-pages@v4` uploads.

URL during RC: `https://dacode45.github.io/raylib-rs/`. The per-example URL is `https://dacode45.github.io/raylib-rs/examples/<cat>/<name>.html`. Banner on the RC site: *"Preview build of raylib-rs 6.0-rc — final docs at raylib-rs.github.io after release."*

## 7. Components

| Component | Path | Owns |
|---|---|---|
| Workspace root | `Cargo.toml` | Adds `"showcase"` to `members`. |
| Showcase manifest | `showcase/Cargo.toml` | Version `6.0.0-rc.1`; `auto-examples = false`; explicit `[[example]]` entries; `[[bin]]` entries for the three xtask binaries; feature flags mirroring raylib's (`software_renderer`, `raygui`, etc.); `phf` + `phf_codegen` build-deps. |
| Build script | `showcase/build.rs` | Registry + meta-JSON generation; rerun-if-changed; pairing validation. |
| Lib crate | `showcase/src/lib.rs` | Re-exports: `viewer::SourceViewer`, `registry::{EXAMPLES, ExampleMeta, lookup}`. |
| Source viewer | `showcase/src/viewer.rs` | `SourceViewer` struct + behavior; thumbnail-capture branch. |
| Registry shim | `showcase/src/registry.rs` | `include!` of the OUT_DIR registry; `lookup(name)` helper. |
| Thumbnail tool | `showcase/src/bin/gen_thumbnails.rs` | Subprocess spawning, `_manifest.json` writing. |
| Wasm build tool | `showcase/src/bin/xtask-wasm-build.rs` | Iterates examples × `wasm-exclude.toml`, runs per-example wasm build, collects failures + summary. |
| Pages build tool | `showcase/src/bin/xtask-build-pages.rs` | Index generator: reads metadata, populates template, copies wasm shells + thumbnails + style/script into `_site/`. |
| Example files | `showcase/examples/<cat>/<name>.rs` | Self-contained `fn main()`, visually parallel to C. |
| Wasm exclude list | `showcase/wasm-exclude.toml` | `[[exclude]] name = "<n>" reason = "<r>"` entries. |
| Thumbnail overrides | `showcase/thumbnails.toml` | Optional `[[example]] name = "<n>" frames = <N>` and/or `skip = true`. |
| Pages templates | `showcase/index/{template.html,style.css,script.js}` | Vanilla HTML/CSS/JS; no framework. |
| Resources | `showcase/resources/<cat>/...` | Vendored upstream resources; `README.md` with license attribution per file. |
| Showcase CI | `.github/workflows/showcase.yml` | 3-OS matrix: `cargo build --examples` (strict, gates) + nextest unit tests; Linux-only wasm-build (strict-after-exclusions, gates). |
| Pages CI | `.github/workflows/pages.yml` | Build wasm → run `gen-thumbnails` → run `xtask-build-pages` → deploy to Pages. |
| raygui examples vendor | `raylib-sys/raygui-examples/` | New submodule; CI check that its pin matches the raygui hand-patch base. |
| Book cross-links | `book/src/**/*.md` (existing) | Per-chapter footer additions. |
| Book Examples appendix | `book/src/appendix-examples.md` | New file enumerating all examples by category with gallery URLs. |
| Done-note | `docs/superpowers/notes/ws9-showcase-complete.md` | P5 artifact. |
| Skill (user-trigger) | `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` | The reusable porting workflow. |
| Skill (repo-tracked) | `docs/superpowers/skills/raylib-showcase-port-flow.md` | Source-of-truth mirror of the skill. |

## 8. Data flows

### Flow A — Build-time (example compilation)

1. `cargo build --example core_basic_window` triggers `showcase/build.rs`.
2. `build.rs` walks raylib-sys/raylib/examples + raygui-examples; pairs with `showcase/examples/<cat>/<name>.rs`.
3. For each pair, the C and Rust files are `include_str!`-loaded at codegen and appended to a `phf_codegen` map at `$OUT_DIR/source_registry.rs`.
4. `build.rs` emits `$OUT_DIR/examples_meta.json` for the index generator.
5. `cargo:rerun-if-changed=` per walked file (+ TOMLs).
6. `cargo` compiles `core_basic_window.rs` against `raylib_showcase`. `SourceViewer::for_current_example()` resolves `env!("CARGO_BIN_NAME")` → registry lookup at runtime.
7. Final binary embeds both sources as static `&'static str`.

### Flow B — Runtime (desktop, source-viewer toggle)

1. User runs `cargo run --example core_basic_window`.
2. `fn main()` constructs `RaylibHandle`, builds viewer, enters `while !rl.window_should_close()`.
3. Per frame: `viewer.update(&mut rl)` checks `KEY_F1` → toggles `visible`; if visible, also processes scroll/tab/jump keys.
4. `d.clear_background(...)` → example draws → `viewer.draw(&mut d)`.
5. If `!visible`: only the bottom-right hint button.
6. If `visible`: full-screen overlay with C/Rust tabs and scrollable text.

### Flow C — Runtime (wasm)

Same as Flow B. Each example owns its own emscripten init via raylib's normal cross-platform path (`emscripten_set_main_loop` driven from `while !window_should_close` is handled by raylib-rs's existing wasm shim). Resources preloaded via emscripten `--preload-file showcase/resources` flag injected by `build.rs` when `target_arch = "wasm32"`.

### Flow D — Thumbnail generation (Pages CI)

1. `xtask-build-pages` invokes `gen-thumbnails` after wasm build.
2. `gen-thumbnails` reads `EXAMPLES` from the registry + `thumbnails.toml` overrides.
3. For each non-excluded, non-`skip` example: spawn `cargo run --example <name> --features software_renderer --release` with env vars `RAYLIB_SHOWCASE_THUMBNAIL_FRAMES=<N>` + `RAYLIB_SHOWCASE_THUMBNAIL_OUT=target/thumbnails/<cat>_<name>.png`.
4. Inside that subprocess: `SourceViewer::update` checks env vars per frame; at frame `N`, captures framebuffer, encodes PNG, exits.
5. 30s subprocess timeout per example; failures get logged with reason → placeholder.
6. Writes `target/thumbnails/_manifest.json` for `xtask-build-pages` to consume.

### Flow E — Pages deploy

1. Push to `fork/6.0-rc` triggers `pages.yml` (gates on `showcase.yml` success).
2. Job: checkout (with submodules) → install emscripten → `xtask-wasm-build` → `gen-thumbnails` → `xtask-build-pages` → upload `_site/` → `actions/deploy-pages@v4`.
3. Deployed to `dacode45.github.io/raylib-rs/`. Per-example shells at `/examples/<cat>/<name>.html`.

### Flow F — Future raylib bump (the skill's flow)

1. Maintainer bumps `raylib-sys/raylib` submodule to a new tag (and updates the raygui hand-patch base + the `raygui-examples` submodule if raygui changed).
2. Runs the skill: `~/.claude/skills/raylib-showcase-port-flow/SKILL.md`.
3. Skill walks them through:
   a. `git diff <prev-tag>..<new-tag> -- examples/` to surface added/removed/renamed examples.
   b. For each addition: scaffold `showcase/examples/<cat>/<name>.rs` from the reference template, port C → Rust applying the visual-parity rule, add `[[example]]` entry to `showcase/Cargo.toml`, verify `cargo build --example <name>`, decide wasm-exclude.
   c. For each rename: rename Rust file + `[[example]]` entry; verify build.
   d. For each removal: delete Rust file + `[[example]]` entry + any thumbnail override.
4. Skill includes a checklist: viewer wired? Resources vendored? wasm-exclude considered? thumbnails.toml needed? CI green?

## 9. Phasing

### P0 — Scaffolding & end-to-end smoke (sequential; blocks all subsequent phases)

Add `"showcase"` to root workspace; bump version `5.5.1 → 6.0.0-rc.1`; delete `showcase/src/main.rs` and `showcase/src/example/`; delete `showcase/original/`; add `raylib-sys/raygui-examples/` submodule; implement `showcase/build.rs` + registry generation; implement `src/lib.rs`, `src/viewer.rs`, `src/registry.rs`; implement `src/bin/gen_thumbnails.rs` skeleton + the env-var thumbnail-capture branch in `viewer.update`; implement `src/bin/xtask-wasm-build.rs` skeleton; implement `src/bin/xtask-build-pages.rs` skeleton; define `wasm-exclude.toml` + `thumbnails.toml` schemas; port the reference example (`examples/core/core_basic_window.rs`); add `[[example]]` entry; wire `.github/workflows/showcase.yml` (desktop strict + wasm strict-after-exclusions); wire `.github/workflows/pages.yml` skeleton with template/style/script; fold-in: add `integration_rgui_icons` to `test.yml` raygui leg.

**Done bar:**
- `cargo build --examples` green on 3-OS in `showcase.yml`.
- `cargo run --example core_basic_window` shows the window; F1 toggles the source viewer with C and Rust tabs working.
- `cargo build --target wasm32-unknown-emscripten --example core_basic_window` succeeds.
- `pages.yml` deploys a single-example gallery to `dacode45.github.io/raylib-rs/`.
- All 7 workflows green on `fork/6.0-rc` at P0 close (`check`, `test`, `web`, `sanitizers`, `book`, `showcase`, `pages`).

### P1 — Vendor resources (can overlap with P2's first wave)

Walk `raylib-sys/raylib/examples/<cat>/resources/` + `raylib-sys/raygui-examples/examples/resources/` → copy to `showcase/resources/<cat>/...` preserving paths. Write `showcase/resources/README.md` capturing per-file license attribution. CI check that every upstream resource path is mirrored in `showcase/resources/`.

**Done bar:**
- Every upstream resource file present at the mirrored path in `showcase/resources/`.
- `showcase/resources/README.md` exists and lists every file's license.
- `cargo run --example` for any resource-loading example finds its resources at the expected path.

### P2 — Port raylib core examples (2-at-a-time parallel batches)

Four waves of 2 categories each:

| Wave | Category A | Category B | Total |
|---|---|---|---|
| W1 | `audio` (11) | `others` (3) | 14 |
| W2 | `core` (49) | `text` (16) | 65 |
| W3 | `models` (30) | `shaders` (35) | 65 |
| W4 | `shapes` (41) | `textures` (32) | 73 |

Per wave: 2 parallel implementer dispatches → 2 reviewer passes for visual-parity compliance → CI verification → push, before starting the next wave. Categories with >30 examples (`core`, `shaders`, `shapes`, `textures`) may further split into 2 sub-batches inside their wave if implementer context grows unwieldy. Reviewer rejects anything that violates the `showcase-c-rust-port-style` rule.

Implementer responsibilities per example:
1. Port C → Rust applying visual parity.
2. Add `[[example]]` entry to `showcase/Cargo.toml`.
3. Run `cargo build --example <name>` and `cargo build --target wasm32-unknown-emscripten --example <name>`.
4. If wasm build fails → add to `wasm-exclude.toml` with reason.
5. If default 60-frame thumbnail capture is unrepresentative (animations, late-loading models) → add `thumbnails.toml` override.
6. If the example needs safe-API surface that's missing → either extend the API in a small PR within the same batch, use `unsafe { ffi::... }` with `// SAFETY:` comment, or escalate to spec amendment if it's a large surface.

**Done bar:**
- All 218 raylib core examples build (`cargo build --examples` green on 3-OS).
- Non-excluded examples wasm-build.
- `showcase.yml` green.

### P3 — Port raygui examples (single batch, sequential after P2)

Port the ~10-15 raygui upstream examples under `showcase/examples/raygui/<name>.rs`. Apply `#[cfg(feature = "raygui")]` gates per the `rgui-feature-gate-rule` memory. Each gets its `[[example]]` entry with `required-features = ["raygui"]`.

**Done bar:**
- All raygui examples build under `cargo build --examples --features raygui`.
- `showcase.yml` green (with `raygui` feature added to the build matrix).

### P4 — Pages polish + book cross-links (sequential)

- `xtask-build-pages` produces a full categorized index page with search/filter (vanilla JS), per-example shells, "desktop only" badges per `wasm-exclude.toml`, thumbnails (full `gen-thumbnails` run in Pages CI).
- Book: per-chapter "See also: showcase examples" footers; `book/src/appendix-examples.md` enumerating all examples by category with gallery URLs.
- Book CI verification (existing `book.yml` stays green).

**Done bar:**
- Pages site at `dacode45.github.io/raylib-rs/` renders the full ~228-example gallery with categories, thumbnails, and per-example pages.
- Book builds with footers + appendix; `book.yml` + `pages.yml` green.

### P5 — Skill, done-note, status flip (sequential, final)

- Author `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` documenting Flow F (detection, scaffolding, porting checklist, wasm-exclude triage, thumbnail decisions, CI verification). Mirror to `docs/superpowers/skills/raylib-showcase-port-flow.md`. README points at both.
- Write `docs/superpowers/notes/ws9-showcase-complete.md`: final coverage table (per-category counts), exclusion list with reasons, tracked-deferred items, CI inventory.
- Flip `CLAUDE.md` workstream status line: `safe-abstractions for GuiGetIcons/GuiLoadIcons + PR #296 ✅ → WS9 showcase ✅ → final-release ← NEXT`.
- Final all-7-workflows-green verification on `fork/6.0-rc`.

**Done bar:**
- All 7 workflows green on `fork/6.0-rc`.
- CLAUDE.md flipped.
- WS9 done-note exists.
- New skill exists at both paths.
- Next workstream (final-release) is unblocked.

## 10. Testing & CI

### 10.1 What gets verified, by layer

| Layer | Mechanism | Verifies |
|---|---|---|
| `build.rs` validation | Hard error during build | Every C example has a paired Rust file (or documented exception); every Rust example has a paired C source. Registry compiles. |
| Unit tests (Tier-1) | `cargo nextest run -p raylib-showcase` | `registry::lookup` round-trip; `wasm-exclude.toml` parses; `thumbnails.toml` parses; default frame count is 60. No window opens. |
| Compile gate (desktop) | `cargo build --examples` on 3-OS | Every example compiles against raylib 6.0. |
| Compile gate (wasm) | `cargo build --target wasm32-unknown-emscripten --examples` minus `wasm-exclude.toml` | Every wasm-eligible example builds. |
| `gen-thumbnails` smoke | 5-example subset thumbnail-rendered on Linux | Thumbnail pipeline doesn't regress. Full run only in `pages.yml`. |
| Source-viewer integration | One Tier-2 test: construct `SourceViewer::for_example("core_basic_window")` headless, simulate F1 + PgDn, assert state changes | Viewer input wiring is correct. No pixel test. |
| Visual-parity rule | **Reviewer pass per P2/P3 batch.** | Human-enforced. |
| Pages site smoke | `pages.yml` runs end-to-end on every push to `6.0-rc` | Deploy doesn't break. |

### 10.2 `showcase.yml`

```yaml
name: showcase
on:
  push:
    branches: [6.0-rc, unstable]
  pull_request:
    branches: [6.0-rc, unstable]

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@1.85.0
      - run: cargo build -p raylib-showcase --examples
      - run: cargo build -p raylib-showcase --examples --features raygui
      - run: cargo nextest run -p raylib-showcase

  wasm-build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@1.85.0
        with:
          targets: wasm32-unknown-emscripten
      - uses: mymindstorm/setup-emsdk@v14
        with:
          version: 3.1.50  # pinned; matches web.yml
      - run: cargo run -p raylib-showcase --bin xtask-wasm-build --release
```

### 10.3 `pages.yml`

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

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@1.85.0
        with:
          targets: wasm32-unknown-emscripten
      - uses: mymindstorm/setup-emsdk@v14
        with:
          version: 3.1.50
      - run: cargo run -p raylib-showcase --bin xtask-wasm-build --release
      - run: cargo run -p raylib-showcase --bin gen-thumbnails --features software_renderer --release
      - run: cargo run -p raylib-showcase --bin xtask-build-pages --release
      - uses: actions/upload-pages-artifact@v3
        with:
          path: showcase/_site
      - id: deployment
        uses: actions/deploy-pages@v4
```

### 10.4 CI inventory after WS9 ships

| Workflow | Was | Becomes |
|---|---|---|
| `check.yml` | fmt + clippy + cargo-deny | unchanged |
| `test.yml` | nextest unit + integration + SR | unchanged (SR doctest leg still deferred) — fold-in adds `integration_rgui_icons` to raygui leg |
| `web.yml` | wasm-build raylib + raylib-sys | unchanged (showcase wasm goes in `showcase.yml`) |
| `sanitizers.yml` | UBSAN + ASAN over FFI | unchanged |
| `book.yml` | mdBook build | unchanged; book content gains appendix + footers |
| `showcase.yml` | — | **new:** desktop strict (3-OS) + wasm strict-after-exclusions |
| `pages.yml` | — | **new:** RC URL deploy on push to `6.0-rc` |

Total: 7 workflows green on `fork/6.0-rc` at WS9 done.

### 10.5 What's explicitly not tested

- Visual regression of the Pages site.
- Per-example screenshot diffs beyond the 5-example thumbnail smoke.
- Wasm runtime behavior (browser-side smoke). Wasm compile-passing is the gate; "actually runs in a browser" is verified by the human deploying to the RC URL.
- `gen-thumbnails` full-run in PR CI (only the 5-example smoke is PR-gated; full run is `pages.yml`-only).
- Cross-OS wasm build (Linux only — matches `web.yml` convention).

## 11. Risks

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Some raylib 6.0 examples rely on safe-API surface we haven't wrapped (raymath, rlgl, raygui multi-line text) | Medium | Blocks individual examples | Per-example: extend the safe API (small surface, fold into the example's PR), use `unsafe { ffi::... }` with `// SAFETY:` comment, or wasm-exclude with reason. Reviewer decides per case. Big surfaces escalate to spec amendment. |
| Visual-parity rule conflicts with idiomatic Rust often enough that porting stalls in debate | Medium | Slows P2 | New skill (P5) documents 5-10 worked examples of "parity preserved" + "ergonomics via comment." Reviewer applies the rule mechanically; debates escalate to owner without blocking adjacent batches. |
| `raylib-sys/raygui-examples/` submodule pin drifts vs the raygui hand-patch base | Low | Confusing test failures | Pin both to the same commit; CI check that they match. Document in the new skill. |
| Emscripten upstream churn breaks the wasm leg mid-WS9 | Medium | Blocks `showcase.yml` + `pages.yml` | Pin emscripten via `setup-emsdk@v14` with explicit `version:` field. Track failures in tracked-deferred if unfixable. |
| `gen-thumbnails` subprocess races / hangs / GPU context conflicts on CI | Medium | Blocks `pages.yml` | 30s subprocess timeout per example; failures → placeholder (don't fail workflow). Run thumbnail-gen only in `pages.yml`, never PR CI. |
| `phf_codegen` build-time cost grows large with 228 entries | Low | Slower `cargo build` | Benchmark in P0. Fall back to sorted slice + binary search if too slow. Likely a non-issue at this size. |
| Pages site at `dacode45.github.io/raylib-rs/` confuses users vs eventual canonical URL | Low | Minor user confusion | RC banner explaining preview state. URL flip handled in final-release. |
| `Cargo.toml` `[[example]]` entries (228 stanzas) make manifest noisy | Low | Manifest readability | Accept the noise; flat `examples/<cat>_<name>.rs` naming would break the visual mirror with raylib-sys. Optional `xtask-regen-cargo-toml` bin in tracked-deferred. |
| Resource vendoring blows the repo size | Medium | Slower `git clone`; possible LFS needed | Estimate after P1. If >100 MB, evaluate git-lfs. Expected <50 MB (raylib resources are mostly small textures + short audio loops). |
| New skill at `~/.claude/skills/` lives outside the repo, drifts vs repo state | Low | Skill drift | Mirror at `docs/superpowers/skills/raylib-showcase-port-flow.md` (source of truth); README points at both. |

## 12. Tracked-deferred

### Carry-forward from prior workstreams (status in WS9)

| Item | WS9 disposition |
|---|---|
| SR doctest leg re-enable in `test.yml` | Deferred. Blocks on unrelated pre-existing failures. → post-release. |
| `ease.rs::back_in_out` + `expo_in_out` math bugs | Deferred. → post-release. |
| PR #277 wrapper-soundness refactor | Deferred. → post-release. |
| `get_gamepad_button_pressed` transmute UB | Deferred. → post-release. |
| `structopt → clap`, `paste` rewrite/swap | Deferred. → post-release. |
| macOS / Windows UBSAN coverage | Deferred. → post-release. |
| bevy-raylib crate | Deferred. → post-release (owner's stated intent). |
| `gui_load_style` silent-failure pre-validation | Deferred. → post-release. |
| `GuiLoadIconsFromMemory` upstream multi-call leak | Deferred. Documented in wrapper rustdoc; needs upstream fix. |
| Next raygui re-vendor cleanup | Deferred. Re-evaluate when raysan5/raygui cuts a release. |
| `integration_rgui_icons` in `test.yml` raygui leg | **Folded into P0.** One-line CI addition. |
| `showcase` Cargo version bump `5.5.1 → 6.0.0-rc.1` | **Folded into P0.** |
| `nobuild` CI matrix expansion | Deferred. |
| Color `From&`-vs-`Clone` audit | Deferred. |
| `thiserror` migration crate-wide | Deferred. |
| `DataBuf` + `Mesh` testing umbrella | Deferred. |
| `rlgl` coverage audit | Deferred. |

### New from WS9 (carries forward to final-release or post-release)

| Item | Disposition |
|---|---|
| HTML-chrome source viewer for web-only | Deferred. In-canvas viewer is uniform; HTML chrome is a possible post-release polish. |
| Live wasm canvases in mdBook chapters | Deferred. Cross-link-only chosen for WS9. |
| Per-example screenshot regression tests | Deferred. Beyond the 5-example thumbnail smoke. |
| `[[example]]` manifest auto-regenerator | Deferred. Build only if needed. |
| Pages URL flip from fork → canonical | Deferred to final-release workstream. |
| Resource attribution audit (CC-BY items) | **Folded into P1** (README at `showcase/resources/README.md`). |

## 13. Done criteria (the WS9 done bar)

1. Every raylib 6.0 example (218 raylib core + ~10-15 raygui) has a Rust port at `showcase/examples/<cat>/<name>.rs`, **or** is on a documented exception list in the spec/skill with a one-line reason.
2. Every Rust port is visually parallel to its C original per the `showcase-c-rust-port-style` rule.
3. Every example displays its (C source, Rust source) pair via the raygui in-canvas viewer (`F1` toggle, C/Rust tabs, scroll).
4. `cargo build --examples` succeeds on the 3-OS CI matrix in `showcase.yml`.
5. `cargo build --target wasm32-unknown-emscripten --examples` minus `wasm-exclude.toml` succeeds in `showcase.yml`.
6. GitHub Pages site at `dacode45.github.io/raylib-rs/` renders the full ~228-example gallery: categorized index, per-example pages with embedded canvas + viewer, thumbnails, "desktop only" badges per `wasm-exclude.toml`.
7. mdBook has per-chapter "See also" footers and an Examples appendix; `book.yml` green.
8. New skill exists at `~/.claude/skills/raylib-showcase-port-flow/SKILL.md` (and mirrored at `docs/superpowers/skills/raylib-showcase-port-flow.md`) documenting bump-detection + porting workflow.
9. Done-note at `docs/superpowers/notes/ws9-showcase-complete.md` with final coverage table, exclusion list, tracked-deferred, CI inventory.
10. `CLAUDE.md` workstream status line flipped to `WS9 showcase ✅ → final-release ← NEXT`.
11. All 7 CI workflows green on `fork/6.0-rc` at WS9 close.

## 14. Appendix: the visual-parity rule

(Mirrored from the `showcase-c-rust-port-style` memory; applied by reviewers in P2/P3.)

Rust ports in `showcase/examples/` should be visually parallel to their `raylib-sys/raylib/examples/<cat>/<name>.c` counterparts — same blank-line groupings, same comment placement, same function/loop ordering, same variable-introduction order. If the idiomatic Rust shape would diverge, keep the parallel structure and call out the more-ergonomic alternative in a `// In idiomatic Rust this would be…` comment rather than refactoring away from the C layout.

**Application rules:**
- Open `raylib-sys/raylib/examples/<cat>/<name>.c` alongside the Rust file; keep line numbers visually aligned.
- Preserve C's blank-line groupings and comment positions verbatim (translate comment text; keep its location).
- Don't collapse C's explicit init / update / draw phases into Rust closures or builders.
- Don't replace C's index-based `for` loops with Rust iterator chains just for style; if iterators read better, leave the parallel `for` and add a single-line `// idiomatic: foo.iter().map(...).collect()` comment.
- Rust-only ergonomics worth a comment: RAII drop vs explicit `Unload*`, `&str` vs `*const c_char`, `Result`/`?` vs raw return-code checks, native math operator overloads vs raymath function calls.
- This is *visual parity*, not *semantic transliteration* — still use the safe Rust API (`RaylibHandle`, `&mut RaylibDrawHandle`, etc.); just keep its *layout* aligned with the C.

The two single-line viewer wiring additions (`SourceViewer::for_current_example()` after init; `viewer.update + viewer.draw` inside the loop) are exempt from the parity rule.
