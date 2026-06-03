# WS7b — Book chapters (Core Concepts + Modules + Ecosystem) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fill the 21 placeholder chapters left by WS7a (5 Core Concepts + 14 Modules + 2 Ecosystem) with narrative + worked examples + cross-references. Every chapter's Rust code block compiles under `mdbook test`; window-opening examples use ` ```rust,no_run `.

**Architecture:** One markdown file per chapter under `book/src/{core-concepts,modules,ecosystem}/`. Each chapter follows the shape defined below (Chapter template). After each chapter file is added, replace its draft entry in `book/src/SUMMARY.md` with the file path so mdBook links it into the TOC.

**Tech Stack:** mdBook, the `raylib` crate at `--features full`, ` ```rust ` doctest blocks linked via `mdbook test -L target/debug/deps`.

**Spec:** `docs/superpowers/specs/2026-05-27-ws7-docs-and-book-design.md` §3 (WS7b), §4 (book layout).

**Prerequisite:** WS7a complete (book.yml green, getting-started chapters landed). **Working model:** branch `6.0-rc`; push to `fork`; watch with `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` last. Subagents work in parallel where chapters are independent; the controller verifies `mdbook test` after every merge.

---

## Chapter template (the contract every chapter satisfies)

Each chapter file follows this shape. The wording is the author's; the structure is mandatory.

```markdown
# <Chapter title>

<One-paragraph intro: what this module/concept is for; which raylib-rs types/traits represent it; why a reader would land on this page.>

## API surface

<Bulleted list of headline types/functions with links to docs.rs rustdoc — e.g.
`[Image]: https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html`. 5–12 bullets;
not exhaustive — point readers at rustdoc for the rest.>

## Example

<One worked example. Software-renderer-runnable where possible (real ```rust doctest);
window-opening otherwise (```rust,no_run). Imports + types + the headline API call(s).
Minimum viable demonstration of the chapter's topic.>

## Gotchas

<2–5 bullets, ideally pulled from the WS3/WS5/WS6 completion notes. Examples:
the !Send invariant; raymath's Quaternion aliasing Vector4 in C; MintVec* deprecation;
RAII teardown order; lifetime-bound audio resources. Empty list is acceptable for
genuinely trivial modules.>

## See also

<Cross-references: 2–4 links to related chapters and to docs.rs.>
```

**Length target:** 300–700 words per chapter. Quality > word-count.

**Doctest discipline:**
- Software-renderer-friendly snippet → ` ```rust ` (real doctest).
- Window-opening snippet → ` ```rust,no_run `.
- Pseudocode or output-only snippets → ` ```text `.
- Never use ` ```rust,ignore ` unless there's a concrete reason (e.g., requires an asset file). If used, the chapter must explain why in a footnote.

---

## File structure

| File | Section | Task |
|------|---------|------|
| `book/src/core-concepts/handle-and-thread.md` | Core Concepts | 1 |
| `book/src/core-concepts/raii-and-resources.md` | Core Concepts | 2 |
| `book/src/core-concepts/strings-and-allocs.md` | Core Concepts | 3 |
| `book/src/core-concepts/safety.md` | Core Concepts | 4 |
| `book/src/core-concepts/features.md` | Core Concepts | 5 |
| `book/src/modules/window-and-drawing.md` | Modules | 6 |
| `book/src/modules/input.md` | Modules | 7 |
| `book/src/modules/shapes.md` | Modules | 8 |
| `book/src/modules/textures-and-images.md` | Modules | 9 |
| `book/src/modules/text-and-fonts.md` | Modules | 10 |
| `book/src/modules/3d-models.md` | Modules | 11 |
| `book/src/modules/audio.md` | Modules | 12 |
| `book/src/modules/raymath.md` | Modules | 13 |
| `book/src/modules/collision.md` | Modules | 14 |
| `book/src/modules/raygui.md` | Modules | 15 |
| `book/src/modules/rlgl.md` | Modules | 16 |
| `book/src/modules/software-renderer.md` | Modules | 17 |
| `book/src/modules/callbacks-and-logging.md` | Modules | 18 |
| `book/src/modules/error-handling.md` | Modules | 19 |
| `book/src/ecosystem/glam-mint-serde.md` | Ecosystem | 20 |
| `book/src/ecosystem/what-next.md` | Ecosystem | 21 |
| `book/src/SUMMARY.md` (modify, repeatedly) | TOC | every task |

Every task ends with: build check (`cargo build -p raylib --features full && mdbook build book && mdbook test book -L target/debug/deps`), update SUMMARY.md, commit.

---

## Task 1: `core-concepts/handle-and-thread.md`

**Files:**
- Create: `book/src/core-concepts/handle-and-thread.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Handle and thread]()` with `- [Handle and thread](./core-concepts/handle-and-thread.md)`.

**Chapter contract (specific to this topic):**
- **Intro:** `RaylibHandle` is the entry point for almost every raylib operation; it's returned (with `RaylibThread`) by `raylib::init().build()`. Together they enforce single-init and proper window teardown.
- **API surface bullets:** `RaylibHandle`, `RaylibThread`, `RaylibBuilder`, `raylib::init()`, `RaylibHandle::window_should_close`, `RaylibHandle::begin_drawing`. Link each to docs.rs.
- **Example:** the canonical `init() -> loop -> draw` shape, ``` ```rust,no_run ```. Minimal hello-window similar to quickstart but used to illustrate the handle-thread split (show that `&thread` is passed into `begin_drawing`).
- **Gotchas:**
  - `RaylibThread` is `!Send` — never put it in a `Mutex`/`Arc`/`thread::spawn` payload.
  - You cannot construct a second `RaylibHandle` while the first is live — `init()` panics.
  - Window teardown happens when `RaylibHandle` is dropped; if you `mem::forget` it, raylib's GL context leaks.
