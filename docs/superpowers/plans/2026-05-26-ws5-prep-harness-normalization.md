# WS5-prep — Harness Readback Normalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `test_harness::render_frame` return a true top-left RGBA `Image` (channels and Y-orientation corrected), so WS5a/WS5b render tests use natural draw coordinates and colors instead of compensating for the rlsw BGRA + Y-inverted readback per-probe.

**Architecture:** `render_frame` reads back the rlsw framebuffer (BGRA, Y-inverted) then normalizes it in-place: vertical flip via the existing safe `Image::flip_vertical`, plus an R↔B channel swap over the raw RGBA8 buffer. A new `render_frame_raw` preserves the current un-normalized behavior as a documented escape hatch. The two existing WS4b tests are rewritten to natural coordinates.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), raylib-rs `software_renderer` feature (rlsw + Memory platform), `raylib::test_harness`.

**Spec:** `docs/superpowers/specs/2026-05-26-ws5-raygui-rlgl-design.md` (§WS5-prep).

**Background facts (don't re-derive — from `notes/ws4b-complete.md`):**
- `load_image_from_screen` returns an `Image` of format `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8` (4 bytes/pixel) whose bytes are actually **BGRA** (rlsw built with `SW_FRAMEBUFFER_OUTPUT_BGRA`), and whose rows are **Y-inverted** (`y_img = (h-1) - y_screen`).
- Both effects are deterministic on every OS (pure compile-time C).
- `assert_pixel` / `pixel_at` compare R/G/B only; alpha is intentionally ignored. Keep that.
- CI/test feature set requires ALL FIVE modules: `software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO`.

---

## File Structure

- `raylib/src/test_harness.rs` — add channel-swap normalization to `render_frame`; add `render_frame_raw`; update module/fn docs.
- `raylib/tests/render_shapes.rs` — rewrite probes to natural top-left RGBA coordinates/colors.
- `raylib/tests/render_text.rs` — confirm still green (scans the whole image; expected no logic change). Update the stale doc comment if needed.

The relevant existing API in `raylib/src/core/texture.rs`:
- `Image::flip_vertical(&mut self)` (line ~752) — safe, in-place vertical flip.
- `Image::width()` / `Image::height()` — dimensions (used by `render_text.rs` already).
- `unsafe fn Image::data(&self) -> *mut c_void` (line ~202) — raw pixel buffer pointer.
- `Image::format(&self) -> consts::PixelFormat` (line ~236).

---

## Task 1: Normalize `render_frame`, add `render_frame_raw`

**Files:**
- Modify: `raylib/src/test_harness.rs`

- [ ] **Step 1: Add `render_frame_raw` and rewrite `render_frame` to normalize**

Replace the existing `render_frame` function (lines ~37–58) with the following two functions. `render_frame_raw` is the old body; `render_frame` calls it then normalizes.

```rust
/// Draw one frame via the `draw` closure and read the software framebuffer back
/// **without normalization** — the returned [`Image`] is in raw rlsw readback
/// form: bytes are BGRA (R and B swapped vs. what was drawn) and rows are
/// Y-inverted (`y_img = (h - 1) - y_screen`). Prefer [`render_frame`] unless you
/// specifically need the raw readback; see `notes/ws4b-complete.md`.
///
/// `EndDrawing`-on-drop flushes rlsw into the memory framebuffer before
/// [`load_image_from_screen`](RaylibHandle::load_image_from_screen) reads it.
#[inline]
#[must_use]
pub fn render_frame_raw<F: FnOnce(&mut RaylibDrawHandle<'_>)>(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    draw: F,
) -> Image {
    {
        let mut d = rl.begin_drawing(thread);
        draw(&mut d);
    } // Drop here calls EndDrawing, flushing rlsw into the memory framebuffer.
    rl.load_image_from_screen(thread)
}

/// Draw one frame via the `draw` closure, then read the software framebuffer
/// back as a **normalized** top-left RGBA [`Image`].
///
/// The raw rlsw readback is BGRA + Y-inverted (deterministic on every OS; see
/// `notes/ws4b-complete.md`). This function corrects both so callers use natural
/// draw coordinates and colors: a `Color::RED` rectangle reads back as red at the
/// screen `(x, y)` it was drawn at. Use [`render_frame_raw`] for the un-normalized
/// buffer.
#[inline]
#[must_use]
pub fn render_frame<F: FnOnce(&mut RaylibDrawHandle<'_>)>(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    draw: F,
) -> Image {
    let mut img = render_frame_raw(rl, thread, draw);
    normalize_readback(&mut img);
    img
}

/// Correct the rlsw readback in place: flip vertically and swap the R/B channels,
/// yielding a true top-left RGBA image.
fn normalize_readback(img: &mut Image) {
    // Fix Y-inversion using the existing safe image op.
    img.flip_vertical();

    // Fix the BGRA-as-RGBA byte order by swapping byte 0 (R) and byte 2 (B) of
    // every 4-byte pixel. The readback is always PIXELFORMAT_UNCOMPRESSED_R8G8B8A8.
    debug_assert_eq!(
        img.format(),
        crate::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        "normalize_readback assumes 32-bit RGBA readback"
    );
    let len = (img.width() * img.height()) as usize * 4;
    // SAFETY: `img` is a freshly loaded screen image in 32-bit RGBA8 format, so
    // its data buffer is exactly `width * height * 4` valid, initialized bytes.
    // We only swap two in-bounds bytes per pixel; no aliasing (single &mut slice).
    unsafe {
        let bytes = std::slice::from_raw_parts_mut(img.data() as *mut u8, len);
        for px in bytes.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }
}
```

- [ ] **Step 2: Update the module-level doc example to natural coordinates**

The module doc (lines ~6–17) draws `Color::BLACK` and probes `(0,0)` for black — that stays valid (black is unaffected by the swap). No change required to the example, but update the module header note so the example's intent (natural coords now) is accurate. Replace the first doc paragraph (lines ~1–5, before the `# Example`) with:

```rust
//! Headless render-test harness (software_renderer / Memory platform).
//!
//! raylib is single-init per process, so a Tier-2 test file inits **once** via
//! [`with_headless`], draws inside the closure, then probes the framebuffer.
//!
//! [`render_frame`] returns a **normalized** top-left RGBA image (natural draw
//! coordinates and colors). [`render_frame_raw`] returns the raw rlsw readback
//! (BGRA + Y-inverted) for callers that need it.
```

- [ ] **Step 3: Build the harness to confirm it compiles**

Run:
```
cargo build -p raylib --no-default-features --features software_renderer
```
Expected: builds clean (no warnings about unused `render_frame_raw` — it's `pub`).

- [ ] **Step 4: Commit**

```bash
git add raylib/src/test_harness.rs
git commit -m "feat(ws5): normalize render_frame to top-left RGBA; add render_frame_raw

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: Rewrite `render_shapes.rs` to natural coordinates

**Files:**
- Modify: `raylib/tests/render_shapes.rs`

- [ ] **Step 1: Replace the test body with natural-coordinate probes**

Replace the entire file with the following. Probes now use the screen coordinates and the drawn colors directly (no BGRA/Y-flip compensation). The only residual rlsw quirk is the "topmost line lands one row in," so the blue top-edge line is probed at screen `y = 1`.

```rust
//! WS4b Tier-2 (updated WS5-prep): software-rendered shapes land the expected
//! pixels. Headless, no GPU. Uses the normalized `render_frame`, so probes are in
//! natural top-left coordinates with the drawn colors (no BGRA / Y-flip
//! compensation — `render_frame` handles both).
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::{assert_pixel, render_frame, with_headless};

// One `#[test]` per file: `with_headless` calls `InitWindow`, which raylib permits
// only once per process, so every probe shares a single headless context.
#[test]
fn shapes_render_expected_pixels() {
    with_headless(64, 64, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_rectangle(8, 8, 16, 16, Color::RED); // solid red block, screen (8..23, 8..23)
            d.draw_circle(48, 48, 8.0, Color::GREEN); // green disc, screen centre (48,48)
            d.draw_line(0, 0, 63, 0, Color::BLUE); // top edge blue, screen y≈0
        });

        // Background: well outside every primitive.
        assert_pixel(&img, 40, 20, Color::BLACK, 8);

        // Rectangle interior reads back as the drawn red.
        assert_pixel(&img, 16, 16, Color::RED, 16);

        // Circle centre reads back as the drawn green.
        assert_pixel(&img, 48, 48, Color::GREEN, 16);

        // Top-edge line reads back as the drawn blue. rlsw places the topmost line
        // one row in, so probe screen y=1 rather than y=0.
        assert_pixel(&img, 32, 1, Color::BLUE, 16);
    });
}
```

- [ ] **Step 2: Run the test**

Run:
```
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO --test render_shapes -- --test-threads=1
```
Expected: PASS.

- [ ] **Step 3: If the top-line probe fails, adjust the single row**

The blue line is the only probe with a known rlsw edge quirk. If Step 2 fails *only* on the `Color::BLUE` assertion, re-run after changing the probe `y` to `0` (then, if still failing, `2`). Leave a brief comment recording the row that passed. All other probes (background, rectangle, circle) must pass as written; if they don't, stop — that indicates a normalization bug in Task 1, not a probe-coordinate issue.

- [ ] **Step 4: Commit**

```bash
git add raylib/tests/render_shapes.rs
git commit -m "test(ws5): probe shapes in natural coords via normalized render_frame

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: Confirm `render_text.rs` still passes

