/* raylib-rs
   lib.rs - Main library code (the safe layer)

Copyright (c) 2018-2024 raylib-rs team

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/

//! # raylib-rs
//!
//! `raylib` is a safe Rust binding to [Raylib](https://www.raylib.com/), a C library for enjoying games programming.
//!
//! To get started, take a look at the [`init_window`] function. This initializes Raylib and shows a window, and returns a [`RaylibHandle`]. This handle is very important, because it is the way in which one accesses the vast majority of Raylib's functionality. This means that it must not go out of scope until the game is ready to exit. You will also receive a !Send and !Sync [`RaylibThread`] required for thread local functions.
//!
//! For more control over the game window, the [`init`] function will return a [`RaylibBuilder`] which allows for tweaking various settings such as Vsync, anti-aliasing, fullscreen, and so on. Calling [`RaylibBuilder::build`] will then provide a [`RaylibHandle`].
//!
//! Some useful constants can be found in the [`consts`] module, which is also re-exported in the [`prelude`] module. In most cases you will probably want to `use raylib::prelude::*;` to make your experience more smooth.
//!
//! [`init_window`]: fn.init_window.html
//! [`init`]: fn.init.html
//! [`RaylibHandle`]: struct.RaylibHandle.html
//! [`RaylibThread`]: struct.RaylibThread.html
//! [`RaylibBuilder`]: struct.RaylibBuilder.html
//! [`RaylibBuilder::build`]: struct.RaylibBuilder.html#method.build
//! [`consts`]: consts/index.html
//! [`prelude`]: prelude/index.html
//!
//! # Examples
//!
//! The classic "Hello, world":
//!
//! ```ignore
//! use raylib::prelude::*;
//!
//! fn main() {
//!     let (mut rl, thread) = raylib::init()
//!         .size(640, 480)
//!         .title("Hello, World")
//!         .build();
//!
//!     while !rl.window_should_close() {
//!         let mut d = rl.begin_drawing(&thread);
//!
//!         d.clear_background(Color::WHITE);
//!         d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
//!     }
//! }
//! ```
//#![cfg_attr(feature = "nightly", feature(auto_traits))]

#![forbid(clippy::correctness, clippy::perf)]
#![warn(
    clippy::unwrap_used,
    reason = "debugging is easier when you know what the unwrap represents"
)]
#![warn(
    clippy::missing_safety_doc,
    reason = "nobody can use your unsafe code soundly if you don't explain the requirements for soundness"
)]
#![warn(
    clippy::undocumented_unsafe_blocks,
    reason = "it is extremely difficult to verify soundness if you don't explain your assumptions"
)]
#![warn(
    clippy::multiple_unsafe_ops_per_block,
    reason = "the safety requirements of two unsafe operations are rarely (though not never) covered by the same documentation"
)]
#![warn(
    missing_docs,
    reason = "users deserve to know what a public API does and how to use it"
)]
#![warn(
    clippy::allow_attributes_without_reason,
    reason = "making lints less restrictive should not be done without reason, and you will not always be there to explain it"
)]
#![warn(
    clippy::inline_always,
    reason = "sure, go ahead and tell the compiler you want stuff inlined. But don't go thinking inline is some magical
            performance fountain that will somehow always be faster than not inlining. You told the compiler to inline it,
            it decided it shouldn't. So you'd better have some DATA before you tell it to proceed *in spite of that*."
)]
#![warn(
    clippy::missing_const_for_fn,
    reason = "another function could be made const if this one was; or another function may *only* not be const because this one isn't"
)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::unnecessary_cast,
    reason = r#"if this is intentional, indicate it with a reason (`#[allow(clippy::..., reason = "...")]`)"#
)]
#![deny(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment,
    clippy::cast_abs_to_unsigned,
    clippy::cast_enum_constructor,
    clippy::cast_slice_from_raw_parts,
    clippy::fn_to_numeric_cast,
    clippy::fn_to_numeric_cast_any,
    clippy::fn_to_numeric_cast_with_truncation,
    clippy::cast_nan_to_int,
    clippy::ref_as_ptr,
    clippy::ptr_as_ptr,
    clippy::as_ptr_cast_mut,
    clippy::as_underscore,
    clippy::borrow_as_ptr,
    clippy::ptr_cast_constness,
    reason = r"do not mistake `as` for being lossless. it can fail, but it won't tell you, and you need to handle that"
)]
#![allow(dead_code, reason = "future use, and some features are incomplete")]
pub mod consts;
pub mod core;
pub mod ease;
pub mod prelude;
pub mod rgui;

/// The raw, unsafe FFI binding, in case you need that escape hatch or the safe layer doesn't provide something you need.
pub mod ffi {
    pub use raylib_sys::*;
}

pub use crate::core::collision::*;

pub use ffi::Matrix as MintMatrix;
pub use ffi::Quaternion as MintQuat;
pub use ffi::Vector2 as MintVec2;
pub use ffi::Vector3 as MintVec3;
pub use ffi::Vector4 as MintVec4;

pub use crate::core::logging::*;
pub use crate::core::misc::open_url;
pub use crate::core::*;

// Re-exports
#[cfg(feature = "serde")]
pub use serde;
