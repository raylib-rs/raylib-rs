# raylib-test delete + salvage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Salvage the 17 still-relevant tests from `raylib-test/` into in-tree Tier-1 (`raylib/src/core/texture.rs`) and Tier-2 (`raylib/tests/integration_*.rs`) targets, then delete `raylib-test/` entirely and clean up the CI + workspace + docs.

**Architecture:** 2 Tier-1 unit tests for window-independent Image API + 6 Tier-2 integration files (one `#[test]` per file due to raylib's `InitWindow` single-init constraint) + 3 fixture assets copied from `raylib-test/resources/`. Each integration file batches related assertions inside a single `with_headless(...)` block. After tests are in place, delete the legacy crate + remove the non-required `integration-xvfb` CI job + update sanitizers.yml to point at the new model-animation test.

**Tech Stack:** Rust 1.85 (edition 2024), `software_renderer` feature for Tier-2 (`test_harness::with_headless`), `--test-threads=1` per existing WS4/WS5 convention.

**Spec reference:** `docs/superpowers/specs/2026-05-29-raylib-test-delete-and-salvage-design.md`.

---

### Task 1: Copy fixture assets to raylib/tests/fixtures/

**Files:**
- Create: `raylib/tests/fixtures/billboard.png`
- Create: `raylib/tests/fixtures/alagard.png`
- Create: `raylib/tests/fixtures/pixeloid.ttf`

Foundation for the file-loading tests. Copy three binary files from `raylib-test/resources/` to the new fixtures directory. Tests reference them via `tests/fixtures/<name>` relative to the workspace root.

- [ ] **Step 1: Create the fixtures directory + copy assets**

Run via Bash tool:
```bash
mkdir -p raylib/tests/fixtures
cp raylib-test/resources/billboard.png raylib/tests/fixtures/billboard.png
cp raylib-test/resources/alagard.png raylib/tests/fixtures/alagard.png
cp raylib-test/resources/pixeloid.ttf raylib/tests/fixtures/pixeloid.ttf
```

- [ ] **Step 2: Verify all three files exist with non-zero sizes**

Run: `ls -la raylib/tests/fixtures/`
Expected: three files listed, none empty. Sizes should match the originals at `raylib-test/resources/`.

- [ ] **Step 3: Commit Task 1**

```bash
git add raylib/tests/fixtures/billboard.png raylib/tests/fixtures/alagard.png raylib/tests/fixtures/pixeloid.ttf
git commit -m "$(cat <<'EOF'
feat(test-fixtures): copy assets from raylib-test/resources to raylib/tests/fixtures

Three binary assets copied verbatim:
- billboard.png — used by image_load + texture_load tests.
- alagard.png — used by font_load (PNG bitmap font).
- pixeloid.ttf — used by font_load_ex (TTF font).

These are the only fixtures needed for the salvaged tests; OBJ +
animation assets stay sourced from vendored
raylib-sys/raylib/examples/models/resources/.

Per spec §7. Originals remain at raylib-test/resources/ until Task 10
deletes the crate.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Add Tier-1 Image tests to raylib/src/core/texture.rs

**Files:**
- Modify: `raylib/src/core/texture.rs`

Two new window-independent `#[test]` fns: image loading happy + error paths, and an in-memory Image manipulation smoke test.

- [ ] **Step 1: Locate the existing `#[cfg(test)] mod tests` block (or end of file if absent)**

Run: `grep -n '^#\[cfg(test)\]' raylib/src/core/texture.rs`
Expected: 0-1 matches. If 0, append a new `mod tests` block at the end of the file. If 1, append the two tests inside the existing block.

- [ ] **Step 2: Append the test module + the two tests**

If `mod tests` doesn't exist, append this block at the end of `raylib/src/core/texture.rs`:

```rust

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::Color;

    /// Salvaged from raylib-test/src/texture.rs `test_image_loading`.
    /// Image is CPU-side; no window context needed.
    #[test]
    fn image_load_from_file_happy_and_error() {
        // Happy: load the bundled fixture.
        let path = "tests/fixtures/billboard.png";
        if std::path::Path::new(path).exists() {
            let img = Image::load_image(path).expect("billboard.png loads");
            assert!(img.width() > 0, "loaded image has non-zero width");
            assert!(img.height() > 0, "loaded image has non-zero height");
        } else {
            eprintln!("SKIP: {path} not found (cargo invoked from a non-workspace-root cwd?)");
        }

        // Error: nonexistent path returns Err.
        Image::load_image("tests/fixtures/does_not_exist.png")
            .expect_err("nonexistent file should error");
    }

    /// Salvaged from raylib-test/src/texture.rs `test_image_manipulations`.
    /// Exercises the Image CPU-side API without a window context.
    #[test]
    fn image_manipulations_no_segfault() {
        let mut i = Image::gen_image_color(32, 32, Color::new(230, 41, 55, 255));
        let mut canvas = Image::gen_image_color(32, 32, Color::new(0, 0, 0, 0));
        let mask = Image::gen_image_checked(
            32, 32, 8, 8,
            Color::new(255, 255, 255, 255),
            Color::new(0, 0, 0, 0),
        );

        let mut c = i.clone();
        c.alpha_mask(&mask);
        c.alpha_clear(Color::new(0, 0, 255, 255), 0.5);
        c.alpha_crop(0.5);
        c.alpha_premultiply();

        let mut blurry = c.clone();
        blurry.resize(64, 64);
        c.resize_nn(64, 64);
        i.resize_canvas(64, 64, 10, 10, Color::new(0, 0, 255, 255));

        c.mipmaps();
        blurry.dither(128, 128, 128, 128);

        let colors = c.extract_palette(100);
        assert_eq!(
            colors.len(),
            2,
            "checker-masked single-color image has 2-color palette"
        );

        canvas.draw(
            &i,
            Rectangle::new(0.0, 0.0, 20.0, 20.0),
            Rectangle::new(0.0, 0.0, 20.0, 20.0),
            Color::new(255, 255, 255, 255),
        );
        canvas.draw_rectangle_lines(
            Rectangle::new(20.0, 0.0, 20.0, 20.0),
            4,
            Color::new(0, 228, 48, 255),
        );
        canvas.draw_rectangle(40, 0, 20, 20, Color::new(255, 161, 0, 255));

        canvas.flip_vertical();
        canvas.flip_horizontal();
        canvas.rotate_cw();
        canvas.rotate_ccw();

        canvas.color_tint(Color::new(255, 109, 194, 255));
        canvas.color_invert();
        canvas.color_contrast(0.5);
        canvas.color_brightness(128);
        canvas.color_replace(
            Color::new(0, 228, 48, 255),
            Color::new(230, 41, 55, 255),
        );

        // Test reaches here = no segfault during the manipulation pipeline.
    }
}
```

If `mod tests` already exists, append only the two `#[test]` fns inside it (no surrounding `mod` wrapper).

- [ ] **Step 3: Verify Tier-1 tests pass**

Run: `cargo test -p raylib --lib --features full image_load_from_file image_manipulations 2>&1 | tail -15`
Expected: `test result: ok. 2 passed; 0 failed`.

If `extract_palette` returns a different count (e.g., 1 or 3) on this platform, the `assert_eq!(colors.len(), 2)` may need adjustment. The original raylib-test asserts 2; the C path generates an in-memory checker pattern of two distinct colors. If it fails, capture the actual count and reconsider the assertion.

If any Image API signature differs (e.g., `extract_palette` takes a different arg type), adapt to the current 6.0 shape; the spec lists what's available.

- [ ] **Step 4: Verify fmt + clippy**

Run:
```bash
cargo fmt --check 2>&1 | tail -3
cargo clippy -p raylib --features full --tests -- -D warnings 2>&1 | tail -5
```
Expected: both clean.

- [ ] **Step 5: Commit Task 2**

```bash
git add raylib/src/core/texture.rs
git commit -m "$(cat <<'EOF'
test(image): salvage Tier-1 Image tests from raylib-test

Two #[test] fns added to raylib/src/core/texture.rs:
- image_load_from_file_happy_and_error: Image::load_image happy +
  error paths. Uses raylib/tests/fixtures/billboard.png (added in
  Task 1). Guards missing-asset with SKIP log.
- image_manipulations_no_segfault: exercises ~15 Image CPU-side fns
  (alpha_*, resize*, mipmaps, dither, palette extraction, draw_*,
  flip_*, rotate_*, color_*). Asserts extract_palette returns 2
  colors for a 2-color checker pattern; rest is "doesn't segfault"
  smoke coverage.

Tier-1 because Image is CPU-side — no window or RaylibHandle needed.
Salvaged from raylib-test/src/texture.rs.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: integration_model_animations.rs (the keeper — ModelAnimations RAII)

**Files:**
- Create: `raylib/tests/integration_model_animations.rs`

The most valuable salvage: explicit `ModelAnimations::Drop` assertion. The sanitizers workflow's WS6a comment specifically wants to run this test. References vendored `guyanim.iqm` (no fixture copy needed).

- [ ] **Step 1: Create the file**

Write `raylib/tests/integration_model_animations.rs` with this content:

```rust
//! Tier-2: validate ModelAnimations::Drop frees the raylib heap array
//! exactly once. The sanitizers workflow (ASAN/UBSAN) runs this test
//! to catch double-free / use-after-free regressions in the WS3 RAII
//! redesign.
//!
//! Salvaged from raylib-test/tests/model_animation_raii.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn model_animations_load_and_drop() {
    with_headless(64, 64, |rl, thread| {
        // Vendored raylib example asset — bundled with the raylib C source.
        let path = "raylib-sys/raylib/examples/models/resources/guy/guyanim.iqm";
        if std::path::Path::new(path).exists() {
            let anims = rl
                .load_model_animations(thread, path)
                .expect("animations load");
            assert!(!anims.is_empty(), "expected >= 1 animation");
            drop(anims); // exercises UnloadModelAnimations exactly once.
        } else {
            eprintln!("SKIP: animation asset not found at {path}");
        }
    });
}
```

- [ ] **Step 2: Run the test under software_renderer**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test integration_model_animations -- --test-threads=1 2>&1 | tail -10
```
Expected: `test result: ok. 1 passed; 0 failed`.

