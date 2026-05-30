# WS3 Kickoff — Safe-API 6.0 Parity (fresh-session brief)

You are resuming the **raylib 6.0 upgrade**. WS0, WS1, WS2a, WS2b are **done and CI-green** on branch `6.0-rc`. This brief is your starting point for **WS3**. (Background context is in `CLAUDE.md` — loaded every session — and in the roadmap spec; this file is the WS3-specific working brief.)

## What WS3 is

Bring the **safe `raylib` crate back to green against raylib 6.0** and make it 6.0-parity:
1. **Adopt the native `raylib-sys` types as the public API** — `raylib::Vector2/3/4/Matrix/Quaternion` become the `raylib_sys` types (WS2 built these). This retires glam from the safe crate's public surface.
2. **Fix the 37 captured compile errors** (the worklist).
3. **Wrap (or consciously defer) the new 6.0 functions** (+40 filesystem, +30 text).
4. **Cherry-pick the WS3-tagged backlog PRs** with attribution.
5. **Tier-1 unit tests** for window-independent wrappers (collision, color, file/text, math already covered in sys).

**Done = the `raylib` crate compiles clean, tests pass, and the fork 3-OS CI is green** (the full safe-API surface, not just `raylib-sys`).

## Orienting artifacts (read these)

- **The exact 37-error worklist:** `docs/superpowers/notes/ws1-breakage-baseline.md` — grouped by category with per-symbol action items. This is your primary checklist.
- **Roadmap + decisions D1–D13:** `docs/superpowers/specs/2026-05-25-raylib-rs-6.0-roadmap-design.md`.
- **Backlog triage (which PRs to fold into WS3):** `docs/superpowers/inventory.md`.
- **What WS2 delivered (the native types + raymath wrappers you'll adopt):** `raylib-sys/src/{math.rs,vector_math.rs,matrix_quat_math.rs}`, plus optional `mint_conv.rs`/`glam_conv.rs`.
- **Memory:** `raylib-6.0-effort.md` (state) + `prefer-bindgen-generated-types.md` (a standing preference).

## The shape of the change (key facts)

- **Public-type flip:** `raylib/src/core/math.rs` currently aliases `Vector2 = glam::Vec2` (lines ~25–27) and **hand-rolls `Quaternion` (struct+impl ~54–470) and `Matrix` (~451–960)**. In 6.0 those types + their math live in `raylib-sys`. WS3: re-export the sys types (`pub use ffi::{Vector2, Vector3, Vector4, Matrix, Quaternion};`) and **delete the redundant hand-rolled safe Matrix/Quaternion** (the sys versions have the raymath methods now). Keep `Ray`/`BoundingBox`/`RayCollision`/`Transform`/`Rectangle` wrappers.
- **The `MintVec*` indirection collapses.** `raylib/src/lib.rs` defines `MintVec2=ffi::Vector2` … and the API threads `impl Into<MintVec2>` through **~172 call sites across 9 files** (105 `MintVec2`, 57 `MintVec3`, 8 `MintMatrix`, …; heaviest in `core/drawing.rs`). Since the public `Vector2` now *is* `ffi::Vector2`, `MintVec2` == `Vector2`. Decide (see Open Decisions) whether to **deprecate** `MintVec*` as aliases or **remove** them; either way sweep `impl Into<MintVec2>` → `impl Into<Vector2>`.
- **glam coupling is tiny:** only the 3 type aliases in `math.rs` + ~10 method calls (`.normalize()`, `.dot()`, …) that **now resolve to the native raymath-wrapper methods** WS2a added. Remove the hard `glam` dep from `raylib/Cargo.toml`; expose `glam`/`mint`/`serde` as optional features that forward to `raylib-sys/<feat>`.
- **6.0 API specifics to handle (from the worklist):**
  - **Skeletal animation (RAII-sensitive!):** `Model.bones/boneCount/bindPose` → `Model.skeleton.{...}`; `ModelAnimation.frameCount`→`keyframeCount`, `framePoses`→`keyframePoses` (`ModelAnimPose = Transform*`). The `Drop`/unload paths must use **`UnloadModelAnimations`** (plural) — get this right to avoid use-after-free. Add a focused RAII test.
  - **Removed:** `DrawModelPoints`/`DrawModelPointsEx` (remove/retire the safe wrappers).
  - **Renamed/changed:** `UnloadModelAnimation`→`UnloadModelAnimations`; `DrawCircleGradient` now takes a `Vector2` center; `LoadFontData` gained a trailing `glyphCount: *mut i32`; `UpdateModelAnimation` `frame` is `f32`; `SetSaveFileTextCallback` text arg is `*const i8`; `DecodeDataBase64` pointer type.

## Suggested task ordering (for the WS3 plan)

1. Flip `core/math.rs` to re-export sys types; delete redundant hand-rolled Matrix/Quaternion; fix the ~10 internal glam method calls (they map to native methods).
2. Resolve `MintVec*` in `lib.rs` (deprecate or remove per decision) + sweep `impl Into<MintVec2>` → `impl Into<Vector2>` across the 9 files.
3. `raylib/Cargo.toml`: drop hard `glam`; add optional `glam`/`mint`/`serde` features forwarding to `raylib-sys`.
4. Fix the model/skeletal-animation errors (+ RAII test), then font/draw/callback/databuf signature fixes — work the `ws1-breakage-baseline.md` list to zero.
5. Wrap (or defer-with-rationale) the +40 file / +30 text fns; build a parity checklist from `raylib.h` (the repo has `checklist.md` + `find_unimplemented.py` to help).
6. Cherry-pick WS3-tagged PRs with attribution (`inventory.md`): #272, #268, #266, #263, #259, #252, #250, #257, #277, #282, #275, #270, … — verify each still applies to 6.0 before taking.
7. Tier-1 tests for window-independent wrappers; push to fork; drive 3-OS CI green (safe crate now builds, so the matrix should build the whole workspace — consider extending `baseline.yml` to `cargo build`/`test` the workspace, not just `raylib-sys`).

## Open decisions — confirm with the user FIRST (use AskUserQuestion)

1. **`MintVec*` fate:** deprecate as `#[deprecated]` aliases for one release (courtesy), or remove outright (it's a 6.0 major bump, so removal is allowed)?
2. **New-fn coverage:** wrap **all** +40 file / +30 text fns in WS3, or wrap the high-value ones and defer the long tail (tracked) to keep WS3 bounded?
3. **`DrawModelPoints`/Ex:** remove from the safe API, or keep deprecated stubs?
4. **WS3 size:** likely worth splitting (e.g. WS3a math/type adoption + sweep; WS3b model/font/draw 6.0 fixes; WS3c new-fn wrappers + PR cherry-picks). Confirm the split.

## Working model (unchanged — see CLAUDE.md)

Branch `6.0-rc` in the fork (`fork` remote). **Push to the fork to run CI**; one merge to `raylib-rs/unstable` only at the very end (WS8). Workflow: confirm decisions → **writing-plans** (per the split) → **subagent-driven-development** (fresh subagent per task, spec + quality review, controller verifies, frequent commits with `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`). Cherry-picked PRs keep their author via `Co-authored-by`.
