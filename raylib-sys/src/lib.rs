#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::approx_constant)]

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

impl Default for TraceLogLevel {
    fn default() -> Self {
        TraceLogLevel::LOG_INFO
    }
}