- **See also:** `core-concepts/raii-and-resources.md`, `modules/window-and-drawing.md`, docs.rs `RaylibHandle`.

- [ ] **Step 1: Write `book/src/core-concepts/handle-and-thread.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — link the chapter.**

Replace `- [Handle and thread]()` with `- [Handle and thread](./core-concepts/handle-and-thread.md)`.

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

Expected: clean build, zero doctest failures.

- [ ] **Step 4: Commit:**

```bash
git add book/src/core-concepts/handle-and-thread.md book/src/SUMMARY.md
git commit -m "docs(ws7b): core-concepts/handle-and-thread chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: `core-concepts/raii-and-resources.md`

**Files:**
- Create: `book/src/core-concepts/raii-and-resources.md`
- Modify: `book/src/SUMMARY.md` — replace `- [RAII and resources]()` with `- [RAII and resources](./core-concepts/raii-and-resources.md)`.

**Chapter contract:**
- **Intro:** Every raylib resource is a Rust struct that cleans up on drop. `Unload*` is never exposed.
- **API surface bullets:** `Image`, `Texture2D`, `RenderTexture2D`, `Font`, `Mesh`, `Shader`, `Material`, `Model`, `ModelAnimations` — link each to docs.rs.
- **Example:** ``` ```rust ``` software-renderer doctest that loads a generated image (`Image::gen_color`) and the image is dropped at end-of-scope. Show explicit `drop()` of a texture to demonstrate ordered teardown.
- **Gotchas:**
  - Lifetime-bound audio resources: `Wave`, `Sound`, `Music`, `AudioStream` are bound to `AudioHandle` — see audio chapter.
  - raylib-allocated buffers wrap as `ManuallyDrop<Box<[T]>>` and free via the matching `Unload*` (see `DECISIONS.md`) — never `libc::free`.
  - `Model` owns its `Mesh` and `Material` arrays; do not separately drop them.
- **See also:** `core-concepts/handle-and-thread.md`, `modules/textures-and-images.md`, `modules/audio.md`.

- [ ] **Step 1: Write `book/src/core-concepts/raii-and-resources.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [RAII and resources]()` with `- [RAII and resources](./core-concepts/raii-and-resources.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/core-concepts/raii-and-resources.md book/src/SUMMARY.md
git commit -m "docs(ws7b): core-concepts/raii-and-resources chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: `core-concepts/strings-and-allocs.md`

**Files:**
- Create: `book/src/core-concepts/strings-and-allocs.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Strings and allocations]()` with `- [Strings and allocations](./core-concepts/strings-and-allocs.md)`.

**Chapter contract:**
- **Intro:** raylib is a C API, so strings cross an FFI boundary. raylib-rs takes `&str` and returns `String` for most operations; per-frame gui draw fns take `&CStr` to avoid allocations.
- **API surface bullets:** the `rstr!` macro, `&CStr` vs `&str` parameter conventions, the `impl AsRef<str>` + thread-local scratch buffer pattern in raygui (WS5).
- **Example:** ``` ```rust ``` (no window needed) showing `rstr!("hello")` constructing a `&CStr` literal.
- **Gotchas:**
  - Per-frame gui calls use `&CStr` for zero-alloc — use `rstr!` for compile-time literals.
  - `OsStr::to_string_lossy` does NOT replace embedded nul bytes (backlog #220); raylib-rs validates input but be aware on the C side.
  - raylib-allocated buffers: `ManuallyDrop<Box<[T]>>` — see `DECISIONS.md`.
- **See also:** `core-concepts/safety.md`, `modules/raygui.md`, `modules/text-and-fonts.md`.

- [ ] **Step 1: Write `book/src/core-concepts/strings-and-allocs.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Strings and allocations]()` with `- [Strings and allocations](./core-concepts/strings-and-allocs.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/core-concepts/strings-and-allocs.md book/src/SUMMARY.md
git commit -m "docs(ws7b): core-concepts/strings-and-allocs chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: `core-concepts/safety.md`

**Files:**
- Create: `book/src/core-concepts/safety.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Safety]()` with `- [Safety](./core-concepts/safety.md)`.

**Chapter contract:**
- **Intro:** raylib-rs is a safe wrapper over a C library; `unsafe` is used sparingly and every `unsafe fn` has a `# Safety` doc, every `unsafe { ... }` block has a `// SAFETY:` comment (see `CONTRIBUTE.md`).
- **API surface bullets:** how raylib-rs uses `unsafe`; the wrapping conventions for RAII resources; the no-`Send`/`Sync` types (`RaylibThread`).
- **Example:** ``` ```rust ``` (or `text`) excerpt showing a `# Safety` doc on an `unsafe fn` and a `// SAFETY:` block — pulled from the actual source. Example: from `core/window.rs` or `core/audio.rs`.
- **Gotchas:**
  - Don't assume FFI calls are "safe just because the C looks innocuous."
  - The `AsRef`/`AsMut` unsoundness fix (WS6-prep, partial #277) — pointer-owning wrappers do not implement these traits.
  - Skeletal-animation Drop correctness (the redesign that landed in WS3).
- **See also:** `CONTRIBUTE.md`, `core-concepts/raii-and-resources.md`, docs.rs unsafe-fn pages.

- [ ] **Step 1: Write `book/src/core-concepts/safety.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Safety]()` with `- [Safety](./core-concepts/safety.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/core-concepts/safety.md book/src/SUMMARY.md
git commit -m "docs(ws7b): core-concepts/safety chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: `core-concepts/features.md`

**Files:**
- Create: `book/src/core-concepts/features.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Features and platforms]()` with `- [Features and platforms](./core-concepts/features.md)`.

**Chapter contract:**
- **Intro:** raylib-rs uses Cargo features to gate platform back-ends, optional math integrations, and the software renderer. Default features mirror raylib 6.0's `config.h` `SUPPORT_*` set.
- **API surface bullets:** the headline features — `opengl_33` (default), `opengl_21`, `opengl_es_20`, `drm`, `wayland`, `software_renderer`, `glam`, `mint`, `serde`, `nobuild`, `nobindgen`, `full` (the WS6 alias).
- **Example:** ``` ```text ``` Cargo.toml snippets — three common stanzas (default, headless via `software_renderer`, with `glam` and `serde`).
- **Gotchas:**
  - `opengl_*` features are mutually exclusive; the build script panics on conflicts.
  - `software_renderer` is mutually exclusive with `opengl_*` (and currently with `wasm32-unknown-emscripten` — see tracked-deferred).
  - `--all-features` is **invalid** for `raylib-sys`; use `--features full`.
- **See also:** `getting-started/install-{windows,macos,linux,web}.md`, `modules/software-renderer.md`, `ecosystem/glam-mint-serde.md`.

- [ ] **Step 1: Write `book/src/core-concepts/features.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Features and platforms]()` with `- [Features and platforms](./core-concepts/features.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/core-concepts/features.md book/src/SUMMARY.md
git commit -m "docs(ws7b): core-concepts/features chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: `modules/window-and-drawing.md`

**Files:**
- Create: `book/src/modules/window-and-drawing.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Window and drawing]()` with `- [Window and drawing](./modules/window-and-drawing.md)`.

**Chapter contract:**
- **Intro:** The window is opened by `raylib::init().build()`; drawing happens between `begin_drawing` and the returned guard's drop. All drawing primitives hang off the `RaylibDrawHandle` trait.
- **API surface bullets:** `RaylibBuilder::{size, title, vsync, msaa_4x, undecorated, ...}`, `RaylibHandle::begin_drawing`, the `RaylibDraw` trait (`clear_background`, `draw_text`, `draw_rectangle`, `draw_line`, etc.), `RaylibHandle::set_target_fps`.
- **Example:** ``` ```rust,no_run ``` — open a 640x480 window, set FPS, run a loop that clears and draws text.
- **Gotchas:**
  - Drawing primitives are methods on the `RaylibDrawHandle` guard; you cannot draw outside `begin_drawing`/`end_drawing`.
  - FPS pacing: `set_target_fps(0)` disables the limiter.
  - Window state queries (`is_window_minimized`, …) — backlog #287 / PR #252 — verify behavior on current state.
