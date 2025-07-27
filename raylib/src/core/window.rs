//! Window manipulation functions
use crate::core::math::{Matrix, Ray, Vector2};
use crate::core::{RaylibHandle, RaylibThread};
use crate::{MintVec2, MintVec3, ffi};
use std::ffi::{CStr, CString, NulError};
use std::str::Utf8Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [`MonitorInfo`] grabs the sizes (virtual and physical) of your monitor
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MonitorInfo {
    /// The virtual width of the monitor
    pub width: i32,
    /// The virtual height of the monitor
    pub height: i32,
    /// The physical width of the monitor
    pub physical_width: i32,
    /// The physical height of the monitor
    pub physical_height: i32,
    /// The name of the monitor
    pub name: String,
    /// The position of the monitor
    pub position: Vector2,
}

/// Bitflags describing the configuration of the window.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WindowState(u32);

impl WindowState {
    /// Whether to try enabling V-Sync on GPU
    #[must_use]
    pub const fn vsync_hint(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_VSYNC_HINT as u32) != 0
    }

    /// Set to try enabling V-Sync on GPU
    pub const fn set_vsync_hint(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_VSYNC_HINT as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_VSYNC_HINT as u32);
        }
        self
    }

    /// Whether to run program in fullscreen
    #[must_use]
    pub const fn fullscreen_mode(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as u32) != 0
    }

    /// Set to run program in fullscreen
    pub const fn set_fullscreen_mode(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as u32);
        }
        self
    }

    /// Whether to allow resizable window
    #[must_use]
    pub const fn window_resizable(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32) != 0
    }

    /// Set to allow resizable window
    pub const fn set_window_resizable(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32);
        }
        self
    }

    /// Whether to disable window decoration (frame and buttons)
    #[must_use]
    pub const fn window_undecorated(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as u32) != 0
    }

    /// Set to disable window decoration (frame and buttons)
    pub const fn set_window_undecorated(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as u32);
        }
        self
    }

    /// Whether to hide window
    #[must_use]
    pub const fn window_hidden(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as u32) != 0
    }

    /// Set to hide window
    pub const fn set_window_hidden(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as u32);
        }
        self
    }

    /// Whether to minimize window (iconify)
    #[must_use]
    pub const fn window_minimized(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as u32) != 0
    }

    /// Set to minimize window (iconify)
    pub const fn set_window_minimized(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as u32);
        }
        self
    }

    /// Whether to maximize window (expanded to monitor)
    #[must_use]
    pub const fn window_maximized(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as u32) != 0
    }

    /// Set to maximize window (expanded to monitor)
    pub const fn set_window_maximized(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as u32);
        }
        self
    }

    /// Whether to window non focused
    #[must_use]
    pub const fn window_unfocused(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as u32) != 0
    }

    /// Set to window non focused
    pub const fn set_window_unfocused(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as u32);
        }
        self
    }

    /// Whether to window always on top
    #[must_use]
    pub const fn window_topmost(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as u32) != 0
    }

    /// Set to window always on top
    pub const fn set_window_topmost(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as u32);
        }
        self
    }

    /// Whether to allow windows running while minimized
    #[must_use]
    pub const fn window_always_run(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as u32) != 0
    }

    /// Set to allow windows running while minimized
    pub const fn set_window_always_run(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as u32);
        }
        self
    }

    /// Whether to allow transparent framebuffer
    #[must_use]
    pub const fn window_transparent(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as u32) != 0
    }

    /// Set to allow transparent framebuffer
    pub const fn set_window_transparent(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as u32);
        }
        self
    }

    /// Whether to support high DPI
    #[must_use]
    pub const fn window_highdpi(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32) != 0
    }

    /// Set to support high DPI
    pub const fn set_window_highdpi(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32);
        }
        self
    }

    /// Whether to try enabling MSAA 4X
    #[must_use]
    pub const fn msaa(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_MSAA_4X_HINT as u32) != 0
    }

    /// Set to try enabling MSAA 4X
    pub const fn set_msaa(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_MSAA_4X_HINT as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_MSAA_4X_HINT as u32);
        }
        self
    }

    /// Whether to try enabling interlaced video format (for V3D)
    #[must_use]
    pub const fn interlaced_hint(&self) -> bool {
        self.0 & (ffi::ConfigFlags::FLAG_INTERLACED_HINT as u32) != 0
    }

    /// Set to try enabling interlaced video format (for V3D)
    pub const fn set_interlaced_hint(&mut self, enabled: bool) -> &mut Self {
        if enabled {
            // set the bit
            self.0 |= ffi::ConfigFlags::FLAG_INTERLACED_HINT as u32;
        } else {
            // enable the bit
            self.0 &= !(ffi::ConfigFlags::FLAG_INTERLACED_HINT as u32);
        }
        self
    }
}

