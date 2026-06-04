# raylib-rs Changelog

## Unreleased

### Added

- Opt-in `log` feature: `RaylibBuilder::log_to_rust()` forwards raylib's
  `TraceLog` output into the [`log`](https://docs.rs/log) facade (target
  `"raylib"`), making `RUST_LOG`-style filtering the single source of
  truth. Levels map `TRACE/DEBUG/INFO/WARNING→trace/debug/info/warn`,
  `ERROR`+`FATAL→error`. See the *Callbacks and logging* book chapter.

## 6.0.0-rc.2 — 2026-06-02

Upgrade from raylib 5.x to **raylib 6.0**. MSRV bumped to **1.85** (edition 2024).

> Release candidate 2 for the 6.0 line — published as `6.0.0-rc.2` to
> get wider eyes on the canonical merge before the final `6.0.0` cut.
> The headings under this block describe what will ship in `6.0.0`
> final; `rc.2` is the verbatim source snapshot of the current canonical
> `unstable` HEAD. The final-cut commit will flip this header to
> `## 6.0.0 — YYYY-MM-DD` and bump the Cargo.toml versions accordingly.

### Highlights

- raylib C source bumped to 6.0; bindings regenerated. `raylib-sys` compiles against 6.0 without modification.
- Math types are now native `#[repr(C)]` Rust structs (`Vector2`/`Vector3`/`Vector4`, `Matrix`, `Quaternion`) with **zero math-crate dependencies by default**. `mint`, `glam`, `serde` are opt-in features.
- Skeletal-animation API redesigned around RAII (`ModelAnimations` collection; the singular `UnloadModelAnimation` is gone in 6.0, making per-item ownership unsound).
- New `software_renderer` feature wires raylib's `rlsw` (Platform::Memory) backend for fully headless rendering — no GPU or window required.
- raygui at 6.0 parity (57/57 functions, module split into grouped sub-traits, `impl AsRef<str>` + thread-local scratch buffer); new safe immediate-mode `rlgl` module (`RlMatrix`/`RlImmediate` RAII guards, `&Texture2D`/`&Shader` bind helpers).
- Layered CI: `check.yml` / `test.yml` / `web.yml` / `sanitizers.yml` / `book.yml`; quality hard-gates (fmt, clippy `-Dwarnings`, `deny(missing_docs)`, cargo-deny, MSRV 1.85) fail on violation.
- mdBook docs at `book/` — 28 chapters covering quickstart, platform build guides, core concepts, and per-module chapters. WS9 added per-module "See also" footers + a new Showcase examples appendix.
- **WS9 showcase finale** — new `showcase` workspace crate at `showcase/` ports **229** raylib examples (217 raylib core + 12 raygui) to idiomatic raylib-rs under a visual-parity rule. Each port carries an in-canvas F1 source-viewer overlay with C-vs-Rust tabs and a "Source on GitHub" deep-link footer (URLs derived at build time from `.gitmodules` + submodule SHAs). Deployed as a Pages gallery at <https://raylib-rs.github.io/raylib-rs/> with thumbnail tiles, per-tile C/Rust GitHub links, name filter, and per-example emscripten output wrapped in gallery chrome via a shared `example_shell.html`. CI matrix gate is `WS9_STRICT_PAIRING=1` — missing pairs escalate to build break, not warn-only noise.

### Breaking

