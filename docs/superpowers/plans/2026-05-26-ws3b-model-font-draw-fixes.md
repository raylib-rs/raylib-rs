# WS3b — The 37 FFI-Change Errors + Skeletal-Animation RAII Redesign

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Resolve the 37 captured compile errors so the safe `raylib` crate **builds clean and its tests pass** against raylib 6.0, with the redesigned skeletal-animation ownership correct (no use-after-free / invalid-free).

**Prerequisite:** WS3a complete — the safe crate's only remaining errors are the documented 37 FFI-change errors (math types adopted, MintVec swept, glam optional). Start from the `/tmp/ws3a-final.txt` fingerprint handed off by WS3a.

**Architecture:** raylib 6.0 lifted `Model.bones/boneCount/bindPose` into `Model.skeleton: ModelSkeleton`, renamed `ModelAnimation.frameCount→keyframeCount`/`framePoses→keyframePoses` (`ModelAnimPose = Transform*`), removed per-animation `bones`, made `UpdateModelAnimation`'s `frame` an `f32`, removed `UpdateModelAnimationBones`/`DrawModelPoints(Ex)`/the singular `UnloadModelAnimation`, dropped `FilePathList.capacity`, and tightened two callback/decoder pointer types to `*const`. **The sharpest change is animation ownership:** 6.0's only unload is `UnloadModelAnimations(animations, count)`, which frees each animation's `keyframePoses` **and then `RL_FREE(animations)` (the array pointer)**. So a per-animation owning wrapper is unsound; this plan introduces a `ModelAnimations` collection that owns the original heap array and frees it exactly once on `Drop`, demoting `ModelAnimation` to a non-owning view.

**Tech Stack:** Rust 1.85, `raylib-sys` 6.0 FFI. Branch `6.0-rc`; push to `fork` for CI.

**Reference:** `docs/superpowers/notes/ws1-breakage-baseline.md` (the 37, grouped); 6.0 structs from `raylib.h` (verified): `Model { transform, meshCount, materialCount, meshes, materials, meshMaterial, skeleton: ModelSkeleton, currentPose: ModelAnimPose, boneMatrices: *mut Matrix }`; `ModelSkeleton { boneCount: i32, bones: *mut BoneInfo, bindPose: ModelAnimPose }`; `ModelAnimation { name[32], boneCount: i32, keyframeCount: i32, keyframePoses: *mut ModelAnimPose }`; `type ModelAnimPose = *mut Transform`; `FilePathList { count, paths }`. 6.0 unload (verified in `rmodels.c`): `UnloadModelAnimations` loops `RL_FREE(keyframePoses[i])`, `RL_FREE(keyframePoses)`, then `RL_FREE(animations)`.

**Pre-flight:** `git rev-parse --abbrev-ref HEAD` → `6.0-rc`; `cargo build -p raylib 2>&1 | rg -c "^error"` equals the WS3a hand-off count.

