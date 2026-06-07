#[macro_use]
mod macros;

pub use macros::AsRawMut;

pub mod audio;
/// Automation event recording and playback for deterministic input replay.
///
/// Wraps raylib's automation-event API so input streams (key presses, mouse moves, gamepad activity)
/// can be captured into an `AutomationEventList`, persisted to disk, and replayed frame-by-frame.
/// Useful for scripted demos, regression tests, and benchmark reproducibility.
///
/// # See also
///
/// See the *Input* chapter of the book for the underlying input API.
pub mod automation;
/// Custom callback hooks for audio streams and logging.
///
/// Provides safe wrappers around raylib's C function-pointer hooks: install a trace-log callback
/// to redirect raylib's diagnostic output into Rust's logging stack, or attach an audio-stream
/// processor to mutate sample data per frame. The wrappers stash boxed closures in static slots and
/// trampoline through `extern "C"` shims so user code can stay safe.
///
/// # See also
///
/// See the *Callbacks and Logging* chapter of the book.
pub mod callbacks;
#[cfg(not(feature = "nobuild"))]
pub mod camera;

pub mod collision;
/// Re-export module exposing the raylib `Color` type at `core::color::Color`.
///
/// The bulk of color manipulation — `from_hex`, `color_to_int`, `color_normalize`, `color_to_hsv`,
/// `alpha`, and the named-constant set (`RAYWHITE`, `BLACK`, …) — lives on the [`Color`] type itself
/// as inherent `impl` methods. This module exists solely to give `Color` a conventional path under
/// `raylib::core::color` alongside the other domain submodules; ordinary user code should reach for
/// `raylib::prelude::Color` instead.
///
/// [`Color`]: crate::ffi::Color
///
/// # See also
///
/// See the *Window and drawing* chapter of the book.
pub mod color {
    #[allow(unused_imports)]
    pub use crate::ffi::Color;
}
/// Low-level data utilities: DEFLATE compression and Base64 encoding.
///
/// Thin safe wrappers around raylib's `CompressData`/`DecompressData` (DEFLATE via sdefl/sinfl) and
/// `EncodeDataBase64`/`DecodeDataBase64`. Each returns a [`DataBuf`](crate::core::databuf::DataBuf)
/// that owns the raylib-allocated buffer and frees it through `MemFree` on drop. For cryptographic
/// digests see [`crate::core::hashes`]; for raw FFI access see [`crate::ffi`].
///
/// # See also
///
/// See the *Strings and allocations* chapter of the book.
pub mod data;
pub mod databuf;
pub mod drawing;
pub mod error;
pub mod file;

pub mod hashes;
pub mod input;
pub mod logging;
pub mod math;
pub mod misc;
pub mod models;
pub mod pixel;
pub mod shaders;
pub mod text;
pub mod texture;
pub mod vr;
pub mod window;

use raylib_sys::TraceLogLevel;

#[cfg(test)]
mod color_tests {
    use crate::ffi::Color;

    // --- Color::new / field access ---

    #[test]
    fn rgba_fields() {
        let c = Color::new(10, 20, 30, 40);
        assert_eq!((c.r, c.g, c.b, c.a), (10, 20, 30, 40));
    }

    // --- Color::from_hex ---