- **MSRV is now 1.85** (edition 2024).
- `MintVec2`/`MintVec3`/`MintVec4`/`MintMatrix`/`MintQuat` are `#[deprecated]` — use the native types directly. The `mint` feature opt-in remains.
- `glam`/`mint`/`serde` are **no longer default-on** for `raylib-sys`; enable them explicitly as optional features.
- **raygui** module split into grouped sub-traits (`RaylibGuiState`, `RaylibGuiContainers`, `RaylibGuiControls`, `RaylibGuiAdvanced`, `RaylibGuiIcons`); control-label parameters are now `impl AsRef<str>` (no `CStr` required).
- **Skeletal-animation loading** returns a `ModelAnimations` RAII wrapper (owns the heap array; frees all frames on drop). The old per-animation owning pattern is removed.
- **Removed in 6.0:** `DrawModelPoints` / `DrawModelPointsEx` (no 6.0 replacement); `UpdateModelAnimationBones` (superseded by redesigned `UpdateModelAnimation(model, anim, frame: f32)`); the singular `unload_model_animation`; `FilePathList::capacity` (removed from the C struct); the `custom_audio_stream_callback` trampoline (`set_audio_stream_callback` on `RaylibHandle`; the generic callback in `callbacks/` is the live path).
- **`SUPPORT_*` feature flag set reconciled with raylib 6.0 `config.h`:** removed `SUPPORT_GIF_RECORDING`, `SUPPORT_IMAGE_MANIPULATION`, `SUPPORT_DEFAULT_FONT`, `SUPPORT_FONT_ATLAS_WHITE_REC`, `SUPPORT_TEXT_MANIPULATION`, `SUPPORT_STANDARD_FILEIO` (unconditional in 6.0), `SUPPORT_DISTORTION_SHADER`, `SUPPORT_FONT_TEXTURE`, `SUPPORT_VR_SIMULATOR` from the feature list; added `SUPPORT_FILEFORMAT_PNM` and `SUPPORT_GPU_SKINNING` (both default-off in 6.0).
- **Signature changes (6.0 ABI):** `DrawCircleGradient` takes a `Vector2` center instead of separate `i32 x, y`; `UpdateModelAnimation` `frame` is now `f32`; `LoadFontData` gained a trailing `glyphCount` out-parameter; `SaveFileTextCallback` trampoline `text` is now `*const i8`; `DecodeDataBase64` input is `*const i8`.
- `Image::gen_image_*` family is now cfg-gated on `SUPPORT_IMAGE_GENERATION` (MSVC link fix).
- **`samples/` directory removed.** Migration: see [`showcase/`](./showcase) for runnable Rust ports of raylib's C examples. Anyone running `cd samples && cargo run --bin <name>` against the pre-release 6.0-rc branch should switch to `showcase/` instead. The WS9 finale of the 6.0 effort completes the port of all upstream examples and publishes the gallery as a GitHub Pages site.
- `RaylibGuiIcons::gui_get_icons_raw` removed; use `gui_get_icons` / `gui_get_icons_mut`.
- `RaylibGuiIcons::gui_load_icons_raw` removed; use `gui_load_icons` / `gui_load_icons_with_names`.
- `SetLogError` (core/callbacks) is replaced by `SetCallbackError` in `core::error` — a `thiserror` struct without the artificial lifetime parameter. The ~10 `set_*_callback` functions/methods now return `Result<(), SetCallbackError>`.
- `RaylibError` is now `#[non_exhaustive]` and gained `#[from]` variants for `UpdateAudioStreamError`, `InvalidMeshError`, `GenMeshError`, `Base64Error`, `LoadIconsError`, `LoadStyleFromMemoryError`, `PixelColorError`, and `SetCallbackError` — every leaf error now composes via `?`. Exhaustive matches on `RaylibError` need a wildcard arm.
- The identity `impl From<&Color> for Color` is removed — `&Color` no longer satisfies `Into<Color>` bounds (e.g. on draw functions). Dereference instead: `d.draw_x(.., *c)`. `Color` is `Copy`; no other type had such an impl.

### Added

