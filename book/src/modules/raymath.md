# raymath

raylib-rs 6.0 ships native `#[repr(C)]` math types directly from `raylib-sys`:
[`Vector2`], [`Vector3`], [`Vector4`], [`Matrix`], and [`Quaternion`].
Their arithmetic comes from raylib's own raymath via a C shim
(`binding/raymath_shim.c`, `RAYMATH_IMPLEMENTATION`), wrapped as Rust
methods and operators.  No third-party math crate is needed by default —
`glam`, `mint`, and `serde` are **opt-in** Cargo features.

You'll land here when you need vector math, matrix transforms, or quaternion
rotation without opening a window.

## API surface

- [`Vector2`](https://docs.rs/raylib/latest/raylib/math/struct.Vector2.html) — 2D
  vector with `.dot`, `.cross`, `.normalize`, `.length`, `.length_sqr`,
  `.rotate`, and operator overloads.
- [`Vector3`](https://docs.rs/raylib/latest/raylib/math/struct.Vector3.html) — 3D
  vector with `.dot`, `.cross`, `.normalize`, `.length`, `.rotate_by_quaternion`,
  `.rotate_by_axis_angle`.
- [`Vector4`](https://docs.rs/raylib/latest/raylib/math/struct.Vector4.html) — 4D
  vector; also the FFI representation of a quaternion on the C side.
- [`Matrix`](https://docs.rs/raylib/latest/raylib/math/struct.Matrix.html) —
  row-major 4×4 matrix (row-major in memory; see Gotchas); constructors
  `Matrix::identity`, `Matrix::translate`, `Matrix::rotate`,
  `Matrix::rotate_x/y/z`, `Matrix::rotate_xyz/zyx`.
- [`Quaternion`](https://docs.rs/raylib/latest/raylib/ffi/struct.Quaternion.html) —
  distinct `#[repr(C)]` struct (C aliases it to `Vector4`). Constructors:
  `Quaternion::identity`, `Quaternion::from_axis_angle`, `Quaternion::normalize`,
  `Quaternion::length`.
- [`Ray`](https://docs.rs/raylib/latest/raylib/math/struct.Ray.html) /
  [`BoundingBox`](https://docs.rs/raylib/latest/raylib/math/struct.BoundingBox.html) —
  used by the 3D collision and raycasting helpers.
- [`lerp`](https://docs.rs/raylib/latest/raylib/math/fn.lerp.html) — `f32`
  linear interpolation convenience function.
- [`rquat`](https://docs.rs/raylib/latest/raylib/math/fn.rquat.html) — shorthand
  constructor for `Quaternion`.

## Example

The math types are window-independent — this doctest runs with no GPU context.

```rust
# extern crate raylib;
use raylib::math::{Vector3, Matrix};

// Vector operations
let a = Vector3::new(1.0, 0.0, 0.0);
let b = Vector3::new(0.0, 1.0, 0.0);

let d = a.dot(b);
assert_eq!(d, 0.0, "orthogonal vectors have zero dot product");

let c = a.cross(b);
assert!((c.z - 1.0).abs() < 1e-6, "cross product of x and y should be z");

let len = a.normalize().length();
assert!((len - 1.0).abs() < 1e-6, "normalized vector has unit length");

// Matrix identity
let m = Matrix::identity();
// raylib's Matrix is row-major in memory; identity diagonal is m0/m5/m10/m15.
assert_eq!(m.m0, 1.0);
assert_eq!(m.m5, 1.0);
assert_eq!(m.m10, 1.0);
assert_eq!(m.m15, 1.0);
assert_eq!(m.m1, 0.0);

// Translation matrix
let t = Matrix::translate(3.0, 0.0, 0.0);
// m12 is the x-translation entry in raylib's row-major memory layout.
assert_eq!(t.m12, 3.0);
```

## Gotchas

- **`Quaternion` is distinct from `Vector4`.**
  In C, `Quaternion` is a typedef for `Vector4`.  raylib-rs uses a distinct
  `#[repr(C)]` struct with the same layout for type safety — the compiler will not
  let you pass a `Vector4` where a `Quaternion` is expected.  Use
  `Quaternion::from(v4)` / `Vector4::from(q)` for explicit zero-cost conversion.
- **`MintVec*` types are deprecated.**
  The `MintVec2`/`MintVec3`/`MintVec4`/`MintMatrix`/`MintQuat` type aliases from
  5.x are still present but carry `#[deprecated(since = "6.0.0")]`.  Replace them
  with the native `Vector2`/`Vector3`/`Vector4`/`Matrix`/`Quaternion` types.  See
  [`ecosystem/glam-mint-serde.md`](../ecosystem/glam-mint-serde.md) for the
  opt-in integration story.
- **Zero math-crate deps by default.**
  The default build depends only on raylib's own raymath C implementation.
  Enable `--features glam` / `--features mint` / `--features serde` for
  conversions/trait impls.  Never assume those features are present in library
  code.
- **Row-major memory layout, semantic column-major ops.**
  raylib's `Matrix` is **row-major in memory** — the C source (`raymath.h`)
  writes translation into `m12/m13/m14`.  The math operations and parameter
  naming, however, treat the matrix as column-major; the two conventions describe
  the same data via different access patterns.  `glam::Mat4` is column-major in
  memory, so when comparing byte layouts side-by-side they will differ — but the
  `--features glam` adapter handles the mapping so values round-trip correctly.

## See also

- [3D models](./3d-models.md) — `Matrix` transforms used in model rendering.
- [Collision](./collision.md) — uses `Vector2`/`Vector3`/`BoundingBox`.
- [glam, mint, serde](../ecosystem/glam-mint-serde.md) — opt-in conversions.
- [`Vector3` docs.rs](https://docs.rs/raylib/latest/raylib/math/struct.Vector3.html) /
  [`Matrix` docs.rs](https://docs.rs/raylib/latest/raylib/math/struct.Matrix.html)

### Showcase examples

Showcase examples that exercise this module:

- [shapes_math_angle_rotation](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_math_angle_rotation.html)
- [shapes_math_sine_cosine](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_math_sine_cosine.html)
- [shapes_vector_angle](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_vector_angle.html)
- [shapes_easings_testbed](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_easings_testbed.html)
- [models_orthographic_projection](https://dacode45.github.io/raylib-rs/examples/models/models_orthographic_projection.html)
- [models_yaw_pitch_roll](https://dacode45.github.io/raylib-rs/examples/models/models_yaw_pitch_roll.html)
- [models_tesseract_view](https://dacode45.github.io/raylib-rs/examples/models/models_tesseract_view.html)