/// Get number of connected monitors
#[inline]
#[must_use]
pub fn get_monitor_count() -> i32 {
    unsafe { ffi::GetMonitorCount() }
}

/// Get current connected monitor
#[inline]
#[must_use]
pub fn get_current_monitor() -> i32 {
    unsafe { ffi::GetCurrentMonitor() }
}

/// Get current connected monitor
#[inline]
#[must_use]
pub fn get_current_monitor_index() -> i32 {
    get_current_monitor()
}

/// Get specified monitor refresh rate
#[inline]
#[must_use]
pub fn get_monitor_refresh_rate(monitor: i32) -> i32 {
    debug_assert!(
        0 <= monitor && monitor < get_monitor_count(),
        "monitor index out of range"
    );

    unsafe { ffi::GetMonitorRefreshRate(monitor) }
}

/// Get width of monitor
///
/// Only checks that monitor index is in range in debug mode
#[inline]
#[must_use]
pub fn get_monitor_width(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    unsafe { ffi::GetMonitorWidth(monitor) }
}

/// Get height of monitor
///
/// Only checks that monitor index is in range in debug mode
#[inline]
#[must_use]
pub fn get_monitor_height(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    unsafe { ffi::GetMonitorHeight(monitor) }
}

/// Get physical width of monitor
/// Only checks that monitor index is in range in debug mode
#[inline]
#[must_use]
pub fn get_monitor_physical_width(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    unsafe { ffi::GetMonitorPhysicalWidth(monitor) }
}

/// Get physical height of monitor
///
/// Only checks that monitor index is in range in debug mode
#[inline]
#[must_use]
pub fn get_monitor_physical_height(monitor: i32) -> i32 {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    unsafe { ffi::GetMonitorPhysicalHeight(monitor) }
}

/// Get name of monitor
///
/// Only checks that monitor index is in range in debug mode
///
/// # Errors
///
/// This function returns [`Utf8Error`] if [`ffi::GetMonitorName`] returns a
/// string that is not encoded in UTF-8.
#[inline]
pub fn get_monitor_name(monitor: i32) -> Result<String, Utf8Error> {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    let monitor_name = unsafe { ffi::GetMonitorName(monitor) };
    let c = unsafe { CStr::from_ptr(monitor_name) };
    Ok(c.to_str()?.to_owned())
}

/// Get position of monitor
///
/// Only checks that monitor index is in range in debug mode
#[inline]
#[must_use]
pub fn get_monitor_position(monitor: i32) -> Vector2 {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    unsafe { ffi::GetMonitorPosition(monitor).into() }
}

/// Gets the attributes of the monitor as well as the name
///
/// # Example
/// ```
/// # use std::str::Utf8Error;
/// # use raylib::prelude::*;
/// fn main() -> Result<(), Utf8Error> {
///     let count = get_monitor_count();
///     for i in 0..count {
///         println!("{:?}", get_monitor_info(i)?);
///     }
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// This function returns [`Utf8Error`] if [`get_monitor_name`] fails.
pub fn get_monitor_info(monitor: i32) -> Result<MonitorInfo, Utf8Error> {
    let len = get_monitor_count();
    debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");

    Ok(MonitorInfo {
        width: get_monitor_width(monitor),
        height: get_monitor_height(monitor),
        physical_height: get_monitor_physical_height(monitor),
        physical_width: get_monitor_physical_width(monitor),
        name: get_monitor_name(monitor)?,
        position: get_monitor_position(monitor),
    })
}