**Scope boundary:** Edit only `raylib/src/core/{models,text,drawing,file,callbacks,data}.rs` and the parity checklist. No new 6.0 functions here (that's WS3c) beyond what's required to make the crate compile. Re-run `python find_unimplemented.py` at the end to refresh the checklist.

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `raylib/src/core/models.rs` | Skeleton accessor field paths; ModelAnimation 6.0 redesign + `ModelAnimations` RAII owner; remove DrawModelPoints/Ex, UpdateModelAnimationBones, singular unload; frame `f32` | 1,2,3 |
| `raylib/src/core/drawing.rs:677` | `DrawCircleGradient` → `Vector2` center | 4 |
| `raylib/src/core/text.rs:261,270` | `LoadFontData` trailing `glyphCount: *mut i32` | 5 |
| `raylib/src/core/file.rs` | Remove `FilePathList::capacity` (2 accessors + construction-site literals) | 6 |
| `raylib/src/core/callbacks.rs` | `SaveFileTextCallback` trampoline `text: *const i8` | 7 |
| `raylib/src/core/data.rs:105` | `DecodeDataBase64` pointer cast `*const i8` | 7 |
| `raylib-test/tests/...` (or `models.rs` `#[cfg(test)]`) | Focused RAII test for `ModelAnimations` Drop | 2 |

---

## Task 1: Fix `Model` skeleton accessor field paths

**Files:** Modify `raylib/src/core/models.rs` (lines ~104, ~257–301).

6.0 moved `bones`/`boneCount`/`bindPose` into `Model.skeleton`. These are read accessors + one null check — pure field-path changes.

- [ ] **Step 1: Fix the `load_model` null check (~line 104).** Before:
```rust
if m.meshes.is_null() && m.materials.is_null() && m.bones.is_null() && m.bindPose.is_null()
```
After:
```rust
if m.meshes.is_null() && m.materials.is_null() && m.skeleton.bones.is_null() && m.skeleton.bindPose.is_null()
```

- [ ] **Step 2: Fix the `bones()`/`bones_mut()` accessors on `RaylibModel` (~lines 257–283).** Replace the four field reads `self.as_ref().bones` → `self.as_ref().skeleton.bones`, `self.as_ref().boneCount` → `self.as_ref().skeleton.boneCount`, and the `_mut` equivalents (`self.as_mut().bones` → `self.as_mut().skeleton.bones`, `self.as_mut().boneCount` → `self.as_mut().skeleton.boneCount`). The null check at the top of each becomes `self.as_ref().skeleton.bones.is_null()`.

- [ ] **Step 3: Fix the `bind_pose()`/`bind_pose_mut()` accessors (~lines 287–301).** `self.as_ref().bindPose` → `self.as_ref().skeleton.bindPose`; `self.as_mut().bindPose` → `self.as_mut().skeleton.bindPose`; null checks likewise. (`skeleton.bindPose` is `ModelAnimPose` = `*mut Transform`, same pointer shape the existing `transmute` already assumes — the transmute is unchanged.)

- [ ] **Step 4: Verify these specific errors are gone.** Run `cargo build -p raylib 2>&1 | rg "bones|boneCount|bindPose"` → expect **no** errors referencing a missing `Model` field (other ModelAnimation errors remain for Task 2).

- [ ] **Step 5: `cargo fmt --all` and commit.**
```bash
git add raylib/src/core/models.rs
git commit -m "$(printf 'fix(ws3b): Model bones/bindPose now live in Model.skeleton (6.0)\n\nUpdate the load_model null check and the RaylibModel bones/bind_pose\naccessors to read through Model.skeleton (ModelSkeleton).\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: `ModelAnimation` 6.0 redesign + `ModelAnimations` RAII owner

**Files:** Modify `raylib/src/core/models.rs` (wrapper decl ~51–57; `load_model_animations` ~132–155; `update_model_animation` ~159–169; `update_model_animation_bones` ~171–183; `RaylibModelAnimation` ~858–940; `unload_model_animation` ~1008–1014). Add a RAII test.

This is the load-bearing RAII change. 6.0 has **no singular unload**; `UnloadModelAnimations(ptr, count)` frees each animation's `keyframePoses` **and `RL_FREE`s the array pointer**. Therefore the owner must be the whole array.

- [ ] **Step 1: Demote `ModelAnimation` to a non-owning wrapper (~lines 51–57).** Change its drop function from the (now-removed) `ffi::UnloadModelAnimation` to `no_drop`:
```rust
make_thin_wrapper!(
    /// A single model animation: a non-owning view into a `ModelAnimations` collection.
    ModelAnimation,
    ffi::ModelAnimation,
    no_drop
);
```
(`WeakModelAnimation` stays as-is — also `no_drop`.)

- [ ] **Step 2: Add the `ModelAnimations` RAII owner.** Add near the wrapper decls:
```rust
/// Owns the heap array returned by `LoadModelAnimations`. Frees it exactly once on
/// drop via `UnloadModelAnimations` (which frees each animation's keyframe poses AND
/// the array pointer). Individual animations are borrowed, non-owning `ModelAnimation`s.
#[derive(Debug)]
pub struct ModelAnimations {
    ptr: *mut ffi::ModelAnimation,
    count: usize,
}

impl ModelAnimations {
    /// Number of animations in the array.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize { self.count }
    /// Whether the array is empty.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool { self.count == 0 }
    /// The animations as a borrowed slice of non-owning views.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[ModelAnimation] {
        // SAFETY: ModelAnimation is #[repr(transparent)] over ffi::ModelAnimation and the
        // array of `count` elements at `ptr` is valid for the lifetime of `self`.
        unsafe { std::slice::from_raw_parts(self.ptr as *const ModelAnimation, self.count) }
    }
    /// The animations as a mutable borrowed slice of non-owning views.
    #[inline]
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [ModelAnimation] {
        // SAFETY: see as_slice; exclusive borrow guarantees no aliasing.
        unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut ModelAnimation, self.count) }
    }
}

