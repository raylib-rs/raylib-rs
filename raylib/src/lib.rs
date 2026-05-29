/* raylib-rs
   lib.rs - Main library code (the safe layer)

Copyright (c) 2018-2024 raylib-rs team

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/

//! Safe Rust bindings to raylib 6.0.
//!
//! This crate provides idiomatic Rust access to [raylib](https://www.raylib.com/)'s full feature
//! set: window management, 2D/3D drawing, input handling, raymath (vectors, matrices,
//! quaternions), audio playback, raygui immediate-mode UI, safe `rlgl` OpenGL abstractions, and a
//! headless software-renderer test harness — all without unsafe code in normal usage.
//!
//! ## Getting started
//!
//! Call [`init`] to obtain a [`RaylibBuilder`], configure it with chained methods, then call
//! [`RaylibBuilder::build`] to receive a `(RaylibHandle, RaylibThread)` pair. Everything in the
//! API hangs off [`RaylibHandle`]; keep it alive for the lifetime of your game loop. The
//! [`RaylibThread`] token is `!Send` and `!Sync` — it may only be used on the thread raylib was
//! initialised from.
//!
//! ## Feature gates
//!
//! - Math-crate integrations: `glam`, `mint`, `serde` (all opt-in, none default).
//! - `software_renderer` — wires the `rlsw` memory-platform backend for windowless/headless
//!   rendering; exposes the `test_harness` module for pixel-probe tests. Mutually exclusive with
//!   the `opengl_*` features.
//! - OpenGL back-end selection: `opengl_33` (default), `opengl_21`, `opengl_es_20`.
//! - Platform targets: `drm` (DRM/KMS tty, requires `opengl_es_20`), `wayland`.
//! - `full` — convenience alias enabling all optional API surface except back-end selectors.
//!
//! ## Examples
//!
//! The classic "Hello, world":
//!
//! ```no_run
//! use raylib::prelude::*;
//!
//! let (mut rl, thread) = raylib::init()
//!     .size(640, 480)
//!     .title("Hello, World")
//!     .build();
//!
//! while !rl.window_should_close() {
//!     let mut d = rl.begin_drawing(&thread);
//!
//!     d.clear_background(Color::WHITE);
//!     d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
//! }
//! ```
//#![cfg_attr(feature = "nightly", feature(auto_traits))]

#![allow(dead_code)]
#![deny(missing_docs)]
pub mod consts;
/// Core raylib functionality — window, drawing, input, audio, and other subsystems.
pub mod core;
pub mod ease;
pub mod prelude;
#[cfg(not(feature = "nobuild"))]
pub mod rgui;
pub mod rlgl;
/// Headless software-render test harness (enabled by the `software_renderer` feature).
#[cfg(feature = "software_renderer")]
pub mod test_harness;

/// The raw, unsafe FFI binding, in case you need that escape hatch or the safe layer doesn't provide something you need.
pub mod ffi {
    pub use raylib_sys::*;
}

pub use crate::core::collision::*;

/// Deprecated alias for [`ffi::Vector2`]; the public type is now the native `raylib-sys` type.
#[deprecated(
    since = "6.0.0",
    note = "use `Vector2` (the native type); `MintVec2` is now identical"
)]
pub type MintVec2 = ffi::Vector2;
/// Deprecated alias for [`ffi::Vector3`].
#[deprecated(since = "6.0.0", note = "use `Vector3`; `MintVec3` is now identical")]
pub type MintVec3 = ffi::Vector3;
/// Deprecated alias for [`ffi::Vector4`].
#[deprecated(since = "6.0.0", note = "use `Vector4`; `MintVec4` is now identical")]
pub type MintVec4 = ffi::Vector4;
/// Deprecated alias for [`ffi::Matrix`].
#[deprecated(since = "6.0.0", note = "use `Matrix`; `MintMatrix` is now identical")]
pub type MintMatrix = ffi::Matrix;
/// Deprecated alias for [`ffi::Quaternion`].
#[deprecated(
    since = "6.0.0",
    note = "use `Quaternion`; `MintQuat` is now identical"
)]
pub type MintQuat = ffi::Quaternion;

pub use crate::core::logging::*;
pub use crate::core::misc::open_url;
pub use crate::core::*;

// Re-exports
#[cfg(feature = "serde")]
pub use serde;
