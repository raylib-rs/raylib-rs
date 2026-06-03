# WS4b complete — headless render-test harness + Tier-2 proof tests

**Status:** DONE. Branch `6.0-rc`, pushed to `fork`. 3-OS CI green (run `26488436443`, HEAD `131c6c3`): the `software-render` job's **Tier-2 headless render tests** step passes on ubuntu, macOS, and windows. WS4 (a+b) is now complete.

Plan: `docs/superpowers/plans/2026-05-27-ws4b-headless-render-harness.md`. Built on the WS4a `software_renderer` foundation (`PLATFORM=Memory` / rlsw, see `notes/spike-rlsw.md` and the WS4a section in the memory note).

## What shipped

- **`raylib/src/test_harness.rs`** (feature-gated `pub mod test_harness;` in `lib.rs`, `#[cfg(feature = "software_renderer")]`). No `unsafe` — it only composes existing safe wrappers:
  - `with_headless(w, h, |rl, thread| …)` — one-shot windowless init via `init().size().title().build()`; tears down on `RaylibHandle` Drop. **Single-init per process** (raylib panics on a second `InitWindow`).
  - `render_frame(rl, thread, |d| …) -> Image` — draws inside a scoped `begin_drawing` handle (EndDrawing flushes rlsw on drop) then reads back via `rl.load_image_from_screen(thread)`.
  - `Px { r, g, b, a }`, `pixel_at(img, x, y) -> Px`, `assert_pixel(img, x, y, expected, tol)` — tolerance probe that compares **R/G/B only** (alpha deliberately ignored; see findings).
- **`raylib/tests/render_shapes.rs`** — one `#[test]`: clears background, draws rectangle/circle/line, probes 4 pixels.
- **`raylib/tests/render_text.rs`** — one `#[test]`: draws `"Hi"` with the embedded default font, counts near-white pixels (measured 96; threshold `> 40`).
- **`.github/workflows/baseline.yml`** — Tier-2 step in the `software-render` job.

Commits: `e54b87f` (harness) · `9f51324` (assert_pixel RGB-only fix) · `982f152` (harness doc polish) · `5bf1717` (tests) · `19d81c7` + `8b18105` (test comment/threshold polish) · `da1aad8` (CI step) · `131c6c3` (final-review doc polish).

## Durable findings (don't re-derive)

1. **The software-renderer readback (`load_image_from_screen`) is BGRA + Y-inverted — and it's DETERMINISTIC ON EVERY OS, not platform-specific.** Both effects are pure compile-time C, so Linux/macOS/Windows behave identically (confirmed: same hardcoded probes pass on all three in CI):
   - **R↔B swap:** rlsw is built with its upstream default `#define SW_FRAMEBUFFER_OUTPUT_BGRA true` (`raylib-sys/raylib/src/external/rlsw.h`), and `rlReadScreenPixels` (`rlgl.h`) labels the buffer `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8` without reordering. So a drawn `Color::RED {255,0,0}` reads back as `{0,0,255}`.
   - **Inverted Y:** `rlReadScreenPixels` flips vertically unconditionally; combined with rlsw's buffer origin the result is `y_img = (h-1) - y_screen`.
   - The WS4a kickoff's "vertical flip handled internally" note was about dims/validity, not orientation — WS4a never checked pixel colors/positions. WS4b did.
2. **The safe `raylib` crate links symbols from ALL FIVE modules**, so a `-p raylib` test binary needs every `SUPPORT_MODULE_*` enabled or it fails to link (notably the MSVC linker, which doesn't drop unresolved externals — see the build.rs note). The WS4b kickoff only listed RTEXTURES/RSHAPES/RTEXT; **RMODELS and RAUDIO are also required.**
3. **Default font** is loaded unconditionally at `InitWindow` when `SUPPORT_MODULE_RTEXT` is on (`rcore.c`, `LoadFontDefault()` gated only by RTEXT). raylib 6.0 has **no** `SUPPORT_DEFAULT_FONT` flag, so text rendering works in a `--no-default-features` build as long as RTEXT is enabled.
4. **`assert_pixel` ignores alpha on purpose:** the Memory framebuffer can report alpha that doesn't match an opaque `Color`'s `a`, which would make probes fail spuriously.

## Reproduce locally

Harness build: `cargo build -p raylib --no-default-features --features software_renderer`

Tier-2 tests (the exact CI command):
```
cargo test -p raylib --no-default-features \
  --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO \
  --test render_shapes --test render_text -- --test-threads=1
```

## Handoff to WS5 (and showcase)

- The `test_harness` module is the reusable basis for WS5 (raygui/rlgl) and showcase render-verification.
- **Open ergonomics question for WS5:** every render test currently must compensate for the BGRA swap + Y-flip itself. Consider whether `render_frame` (or a variant) should *normalise* the readback to true top-left RGBA so callers use natural draw coordinates/colors. Deferred deliberately — the WS4b plan scoped tests to "adapt to real rlsw output and note it," not to change the readback. Decide in WS5 with this finding in hand.
- WS3 tracked-deferred items (idiom PRs #272/#268/#266, soundness #277/#257/#256/#118, the ~49+9 new-fn tail, the `custom_audio_stream_callback` deprecation warning that blocks WS6 `-Dwarnings`, the model-animation RAII test under sanitizers) remain folded into WS5/WS6.