impl std::ops::Deref for ModelAnimations {
    type Target = [ModelAnimation];
    #[inline]
    fn deref(&self) -> &Self::Target { self.as_slice() }
}
impl std::ops::DerefMut for ModelAnimations {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { self.as_mut_slice() }
}

impl Drop for ModelAnimations {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: `ptr`/`count` are exactly what LoadModelAnimations returned; 6.0's
            // UnloadModelAnimations frees each keyframePoses[i], keyframePoses, then the
            // array. Called exactly once (this owner is the sole holder of `ptr`).
            unsafe { ffi::UnloadModelAnimations(self.ptr, self.count as i32); }
        }
    }
}
```

- [ ] **Step 3: Rewrite `load_model_animations` to return the owner (~lines 132–155).** Replace the body so it keeps the raylib array instead of copying-then-`MemFree`:
```rust
pub fn load_model_animations(
    &mut self,
    _: &RaylibThread,
    filename: &str,
) -> Result<ModelAnimations, LoadModelAnimError> {
    let c_filename = CString::new(filename).unwrap();
    let mut m_size = 0;
    let m_ptr = unsafe { ffi::LoadModelAnimations(c_filename.as_ptr(), &mut m_size) };
    if m_ptr.is_null() || m_size <= 0 {
        return Err(LoadModelAnimError::NoAnimationsLoaded { path: filename.into() });
    }
    Ok(ModelAnimations { ptr: m_ptr, count: m_size as usize })
}
```
(The old code copied each value into a `Vec<ModelAnimation>` then `MemFree`'d only the array — leaving per-animation frees to a singular unload that no longer exists. The owner now holds the array and frees everything via `UnloadModelAnimations`.)

- [ ] **Step 4: `UpdateModelAnimation` frame → `f32` (~lines 159–169).** Change the `frame: i32` parameter to `frame: f32`:
```rust
pub fn update_model_animation(
    &mut self,
    _: &RaylibThread,
    mut model: impl AsMut<ffi::Model>,
    anim: impl AsRef<ffi::ModelAnimation>,
    frame: f32,
) {
    unsafe { ffi::UpdateModelAnimation(*model.as_mut(), *anim.as_ref(), frame); }
}
```

- [ ] **Step 5: Remove `update_model_animation_bones` (~lines 171–183).** `UpdateModelAnimationBones` was removed in 6.0 (superseded by the redesigned `UpdateModelAnimation`). Delete the whole method. (If a blended variant is wanted, `UpdateModelAnimationEx` is tracked for WS3c.)

- [ ] **Step 6: Remove per-animation `bones()`/`bones_mut()` from `RaylibModelAnimation` (~lines 859–881).** 6.0 `ModelAnimation` has no `bones` field (bones live in `Model.skeleton`). Delete both methods. Keep `boneCount` usage in the pose accessors (the field still exists).

- [ ] **Step 7: Update the frame-pose accessors to keyframe names (~lines 883–939).** In `frame_poses`, `frame_poses_iter`, `frame_poses_mut`, `frame_poses_iter_mut`: `anim.frameCount` → `anim.keyframeCount`, `anim.framePoses` → `anim.keyframePoses`. `anim.boneCount` is unchanged. `keyframePoses` has type `*mut ModelAnimPose` (= `*mut *mut Transform`), so the existing `as *const *const Transform` / `as *mut *mut Transform` casts and the `FramePoseIter::new(anim.keyframePoses, ...)` call remain valid (add `.cast()` only if the compiler flags the `*mut ModelAnimPose` vs `*mut *mut Transform` nominal difference).

- [ ] **Step 8: Remove the singular `unload_model_animation` (~lines 1008–1014).** Delete the `pub unsafe fn unload_model_animation(...)` method — it called the removed singular unload, and ownership/cleanup now lives in `ModelAnimations::drop`. (Manual unload of a leaked weak array is not part of the safe API; document in the commit.)

- [ ] **Step 9: Write the focused RAII test.** Add an integration test that loads real animations and drops the collection, proving the single-`UnloadModelAnimations` path runs without invalid-free. Put it where windowed/asset tests live (`raylib-test`), using an asset that ships with raylib's examples. Create `raylib-test/tests/model_animation_raii.rs`:
```rust
//! WS3b: ModelAnimations must free the raylib heap array exactly once on drop.
//! Run under the sanitizers workflow (WS6) to assert no invalid/double free.
use raylib::prelude::*;

