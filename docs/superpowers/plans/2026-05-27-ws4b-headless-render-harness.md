# WS4b — Headless Render-Test Harness + Tier-2 Proof Tests

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Build a small headless test harness on top of the WS4a `software_renderer` feature that initialises a windowless context, draws, reads back the in-memory framebuffer, and offers **pixel-probe + tolerance** assertions — then prove it with Tier-2 rendering tests over a handful of shape/text primitives. This unlocks headless rendering verification on every OS (the WS5/showcase render tests build on it).

**Prerequisite:** WS4a complete — `software_renderer` builds with `PLATFORM=Memory` and the framebuffer readback API is confirmed (smoke test green). Use the exact readback call WS4a verified (`LoadImageFromScreen` → `Image`, or the documented alternative).

**Architecture:** raylib enforces single-init via `RaylibHandle` (one `InitWindow` per process). Cargo runs each integration-test **file** as its own binary but runs `#[test]` fns **within** a binary on multiple threads — so multiple `InitWindow`s in one binary would trip single-init. The harness therefore exposes a single entry point that initialises the headless context **once**, hands a closure a `RaylibHandle` + `RaylibThread`, and each Tier-2 test file contains **one** `#[test]` that draws several primitives and probes their pixels (or tests run with `--test-threads=1` and close between). Pixel-probe assertions read the readback `Image` and compare a pixel/region to an expected `Color` within a per-channel tolerance (robust to rasteriser rounding; no committed binary artifacts).

**Pixel-probe decision (owner, 2026-05-27):** assert specific pixels/regions within tolerance — NOT golden-image snapshots.

**Tech Stack:** Rust 1.85, raylib 6.0 `software_renderer` (Memory platform / rlsw). Branch `6.0-rc`; push to `fork` for CI (the WS4a `software-render` job runs these).

**Reference:** `docs/superpowers/plans/2026-05-27-ws4a-software-renderer-feature.md` (the feature + confirmed readback); `docs/superpowers/notes/spike-rlsw.md`; roadmap D10/D11 + §4 "Testability"; `raylib/src/core/{drawing,texture,text}.rs` (draw + `Image` pixel APIs); the existing `Image` pixel-access methods (e.g. `get_image_data`/`get_color`/colors()).

**Pre-flight:** `cargo build -p raylib --no-default-features --features software_renderer` green; WS4a smoke test green.

**Scope boundary:** Add a `software_renderer`-gated harness module + Tier-2 integration tests. Cover a SMALL proof set (background, rectangle, circle, line, text) per the WS4 done criteria — broad coverage is backfilled in WS5/showcase. Do not change rendering wrappers themselves.

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `raylib/src/test_harness.rs` (feature-gated `pub mod`) | Headless init helper + framebuffer readback + pixel-probe assertion API | 1 |
| `raylib/src/lib.rs` | `#[cfg(feature = "software_renderer")] pub mod test_harness;` | 1 |
| `raylib/tests/render_shapes.rs` | Tier-2 proof: background/rectangle/circle/line pixel probes | 2 |
| `raylib/tests/render_text.rs` | Tier-2 proof: drawn text produces foreground pixels in its bounds | 2 |
| `.github/workflows/baseline.yml` | Run Tier-2 tests in the `software-render` job | 3 |

---

## Task 1: The headless harness + pixel-probe API

**Files:** Create `raylib/src/test_harness.rs`; register it in `raylib/src/lib.rs`.

- [ ] **Step 1: Register the module.** In `raylib/src/lib.rs`, after the other module declarations:
```rust
/// Headless software-render test harness (enabled by the `software_renderer` feature).
#[cfg(feature = "software_renderer")]
pub mod test_harness;
```

