#[macro_use]
mod macros;

pub mod audio;
pub mod automation;
pub mod callbacks;
#[cfg(not(feature = "nobuild"))]
pub mod camera;

pub mod collision;
pub mod color {
    #[allow(unused_imports)]
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
        unsafe {
            if ffi::IsWindowReady() {
                ffi::CloseWindow();
                // NOTE(IOI_XD): If imgui is enabled, we don't call the destructor here because we're using a context that Rust expects to free, and the only other thing in that function is the free'ing of FontTexture...an action which causes a segfault.
                // It then gets successfully replaced if rlImGuiReloadFonts is called, so we'll take it.
            }
        }
    }
}

/// A builder that allows more customization of the game window shown to the user before the `RaylibHandle` is created.
#[derive(Debug, Default)]
pub struct RaylibBuilder<'a> {
    fullscreen_mode: bool,
    window_resizable: bool,
    window_undecorated: bool,
    window_transparent: bool,
    msaa_4x_hint: bool,
    vsync_hint: bool,
    window_hidden: bool,
    window_always_run: bool,
    window_minimized: bool,
    window_maximized: bool,
    window_unfocused: bool,
    window_topmost: bool,
    window_highdpi: bool,
    window_mouse_passthrough: bool,
    borderless_windowed_mode: bool,
    interlaced_hint: bool,
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
        self.fullscreen_mode = true;
        self
    }

    /// Set the builder's log level.
    pub const fn log_level(&mut self, level: TraceLogLevel) -> &mut Self {
        self.log_level = level;
        self
    }
    /// Sets the window to be resizable.
    pub const fn resizable(&mut self) -> &mut Self {
        self.window_resizable = true;
        self
    }

    /// Sets the window to be undecorated (without a border).
    pub const fn undecorated(&mut self) -> &mut Self {
        self.window_undecorated = true;
        self
    }

    /// Sets the window to be transparent.
    pub const fn transparent(&mut self) -> &mut Self {
        self.window_transparent = true;
        self
    }

    /// Hints that 4x MSAA (anti-aliasing) should be enabled. The system's graphics drivers may override this setting.
    pub const fn msaa_4x(&mut self) -> &mut Self {
        self.msaa_4x_hint = true;
        self
    }

    /// Hints that vertical sync (VSync) should be enabled. The system's graphics drivers may override this setting.
    pub const fn vsync(&mut self) -> &mut Self {
        self.vsync_hint = true;
        self
    }

    /// Set to hide window
    pub const fn hidden(&mut self) -> &mut Self {
        self.window_hidden = true;
        self
    }

    /// Set to allow windows running while minimized
    pub const fn always_run(&mut self) -> &mut Self {
        self.window_always_run = true;
        self
    }

    /// Set to minimize window (iconify)
    pub const fn minimized(&mut self) -> &mut Self {
        self.window_minimized = true;
        self
    }

    /// Set to maximize window (expanded to monitor)
    pub const fn maximized(&mut self) -> &mut Self {
        self.window_maximized = true;
        self
    }

    /// Set to window non focused
    pub const fn unfocused(&mut self) -> &mut Self {
        self.window_unfocused = true;
        self
    }

    /// Set to window always on top
    pub const fn topmost(&mut self) -> &mut Self {
        self.window_topmost = true;
        self
    }

    /// Set to support HighDPI
    pub const fn highdpi(&mut self) -> &mut Self {
        self.window_highdpi = true;
        self
    }

    /// Set to support mouse passthrough, only supported when [`Self::undecorated`]
    pub const fn mouse_passthrough(&mut self) -> &mut Self {
        self.window_mouse_passthrough = true;
        self
    }

    /// Set to run program in borderless windowed mode
    pub const fn borderless_windowed_mode(&mut self) -> &mut Self {
        self.borderless_windowed_mode = true;
        self
    }

    /// Set to try enabling interlaced video format (for V3D)
    pub const fn interlaced_hint(&mut self) -> &mut Self {
        self.interlaced_hint = true;
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
    pub fn build(&self) -> (RaylibHandle, RaylibThread) {
        use crate::consts::ConfigFlags::*;
        let mut flags = 0u32;
        if self.fullscreen_mode {
            flags |= FLAG_FULLSCREEN_MODE as u32;
        }
        if self.window_resizable {
            flags |= FLAG_WINDOW_RESIZABLE as u32;
        }
        if self.window_undecorated {
            flags |= FLAG_WINDOW_UNDECORATED as u32;
        }
        if self.window_hidden {
            flags |= FLAG_WINDOW_HIDDEN as u32;
        }
        if self.window_minimized {
            flags |= FLAG_WINDOW_MINIMIZED as u32;
        }
        if self.window_maximized {
            flags |= FLAG_WINDOW_MAXIMIZED as u32;
        }
        if self.window_unfocused {
            flags |= FLAG_WINDOW_UNFOCUSED as u32;
        }
        if self.window_topmost {
            flags |= FLAG_WINDOW_TOPMOST as u32;
        }
        if self.window_always_run {
            flags |= FLAG_WINDOW_ALWAYS_RUN as u32;
        }
        if self.window_transparent {
            flags |= FLAG_WINDOW_TRANSPARENT as u32;
        }
        if self.window_highdpi {
            flags |= FLAG_WINDOW_HIGHDPI as u32;
        }
        if self.window_mouse_passthrough {
            flags |= FLAG_WINDOW_MOUSE_PASSTHROUGH as u32;
        }
        if self.borderless_windowed_mode {
            flags |= FLAG_BORDERLESS_WINDOWED_MODE as u32;
        }
        if self.msaa_4x_hint {
            flags |= FLAG_MSAA_4X_HINT as u32;
        }
        if self.vsync_hint {
            flags |= FLAG_VSYNC_HINT as u32;
        }
        if self.interlaced_hint {
            flags |= FLAG_INTERLACED_HINT as u32;
        }

        unsafe {
            ffi::SetConfigFlags(flags);
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
fn init_window(width: i32, height: i32, title: &str) -> RaylibHandle {
    if unsafe { ffi::IsWindowReady() } {
        panic!("Attempted to initialize raylib-rs more than once!");
    } else {
        unsafe {
            let c_title = CString::new(title).unwrap();
            ffi::InitWindow(width, height, c_title.as_ptr());
        }
        if !unsafe { ffi::IsWindowReady() } {
            panic!("Attempting to create window failed!");
        }

        RaylibHandle(())
    }
}
