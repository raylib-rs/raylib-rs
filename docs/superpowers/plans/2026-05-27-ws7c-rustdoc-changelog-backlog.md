# WS7c — Rustdoc enrichment + CHANGELOG + backlog fold-in — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enrich rustdoc on ~25 high-traffic types/items in the `raylib` crate (`# Examples`, expanded one-paragraph summaries, `# Safety` notes where relevant), write the `CHANGELOG.md` 6.0.0 entry covering WS1–WS6 deltas, fold in two backlog doc-only PRs with attribution (#284, #273), and verify two backlog doc issues (#291, #290).

**Architecture:** Each rustdoc-enrichment target is a focused edit to an existing file under `raylib/src/`. Examples are software-renderer-runnable where possible (real doctest); window-opening examples are `no_run`. The crate-wide `deny(missing_docs)` gate already enforces that doc strings exist — this workstream is about depth, not coverage. The `cargo doc -p raylib --features full` build with `RUSTDOCFLAGS=-Dwarnings` catches broken intra-doc-links.

**Tech Stack:** Rustdoc, `cargo doc --features full`, the `software_renderer` feature for runnable doctests, `git cherry-pick` / `git commit --author` for backlog fold-ins.

**Spec:** `docs/superpowers/specs/2026-05-27-ws7-docs-and-book-design.md` §3 (WS7c), §5 (rustdoc enrichment scope), §6 (CHANGELOG seed).

**Prerequisite:** WS7b complete (book chapters landed). **Working model:** branch `6.0-rc`; push to `fork`; watch with `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` last. Subagents work in parallel per file.

---

## Doctest conventions for rustdoc examples

- **No-window / software-renderer example** → ` ```rust ` (real doctest, runs under `cargo test --doc -p raylib --features full`).
- **Window-opening example** → ` ```rust,no_run ` (compiles but does not execute; per the existing convention).
- **Type-extraction-only example** (where the snippet illustrates a type signature) → ` ```rust ` if it compiles without context, else `,ignore` with a reason comment.

After every edit:
```bash
cargo doc -p raylib --features full --no-deps   # RUSTDOCFLAGS=-Dwarnings via .cargo/config or env
cargo test --doc -p raylib --features full
```

These two commands are the per-task verifier.

---

## File structure

| File | Items to enrich | Task |
|------|-----------------|------|
| `raylib/src/lib.rs` | crate-level docs | 1 |
| `raylib/src/prelude.rs` | prelude example | 1 |
| `raylib/src/core/window.rs` | `RaylibHandle`, `RaylibThread`, `RaylibBuilder` | 2 |
| `raylib/src/core/drawing.rs` | `RaylibDraw` trait | 3 |
| `raylib/src/core/mod.rs` | `Color`, `Rectangle` | 4 |
| `raylib/src/core/texture.rs` | `Image`, `Texture2D`, `RenderTexture2D`, load methods | 5 |
| `raylib/src/core/models.rs` | `Mesh`, `Model`, `Material`, `ModelAnimations` | 6 |
| `raylib/src/core/audio.rs` | `AudioHandle`, `Wave`, `Sound`, `Music`, `AudioStream` | 7 |
| `raylib/src/core/shaders.rs` | `Shader`, locations/uniforms flow | 8 |
| `raylib/src/core/text.rs` | `Font`, `Font::load_ex` | 9 |
| `raylib/src/core/math.rs` | `Vector2`/`3`/`4`, `Matrix`, `Quaternion`, headline raymath methods | 10 |
| `raylib/src/core/collision.rs` | single family-level example | 11 |
| `raylib/src/test_harness.rs` | BGRA/Y-flip + all-5-modules link req | 12 |
| `raylib/src/rgui/mod.rs` | module-level only | 13 |
| `raylib/src/rlgl/mod.rs` | module-level only | 14 |
| `raylib/src/core/color.rs` (or wherever `Color` consts live) | **PR #284 fold-in (LBreede)** — cheatsheet docs | 15 |
| `raylib/src/core/logging.rs` | **PR #273 fold-in (AmityWilder)** — module-doc fix | 16 |
| Various | **Issue #291 sweep** — broken doc examples | 17 |
| Various | **Issue #290 sweep** — dead links | 18 |
| `CHANGELOG.md` | 6.0.0 (unreleased) entry | 19 |

---

## Task 1: `lib.rs` + `prelude.rs` enrichment

**Files:**
- Modify: `raylib/src/lib.rs:1-30` (the crate-level `//!` block)
- Modify: `raylib/src/prelude.rs:1-30` (the module-level `//!` block)

**Contract:**