- **See also:** `core-concepts/handle-and-thread.md`, `modules/input.md`, `modules/shapes.md`.

- [ ] **Step 1: Write `book/src/modules/window-and-drawing.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Window and drawing]()` with `- [Window and drawing](./modules/window-and-drawing.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/window-and-drawing.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/window-and-drawing chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 7: `modules/input.md`

**Files:**
- Create: `book/src/modules/input.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Input]()` with `- [Input](./modules/input.md)`.

**Chapter contract:**
- **Intro:** Input is queried per-frame off `RaylibHandle` — keyboard, mouse, gamepad, touch.
- **API surface bullets:** `RaylibHandle::is_key_down`, `is_key_pressed`, `get_mouse_position`, `get_mouse_wheel_move`, `is_gamepad_available`, `get_gamepad_button_pressed`, the `KeyboardKey`/`MouseButton`/`GamepadButton` enums.
- **Example:** ``` ```rust,no_run ``` — query `is_key_down(KeyboardKey::KEY_SPACE)` inside a frame loop.
- **Gotchas:**
  - `is_key_pressed` is per-frame edge-triggered; `is_key_down` is level.
  - Mouse coordinates are in window pixels (Y-down).
  - **Tracked-deferred:** `get_gamepad_button_pressed`'s `transmute::<u32, GamepadButton>` is UB on out-of-range values (see `notes/ws6b-complete.md`). Documented here so users are aware; small soundness fix will land in a future PR.