/// Returns camera transform matrix (view matrix)
///
/// # Example
/// ```
/// # use raylib::prelude::*;
/// let c = Camera::perspective(
///     Vector3::new(0.0, 0.0, 0.0),
///     Vector3::new(0.0, 0.0, -1.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     90.0,
/// );
/// let m = get_camera_matrix(&c);
/// assert_eq!(m, Matrix::identity());
/// ```
#[must_use]
pub fn get_camera_matrix(camera: impl Into<ffi::Camera>) -> Matrix {
    unsafe { ffi::GetCameraMatrix(camera.into()).into() }
}

/// Returns camera 2D transform matrix (view matrix)
///
/// # Example
/// ```
/// # use raylib::prelude::*;
/// let c = Camera2D::default();
/// let m = get_camera_matrix2D(&c);
/// let mut check = Matrix::zero();
/// check.m10 = 1.0;
/// check.m15 = 1.0;
/// assert_eq!(m, check);
/// ```
#[allow(non_snake_case, reason = "consistent style")]
#[must_use]
pub fn get_camera_matrix2D(camera: impl Into<ffi::Camera2D>) -> Matrix {
    unsafe { ffi::GetCameraMatrix2D(camera.into()).into() }
}

impl RaylibHandle {
    /// Get clipboard text content
    ///
    /// # Errors
    ///
    /// Returns [`Utf8Error`] if the clipboard does not contain valid UTF-8.
    pub fn get_clipboard_text(&self) -> Result<String, Utf8Error> {
        // TODO: can this be null?
        let c = unsafe { ffi::GetClipboardText() };
        let c = unsafe { CStr::from_ptr(c) };
        Ok(c.to_str()?.to_owned())
    }

    /// Set clipboard text content
    ///
    /// # Errors
    ///
    /// This method returns [`NulError`] if `text` contains an internal 0 byte.
    pub fn set_clipboard_text(&mut self, text: &str) -> Result<(), NulError> {
        let s = CString::new(text)?;
        unsafe {
            ffi::SetClipboardText(s.as_ptr());
        }
        Ok(())
    }
}

// Screen-space-related functions
impl RaylibHandle {
    /// Get a ray trace from screen position (i.e mouse)
    #[inline]
    #[must_use]
    pub fn get_screen_to_world_ray(
        &self,
        mouse_position: impl Into<MintVec2>,
        camera: impl Into<ffi::Camera>,
    ) -> Ray {
        unsafe { ffi::GetScreenToWorldRay(mouse_position.into(), camera.into()).into() }
    }

    /// Get a ray trace from screen position (i.e mouse) in a viewport
    #[inline]
    #[must_use]
    pub fn get_screen_to_world_ray_ex(
        &self,
        mouse_position: impl Into<MintVec2>,
        camera: impl Into<ffi::Camera>,
        width: i32,
        height: i32,
    ) -> Ray {
        unsafe {
            ffi::GetScreenToWorldRayEx(mouse_position.into(), camera.into(), width, height).into()
        }
    }

    /// Returns the screen space position for a 3D world space position
    #[inline]
    #[must_use]
    pub fn get_world_to_screen(
        &self,
        position: impl Into<MintVec3>,
        camera: impl Into<ffi::Camera>,
    ) -> Vector2 {
        unsafe { ffi::GetWorldToScreen(position.into(), camera.into()).into() }
    }

    /// Returns the screen space position for a 2D camera world space position
    #[allow(non_snake_case, reason = "consisten style")]
    #[inline]
    #[must_use]
    pub fn get_world_to_screen2D(
        &self,
        position: impl Into<MintVec2>,
        camera: impl Into<ffi::Camera2D>,
    ) -> Vector2 {
        unsafe { ffi::GetWorldToScreen2D(position.into(), camera.into()).into() }
    }

    /// Returns size position for a 3D world space position
    #[inline]
    #[must_use]
    pub fn get_world_to_screen_ex(
        &self,
        position: impl Into<MintVec3>,
        camera: impl Into<ffi::Camera>,
        width: i32,
        height: i32,
    ) -> Vector2 {
        unsafe { ffi::GetWorldToScreenEx(position.into(), camera.into(), width, height).into() }
    }