`lib.rs` crate-level: 8–15 lines. Open with a one-line tagline ("Safe Rust bindings to raylib 6.0."). Follow with three short paragraphs:
1. What you get: window + input + drawing, 2D/3D, raymath, audio, raygui, rlgl, software-renderer headless harness.
2. Where to start: `raylib::init()` returns a `(RaylibHandle, RaylibThread)` tuple; everything hangs off `RaylibHandle`.
3. Feature gates: opt-in `glam`/`mint`/`serde`, `software_renderer` for headless testing, `opengl_21`/`opengl_es_20`/`drm`/`wayland` for back-end selection.

`prelude.rs` module-level: 5–10 lines describing what's in the prelude (the safe-API headline types, the drawing trait, raymath, Color, Rectangle). Add one ``` ```rust ``` (no_run) example showing `use raylib::prelude::*;` then `let (mut rl, thread) = raylib::init().size(640, 480).build();`.

- [ ] **Step 1: Read both files to find the existing `//!` blocks**

Run: `Read raylib/src/lib.rs` and `Read raylib/src/prelude.rs` (first ~30 lines each).

- [ ] **Step 2: Edit `raylib/src/lib.rs` — replace the existing `//!` block with enriched content**

Use the contract above. Keep the `#![deny(missing_docs)]` and other crate-level attributes intact.

- [ ] **Step 3: Edit `raylib/src/prelude.rs` — replace the module-level docs with enriched content + example**

Use the contract above. The example is ``` ```rust,no_run ``` because it opens a window.

- [ ] **Step 4: Verify rustdoc builds clean and doctests pass**

Run:
```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```
Expected: zero warnings/errors.

- [ ] **Step 5: Commit**

```bash
git add raylib/src/lib.rs raylib/src/prelude.rs
git commit -m "docs(ws7c): enrich crate-level and prelude rustdoc

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: `window.rs` — `RaylibHandle`, `RaylibThread`, `RaylibBuilder`

**Files:**
- Modify: `raylib/src/core/window.rs`

**Items to enrich (locate each with `Grep`):**
- `pub struct RaylibHandle` — add a paragraph + `# Examples` block.
- `pub struct RaylibThread` — paragraph (the `!Send` invariant is the headline).
- `pub struct RaylibBuilder` — paragraph + builder-fluent example.