- **See also:** `modules/window-and-drawing.md`, docs.rs `KeyboardKey`.

- [ ] **Step 1: Write `book/src/modules/input.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Input]()` with `- [Input](./modules/input.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/input.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/input chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 8: `modules/shapes.md`

**Files:**
- Create: `book/src/modules/shapes.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Shapes]()` with `- [Shapes](./modules/shapes.md)`.

**Chapter contract:**
- **Intro:** 2D primitives — rectangles, lines, circles, triangles, polygons. All exposed via `RaylibDraw` trait methods on the drawing guard.
- **API surface bullets:** `draw_rectangle`, `draw_rectangle_v`, `draw_rectangle_rec`, `draw_circle`, `draw_circle_v`, `draw_line`, `draw_line_ex`, `draw_triangle`, `draw_poly`, `draw_pixel`.
- **Example:** ``` ```rust ``` software-renderer doctest using `test_harness::render_frame` to draw a rectangle + circle and read pixels back (mirror the WS4b/WS5 render-test pattern).
- **Gotchas:** None notable; shapes are stable across raylib 5.x/6.0.
- **See also:** `modules/window-and-drawing.md`, `modules/textures-and-images.md`, `modules/software-renderer.md`.

- [ ] **Step 1: Write `book/src/modules/shapes.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Shapes]()` with `- [Shapes](./modules/shapes.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/shapes.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/shapes chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 9: `modules/textures-and-images.md`

**Files:**
- Create: `book/src/modules/textures-and-images.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Textures and images]()` with `- [Textures and images](./modules/textures-and-images.md)`.

**Chapter contract:**
- **Intro:** raylib has two related types: `Image` is CPU-side pixel data, `Texture2D` is GPU-side. You load/manipulate via `Image`, then `load_texture_from_image` to upload.
- **API surface bullets:** `Image::load_image`, `Image::gen_color`, `Image::gen_perlin_noise`, `Image::draw`, `Texture2D::load_from_image`, `RenderTexture2D`, the `RaylibDraw::draw_texture` family.
- **Example:** ``` ```rust ``` software-renderer doctest generating an image and reading a pixel back.
- **Gotchas:**
  - `Image` is CPU; `Texture2D` is GPU. Modifications happen on `Image`, then re-upload.
  - `RenderTexture2D` flips Y on readback in the WS4 harness (BGRA/Y-flip — see `notes/ws4b-complete.md`).
  - Backlog: `export_image_to_memory()` memory-leak (#247) — fixed by #250 in WS3.
- **See also:** `modules/software-renderer.md`, `modules/3d-models.md`, docs.rs `Image`/`Texture2D`.

- [ ] **Step 1: Write `book/src/modules/textures-and-images.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Textures and images]()` with `- [Textures and images](./modules/textures-and-images.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/textures-and-images.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/textures-and-images chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 10: `modules/text-and-fonts.md`

**Files:**
- Create: `book/src/modules/text-and-fonts.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Text and fonts]()` with `- [Text and fonts](./modules/text-and-fonts.md)`.

**Chapter contract:**
- **Intro:** raylib bundles a default font; custom fonts are loaded via `Font::load_font` / `Font::load_font_ex` (with codepoints).
- **API surface bullets:** `Font::load_font`, `Font::load_font_ex` (codepoints), `RaylibDraw::draw_text`, `RaylibDraw::draw_text_ex`, `measure_text`, `measure_text_ex`.
- **Example:** ``` ```rust ``` software-renderer doctest drawing default-font text and reading a pixel.
- **Gotchas:**
  - 6.0 adds +30 new text symbols (per WS1 parity); cross-link to the `text.rs` rustdoc.
  - Codepoint loading: `Font::load_font_ex` lets you preload a non-ASCII set for performance.