    /// Returns the world space position for a 2D camera screen space position
    #[allow(non_snake_case, reason = "consistent style")]
    #[inline]
    #[must_use]
    pub fn get_screen_to_world2D(
        &self,
        position: impl Into<MintVec2>,
        camera: impl Into<ffi::Camera2D>,
    ) -> Vector2 {
        unsafe { ffi::GetScreenToWorld2D(position.into(), camera.into()).into() }
    }
}

// Timing related functions
impl RaylibHandle {
    /// Set target FPS (maximum)
    ///
    /// # Panics
    ///
    /// This method will panic if `fps` is greater than [`i32::MAX`].
    #[inline]
    pub fn set_target_fps(&mut self, fps: u32) {
        unsafe {
            ffi::SetTargetFPS(fps.try_into().expect("fps should not exceed i32::MAX"));
        }
    }

    /// Returns current FPS
    ///
    /// # Panics
    ///
    /// This method will panic if [`ffi::GetFPS`] returns a negative number.
    #[inline]
    #[must_use]
    pub fn get_fps(&self) -> u32 {
        unsafe {
            ffi::GetFPS()
                .try_into()
                .expect("FPS should never be negative")
        }
    }

    /// Returns time in seconds for last frame drawn
    #[inline]
    #[must_use]
    pub fn get_frame_time(&self) -> f32 {
        unsafe { ffi::GetFrameTime() }
    }

    /// Returns elapsed time in seconds since [`ffi::InitWindow()`]
    #[inline]
    #[must_use]
    pub fn get_time(&self) -> f64 {
        unsafe { ffi::GetTime() }
    }
}

// Window handling functions
impl RaylibHandle {
    /// Checks if `KEY_ESCAPE` or Close icon was pressed.
    /// Do not call on web unless you are compiling with asyncify.
    #[inline]
    #[must_use]
    pub fn window_should_close(&self) -> bool {
        unsafe { ffi::WindowShouldClose() }
    }

    /// Checks if window has been initialized successfully.
    #[inline]
    #[must_use]
    pub fn is_window_ready(&self) -> bool {
        unsafe { ffi::IsWindowReady() }
    }

    /// Set window state: maximized, if resizable
    #[inline]
    pub fn maximize_window(&mut self) {
        unsafe { ffi::MaximizeWindow() }
    }

    /// Set window state: minimized, if resizable
    #[inline]
    pub fn minimize_window(&mut self) {
        unsafe { ffi::MinimizeWindow() }
    }

    /// Set window state: not minimized/maximized
    #[inline]
    pub fn restore_window(&mut self) {
        unsafe { ffi::RestoreWindow() }
    }

    /// Check if window is currently maximized
    #[inline]
    #[must_use]
    pub fn is_window_maximized(&self) -> bool {
        unsafe { ffi::IsWindowMaximized() }
    }

    /// Checks if window has been minimized (or lost focus).
    #[inline]
    #[must_use]
    pub fn is_window_minimized(&self) -> bool {
        unsafe { ffi::IsWindowMinimized() }
    }

    /// Checks if window has been resized.
    #[inline]
    #[must_use]
    pub fn is_window_resized(&self) -> bool {
        unsafe { ffi::IsWindowResized() }
    }

    /// Checks if window has been hidden.
    #[inline]
    #[must_use]
    pub fn is_window_hidden(&self) -> bool {
        unsafe { ffi::IsWindowHidden() }
    }

    /// Returns whether or not window is in fullscreen mode
    #[inline]
    #[must_use]
    pub fn is_window_fullscreen(&self) -> bool {
        unsafe { ffi::IsWindowFullscreen() }
    }

    /// Check if window is currently focused (only `PLATFORM_DESKTOP`)
    #[inline]
    #[must_use]
    pub fn is_window_focused(&self) -> bool {
        unsafe { ffi::IsWindowFocused() }
    }

    /// Check if window is currently focused (only `PLATFORM_DESKTOP`)
    #[inline]
    #[must_use]
    pub fn get_window_scale_dpi(&self) -> Vector2 {
        unsafe { ffi::GetWindowScaleDPI().into() }
    }

    /// Check if cursor is on the current screen.
    #[inline]
    #[must_use]
    pub fn is_cursor_on_screen(&self) -> bool {
        unsafe { ffi::IsCursorOnScreen() }
    }