#[test]
fn model_animations_load_and_drop() {
    let (mut rl, thread) = raylib::init().size(64, 64).title("anim-raii").build();
    // Asset shipped with raylib examples; adjust path if the test harness copies assets elsewhere.
    let path = "../raylib-sys/raylib/examples/models/resources/models/iqm/guy.iqm";
    if std::path::Path::new(path).exists() {
        let anims = rl.load_model_animations(&thread, path).expect("animations load");
        assert!(!anims.is_empty(), "expected >=1 animation");
        let first_bone_count = anims[0].keyframe_count_for_test();
        let _ = first_bone_count;
        drop(anims); // exercises UnloadModelAnimations exactly once
    } else {
        eprintln!("SKIP: animation asset not found at {path}");
    }
}
```
If `keyframe_count` isn't exposed, assert on `anims.len()` only. The key assertion is that `drop(anims)` does not crash/abort. (If `raylib-test` can't run in the controller's environment, mark this test `#[ignore]` with a comment that WS6 sanitizers run it; do NOT delete it.)

- [ ] **Step 10: Verify all ModelAnimation errors are gone.** Run `cargo build -p raylib 2>&1 | rg "frameCount|framePoses|UnloadModelAnimation|UpdateModelAnimationBones|ModelAnimation::bones"` → expect **none**.

- [ ] **Step 11: `cargo fmt --all` and commit.**
```bash
git add raylib/src/core/models.rs raylib-test/tests/model_animation_raii.rs
git commit -m "$(printf 'fix(ws3b)!: redesign ModelAnimation ownership for raylib 6.0\n\n6.0 removed the singular UnloadModelAnimation; UnloadModelAnimations frees\neach keyframePoses AND RL_FREEs the array pointer. Introduce a\nModelAnimations RAII owner that holds the LoadModelAnimations array and\nfrees it exactly once on drop; ModelAnimation becomes a non-owning view.\nRename frameCount/framePoses -> keyframeCount/keyframePoses, drop the\nper-animation bones accessors, make UpdateModelAnimation frame f32, and\nremove UpdateModelAnimationBones + the singular unload. Adds a focused\nRAII drop test.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 3: Remove `DrawModelPoints`/`DrawModelPointsEx`

**Files:** Modify `raylib/src/core/models.rs`.

Removed upstream in 6.0 (confirmed absent from `raylib.h`); per the locked decision, delete the wrappers (no FFI symbol exists; a stub could only no-op).

- [ ] **Step 1: Find and delete the wrappers.** Run `rg -n "draw_model_points|DrawModelPoints" raylib/src/core/models.rs`. Delete the `draw_model_points` and `draw_model_points_ex` safe methods entirely (including their doc comments).

- [ ] **Step 2: Verify.** Run `cargo build -p raylib 2>&1 | rg "DrawModelPoints"` → expect **none**.

- [ ] **Step 3: Commit.**
```bash
git add raylib/src/core/models.rs
git commit -m "$(printf 'fix(ws3b)!: remove DrawModelPoints/DrawModelPointsEx (removed in 6.0)\n\nNo FFI symbol exists in raylib 6.0; removed rather than stubbed.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 4: `DrawCircleGradient` → `Vector2` center

**Files:** Modify `raylib/src/core/drawing.rs` (~lines 677–688).

6.0 sig: `DrawCircleGradient(Vector2 center, float radius, Color inner, Color outer)`.

