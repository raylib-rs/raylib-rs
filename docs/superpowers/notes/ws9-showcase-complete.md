# WS9 complete — raylib-rs 6.0 showcase finale

**Status:** DONE on `Dacode45:6.0-rc` (fork; canonical merge deferred to the future final-release workstream).

Spec: `docs/superpowers/specs/2026-05-31-ws9-showcase-design.md` (578 lines, 10 locked decisions + 8 sub-decisions).
Plan: `docs/superpowers/plans/2026-06-01-ws9-showcase.md` (~3290 lines, 34 tasks across 6 phases).

Builds on WS8 (`docs/superpowers/notes/ws8-complete.md`) — version bumps + release workflows + samples/ removal.

This is the publishable artifact for raylib-rs 6.0: a Pages gallery of **229 examples** (217 raylib core + 12 raygui) with in-canvas source viewer + GitHub deep-links per source, plus per-chapter "See also" footers and a new Examples appendix in the mdBook.

Live gallery: <https://dacode45.github.io/raylib-rs/>.

---

## Coverage

| Category | C examples | Rust ports | wasm-buildable | wasm-excluded |
|---|---|---|---|---|
| audio | 11 | 11 | 10 | 1 |
| core | 49 | 49 | 44 | 5 |
| models | 30 | 30 | 25 | 5 |
| others | 3 | 3 | 1 | 2 |
| raygui | 12 | 12 | 9 | 3 |
| shaders | 35 | 35 | 28 | 7 |
| shapes | 41 | 41 | 41 | 0 |
| text | 16 | 16 | 16 | 0 |
| textures | 32 | 32 | 31 | 1 |
| **total** | **229** | **229** | **205** | **24** |

`showcase/wasm-exclude.toml` is the source of truth for the 24 desktop-only entries. Reasons cluster around: glsl330-pinned shaders (no glsl100 fallback), MRT / depth-texture sampling / gl_FragDepth / compute shaders not in WebGL, LoadDroppedFiles + native dialogs, audio recording, screen recording (GIF encoder), VR stereo, custom-frame-control, async clipboard.

## Phases

### P0 — scaffolding (tag `ws9-p0-complete` at `f947edc`)

What shipped: workspace + `showcase` member, `build.rs` registry + examples_meta.json sidecar, `SourceViewer` overlay (F1 toggle + tab swap + scroll), 4 xtask bins (`gen_thumbnails`, `xtask_wasm_build`, `xtask_build_pages`, `xtask_vendor_resources`), reference port (`core/core_basic_window`), 2 new CI workflows (`showcase.yml` 3-OS desktop + Linux wasm, `pages.yml` deploys to `dacode45.github.io/raylib-rs/`), Tier-1 registry tests.

Key design fixes caught during the P0 work:

1. `build.rs` pairing warn-by-default — missing Rust ports surface as `cargo:warning=showcase: …` lines, not `panic!`, so incremental porting through P2 doesn't block CI.
2. `SourceViewer::for_current_example` resolves the example name at runtime via `std::env::current_exe().file_stem()` (not `env!("CARGO_BIN_NAME")` which is undefined in lib code).
3. `viewer.update` is **two-arg** (`&mut rl, &thread`), not one — the plan template was outdated.
4. Resources live at `showcase/resources/<cat>/<file>` so per-cat paths translate `"resources/<file>"` → `"resources/<cat>/<file>"`.
5. `showcase.yml` wasm-build leg needs the full X11/audio/GL apt stack for the native preflight build that populates `examples_meta.json`.
6. `xtask_build_pages` uses an existence-checking `find_examples_meta` helper (Cargo creates multiple `raylib-showcase-<hash>` build dirs).

### P1 — resource vendor (tag `ws9-p1-complete` at `8e97276`)

381 resource files / 48.1 MB mirrored under `showcase/resources/<cat>/...` preserving upstream paths. Attribution skeleton at `showcase/resources/README.md`; per-file CC-BY/CC0 callouts added as resources surfaced during P2/P3.

### P2 — raylib core ports (217 across 4 waves)

**Wave 1** — tag `ws9-p2-w1-complete`: audio (11) + others (3). 14 ports + the xtask required-features fix.

