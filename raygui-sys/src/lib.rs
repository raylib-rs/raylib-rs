#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::approx_constant)]

use raylib_sys::*;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(target_os = "macos")]
pub const MAX_MATERIAL_MAPS: u32 = 12;