- [ ] **Step 1: Rewrite the wrapper.** Before (5 args, separate x/y):
```rust
fn draw_circle_gradient(
    &mut self,
    center_x: i32,
    center_y: i32,
    radius: f32,
    color1: impl Into<ffi::Color>,
    color2: impl Into<ffi::Color>,
) {
    unsafe {
        ffi::DrawCircleGradient(center_x, center_y, radius, color1.into(), color2.into());
    }
}
```
After (Vector2 center; keep `i32` x/y at the Rust boundary for source-compatibility and build the `Vector2`):
```rust
fn draw_circle_gradient(
    &mut self,
    center_x: i32,
    center_y: i32,
    radius: f32,
    inner: impl Into<ffi::Color>,
    outer: impl Into<ffi::Color>,
) {
    unsafe {
        ffi::DrawCircleGradient(
            Vector2 { x: center_x as f32, y: center_y as f32 },
            radius,
            inner.into(),
            outer.into(),
        );
    }
}
```
(Ensure `Vector2` is in scope in `drawing.rs`; it is via `crate::core::math::*`/`crate::ffi`. Keeping `center_x/center_y: i32` avoids rippling call sites while matching the 6.0 C signature internally.)

- [ ] **Step 2: Verify + commit.** Run `cargo build -p raylib 2>&1 | rg "DrawCircleGradient"` → expect **none**.
```bash
git add raylib/src/core/drawing.rs
git commit -m "$(printf 'fix(ws3b): DrawCircleGradient takes a Vector2 center (6.0)\n\nBuild the Vector2 from the i32 x/y at the boundary; rename color args\nto inner/outer to match the new signature.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 5: `LoadFontData` trailing `glyphCount` out-param

**Files:** Modify `raylib/src/core/text.rs` (~lines 257–285).

6.0 sig: `LoadFontData(fileData, dataSize, fontSize, codepoints, codepointCount, type, glyphCount: *mut i32) -> *mut GlyphInfo`.

- [ ] **Step 1: Thread the new out-param through both call branches.** Add `let mut glyph_count: i32 = 0;` at the top of the `unsafe` block, and pass `&mut glyph_count` as the trailing argument to both `ffi::LoadFontData(...)` calls (the `Some(c)` branch ~261 and the `None` branch ~270). Example for the `None` branch:
```rust
None => ffi::LoadFontData(
    data.as_ptr(),
    data.len() as i32,
    font_size,
    std::ptr::null_mut(),
    0,
    sdf,
    &mut glyph_count,
),
```
Apply the same trailing `&mut glyph_count` to the `Some(c)` branch.

- [ ] **Step 2: Verify + commit.** Run `cargo build -p raylib 2>&1 | rg "LoadFontData"` → expect **none**. (Note in the commit: the wrapper still returns only the first glyph — the pre-existing single-glyph behavior and the array it leaves unfreed are tracked in the parity checklist as a WS3c follow-up; this task is the signature fix only.)
```bash
git add raylib/src/core/text.rs
git commit -m "$(printf 'fix(ws3b): LoadFontData gained a trailing glyphCount out-param (6.0)\n\nThread &mut glyph_count through both call branches. Single-glyph return\nbehavior unchanged; full-array handling tracked for WS3c.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 6: Remove `FilePathList::capacity`

**Files:** Modify `raylib/src/core/file.rs`.

6.0 `FilePathList` is `{ count, paths }` — no `capacity`. (`AutomationEventList.capacity` still exists — do NOT touch `automation.rs`.)

- [ ] **Step 1: Delete the two `capacity()` accessors.** Remove the `pub const fn capacity(&self) -> u32 { self.0.capacity }` method from both `impl FilePathList` (~135–137) and `impl DroppedFilePathList` (~159–161), including their doc comments.

- [ ] **Step 2: Remove the `capacity` field from every `ffi::FilePathList { .. }` literal.** Run `rg -n "capacity" raylib/src/core/file.rs` → expect hits at the construction sites (~277, 290, 304, 338, 382) and the two doctests (~29, 51). In each `ffi::FilePathList { capacity: <expr>, count: .., paths: .. }`, delete the `capacity: <expr>,` line. Fix the two doc-comment examples likewise so doctests compile.

- [ ] **Step 3: Verify.** Run `cargo build -p raylib 2>&1 | rg "capacity"` → expect **none** from `file.rs`. Run `rg -n "capacity" raylib/src/core/file.rs` → expect **zero** matches.