- **See also:** `modules/window-and-drawing.md`, docs.rs `Font`.

- [ ] **Step 1: Write `book/src/modules/text-and-fonts.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Text and fonts]()` with `- [Text and fonts](./modules/text-and-fonts.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/text-and-fonts.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/text-and-fonts chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 11: `modules/3d-models.md`

**Files:**
- Create: `book/src/modules/3d-models.md`
- Modify: `book/src/SUMMARY.md` — replace `- [3D models]()` with `- [3D models](./modules/3d-models.md)`.

**Chapter contract:**
- **Intro:** 3D rendering uses `Camera3D` + a `BeginMode3D` guard; models live as `Model` (containing `Mesh` arrays + `Material` arrays), with optional `ModelAnimations` for skeletal animation.
- **API surface bullets:** `Model::load_model`, `Mesh`, `Material`, `ModelAnimations` (new RAII in 6.0), `Camera3D`, `RaylibDrawHandle::begin_mode_3d`, `RaylibDraw3D::draw_model`.
- **Example:** ``` ```rust,no_run ``` — open a window, load a model, draw it from a camera.
- **Gotchas:**
  - `ModelAnimations` is the new RAII wrapper for skeletal animation (replaces the manual `UnloadModelAnimations` dance — see `notes/ws3-complete.md`).
  - Mesh accessor soundness (PRs #257/#118/#256 folded in WS6-prep) — the safe accessors return `&[T]` where the C field guarantees length; out-of-bounds is impossible.
  - Backlog: #283 (improper index calculation), #207 (3D needs work) — both tracked-deferred to future workstreams.
- **See also:** `modules/textures-and-images.md`, `modules/raymath.md`, docs.rs `Model`.

- [ ] **Step 1: Write `book/src/modules/3d-models.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [3D models]()` with `- [3D models](./modules/3d-models.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/3d-models.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/3d-models chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 12: `modules/audio.md`

**Files:**
- Create: `book/src/modules/audio.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Audio]()` with `- [Audio](./modules/audio.md)`.

**Chapter contract:**
- **Intro:** Audio sits behind a separate `AudioHandle` (init via `RaylibAudio::init_audio_device()`). `Wave`, `Sound`, `Music`, `AudioStream` are **lifetime-bound to the handle** — the borrow checker enforces correct teardown.
- **API surface bullets:** `RaylibAudio::init_audio_device`, `Wave`, `Sound`, `Music`, `AudioStream`, `Sound::play`, `Music::play_stream`, `AudioStream::is_processed`.
- **Example:** ``` ```rust,no_run ``` — init audio, load a Sound, play it.
- **Gotchas:**
  - `AudioHandle` must outlive every audio resource — this is enforced by lifetime parameters, not runtime checks.
  - WS6-prep removed the unsound default `Sound` impl (part of #277).
  - The deprecated audio callback was removed outright in 6.0 — use `AudioStream` streaming instead.
- **See also:** `core-concepts/raii-and-resources.md`, docs.rs `AudioHandle`/`Sound`.

- [ ] **Step 1: Write `book/src/modules/audio.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Audio]()` with `- [Audio](./modules/audio.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/audio.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/audio chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 13: `modules/raymath.md`

**Files:**
- Create: `book/src/modules/raymath.md`
- Modify: `book/src/SUMMARY.md` — replace `- [raymath]()` with `- [raymath](./modules/raymath.md)`.

**Chapter contract:**
- **Intro:** 6.0 ships native `#[repr(C)]` math types — `Vector2`, `Vector3`, `Vector4`, `Matrix`, `Quaternion`. Math comes from raylib's own raymath via a C-shim, wrapped as methods/operators. `mint`/`glam`/`serde` are opt-in adapters.
- **API surface bullets:** `Vector2`/`Vector3`/`Vector4`, `Matrix`, `Quaternion`, key methods: `dot`, `cross`, `normalize`, `length`, `Matrix::identity`, `Matrix::rotate_*`, `Matrix::translate`, `Quaternion::from_axis_angle`, etc.
- **Example:** ``` ```rust ``` doctest (no window) — vector dot/cross, matrix construction.
- **Gotchas:**
  - C aliases `Quaternion` to `Vector4`; raylib-rs treats it as a distinct `#[repr(C)]` type for type-safety. Conversion is explicit.
  - **`MintVec*` types are deprecated** — see `ecosystem/glam-mint-serde.md`.
  - Default build has zero math-crate deps; `glam`/`mint`/`serde` are opt-in (see `core-concepts/features.md`).
- **See also:** `modules/3d-models.md`, `modules/collision.md`, `ecosystem/glam-mint-serde.md`, docs.rs `Vector3`/`Matrix`.

- [ ] **Step 1: Write `book/src/modules/raymath.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [raymath]()` with `- [raymath](./modules/raymath.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/raymath.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/raymath chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 14: `modules/collision.md`

**Files:**
- Create: `book/src/modules/collision.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Collision]()` with `- [Collision](./modules/collision.md)`.

**Chapter contract:**
- **Intro:** Collision helpers for rectangles, circles, points, lines, and 3D spheres/boxes. Pure functions; no handle needed.
- **API surface bullets:** `check_collision_recs`, `check_collision_circles`, `check_collision_point_rec`, `check_collision_circle_rec`, `check_collision_point_line`, `check_collision_lines`, `check_collision_point_triangle`, 3D: `check_collision_spheres`, `check_collision_boxes`.
- **Example:** ``` ```rust ``` doctest (no window) — instantiate two `Rectangle`s and call `check_collision_recs`.
- **Gotchas:** None notable; these are pure-data functions.
- **See also:** `modules/raymath.md`, `modules/shapes.md`, docs.rs `check_collision_recs`.

