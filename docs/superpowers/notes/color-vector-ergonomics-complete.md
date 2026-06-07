# Color/Vector conversion ergonomics audit — done-note (queue item 14)

**Date:** 2026-06-07 · **Spec:** `docs/superpowers/specs/2026-06-07-vector-tuple-conversions-design.md`

## Audit conclusion

The named finding (PR review comment 5) — the redundant
`impl From<&Color> for Color` identity impl — was **already resolved** in
the 6.0 breaking changes (prior artifacts:
`docs/superpowers/specs/2026-06-03-color-ref-from-removal-design.md` + plan;
CHANGELOG 6.0.0 Breaking entry). Re-verified on `unstable`:

- Zero `From<&T> for T` identity impls remain on the math/color types.
- The surviving `From<&T>` impls in the safe crate (`&Camera2D`, `&Ray`,
  `&BoundingBox`, `&Transform`, `&NPatchInfo`, `&VrDeviceInfo`, `&Camera3D`)
  convert wrapper → *distinct* FFI type — genuine, kept.
- Draw API already takes `impl Into<ffi::Color>` / `impl Into<Vector2>`
  pervasively (~393 sites); values pass directly.

## What changed

The one real asymmetry: `Color` had `From<(u8,u8,u8,u8)>` but the vectors
had no tuple/array `From`. Added six impls in
`raylib-sys/src/vector_math.rs` — `From<(f32,…)>` and `From<[f32; N]>` for
Vector2/3/4. Purely additive (no blanket-`From` conflict; glam/mint are
distinct named types), `no_std`-clean. Tier-1 round-trip + `Into`-inference
tests in `raylib-sys/tests/conversions.rs` (unconditional module). Ships in
6.0.0.

## Not done (deliberately, YAGNI)

- No `Color` `From<[u8;4]>` / 3-tuple-alpha-255 forms — nobody asked, and the
  maintainer scoped item 14 to the vector gap. Easy additive follow-up if a
  user files it.
- No `From<&...>` forms on the vectors — that is the ceremony the
  From<&Color> removal established the project avoids.

## Verification

`cargo nextest run -p raylib-sys` (81 tests, +4 new), clippy
`-D warnings`, `cargo check -p raylib`, and the thumbv7em no-std proof leg
(the new impls compile cross-target) — all green locally. Queue item 14
closed.