    /// Set mouse cursor
    #[inline]
    pub fn set_mouse_cursor(&self, cursor: crate::consts::MouseCursor) {
        unsafe { ffi::SetMouseCursor(cursor as i32) }
    }

    /// Toggles fullscreen mode (only on desktop platforms).
    #[inline]
    pub fn toggle_fullscreen(&mut self) {
        unsafe {
            ffi::ToggleFullscreen();
        }
    }

    /// Set window configuration state using flags
    #[inline]
    pub fn set_window_state(&mut self, state: WindowState) {
        unsafe { ffi::SetWindowState(state.0) }
    }

    /// Clear window configuration state flags
    #[inline]
    pub fn clear_window_state(&mut self, state: WindowState) {
        unsafe { ffi::ClearWindowState(state.0) }
    }

    /// Get the window config state
    #[must_use]
    pub fn get_window_state(&self) -> WindowState {
        let mut state = WindowState::default();

        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_VSYNC_HINT as u32) } {
            state.set_vsync_hint(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_FULLSCREEN_MODE as u32) } {
            state.set_fullscreen_mode(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_RESIZABLE as u32) } {
            state.set_window_resizable(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_UNDECORATED as u32) } {
            state.set_window_undecorated(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as u32) } {
            state.set_window_hidden(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_MINIMIZED as u32) } {
            state.set_window_minimized(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_MAXIMIZED as u32) } {
            state.set_window_maximized(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_UNFOCUSED as u32) } {
            state.set_window_unfocused(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_TOPMOST as u32) } {
            state.set_window_topmost(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_ALWAYS_RUN as u32) } {
            state.set_window_always_run(true);
        }

        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_TRANSPARENT as u32) } {
            state.set_window_transparent(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_WINDOW_HIGHDPI as u32) } {
            state.set_window_highdpi(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_MSAA_4X_HINT as u32) } {
            state.set_msaa(true);
        }
        if unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_INTERLACED_HINT as u32) } {
            state.set_interlaced_hint(true);
        }
        state
    }

    /// Sets icon for window (only on desktop platforms).
    #[inline]
    pub fn set_window_icon(&mut self, image: impl AsRef<ffi::Image>) {
        unsafe {
            ffi::SetWindowIcon(*image.as_ref());
        }
    }

    /// Set icon for window (multiple images, RGBA 32bit)
    ///
    /// # Panics
    ///
    /// This method will panic if `images` has a length greater than [`i32::MAX`].
    #[inline]
    pub fn set_window_icons(&mut self, images: &mut [ffi::Image]) {
        unsafe {
            ffi::SetWindowIcons(
                images.as_mut_ptr(),
                images
                    .len()
                    .try_into()
                    .expect("images should not exceed i32::MAX elements"),
            );
        }
    }

    /// Sets title for window (only on desktop platforms).
    ///
    /// # Panics
    ///
    /// This method will panic if `title` contains an internal 0 byte.
    #[inline]
    pub fn set_window_title(&self, _: &RaylibThread, title: &str) {
        let c_title = CString::new(title).expect("title should not contain an internal 0 byte");
        unsafe {
            ffi::SetWindowTitle(c_title.as_ptr());
        }
    }

    /// Sets window position on screen (only on desktop platforms).
    #[inline]
    pub fn set_window_position(&mut self, x: i32, y: i32) {
        unsafe {
            ffi::SetWindowPosition(x, y);
        }
    }

    /// Sets monitor for the current window (fullscreen mode).
    #[inline]
    pub fn set_window_monitor(&mut self, monitor: i32) {
        let len = get_monitor_count();
        debug_assert!(0 <= monitor && monitor < len, "monitor index out of range");
        unsafe {
            ffi::SetWindowMonitor(monitor);
        }
    }

    /// Sets minimum window dimensions (for `FLAG_WINDOW_RESIZABLE`).
    #[inline]
    pub fn set_window_min_size(&mut self, width: i32, height: i32) {
        unsafe {
            ffi::SetWindowMinSize(width, height);
        }
    }

    /// Sets maximum window dimensions (for `FLAG_WINDOW_RESIZABLE`).
    #[inline]
    pub fn set_window_max_size(&mut self, width: i32, height: i32) {
        unsafe {
            ffi::SetWindowMaxSize(width, height);
        }
    }

    /// Sets window dimensions.
    #[inline]
    pub fn set_window_size(&mut self, width: i32, height: i32) {
        unsafe {
            ffi::SetWindowSize(width, height);
        }
    }

    /// Set window opacity, value opacity is between 0.0 and 1.0
    #[inline]
    pub fn set_window_opacity(&mut self, opacity: f32) {
        unsafe { ffi::SetWindowOpacity(opacity) }
    }

    /// Get current render width which is equal to screen width * dpi scale
    #[inline]
    #[must_use]
    pub fn get_render_width(&self) -> i32 {
        unsafe { ffi::GetRenderWidth() }
    }

    /// Get current render width which is equal to screen height * dpi scale
    #[inline]
    #[must_use]
    pub fn get_render_height(&self) -> i32 {
        unsafe { ffi::GetRenderHeight() }
    }

    /// Get current screen width.
    #[inline]
    #[must_use]
    pub fn get_screen_width(&self) -> i32 {
        unsafe { ffi::GetScreenWidth() }
    }

    /// Gets current screen height.
    #[inline]
    #[must_use]
    pub fn get_screen_height(&self) -> i32 {
        unsafe { ffi::GetScreenHeight() }
    }

    /// Get window position
    #[inline]
    #[must_use]
    pub fn get_window_position(&self) -> Vector2 {
        unsafe { ffi::GetWindowPosition().into() }
    }

    /// Toggle window state: borderless windowed (only on desktop platforms).
    #[inline]
    pub fn toggle_borderless_windowed(&self) {
        unsafe { ffi::ToggleBorderlessWindowed() }
    }

    /// Focus the window (only on desktop platforms)
    #[inline]
    pub fn set_window_focused(&self) {
        unsafe { ffi::SetWindowFocused() }
    }
}

