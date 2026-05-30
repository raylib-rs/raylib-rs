/* raylib-rs
   ease.rs - Easings/interpolation helpers

Copyright (c) 2018-2019 Paul Clement (@deltaphc)

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/

//! Convenience re-export of the most commonly used raylib-rs items.
//!
//! A single `use raylib::prelude::*;` brings into scope:
//! - [`RaylibHandle`] and [`RaylibThread`] — the two handles returned by [`init`].
//! - [`RaylibBuilder`] — for configuring the window before creation via [`init`].
//! - [`RaylibDraw`] and all drawing extension traits.
//! - [`Color`] and `Rectangle` — fundamental geometry types.
//! - All raymath types (`Vector2`, `Vector3`, `Vector4`, `Matrix`, `Quaternion`) and their methods.
//! - Audio, texture, model, shader, text, and GUI types.
//! - `consts` re-exports (key codes, mouse buttons, etc.).
//!
//! # Example
//!
//! ```no_run
//! use raylib::prelude::*;
//!
//! let (mut rl, thread) = raylib::init()
//!     .size(640, 480)
//!     .title("My Game")
//!     .build();
//!
//! while !rl.window_should_close() {
//!     let mut d = rl.begin_drawing(&thread);
//!     d.clear_background(Color::RAYWHITE);
//! }
//! ```

pub use crate::callbacks::*;
pub use crate::consts::*;
pub use crate::core::audio::*;
pub use crate::core::automation::*;

#[cfg(not(feature = "nobuild"))]
pub use crate::core::camera::*;

pub use crate::core::collision::*;
pub use crate::core::color::*;
pub use crate::core::data::*;
pub use crate::core::databuf::*;
pub use crate::core::drawing::*;
pub use crate::core::file::*;
pub use crate::core::input::*;
pub use crate::core::logging::*;
pub use crate::core::math::*;
pub use crate::core::misc::*;
pub use crate::core::models::*;
pub use crate::core::pixel::*;
pub use crate::core::shaders::*;
pub use crate::core::text::*;
pub use crate::core::texture::*;
pub use crate::core::vr::*;
pub use crate::core::window::*;
pub use crate::core::*;

#[cfg(not(feature = "nobuild"))]
pub use crate::rgui::*;
pub use crate::rlgl::*;

pub use crate::*;