If the asset isn't found (the SKIP path triggers), the test still passes with the SKIP log printed.

- [ ] **Step 3: Commit Task 3**

```bash
git add raylib/tests/integration_model_animations.rs
git commit -m "$(cat <<'EOF'
test(integration): salvage model_animation_raii — ModelAnimations Drop RAII

New raylib/tests/integration_model_animations.rs (Tier-2,
software_renderer-gated). Explicit ModelAnimations::Drop assertion
matching what the sanitizers workflow wants to run for ASAN/UBSAN
coverage of the WS3 redesign.

Uses vendored raylib-sys/raylib/examples/models/resources/guy/guyanim.iqm
(no fixture copy needed). File-existence guarded with SKIP log.

Salvaged verbatim from raylib-test/tests/model_animation_raii.rs;
switched from raylib::init().build() to with_headless(64, 64, ...)
matching the existing Tier-2 convention.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: integration_random_seed.rs (deterministic-seed assertions)

**Files:**
- Create: `raylib/tests/integration_random_seed.rs`

Real assertions (not "doesn't panic") — verifies raylib's deterministic-seed contract.

- [ ] **Step 1: Create the file**

Write `raylib/tests/integration_random_seed.rs` with this content:

```rust
//! Tier-2: raylib's set_random_seed contract is deterministic.
//! Salvaged from raylib-test/src/random.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn random_seed_is_deterministic() {
    with_headless(64, 64, |rl, _thread| {
        // Single value with seed 1: documented to return 2 for range 0..4.
        rl.set_random_seed(1);
        let r: i32 = rl.get_random_value(0..4);
        assert_eq!(r, 2, "set_random_seed(1) + get_random_value(0..4) is deterministic");

        // Sequence with seed 1: documented values from raylib-test/src/random.rs.
        rl.set_random_seed(1);
        let seq = rl.load_random_sequence(1..10, 10);
        let expected = [8, 7, 6, 4, 10, 3, 5, 1, 2, 9];
        assert_eq!(seq.len(), expected.len(), "sequence length matches");
        for (i, (got, want)) in seq.iter().zip(expected.iter()).enumerate() {
            assert_eq!(*got, *want, "sequence element {i} matches");
        }
    });
}
```

- [ ] **Step 2: Run the test**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test integration_random_seed -- --test-threads=1 2>&1 | tail -10
```
Expected: `test result: ok. 1 passed; 0 failed`.