- [ ] **Step 2: Write the harness.** Create `raylib/src/test_harness.rs`. Provide: a one-shot headless init, a draw+readback helper, and pixel-probe assertions. (Adjust the readback + pixel-access calls to the exact API WS4a confirmed and the crate's real `Image` accessors.)
```rust
//! Headless render-test harness (software_renderer / Memory platform).
//!
//! raylib is single-init per process, so a Tier-2 test file inits ONCE via
//! [`with_headless`], draws inside the closure, then probes the framebuffer.
use crate::prelude::*;

/// Initialise a windowless software-rendered context of `w`×`h`, run `body`, then
/// tear down. Call at most once per test process (raylib single-init).
pub fn with_headless<F: FnOnce(&mut RaylibHandle, &RaylibThread)>(w: i32, h: i32, body: F) {
    let (mut rl, thread) = init().size(w, h).title("headless").build();
    body(&mut rl, &thread);
    // RaylibHandle's Drop calls CloseWindow.
}

/// Draw one frame via `draw` and read the software framebuffer back as an `Image`.
pub fn render_frame<F: FnOnce(&mut RaylibDrawHandle)>(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    draw: F,
) -> Image {
    {
        let mut d = rl.begin_drawing(thread);
        draw(&mut d);
    } // EndDrawing on drop flushes rlsw into the memory framebuffer
    // Confirmed-by-WS4a readback (adjust if WS4a found a different call):
    rl.load_image_from_screen(thread)
}

/// One RGBA sample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Px { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

/// Read the pixel at (x, y) from a readback `Image`.
pub fn pixel_at(img: &Image, x: i32, y: i32) -> Px {
    let colors = img.get_image_data(); // Vec<Color>-like; use the crate's real accessor
    let idx = (y * img.width + x) as usize;
    let c = colors[idx];
    Px { r: c.r, g: c.g, b: c.b, a: c.a }
}

/// Assert pixel (x,y) ≈ expected within per-channel `tol`.
pub fn assert_pixel(img: &Image, x: i32, y: i32, expected: Color, tol: u8) {
    let p = pixel_at(img, x, y);
    let near = |a: u8, b: u8| (a as i16 - b as i16).unsigned_abs() as u8 <= tol;
    assert!(
        near(p.r, expected.r) && near(p.g, expected.g) && near(p.b, expected.b),
        "pixel ({x},{y}) = {p:?}, expected ~({},{},{}) ±{tol}",
        expected.r, expected.g, expected.b
    );
}
```
If `init()`/`begin_drawing`/`load_image_from_screen`/`get_image_data` differ from the real API, match the real signatures (read `core/mod.rs`, `core/drawing.rs`, `core/texture.rs`). If `load_image_from_screen` requires the thread or has a different name, use the real one. The `RaylibDrawHandle`/`RaylibDraw` trait is what `begin_drawing` returns — confirm the type name.

- [ ] **Step 3: Build the harness.** `cargo build -p raylib --no-default-features --features software_renderer`. Fix any API mismatches until it compiles. (It won't be exercised until Task 2.)

- [ ] **Step 4: Commit.**
```bash
git add raylib/src/test_harness.rs raylib/src/lib.rs
git commit -m "$(printf 'feat(ws4b): headless render-test harness (software_renderer)\n\nwith_headless one-shot init (respects single-init), render_frame draw+\nframebuffer readback, and assert_pixel tolerance probes.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: Tier-2 proof tests (shapes + text)

**Files:** Create `raylib/tests/render_shapes.rs` and `raylib/tests/render_text.rs`.

Each file = ONE `#[test]` (single-init per binary). Each does several draw+probe checks in one frame where possible, or sequential frames within the same context.

- [ ] **Step 1: Shapes proof.** Create `raylib/tests/render_shapes.rs`:
```rust
//! WS4b Tier-2: software-rendered shapes land the expected pixels. Headless, no GPU.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::{assert_pixel, render_frame, with_headless};

#[test]
fn shapes_render_expected_pixels() {
    with_headless(64, 64, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_rectangle(8, 8, 16, 16, Color::RED);        // solid red block
            d.draw_circle(48, 48, 8.0, Color::GREEN);          // green disc
            d.draw_line(0, 0, 63, 0, Color::BLUE);             // top edge blue
        });
        // Background stays black where nothing drawn.
        assert_pixel(&img, 40, 20, Color::BLACK, 8);
        // Rectangle interior is red.
        assert_pixel(&img, 16, 16, Color::RED, 16);
        // Circle centre is green.
        assert_pixel(&img, 48, 48, Color::GREEN, 16);
        // Line pixel near top is blue.
        assert_pixel(&img, 32, 0, Color::BLUE, 16);
    });
}
```
Tolerances are generous (rlsw rounding / line AA). If a probe is off because the coordinate isn't quite inside the shape, nudge the probe coordinate (not the tolerance) to a clearly-interior point. raylib's framebuffer origin is top-left.

- [ ] **Step 2: Text proof.** Create `raylib/tests/render_text.rs`:
```rust
//! WS4b Tier-2: drawn text produces foreground pixels within its bounds. Headless.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::{render_frame, with_headless};

#[test]
fn text_draws_foreground_pixels() {
    with_headless(120, 32, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_text("Hi", 4, 4, 20, Color::WHITE);
        });
        // Count near-white pixels in the text region; glyphs must mark some.
        let data = img.get_image_data();
        let mut white = 0;
        for y in 0..img.height {
            for x in 0..40.min(img.width) {
                let c = data[(y * img.width + x) as usize];
                if c.r > 180 && c.g > 180 && c.b > 180 { white += 1; }
            }
        }
        assert!(white > 10, "expected drawn glyph pixels, found {white} near-white");
    });
}
```
(Uses the default font, which ships embedded — no asset file needed. If the default font needs `SUPPORT_FILEFORMAT_TTF`/`SUPPORT_MODULE_RTEXT`, ensure the harness build enables `raylib-sys/SUPPORT_MODULE_RTEXT` — add to the feature set used by these tests / the `software-render` CI job.)

- [ ] **Step 3: Run the Tier-2 tests.** Single-threaded to be safe across binaries sharing nothing but to honour single-init within each:
`cargo test -p raylib --no-default-features --features software_renderer --test render_shapes --test render_text -- --test-threads=1`
Expected: PASS. Debug failures by dumping a few probed pixels (the harness already prints actual vs expected). If a primitive renders differently than assumed (e.g. y-flip, premultiplied alpha), adjust probe coordinates/expected values to the real rlsw output and note it.

- [ ] **Step 4: Commit.**
```bash
git add raylib/tests/render_shapes.rs raylib/tests/render_text.rs
git commit -m "$(printf 'test(ws4b): Tier-2 headless render proofs (shapes + text)\n\nPixel-probe assertions over rlsw-rendered primitives via the headless\nharness. Tier-2 rendering is now verifiable with no GPU/window.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 3: Run Tier-2 tests in CI

**Files:** `.github/workflows/baseline.yml` (the `software-render` job from WS4a).

- [ ] **Step 1: Add the Tier-2 step** to the `software-render` job, after the smoke test:
```yaml
      - name: Tier-2 render tests (RTEXT enabled for the default font)
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RSHAPES --test render_shapes --test render_text -- --test-threads=1
```
(Include whatever `SUPPORT_MODULE_*` the shapes/text wrappers require — at minimum RSHAPES + RTEXT. If the safe crate's `draw_*` wrappers are unconditionally compiled, the modules must be on or the C symbols won't link in the test binary.)

- [ ] **Step 2: Push and drive green.** Commit (Co-Authored-By trailer), `git push fork 6.0-rc`, `gh run watch`. **Done-gate for WS4b:** Tier-2 shape + text render tests pass headlessly on ubuntu/macOS/windows.

---

## WS4b done criteria

- A `software_renderer`-gated `test_harness` module: one-shot headless init, draw+readback, `assert_pixel` tolerance probes.
- Tier-2 proof tests for background/rectangle/circle/line + text pass headlessly (no GPU/window) locally and in 3-OS CI.
- The harness is reusable by WS5/showcase render tests.
- Handoff: WS5 (raygui + rlgl) and any drawing wrapper can now be render-verified; broad Tier-2 backfill continues there.