**Wave 2** — tag `ws9-p2-w2-complete`: core (49) + text (16). 64 ports across 16 commits, plus 4 fix commits for `core_3d_camera_fps` clamp persistence, `core_keyboard_testbed` transmute UB, `core_compute_hash`/`core_delta_time` C-output mismatches, and ~29 missing SAFETY comments.

**Wave 3** — tag `ws9-p2-w3-complete`: models (30) + shaders (35). 65 ports across 13 commits + 1 fix commit for the shaders batch-A wasm over-exclusion (12 examples re-enabled via `#[cfg(target_family = "wasm")]` GLSL_VERSION fork) + 2 SAFETY comments in `models_decals`.

**Wave 4** — tag `ws9-p2-w4-complete`: shapes (41) + textures (32). 73 ports across 18 commits + 1 lerp-param-order fix in `shapes_starfield_effect` + 1 borrow-checker fix in `shapes_rounded_rectangle_drawing`.

**P2 close** — tag `ws9-p2-complete`: flipped `WS9_STRICT_PAIRING=1` in `showcase.yml` env so missing pairs escalate to a build break (a new upstream C example can never silently land without a Rust port).

### P3 — raygui ports (12 across 4 commits)

Tag `ws9-p3-complete` at `5e143d9`. All 12 raygui-examples submodule entries ported with `required-features = ["raygui"]`. 3 wasm-exclusions (`custom_file_dialog`, `portable_window`, `image_importer_raw`).

