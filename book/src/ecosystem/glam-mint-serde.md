# glam, mint, serde

raylib-rs 6.0 makes math-ecosystem integration **strictly opt-in**.  The
default build ships zero extra math-crate dependencies — native `#[repr(C)]`
types (`Vector2`, `Vector3`, `Vector4`, `Matrix`, `Quaternion`) cover every
API surface with no third-party crate required.

Enable `glam`, `mint`, or `serde` as Cargo features to unlock `From`/`Into`
conversions and serialization trait derives on those native types.

## API surface

- **`--features glam`** — adds [`From`/`Into`][glam_conv] between:
  - `Vector2` ↔ `glam::Vec2`
  - `Vector3` ↔ `glam::Vec3`
  - `Vector4` ↔ `glam::Vec4`
  - `Quaternion` ↔ `glam::Quat`
  - `Matrix` ↔ `glam::Mat4` (column-major round-trip; see Gotchas)
- **`--features mint`** — adds [`From`/`Into`][mint_conv] between:
  - `Vector2` ↔ `mint::Vector2<f32>`
  - `Vector3` ↔ `mint::Vector3<f32>`
  - `Vector4` ↔ `mint::Vector4<f32>`
  - `Quaternion` ↔ `mint::Quaternion<f32>`
  - `Matrix` ↔ `mint::ColumnMatrix4<f32>`
- **`--features serde`** — adds `Serialize`/`Deserialize` derives on the math
  types (`Vector2`/`Vector3`/`Vector4`/`Matrix`/`Quaternion`) and on
  [`Color`](https://docs.rs/raylib/latest/raylib/color/struct.Color.html).

[glam_conv]: https://docs.rs/raylib-sys/latest/raylib_sys/index.html
[mint_conv]: https://docs.rs/raylib-sys/latest/raylib_sys/index.html

## Example

`full` includes `glam`, so this doctest runs under `cargo test --features full`
and under `mdbook test -L target/debug/deps`.

```rust
# extern crate raylib;
# extern crate glam;
use raylib::math::Vector3;

// Convert to glam for library interop, then convert back.
let rl_v = Vector3::new(1.0, 2.0, 3.0);
let gv: glam::Vec3 = rl_v.into();
assert_eq!(gv, glam::Vec3::new(1.0, 2.0, 3.0));

let round_trip: Vector3 = gv.into();
assert_eq!(round_trip.x, rl_v.x);
assert_eq!(round_trip.y, rl_v.y);
assert_eq!(round_trip.z, rl_v.z);
```

## Gotchas

- **`MintVec*` / `MintMatrix` / `MintQuat` are deprecated.**
  The `MintVec2`, `MintVec3`, `MintVec4`, `MintMatrix`, and `MintQuat` type
  aliases from the 5.x era are still present but carry
  `#[deprecated(since = "6.0.0")]`.  Replace them with the native types
  (`Vector2`, `Vector3`, `Vector4`, `Matrix`, `Quaternion`) and, if you need
  mint interop, enable `--features mint`.
- **`glam::Mat4` column-major vs. raylib `Matrix` row-major.**
  raylib's `Matrix` is row-major in memory (translation lives in `m12/m13/m14`;
  see [raymath chapter](../modules/raymath.md)).  `glam::Mat4` is column-major
  in memory.  The `--features glam` adapter handles the field permutation so
  values round-trip correctly — but if you inspect raw byte layouts
  side-by-side, `m0..m15` will differ from `glam`'s internal storage order.
- **Zero math-crate deps by default.**
  Library crates built on raylib-rs should **not** enable `glam`/`mint`/`serde`
  in their own `[features]` defaults unless they genuinely need them — doing so
  forces the dependency onto every downstream consumer.

## See also

- [raymath](../modules/raymath.md) — the native math types and their methods.
- [Features and platforms](../core-concepts/features.md) — the full feature flag
  reference.
- [docs.rs feature-gated items](https://docs.rs/raylib/latest/raylib/) — use the
  **Feature flags** section on docs.rs to browse the glam/mint/serde items.