// Cursor-related functions
impl RaylibHandle {
    /// Shows mouse cursor.
    #[inline]
    pub fn show_cursor(&mut self) {
        unsafe {
            ffi::ShowCursor();
        }
    }

    /// Hides mouse cursor.
    #[inline]
    pub fn hide_cursor(&mut self) {
        unsafe {
            ffi::HideCursor();
        }
    }

    /// Checks if mouse cursor is not visible.
    #[inline]
    #[must_use]
    pub fn is_cursor_hidden(&self) -> bool {
        unsafe { ffi::IsCursorHidden() }
    }

    /// Enables mouse cursor (unlock cursor).
    #[inline]
    pub fn enable_cursor(&mut self) {
        unsafe {
            ffi::EnableCursor();
        }
    }

    /// Disables mouse cursor (lock cursor).
    #[inline]
    pub fn disable_cursor(&mut self) {
        unsafe {
            ffi::DisableCursor();
        }
    }

    /// Get native window handle
    #[inline]
    #[must_use]
    pub unsafe fn get_window_handle(&mut self) -> *mut ::std::os::raw::c_void {
        unsafe { ffi::GetWindowHandle() }
    }
}

// Advanced "frame control" functions.
// NOTE: Those functions are intended for advanced users that want full control over the frame processing
// By default EndDrawing() does this job: draws everything + SwapScreenBuffer() + manage frame timing + PollInputEvents()
// To avoid that behaviour and control frame processes manually, enable in config.h: SUPPORT_CUSTOM_FRAME_CONTROL
#[cfg(feature = "SUPPORT_CUSTOM_FRAME_CONTROL")]
impl RaylibHandle {
    /// Swap back buffer with front buffer (screen drawing)
    /// This function, by default, is already done when the handle is dropped.
    pub fn swap_screen_buffer(&self) {
        unsafe { ffi::SwapScreenBuffer() }
    }

    /// Register all input events
    pub fn poll_input_events(&self) {
        unsafe { ffi::PollInputEvents() }
    }

    /// Wait for some time (halt program execution)
    pub fn wait_time(&self, seconds: f64) {
        unsafe { ffi::WaitTime(seconds) }
    }
}
