# WS1 breakage baseline — `raylib` (safe) crate against raylib 6.0

Captured after bumping the submodule to the raylib 6.0 tag (WS0 Task 4).
`raylib-sys` builds clean; the safe crate does not yet. This is the worklist for WS1.

## Submodule
- raylib tag: 6.0 (commit dbc56a87da87d973a9c5baa4e7438a9d20121d28)

## build.rs / lib.rs changes made in WS0 (to keep raylib-sys building)
- none — raylib-sys compiled against 6.0 without any modifications to build.rs or lib.rs.
- Note: `MAX_MATERIAL_MAPS` override in `lib.rs:13` (macOS-only, value 12) remains correct; raylib 6.0 still defines `MAX_MATERIAL_MAPS = 12` in `rmodels.c`.

## Safe-crate compile errors (grouped, with counts)

Total distinct errors: 37

### Removed/renamed FFI symbols (E0425 — 5 errors)
Functions removed or renamed between 5.5 and 6.0:
- `DrawModelPoints` — removed (was for point-cloud drawing; no direct 6.0 replacement visible in bindings)
- `DrawModelPointsEx` — removed (extended variant of above)
- `UpdateModelAnimationBones` — removed; superseded by the redesigned `UpdateModelAnimation(model, anim, frame: f32)`
- `UnloadModelAnimation` (singular) — removed (x2 call sites); replaced by `UnloadModelAnimations` (plural, takes count)

### Struct field/layout changes (E0609 — 27 errors)

**`Model` struct — bones/skeleton lifted into new `ModelSkeleton` sub-struct:**
Old fields `bones: *mut BoneInfo`, `boneCount: i32`, `bindPose: *mut Transform` are gone from `Model`.
In 6.0 they live inside `Model::skeleton: ModelSkeleton` (which itself has `boneCount`, `bones`, `bindPose: ModelAnimPose`).
- `Model::bones` — removed (6 error sites; now `model.skeleton.bones`)
- `Model::boneCount` — removed (2 error sites; now `model.skeleton.boneCount`)
- `Model::bindPose` — removed (4 error sites; now `model.skeleton.bindPose`)

**`ModelAnimation` struct — frame data renamed and bones removed:**
Old fields `frameCount`, `framePoses: *mut *mut Transform`, `bones: *mut BoneInfo` are gone.
In 6.0: `keyframeCount: i32`, `keyframePoses: *mut ModelAnimPose`, and bones were lifted out entirely.
- `ModelAnimation::frameCount` — removed (6 error sites; now `anim.keyframeCount`)
- `ModelAnimation::framePoses` — removed (4 error sites; now `anim.keyframePoses`)
- `ModelAnimation::bones` — removed (2 error sites; no per-animation bones in 6.0)

**`FilePathList` struct — `capacity` field removed:**
Old `FilePathList { capacity, count, paths }`. In 6.0 it is `FilePathList { count, paths }`.
- `FilePathList::capacity` — removed (2 error sites)

### Signature changes (E0061, E0308 — 5 errors)

- **`DrawCircleGradient`** — signature changed from
  `(center_x: i32, center_y: i32, radius: f32, color1: Color, color2: Color)` (5 args) to
  `(center: Vector2, radius: f32, inner: Color, outer: Color)` (4 args).
  (1 error: "4 arguments but 5 supplied")

- **`LoadFontData`** — gained a new trailing out-parameter `glyphCount: *mut i32`.
  Now: `(fileData, dataSize, fontSize, codepoints, codepointCount, type_, glyphCount) -> *mut GlyphInfo` (7 args).
  (2 errors: "7 arguments but 6 supplied")

- **`UpdateModelAnimation`** — `frame` parameter changed from `i32` to `f32`.
  (1 error: mismatched types)

- **`SetSaveFileTextCallback`** / `SaveFileTextCallback` — the `text` argument mutability changed:
  old callback expected `*mut i8`, new expects `*const i8`.
  (1 error: mismatched types / mutability mismatch)

- **`DecodeDataBase64`** — caller passed `*const u8`; function now expects `*const i8`.
  (1 error: mismatched types, pointer element type)

## Notes for WS1

### 6.0 structural changes to address
1. **Skeletal animation redesign**: `Model` no longer stores `bones`/`bindPose` directly; they live in
   `Model::skeleton: ModelSkeleton`. `ModelAnimation` lost its own `bones` slice (shared via skeleton) and
   renamed `frameCount → keyframeCount`, `framePoses → keyframePoses: *mut ModelAnimPose`.
   New opaque `ModelAnimPose` type wraps pose data. All safe-wrapper code in `models.rs` accessing these
   fields must be rewritten.
2. **`UnloadModelAnimation` → `UnloadModelAnimations`**: The singular unload is gone. The `Drop` impl for
   `ModelAnimation` must call `UnloadModelAnimations` with count = 1 (or the API must change to batch).
3. **`DrawModelPoints`/`DrawModelPointsEx`**: No direct 6.0 equivalents visible; check 6.0 release notes
   for replacement or stub/remove these from the safe API.
4. **`DrawCircleGradient`**: Takes `Vector2` center instead of separate `i32` x/y. Update call site.
5. **`LoadFontData`**: New `glyphCount` out-parameter; threading it through the safe wrapper is mechanical.
6. **`FilePathList::capacity`**: Removed. Any code that relied on the capacity field must be updated.
7. **Callback mutability (`SaveFileTextCallback`)**: `text` is now `*const i8`; fix the safe wrapper's
   trampoline function signature.
8. **`DecodeDataBase64`**: Pointer cast `*const u8 → *const i8` needed (or change caller type).
9. **`UpdateModelAnimation` frame type**: Change `i32` → `f32` at the call site.

### New 6.0 surface to ADD (from raylib 6.0 release notes)
- ~40 filesystem functions (`LoadDirectoryFilesEx` enhancements, etc.)
- ~30 new text-processing functions
- Redesigned skeletal animation (`ModelSkeleton`, `ModelAnimPose`, new `UpdateModelAnimation` with `f32` frame)
- New structs: `ModelSkeleton`, `ModelAnimPose` (bindgen already generates these; safe wrappers needed)
- Memory/platform symbols and `rlsw` additions (review generated bindings for new `rl`-prefix fns)
