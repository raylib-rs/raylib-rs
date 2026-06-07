# Wrapper-soundness refactor — design

**Date:** 2026-06-06
**Status:** approved (maintainer sign-off on the audit-first direction and this design, this session)
**Queue:** post-release flexible-queue item 17 (full PR #277 refactor, from WS6 tracked-deferred)
**Resolves:** issue #276 (`AsRef`/`AsMut` undermine safety); supersedes PR #277
(AmityWilder) — adopt-with-attribution.

## The unsoundness, precisely

`deref_impl_wrapper!` gives every thin wrapper `Deref`/`DerefMut`/`AsRef`/`AsMut`
to its raw FFI struct. The **mut half** is unsound exactly where the wrapper's
safe API trusts raw fields for memory safety:

- **(P1)** safe accessors build slices from count/dim/pointer fields
  (`mesh.vertexCount += 5; mesh.vertices()` → OOB in safe code), or
- **(P2)** `Drop` frees pointer fields (`model.meshes = p` → invalid free).

If safe code can put a value into a state where other safe code commits UB,
the API is unsound. The **read half** (`Deref`/`AsRef`) cannot violate these
invariants and stays.

This is deliberately **not** a blanket removal (what #277 did): types whose
FFI structs hold only inline data or GPU ids (corrupting an id confuses
raylib's C side but causes no Rust-side UB) keep full deref.

## Classification (2026-06-06 audit; evidence file:line in the audit notes)

| Verdict | Types |
|---|---|
| **readonly (P1+P2)** | `Image`, `Mesh`, `Model`, `Material`, `Font`, `Shader`, `FilePathList`, `DroppedFilePathList` |
| **readonly (P1; `no_drop`)** | `WeakMesh`, `WeakModel`, `WeakMaterial`, `WeakFont`, `WeakShader`, `ModelAnimation`, `WeakModelAnimation` |
| **readonly (P2 only)** | `Wave`, `Sound`, `Music`, `AudioStream` |
| **rslice mut-half removed (P2)** | `ImageColors`, `ImagePalette` (`AsMut<Box<[T]>>` allows `mem::take` → the taken Box frees via the global allocator while `Drop` still calls `UnloadImage*`) |
| **keep full deref (SOUND)** | `Texture2D`, `WeakTexture2D`, `RenderTexture2D`, `WeakRenderTexture2D`, `VrStereoConfig`, `BoneInfo`, `GlyphInfo`, `MaterialMap` |
| **already opted out** | `AutomationEventList`, `AutomationEvent` |

Audit surprises baked into this table:
- `Sound` still has the full deref family — the WS6 note claiming #277's
  Sound fix was folded is stale (likely lost in the WS3 audio redesign).
- The `Weak*` family is the *live* unsound surface: showcase writes go
  through `model.materials_mut()[0].as_mut().shader`.
- `VrStereoConfig` is SOUND despite the `Unload*` drop fn — all fields are
  inline `Matrix`/`f32`.

## Mechanism

1. **Third macro mode.** `make_thin_wrapper!` / `make_thin_wrapper_lifetime!`
   gain a `readonly` mode (alongside `true`/`false`): emits `Deref` + `AsRef`
   only, plus:

   ```rust
   /// # Safety
   /// Callers must uphold the wrapper's invariants: do not corrupt
   /// count/dimension/format fields that safe accessors trust (P1) and do
   /// not reassign pointer fields owned and freed by `Drop` (P2).
   pub unsafe fn as_raw_mut(&mut self) -> &mut ffi::T
   ```

2. **`make_rslice!`** drops the `Box`-exposing mut half (`AsMut<Box<[T]>>`
   and `DerefMut<Target = Box<[T]>>` — those allow `mem::take`, the actual
   P2). In their place: a safe `as_mut_slice(&mut self) -> &mut [T]`
   exposing the *elements* but not the `Box` itself — element mutation
   cannot corrupt the allocation. Reads keep `Deref`/`AsRef`. Zero current
   users of the removed surface (audit-verified), so no fallout.

3. **`// SOUNDNESS:` comments at every macro call site** stating the
   classification and the one-line P1/P2 evidence (maintainer requirement;
   makes the next audit diffable).

4. **Safe setters** for legitimate mutations on locked types:
   - `RaylibMaterial::set_shader(&Shader)` — same bitwise-copy semantics as
     today's manual `material.shader = *shader` writes (~35 showcase sites).
     Documented aliasing caveat: the `Shader` wrapper still owns `locs`; the
     material holds a copy of the inline struct, so the shader must outlive
     the material's use (pre-existing semantics, now documented).
   - `RaylibMaterial::set_map_color(index, Color)` and
     `set_map_value(index, f32)` — bounds-checked writes into the `maps`
     array (~17 sites currently doing raw `maps.offset(i)` writes in ad-hoc
     `unsafe` blocks become safe calls).
   - `Model::set_transform` stays (reimplemented via `.0`).
   - `Shader::locs_mut() -> &mut [i32]` stays — writing *into* the array is
     safe; only pointer reassignment was the hazard.

5. **Genuine raw cases** use `as_raw_mut()`: the `texcoords2` pointer
   install in `shaders_lightmap_rendering.rs:69` (already inside `unsafe`).

## Fallout (audit §2)

- ~35 showcase `*.shader = …` writes → `set_shader`.
- ~17 showcase map-color/value raw writes → `set_map_color`/`set_map_value`.
- 1 mesh pointer install → `as_raw_mut()`.
- `raylib/src` internal: `Model::set_transform` only (switch to `.0`);
  internal code generally uses `.0` already. Plan re-verifies with a build.
- `raylib/tests/`: zero write sites (audit-verified).
- Mesh-from-scratch examples build raw `ffi::Mesh` values before wrapping —
  unaffected.

## Testing

- **compile_fail doctests** on representative types (`Mesh`, `Sound`):
  `wrapper.as_mut()` / `*wrapper = …` no longer compiles.
- Tier-2 (software_renderer) test for `set_shader` + `set_map_color`/`value`
  round-trip via `LoadMaterialDefault`.
- Existing Tier-1/Tier-2 suites must stay green; full quality gates
  (fmt, clippy `-Dwarnings` all legs incl. showcase, doctests, MSRV 1.88).

## Documentation & release

- CHANGELOG **Breaking** entry (rides the 6.0.0 final — last call before
  publish): mut-half removed from the listed wrappers, new setters, new
  `unsafe as_raw_mut`, rslice mut removal.
- Book sweep for `as_mut()` patterns on locked types (plan task).
- Stale WS6 claim about Sound corrected in the done-note.
- PR resolves #276; credits @AmityWilder (#277) as the originating analysis.

## Done criteria

- All UNSOUND-classified wrappers no longer expose safe `&mut ffi::T`
  (compile_fail-proven); SOUND types unchanged.
- Every macro call site carries a `// SOUNDNESS:` comment.
- Showcase + book + tests green across all CI legs.
- Queue item 17 closed; #276 closed.