- [ ] **Step 1: Write `book/src/modules/collision.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Collision]()` with `- [Collision](./modules/collision.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/collision.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/collision chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 15: `modules/raygui.md`

**Files:**
- Create: `book/src/modules/raygui.md`
- Modify: `book/src/SUMMARY.md` — replace `- [raygui]()` with `- [raygui](./modules/raygui.md)`.

**Chapter contract:**
- **Intro:** raygui is an immediate-mode GUI toolkit shipped with raylib. raylib-rs's 6.0 binding hits parity with raygui 6.0 — 57 functions, all under grouped sub-traits, signatures use `impl AsRef<str>` + a thread-local scratch buffer (see `notes/ws5-complete.md`).
- **API surface bullets:** the grouped sub-traits (button/label/slider/combo/textbox/messagebox, …), `gui_button`, `gui_label`, `gui_slider`, `gui_set_state`, `gui_load_style`.
- **Example:** ``` ```rust ``` software-renderer doctest using `test_harness::render_gui` (per WS5) to draw a button and read pixels.
- **Gotchas:**
  - String params are `impl AsRef<str>` — pass `&str`, `String`, or anything that derefs.
  - Backlog: #296 (`gui_load_style_from_memory`) is partially landed in WS5; the safe wrapper covers what raygui currently exports.
  - State is global (raygui's design); call ordering matters.
- **See also:** `modules/window-and-drawing.md`, `core-concepts/strings-and-allocs.md`, docs.rs raygui module.

- [ ] **Step 1: Write `book/src/modules/raygui.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [raygui]()` with `- [raygui](./modules/raygui.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/raygui.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/raygui chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 16: `modules/rlgl.md`

**Files:**
- Create: `book/src/modules/rlgl.md`
- Modify: `book/src/SUMMARY.md` — replace `- [rlgl]()` with `- [rlgl](./modules/rlgl.md)`.

**Chapter contract:**
- **Intro:** `rlgl` is raylib's immediate-mode OpenGL abstraction layer. raylib-rs 6.0 exposes a safe wrapper with `RlMatrix` and `RlImmediate` RAII guards plus `&Texture2D`/`&Shader` bind helpers (`notes/ws5-complete.md`).
- **API surface bullets:** `rl_begin`, `rl_draw` (the vertex-stream guard), `rl_push_matrix`/`rl_pop_matrix` via `RlMatrix`, `rl_translatef`, `rl_rotatef`, `rl_scalef`, `rl_ortho`, `rl_set_matrix_*`, `rl_set_texture`, `rl_enable_shader`.
- **Example:** ``` ```rust ``` software-renderer doctest using `test_harness::render_rlgl` (per WS5) to draw a custom vertex stream.
- **Gotchas:**
  - `RlMatrix` auto-pops on drop — don't manually `rl_pop_matrix`.
  - GL-object lifecycle for raw textures/shaders is left to existing RAII + raw `ffi`; this safe layer is for the immediate-mode subset.
- **See also:** `modules/window-and-drawing.md`, `modules/software-renderer.md`, docs.rs `rlgl` module.

- [ ] **Step 1: Write `book/src/modules/rlgl.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [rlgl]()` with `- [rlgl](./modules/rlgl.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/rlgl.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/rlgl chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 17: `modules/software-renderer.md`

**Files:**
- Create: `book/src/modules/software-renderer.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Software renderer]()` with `- [Software renderer](./modules/software-renderer.md)`.

**Chapter contract:**
- **Intro:** raylib 6.0 adds `rlsw`, a software renderer backend. raylib-rs exposes it via the `software_renderer` Cargo feature + the `raylib::test_harness` module — a windowless context that renders into an in-memory framebuffer and offers pixel-probe assertions.
- **API surface bullets:** `software_renderer` Cargo feature, `raylib::test_harness::{init, render_frame, render_gui, render_rlgl, …}`, the pixel readback functions.
- **Example:** ``` ```rust ``` doctest (under `cfg(feature = "software_renderer")`) — initialize a 256x256 framebuffer, draw a red rectangle, read the center pixel, assert RED.
- **Gotchas:**
  - The harness reads back **BGRA top-left** RGBA; the raylib backbuffer is GL-style bottom-up so the harness Y-flips for you (see `notes/ws5-complete.md`).
  - `software_renderer` is mutually exclusive with `opengl_*` — and currently with `wasm32-unknown-emscripten` (tracked-deferred, see `notes/ws6b-complete.md`).
  - All 5 raylib modules (rcore/rshapes/rtextures/rtext/rmodels) must be linked for the harness to work — see memory note `software-renderer-headless-testing`.