Notable simplifications (each `// SIMPLIFIED:` in source):
- `animation_curve` — 543-line `GuiCurveEditor` helper not in public FFI → replaced with hardcoded easing curves.
- `controls_test_suite` — `gui_value_box_float` helper ported as inline Rust function.
- `custom_input_box` — `GuiFloatBox` (private raygui state) → safe `gui_value_box_float` wrapper.
- `custom_sliders` — six private-state slider variants → horizontal sliders in two preserved group boxes.
- `custom_file_dialog` — 624-line `gui_window_file_dialog.h` → `gui_text_input_box` modal + drag-and-drop.
- `property_list` — 868-line `dm_property_list.h` → stacked panel of safe widgets (check_box, spinner, slider, text_box, combo, color_panel).
- `floating_window` — `BeginScissorMode` wrap dropped (the scissor handle isn't a `RaylibDraw` impl).

Vendored 13 `.rgs` style files at `showcase/resources/raygui/styles/` (the existing `xtask_vendor_resources` doesn't traverse the top-level `raygui-examples/styles/` directory).

Final review caught a UB-prone `unsafe { transmute::<i32, PixelFormat>(combo_idx+1) }` in `image_exporter`; replaced with an explicit `match` (no transmute, no UB).

### P4 — Pages polish + book cross-links + GitHub deep-links

Tag `ws9-p4-complete` (after this note is filed). Commits:

- `91d1d6f` — bake submodule + repo URLs into `SourcePair` + `examples_meta.json`. `build.rs` resolves: raylib submodule via `git config -f .gitmodules submodule.raylib-sys/raylib.url` + `git -C raylib-sys/raylib rev-parse HEAD`; same for raygui; `git remote get-url origin` + `git rev-parse HEAD` for the Rust ports. Generated URLs:
  - `c_url` (raylib): `https://github.com/raysan5/raylib/blob/<sha>/examples/<cat>/<name>.c`
  - `c_url` (raygui): `https://github.com/raysan5/raygui/blob/<sha>/examples/<name>/<name>.c`
  - `rust_url`: `https://github.com/raylib-rs/raylib-rs/blob/<sha>/showcase/examples/<cat>/<name>.rs`
- `d5bc250` — categorized Pages index with thumbnail tiles + per-tile "C / Rust" GitHub deep-link strip + name filter + desktop-only badge + placeholder pages for wasm-excluded examples.
- `47da463` — `SourceViewer` overlay shows a "Source on GitHub: <url>" footer line per tab.
- `466f10f` — `xtask_wasm_build` passes `--shell-file showcase/index/example_shell.html` via `EMCC_CFLAGS` for every emscripten build (wraps the canvas in gallery chrome + back-nav + F1 hint).
- `bb18be4` — `book/src/modules/*.md` gained a "See also → Showcase examples" sub-section in 12 chapters (window-and-drawing, input, shapes, textures-and-images, text-and-fonts, 3d-models, audio, raymath, collision, raygui, rlgl, callbacks-and-logging).
- `0039731` — new `book/src/appendix/showcase-examples.md` (264 lines) — full 229-entry inventory grouped by category with deep links; wired into `SUMMARY.md` as a top-level Appendix section.

### P5 — skill + done-note + CHANGELOG + status flip

This note + the new `raylib-showcase-port-flow` skill + the CHANGELOG entry + the CLAUDE.md status flip + tag `ws9-complete`.

## CI inventory (all 7 workflows green on `fork/6.0-rc`)

| Workflow | What it does |
|---|---|
| `check.yml` | fmt + clippy `-Dwarnings` + cargo-deny + MSRV 1.85 + doc-link gates |
| `test.yml` | cargo-nextest unit + integration + doctests across 3 OS |
| `web.yml` | wasm32-unknown-emscripten build of `raylib` |
| `sanitizers.yml` | ASAN/UBSAN over FFI (informational) |
| `book.yml` | mdbook build + link check |
| `showcase.yml` | 3-OS desktop build of all 229 examples + Linux wasm build (205 wasm-buildable) — strict-pairing on |
| `pages.yml` | xtask-build-pages + deploy to `dacode45.github.io/raylib-rs/` |

## Lessons learned (folded into the new `raylib-showcase-port-flow` skill)

1. **Visual parity is the reviewer's primary criterion.** Blank-line groupings, comment positions, init/loop/close structure must match the C verbatim. Translate comments; preserve location. Don't collapse phases. Index-for loops stay index-based unless an iterator chain genuinely reads better (with `// idiomatic:` comment).
2. **Every `unsafe { … }` block requires a `// SAFETY: …` comment.** Two recurring patterns: "pure FFI value-in/out" and "owner outlives slice".
3. **Transmuting C magic numbers to typed Rust enums is UB.** Caught twice (`core_keyboard_testbed` placeholder 162; `image_exporter` combo→PixelFormat). Fix: explicit `match` returning known-valid variants, with a sentinel `None` for placeholder values.
4. **Cargo `required-features` must propagate to every iterator.** `xtask_wasm_build` and `gen_thumbnails` originally iterated `examples_meta.json` without consulting per-example `required-features`; raygui-gated examples failed with exit 101. Fix in `4147f51`: parse `showcase/Cargo.toml`, build a name → features map, append `--features X,Y`.
5. **GLSL_VERSION: mirror the C cfg-fork, don't blanket-pin.** Many shader examples have BOTH glsl330 and glsl100 versions; the Rust port should `#[cfg(target_family = "wasm")] const GLSL_VERSION: i32 = 100;` else `330`. Wave 3 caught a shader sub-batch that pinned all to 330 and wasm-excluded 12 unnecessarily; reviewer flagged → fixed.
6. **Reviewers must build with `--features raygui`.** A raygui-gated example silently passed the default-feature build (cargo skips required-features-disabled examples without erroring) but failed under `--features raygui`. Wave 4 caught `shapes_rounded_rectangle_drawing` borrow-checker error this way.
7. **Subagent dispatch order:** implementers sequential per category batch (all touch `showcase/Cargo.toml`; parallel causes write conflicts), reviewers parallel (read-only).
8. **Helper headers (curve-editor, file-dialog, property-list):** when a raygui example pulls in a multi-hundred-line `.h` helper that depends on private raygui state, port a simplified inline version with a `// SIMPLIFIED:` comment. The reviewer accepts simplifications when documented.
9. **wasm-exclude.toml is for desktop-only APIs, not laziness.** LoadDroppedFiles / native dialogs / threads / audio recording / GIF encoder / VR stereo / MRT / gl_FragDepth / depth-texture sampling / compute shaders / filesystem polling / hot-reload / async clipboard.
10. **WS9_STRICT_PAIRING:** flipped on at P2 close so new upstream C examples surface as build breaks rather than warn-only noise.

## Tracked-deferred (not blocking, follow-up)

- **Safe-API gaps surfaced during ports (each documented inline with `// SAFETY:` to the unsafe ffi fallback):**
  - `Image::gen_image_*` family cfg-gated on `SUPPORT_IMAGE_GENERATION` (not propagated through showcase); used by ~6 examples.
  - `Font::texture_mut()` missing; `RaylibHandle::load_codepoints` is `pub(crate)`; `load_font_data` returns single `GlyphInfo` not the full array; `load_font_ex` accepts only `&str` not codepoint slice; `get_font_default` outside `&self rl`; `RlMatrix<'_, T>` doesn't impl `RaylibDraw` / `RaylibRlgl`.
  - rlgl-compute family (`rlLoadComputeShaderProgram`, `rlComputeShaderDispatch`, `rlLoadShaderBuffer`, `rlBindShaderBuffer`) not in safe wrapper.
  - rlgl FBO / depth attachment family (`rlLoadFramebuffer`, `rlFramebufferAttach`, `rlLoadTextureDepth`, `rlUnloadFramebuffer`) not in safe wrapper.
  - `DrawMeshInstanced`, `GetRayCollisionMesh`, `UpdateModelAnimation`, `AttachAudioStreamProcessor`/`Detach`, `SetTraceLogCallback`, `GetClipboardImage` (Windows-cfg-gated wrapper exists), `GuiTextBoxProperty` missing from `impl GuiProperty` list, gamepad transmute (WS6b-tracked).
- **`models_skybox_rendering` non-HDR branch only** (HDR cubemap generation needs rlgl framebuffer plumbing the safe wrapper doesn't expose).
- **`shaders_shadowmap_rendering` aspect-ratio nit** — manual `rlEnableFramebuffer + rlViewport` skips the `CORE.Window.currentFbo.width/height` update that `BeginTextureMode` performs; the subsequent `BeginMode3D(light_camera)` uses the main-window aspect rather than the shadowmap's. Desktop-only (wasm-excluded), no crash, but a visible divergence.
- **`shapes_clock_of_clocks` / `shapes_digital_clock` show UTC, not local time.** The C uses `localtime`; Rust uses `std::time::SystemTime` (no zero-dep TZ). Documented inline.
- **`models_animation_blending` omits `PROGRESS_SIDE` raygui style override** (raygui binding gap; blend bar always Left→Right).
- **`models_animation_blend_custom` runs CPU-skinning fallback verbatim** regardless of `SUPPORT_GPU_SKINNING` feature.
- **`core_screen_recording` is a partial port** — visuals match but CTRL+R toggles a flag instead of writing a GIF.
- **Three audio examples carry intentional `unused_assignments` warnings** on `let mut time_played: f32 = 0.0;` for parity with C's initialize-then-overwrite pattern.
- **Thumbnail generation not yet integrated into the Pages build** — `gen_thumbnails` exists but `pages.yml` doesn't run it (CI baseline doesn't have OpenGL for software-renderer either). Pages tiles currently show CSS placeholder gradients; thumbnails populate when generated locally and pushed.

## Final-release prep

The `final-release` workstream picks up from here:

1. Bump `raylib`/`raylib-sys` `6.0.0-rc.1` → `6.0.0`; flip CHANGELOG `## 6.0.0 (unreleased)` → `## 6.0.0 — 2026-MM-DD`; sync `CLAUDE.md` MSRV / version blurbs.
2. Open the canonical merge PR from `Dacode45:6.0-rc` → `raylib-rs:unstable`.
3. Tag `v6.0.0` on the canonical merge.
4. Run `release-sys.yml` (`dry_run: false`) — publishes `raylib-sys` 6.0.0 to crates.io.
5. Wait for crates.io to index → run `release-safe.yml` (`dry_run: false`) — publishes `raylib` 6.0.0.
6. Flip Pages deploy from `Dacode45/ms-raylib-rs` to `raylib-rs/raylib-rs` (update `pages.yml` environment + the SourceViewer `rust_url` baseline once HEAD lands on canonical).
7. Announce.

After release: `Dacode45`'s bevy-raylib crate (see `docs/superpowers/notes/ws8e-checkpoint-review-feedback.md`) + the flexible queue.