**Example contracts:**
- `RaylibHandle`: ``` ```rust,no_run ``` showing `raylib::init().build()` + `window_should_close` + `begin_drawing`.
- `RaylibThread`: ``` ```text ``` annotation explaining `RaylibThread: !Send` (a code block that just asserts compilation isn't possible). Prose-only is fine here; if a Rust block, ``` ```rust,compile_fail ``` showing `fn require_send<T: Send>() {}; require_send::<RaylibThread>();`.
- `RaylibBuilder`: ``` ```rust,no_run ``` showing chained `.size(640, 480).title("X").vsync().build()`.

- [ ] **Step 1: Locate the three structs**

Run: `Grep -n "pub struct RaylibHandle\|pub struct RaylibThread\|pub struct RaylibBuilder" raylib/src/core/window.rs`

- [ ] **Step 2: Edit each struct's doc block per the contract above**

Preserve existing `# Safety` notes; add `# Examples` blocks below the prose paragraph.

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/window.rs
git commit -m "docs(ws7c): enrich window.rs (RaylibHandle, RaylibThread, RaylibBuilder)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: `drawing.rs` — `RaylibDraw` trait

**Files:**
- Modify: `raylib/src/core/drawing.rs`

**Items to enrich:**
- `pub trait RaylibDraw` (and any companion `RaylibDraw3D` / `RaylibDrawHandle` types) — add an extended trait-level doc explaining the immediate-mode drawing model: methods are called on the guard returned by `begin_drawing`; the guard's drop calls `EndDrawing`.

**Example contract:**
- ``` ```rust ``` software-renderer doctest using `test_harness::render_frame` to clear background and draw a rectangle; verify a pixel. (Gated on `cfg(feature = "software_renderer")` via doc test attributes — see existing patterns in `test_harness.rs`.)

- [ ] **Step 1: Locate the trait**

Run: `Grep -n "pub trait RaylibDraw" raylib/src/core/drawing.rs`

- [ ] **Step 2: Edit the trait's doc block per the contract**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/drawing.rs
git commit -m "docs(ws7c): enrich drawing.rs RaylibDraw trait

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: `core/mod.rs` — `Color`, `Rectangle`

**Files:**
- Modify: `raylib/src/core/mod.rs` (or wherever `Color`/`Rectangle` are defined in the safe crate; if they live in `raylib-sys`, the doc-enrichment task lives there instead — verify location with Grep)

**Items to enrich:**
- `Color` — paragraph explaining the RGBA8 layout + the bundled constants (`Color::RED`, etc.).
- `Rectangle` — paragraph explaining the x/y/width/height layout + the relation to `check_collision_recs`.

**Example contracts:**
- `Color`: ``` ```rust ``` doctest constructing a Color and comparing to a constant: `assert_eq!(Color::new(255, 0, 0, 255), Color::RED);`.
- `Rectangle`: ``` ```rust ``` doctest constructing two rectangles and calling a pure function (e.g., a manual containment check — collision is in `collision.rs`).

- [ ] **Step 1: Locate the types**

Run: `Grep -rn "pub struct Color\|pub struct Rectangle" raylib/src/ raylib-sys/src/`

- [ ] **Step 2: Edit the doc blocks per the contracts**

If the types live in `raylib-sys`, edit there instead; doc enrichment applies to whichever crate owns the type.

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
# adjust the path if the types live in raylib-sys
git add raylib/src/core/mod.rs
git commit -m "docs(ws7c): enrich Color and Rectangle rustdoc

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: `texture.rs` — `Image`, `Texture2D`, `RenderTexture2D`

**Files:**
- Modify: `raylib/src/core/texture.rs`

**Items to enrich:**
- `pub struct Image` — paragraph: CPU-side pixel data; loaded via `load_image`/`gen_color`/etc.; uploaded to a `Texture2D` for drawing.
- `pub struct Texture2D` — paragraph: GPU-side texture; created from an `Image` via `load_texture_from_image` (or directly via `load_texture`).
- `pub struct RenderTexture2D` — paragraph: an off-screen GPU framebuffer; drawn to via `begin_texture_mode`/end; consumed via `texture()` accessor.
- `Image::load_image` / `Image::gen_color` / `Image::gen_perlin_noise` — short paragraph each pointing at how-to.

**Example contract:**
- `Image`: ``` ```rust ``` software-renderer doctest using `Image::gen_color` + a pixel readback.
- `Texture2D`: ``` ```rust,no_run ``` showing load_texture_from_image then draw_texture.
- `RenderTexture2D`: ``` ```rust,no_run ``` showing begin_texture_mode → draw → end_texture_mode → texture().

- [ ] **Step 1: Locate the three types**

Run: `Grep -n "pub struct Image\|pub struct Texture2D\|pub struct RenderTexture2D" raylib/src/core/texture.rs`

- [ ] **Step 2: Edit each struct's doc block**

- [ ] **Step 3: Enrich the headline `Image::*` constructors with one-sentence summaries + `# Examples` only where they're load-paths**

- [ ] **Step 4: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 5: Commit**

```bash
git add raylib/src/core/texture.rs
git commit -m "docs(ws7c): enrich texture.rs (Image, Texture2D, RenderTexture2D + load methods)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: `models.rs` — `Mesh`, `Model`, `Material`, `ModelAnimations`

**Files:**
- Modify: `raylib/src/core/models.rs`

**Items to enrich:**
- `pub struct Mesh` — paragraph: per-mesh vertex/index data; mesh-accessor soundness (WS3) returns `&[T]` with C-guaranteed length.
- `pub struct Model` — paragraph: owns `Mesh` array + `Material` array; load via `load_model`.
- `pub struct Material` — paragraph: shader + texture maps.
- `pub struct ModelAnimations` — paragraph: **the headline 6.0 redesign** — RAII wrapper that replaces the manual `UnloadModelAnimations` dance from 5.x. Cross-reference `notes/ws3-complete.md` for the redesign rationale.

**Example contracts:**
- `Mesh`: ``` ```rust,no_run ``` accessing a mesh's `vertices()` accessor returning `&[Vector3]`.
- `Model`: ``` ```rust,no_run ``` loading + drawing in a frame loop.
- `ModelAnimations`: ``` ```rust,no_run ``` loading + advancing one frame.

- [ ] **Step 1: Locate the four types**

Run: `Grep -n "pub struct Mesh\|pub struct Model\|pub struct Material\|pub struct ModelAnimations" raylib/src/core/models.rs`

- [ ] **Step 2: Edit each struct's doc block**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/models.rs
git commit -m "docs(ws7c): enrich models.rs (Mesh, Model, Material, ModelAnimations)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 7: `audio.rs` — `AudioHandle`, `Wave`, `Sound`, `Music`, `AudioStream`

**Files:**
- Modify: `raylib/src/core/audio.rs`

**Items to enrich:**
- `pub struct AudioHandle` — paragraph: separate handle from `RaylibHandle`; all audio resources are lifetime-bound to it.
- `pub struct Wave` — paragraph: loaded waveform data (CPU-side); convert to `Sound` for playback.
- `pub struct Sound` — paragraph: GPU/audio-card-ready playable sample.
- `pub struct Music` — paragraph: streamed audio for longer playback.
- `pub struct AudioStream` — paragraph: low-level streaming primitive; the replacement for the removed 5.x audio callback.

**Example contract:**
- `AudioHandle`: ``` ```rust,no_run ``` showing `RaylibAudio::init_audio_device()`.
- `Sound`: ``` ```rust,no_run ``` showing load + play.
- `AudioStream`: ``` ```rust,no_run ``` showing the streaming-callback replacement pattern.

- [ ] **Step 1: Locate the five types**

Run: `Grep -n "pub struct AudioHandle\|pub struct Wave\|pub struct Sound\|pub struct Music\|pub struct AudioStream" raylib/src/core/audio.rs`

- [ ] **Step 2: Edit each struct's doc block**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/audio.rs
git commit -m "docs(ws7c): enrich audio.rs (AudioHandle, Wave, Sound, Music, AudioStream)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 8: `shaders.rs` — `Shader`

**Files:**
- Modify: `raylib/src/core/shaders.rs`

**Items to enrich:**
- `pub struct Shader` — paragraph: GLSL shader; uniform locations queried via `get_shader_location`; values set via `set_shader_value`.

**Example contract:**
- ``` ```rust,no_run ``` loading a shader from file paths, querying a uniform location, setting a value.

- [ ] **Step 1: Locate the struct**

Run: `Grep -n "pub struct Shader" raylib/src/core/shaders.rs`

- [ ] **Step 2: Edit the doc block per the contract**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/shaders.rs
git commit -m "docs(ws7c): enrich shaders.rs Shader

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 9: `text.rs` — `Font`

**Files:**
- Modify: `raylib/src/core/text.rs`

**Items to enrich:**
- `pub struct Font` — paragraph: bundled default + user-loaded via `Font::load_font` / `Font::load_font_ex` (codepoints).

**Example contract:**
- ``` ```rust,no_run ``` showing default-font draw + `load_font_ex` with codepoints.

- [ ] **Step 1: Locate the struct**

Run: `Grep -n "pub struct Font" raylib/src/core/text.rs`

- [ ] **Step 2: Edit the doc block per the contract**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/text.rs
git commit -m "docs(ws7c): enrich text.rs Font

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 10: `math.rs` — `Vector2/3/4`, `Matrix`, `Quaternion`, headline raymath methods

**Files:**
- Modify: `raylib/src/core/math.rs`
- Possibly modify: `raylib-sys/src/...` if the types are bindgen-generated and re-exported (verify with Grep first)

**Items to enrich:**
- `Vector2`, `Vector3`, `Vector4` — short paragraph each (they're trivial), one shared `# Examples` block on `Vector3`.
- `Matrix` — paragraph: row-major (raylib convention); for `glam` users, the `glam` adapter handles the column-major conversion.
- `Quaternion` — paragraph: **the headline gotcha** — C aliases `Quaternion` to `Vector4`; raylib-rs treats it as a distinct `#[repr(C)]` type. Conversion is explicit.
- Headline raymath methods: `dot`, `cross`, `normalize`, `length` on vectors; `identity`, `rotate_x`, `translate` on Matrix; `from_axis_angle` on Quaternion. One-sentence summary each.

**Example contract:**
- `Vector3`: ``` ```rust ``` doctest: `dot` + `cross` + `normalize` operations.
- `Matrix`: ``` ```rust ``` doctest constructing identity + a translation.
- `Quaternion`: ``` ```rust ``` doctest constructing from axis-angle.

- [ ] **Step 1: Locate the types in the safe crate**

Run: `Grep -n "Vector2\|Vector3\|Vector4\|^impl Matrix\|^impl Quaternion" raylib/src/core/math.rs`

The types themselves are likely defined in `raylib-sys` (bindgen-generated per WS2a); the safe crate adds methods via `impl` blocks. Enrich on whichever side carries the docs in the public-facing API.

- [ ] **Step 2: Edit the doc blocks per the contracts**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/math.rs
# add raylib-sys path if edits there were needed
git commit -m "docs(ws7c): enrich math.rs (Vector2/3/4, Matrix, Quaternion + headline ops)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 11: `collision.rs` — single family-level example

**Files:**
- Modify: `raylib/src/core/collision.rs`

**Items to enrich:**
- Module-level `//!` doc block — one paragraph explaining the family + a single ``` ```rust ``` doctest covering `check_collision_recs` + `check_collision_circles`. Per-fn docs stay at the WS6a stub.

**Example contract:**
- ``` ```rust ``` doctest constructing two `Rectangle`s and two circles; assert collision/non-collision for each.

- [ ] **Step 1: Locate the module doc block**

Run: `Read raylib/src/core/collision.rs` (first ~30 lines)

- [ ] **Step 2: Edit the `//!` block per the contract**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/core/collision.rs
git commit -m "docs(ws7c): enrich collision.rs module-level docs with family example

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 12: `test_harness.rs` — BGRA/Y-flip + all-5-modules link req

**Files:**
- Modify: `raylib/src/test_harness.rs`

**Items to enrich:**
- Module-level `//!` doc — three paragraphs:
  1. What the harness is for: windowless rendering tests using `software_renderer` + the `Platform::Memory` backend.
  2. The **BGRA / Y-flip readback quirk** — the readback returns top-left RGBA after the harness handles the Y-flip internally; callers don't need to know unless they're poking the underlying ffi.
  3. The **all-5-modules link requirement** — the harness links rcore/rshapes/rtextures/rtext/rmodels; missing any of these in the feature set will produce link errors. Reference `notes/ws5-complete.md` for the cfg-gating that landed.

**Example contract:**
- ``` ```rust ``` doctest gated on `cfg(feature = "software_renderer")` — `init_test_harness(256, 256)`, draw a red rect, read center pixel, assert RED.

- [ ] **Step 1: Read the existing test_harness.rs**

Run: `Read raylib/src/test_harness.rs` (first ~50 lines)

- [ ] **Step 2: Edit the `//!` block per the contract**

Preserve any existing examples; append the BGRA/Y-flip + link-req paragraphs.

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/test_harness.rs
git commit -m "docs(ws7c): enrich test_harness.rs module docs (BGRA/Y-flip + link req)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 13: `rgui/mod.rs` — module-level enrichment

**Files:**
- Modify: `raylib/src/rgui/mod.rs` (or whichever file owns the `//!` block for the rgui module)

**Items to enrich:**
- Module-level `//!` doc — paragraph: raygui is an immediate-mode GUI from raylib; raylib-rs's wrapper hits 6.0 parity (57 functions, grouped sub-traits, `impl AsRef<str>` + thread-local scratch buffer). Reference `notes/ws5-complete.md`. Per-fn docs stay at WS6a stub.

**Example contract:**
- ``` ```rust ``` software-renderer doctest using `test_harness::render_gui` to draw a button.

- [ ] **Step 1: Locate the rgui module entry**

Run: `Grep -n "^//!" raylib/src/rgui/mod.rs`

- [ ] **Step 2: Edit the `//!` block per the contract**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/rgui/mod.rs
git commit -m "docs(ws7c): enrich rgui module-level docs

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 14: `rlgl/mod.rs` — module-level enrichment

**Files:**
- Modify: `raylib/src/rlgl/mod.rs`

**Items to enrich:**
- Module-level `//!` doc — paragraph: rlgl is raylib's immediate-mode OpenGL abstraction; raylib-rs exposes safe `RlMatrix`/`RlImmediate` RAII guards + bind helpers. Per-fn docs stay at WS6a stub.

**Example contract:**
- ``` ```rust ``` software-renderer doctest using `test_harness::render_rlgl` to draw a vertex stream.

- [ ] **Step 1: Locate the rlgl module entry**

Run: `Grep -n "^//!" raylib/src/rlgl/mod.rs`

- [ ] **Step 2: Edit the `//!` block per the contract**

- [ ] **Step 3: Verify**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

- [ ] **Step 4: Commit**

```bash
git add raylib/src/rlgl/mod.rs
git commit -m "docs(ws7c): enrich rlgl module-level docs

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 15: Fold PR #284 (LBreede, color.rs cheatsheet docs) with attribution

**Files:**
- Modify: `raylib/src/core/color.rs` (or wherever `Color` lives — likely `raylib-sys` per WS2; cross-check with Task 4)

The PR: https://github.com/raylib-rs/raylib-rs/pull/284 — "updated docstrings in color.rs to match the raylib cheatsheet" by LBreede, branch `unstable`.

The PR predates the 6.0 work; the safe-side color.rs may have moved. Fall back to a hand-apply if cherry-pick conflicts.

- [ ] **Step 1: Fetch the PR branch and inspect the commit(s)**

```bash
git fetch origin pull/284/head:pr-284
git log --oneline pr-284 ^6.0-rc
git show pr-284 --stat
```

Capture the original author from `git log pr-284 --pretty=format:'%an <%ae>' -1`.

- [ ] **Step 2: Attempt cherry-pick**

```bash
git cherry-pick pr-284
```

Expected outcomes:
- **Clean** — proceed to Step 4.
- **Conflict** — note which files; abort with `git cherry-pick --abort` and proceed to Step 3 (hand-apply).

- [ ] **Step 3 (only if conflict): Hand-apply with attribution preserved**

```bash
# Extract the patch
git format-patch -1 pr-284 --stdout > /tmp/pr-284.patch

# Inspect the patch — find the doc-comment changes
cat /tmp/pr-284.patch

# Apply the doc changes manually with Edit tool on whichever file owns Color/its constants
# (see Task 4 for the resolved file path)

# Stage and commit with the original author preserved
git add raylib-sys/src/...   # or wherever Color lives
git commit --author="LBreede <LBreede@users.noreply.github.com>" -m "docs(ws7c): fold PR #284 — color.rs cheatsheet docs

Cherry-pick of LBreede's docstring updates aligning color constants with
the raylib cheatsheet. Hand-applied because the PR's color.rs has since
been refactored in WS2/WS3.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

Use `git log unstable -- raylib/src/core/color.rs` or the PR's GitHub page to confirm the author's email if the noreply form above doesn't match.

- [ ] **Step 4 (if clean cherry-pick): amend the message to add the Co-Authored-By line**

```bash
git commit --amend --no-edit  # keeps original author intact; add the Co-Authored-By manually if not already in the commit
# If editing the message:
git commit --amend -m "<original PR message>

Folded from PR #284 (raylib-rs/raylib-rs).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

- [ ] **Step 5: Verify**

```bash
git log -1 --format="%an <%ae>"   # expect LBreede attribution
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

---

## Task 16: Fold PR #273 (AmityWilder, logging.rs module-doc) with attribution

**Files:**
- Modify: `raylib/src/core/logging.rs` (or wherever `logging.rs` lives)

The PR: https://github.com/raylib-rs/raylib-rs/pull/273 — "Fix module doc comment in `logging.rs`" by AmityWilder, branch `outer-logging-comment`. One-line doc fix.

- [ ] **Step 1: Fetch the PR branch**

```bash
git fetch origin pull/273/head:pr-273
git log --oneline pr-273 ^6.0-rc
git show pr-273 --stat
```

- [ ] **Step 2: Attempt cherry-pick**

```bash
git cherry-pick pr-273
```

- [ ] **Step 3 (if conflict): hand-apply with author preserved**

The fix is one line; locate the line per `git show pr-273` and apply it manually with `Edit`. Then:

```bash
git add raylib/src/core/logging.rs
git commit --author="AmityWilder <AmityWilder@users.noreply.github.com>" -m "docs(ws7c): fold PR #273 — fix logging.rs module doc comment

Cherry-pick of AmityWilder's one-line module-doc fix.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

- [ ] **Step 4 (if clean cherry-pick): amend the message to add Co-Authored-By**

```bash
git commit --amend -m "<original message>

Folded from PR #273 (raylib-rs/raylib-rs).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

- [ ] **Step 5: Verify**

```bash
git log -1 --format="%an <%ae>"   # expect AmityWilder attribution
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

---

## Task 17: Issue #291 sweep — broken doc examples

Backlog: https://github.com/raylib-rs/raylib-rs/issues/291 — "Documentation recommends broken API in latest release (5.5.1)".

The crate-wide `deny(missing_docs)` + `RUSTDOCFLAGS=-Dwarnings` in `check.yml` already catches API-drift breakage in rustdoc examples. The issue may already be resolved by WS6a — verify.

- [ ] **Step 1: Read the issue and identify the specific broken examples**

Run: `gh issue view 291 -R raylib-rs/raylib-rs` (if `gh` access available; otherwise paste the URL into a WebFetch or inspect via the GitHub web UI).

- [ ] **Step 2: Verify the examples cited still compile under the current code**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
cargo test --doc -p raylib --features full
```

If both succeed, the issue is resolved-by-WS6a — proceed to Step 3 to annotate. If they fail, fix the examples per the rustdoc-enrichment work done in Tasks 1–14.

- [ ] **Step 3: Close the issue (or leave a comment if not the maintainer)**

If you have repo-maintainer access, close #291 with a comment:
> Resolved by raylib-rs 6.0 work (WS6a's `RUSTDOCFLAGS=-Dwarnings cargo doc` gate + WS7c rustdoc enrichment). All rustdoc examples now compile under the current API surface.

If not the maintainer, leave a comment with the same content recommending closure.

If the issue is **not** resolved (examples still fail), fix the specific broken examples in the relevant files, run the verify commands again, and commit:

```bash
git add raylib/src/...
git commit -m "docs(ws7c): fix broken rustdoc examples cited in issue #291

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 18: Issue #290 sweep — dead links

Backlog: https://github.com/raylib-rs/raylib-rs/issues/290 — "Documentation links broken".

The intra-doc-link checker (`RUSTDOCFLAGS=-Dwarnings cargo doc`) catches broken `[Foo]` links automatically. Broken raw URLs (`https://...`) must be checked manually.

- [ ] **Step 1: Read the issue to identify which links**

Run: `gh issue view 290 -R raylib-rs/raylib-rs` or inspect via web UI.

- [ ] **Step 2: Grep for raw `https://` links in rustdoc and the book**

```bash
# raw https links in rustdoc
grep -rn "https://" raylib/src/ --include="*.rs" | grep -v "// " | head -50
# raw https links in the book
grep -rn "https://" book/src/ --include="*.md"
```

Manually check a sample of the URLs (the most likely-stale ones: links to upstream raylib pages, to docs.rs, to community resources). If you find dead links, fix them — usually replace with the current canonical URL.

- [ ] **Step 3: Verify intra-doc-links are clean**

```bash
RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps
```

Expected: no `unresolved link` warnings.

- [ ] **Step 4: Commit any fixes**

```bash
git add raylib/src/... book/src/...
git commit -m "docs(ws7c): fix dead links cited in issue #290

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

- [ ] **Step 5: Close (or recommend-close) the issue**

Same pattern as Task 17 Step 3.

---

## Task 19: `CHANGELOG.md` — 6.0.0 (unreleased) entry

**Files:**
- Modify: `CHANGELOG.md`

Insert a `## 6.0.0 (unreleased)` block above the existing `## 5.7.0`. Use the seed from spec §6 as the starting structure; fill specifics from the completion notes.

**Source-of-truth for specifics:**
- `docs/superpowers/notes/ws1-breakage-baseline.md` + `ws1-config-reconcile.md` — sys parity + config flag changes.
- `docs/superpowers/notes/ws3-complete.md` — safe API parity, `MintVec*` deprecation, `ModelAnimations` redesign, Tier-1 tests.
- `docs/superpowers/notes/ws4b-complete.md` — `software_renderer` feature + test_harness.
- `docs/superpowers/notes/ws5-complete.md` — raygui rework, safe rlgl, test_harness normalization.
- `docs/superpowers/notes/ws6-prep-complete.md` + `ws6a-complete.md` + `ws6b-complete.md` — layered CI, quality hard-gates, deferred-PR fold-ins, web build-verify, sanitizers.

- [ ] **Step 1: Read each completion note and extract the per-WS public-facing deltas**

For each note, list:
- Breaking changes (API removed, renamed, signature changed)
- Added items (new types, functions, features, modules)
- Fixed items (with PR/issue references)

Keep notes in a scratch buffer; the spec-§6 seed organizes them.

- [ ] **Step 2: Read the current `CHANGELOG.md` to confirm format**

Run: `Read CHANGELOG.md` (the existing 5.7.0 entry sets the bullet-per-change pattern).

- [ ] **Step 3: Write the 6.0.0 entry**

Insert above the existing `## 5.7.0`:

```markdown
## 6.0.0 (unreleased)

Upgrade from raylib 5.x to **raylib 6.0**. MSRV bumped to **1.85** (edition 2024).

### Highlights
- raylib C source bumped to 6.0; bindings regenerated.
- Math types are now native `#[repr(C)]` Rust structs (`Vector2/3/4`, `Matrix`, `Quaternion`) with zero math-crate dependencies by default. `mint`, `glam`, `serde` are opt-in features.
- Skeletal-animation API redesigned around RAII (`ModelAnimations`).
- New `software_renderer` feature wires raylib's `rlsw` backend for headless rendering.
- raygui at 6.0 parity; new safe immediate-mode `rlgl` module.
- Layered CI: `check.yml` / `test.yml` / `web.yml` / `sanitizers.yml` / `book.yml`; quality hard-gates enforce.
- mdBook docs at `book/` (build-only in CI; public deploy lands in WS9).

### Breaking
- MSRV is now 1.85 (edition 2024).
- `MintVec2`/`MintVec3`/`MintVec4` deprecated; use the native types + opt-in `mint`.
- `glam`/`mint`/`serde` are no longer default-on for `raylib-sys`; enable explicitly.
- raygui module split into grouped sub-traits; signatures use `impl AsRef<str>` + a thread-local scratch buffer.
- Skeletal-animation loading returns a `ModelAnimations` RAII wrapper.
- Removed: <list per WS1/WS3 — pull from completion notes; the audio callback is one>

### Added
- `software_renderer` feature + `raylib::test_harness` module.
- Safe `rlgl` module: `rl_begin`/`rl_draw`, `RlMatrix` RAII guard, bind helpers.
- New 6.0 file-system and text symbols (per WS1).
- Tier-1 unit tests for raymath, collision, color, file/text parsing.
- Tier-2 headless rendering tests via `software_renderer`.
- `full` feature alias.
- `deny.toml` (cargo-deny license allowlist + RUSTSEC advisories).
- `book/` mdBook with quickstart, build guides, core concepts, and per-module chapters.

### Fixed
- Mesh-accessor soundness fixes (#257 / #118 / #256 with attribution).
- Removed unsound `AsRef`/`AsMut` impls on pointer-owning wrappers (#277, partial).
- `c"..."` literal modernization (#272).
- Idiom: `Into` → `From` (#268), CStr literal usage (#266).
- Docs: `color.rs` cheatsheet alignment (#284), `logging.rs` module-doc (#273), broken doc examples (#291), dead links (#290).

### Deferred (tracked, not in 6.0.0)
- Full PR #277 wrapper-soundness refactor.
- `get_gamepad_button_pressed` `transmute` (input.rs) — small soundness fix.
- `raylib-test` delete-or-fix decision (revisit at WS9).
- `rlsw` on wasm32 (currently a `compile_error!` for `software_renderer` + emscripten).
- UBSAN through the FFI boundary (linker-runtime work).
- `structopt` → `clap`, `paste` alternative (release-hygiene at WS8).

### Internal
- Long-lived `6.0-rc` branch with single merge to `unstable` at WS8.
```

Fill the `Removed:` bullet from the WS1/WS3 notes. Add any per-WS specifics that surface during the read in Step 1.

- [ ] **Step 4: Verify the entry renders correctly**

Visual: open `CHANGELOG.md` and confirm Markdown rendering looks reasonable. If a Markdown linter is part of the project, run it.

- [ ] **Step 5: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(ws7c): CHANGELOG.md — 6.0.0 (unreleased) entry covering WS1-WS6

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 20: Final push + watch + WS7 sign-off

- [ ] **Step 1: Push everything**

```bash
git push fork 6.0-rc
```

- [ ] **Step 2: Watch all relevant workflows**

```bash
gh run list -R Dacode45/ms-raylib-rs --limit 10
# Watch the latest book.yml run
gh run watch <book-id> -R Dacode45/ms-raylib-rs --exit-status
# Watch the latest check.yml run (rustdoc gate)
gh run watch <check-id> -R Dacode45/ms-raylib-rs --exit-status
```

Expected: both green. `check.yml`'s `docs` job verifies the rustdoc-with-warnings-as-errors path; `book.yml` verifies the book + book doctests.

- [ ] **Step 3: Write `docs/superpowers/notes/ws7-complete.md`**

Mirror the `ws6b-complete.md` shape:
- Status (DONE on branch `6.0-rc`, pushed to `fork`).
- One section per workstream sub-piece (WS7a / WS7b / WS7c) with commit hashes and the proof-it-works summary.
- WS7 done-criteria sign-off against spec §1.
- Tracked-deferred carried out of WS7 (the §9 list from the WS7 spec).
- Closing: "WS7 done. Next: WS8 (release & merge) — version bump, `release.yml`, `cargo publish` for `raylib-sys` then `raylib`, single merge of `6.0-rc` into `unstable`."

- [ ] **Step 4: Commit the completion note**

```bash
git add docs/superpowers/notes/ws7-complete.md
git commit -m "docs(ws7): WS7 complete — book + rustdoc + CHANGELOG done; next WS8

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
```

- [ ] **Step 5: Update `CLAUDE.md` — WS7 ✅, next is WS8**

Read the existing `CLAUDE.md` workstream-status line; replace `WS7 docs & book ← NEXT` with `WS7 ✅ — <one-line summary of what shipped>; WS8 release ← NEXT`.

```bash
git add CLAUDE.md
git commit -m "docs(ws7): mark WS7 complete in CLAUDE.md status line

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
```

- [ ] **Step 6: WS7 sign-off checklist (mirror spec §10)**

- [ ] `book/` exists with all 28 markdown files.
- [ ] `mdbook build book` green.
- [ ] `mdbook test book -L target/debug/deps` green.
- [ ] `book.yml` green on the fork.
- [ ] `CHANGELOG.md` has the `## 6.0.0 (unreleased)` entry covering WS1–WS6.
- [ ] ~25 rustdoc items from spec §5 enriched; `RUSTDOCFLAGS=-Dwarnings cargo doc` green.
- [ ] #284 + #273 folded with `--author` attribution preserved.
- [ ] #291 + #290 verified (closed or annotated).
- [ ] `Cargo.toml` versions still say `5.7.0` (WS8's job).
- [ ] Book is NOT deployed (WS9's job).
- [ ] Tracked-deferred list updated in `ws7-complete.md`.

**WS7 done.** Next: WS8 (release & merge).
