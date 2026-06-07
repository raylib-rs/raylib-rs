# Vector tuple/array `From` conversions — design

**Date:** 2026-06-07
**Status:** approved (maintainer sign-off this session)
**Queue:** post-release flexible-queue item 14 ("Color/Vector conversion ergonomics audit", PR review comment 5)

## Audit conclusion (the named finding is already resolved)

Item 14's specific concern — the redundant `impl From<&Color> for Color`
identity impl — was **already removed** in the 6.0 breaking changes
(prior artifacts: `docs/superpowers/specs/2026-06-03-color-ref-from-removal-design.md`
+ plan). Verified on current `unstable`:

- Zero `From<&T> for T` identity impls remain on the math/color types.
- The `From<&T>` impls left in the safe crate (`&Camera2D → ffi::Camera2D`,
  `&Ray → ffi::Ray`, `&BoundingBox`, `&Transform`, `&NPatchInfo`,
  `&VrDeviceInfo`, `&Camera3D`) are genuine wrapper→FFI conversions between
  *distinct* types — correct, not ceremony.
- The draw API already takes `impl Into<ffi::Color>` / `impl Into<Vector2>`
  pervasively (~393 sites), so `Color`/`Vector2` values pass directly.

## The one real gap

`Color` has `impl From<(u8, u8, u8, u8)>` (raylib-sys/src/color.rs:56), but
`Vector2/3/4` have **no** tuple or array `From` impls. The WS2b plan
explicitly deferred `From<(f32,f32)> for Vector2` ("only if cheap"). So
`draw_pixel_v((10.0, 20.0), c)` fails despite the param being
`impl Into<Vector2>` — callers must write `Vector2::new(10.0, 20.0)`.

## Change (purely additive, non-breaking)

Add six impls in `raylib-sys/src/vector_math.rs` (same crate/module as the
existing operator impls and the `From<Color> for Vector4` precedent):

```rust
impl From<(f32, f32)> for Vector2        // { x, y }
impl From<[f32; 2]> for Vector2
impl From<(f32, f32, f32)> for Vector3   // { x, y, z }
impl From<[f32; 3]> for Vector3
impl From<(f32, f32, f32, f32)> for Vector4  // { x, y, z, w }
impl From<[f32; 4]> for Vector4
```

Each `#[inline]` with a one-line doc comment (`deny(missing_docs)` is
crate-wide). Pure struct construction → `no_std`-clean. **No `From<&...>`
forms** — that is exactly the ceremony the From<&Color> removal established
we avoid; pass tuples/arrays by value.

**Conflict check:** no blanket `From<T>` on the vectors exists; `glam`/`mint`
conversions are over distinct named types — zero overlap. Additive, so no
caller breaks.

## Testing

Tier-1 round-trip unit tests in `raylib-sys/tests/conversions.rs`, in a new
**unconditional** top-level module (the existing modules are mint/glam/serde
feature-gated): `Vector2::from((1.0, 2.0))` + `Vector2::from([1.0, 2.0])`
field-equality, ×3 types. No FFI/window — runs in the default
`cargo nextest run -p raylib-sys` leg.

## Docs / release

- CHANGELOG **Added** entry (additive; rides 6.0.0 final).
- Done-note recording the audit conclusion (named finding already resolved,
  prior-spec pointer) + this tuple-asymmetry closure. No book change.

## Done criteria

- Six `From` impls land; tuple/array round-trip tests green.
- `cargo nextest run -p raylib-sys` + clippy `-Dwarnings` + the no-std proof
  leg (these impls compile for thumb) green.
- CHANGELOG entry; done-note; queue item 14 closed.