- **See also:** `core-concepts/features.md`, docs.rs `test_harness`, `notes/ws4b-complete.md`, `notes/ws5-complete.md`.

- [ ] **Step 1: Write `book/src/modules/software-renderer.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Software renderer]()` with `- [Software renderer](./modules/software-renderer.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/software-renderer.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/software-renderer chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 18: `modules/callbacks-and-logging.md`

**Files:**
- Create: `book/src/modules/callbacks-and-logging.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Callbacks and logging]()` with `- [Callbacks and logging](./modules/callbacks-and-logging.md)`.

**Chapter contract:**
- **Intro:** raylib exposes hooks for trace-log output (and historically for audio streaming). raylib-rs wraps the trace-log hook as a Rust callback.
- **API surface bullets:** `set_trace_log_callback`, `TraceLogLevel`, `set_trace_log_level`, the raylib `trace_log` family (free functions in raylib-rs since the WS2 refactor — see `CHANGELOG.md` 5.7.0 entry on `trace_log`).
- **Example:** ``` ```rust,no_run ``` — install a callback that prints to stderr, then call `trace_log` once.
- **Gotchas:**
  - Callbacks are global state; setting a new one replaces the old.
  - The audio callback was removed outright in 6.0 — use `AudioStream` streaming instead (see `modules/audio.md`).
- **See also:** `modules/audio.md`, docs.rs `set_trace_log_callback`.

- [ ] **Step 1: Write `book/src/modules/callbacks-and-logging.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Callbacks and logging]()` with `- [Callbacks and logging](./modules/callbacks-and-logging.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/callbacks-and-logging.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/callbacks-and-logging chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 19: `modules/error-handling.md`

**Files:**
- Create: `book/src/modules/error-handling.md`
- Modify: `book/src/SUMMARY.md` — replace `- [Error handling]()` with `- [Error handling](./modules/error-handling.md)`.

**Chapter contract:**
- **Intro:** raylib-rs uses `Result<T, String>` for load operations (the C library returns sentinel values; raylib-rs maps these to `Err(String)` with the raylib trace-log message). Drawing operations are infallible.
- **API surface bullets:** the `Result<T, String>` return convention; common error sources (file not found, invalid format, GPU upload failure).
- **Example:** ``` ```rust,no_run ``` — `Image::load_image("nonexistent.png")` returns an `Err`; pattern-match it.
- **Gotchas:**
  - Errors are stringly-typed; this is intentional for a thin C-binding layer. Downstream apps that need typed errors should wrap.
  - No panic-on-error in the safe API — except `raylib::init().build()` which panics if a second init is attempted (window context is global).
- **See also:** `core-concepts/safety.md`, `modules/textures-and-images.md`.

