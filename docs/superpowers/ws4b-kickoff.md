# WS4b Kickoff — Headless Render-Test Harness + Tier-2 Proof Tests (fresh-session brief)

You are resuming the **raylib 6.0 upgrade**. WS0, WS1, WS2a, WS2b, **WS3 (a/b/c)**, and **WS4a** are **done and CI-green** on branch `6.0-rc`. This is your starting point for **WS4b**. (Background is in `CLAUDE.md` — loaded every session — and the roadmap spec; this file is the WS4b-specific working brief.)

## What WS4b is

Build a small **headless render-test harness** on top of the WS4a `software_renderer` feature, then prove it with **Tier-2 rendering tests** (spec D10) that draw primitives and assert on the in-memory framebuffer. This unlocks GPU-free rendering verification on every OS — the basis for WS5 (raygui/rlgl) render tests and the showcase.

**Done = the `test_harness` module exists, Tier-2 shape + text render tests pass headlessly, and the fork 3-OS CI runs them green.**

## The plan (follow it)

- **Authoritative plan:** `docs/superpowers/plans/2026-05-27-ws4b-headless-render-harness.md` — execute it task-by-task with **superpowers:subagent-driven-development** (fresh subagent per task, spec + quality review, controller verifies, frequent commits with `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`). Same model as WS0–WS4a.
- Tasks: (1) `raylib/src/test_harness.rs` feature-gated module — `with_headless` one-shot init + `render_frame` draw/readback + `assert_pixel` tolerance probes; (2) Tier-2 proof tests `raylib/tests/render_{shapes,text}.rs`; (3) run them in the CI `software-render` job.

## The validated WS4a foundation (don't re-derive — this is proven)

- **Feature:** `software_renderer` builds raylib with cmake `PLATFORM=Memory` (rlsw CPU rasteriser + windowless memory platform; no GPU/display/GLFW). Mutually exclusive with `opengl_*`/`drm` (enforced by a `compile_error!`). It is **off by default**; build with `--no-default-features --features software_renderer`.
- **Readback API is CONFIRMED:** `LoadImageFromScreen()` returns the rlsw framebuffer as an `Image` (correct dims, non-null, `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8`, vertical flip handled internally). It lives in **rtextures**, so the test/CI build must enable `SUPPORT_MODULE_RTEXTURES`. (rcore_memory.c also keeps a `platform.pixels` RGBA8888 copy updated by `SwapScreenBuffer`, if a raw-pointer path is ever needed — but use `LoadImageFromScreen`.)
- **WS4a smoke test** (`raylib-sys/tests/software_renderer_smoke.rs`) proves windowless `InitWindow`/`IsWindowReady`/draw/`LoadImageFromScreen` works on ubuntu/macOS/windows. Log shows `RLSW: Software renderer initialized successfully`.

## Key facts that shape WS4b

- **Locked decision:** Tier-2 assertions are **pixel-probe + tolerance** (assert specific pixels/regions ≈ expected `Color` within a per-channel tolerance) — **NOT golden-image snapshots**. No binary artifacts in git.
- **Single-init constraint:** raylib allows one `InitWindow` per process (enforced by `RaylibHandle`). Cargo runs `#[test]` fns in a binary on multiple threads → multiple inits would panic. So: each Tier-2 test **file** = **one** `#[test]` that inits once (via `with_headless`), draws several primitives, and probes; run with `-- --test-threads=1`. The harness's `with_headless` encapsulates the one-shot init + teardown (RaylibHandle Drop calls CloseWindow).
- **Module-feature gating for tests:** the safe crate's `draw_*`/`draw_text` wrappers call FFI symbols from `rshapes`/`rtext`. Building a *test binary* links those, so the test/CI feature set must enable the needed modules: at least `software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT` (RTEXT for the embedded default font used by `draw_text`). Confirm exact names in `raylib-sys/Cargo.toml`.
- **API names to verify against the real code before writing the harness** (the plan's snippets are illustrative): `init().size().title().build()` → `(RaylibHandle, RaylibThread)`; `rl.begin_drawing(&thread)` → the draw handle type and `RaylibDraw` trait; `load_image_from_screen` (exact name/signature in `core/texture.rs`); the `Image` pixel-access method (`get_image_data`/`get_color`/colors — read `core/texture.rs`). Match the real signatures.

## Suggested verify commands

- Build harness: `cargo build -p raylib --no-default-features --features software_renderer`
- Run Tier-2: `cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT --test render_shapes --test render_text -- --test-threads=1`
- The CI `software-render` job in `.github/workflows/baseline.yml` is where these run on 3 OSes (extend its steps).

## Working model (unchanged — see CLAUDE.md)

Branch `6.0-rc` in the fork (`fork` remote = `Dacode45/ms-raylib-rs`). **Push to the fork to run CI**; one merge to `raylib-rs/unstable` only at the very end (WS8). Workflow: (decisions already locked — pixel-probe; no further open decisions expected) → **subagent-driven-development** per the WS4b plan → push → drive the 3-OS `software-render` CI green. Watch CI with `gh run watch <id> --exit-status` (make it the *last* command so its exit status is what's reported).

## After WS4b

WS4 is then complete. Next per the roadmap: **WS5** (raygui + rlgl safe modules, render-verified via this harness) / **WS6** (full platform + CI/CD matrix, clippy `-Dwarnings`, `deny(missing_docs)`, the model-animation RAII test under sanitizers) / **WS7** (docs) — these parallelize. Tracked-deferred items from WS3 (idiom PRs #272/#268/#266, soundness PRs #277/#257/#256/#118, the ~49 new-fn tail in `parity-checklist.md`, the pre-existing `custom_audio_stream_callback` deprecation warning) are folded into WS5/WS6.