- [ ] **Step 4: Commit.**
```bash
git add raylib/src/core/file.rs
git commit -m "$(printf 'fix(ws3b)!: FilePathList lost its capacity field in 6.0\n\nRemove the capacity() accessors and drop capacity from all FilePathList\nstruct literals (incl. doctests). AutomationEventList.capacity is\nunaffected.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 7: Pointer-mutability fixes (`SaveFileTextCallback`, `DecodeDataBase64`)

**Files:** Modify `raylib/src/core/callbacks.rs` and `raylib/src/core/data.rs:105`.

- [ ] **Step 1: Fix the save-file-text trampoline (`callbacks.rs`).** Find the trampoline that matches `SaveFileTextCallback`: run `rg -n "custom_save_file_text_callback" raylib/src/core/callbacks.rs`. In its `extern "C" fn` definition, change the `text` parameter from `*mut ::std::os::raw::c_char` (`*mut i8`) to `*const ::std::os::raw::c_char` (`*const i8`) to match the 6.0 `SaveFileTextCallback` typedef. Update any `as *mut` cast inside to `as *const` and adjust the `CStr::from_ptr`/read accordingly (reading is fine from `*const`).

- [ ] **Step 2: Fix the base64 decode cast (`data.rs:105`).** Before:
```rust
unsafe { ffi::DecodeDataBase64(c_str.as_ptr() as *const u8, output_size.as_mut_ptr()) };
```
After (6.0 expects `*const i8`):
```rust
unsafe { ffi::DecodeDataBase64(c_str.as_ptr() as *const ::std::os::raw::c_char, output_size.as_mut_ptr()) };
```

- [ ] **Step 3: Verify + commit.** Run `cargo build -p raylib 2>&1 | rg "SaveFileTextCallback|DecodeDataBase64"` → expect **none**.
```bash
git add raylib/src/core/callbacks.rs raylib/src/core/data.rs
git commit -m "$(printf 'fix(ws3b): callback/decoder pointer types now *const (6.0)\n\nSaveFileTextCallback text arg and DecodeDataBase64 input are *const i8\nin 6.0. Fix the trampoline signature and the call-site cast.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 8: Green build + tests + refreshed checklist (the WS3b milestone)

**Files:** None new; regenerate the parity checklist.

- [ ] **Step 1: Full clean build.** Run `cargo build -p raylib` → **success, zero errors**. Then `cargo build -p raylib --features glam,mint,serde` → success. Then `cargo build -p raylib --no-default-features` → success (or document the expected minimal-feature gaps).

- [ ] **Step 2: Tests + doctests.** Run from inside `raylib/`: `cargo test` and `cargo test --doc` → all pass. (The `model_animation_raii` test lives in `raylib-test`; run `cd raylib-test && cargo +nightly test model_animations_load_and_drop` if the environment supports it, else confirm it is `#[ignore]`-documented for WS6.)

- [ ] **Step 3: Regenerate the parity checklist.** Run `python find_unimplemented.py`. Confirm `DrawModelPoints`/`DrawModelPointsEx`/`UpdateModelAnimationBones`/`UnloadModelAnimation` are no longer listed as TODO (removed-upstream), and `UnloadModelAnimations`/`UpdateModelAnimation` show implemented.

- [ ] **Step 4: Push to fork and confirm 3-OS CI.** `git push fork 6.0-rc`; watch the `baseline.yml` run. **Done-gate for WS3b:** the safe crate builds clean and tests pass locally and the existing CI legs stay green. (Extending CI to build/test the whole workspace is WS3c Step.)

- [ ] **Step 5: Commit the refreshed checklist.**
```bash
git add docs/superpowers/parity-checklist.md
git commit -m "$(printf 'chore(ws3b): refresh parity checklist after 6.0 error fixes\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## WS3b done criteria

- `cargo build -p raylib` is **green** (default + `glam,mint,serde`).
- `cargo test` + `cargo test --doc` pass from `raylib/`.
- Skeletal-animation ownership is 6.0-correct: `ModelAnimations` owns the array and frees it once via `UnloadModelAnimations`; `ModelAnimation` is non-owning; a focused RAII drop test exists.
- `DrawModelPoints(Ex)`, `UpdateModelAnimationBones`, singular `UnloadModelAnimation`, `FilePathList::capacity` removed; `DrawCircleGradient`/`LoadFontData`/callback/`DecodeDataBase64` signatures fixed.
- Parity checklist refreshed; the 37 baseline errors are at zero.
