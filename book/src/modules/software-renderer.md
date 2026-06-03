# Software renderer

raylib 6.0 adds `rlsw`, a software renderer backend that renders into an
in-memory framebuffer with no GPU or window required.  raylib-rs exposes it
via the `software_renderer` Cargo feature and the `test_harness` module
([source](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs))
— a set of helpers that initialise a windowless context, draw a frame,
read the framebuffer back, and let you probe pixel values.  Note: `test_harness`
is gated on `#[cfg(feature = "software_renderer")]` and does not appear on
docs.rs (docs.rs builds with only the `nobuild` feature).

This is the mechanism behind raylib-rs's Tier-2 render tests: the CI
`software-render` job runs on all three platforms (ubuntu/macOS/windows)
without a display server.

## API surface

- **`software_renderer` Cargo feature** — enables the rlsw backend.  Add
  `features = ["software_renderer"]` to your `Cargo.toml` dependency.
- [`with_headless(w, h, body)`](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs) —
  initialise a `w × h` windowless context, run `body(rl, thread)`, then tear
  down.  Call **at most once per test process** (raylib is single-init per
  process).
- [`render_frame(rl, thread, draw)`](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs) —
  draw one frame via the `draw` closure and return a **normalized top-left RGBA**
  `Image`.  Coordinates and colors match what you drew: `Color::RED` at
  screen `(x, y)` reads back as red at `(x, y)`.
- [`render_frame_raw(rl, thread, draw)`](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs) —
  raw readback: BGRA bytes, Y-inverted.  Use only when you need the unprocessed
  rlsw output.
- [`pixel_at(img, x, y)`](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs) —
  read the pixel at `(x, y)` as a `Px` value with `r/g/b/a` fields.
- [`assert_pixel(img, x, y, expected, tol)`](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs) —
  assert the RGB channels at `(x, y)` match `expected` within a per-channel
  tolerance `tol`.  Alpha is intentionally ignored (the Memory platform
  readback does not guarantee alpha fidelity).

## Example

The example below is `rust,ignore` because raylib's CI build of the rlib that
backs `mdbook test` uses the `full` feature, which falls through to
`PLATFORM=Desktop` with the OpenGL backend.  `test_harness` lives under
`#[cfg(feature = "software_renderer")]`, so it isn't compiled into that rlib.
To actually run a software-renderer test, compile the crate with
`--no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_IMAGE_GENERATION,raygui`
(this is what `check.yml`/`test.yml` do for the Tier-2 render tests).  The WS9
showcase pipeline is the eventual home for runnable demos.

```rust,ignore
# extern crate raylib;
use raylib::prelude::*;
use raylib::test_harness::{with_headless, render_frame, assert_pixel};

#[test]
fn red_rectangle_center_pixel() {
    with_headless(256, 256, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_rectangle(64, 64, 128, 128, Color::RED);
        });

        // Center of the rectangle should be red.
        assert_pixel(&img, 128, 128, Color::RED, 0);
        // A corner well outside the rectangle should be black.
        assert_pixel(&img, 4, 4, Color::BLACK, 0);
    });
}
```

## Gotchas

- **`software_renderer` is not enabled by the `full` feature alias.**
  The `full` alias explicitly excludes mutually-exclusive backend selectors
  (`opengl_*`, `sdl`, `wayland`, `drm`, `software_renderer`) per
  `raylib/Cargo.toml`.  A `--features full` build therefore defaults to
  `PLATFORM=Desktop` with OpenGL — `test_harness` won't be compiled in.
  To use the software renderer, build with
  `--no-default-features --features software_renderer,...` (see the canonical
  command below).  The `compile_error!` in `raylib-sys/build.rs` only fires
  when an explicit `opengl_*` (or `drm`) feature is combined with
  `software_renderer`; `full` alone does not trigger it.
- **`software_renderer` is mutually exclusive with `wasm32-unknown-emscripten`
  (tracked-deferred).**
  rlsw-on-Emscripten is not yet supported; see `docs/superpowers/notes/ws6b-complete.md`.
- **All required raylib modules must be linked.**
  The harness requires
  `SUPPORT_MODULE_RSHAPES`, `SUPPORT_MODULE_RTEXTURES`, `SUPPORT_MODULE_RTEXT`,
  `SUPPORT_MODULE_RMODELS`, `SUPPORT_MODULE_RAUDIO`, plus
  `SUPPORT_IMAGE_GENERATION` (the safe `gen_image_*` family is currently
  ungated; MSVC link fails without it).  The `raygui` feature is also enabled
  in CI so `render_gui` tests link cleanly.  Disabling any required module
  causes a link error; see the memory note `software-renderer-headless-testing`.
- **`with_headless` is single-init.**
  raylib can only be initialised once per process.  Run software-renderer
  tests with:
  ```text
  cargo test -p raylib --no-default-features \
    --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_IMAGE_GENERATION,raygui \
    -- --test-threads=1
  ```
  `--test-threads=1` is required because the harness initialises raylib's
  platform layer once per process.  In a test suite, wrap all headless tests
  inside a single `with_headless` call or use a process-global init strategy
  (e.g., `std::sync::OnceLock`).

## See also

- [Features and platforms](../core-concepts/features.md) — feature flag reference.
- [`test_harness` source](https://github.com/raylib-rs/raylib-rs/blob/unstable/raylib/src/test_harness.rs) — module is cfg-gated on `software_renderer`; not surfaced on docs.rs.
- `docs/superpowers/notes/ws4b-complete.md` — WS4b harness design rationale.
- `docs/superpowers/notes/ws5-complete.md` — readback normalization details.