- [ ] **Step 1: Write `book/src/modules/error-handling.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [Error handling]()` with `- [Error handling](./modules/error-handling.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/modules/error-handling.md book/src/SUMMARY.md
git commit -m "docs(ws7b): modules/error-handling chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 20: `ecosystem/glam-mint-serde.md`

**Files:**
- Create: `book/src/ecosystem/glam-mint-serde.md`
- Modify: `book/src/SUMMARY.md` — replace `- [glam, mint, serde]()` with `- [glam, mint, serde](./ecosystem/glam-mint-serde.md)`.

**Chapter contract:**
- **Intro:** raylib-rs 6.0 makes math-ecosystem integration opt-in. Enable `glam`, `mint`, or `serde` features to add conversions / trait impls for the native `Vector*` / `Matrix` / `Quaternion` types.
- **API surface bullets:** `--features glam` (From/Into for `glam::Vec*`/`glam::Mat4`), `--features mint` (From/Into for `mint::Vector*`/`mint::ColumnMatrix*`), `--features serde` (`Serialize`/`Deserialize` derives).
- **Example:** ``` ```rust ``` doctest gated on `cfg(feature = "glam")` — convert a `raylib::Vector3` to a `glam::Vec3` and back.
- **Gotchas:**
  - **`MintVec*` types are deprecated** (the 5.7.0 era bridge). Use the native types + `--features mint` instead.
  - `glam`'s `Mat4` is column-major; raylib's `Matrix` is row-major. Conversions handle the swap.
  - Default builds carry **zero** math-crate deps — these are strictly opt-in.
- **See also:** `modules/raymath.md`, `core-concepts/features.md`, docs.rs feature-gated items.

- [ ] **Step 1: Write `book/src/ecosystem/glam-mint-serde.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [glam, mint, serde]()` with `- [glam, mint, serde](./ecosystem/glam-mint-serde.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/ecosystem/glam-mint-serde.md book/src/SUMMARY.md
git commit -m "docs(ws7b): ecosystem/glam-mint-serde chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 21: `ecosystem/what-next.md`

**Files:**
- Create: `book/src/ecosystem/what-next.md`
- Modify: `book/src/SUMMARY.md` — replace `- [What next?]()` with `- [What next?](./ecosystem/what-next.md)`.

**Chapter contract:**
- **Intro:** Where to go after the book.
- **API surface bullets:** N/A — replace with a "Pointers" bulleted list.
- **Example:** N/A — this is a links-only chapter; replace the `## Example` heading with `## Pointers`.
- **Pointers list:**
  - **docs.rs rustdoc** — the full API reference at `https://docs.rs/raylib`.
  - **Upstream raylib** — `https://www.raylib.com/` for the C library reference, examples, and community.
  - **The showcase** — the WS9 finale will host a gallery of working Rust ports of raylib examples (placeholder link; updated when WS9 deploys).
  - **The repository** — `https://github.com/raylib-rs/raylib-rs` for source, issues, and the project's superpowers spec/plan archive under `docs/superpowers/`.
  - **CONTRIBUTE.md** — how to contribute (link from the book root).
- **See also:** introduction, the install guides.

- [ ] **Step 1: Write `book/src/ecosystem/what-next.md` per the contract above.**

- [ ] **Step 2: Update `book/src/SUMMARY.md` — replace `- [What next?]()` with `- [What next?](./ecosystem/what-next.md)`.**

- [ ] **Step 3: Build + doctest verification:**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```

- [ ] **Step 4: Commit:**

```bash
git add book/src/ecosystem/what-next.md book/src/SUMMARY.md
git commit -m "docs(ws7b): ecosystem/what-next chapter

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 22: Final push + watch — WS7b sign-off

- [ ] **Step 1: Verify the SUMMARY.md has no remaining `- [Title]()` draft entries**

Run:
```bash
grep -n "\]()" book/src/SUMMARY.md
```
Expected: zero matches (every chapter is linked). If any draft entries remain, investigate which chapter was missed.

- [ ] **Step 2: Full local verify**

```bash
cargo build -p raylib --features full
mdbook build book
mdbook test book -L target/debug/deps
```
Expected: clean. If `mdbook test` fails, the failing chapter must be fixed before sign-off.

- [ ] **Step 3: Push and watch**

```bash
git push fork 6.0-rc
gh run list -R Dacode45/ms-raylib-rs --limit 5
gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status
```
Expected: `book.yml` green.

- [ ] **Step 4: WS7b sign-off**

WS7b is done when:
- All 21 chapter files (5 Core Concepts + 14 Modules + 2 Ecosystem) exist with non-stub content.
- `book/src/SUMMARY.md` has zero draft entries (every chapter is linked).
- `mdbook build book` and `mdbook test book -L target/debug/deps` are both green locally and in CI.

Next: WS7c (`2026-05-27-ws7c-rustdoc-changelog-backlog.md`) lands the rustdoc enrichment, the CHANGELOG 6.0.0 entry, and the backlog cherry-picks (#284, #273) + issue sweeps (#291, #290).