    #[test]
    fn from_hex_red() {
        let c = Color::from_hex("ff0000").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255));
    }

    #[test]
    fn from_hex_white() {
        let c = Color::from_hex("ffffff").unwrap();
        assert_eq!((c.r, c.g, c.b, c.a), (255, 255, 255, 255));
    }

    #[test]
    fn from_hex_invalid_returns_err() {
        assert!(Color::from_hex("zzzzzz").is_err());
    }

    // --- Color::get_color (from u32 hex value) ---

    #[test]
    fn get_color_opaque_red() {
        // 0xRRGGBBAA — raylib packs it as 0xFF0000FF = red fully opaque
        let c = Color::get_color(0xFF0000FF);
        assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255));
    }

    // --- Color::color_to_int ---

    #[test]
    fn color_to_int_roundtrip() {
        let c = Color::new(100, 150, 200, 255);
        let packed = c.color_to_int();
        // Reparse via get_color using the packed u32
        let back = Color::get_color(packed as u32);
        assert_eq!((back.r, back.g, back.b, back.a), (c.r, c.g, c.b, c.a));
    }

    // --- Color::color_normalize / color_from_normalized round-trip ---

    #[test]
    fn normalize_roundtrip() {
        let c = Color::new(51, 102, 153, 204); // 0.2, 0.4, 0.6, 0.8 in [0..1]
        let norm = c.color_normalize();
        let back = Color::color_from_normalized(norm);
        // Round-trip through f32 may drift by 1 LSB
        assert!((back.r as i32 - c.r as i32).abs() <= 1);
        assert!((back.g as i32 - c.g as i32).abs() <= 1);
        assert!((back.b as i32 - c.b as i32).abs() <= 1);
        assert!((back.a as i32 - c.a as i32).abs() <= 1);
    }

    // --- Color::color_to_hsv / color_from_hsv round-trip ---

    #[test]
    fn hsv_roundtrip_red() {
        // Pure red: H=0, S=1, V=1
        let c = Color::new(255, 0, 0, 255);
        let hsv = c.color_to_hsv();
        // hue should be ~0 (or 360), saturation and value ~1.0
        assert!(
            (hsv.y - 1.0).abs() < 1e-3,
            "saturation should be 1.0, got {}",
            hsv.y
        );
        assert!(
            (hsv.z - 1.0).abs() < 1e-3,
            "value should be 1.0, got {}",
            hsv.z
        );

        let back = Color::color_from_hsv(hsv.x, hsv.y, hsv.z);
        assert_eq!((back.r, back.g, back.b), (255, 0, 0));
    }

    #[test]
    fn color_from_hsv_white() {
        // H=any, S=0, V=1 → white
        let c = Color::color_from_hsv(0.0, 0.0, 1.0);
        assert_eq!((c.r, c.g, c.b), (255, 255, 255));
    }

    // --- Color::alpha ---

    #[test]
    fn alpha_fully_transparent() {
        let c = Color::new(255, 0, 0, 255).alpha(0.0);
        assert_eq!(c.a, 0);
    }

    #[test]
    fn alpha_fully_opaque() {
        let c = Color::new(255, 0, 0, 0).alpha(1.0);
        assert_eq!(c.a, 255);
    }

    // --- PartialEq ---

    #[test]
    fn color_equality() {
        let a = Color::new(1, 2, 3, 4);
        let b = Color::new(1, 2, 3, 4);
        assert_eq!(a, b);
    }

    #[test]
    fn color_inequality() {
        let a = Color::new(1, 2, 3, 4);
        let b = Color::new(5, 6, 7, 8);
        assert_ne!(a, b);
    }
}

use crate::ffi;
use std::ffi::CString;
use std::marker::PhantomData;

// shamelessly stolen from imgui
/// Builds a NUL-terminated C string for FFI calls — `&'static CStr` from a string literal, or an
/// owned `CString` from a format expression.
///
/// raygui's draw functions take `*const c_char`, which in the safe wrapper appears as `&CStr`. The
/// naive path — `CString::new(s).unwrap()` — allocates on the heap and copies the bytes every frame.
/// `rstr!` sidesteps that cost for the common case where the label is a literal: it appends a NUL
/// byte at **compile time** via `concat!`, then reinterprets the resulting `&'static str` as a
/// `&'static CStr` with `CStr::from_bytes_with_nul_unchecked`. Because the NUL is statically known
/// to terminate the slice and no interior NUL exists in a string literal that ends with `"\0"`
/// (assuming the caller did not embed one), the unchecked call is sound.
///
/// The two-argument form (`rstr!("score: {}", n)`) forwards to `format!` and wraps the result in a
/// freshly allocated `CString` — use it only when the value genuinely varies between frames, since
/// it does allocate. Prefer the single-argument literal form inside hot GUI loops.
///
/// # Panics
///
/// The two-argument form panics if the formatted string contains an interior NUL byte. The
/// single-argument form does not panic, but the caller must not embed `\0` in the literal — doing
/// so would silently truncate the `&CStr`.
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