If the expected values differ between raylib 5.x (raylib-test's pinned baseline) and raylib 6.0, the test fails with a clear diff. Capture the actual values and either (a) update the expected array to the 6.0 baseline (if the change is intentional / acceptable), or (b) escalate if the determinism contract regressed.

- [ ] **Step 3: Verify the API surface for `load_random_sequence`**

Run: `grep -n 'pub fn load_random_sequence' raylib/src/core/*.rs`
Expected: signature `pub fn load_random_sequence(&mut self, range: Range<i32>, count: u32) -> Vec<i32>` or similar. If it returns a different shape (e.g., `DataBuf<[i32]>` or an iterator), adapt the test's `.iter()` / `.len()` usage accordingly.

- [ ] **Step 4: Commit Task 4**

```bash
git add raylib/tests/integration_random_seed.rs
git commit -m "$(cat <<'EOF'
test(integration): salvage random-seed determinism assertions

New raylib/tests/integration_random_seed.rs (Tier-2,
software_renderer-gated). Two deterministic-seed assertions:
- set_random_seed(1) + get_random_value(0..4) == 2.
- set_random_seed(1) + load_random_sequence(1..10, 10) ==
  [8, 7, 6, 4, 10, 3, 5, 1, 2, 9].

Real assertions, not just smoke. Verifies raylib's documented
deterministic-seed contract still holds under 6.0.

Salvaged from raylib-test/src/random.rs.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: integration_window_api.rs (clipboard + screen_space + timing + cursor + set_title smoke)

**Files:**
- Create: `raylib/tests/integration_window_api.rs`

Batches 5 window-coupled smoke tests into one `#[test]` fn.

- [ ] **Step 1: Create the file**

Write `raylib/tests/integration_window_api.rs` with this content:

```rust
//! Tier-2: window-coupled API smoke tests batched into one test fn
//! (one InitWindow per process per file). Salvaged from
//! raylib-test/src/{misc,window}.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn window_api_smoke() {
    with_headless(64, 64, |rl, thread| {
        // Clipboard round-trip.
        let s = "Hello, world!";
        rl.set_clipboard_text(s).expect("set_clipboard_text");
        let got = rl.get_clipboard_text().expect("get_clipboard_text");
        assert_eq!(got, s, "clipboard round-trip preserves text");

        // Screen-space conversions (don't panic).
        let cam = Camera::orthographic(
            Vector3::ZERO,
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::Y,
            90.0,
        );
        let _ = rl.get_screen_to_world_ray(Vector2::ZERO, &cam);
        let _ = rl.get_world_to_screen(Vector3::ZERO, &cam);

        // Timing fns (don't panic).
        rl.set_target_fps(24);
        let _ = rl.get_fps();
        let _ = rl.get_frame_time();
        let _ = rl.get_time();

        // Cursor: double-show / double-hide / double-disable / double-enable
        // must be idempotent (raylib's C side guards against double-state).
        rl.hide_cursor();
        rl.hide_cursor();
        rl.show_cursor();
        rl.show_cursor();
        rl.disable_cursor();
        rl.disable_cursor();
        rl.enable_cursor();
        rl.enable_cursor();

        // Window title + dimensions (with_headless opened a 64x64 window).
        rl.set_window_title(thread, "raylib test");
        assert_eq!(rl.get_screen_width(), 64, "screen width matches headless dim");
        assert_eq!(rl.get_screen_height(), 64, "screen height matches headless dim");
    });
}
```

- [ ] **Step 2: Verify API surface**

Two fns from raylib-test were used with slightly different shapes in 5.x. Check the 6.0 shape:
- Run: `grep -nE 'pub fn (set_clipboard_text|get_clipboard_text|set_target_fps|set_window_title)' raylib/src/core/*.rs | head -10`

If `set_clipboard_text` / `get_clipboard_text` return `Result<(), _>` and `Result<String, _>` respectively, the `.expect(...)` calls are fine. If they return `Option<...>` or different shapes, adapt the test accordingly.

If `set_window_title` doesn't take a `&RaylibThread` in 6.0, drop the `thread` arg from that call.

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test integration_window_api -- --test-threads=1 2>&1 | tail -15
```
Expected: `test result: ok. 1 passed; 0 failed`.

If clipboard access fails on a headless CI runner (no clipboard daemon), the `set_clipboard_text(...).expect(...)` would panic. If that happens, wrap the clipboard block in:
```rust
if let Ok(()) = rl.set_clipboard_text(s) {
    if let Ok(got) = rl.get_clipboard_text() {
        assert_eq!(got, s);
    }
}
```
and add a `// SKIP if no clipboard available (headless CI)` comment. Don't pre-emptively gate; let the test surface the limitation.

- [ ] **Step 4: Commit Task 5**

```bash
git add raylib/tests/integration_window_api.rs
git commit -m "$(cat <<'EOF'
test(integration): salvage window-coupled API smoke tests

New raylib/tests/integration_window_api.rs (Tier-2,
software_renderer-gated). One #[test] batching 5 smoke tests:
- Clipboard round-trip (set + get + assert equality).
- Screen-space conversions (get_screen_to_world_ray +
  get_world_to_screen) — don't panic.
- Timing fns (set_target_fps + get_fps + get_frame_time +
  get_time) — don't panic.
- Cursor double-show / hide / disable / enable idempotency.
- Window title set + screen-dimension verification (64x64 matches
  the with_headless dim).

Salvaged from raylib-test/src/{misc,window}.rs. Single #[test] fn
per the InitWindow single-init constraint.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: integration_image_io.rs (screenshot + screen-load + texture_load + render_texture smoke)

**Files:**
- Create: `raylib/tests/integration_image_io.rs`

Batches image/screen I/O smoke tests. `load_render_texture` may not work on software_renderer; wrap in catch_unwind with SKIP if it doesn't.

- [ ] **Step 1: Create the file**

Write `raylib/tests/integration_image_io.rs` with this content:

```rust
//! Tier-2: image + screen I/O smoke tests. Salvaged from
//! raylib-test/src/{misc,texture}.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn image_io_smoke() {
    with_headless(64, 64, |rl, thread| {
        // take_screenshot writes a file.
        let out_dir = "target/tmp";
        std::fs::create_dir_all(out_dir).expect("mkdir target/tmp");
        let shot = format!("{out_dir}/screenshot.png");
        rl.take_screenshot(thread, &shot);
        assert!(
            std::path::Path::new(&shot).exists(),
            "take_screenshot writes {shot}"
        );

        // load_image_from_screen: make sure it doesn't segfault.
        let _ = rl.load_image_from_screen(thread);

        // texture_load: load PNG from file + from existing Image.
        let path = "tests/fixtures/billboard.png";
        if std::path::Path::new(path).exists() {
            let img = Image::load_image(path).expect("image loads");
            let _tex = rl
                .load_texture(thread, path)
                .expect("texture loads from path");
            let _tex2 = rl
                .load_texture_from_image(thread, &img)
                .expect("texture loads from image");
        } else {
            eprintln!("SKIP: {path} not found");
        }

        // render_texture: wrapped in catch_unwind because software_renderer
        // may not support GL framebuffer objects. If it panics, log a SKIP
        // and continue.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rl.load_render_texture(thread, 256, 256)
                .expect("render texture creates")
        }));
        match result {
            Ok(_rt) => { /* loaded successfully; dropping at end */ }
            Err(_) => eprintln!("SKIP: load_render_texture not supported under software_renderer"),
        }
    });
}
```

- [ ] **Step 2: Verify API surface**

Run: `grep -nE 'pub fn (take_screenshot|load_image_from_screen|load_texture|load_texture_from_image|load_render_texture)' raylib/src/core/*.rs | head -10`

Confirm signatures match. If `take_screenshot` doesn't take `&RaylibThread` in 6.0, drop the `thread` arg.

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test integration_image_io -- --test-threads=1 2>&1 | tail -15
```
Expected: `test result: ok. 1 passed; 0 failed`. The SKIP log for `load_render_texture` may print but the test passes either way.

- [ ] **Step 4: Commit Task 6**

```bash
git add raylib/tests/integration_image_io.rs
git commit -m "$(cat <<'EOF'
test(integration): salvage image + screen I/O smoke tests

New raylib/tests/integration_image_io.rs (Tier-2,
software_renderer-gated). One #[test] batching 4 smoke tests:
- take_screenshot writes target/tmp/screenshot.png; assert file
  exists.
- load_image_from_screen doesn't segfault.
- load_texture (from path) + load_texture_from_image (from Image)
  both succeed using tests/fixtures/billboard.png.
- load_render_texture (256, 256): wrapped in catch_unwind because
  software_renderer may not support GL framebuffer objects.
  Logs SKIP and continues if it panics.

Salvaged from raylib-test/src/{misc,texture}.rs.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 7: integration_fonts.rs (font_load + font_load_ex + font_export smoke)

**Files:**
- Create: `raylib/tests/integration_fonts.rs`

Batches font loading + export smoke tests. Uses both PNG bitmap font (alagard) and TTF font (pixeloid).

- [ ] **Step 1: Create the file**

Write `raylib/tests/integration_fonts.rs` with this content:

```rust
//! Tier-2: font loading + export smoke tests.
//! Salvaged from raylib-test/src/text.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn fonts_load_and_export_smoke() {
    with_headless(64, 64, |rl, thread| {
        // Bitmap font (PNG).
        let png_path = "tests/fixtures/alagard.png";
        if std::path::Path::new(png_path).exists() {
            let _font = rl
                .load_font(thread, png_path)
                .expect("load_font (png) succeeds");
        } else {
            eprintln!("SKIP: {png_path} not found");
        }

        // TTF font with explicit size + None glyph chars (use defaults).
        let ttf_path = "tests/fixtures/pixeloid.ttf";
        if std::path::Path::new(ttf_path).exists() {
            let _font_ex = rl
                .load_font_ex(thread, ttf_path, 32, None)
                .expect("load_font_ex (ttf) succeeds");
        } else {
            eprintln!("SKIP: {ttf_path} not found");
        }

        // export_font_as_code: write a .h file. Use target/tmp/ for output.
        if std::path::Path::new(png_path).exists() {
            std::fs::create_dir_all("target/tmp").expect("mkdir target/tmp");
            let font = rl.load_font(thread, png_path).expect("load_font for export");
            let _ = font.export_font_as_code("target/tmp/font.h");
            assert!(
                std::path::Path::new("target/tmp/font.h").exists(),
                "export_font_as_code writes target/tmp/font.h"
            );
        }
    });
}
```

- [ ] **Step 2: Verify API surface**

Run: `grep -nE 'pub fn (load_font|load_font_ex|export_font_as_code)' raylib/src/core/*.rs | head -10`

Confirm signatures. Especially `load_font_ex`'s third arg in 6.0 — the spec sketch passes `None` for glyph chars; verify that's the actual API shape.

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF --test integration_fonts -- --test-threads=1 2>&1 | tail -10
```
Expected: `test result: ok. 1 passed; 0 failed`.

Note the added `SUPPORT_FILEFORMAT_TTF` in the feature set — TTF support is feature-gated. If `load_font_ex` fails because TTF isn't enabled, this is the fix.

- [ ] **Step 4: Commit Task 7**

```bash
git add raylib/tests/integration_fonts.rs
git commit -m "$(cat <<'EOF'
test(integration): salvage font load + export smoke tests

New raylib/tests/integration_fonts.rs (Tier-2,
software_renderer-gated). One #[test] batching 3 smoke tests:
- load_font (PNG bitmap font) using tests/fixtures/alagard.png.
- load_font_ex (TTF font, size 32, default glyphs) using
  tests/fixtures/pixeloid.ttf.
- export_font_as_code writes target/tmp/font.h; assert file exists.

Each test guards the asset path with file-existence + SKIP log.
TTF support requires SUPPORT_FILEFORMAT_TTF in the feature set.

Salvaged from raylib-test/src/text.rs.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 8: integration_models.rs (load_obj_model + model_from_generated_mesh smoke)

**Files:**
- Create: `raylib/tests/integration_models.rs`

Model loading from OBJ + model-from-generated-mesh smoke tests. Both may have GL dependencies; wrap GL-touching calls in catch_unwind if software_renderer doesn't support them.

- [ ] **Step 1: Create the file**

Write `raylib/tests/integration_models.rs` with this content:

```rust
//! Tier-2: model loading + generated-mesh smoke tests.
//! Salvaged from raylib-test/src/models.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn models_load_and_generated_smoke() {
    with_headless(64, 64, |rl, thread| {
        // OBJ model loading: use vendored cube.obj from raylib examples.
        let obj_path = "raylib-sys/raylib/examples/models/resources/models/obj/cube.obj";
        if std::path::Path::new(obj_path).exists() {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = rl.load_model(thread, obj_path);
            }));
            if result.is_err() {
                eprintln!("SKIP: load_model not supported under software_renderer");
            }
        } else {
            eprintln!("SKIP: {obj_path} not found");
        }

        // Model from generated mesh: gen_mesh_cube + load_model_from_mesh.
        // May fail under software_renderer if GL VBO upload is required.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mesh = unsafe {
                Mesh::gen_mesh_cube(thread, 1.0, 1.0, 1.0).make_weak()
            };
            let _model = rl.load_model_from_mesh(thread, mesh)
                .expect("load_model_from_mesh");
        }));
        if result.is_err() {
            eprintln!("SKIP: gen_mesh_cube + load_model_from_mesh not supported under software_renderer");
        }
    });
}
```

- [ ] **Step 2: Verify API surface**

Run: `grep -nE 'pub fn (load_model|load_model_from_mesh|gen_mesh_cube|make_weak)' raylib/src/core/*.rs | head -10`

Confirm signatures. Especially `gen_mesh_cube` (associated fn on Mesh? requires `&RaylibThread`?) and `make_weak` (unsafe?). The spec sketch is taken verbatim from raylib-test which was 5.x — the 6.0 API may have shifted. Adapt.

- [ ] **Step 3: Run the test**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION --test integration_models -- --test-threads=1 2>&1 | tail -10
```
Expected: `test result: ok. 1 passed; 0 failed`. The SKIP paths may print but the test passes.

Feature set additions: `SUPPORT_FILEFORMAT_OBJ` + `SUPPORT_MESH_GENERATION` (OBJ loading + mesh generation are feature-gated).

- [ ] **Step 4: Commit Task 8**

```bash
git add raylib/tests/integration_models.rs
git commit -m "$(cat <<'EOF'
test(integration): salvage model loading + generated-mesh smoke tests

New raylib/tests/integration_models.rs (Tier-2,
software_renderer-gated). One #[test] batching 2 smoke tests:
- load_model (OBJ) using vendored
  raylib-sys/raylib/examples/models/resources/models/obj/cube.obj.
- gen_mesh_cube + load_model_from_mesh: generated 1x1x1 cube mesh,
  loaded as model.

Both calls wrapped in catch_unwind because software_renderer may not
support GL VBO upload required by model loading. Logs SKIP and
continues if either panics.

Salvaged from raylib-test/src/models.rs. Feature set requires
SUPPORT_FILEFORMAT_OBJ + SUPPORT_MESH_GENERATION.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 9: Full Tier-1 + Tier-2 verification before delete

**Files:** none (verification only)

Verify all salvaged tests pass under the canonical feature sets before deleting raylib-test.

- [ ] **Step 1: Tier-1 lib tests pass**

Run: `cargo test -p raylib --lib --features full 2>&1 | tail -10`
Expected: previous count + 2 (the new image_load + image_manipulations). If pre-salvage was 71 (from mixed-audio's "69 baseline + 2 Tier-1"), expect 73 now.

- [ ] **Step 2: All 6 new Tier-2 integration tests pass**

Run a single command that picks up all integration files:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION -- --test-threads=1 2>&1 | tail -20
```
Expected: all 6 new integration tests pass (plus the 4 pre-existing render_* tests + any others). Total count should be: previous + 6 = 10+ tests passing.

- [ ] **Step 3: Clippy + fmt clean**

Run:
```bash
cargo fmt --check 2>&1 | tail -3
cargo clippy -p raylib --features full --tests -- -D warnings 2>&1 | tail -5
```
Expected: both clean.

No commit in this task (verification only). Proceed to Task 10 with confidence that the salvage works.

---

### Task 10: Delete raylib-test/ + workspace cleanup

**Files:**
- Delete: `raylib-test/` (entire directory)
- Modify: `Cargo.toml` (workspace root)

Now that the salvage is complete and verified, delete the legacy crate.

- [ ] **Step 1: Confirm `raylib-test` is in the workspace exclude list**

Run: `grep -nE 'raylib-test|^exclude' Cargo.toml`
Expected: at least one match showing `raylib-test` is in an `exclude = [...]` array. Note the structure (it's typically a multi-line array).

- [ ] **Step 2: Remove the `raylib-test` entry from workspace `Cargo.toml`**

Edit `Cargo.toml` and remove the `raylib-test` entry from the `exclude` list. Keep the rest of the list intact (likely `showcase` remains).

- [ ] **Step 3: Delete the raylib-test directory**

Run via Bash tool:
```bash
git rm -r raylib-test/ 2>&1 | tail -5
```
Expected: many lines listing each removed file. The tracked files (~1591 LOC + Cargo.toml + README.md + tests/ + src/ + resources/) all get removed.

If `raylib-test/target/` or other untracked files linger, remove them via:
```bash
rm -rf raylib-test/
```
to clear the on-disk directory entirely.

- [ ] **Step 4: Verify workspace still builds**

Run: `cargo build --workspace --features full 2>&1 | tail -10`
Expected: clean build. No "no such file or directory" warnings about raylib-test.

- [ ] **Step 5: Verify no stray references to raylib-test in tracked files**

Run: `grep -rnE 'raylib-test|raylib_test' --include='*.rs' --include='*.toml' --include='*.yml' --include='*.md' . 2>&1 | grep -v 'docs/superpowers/' | grep -v '^Binary' | head -20`

Expected: matches only in:
- `.github/workflows/test.yml` (will be cleaned in Task 11).
- `.github/workflows/sanitizers.yml` (will be updated in Task 11).
- `CLAUDE.md` (will be cleaned in Task 12).
- `CHANGELOG.md` historical pre-6.0 entries (leave alone).

If matches appear elsewhere (e.g., a stray `use raylib_test::...` in a salvaged test), that's a real bug — fix it before continuing.

- [ ] **Step 6: Commit Task 10**

```bash
git add -u  # picks up the workspace Cargo.toml + the deletions
git commit -m "$(cat <<'EOF'
feat(workspace)!: delete raylib-test crate, salvage moved in-tree

Removes the entire raylib-test/ directory (~1591 LOC of stale tests
+ Cargo.toml + README + resources/). Also removes raylib-test from
the workspace Cargo.toml exclude list (no longer needed since the
directory is gone).

All 17 still-relevant tests have been salvaged to in-tree Tier-1 +
Tier-2 targets in prior tasks of this workstream:
- raylib/src/core/texture.rs gets 2 Tier-1 Image tests.
- raylib/tests/integration_*.rs gets 6 integration files (one #[test]
  per file per the InitWindow single-init constraint).
- raylib/tests/fixtures/ holds 3 copied assets.

CI cleanup (integration-xvfb job removal + sanitizers.yml comment
update + CLAUDE.md workspace-layout cleanup) lands in subsequent
tasks of this workstream.

BREAKING for raylib-test consumers: the crate is gone. It was never
published, so no downstream impact expected. See
docs/superpowers/notes/spike-raylib-test-delete-or-fix.md for the
decision history.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 11: CI cleanup — remove integration-xvfb job + update sanitizers.yml

**Files:**
- Modify: `.github/workflows/test.yml`
- Modify: `.github/workflows/sanitizers.yml`

- [ ] **Step 1: Find the `integration-xvfb` job in test.yml**

Run: `grep -nE 'integration-xvfb|raylib-test' .github/workflows/test.yml`
Expected: matches around the `:87-103` range showing the job definition + its preamble comments.

- [ ] **Step 2: Remove the entire `integration-xvfb` job + preamble**

Edit `.github/workflows/test.yml`. Delete from the start of the preamble comment (likely `# Window-opening integration tests...`) through the last step of the job. The result should be a clean `test.yml` with no references to `raylib-test` or `integration-xvfb`.

- [ ] **Step 3: Verify test.yml still parses**

Run:
```bash
NODE_PATH=/tmp/yaml-check/node_modules node -e "const yaml=require('js-yaml');const fs=require('fs');yaml.load(fs.readFileSync('.github/workflows/test.yml','utf8'));console.log('OK')" 2>&1 | tail -3
```
Expected: prints `OK`. If `js-yaml` isn't installed at `/tmp/yaml-check/node_modules`, install via `mkdir -p /tmp/yaml-check && cd /tmp/yaml-check && npm install js-yaml --silent --no-audit --no-fund` (matches the WS8c pattern).

- [ ] **Step 4: Update sanitizers.yml comment**

Find the existing comment in `.github/workflows/sanitizers.yml`:
```yaml
# NOTE: the originally-specified model_animation_raii target lives in raylib-test,
# which is stale vs the 6.0 API (see notes/spike-raylib-test-delete-or-fix.md);
```

Replace with:
```yaml
# Runs the ModelAnimations RAII test (raylib/tests/integration_model_animations.rs)
# under ASAN/UBSAN to validate ModelAnimations::Drop doesn't double-free the heap
# array allocated by LoadModelAnimations.
```

- [ ] **Step 5: Verify sanitizers.yml's feature set picks up the new test**

Run: `grep -nE 'software_renderer|features' .github/workflows/sanitizers.yml | head -5`

The sanitizers job must use the `software_renderer` feature + the model-related `SUPPORT_MODULE_*` flags to run the new `integration_model_animations.rs` test. If it doesn't, augment the feature list to match the WS6 software-renderer invocation:

```
software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION
```

If sanitizers.yml already uses this set, no edit needed beyond the comment update.

- [ ] **Step 6: Verify sanitizers.yml still parses**

Run:
```bash
NODE_PATH=/tmp/yaml-check/node_modules node -e "const yaml=require('js-yaml');const fs=require('fs');yaml.load(fs.readFileSync('.github/workflows/sanitizers.yml','utf8'));console.log('OK')" 2>&1 | tail -3
```
Expected: prints `OK`.

- [ ] **Step 7: Commit Task 11**

```bash
git add .github/workflows/test.yml .github/workflows/sanitizers.yml
git commit -m "$(cat <<'EOF'
ci(test,sanitizers): remove integration-xvfb job; update sanitizers comment

- test.yml: remove the entire `integration-xvfb` Linux job + its
  preamble comments. The job was non-required (continue-on-error)
  and has been a no-op since WS6a when raylib-test went stale.
  With raylib-test deleted (Task 10 of this workstream), the job
  is dead.
- sanitizers.yml: update the stale "model_animation_raii target
  lives in raylib-test, which is stale vs the 6.0 API" comment to
  point at the new test location:
  raylib/tests/integration_model_animations.rs (Task 3 of this
  workstream). Verify the workflow's feature set includes
  software_renderer + the SUPPORT_MODULE_* flags needed for the
  new test.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 12: Docs cleanup — CLAUDE.md + spike doc + CHANGELOG

**Files:**
- Modify: `CLAUDE.md`
- Modify: `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Update CLAUDE.md Workspace layout**

Find the existing bullet:
```
- `raylib-test/` — integration tests that open a window. Excluded from workspace; requires nightly. Run from inside the directory.
```

Run: `grep -n 'raylib-test/' CLAUDE.md`
Expected: at least one match in the Workspace layout section.

Remove the entire line. The Workspace layout section should now have one fewer bullet.

- [ ] **Step 2: Update CLAUDE.md status line**

Find the existing status line containing `raylib-test ← NEXT`:

Run: `grep -nE 'raylib-test ← NEXT' CLAUDE.md`

Replace `raylib-test delete-or-fix ← NEXT` (or whatever exact phrasing) with:
```
raylib-test ✅ → UBSAN ← NEXT
```

Preserve the rest of the chain (`UBSAN → rustdoc rewrite → safe abstractions → WS9 showcase → final-release`).

- [ ] **Step 3: Append "Decision" section to the spike doc**

Open `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` and append at the end:

```markdown

## Decision (2026-05-29)

**Outcome:** Option A + salvage. The raylib-test crate is deleted;
the still-relevant tests are migrated to the in-tree Tier-1 / Tier-2
surface (2 Image-API unit tests in `raylib/src/core/texture.rs`;
15 window-coupled tests grouped into 6 integration files under
`raylib/tests/integration_*.rs`).

**Sanitizers coverage preserved** via the new
`raylib/tests/integration_model_animations.rs` target — the
ModelAnimations RAII Drop test that the sanitizers workflow's
WS6a comment explicitly wanted to run.

**Window-real-GLFW coverage retired.** The `integration-xvfb` CI
job is removed. The `software_renderer` Tier-2 tests (rlsw +
Memory platform) remain the gating headless coverage. WS9's
showcase port will provide broad real-API exercise once the
showcase is feature-complete.

**Workstream:** `docs/superpowers/specs/2026-05-29-raylib-test-delete-and-salvage-design.md`
+ `docs/superpowers/plans/2026-05-29-raylib-test-delete-and-salvage.md`.
```

- [ ] **Step 4: Append CHANGELOG ### Internal entry**

Open `CHANGELOG.md` and find the `### Internal` section under `## 6.0.0-rc.1 (unreleased)`. Append:

```markdown
- raylib-test crate removed in favor of in-tree integration tests at
  `raylib/tests/integration_*.rs`. 17 salvaged tests (2 Tier-1 image
  + 15 Tier-2 window-coupled, grouped into 6 files by topic) replace
  the stale nightly-harness crate. Window-real-GLFW coverage via
  xvfb retired; `software_renderer` is the gating headless coverage.
  Sanitizers workflow now targets the new
  `integration_model_animations.rs` directly. See
  `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` for
  the decision history.
```

- [ ] **Step 5: Verify all four docs updated**

Run: `grep -nE 'raylib-test' CLAUDE.md docs/superpowers/notes/spike-raylib-test-delete-or-fix.md CHANGELOG.md | head -20`

Expected: only historical references remain (the spike doc's body, the CHANGELOG entries, and the status line). No "← NEXT" or workspace-layout references.

- [ ] **Step 6: Commit Task 12**

```bash
git add CLAUDE.md docs/superpowers/notes/spike-raylib-test-delete-or-fix.md CHANGELOG.md
git commit -m "$(cat <<'EOF'
docs(raylib-test): close out the delete-or-fix decision

- CLAUDE.md: drop the raylib-test/ workspace-layout bullet (the
  crate is gone). Status line: raylib-test ← NEXT → raylib-test ✅
  → UBSAN ← NEXT.
- docs/superpowers/notes/spike-raylib-test-delete-or-fix.md:
  append "## Decision (2026-05-29)" section recording Option A +
  salvage as the outcome. Preserves the rest of the spike as
  historical record.
- CHANGELOG.md ## 6.0.0-rc.1 (unreleased) ### Internal: new entry
  summarizing the raylib-test retirement, the salvage shape (2
  Tier-1 + 15 Tier-2 across 6 files), and the sanitizers-coverage
  preservation.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 13: Final verification — build, clippy, fmt, rustdoc, mdbook

**Files:** none (verification only)

- [ ] **Step 1: Full workspace builds**

Run: `cargo build --workspace --features full 2>&1 | tail -5`
Expected: `Finished` line, no errors.

- [ ] **Step 2: Tier-1 lib tests pass**

Run: `cargo test -p raylib --lib --features full 2>&1 | tail -10`
Expected: previous count + 2. The 2 new Image tests are included.

- [ ] **Step 3: Tier-2 integration tests pass**

Run:
```bash
cargo test -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION -- --test-threads=1 2>&1 | tail -20
```
Expected: all 6 new integration tests + the 4 pre-existing render_* tests pass.

- [ ] **Step 4: Clippy clean**

Run: `cargo clippy --workspace --features full -- -D warnings 2>&1 | tail -5`
Expected: no warnings.

Also under software_renderer:
```bash
cargo clippy -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_FILEFORMAT_TTF,SUPPORT_FILEFORMAT_OBJ,SUPPORT_MESH_GENERATION -- -D warnings 2>&1 | tail -5
```
Expected: no warnings.

- [ ] **Step 5: Rustfmt clean**

Run: `cargo fmt --check 2>&1 | tail -3`
Expected: zero output, exit 0.

- [ ] **Step 6: Rustdoc clean**

Run: `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps 2>&1 | tail -10`
Expected: clean build.

- [ ] **Step 7: Mdbook builds**

Run: `mdbook build book 2>&1 | tail -5`
Expected: clean (no errors).

No commit in this task (verification only).

---

### Task 14: Final push to fork

**Files:** none (push only)

- [ ] **Step 1: Confirm clean working tree**

Run: `git status --short`
Expected: only the three permitted untracked files (`TODO.md`, `prompt.md`, `next-session-prompt.md`); no `M` or `D` lines.

- [ ] **Step 2: Inventory the commits about to push**

Run: `git log --oneline e988ac1..HEAD`
Expected: ~12 commits (Tasks 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12 each commit; Tasks 9 and 13 are verification-only).

- [ ] **Step 3: Push to fork's 6.0-rc**

Run: `git push fork 6.0-rc 2>&1 | tail -3`
Expected: clean push, range `<prev-SHA>..<new-SHA>  6.0-rc -> 6.0-rc`.

- [ ] **Step 4: Push to fork's unstable**

Run: `git push fork 6.0-rc:unstable 2>&1 | tail -3`
Expected: same SHA range pushed to unstable.

---

## raylib-test delete-and-salvage complete when

- [ ] `raylib/tests/fixtures/` has billboard.png + alagard.png + pixeloid.ttf.
- [ ] `raylib/src/core/texture.rs` has 2 new Tier-1 `#[test]` fns (image_load + image_manipulations).
- [ ] 6 new files exist under `raylib/tests/`: `integration_window_api.rs`, `integration_image_io.rs`, `integration_random_seed.rs`, `integration_fonts.rs`, `integration_model_animations.rs`, `integration_models.rs`. Each has `#![cfg(feature = "software_renderer")]` + one `#[test]` fn using `with_headless(...)`.
- [ ] `raylib-test/` directory deleted entirely.
- [ ] Workspace `Cargo.toml` no longer lists `raylib-test` in `exclude`.
- [ ] `.github/workflows/test.yml` `integration-xvfb` job + preamble removed.
- [ ] `.github/workflows/sanitizers.yml` comment updated; feature set picks up the new test.
- [ ] `CLAUDE.md` workspace-layout `raylib-test` bullet removed; status line: `raylib-test ✅ → UBSAN ← NEXT`.
- [ ] `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` has "## Decision (2026-05-29)" appended.
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Internal` lists the raylib-test retirement entry.
- [ ] Tier-1 tests pass (count grew by 2).
- [ ] Tier-2 integration tests pass (6 new tests).
- [ ] `cargo build --workspace --features full` / `cargo clippy ... -- -D warnings` / `cargo fmt --check` / `RUSTDOCFLAGS=-Dwarnings cargo doc -p raylib --features full --no-deps` / `mdbook build book` all clean.
- [ ] All commits include the `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>` trailer.
- [ ] Pushed to both `fork/6.0-rc` and `fork/unstable`.

Next workstream: **UBSAN-through-FFI** — promoted from long-tail. Wire `-C linker=gcc` + `libubsan` into the sanitizers workflow so UBSAN actually exercises the FFI boundary. Brainstorm starts when the owner is ready.
