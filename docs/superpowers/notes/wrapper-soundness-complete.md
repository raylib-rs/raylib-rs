# Wrapper-soundness refactor — done-note (queue item 17)

**Date:** 2026-06-07 · **Spec:** `docs/superpowers/specs/2026-06-06-wrapper-soundness-design.md`
**Resolves:** #276 · **Supersedes:** PR #277 (analysis by @AmityWilder, credited)

## What landed

- `AsRawMut<T>` trait (`unsafe fn as_raw_mut`) + `readonly` mode on
  `make_thin_wrapper!`/`make_thin_wrapper_lifetime!` (Deref/AsRef kept, mut
  half removed); `make_rslice!` no longer exposes the `Box` mutably
  (`as_mut_slice` exposes elements only).
- 19 wrappers reclassified `readonly` (P1: slice accessors trust counts;
  P2: Drop frees pointers): Model/Mesh/Material/ModelAnimation + Weaks,
  Image, Font/WeakFont, Shader/WeakShader, FilePathList ×2,
  Wave/Sound/Music/AudioStream. 8 kept full deref (inline-data/GPU-id
  only). Every macro call site carries a `// SOUNDNESS:` comment — the
  diffable audit artifact.
- Six accessor traits rebound `AsMut<ffi::T>` → `AsRawMut<ffi::T>`;
  ~32 internal sites converted to documented `unsafe` blocks.
- New safe setters: `RaylibMaterial::{set_shader, clear_shader,
  set_map_color, set_map_value}`, `Music::set_looping`; showcase also
  adopted the pre-existing `set_material_texture` for whole-texture
  installs. Showcase `as_raw_mut` census: 61 → 34 lines; the remainder
  are genuine raw-pointer work (lightmap texcoords2 install, custom CPU
  bone-blend, texture-id zeroing to defeat double-free).
- compile_fail doctests prove `mesh.vertexCount += 5` and
  `sound.stream.buffer = null` no longer compile.

## Found-and-fixed along the way

- `update_model_animation{,_ex}` took `impl AsMut<ffi::Model>` — became
  uncallable after the readonly switch (caught by its own doctest in
  review); both only pass the model BY VALUE to C → bounds relaxed to
  `AsRef`.
- **`RSliceGlyphInfo`** was a third, hand-written rslice with the same
  `mem::take` double-free the generated rslices lost — missed by the
  audit (it only enumerated `make_rslice!` outputs). Same fix applied.
- **Stale WS6 claim corrected:** `ws6b-complete.md` says #277's unsound
  `Sound` impls were folded in WS6 — they were NOT present-tense true on
  `unstable` (likely lost in the WS3 audio-lifetime redesign). Sound is
  now readonly like the rest of the audio family.

## Future work seeded

- `impl_rslice!`'s `as_mut_slice` hardcodes `&mut [ffi::Color]` — correct
  for both current users; any future non-Color rslice hits an immediate
  compile error. Generalize by emitting the method from `make_rslice!`
  (where the element type is in scope) if a third generated rslice appears.
- The `Weak*` types still implement the full accessor traits; a future
  ergonomics pass could consider whether `WeakShader`'s mutating uniform
  uploads should require the owning `Shader`.

## Verification

5 clippy `-D warnings` legs, 193 nextest tests, 149 doctests
(incl. 5 compile_fail), fmt, Tier-2 `material_setters_roundtrip` under
`software_renderer` — all green locally; CI 28/28 expected on the PR.
