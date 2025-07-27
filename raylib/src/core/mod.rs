//! Raylib core features and functionality

#[macro_use]
mod macros;

pub mod audio;
pub mod automation;
pub mod callbacks;
pub mod camera;
pub mod collision;
pub mod color {
    //! Color
    pub use crate::ffi::Color;
}
pub mod data;
pub mod databuf;
pub mod drawing;
pub mod error;
pub mod file;

pub mod input;
pub mod logging;
pub mod math;
pub mod misc;
pub mod models;
pub mod shaders;
pub mod text;
pub mod texture;
pub mod vr;
pub mod window;

use raylib_sys::TraceLogLevel;

use crate::ffi;
use std::ffi::CString;
use std::marker::PhantomData;

#[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
use crate::consts::ConfigFlags::*;

/// Construct a cstring as you would a Rust string
// shamelessly stolen from imgui
#[macro_export]
macro_rules! rstr {
    ($e:tt) => ({
        #[allow(unused_unsafe)]
        unsafe {
          std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($e, "\0").as_bytes())
        }
    });
    ($e:tt, $($arg:tt)*) => ({
        #[allow(unused_unsafe)]
        unsafe {
          std::ffi::CString::new(format!($e, $($arg)*)).unwrap()
        }
    })
}

/// This token is used to ensure certain functions are only running on the same
/// thread raylib was initialized from. This is useful for architectures like macos
/// where cocoa can only be called from one thread.
#[derive(Clone, Debug)]
pub struct RaylibThread(PhantomData<*const ()>);

/// The main interface into the Raylib API.
///
/// This is the way in which you will use the vast majority of Raylib's functionality. A `RaylibHandle` can be constructed using the [`init_window`] function or through a [`RaylibBuilder`] obtained with the [`init`] function.
///
/// [`init_window`]: fn.init_window.html
/// [`RaylibBuilder`]: struct.RaylibBuilder.html
/// [`init`]: fn.init.html
#[derive(Debug)]
pub struct RaylibHandle(()); // inner field is private, preventing manual construction

impl Drop for RaylibHandle {
    fn drop(&mut self) {
        if unsafe { ffi::IsWindowReady() } {
            unsafe {
                ffi::CloseWindow();
            }
            // NOTE(IOI_XD): If imgui is enabled, we don't call the destructor here because we're using a context that Rust expects to free, and the only other thing in that function is the free'ing of FontTexture...an action which causes a segfault.
            // It then gets successfully replaced if rlImGuiReloadFonts is called, so we'll take it.
        }
    }
}

/// A builder that allows more customization of the game window shown to the user before the `RaylibHandle` is created.
#[derive(Debug, Default)]
pub struct RaylibBuilder<'a> {
    flags: u32,
    log_level: TraceLogLevel,
    width: i32,
    height: i32,
    title: &'a str,
}

#[inline]
#[must_use]
/// Creates a `RaylibBuilder` for choosing window options before initialization.
pub fn init<'a>() -> RaylibBuilder<'a> {
    RaylibBuilder {
        width: 640,
        height: 480,
        title: "raylib-rs",
        ..Default::default()
    }
}

impl<'a> RaylibBuilder<'a> {
    /// Sets the window to be fullscreen.
    pub const fn fullscreen(&mut self) -> &mut Self {
        self.flags |= FLAG_FULLSCREEN_MODE as u32;
        self
    }

    /// Set the builder's log level.
    pub const fn log_level(&mut self, level: TraceLogLevel) -> &mut Self {
        self.log_level = level;
        self
    }
    /// Sets the window to be resizable.
    pub const fn resizable(&mut self) -> &mut Self {
        self.flags |= FLAG_WINDOW_RESIZABLE as u32;
        self
    }

    /// Sets the window to be undecorated (without a border).
    pub const fn undecorated(&mut self) -> &mut Self {
        self.flags |= FLAG_WINDOW_UNDECORATED as u32;
        self
    }

    /// Sets the window to be transparent.
    pub const fn transparent(&mut self) -> &mut Self {
        self.flags |= FLAG_WINDOW_TRANSPARENT as u32;
        self
    }

    /// Hints that 4x MSAA (anti-aliasing) should be enabled. The system's graphics drivers may override this setting.
    pub const fn msaa_4x(&mut self) -> &mut Self {
        self.flags |= FLAG_MSAA_4X_HINT as u32;
        self
    }

    /// Hints that vertical sync (Vsync) should be enabled. The system's graphics drivers may override this setting.
    pub const fn vsync(&mut self) -> &mut Self {
        self.flags |= FLAG_VSYNC_HINT as u32;
        self
    }

    /// Sets the window's width.
    pub const fn width(&mut self, w: i32) -> &mut Self {
        self.width = w;
        self
    }

    /// Sets the window's height.
    pub const fn height(&mut self, h: i32) -> &mut Self {
        self.height = h;
        self
    }

    /// Sets the window's width and height.
    pub const fn size(&mut self, w: i32, h: i32) -> &mut Self {
        self.width = w;
        self.height = h;
        self
    }

    /// Sets the window title.
    pub const fn title(&mut self, text: &'a str) -> &mut Self {
        self.title = text;
        self
    }

    /// Builds and initializes a Raylib window.
    ///
    /// # Panics
    ///
    /// Attempting to initialize Raylib more than once will result in a panic.
    #[must_use]
    pub fn build(&self) -> (RaylibHandle, RaylibThread) {
        unsafe {
            ffi::SetConfigFlags(self.flags);
        }

        unsafe {
            ffi::SetTraceLogLevel(self.log_level as i32);
        }

        let rl = init_window(self.width, self.height, self.title);

        (rl, RaylibThread(PhantomData))
    }
}

/// Initializes window and OpenGL context.
///
/// # Panics
///
/// Attempting to initialize Raylib more than once will result in a panic.
///
/// This function will panic if `title` contains an internal 0 byte.
fn init_window(width: i32, height: i32, title: &str) -> RaylibHandle {
    assert!(
        !unsafe { ffi::IsWindowReady() },
        "Attempted to initialize raylib-rs more than once!"
    );
    unsafe {
        let c_title = CString::new(title).expect("title should not contain an internal 0 byte");
        ffi::InitWindow(width, height, c_title.as_ptr());
    }
    assert!(
        unsafe { ffi::IsWindowReady() },
        "Attempting to create window failed!",
    );

    RaylibHandle(())
}