/// Token proving that the current code is executing on the raylib init thread.
///
/// `RaylibThread` is `!Send` and `!Sync` — it **must never** be sent to another thread, stored in
/// an `Arc`/`Mutex`, or moved out of the thread on which [`init`] was called. This invariant is
/// required because many platforms (notably macOS via Cocoa) mandate that all window/GL calls
/// originate from the main thread. Functions that require this guarantee accept a `&RaylibThread`
/// parameter; the borrow checker then enforces the constraint at compile time.
///
/// You receive a `RaylibThread` as the second element of the pair returned by
/// [`RaylibBuilder::build`].
///
/// ```rust,compile_fail
/// use raylib::prelude::*;
/// // RaylibThread is !Send — this must not compile:
/// fn require_send<T: Send>() {}
/// require_send::<RaylibThread>();
/// ```
#[derive(Clone, Debug)]
pub struct RaylibThread(PhantomData<*const ()>);

/// The main interface into the raylib API.
///
/// `RaylibHandle` owns the raylib window and OpenGL context. All drawing, input, audio, and
/// windowing functions are methods on this type (or on draw-mode guards borrowed from it). Obtain
/// a `RaylibHandle` by calling [`init`] to configure options such as VSync, MSAA, fullscreen, and
/// window title before opening the window.
///
/// When `RaylibHandle` is dropped, raylib's `CloseWindow` is called automatically — no explicit
/// teardown is needed.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init()
///     .size(800, 600)
///     .title("My Game")
///     .vsync()
///     .build();
///
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_text("Hello, raylib!", 20, 20, 24, Color::BLACK);
/// }
/// ```
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

/// Fluent builder for configuring the raylib window before it is created.
///
/// Obtain a `RaylibBuilder` via [`init`], chain the desired options, then call [`build`] to open
/// the window and receive a [`(RaylibHandle, RaylibThread)`](RaylibHandle) pair.
///
/// [`build`]: RaylibBuilder::build
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init()
///     .size(1280, 720)
///     .title("My Game")
///     .vsync()
///     .msaa_4x()
///     .build();
///
/// // rl is now ready; enter the game loop.
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
/// }
/// ```
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
    #[cfg(feature = "log")]
    bridge_log: bool,
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

    /// Forwards raylib's `TraceLog` output into the [`log`] crate facade
    /// (target `"raylib"`), so the application's logger (`env_logger`,
    /// `tracing-log`, …) receives raylib's logs under standard
    /// `RUST_LOG`-style filtering.
    ///
    /// Installed during [`build`](Self::build) *before* `InitWindow`, so
    /// raylib's init logging is captured too. Makes the `log` facade the
    /// single level filter by setting raylib's own threshold to `LOG_ALL`
    /// — overriding any [`log_level`](Self::log_level) — and claims the
    /// same single callback slot as
    /// [`set_trace_log_callback`](crate::core::callbacks::set_trace_log_callback)
    /// (mutually exclusive; last writer wins).
    ///
    /// The bridge only emits into the facade; install a logger yourself
    /// (e.g. `env_logger::init()`) or the messages are silently dropped.
    ///
    /// Requires the `SUPPORT_TRACELOG` feature (included in `default`);
    /// without it raylib's C-side `TraceLog` is compiled out and the
    /// bridge receives nothing.
    ///
    /// The bridge runs inside raylib's trace-log callback: a logger that
    /// itself calls back into raylib logging (e.g. via
    /// [`trace_log`](crate::core::logging::trace_log)) would recurse
    /// unboundedly — don't log to raylib from your `log` backend.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // env_logger::init();   // any `log`-compatible logger
    /// let (mut rl, thread) = raylib::init()
    ///     .size(800, 450)
    ///     .title("game")
    ///     .log_to_rust()
    ///     .build();
    /// log::info!("app and raylib logs share one pipeline now");
    /// ```
    #[cfg(feature = "log")]
    pub const fn log_to_rust(&mut self) -> &mut Self {
        self.bridge_log = true;
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

        // The log bridge claims the trace-log callback slot and makes the
        // `log` facade the single filter — installed before InitWindow so
        // init logging is captured, and after the SetTraceLogLevel above
        // so its LOG_ALL override wins over `.log_level()`.
        #[cfg(feature = "log")]
        if self.bridge_log {
            crate::core::logging::install_log_bridge();
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