**Files:**
- Modify (doc only, if needed): `raylib/tests/render_text.rs`

- [ ] **Step 1: Run the existing text test against the normalized harness**

The text test scans the whole image for near-white pixels, so the Y-flip/channel-swap doesn't change the count (white is `{255,255,255}` — unaffected by R↔B swap; the scan covers all rows). Run:
```
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO --test render_text -- --test-threads=1
```
Expected: PASS (still ~96 near-white pixels, threshold `> 40`).

- [ ] **Step 2: Update the doc comment to note normalization**

Replace the first doc line (line 1) so it reflects that the harness now normalizes:

```rust
//! WS4b Tier-2 (WS5-prep): drawn text produces foreground pixels within its
//! bounds. Headless. Uses the normalized `render_frame`; the whole-image white
//! scan is orientation/channel agnostic.
```

- [ ] **Step 3: Commit (only if the doc was changed)**

```bash
git add raylib/tests/render_text.rs
git commit -m "test(ws5): note normalized harness in render_text doc

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: Full Tier-2 run + handoff note

**Files:**
- None (verification + note only).

- [ ] **Step 1: Run both Tier-2 tests together (the exact CI command)**

Run:
```
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO --test render_shapes --test render_text -- --test-threads=1
```
Expected: both tests PASS.

- [ ] **Step 2: Confirm no clippy regressions in the harness (informational)**

Run:
```
cargo clippy -p raylib --no-default-features --features software_renderer
```
Expected: no new warnings on `test_harness.rs` (the `unsafe` block has a `// SAFETY:` comment; the `debug_assert_eq!` uses the fully-qualified `PixelFormat`). Fix any that appear.

- [ ] **Step 3: Commit any clippy fixes (if needed) and finish**

```bash
git add -A
git commit -m "chore(ws5): clippy clean for normalized harness

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Self-Review

**Spec coverage (§WS5-prep):**
- `render_frame` returns top-left RGBA → Task 1.
- `render_frame_raw` escape hatch → Task 1.
- `assert_pixel`/`pixel_at` keep ignoring alpha → unchanged (no task touches them; verified by Task 2/3 passing).
- Update the two existing WS4b tests → Task 2 (shapes), Task 3 (text).

**Placeholders:** none — every code step shows complete code; the one empirical degree of freedom (the top-line row) is bounded to a single probe with an explicit fallback procedure (Task 2 Step 3).

**Type consistency:** `render_frame` / `render_frame_raw` share the same signature `(&mut RaylibHandle, &RaylibThread, F) -> Image`; `normalize_readback(&mut Image)` is the only new internal symbol and is used exactly once. `Image::flip_vertical`, `Image::data`, `Image::format`, `Image::width`, `Image::height` all exist in `core/texture.rs`. `consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8` is the readback format named in the WS4b notes.