- Wrapper-family lifetime test pass: new `databuf_lifetimes` (windowless Tier-1, ASAN+LeakSanitizer CI target) and `render_alloc_lifetimes` (Tier-2) test binaries; the canonical Tier-2 feature list now includes `SUPPORT_MESH_GENERATION` so Mesh tests actually run in CI.
- `software_renderer` feature + `raylib::test_harness` module (`with_headless`, `render_frame`, `render_frame_raw`, `pixel_at`, `assert_pixel`).
- Safe `rlgl` module (`raylib::rlgl`): `rl_begin` / `rl_draw` / `rl_push_matrix` returning RAII guards (`RlImmediate`, `RlMatrix`); render-state toggles; `&Texture2D`/`&Shader` bind helpers.
- `Vector2::{ZERO,ONE}` / `Vector3::{ZERO,ONE,X,Y,Z}` constants.
- `UpdateModelAnimationEx` (6.0 blended animation).
- `export_image_to_memory` returns `DataBuf<[u8]>` (frees on drop, no leak).
- `get_random_value` now takes `RangeInclusive` matching the FFI semantics.
- `get_window_state` fixed; all 14 window-state setter methods are `#[must_use]`.
- Tier-1 unit tests: 42 window-independent tests across collision (17), color (13), easing (12 + Tween).
- Tier-2 headless render tests via `software_renderer`: `render_shapes`, `render_text`, `render_gui`, `render_rlgl`.
- `full` feature alias — curated max-capability set for `raylib` and `raylib-sys`.
- `deny.toml` — cargo-deny license allowlist + RUSTSEC vulnerability/unsound gates.
- `book/` — mdBook with 28 chapters: introduction, quickstart, 4 platform install guides, 5 Core Concepts, 14 Modules, 2 Ecosystem chapters.
- Rustdoc enriched on ~25 high-traffic types: crate-level docs, `RaylibHandle`/`RaylibThread`/`RaylibBuilder`, `RaylibDraw` trait, `Color`/`Rectangle`, `Image`/`Texture2D`/`RenderTexture2D`, `Mesh`/`Model`/`Material`/`ModelAnimations`, `RaylibAudio`/`Wave`/`Sound`/`Music`/`AudioStream`, `Shader`, `Font`, `Vector2`/`Vector3`/`Vector4`/`Matrix`/`Quaternion`, collision module, `test_harness` module, rgui module, rlgl module.
- **Shapes:** `RaylibDraw::draw_line_dashed` (dashed line; raylib 6.0 addition), `RaylibDraw::draw_ellipse_v` and `RaylibDraw::draw_ellipse_lines_v` (Vector2-center ellipse variants) — cheatsheet-audit follow-up.
- **Input:** `RaylibHandle::get_key_name` — keyboard-layout-aware key labels (e.g. `"q"` for `KEY_A` on AZERTY); useful for HUDs and key-rebinding UIs. Returns `Option<String>` (copied out of raylib's static buffer).
- **Color/pixel:** new module `raylib::core::pixel` with safe wrappers over raylib's pixel-pointer C functions:
  - `get_pixel_color(bytes: &[u8], format: PixelFormat) -> Result<Color, PixelColorError>`
  - `set_pixel_color(bytes: &mut [u8], color: Color, format: PixelFormat) -> Result<(), PixelColorError>`
  - `bytes_per_pixel(format: PixelFormat) -> Option<usize>` helper (exhaustive match — adding a `PixelFormat` variant in a future raylib release fails the build)
  - `PixelColorError` (thiserror) with `InsufficientBytes` and `CompressedFormat` variants
  All four are re-exported through `raylib::prelude`. Closes the `pixel-pointers` workstream from the cheatsheet-parity audit.
- **Hashes:** new module `raylib::core::hashes` with safe wrappers
  over raylib's built-in hash functions:
  - `compute_crc32(data: &[u8]) -> u32` — CRC-32/ISO-HDLC, free-thread.
  - `compute_md5(thread: &RaylibThread, data: &[u8]) -> [u8; 16]` — MD5
    digest in canonical byte order. Requires `&RaylibThread` to pin
    the call to raylib's thread (the C function returns a pointer to
    a shared static buffer that concurrent calls would race against).
  - `compute_sha1(thread: &RaylibThread, data: &[u8]) -> [u8; 20]`
  - `compute_sha256(thread: &RaylibThread, data: &[u8]) -> [u8; 32]`
  All four re-exported through `raylib::prelude`. Module-level rustdoc
  steers security-sensitive callers at the RustCrypto crates
  (`crc32fast`, `md-5`, `sha1`, `sha2`); MD5 and SHA-1 are
  cryptographically broken, and raylib's SHA-256 implementation is
  not constant-time. Closes the `hashes` workstream from the
  cheatsheet-parity audit.
