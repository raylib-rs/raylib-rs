# Software renderer

raylib 6.0 adds `rlsw`, a software renderer backend that renders into an
in-memory framebuffer with no GPU or window required.  raylib-rs exposes it
via the `software_renderer` Cargo feature and the [`raylib::test_harness`]
module — a set of helpers that initialise a windowless context, draw a frame,
read the framebuffer back, and let you probe pixel values.

This is the mechanism behind raylib-rs's Tier-2 render tests: the CI
`software-render` job runs on all three platforms (ubuntu/macOS/windows)
without a display server.

## API surface

- **`software_renderer` Cargo feature** — enables the rlsw backend.  Add
  `features = ["software_renderer"]` to your `Cargo.toml` dependency.
- [`test_harness::with_headless(w, h, body)`](https://docs.rs/raylib/latest/raylib/test_harness/fn.with_headless.html) —
  initialise a `w × h` windowless context, run `body(rl, thread)`, then tear
  down.  Call **at most once per test process** (raylib is single-init per
  process).
- [`test_harness::render_frame(rl, thread, draw)`](https://docs.rs/raylib/latest/raylib/test_harness/fn.render_frame.html) —
  draw one frame via the `draw` closure and return a **normalized top-left RGBA**
  [`Image`].  Coordinates and colors match what you drew: `Color::RED` at
  screen `(x, y)` reads back as red at `(x, y)`.
- [`test_harness::render_frame_raw(rl, thread, draw)`](https://docs.rs/raylib/latest/raylib/test_harness/fn.render_frame_raw.html) —
  raw readback: BGRA bytes, Y-inverted.  Use only when you need the unprocessed
  rlsw output.
- [`test_harness::pixel_at(img, x, y)`](https://docs.rs/raylib/latest/raylib/test_harness/fn.pixel_at.html) —
  read the pixel at `(x, y)` as a [`Px`] with `r/g/b/a` fields.
- [`test_harness::assert_pixel(img, x, y, expected, tol)`](https://docs.rs/raylib/latest/raylib/test_harness/fn.assert_pixel.html) —
  assert the RGB channels at `(x, y)` match `expected` within a per-channel
  tolerance `tol`.  Alpha is intentionally ignored (the Memory platform
  readback does not guarantee alpha fidelity).

## Example

The `software_renderer` feature is **mutually exclusive** with the `opengl_*`
features (and with the `full` alias used by the book's CI).  The example below
is marked `ignore` because it can't compile in the standard book test run.
Enable it by building with `--features software_renderer` (not combined with
any `opengl_*` feature) and running your tests.

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

- **`software_renderer` is mutually exclusive with `opengl_*`.**
  Do not combine these features.  The `full` feature alias enables the OpenGL
  backend, so `--features full,software_renderer` will conflict.  Use a
  separate Cargo profile or test target (e.g., `[profile.test]` with a feature
  flag) to run software-renderer tests.
- **`software_renderer` is mutually exclusive with `wasm32-unknown-emscripten`
  (tracked-deferred).**
  rlsw-on-Emscripten is not yet supported; see `docs/superpowers/notes/ws6b-complete.md`.
- **All 5 raylib modules must be linked.**
  The harness requires
  `SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO`
  to be enabled (they are by default).  Disabling any of them causes a link
  error; see the memory note `software-renderer-headless-testing`.
- **`with_headless` is single-init.**
  raylib can only be initialised once per process.  In a test suite, wrap all
  headless tests inside a single `with_headless` call or use a process-global
  init strategy (e.g., `std::sync::OnceLock`).

## See also

- [Features and platforms](../core-concepts/features.md) — feature flag reference.
- [`test_harness` docs.rs](https://docs.rs/raylib/latest/raylib/test_harness/index.html)
- `docs/superpowers/notes/ws4b-complete.md` — WS4b harness design rationale.
- `docs/superpowers/notes/ws5-complete.md` — readback normalization details.
