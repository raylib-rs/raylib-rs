//! Raw FFI bindings for [raylib](https://www.raylib.com/).
//!
//! The crate is unconditionally `#![no_std]` — it depends only on `core`, so
//! it builds for embedded/no-std targets (linking raylib's C library for such
//! a target is the consumer's responsibility; see the `nobuild` and
//! `nobindgen` features). std consumers are unaffected. The `mint` adapter
//! feature is no-std-compatible; `glam` and `serde` currently require a
//! std-capable target.
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::approx_constant)]
// bindgen maps C `long double` to u128 on Linux (128-bit float ABI); the generated
// extern fns for these standard-math intrinsics are never called by raylib-rs, but
// the `improper_ctypes` lint fires on the `include!`-d bindings.rs. Suppress it
// crate-wide since we cannot annotate the generated file directly.
#![allow(improper_ctypes)]

#[cfg(not(feature = "nobindgen"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "nobindgen")]
include!(env!("RAYLIB_BINDGEN_LOCATION"));

#[cfg(target_os = "macos")]
pub const MAX_MATERIAL_MAPS: u32 = 12;

mod color;
#[cfg(feature = "glam")]
mod glam_conv;
mod math;
mod matrix_quat_math;
#[cfg(feature = "mint")]
mod mint_conv;
mod vector_math;
#[allow(unused_imports)]
pub use color::*;
#[allow(unused_imports)]
pub use math::*;
pub use matrix_quat_math::{matrix_decompose, quaternion_to_axis_angle};
pub use vector_math::vector3_ortho_normalize;

#[allow(clippy::derivable_impls)] // LOG_INFO != discriminant 0 (LOG_ALL); derived Default would return LOG_ALL
impl Default for TraceLogLevel {
    fn default() -> Self {
        TraceLogLevel::LOG_INFO
    }
}
