# WS3 complete — safe-API raylib 6.0 parity

**Status:** DONE. The safe `raylib` crate compiles clean against raylib 6.0, unit + doc tests pass, and the 3-OS fork CI (`baseline.yml`) is green (fmt + `build-sys` ×3 + `build-safe` ×3). Executed subagent-driven on branch `6.0-rc`, split into WS3a/b/c.

## WS3a — parity checklist + native math-type adoption (commits `a62097e`..`719a93d`)
- **Parity checklist** is now the source of truth: `find_unimplemented.py` classifies every `raylib.h` RLAPI fn → `[x]` implemented / `[~]` skipped (Rust std, with reason) / `[ ]` TODO, emitting `docs/superpowers/parity-checklist.md`. Locked **std-only skip line**: `Text*`/UTF-8/file-IO/path fns are skipped; base64/DEFLATE/CRC/MD5/SHA1 are KEPT (not in std).
- `core/math.rs` now re-exports the native `ffi::{Vector2,Vector3,Vector4,Matrix,Quaternion}`; the hand-rolled safe `Matrix`/`Quaternion` were deleted (the WS2 sys types carry the raymath methods). Kept `Ray`/`BoundingBox`/`RayCollision`/`Transform`/`Rectangle` + `lerp`/`rquat`.
- Removing glam from the safe surface eliminated 22 glam↔native conversion errors, leaving exactly the 37 documented FFI-change errors for WS3b.
- `MintVec2/3/4`/`MintMatrix`/`MintQuat` are now `#[deprecated]` aliases (one-release courtesy bridge); all 169 internal uses across 9 files swept to the native names (zero internal deprecation warnings).
- `raylib/Cargo.toml`: hard `glam` dep dropped; `glam`/`mint`/`serde` are opt-in features forwarding to `raylib-sys`. Default build pulls **zero** math/serialization deps (verified via `cargo tree`).
- Note for future work: `ffi::Matrix` is 16 `m0..m15` floats and derives `Default` (so `Matrix::default()` is the zero matrix); `ffi::Vector4` has no `From<[f32;4]>`.

## WS3b — the 37 FFI-change errors + skeletal-animation RAII (commits `38ac447`..`0b90bdd`)
- **Skeletal-animation RAII redesign (the load-bearing change).** raylib 6.0 removed the singular `UnloadModelAnimation`; the only unload is `UnloadModelAnimations(ptr, count)`, which frees each `keyframePoses[i]`, then `keyframePoses`, then `RL_FREE`s the **array pointer** itself. A per-animation owning wrapper is therefore unsound. New `ModelAnimations` collection **owns** the `LoadModelAnimations` heap array and frees it exactly once on `Drop` (null-guarded); `ModelAnimation` is demoted to a non-owning `#[repr(transparent)]` view (`Deref → [ModelAnimation]`). `load_model_animations` now returns `ModelAnimations`. A focused RAII drop test lives at `raylib-test/tests/model_animation_raii.rs` (asset-gated on `guyanim.iqm`; runs under the WS6 sanitizers workflow).
- `Model.bones/boneCount/bindPose` → `Model.skeleton.*`; `ModelAnimation.frameCount/framePoses` → `keyframeCount/keyframePoses`; per-animation `bones` accessors removed (bones live in the skeleton); `UpdateModelAnimation` `frame` → `f32`.
- **Removed (gone in 6.0):** `DrawModelPoints`/`DrawModelPointsEx` (were in `drawing.rs` `RaylibDraw3D`), `UpdateModelAnimationBones`, the singular `unload_model_animation`, `FilePathList::capacity` (accessors + all struct literals; `AutomationEventList.capacity` is untouched).
- **Signature fixes:** `DrawCircleGradient` takes a `Vector2` center; `LoadFontData` gained a trailing `glyphCount` out-param; `SaveFileTextCallback` trampoline + `DecodeDataBase64` input are `*const i8`.
- Result: `cargo build -p raylib` green (default / `--no-default-features` / `--features glam,mint,serde`); unit 9/9 + doctests 14/14 pass. (A doctest `Matrix::zero()` → `Matrix::default()` and a now-unused import were cleaned up in `0b90bdd`.)

## WS3c — new fns + backlog + Tier-1 tests + CI (commits `22e9aa5`..`f7f2013`)
- **CI:** added a 3-OS `build-safe` job to `baseline.yml` (build + window-independent unit/doc tests; default, `--no-default-features`, `--features glam,mint,serde`). Found and fixed a Linux-only break: `--no-default-features` disables both X11 and Wayland, which raylib's CMake rejects — Linux re-adds `GLFW_BUILD_X11`. **All 7 CI jobs green.**
- **Tier-1 tests (spec D10):** 42 window-independent unit tests across collision (17), color (13), easing (12) + Tween. This surfaced and **fixed a real pre-existing bug**: `ease::quad_in_out` undershot to `b+c/2` at `t=d` (wrong term in the Penner port).
- **Backlog PRs cherry-picked with attribution:**
  - #259 mipmaps accessor returned width → fixed (Amy Wilder).
  - #263 `get_random_value` now `RangeInclusive` matching FFI (Amy Wilder).
  - #252 `get_window_state` (the setters were consumed by-value and discarded — genuinely broken; fixed + `#[must_use]` added to the 14 setters) (zacharysiegel).
  - #250 `export_image_to_memory` leak → returns `DataBuf<[u8]>` (frees on drop) (Dennis Schön).
- **New fn wrapped:** `UpdateModelAnimationEx` (6.0 blended animation; pairs with the RAII work).

## Tracked / deferred (NOT done in WS3 — recorded so nothing is lost)
- **New-fn long tail:** the 49 raylib `[ ]` TODO + 9 raygui in `parity-checklist.md` are the tracked backlog (per the approved "wrap high-value, defer tail" decision). std-equivalent fns stay `[~]`.
- **Idiom-refactor PRs (WS3-tagged, deferred):** #272 (`c"..."` literals), #268 (`Into`→`From`), #266 (seal `AudioSample`).
- **Soundness-adaptation PRs:** #277 (drop unsound trait impls), #257/#256 (mesh accessor null-safety), #118 (texcoords) — fold in during a focused mesh/wrapper pass.
- **WS6:** the `model_animation_raii` test under sanitizers; clippy `-Dwarnings` (one pre-existing warning remains: internal `custom_audio_stream_callback` deprecation) + `deny(missing_docs)`; raygui/rlgl are WS5.

## Handoff
Next: **WS4** — software renderer (`rlsw`) + headless test harness (Tier-2 rendering tests). See `docs/superpowers/notes/spike-rlsw.md`.