- **Mixed audio bus:** new public API for closure-driven processors on
  raylib's global mixed audio bus:
  - `attach_audio_mixed_processor(audio: &RaylibAudio, processor: &mut F) -> Pin<Box<MixedAudioProcessorCallback<'_, F>>>`
    where `F: FnMut(&mut [f32], u32) + Send + 'static`.
  - `MixedAudioProcessorCallback` — the RAII guard type. Dropping it
    calls `DetachAudioMixedProcessor` and frees the closure slot,
    mirroring the WS8e per-stream-processor soundness fix.
  - Multiple mixed-bus processors can be attached simultaneously
    (they chain in raylib's internal linked list). They share the
    same 30-slot trampoline pool with the per-stream processors.
  - Re-exported via `raylib::prelude`. Closes the `mixed-audio`
    workstream from the cheatsheet-parity audit.
- **Showcase crate (`raylib-showcase`):**
  - 229 runnable Rust ports of raylib's C examples organized as `[[example]]` targets under `showcase/examples/<category>/`. Categories: audio (11), core (49), models (30), others (3), raygui (12), shaders (35), shapes (41), text (16), textures (32).
  - `SourceViewer` overlay (`raylib_showcase::SourceViewer`) — F1 toggles a full-canvas C/Rust source view with tab swap, PageUp/PageDown scroll, and a "Source on GitHub: <url>" footer link per tab (URLs baked into the static registry at build time).
  - Build-time `SourcePair` registry generated by `showcase/build.rs` — walks `raylib-sys/raylib/examples/` and `raylib-sys/raygui-examples/examples/`, enforces 1:1 C↔Rust pairing (warn-only by default; `WS9_STRICT_PAIRING=1` escalates to build break), emits a phf map keyed by example name with `c_url` and `rust_url` deep-links resolved from `.gitmodules` + each submodule's pinned SHA.
  - Four xtask binaries: `gen_thumbnails` (software-renderer-driven screenshots), `xtask_wasm_build` (iterates emscripten builds with `--shell-file` injection), `xtask_build_pages` (assembles `showcase/_site/` with categorized grid, filter, and per-example pages), `xtask_vendor_resources` (one-shot resource mirror).
  - `wasm-exclude.toml` — 24 entries flagging desktop-only examples (gl_FragDepth, MRT, compute shaders, audio/screen recording, native dialogs, VR stereo, etc.); these render as "desktop only" badges in the gallery and as placeholder pages instead of wasm artifacts.
- **Book — showcase cross-links and appendix:**
  - 12 module chapters (`window-and-drawing`, `input`, `shapes`, `textures-and-images`, `text-and-fonts`, `3d-models`, `audio`, `raymath`, `collision`, `raygui`, `rlgl`, `callbacks-and-logging`) gain a `### Showcase examples` sub-section under their existing `## See also` heading, listing 3-6 representative examples linking to the deployed gallery.
  - New `book/src/appendix/showcase-examples.md` — the full 229-entry inventory grouped by category with deep links to the gallery; wasm-excluded entries flagged with "— desktop only".
  - `book/src/SUMMARY.md` gained a top-level Appendix section linking to the new inventory.
- **`raylib-showcase-port-flow` skill** (`docs/superpowers/skills/raylib-showcase-port-flow/`) — captures the canonical WS9 port workflow + 15 specific lessons learned (visual parity, SAFETY-comment rule, transmute UB pitfalls, required-features propagation, GLSL_VERSION cfg-mirror pattern, subagent dispatch ordering, etc.) for future sessions.
- **rgui icons and style-from-memory safe abstractions:**
  - `RaylibGuiIcons::gui_get_icons(&self) -> &[[u32; 8]; 256]` and `gui_get_icons_mut(&mut self) -> &mut [[u32; 8]; 256]` — safe typed-grid access to raygui's internal icons buffer.
  - `RaylibGuiIcons::gui_load_icons` / `gui_load_icons_with_names` (file) and `gui_load_icons_from_memory` / `gui_load_icons_from_memory_with_names` (in-memory) — safe `.rgi` loaders with Rust-side header validation; `GuiLoadIconsFromMemory` wrapped under the hood.
  - `RaylibGuiState::gui_load_style_from_memory` — safe wrapper around `GuiLoadStyleFromMemory` (PR #296's intent; upstream raygui#549).
  - Public constants `raylib::rgui::RAYGUI_ICON_MAX_ICONS` (= 256) and `raylib::rgui::RAYGUI_ICON_DATA_ELEMENTS` (= 8).
  - Error enums `LoadIconsError` and `LoadStyleFromMemoryError` in `raylib::core::error` (`thiserror`-based).

### Changed

- raygui (vendored at `raylib-sys/binding/raygui.h`) hand-patched to expose `GuiLoadStyleFromMemory` (raysan5/raygui#549 is merged upstream but unreleased; last tagged raygui release is v4.0 / 2023-09-11).
- raygui now shares raylib's allocator: `RAYGUI_MALLOC` / `_CALLOC` / `_FREE` route through `RL_MALLOC` / `_CALLOC` / `_FREE` via `binding/rgui_wrapper.c`.

### Fixed

- `DataBuf::<[T]>::alloc_from_clone` now performs a real element-wise clone (`T: Clone`; was bound `T: Copy` and identical to `alloc_from_copy`). A `clone()` panic mid-initialization drops the cloned prefix and frees the allocation before propagating.
- `compress_data(b"")`, `decompress_data` on empty/invalid input, and `decode_data_base64(b"")` no longer panic (raylib returns a non-null, zero-length buffer for these; the wrappers now free it and return `Err` — the contract is documented on each fn).
- **Mesh-accessor soundness** — all 10 `RaylibMesh` slice accessors guarded against null/zero (were `slice::from_raw_parts(null, n)`); `indices`/`indices_mut` corrected to `triangleCount * 3` (was `vertexCount`); 4 safe `texcoords`/`texcoords2` accessors added (PR #257 / #118 / #256 with attribution).
- **Sound unsound impls** — removed `AsRef`/`AsMut<ffi::AudioStream> for Sound` which exposed raw pointer fields to safe mutation (from PR #277, partial — full refactor deferred).
- **`c"..."` literal modernization** (PR #272, AmityWilder) — `CStr::from_bytes_with_nul` replaced by C-string literals in audio and file modules.
- **`Into`→`From` idiom** (PR #268, AmityWilder) — 11 `impl Into<T> for U` → `impl From<U> for T` across color, camera, texture, vr modules.
- **`AudioSample` sealed** (PR #266, AmityWilder) — `AudioSample` is now a sealed trait (closes #213).
- **Platform enum acronyms** — `Platform::{DRM,RPI}` → `{Drm,Rpi}`, `PlatformOS::{BSD,OSX}` → `{Bsd,Osx}` to satisfy `clippy::upper_case_acronyms`.
- **mipmaps accessor** — returned width instead of mipmap count (PR #259, AmityWilder).
- **`ease::quad_in_out` bug** — undershot at `t=d` due to wrong term in the Penner port (surfaced by Tier-1 test).
- **Broken intra-doc links** — 17 pre-existing broken links fixed by WS6a's `RUSTDOCFLAGS=-Dwarnings` sweep.
- **Docs: color cheatsheet** — `color.rs` docstrings aligned with the raylib cheatsheet (PR #284, LBreede).
- **Docs: logging module-doc** — `logging.rs` outer doc comment corrected (PR #273, AmityWilder).
- **Issue #291** — broken `RaylibHandle::draw` API in docs: resolved by PR #152 (already merged before 6.0 work); confirmed by `RUSTDOCFLAGS=-Dwarnings` CI gate.
- **Issue #290** — broken docs.rs links: resolved by MSRV bump to Rust 1.85 (rustdoc re-export path fix) + `RUSTDOCFLAGS=-Dwarnings` CI gate.
- **Audio:** `DetachAudioStreamProcessor` lifecycle — the user-data audio stream processor wrapper (`stream_processor_with_user_data_wrapper.rs`) now calls the C-side `DetachAudioStreamProcessor` with the matching trampoline pointer *before* clearing the closure slot. Previously only the slot was freed, leaving raylib iterating its processor list against dangling state.

### Deferred (tracked, not in 6.0.0)

- **Full PR #277 wrapper-soundness refactor** — remove macro-generated `AsRef`/`AsMut`/`Deref`/`DerefMut` on pointer-owning wrappers; convert `impl AsRef<ffi::X>` param bounds to typed references (WS3-scale).
- **`get_gamepad_button_pressed` transmute** (`input.rs`) — `transmute::<u32, GamepadButton>` where `GamepadButton` is `#[repr(i32)]`; latent UB on out-of-range return; small fix.
- **`raylib-test` delete-or-fix** — stale integration-test suite; `integration-xvfb` CI job is non-required; decision deferred to WS9.
- **`rlsw` on wasm32** — `software_renderer` + emscripten is currently a `compile_error!` guard; a real fix requires re-ordering `platform_from_target` in `build.rs`.
- **UBSAN through the FFI boundary** — C-side UBSAN runtime link fails under `rust-lld`; informational only (requires `-C linker=gcc`/`libubsan`).
- **`paste` alternative** — cargo-deny flags this direct dep as unmaintained; rewrite or library swap tracked for a future workstream (accepted with rationale in `deny.toml`).
- **Full rustdoc rewrite** of remaining 208 stub-level items — selective enrichment only in WS7; future passes can extend.
- **Public Pages deploy** of book + showcase gallery — WS9 shipped the showcase gallery deploy at <https://raylib-rs.github.io/raylib-rs/>; canonical book + gallery deploy under `raylib-rs.github.io` lands with the final-release publish step.
- **Thumbnail generation in CI** — `gen_thumbnails` exists and runs locally via the `software_renderer` feature, but Pages CI doesn't have the rlsw-on-emscripten link path yet (deferred with the existing `rlsw on wasm32` item). Tiles render with CSS placeholder gradients until thumbnails are generated locally and pushed.

### Internal

- Long-lived `6.0-rc` branch; single merge to `raylib-rs/unstable` at WS8 — never piecemeal.
- `baseline.yml` retired; replaced by layered `check`/`test`/`web`/`sanitizers`/`book` workflows.
- Parity checklist at `docs/superpowers/parity-checklist.md` tracks every `raylib.h` RLAPI function.
- Accepted `paste 1.0` cargo-deny unmaintained advisory (RUSTSEC-2024-0436) with rationale in `deny.toml`. Rewrite or library swap tracked for a future workstream.
- Dropped `structopt 0.3` dev-dependency (only used by the removed `samples/` binaries); clears the cargo-deny unmaintained advisory for that crate.
- `raylib-test` crate removed in favor of in-tree integration tests at
  `raylib/tests/integration_*.rs`. 17 salvaged tests (2 Tier-1 image
  + 15 Tier-2 window-coupled, grouped into 6 files by topic) replace
  the stale nightly-harness crate. Window-real-GLFW coverage via
  xvfb retired; `software_renderer` is the gating headless coverage.
  Sanitizers workflow now targets the new
  `integration_model_animations.rs` directly. See
  `docs/superpowers/notes/spike-raylib-test-delete-or-fix.md` for
  the decision history.

## 5.7.0
- More improved ergonomics
- REFACTOR: Everything that interfaces with `raylib-sys` **has to use mint vectors** because as it has the most common supported interface type in the rust ecosystem. (tl;dr Replaced `ffi::Vectors -> mint::Vectors`)
- REFACTOR: Everything that interfaces with raylib safe bindings will often **take as input mint vectors but output/store glam-rs vectors**(storing them in Camera, Mesh, Boundingbox, etc), This can easily be swapped out. (tl;dr Replaced: `math::core::Vectors -> glam::Vectors`. glam Matrix and Quat however are incompatible so these stay as is)
	- BREAKING: Code that uses `Vector4` constructors break, instead use `Vector4::new()
- REFACTOR:  `Camera3D` ported methods in favor of calling ffi for easier maintenance reasons
- REFACTOR: trace_log from needing to be on the RaylibHandle and take &self
- MOVED: `color.rs` to `raylib-sys` because having 2 versions of this simple structure is pointless
- MOVED: `Rectangle` to `raylib-sys` because having 2 versions of this simple structure is pointless
- REMOVED: needless `target_os = windows` for rlgl getting&setting matrix functions
- BUGFIX : `build.rs` gen_utils function generated the `util_log.c` as `rgui`making raygui not work
- Removed: Removed imgui from being a feature on `raylib-sys`, instead check [imgui example](https://github.com/raylib-rs/raylib-rs/blob/unstable/samples/imgui.rs) for integration
## build script changes:
- Blacklist Vector2, Vector3, Vector4, Matrix, Quaternion, Rectangle, Color from generating in bindgen as they are replaced by mint and manual implementations
- BUGFIX: Fixed bug where `utils_log` compiled as "rgui" making the rust build fail in some cases
- Prevent android builds from turning on GLFW flags
- Added/exposed various feature flags
- Invert `bindgen` feature flag to `nobindgen` since its a more saner default


## 3.7.0

- [core] ADDED: LoadVrStereoConfig()
- [core] ADDED: UnloadVrStereoConfig()
- [core] ADDED: BeginVrStereoMode()
- [core] ADDED: EndVrStereoMode()

- [core] ADDED: GetCurrentMonitor() (#1485) by @object71
  [core] ADDED: SetGamepadMappings() (#1506)
- [core] RENAMED: struct Camera: camera.type to camera.projection
- [core] RENAMED: LoadShaderCode() to LoadShaderFromMemory() (#1690)
- [core] RENAMED: SetMatrixProjection() to rlSetMatrixProjection()
- [core] RENAMED: SetMatrixModelview() to rlSetMatrixModelview()
- [core] RENAMED: GetMatrixModelview() to rlGetMatrixModelview()
- [core] RENAMED: GetMatrixProjection() to rlGetMatrixProjection()
- [core] RENAMED: GetShaderDefault() to rlGetShaderDefault()
  [core] RENAMED: GetTextureDefault() to rlGetTextureDefault()
  [core] REMOVED: GetShapesTexture()
  [core] REMOVED: GetShapesTextureRec()
  [core] REMOVED: GetMouseCursor()
- [core] REMOVED: SetTraceLogExit()
  [core] REVIEWED: GetFileName() and GetDirectoryPath() (#1534) by @gilzoide
  [core] REVIEWED: Wait() to support FreeBSD (#1618)
  [core] REVIEWED: HighDPI support on macOS retina (#1510)
  [core] REDESIGNED: GetFileExtension(), includes the .dot
  [core] REDESIGNED: IsFileExtension(), includes the .dot
  [core] REDESIGNED: Compression API to use sdefl/sinfl libs
- [rlgl] ADDED: SUPPORT_GL_DETAILS_INFO config flag
  [rlgl] REMOVED: GenTexture\*() functions (#721)
  [rlgl] REVIEWED: rlLoadShaderDefault()
  [rlgl] REDESIGNED: rlLoadExtensions(), more details exposed
  [raymath] REVIEWED: QuaternionFromEuler() (#1651)
  [raymath] REVIEWED: MatrixRotateZYX() (#1642)
- [shapes] ADDED: DrawLineBezierQuad() (#1468) by @epsilon-phase
  [shapes] ADDED: CheckCollisionLines()
- [shapes] ADDED: CheckCollisionPointLine() by @mkupiec1
  [shapes] REVIEWED: CheckCollisionPointTriangle() by @mkupiec1
  [shapes] REDESIGNED: SetShapesTexture()
- [shapes] REDESIGNED: DrawCircleSector(), to use float params
- [shapes] REDESIGNED: DrawCircleSectorLines(), to use float params
- [shapes] REDESIGNED: DrawRing(), to use float params
- [shapes] REDESIGNED: DrawRingLines(), to use float params
- [textures] ADDED: DrawTexturePoly() and example (#1677) by @chriscamacho
- [textures] ADDED: UnloadImageColors() for allocs consistency
  [textures] RENAMED: GetImageData() to LoadImageColors()
  [textures] REVIEWED: ImageClearBackground() and ImageDrawRectangleRec() (#1487) by @JeffM2501
  [textures] REVIEWED: DrawTexturePro() and DrawRectanglePro() transformations (#1632) by @ChrisDill
  [text] REDESIGNED: DrawFPS()
- [models] ADDED: UploadMesh() (#1529)
  [models] ADDED: UpdateMeshBuffer()
  [models] ADDED: DrawMesh()
  [models] ADDED: DrawMeshInstanced()
  [models] ADDED: UnloadModelAnimations() (#1648) by @object71
  :( [models] REMOVED: DrawGizmo()
- [models] REMOVED: LoadMeshes()
  [models] REMOVED: MeshNormalsSmooth()
- [models] REVIEWED: DrawLine3D() (#1643)
  [audio] REVIEWED: Multichannel sound system (#1548)
  [audio] REVIEWED: jar_xm library (#1701) by @jmorel33
  [utils] ADDED: SetLoadFileDataCallback()
  [utils] ADDED: SetSaveFileDataCallback()
  [utils] ADDED: SetLoadFileTextCallback()
  [utils] ADDED: SetSaveFileTextCallback()
  [examples] ADDED: text_draw_3d (#1689) by @Demizdor
  [examples] ADDED: textures_poly (#1677) by @chriscamacho
  [examples] ADDED: models_gltf_model (#1551) by @object71
  [examples] RENAMED: shaders_rlgl_mesh_instanced to shaders_mesh_intancing
  [examples] REDESIGNED: shaders_rlgl_mesh_instanced by @moliad
  [examples] REDESIGNED: core_vr_simulator
  [examples] REDESIGNED: models_yaw_pitch_roll
  [build] ADDED: Config flag: SUPPORT_STANDARD_FILEIO
  [build] ADDED: Config flag: SUPPORT_WINMM_HIGHRES_TIMER (#1641)
  [build] ADDED: Config flag: SUPPORT_GL_DETAILS_INFO
  [build] ADDED: Examples projects to VS2019 solution
  [build] REVIEWED: Makefile to support PLATFORM_RPI (#1580)
  [build] REVIEWED: Multiple typecast warnings by @JeffM2501
  [build] REDESIGNED: VS2019 project build paths
  [build] REDESIGNED: CMake build system by @object71
  [*] RENAMED: Several functions parameters for consistency
  [*] UPDATED: Multiple bindings to latest version
  [*] UPDATED: All external libraries to latest versions
  [*] Multiple code improvements and fixes by multiple contributors!

## 3.5.0 (Done)

Added: SetWindowState
Added: ClearW‌indowState
Added: IsWindowFocused
Added: GetWindowScaleDPI
Added: GetMonitorRefreshRate
Added: IsCursorOnScreen
Added: SetMouseCursor/GetMouseCursor
Added: Normalize
Added: Remap
Added: Vector2Reflect
Added: Vector2LengthSqr
Added: Vector2MoveTowards
Added: UnloadFontData
Added: LoadFontFromMemory(ttf)
Added: ColorAlphaBlend
Added: GetPixelColor
Added: SetPixelColor
Added: LoadImageFromMemory
Added: LoadImageAnim
Added: DrawTextureTiled
Added: UpdateTextureRec
Added: UnloadImageColors,
Added: UnloadImagePallet,
Added: UnloadWaveSample
Added: DrawTriangle3D
Added: DrawTriangleStrip3D
Added: LoadWaveFromMemory
Added: MemAlloc() / MemFree()
Added: UnloadFileData
Added: UnloadFileText

## 0.10.0 (WIP)

- Basic macOS support. Currently untested.
- Improved ergonomics across the board:
  - Copied over and tweaked many FFI structs so that fields use proper types instead of FFI types.
  - Added `vec2`, `vec3`, `quat`, `rgb`, and `rgba` convenience functions for a middle ground between `From` conversion and `new` methods.
  - Changed several key and gamepad functions to use `u32`, making it more ergonomic with key/gamepad constants.
  - Added optional `prelude` module for conveniently bringing in all the common types and definitions.
- Fixed unnecessary `&mut` in `load_image_ex` and `draw_poly_ex`.
- Fixed linking on MSVC toolchains by including `user32`.
- Prevent `RaylibHandle` from being manually constructed. Fixes a safety soundness hole.

## 0.9.1

- Fixed docs.rs build by removing use of a uniform module path. This also keeps the crate compatible with Rust 1.31+.

## 0.9.0

- Initial crates.io release
