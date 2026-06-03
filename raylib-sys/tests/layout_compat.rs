//! Static guarantee that hand-defined (bindgen-blocklisted) types keep the exact
//! C layout raylib 6.0 expects. A mismatch here is undefined behavior across FFI.
use raylib_sys::{Color, Matrix, Quaternion, Rectangle, Vector2, Vector3, Vector4};
use std::mem::{align_of, size_of};

const _: () = assert!(size_of::<Vector2>() == 8 && align_of::<Vector2>() == 4);
const _: () = assert!(size_of::<Vector3>() == 12 && align_of::<Vector3>() == 4);
const _: () = assert!(size_of::<Vector4>() == 16 && align_of::<Vector4>() == 4);
const _: () = assert!(size_of::<Quaternion>() == 16 && align_of::<Quaternion>() == 4);
const _: () = assert!(size_of::<Matrix>() == 64 && align_of::<Matrix>() == 4);
const _: () = assert!(size_of::<Rectangle>() == 16 && align_of::<Rectangle>() == 4);
const _: () = assert!(size_of::<Color>() == 4 && align_of::<Color>() == 1);

#[test]
fn layouts_match_raylib_6_0() {
    // The const _ assertions above evaluate at compile time; this gives a named target.
    // If raylib 6.0 ever changes a layout, the build fails on the failing assert.
}
