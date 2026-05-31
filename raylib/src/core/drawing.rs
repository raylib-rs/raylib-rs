//! Immediate-mode drawing surfaces and RAII mode guards.
//!
//! All drawing in raylib-rs follows an immediate-mode model: you call [`RaylibHandle::begin_drawing`]
//! to receive a [`RaylibDrawHandle`] guard, issue draw calls on it, then let the guard drop —
//! which automatically calls `EndDrawing` (presenting the frame and timing). Types that can serve
//! as a drawing surface implement the [`RaylibDraw`] trait.
//!
//! Additional mode guards nest inside a draw frame:
//! - [`RaylibTextureMode`] — off-screen rendering via `begin_texture_mode` / `EndTextureMode`.
//! - [`RaylibMode2D`] — 2D camera transform via `begin_mode2D` / `EndMode2D`.
//! - [`RaylibMode3D`] — 3D camera via `begin_mode3D` / `EndMode3D`.
//! - [`RaylibVRMode`] — VR stereo rendering via `begin_vr_stereo_mode`.
//! - [`RaylibShaderMode`] — custom shader pass via `begin_shader_mode`.
//! - [`RaylibScissorMode`] — scissor-rectangle clipping via `begin_scissor_mode`.
//!
//! Each guard also implements [`RaylibDraw`] (and [`RaylibDraw3D`] where applicable), so you can
//! chain modes by calling `begin_mode3D` on a `RaylibDrawHandle`, for example.

use raylib_sys::Rectangle;

use crate::core::texture::Texture2D;
use crate::core::vr::VrStereoConfig;
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi;
use crate::math::Matrix;
use crate::math::Vector2;
use crate::math::Vector3;
use crate::models::WeakMaterial;
use std::ffi::CString;
use std::{convert::AsRef, marker::PhantomData};

use super::shaders::Shader;

/// Seems like all draw commands must be issued from the main thread
impl RaylibHandle {
    #[inline]
    #[must_use]
    /// Setup canvas (framebuffer) to start drawing.
    /// Prefer using the closure version, [RaylibHandle::draw]. This version returns a handle that calls [raylib_sys::EndDrawing] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    pub fn begin_drawing<'a>(&'a mut self, _: &RaylibThread) -> RaylibDrawHandle<'a> {
        unsafe {
            ffi::BeginDrawing();
        };

        let d = RaylibDrawHandle(self);
        d
    }
    /// Setup canvas (framebuffer) to start drawing.
    // Every FnMut is a FnOnce, but not every FnOnce is a FnMut. The closure may possibly execute multiple times throughout the program, but not multiple times in a single call to this method.
    // Taking a FnOnce instead of a FnMut when the function only needs to be called once in this method makes the method slightly more versatile/less needlessly restrictive for no actual cost.
    pub fn draw<'a>(&'a mut self, _: &RaylibThread, func: impl FnOnce(RaylibDrawHandle<'a>)) {
        unsafe {
            ffi::BeginDrawing();
        };
        func(RaylibDrawHandle(self));
        // Uncomment the following if RaylibDrawHandle has been changed to no longer call EndDrawing() in its drop implementation:
        // unsafe {
        //     ffi::EndDrawing();
        // }
    }
}

/// RAII draw handle returned by [`RaylibHandle::begin_drawing`] — calls `EndDrawing` on drop.
///
/// Holds an exclusive borrow of the [`RaylibHandle`] for the duration of a frame. Drop runs
/// raylib's `EndDrawing`, which flushes the queued GPU commands, swaps buffers, presents the
/// frame, and enforces the configured target frame rate. All [`RaylibDraw`] methods and the
/// `begin_*` mode entries (texture / 2D / 3D / VR / shader / blend / scissor) hang off this
/// guard via [`Deref`](std::ops::Deref) to `RaylibHandle` and the extension traits.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_text("hello", 10, 10, 20, Color::BLACK);
///     // `d` drops here → EndDrawing is called automatically.
/// }
/// ```
///
/// # See also
///
/// - [`RaylibHandle::begin_drawing`] — constructor for this guard.
/// - [`RaylibHandle::draw`] — closure-form alternative that scopes the guard for you.
/// - [`RaylibDraw`] — drawing methods available on the guard.
/// - [`RaylibTextureMode`], [`RaylibMode2D`], [`RaylibMode3D`] — nested mode guards.
pub struct RaylibDrawHandle<'a>(&'a mut RaylibHandle);

impl RaylibDrawHandle<'_> {
    #[deprecated = "Calling begin_drawing within RaylibDrawHandle will result in a runtime error."]
    #[doc(hidden)]
    pub fn begin_drawing(&'_ mut self, _: &RaylibThread) -> RaylibDrawHandle<'_> {
        panic!("Nested begin_drawing call")
    }
    #[deprecated = "Calling draw within RaylibDrawHandle will result in a runtime error."]
    #[doc(hidden)]
    pub fn draw(&mut self, _: &RaylibThread, mut _func: impl FnMut(RaylibDrawHandle)) {
        panic!("Nested draw call")
    }
}

impl Drop for RaylibDrawHandle<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::EndDrawing();
        }
    }
}

impl std::ops::Deref for RaylibDrawHandle<'_> {
    type Target = RaylibHandle;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl std::ops::DerefMut for RaylibDrawHandle<'_> {
    fn deref_mut(&mut self) -> &mut RaylibHandle {
        self.0
    }
}
impl RaylibDraw for RaylibDrawHandle<'_> {}

// Texture2D Stuff

// The texture does not need to be held exclusively for the duration of the *previous* draw mode, nor does the *previous* draw mode need to outlive the texture reference.
// The texture exclusivity and previous draw mode only need to outlive the *current* draw mode. So they can (and should) have separate lifetimes.
//
// Additionally: the texture is not actually *used* by this wrapper after construction, only the previous draw mode.
// Some space can be saved by using a phantom instead of copying the actual reference.
// The PhantomData will ensure that the borrow checker still analyzes as though the mutable texture reference was held, without physically storing it in the runtime memory.
/// RAII draw handle for rendering to a `RenderTexture2D` — calls `EndTextureMode` on drop.
///
/// Drop runs raylib's `EndTextureMode`, unbinding the render-texture's framebuffer and
/// restoring the default framebuffer as the active draw target. Mutates the underlying
/// `RenderTexture2D`'s color attachment in place; the texture remains usable for sampling
/// (e.g. via [`RaylibDraw::draw_texture`]) once the guard drops.
///
/// Construct via [`RaylibTextureModeExt::begin_texture_mode`] on a [`RaylibDrawHandle`] or
/// directly on a [`RaylibHandle`]. The guard implements [`RaylibDraw`], so all 2D drawing
/// methods are available through it.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "software_renderer")] {
/// use raylib::prelude::*;
/// use raylib::test_harness::*;
///
/// with_headless(64, 64, |rl, thread| {
///     let mut target = rl
///         .load_render_texture(thread, 32, 32)
///         .expect("render texture");
///     let img = render_frame(rl, thread, |d| {
///         d.clear_background(Color::WHITE);
///         {
///             let mut t = d.begin_texture_mode(thread, &mut target);
///             t.clear_background(Color::BLUE);
///             t.draw_rectangle(0, 0, 32, 32, Color::RED);
///         }
///         d.draw_texture(target.texture(), 0, 0, Color::WHITE);
///     });
///     // Off-screen content blitted into the default framebuffer.
///     assert_pixel(&img, 5, 5, Color::RED, 0);
///     // Pixels outside the blitted region remain the WHITE clear.
///     assert_pixel(&img, 50, 50, Color::WHITE, 0);
/// });
/// # }
/// ```
///
/// # See also
///
/// - [`RaylibTextureModeExt::begin_texture_mode`] — constructor for this guard.
/// - [`RaylibTextureModeExt::draw_texture_mode`] — closure-form alternative.
/// - [`RaylibDraw`] — drawing methods available on this guard.
pub struct RaylibTextureMode<'a, 'b, T: 'a>(&'a mut T, PhantomData<&'b mut ffi::RenderTexture2D>);

impl<'a, T: 'a> Drop for RaylibTextureMode<'a, '_, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndTextureMode() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibTextureMode<'a, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> std::ops::DerefMut for RaylibTextureMode<'a, '_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}
// framebuffer: &'a mut ffi::RenderTexture2D,

/// Extension trait providing render-texture mode entry on draw handles.
///
/// Implemented for [`RaylibDrawHandle`] (so you can begin a texture-mode pass inside a
/// regular draw frame) and for [`RaylibHandle`] itself (off-screen renders outside an
/// active draw frame are also valid). The trait's two methods are alternative forms of the
/// same operation: [`begin_texture_mode`](Self::begin_texture_mode) returns a RAII guard,
/// while [`draw_texture_mode`](Self::draw_texture_mode) takes a closure and ends the mode
/// when the closure returns.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let mut target = rl
///     .load_render_texture(&thread, 256, 256)
///     .expect("render texture");
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.draw_texture_mode(&thread, &mut target, |mut t| {
///         t.clear_background(Color::BLUE);
///         t.draw_circle(128, 128, 64.0, Color::YELLOW);
///     });
///     d.clear_background(Color::RAYWHITE);
///     d.draw_texture(target.texture(), 0, 0, Color::WHITE);
/// }
/// ```
///
/// # See also
///
/// - [`RaylibTextureMode`] — RAII guard returned by `begin_texture_mode`.
pub trait RaylibTextureModeExt
where
    Self: Sized,
{
    /// Begin drawing to render texture.
    /// Prefer using the closure version, [RaylibTextureModeExt::draw_texture_mode] . This version returns a handle that calls [raylib_sys::EndTextureMode] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    fn begin_texture_mode<'a, 'b>(
        &'a mut self,
        _: &RaylibThread,
        framebuffer: &'b mut ffi::RenderTexture2D,
    ) -> RaylibTextureMode<'a, 'b, Self> {
        unsafe { ffi::BeginTextureMode(*framebuffer) }
        RaylibTextureMode(self, PhantomData)
    }

    /// Begin drawing to render texture.
    fn draw_texture_mode<'a, 'b>(
        &'a mut self,
        _: &RaylibThread,
        framebuffer: &'b mut ffi::RenderTexture2D,
        func: impl FnOnce(RaylibTextureMode<'a, 'b, Self>),
    ) {
        unsafe { ffi::BeginTextureMode(*framebuffer) }
        func(RaylibTextureMode(self, PhantomData));
        // Uncomment the following if RaylibTextureMode has been changed to no longer call EndTextureMode() in its drop implementation:
        // unsafe { ffi::EndTextureMode(); }
    }
}

// Only the DrawHandle and the RaylibHandle can start a texture
impl RaylibTextureModeExt for RaylibDrawHandle<'_> {}
impl RaylibTextureModeExt for RaylibHandle {}
impl<'a, T: 'a> RaylibDraw for RaylibTextureMode<'a, '_, T> {}

// VR Stuff

// Lifetime 'a is duplicatively stored in a PhantomData so that the borrow checker knows T is being held *exclusively* for the lifetime of the mode, without giving the false impression that it can actually be mutated by the library.
/// RAII draw handle for VR stereo rendering — calls `EndVrStereoMode` on drop.
///
/// While the guard is alive, raylib renders each draw call twice (once per eye) using the
/// stereo projection / view matrices in the borrowed [`VrStereoConfig`]. Drop runs
/// `EndVrStereoMode`, restoring the standard mono projection. The guard implements
/// [`RaylibDraw`] via deref, so all 2D / 3D draw calls issued during the scope are stereo.
///
/// Requires a real or simulator VR backend; not exercised by the headless software
/// renderer.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("vr-demo").build();
/// let device = VrDeviceInfo {
///     h_resolution: 2160,
///     v_resolution: 1200,
///     h_screen_size: 0.133793,
///     v_screen_size: 0.0669,
///     eye_to_screen_distance: 0.041,
///     lens_separation_distance: 0.07,
///     interpupillary_distance: 0.07,
///     lens_distortion_values: [1.0, 0.22, 0.24, 0.0],
///     chroma_ab_correction: [0.996, -0.004, 1.014, 0.0],
/// };
/// let mut config = rl.load_vr_stereo_config(&thread, device);
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     {
///         let mut v = d.begin_vr_stereo_mode(&thread, &mut config);
///         v.draw_rectangle(10, 10, 100, 50, Color::RED);
///         // Drop ends stereo mode automatically.
///     }
/// }
/// ```
///
/// # See also
///
/// - [`RaylibVRModeExt::begin_vr_stereo_mode`] — constructor for this guard.
/// - [`VrStereoConfig`] — stereo projection configuration the guard borrows.
pub struct RaylibVRMode<'a, 'b, T: 'a>(
    &'a T,
    PhantomData<&'a mut T>,
    PhantomData<&'b mut VrStereoConfig>,
);
impl<'a, T: 'a> Drop for RaylibVRMode<'a, '_, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndVrStereoMode() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibVRMode<'a, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Extension trait providing VR stereo mode entry on draw handles.
///
/// Implemented for every [`RaylibDraw`] surface so a VR stereo pass can begin inside any
/// active draw frame. The two methods are alternative forms of the same operation:
/// [`begin_vr_stereo_mode`](Self::begin_vr_stereo_mode) returns a RAII guard, while
/// [`draw_vr_stereo_mode`](Self::draw_vr_stereo_mode) takes a closure and ends the mode
/// automatically when the closure returns.
///
/// # Examples
///
/// Closure form (preferred when the scope is small enough to fit in one block):
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("vr-demo").build();
/// # let device = VrDeviceInfo {
/// #     h_resolution: 2160, v_resolution: 1200,
/// #     h_screen_size: 0.133793, v_screen_size: 0.0669,
/// #     eye_to_screen_distance: 0.041, lens_separation_distance: 0.07,
/// #     interpupillary_distance: 0.07,
/// #     lens_distortion_values: [1.0, 0.22, 0.24, 0.0],
/// #     chroma_ab_correction: [0.996, -0.004, 1.014, 0.0],
/// # };
/// let mut config = rl.load_vr_stereo_config(&thread, device);
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.draw_vr_stereo_mode(&mut config, |mut v| {
///         v.clear_background(Color::RAYWHITE);
///         v.draw_rectangle(10, 10, 100, 50, Color::RED);
///     });
/// }
/// ```
///
/// See [`RaylibVRMode`] for the equivalent guard-form example.
///
/// # See also
///
/// - [`RaylibVRMode`] — RAII guard returned by `begin_vr_stereo_mode`.
/// - [`VrStereoConfig`] — stereo projection / lens configuration consumed by both methods.
pub trait RaylibVRModeExt
where
    Self: Sized,
{
    /// Begin stereo rendering (requires VR simulator).
    /// Prefer using the closure version, [RaylibVRModeExt::draw_vr_stereo_mode] . This version returns a handle that calls [raylib_sys::EndVrStereoMode] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    fn begin_vr_stereo_mode<'a, 'b>(
        &'a mut self,
        _: &RaylibThread,
        vr_config: &'b mut VrStereoConfig,
    ) -> RaylibVRMode<'a, 'b, Self> {
        unsafe { ffi::BeginVrStereoMode(*vr_config.as_ref()) }
        RaylibVRMode(self, PhantomData, PhantomData)
    }

    /// Begin stereo rendering (requires VR simulator).
    fn draw_vr_stereo_mode<'a, 'b>(
        &'a mut self,
        vr_config: &'b mut VrStereoConfig,
        func: impl FnOnce(RaylibVRMode<'a, 'b, Self>),
    ) {
        unsafe { ffi::BeginVrStereoMode(*vr_config.as_ref()) }
        func(RaylibVRMode(self, PhantomData, PhantomData));
        // Uncomment the following if RaylibVRMode has been changed to no longer call EndTextureMode() in its drop implementation:
        // unsafe { ffi::EndVrStereoMode(); }
    }
}

impl<D: RaylibDraw> RaylibVRModeExt for D {}
impl<'a, T: 'a> RaylibDraw for RaylibVRMode<'a, '_, T> {}

// 2D Mode

/// RAII draw handle for 2D camera mode — calls `EndMode2D` on drop.
///
/// While alive, all 2D draw calls are transformed by the camera's `offset`, `target`,
/// `rotation`, and `zoom`. Drop runs `EndMode2D`, restoring the screen-space identity
/// transform — subsequent draws on the parent guard render in raw screen coordinates again.
/// Implements [`RaylibDraw`] via deref, so the full 2D drawing surface is available
/// through the camera.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "software_renderer")] {
/// use raylib::prelude::*;
/// use raylib::test_harness::*;
///
/// with_headless(64, 64, |rl, thread| {
///     let camera = Camera2D {
///         offset: Vector2::new(0.0, 0.0),
///         target: Vector2::new(10.0, 10.0), // shifts world origin
///         rotation: 0.0,
///         zoom: 1.0,
///     };
///     let img = render_frame(rl, thread, |d| {
///         d.clear_background(Color::WHITE);
///         let mut c = d.begin_mode2D(camera);
///         // World rect at (10, 10) → screen (0, 0) after the camera translation.
///         c.draw_rectangle(10, 10, 20, 20, Color::RED);
///     });
///     assert_pixel(&img, 5, 5, Color::RED, 0);
///     // Outside the translated rect → still WHITE.
///     assert_pixel(&img, 50, 50, Color::WHITE, 0);
/// });
/// # }
/// ```
///
/// # See also
///
/// - [`RaylibMode2DExt::begin_mode2D`] — constructor for this guard.
/// - [`Camera2D`](crate::core::camera::Camera2D) — the camera passed to `begin_mode2D`.
/// - [`RaylibMode3D`] — sibling guard for 3D camera projections.
pub struct RaylibMode2D<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibMode2D<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndMode2D() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibMode2D<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> std::ops::DerefMut for RaylibMode2D<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

/// Extension trait providing 2D camera mode entry on draw handles.
///
/// Implemented for every [`RaylibDraw`] surface (so 2D mode can begin inside texture-mode,
/// scissor-mode, blend-mode, etc.). The two methods are alternative forms:
/// [`begin_mode2D`](Self::begin_mode2D) returns a RAII guard;
/// [`draw_mode2D`](Self::draw_mode2D) takes a closure and ends 2D mode automatically when
/// the closure returns.
///
/// # Examples
///
/// Closure form:
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let camera = Camera2D {
///     offset: Vector2::new(400.0, 300.0),
///     target: Vector2::zero(),
///     rotation: 0.0,
///     zoom: 2.0,
/// };
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_mode2D(camera, |mut c| {
///         c.draw_rectangle(-10, -10, 20, 20, Color::RED);
///     });
/// }
/// ```
///
/// See [`RaylibMode2D`] for the equivalent guard-form example.
///
/// # See also
///
/// - [`RaylibMode2D`] — RAII guard returned by `begin_mode2D`.
/// - [`Camera2D`](crate::core::camera::Camera2D) — camera passed into both methods.
pub trait RaylibMode2DExt
where
    Self: Sized,
{
    /// Begin 2D mode with custom camera (2D).
    /// Prefer using the closure version, [RaylibMode2DExt::draw_mode2D]. This version returns a handle that calls [raylib_sys::EndMode2D] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[allow(non_snake_case)]
    #[inline]
    #[must_use]
    fn begin_mode2D(&mut self, camera: impl Into<ffi::Camera2D>) -> RaylibMode2D<'_, Self> {
        unsafe {
            ffi::BeginMode2D(camera.into());
        }
        RaylibMode2D(self)
    }

    /// Begin 2D mode with custom camera (2D).
    #[allow(non_snake_case)]
    fn draw_mode2D<'a>(
        &'a mut self,
        camera: impl Into<ffi::Camera2D>,
        func: impl FnOnce(RaylibMode2D<'a, Self>),
    ) {
        unsafe {
            ffi::BeginMode2D(camera.into());
        }
        func(RaylibMode2D(self));
        // Uncomment the following if RaylibMode2D has been changed to no longer call EndMode2D() in its drop implementation:
        // unsafe {
        //     ffi::EndMode2D();
        // }
    }
}

impl<D: RaylibDraw> RaylibMode2DExt for D {}
impl<'a, T: 'a> RaylibDraw for RaylibMode2D<'a, T> {}

// 3D Mode

/// RAII draw handle for 3D camera mode — calls `EndMode3D` on drop.
///
/// While alive, draw calls use the [`Camera3D`](crate::core::camera::Camera3D)'s view + projection matrices, so 3D draw
/// methods on [`RaylibDraw3D`] (`draw_cube`, `draw_sphere`, `draw_line_3D`, etc.) render in
/// world space. Drop runs `EndMode3D`, restoring the 2D screen-space transform. Implements
/// both [`RaylibDraw`] and [`RaylibDraw3D`] via deref.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let camera = Camera3D::perspective(
///     Vector3::new(4.0, 4.0, 4.0),
///     Vector3::new(0.0, 0.0, 0.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     45.0,
/// );
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     {
///         let mut c = d.begin_mode3D(camera);
///         c.draw_cube(Vector3::zero(), 2.0, 2.0, 2.0, Color::RED);
///         c.draw_grid(10, 1.0);
///         // Drop ends 3D mode automatically.
///     }
///     d.draw_text("hello 3D", 10, 10, 20, Color::BLACK);
/// }
/// ```
///
/// # See also
///
/// - [`RaylibMode3DExt::begin_mode3D`] — constructor for this guard.
/// - [`Camera3D`](crate::core::camera::Camera3D) — the camera passed to `begin_mode3D`.
/// - [`RaylibDraw3D`] — 3D-only draw methods available through this guard.
pub struct RaylibMode3D<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibMode3D<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndMode3D() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibMode3D<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> std::ops::DerefMut for RaylibMode3D<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

/// Extension trait providing 3D camera mode entry on draw handles.
///
/// Implemented for every [`RaylibDraw`] surface. The two methods are alternative forms:
/// [`begin_mode3D`](Self::begin_mode3D) returns a RAII guard;
/// [`draw_mode3D`](Self::draw_mode3D) takes a closure and ends 3D mode automatically when
/// the closure returns.
///
/// # Examples
///
/// Closure form:
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let camera = Camera3D::perspective(
///     Vector3::new(4.0, 4.0, 4.0),
///     Vector3::new(0.0, 0.0, 0.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     45.0,
/// );
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_mode3D(camera, |mut c| {
///         c.draw_cube(Vector3::zero(), 2.0, 2.0, 2.0, Color::RED);
///         c.draw_grid(10, 1.0);
///     });
/// }
/// ```
///
/// See [`RaylibMode3D`] for the equivalent guard-form example.
///
/// # See also
///
/// - [`RaylibMode3D`] — RAII guard returned by `begin_mode3D`.
/// - [`Camera3D`](crate::core::camera::Camera3D) — camera passed into both methods.
/// - [`RaylibDraw3D`] — 3D-only draw methods exposed by the guard.
pub trait RaylibMode3DExt
where
    Self: Sized,
{
    /// Begin 3D mode with custom camera (3D).
    /// Prefer using the closure version, [RaylibMode3DExt::draw_mode3D]. This version returns a handle that calls [raylib_sys::EndMode3D] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[allow(non_snake_case)]
    #[must_use]
    #[inline]
    fn begin_mode3D(&mut self, camera: impl Into<ffi::Camera3D>) -> RaylibMode3D<'_, Self> {
        unsafe {
            ffi::BeginMode3D(camera.into());
        }
        RaylibMode3D(self)
    }

    /// Begin 3D mode with custom camera (3D).
    #[allow(non_snake_case)]
    fn draw_mode3D<'a>(
        &'a mut self,
        camera: impl Into<ffi::Camera3D>,
        func: impl FnOnce(RaylibMode3D<'a, Self>),
    ) {
        unsafe {
            ffi::BeginMode3D(camera.into());
        }
        func(RaylibMode3D(self));
        // Uncomment the following if RaylibMode3D has been changed to no longer call EndMode3D() in its drop implementation:
        // unsafe {
        //     ffi::EndMode3D();
        // }
    }
}

impl<D: RaylibDraw> RaylibMode3DExt for D {}
impl<'a, T: 'a> RaylibDraw for RaylibMode3D<'a, T> {}
impl<'a, T: 'a> RaylibDraw3D for RaylibMode3D<'a, T> {}

// shader Mode

/// RAII draw handle for custom shader mode — calls `EndShaderMode` on drop.
///
/// While alive, the borrowed [`Shader`] is bound as the active fragment / vertex program for
/// subsequent draw calls. Drop runs `EndShaderMode`, restoring raylib's default shader.
/// Implements [`RaylibDraw`] and [`RaylibDraw3D`] via deref, so the guard accepts both 2D
/// and 3D draw calls — the shader applies to whichever the parent surface supports.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let mut shader = rl.load_shader(&thread, None, Some("grayscale.fs"));
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     {
///         let mut s = d.begin_shader_mode(&mut shader);
///         s.draw_rectangle(10, 10, 100, 50, Color::WHITE);
///         // Drop ends shader mode automatically.
///     }
/// }
/// ```
///
/// # See also
///
/// - [`RaylibShaderModeExt::begin_shader_mode`] — constructor for this guard.
/// - [`Shader`] — the shader the guard borrows.
pub struct RaylibShaderMode<'a, 'b, T: 'a>(&'a mut T, PhantomData<&'b mut Shader>);

impl<'a, T: 'a> Drop for RaylibShaderMode<'a, '_, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndShaderMode() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibShaderMode<'a, '_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> std::ops::DerefMut for RaylibShaderMode<'a, '_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

/// Extension trait providing custom shader mode entry on draw handles.
///
/// Implemented for every [`RaylibDraw`] surface. The two methods are alternative forms:
/// [`begin_shader_mode`](Self::begin_shader_mode) returns a RAII guard;
/// [`draw_shader_mode`](Self::draw_shader_mode) takes a closure and ends shader mode
/// automatically when the closure returns.
///
/// # Examples
///
/// Closure form:
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let mut shader = rl.load_shader(&thread, None, Some("grayscale.fs"));
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_shader_mode(&mut shader, |mut s| {
///         s.draw_rectangle(10, 10, 100, 50, Color::WHITE);
///     });
/// }
/// ```
///
/// See [`RaylibShaderMode`] for the equivalent guard-form example.
///
/// # See also
///
/// - [`RaylibShaderMode`] — RAII guard returned by `begin_shader_mode`.
/// - [`Shader`] — the shader bound for the duration of the mode.
pub trait RaylibShaderModeExt
where
    Self: Sized,
{
    /// Begin custom shader drawing.
    /// Prefer using the closure version, [RaylibShaderModeExt::draw_shader_mode]. This version returns a handle that calls [raylib_sys::EndShaderMode] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[must_use]
    #[inline]
    fn begin_shader_mode<'a, 'b>(
        &'a mut self,
        shader: &'b mut Shader,
    ) -> RaylibShaderMode<'a, 'b, Self> {
        unsafe { ffi::BeginShaderMode(*shader.as_ref()) }
        RaylibShaderMode(self, PhantomData)
    }

    /// Begin custom shader drawing.
    fn draw_shader_mode<'a, 'b>(
        &'a mut self,
        shader: &'b mut Shader,
        func: impl FnOnce(RaylibShaderMode<'a, 'b, Self>),
    ) {
        unsafe { ffi::BeginShaderMode(*shader.as_ref()) }
        func(RaylibShaderMode(self, PhantomData));
        // Uncomment the following if RaylibShaderMode has been changed to no longer call EndShaderMode() in its drop implementation:
        // unsafe { ffi::EndShaderMode(); }
    }
}

impl<D: RaylibDraw> RaylibShaderModeExt for D {}
impl<'a, T: 'a> RaylibDraw for RaylibShaderMode<'a, '_, T> {}
impl<'a, T: 'a> RaylibDraw3D for RaylibShaderMode<'a, '_, T> {}

// Blend Mode

/// RAII draw handle for a blend mode scope — calls `EndBlendMode` on drop.
///
/// While alive, the active GPU blend equation is the one specified in the constructor
/// (`BLEND_ALPHA`, `BLEND_ADDITIVE`, `BLEND_MULTIPLIED`, `BLEND_ADD_COLORS`,
/// `BLEND_SUBTRACT_COLORS`, `BLEND_ALPHA_PREMULTIPLY`, or `BLEND_CUSTOM`). Drop runs
/// `EndBlendMode`, restoring the previous blend mode. Implements [`RaylibDraw`] and
/// [`RaylibDraw3D`] via deref.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "software_renderer")] {
/// use raylib::prelude::*;
/// use raylib::test_harness::*;
///
/// with_headless(64, 64, |rl, thread| {
///     let img = render_frame(rl, thread, |d| {
///         d.clear_background(Color::WHITE);
///         // Default alpha blend: opaque RED writes are unaffected by the underlying WHITE.
///         let mut b = d.begin_blend_mode(BlendMode::BLEND_ALPHA);
///         b.draw_rectangle(0, 0, 32, 32, Color::RED);
///     });
///     assert_pixel(&img, 5, 5, Color::RED, 0);
///     assert_pixel(&img, 50, 50, Color::WHITE, 0);
/// });
/// # }
/// ```
///
/// # See also
///
/// - [`RaylibBlendModeExt::begin_blend_mode`] — constructor for this guard.
/// - [`BlendMode`](crate::consts::BlendMode) — supported blend equations.
pub struct RaylibBlendMode<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibBlendMode<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndBlendMode() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibBlendMode<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> std::ops::DerefMut for RaylibBlendMode<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

/// Extension trait providing blend mode entry on draw handles.
///
/// Implemented for every [`RaylibDraw`] surface. The two methods are alternative forms:
/// [`begin_blend_mode`](Self::begin_blend_mode) returns a RAII guard;
/// [`draw_blend_mode`](Self::draw_blend_mode) takes a closure and ends the blend scope
/// automatically when the closure returns.
///
/// # Examples
///
/// Closure form:
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::BLACK);
///     d.draw_blend_mode(BlendMode::BLEND_ADDITIVE, |mut b| {
///         // Additive blending: subsequent draws add to the framebuffer.
///         b.draw_circle(200, 200, 80.0, Color::new(64, 0, 0, 255));
///         b.draw_circle(240, 200, 80.0, Color::new(0, 64, 0, 255));
///     });
/// }
/// ```
///
/// See [`RaylibBlendMode`] for the equivalent guard-form example.
///
/// # See also
///
/// - [`RaylibBlendMode`] — RAII guard returned by `begin_blend_mode`.
/// - [`BlendMode`](crate::consts::BlendMode) — supported blend equations.
pub trait RaylibBlendModeExt
where
    Self: Sized,
{
    /// Begin blending mode (alpha, additive, multiplied, subtract, custom).
    /// Prefer using the closure version, [RaylibBlendModeExt::draw_blend_mode]. This version returns a handle that calls [raylib_sys::EndBlendMode] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    fn begin_blend_mode(
        &mut self,
        blend_mode: crate::consts::BlendMode,
    ) -> RaylibBlendMode<'_, Self> {
        unsafe { ffi::BeginBlendMode((blend_mode as u32) as i32) }
        RaylibBlendMode(self)
    }

    /// Begin blending mode (alpha, additive, multiplied, subtract, custom).
    fn draw_blend_mode<'a>(
        &'a mut self,
        blend_mode: crate::consts::BlendMode,
        func: impl FnOnce(RaylibBlendMode<'a, Self>),
    ) {
        unsafe { ffi::BeginBlendMode((blend_mode as u32) as i32) }
        func(RaylibBlendMode(self));
        // Uncomment the following if RaylibBlendMode has been changed to no longer call EndBlendMode() in its drop implementation:
        // unsafe { ffi::EndBlendMode(); }
    }
}

impl<D: RaylibDraw> RaylibBlendModeExt for D {}
impl<'a, T: 'a> RaylibDraw for RaylibBlendMode<'a, T> {}
impl<'a, T: 'a> RaylibDraw3D for RaylibBlendMode<'a, T> {}

// Scissor Mode stuff

/// RAII draw handle for a scissor (clipping rectangle) mode — calls `EndScissorMode` on drop.
///
/// While alive, the GPU discards fragments outside the rectangle `(x, y, width, height)`
/// passed to the constructor — draws are clipped to that region. Drop runs `EndScissorMode`,
/// removing the scissor test so subsequent draws cover the full framebuffer again.
/// Implements [`RaylibDraw`] (and [`RaylibDraw3D`] when the parent surface does) via deref.
///
/// # Examples
///
/// ```rust
/// # #[cfg(feature = "software_renderer")] {
/// use raylib::prelude::*;
/// use raylib::test_harness::*;
///
/// with_headless(64, 64, |rl, thread| {
///     let img = render_frame(rl, thread, |d| {
///         d.clear_background(Color::WHITE);
///         {
///             // Clip subsequent draws to a 20×20 region at (10, 10).
///             let mut s = d.begin_scissor_mode(10, 10, 20, 20);
///             // Even though the rectangle is 64×64, only the clipped 20×20 paints RED.
///             s.draw_rectangle(0, 0, 64, 64, Color::RED);
///         }
///     });
///     // Inside the scissor region: RED.
///     assert_pixel(&img, 15, 15, Color::RED, 0);
///     // Outside the scissor region: untouched WHITE.
///     assert_pixel(&img, 5, 5, Color::WHITE, 0);
///     assert_pixel(&img, 40, 40, Color::WHITE, 0);
/// });
/// # }
/// ```
///
/// # See also
///
/// - [`RaylibScissorModeExt::begin_scissor_mode`] — constructor for this guard.
pub struct RaylibScissorMode<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibScissorMode<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndScissorMode() }
    }
}
impl<'a, T: 'a> std::ops::Deref for RaylibScissorMode<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> std::ops::DerefMut for RaylibScissorMode<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

/// Extension trait providing scissor mode entry on draw handles.
///
/// Implemented for every [`RaylibDraw`] surface. The two methods are alternative forms:
/// [`begin_scissor_mode`](Self::begin_scissor_mode) returns a RAII guard;
/// [`draw_scissor_mode`](Self::draw_scissor_mode) takes a closure and ends the scissor scope
/// automatically when the closure returns.
///
/// # Examples
///
/// Closure form:
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_scissor_mode(100, 100, 200, 150, |mut s| {
///         // Only fragments inside (100, 100, 200, 150) survive the scissor test.
///         s.draw_rectangle(0, 0, 800, 600, Color::RED);
///     });
/// }
/// ```
///
/// See [`RaylibScissorMode`] for the equivalent guard-form example.
///
/// # See also
///
/// - [`RaylibScissorMode`] — RAII guard returned by `begin_scissor_mode`.
pub trait RaylibScissorModeExt
where
    Self: Sized,
{
    /// Begin scissor mode (define screen area for following drawing).
    /// Prefer using the closure version, [RaylibScissorModeExt::draw_scissor_mode]. This version returns a handle that calls [raylib_sys::EndScissorMode] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[must_use]
    #[inline]
    fn begin_scissor_mode(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> RaylibScissorMode<'_, Self> {
        unsafe { ffi::BeginScissorMode(x, y, width, height) }
        RaylibScissorMode(self)
    }

    /// Begin scissor mode (define screen area for following drawing).
    fn draw_scissor_mode<'a>(
        &'a mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        func: impl FnOnce(RaylibScissorMode<'a, Self>),
    ) {
        unsafe { ffi::BeginScissorMode(x, y, width, height) }
        func(RaylibScissorMode(self));
        // Uncomment the following if RaylibScissorMode has been changed to no longer call EndScissorMode() in its drop implementation:
        // unsafe { ffi::EndScissorMode(); }
    }
}

impl<D: RaylibDraw> RaylibScissorModeExt for D {}
impl<'a, T: 'a> RaylibDraw for RaylibScissorMode<'a, T> {}
impl<'a, T: 'a + RaylibDraw3D> RaylibDraw3D for RaylibScissorMode<'a, T> {}

// Actual drawing functions

/// Drawing methods available within an active drawing or mode block (2D shapes, textures, text).
///
/// This trait is implemented on [`RaylibDrawHandle`] (the guard returned by
/// [`RaylibHandle::begin_drawing`]) and on every nested mode guard
/// ([`RaylibTextureMode`], [`RaylibMode2D`], [`RaylibMode3D`], etc.). Call methods on the guard
/// to issue GPU draw commands; all commands accumulate until the guard drops, at which point
/// `EndDrawing` is called and the frame is presented.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(640, 480).title("demo").build();
///
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     d.draw_rectangle(10, 10, 100, 50, Color::RED);
///     d.draw_text("RaylibDraw", 10, 70, 20, Color::BLACK);
///     // `d` drops here → EndDrawing is called automatically
/// }
/// ```
pub trait RaylibDraw {
    /// Sets background color (framebuffer clear color.into()).
    #[inline]
    fn clear_background(&mut self, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::ClearBackground(color.into());
        }
    }

    /// Get texture that is used for shapes drawing
    #[inline]
    #[must_use]
    fn get_shapes_texture(&self) -> Texture2D {
        Texture2D(unsafe { ffi::GetShapesTexture() })
    }

    /// Get texture source rectangle that is used for shapes drawing
    #[inline]
    #[must_use]
    fn get_shapes_texture_rectangle(&self) -> Rectangle {
        unsafe { ffi::GetShapesTextureRectangle() }
    }

    /// Define default texture used to draw shapes
    #[inline]
    fn set_shapes_texture(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        source: impl Into<ffi::Rectangle>,
    ) {
        unsafe { ffi::SetShapesTexture(*texture.as_ref(), source.into()) }
    }

    // // Draw gui widget
    // fn draw_gui<G: crate::rgui::GuiDraw>(&mut self, widget: G) -> crate::rgui::DrawResult {
    //     widget.draw()
    // }

    // SHAPES
    /// Draws a pixel.
    #[inline]
    fn draw_pixel(&mut self, x: i32, y: i32, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawPixel(x, y, color.into());
        }
    }

    /// Draws a pixel (Vector version).
    #[inline]
    fn draw_pixel_v(&mut self, position: impl Into<Vector2>, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawPixelV(position.into(), color.into());
        }
    }

    /// Draws a line.
    #[inline]
    fn draw_line(
        &mut self,
        start_pos_x: i32,
        start_pos_y: i32,
        end_pos_x: i32,
        end_pos_y: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawLine(start_pos_x, start_pos_y, end_pos_x, end_pos_y, color.into());
        }
    }

    /// Draws a line (Vector version).
    #[inline]
    fn draw_line_v(
        &mut self,
        start_pos: impl Into<Vector2>,
        end_pos: impl Into<Vector2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawLineV(start_pos.into(), end_pos.into(), color.into());
        }
    }

    /// Draws a line with thickness.
    #[inline]
    fn draw_line_ex(
        &mut self,
        start_pos: impl Into<Vector2>,
        end_pos: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawLineEx(start_pos.into(), end_pos.into(), thick, color.into());
        }
    }

    /// Draws a line using cubic-bezier curves in-out.
    #[inline]
    fn draw_line_bezier(
        &mut self,
        start_pos: impl Into<Vector2>,
        end_pos: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawLineBezier(start_pos.into(), end_pos.into(), thick, color.into());
        }
    }

    /// Draw a dashed line.
    ///
    /// `dash_size` is the length of each painted segment; `space_size` is the
    /// gap between segments. Both are in pixels.
    #[inline]
    fn draw_line_dashed(
        &mut self,
        start: impl Into<Vector2>,
        end: impl Into<Vector2>,
        dash_size: i32,
        space_size: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawLineDashed(
                start.into(),
                end.into(),
                dash_size,
                space_size,
                color.into(),
            );
        }
    }

    /// Draw lines sequence
    #[inline]
    fn draw_line_strip(&mut self, points: &[Vector2], color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawLineStrip(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                color.into(),
            );
        }
    }

    /// Draws a color-filled circle.
    #[inline]
    fn draw_circle(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircle(center_x, center_y, radius, color.into());
        }
    }
    /// Draw a piece of a circle
    #[inline]
    fn draw_circle_sector(
        &mut self,
        center: impl Into<Vector2>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        segments: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircleSector(
                center.into(),
                radius,
                start_angle,
                end_angle,
                segments,
                color.into(),
            );
        }
    }

    /// Draw circle sector outline
    #[inline]
    fn draw_circle_sector_lines(
        &mut self,
        center: impl Into<Vector2>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        segments: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircleSectorLines(
                center.into(),
                radius,
                start_angle,
                end_angle,
                segments,
                color.into(),
            );
        }
    }

    /// Draws a gradient-filled circle.
    #[inline]
    fn draw_circle_gradient(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: f32,
        inner: impl Into<ffi::Color>,
        outer: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircleGradient(
                Vector2 {
                    x: center_x as f32,
                    y: center_y as f32,
                },
                radius,
                inner.into(),
                outer.into(),
            );
        }
    }

    /// Draws a color-filled circle (Vector version).
    #[inline]
    fn draw_circle_v(
        &mut self,
        center: impl Into<Vector2>,
        radius: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircleV(center.into(), radius, color.into());
        }
    }

    /// Draws circle outline.
    #[inline]
    fn draw_circle_lines(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircleLines(center_x, center_y, radius, color.into());
        }
    }

    /// Draws circle outline. (Vector Version)
    #[inline]
    fn draw_circle_lines_v(
        &mut self,
        center: impl Into<Vector2>,
        radius: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircleLinesV(center.into(), radius, color.into());
        }
    }

    /// Draws ellipse.
    #[inline]
    fn draw_ellipse(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius_h: f32,
        radius_v: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawEllipse(center_x, center_y, radius_h, radius_v, color.into());
        }
    }

    /// Draws ellipse using a Vector2 center.
    #[inline]
    fn draw_ellipse_v(
        &mut self,
        center: impl Into<Vector2>,
        radius_h: f32,
        radius_v: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawEllipseV(center.into(), radius_h, radius_v, color.into());
        }
    }

    /// Draws ellipse.
    #[inline]
    fn draw_ellipse_lines(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius_h: f32,
        radius_v: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawEllipseLines(center_x, center_y, radius_h, radius_v, color.into());
        }
    }

    /// Draws ellipse outline using a Vector2 center.
    #[inline]
    fn draw_ellipse_lines_v(
        &mut self,
        center: impl Into<Vector2>,
        radius_h: f32,
        radius_v: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawEllipseLinesV(center.into(), radius_h, radius_v, color.into());
        }
    }

    /// Draw ring
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn draw_ring(
        &mut self,
        center: impl Into<Vector2>,
        inner_radius: f32,
        outer_radius: f32,
        start_angle: f32,
        end_angle: f32,
        segments: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRing(
                center.into(),
                inner_radius,
                outer_radius,
                start_angle,
                end_angle,
                segments,
                color.into(),
            );
        }
    }

    /// Draw ring lines
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn draw_ring_lines(
        &mut self,
        center: impl Into<Vector2>,
        inner_radius: f32,
        outer_radius: f32,
        start_angle: f32,
        end_angle: f32,
        segments: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRingLines(
                center.into(),
                inner_radius,
                outer_radius,
                start_angle,
                end_angle,
                segments,
                color.into(),
            );
        }
    }

    /// Draws a color-filled rectangle.
    #[inline]
    fn draw_rectangle(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangle(x, y, width, height, color.into());
        }
    }

    /// Draws a color-filled rectangle (Vector version).
    #[inline]
    fn draw_rectangle_v(
        &mut self,
        position: impl Into<Vector2>,
        size: impl Into<Vector2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleV(position.into(), size.into(), color.into());
        }
    }

    /// Draws a color-filled rectangle from `rec`.
    #[inline]
    fn draw_rectangle_rec(&mut self, rec: impl Into<ffi::Rectangle>, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawRectangleRec(rec.into(), color.into());
        }
    }

    /// Draws a color-filled rectangle with pro parameters.
    #[inline]
    fn draw_rectangle_pro(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        origin: impl Into<Vector2>,
        rotation: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectanglePro(rec.into(), origin.into(), rotation, color.into());
        }
    }

    /// Draws a vertical-gradient-filled rectangle.
    ///
    /// **NOTE**: Gradient goes from bottom (`color1`) to top (`color2`).
    #[inline]
    fn draw_rectangle_gradient_v(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color1: impl Into<ffi::Color>,
        color2: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleGradientV(x, y, width, height, color1.into(), color2.into());
        }
    }

    /// Draws a horizontal-gradient-filled rectangle.
    ///
    /// **NOTE**: Gradient goes from bottom (`color1`) to top (`color2`).
    #[inline]
    fn draw_rectangle_gradient_h(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color1: impl Into<ffi::Color>,
        color2: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleGradientH(x, y, width, height, color1.into(), color2.into());
        }
    }

    /// Draws a gradient-filled rectangle with custom vertex colors.
    ///
    /// **NOTE**: Colors refer to corners, starting at top-left corner and going counter-clockwise.
    #[inline]
    fn draw_rectangle_gradient_ex(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        col1: impl Into<ffi::Color>,
        col2: impl Into<ffi::Color>,
        col3: impl Into<ffi::Color>,
        col4: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleGradientEx(
                rec.into(),
                col1.into(),
                col2.into(),
                col3.into(),
                col4.into(),
            );
        }
    }

    /// Draws rectangle outline.
    #[inline]
    fn draw_rectangle_lines(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleLines(x, y, width, height, color.into());
        }
    }

    /// Draws rectangle outline with extended parameters.
    #[inline]
    fn draw_rectangle_lines_ex(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        line_thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleLinesEx(rec.into(), line_thick, color.into());
        }
    }
    /// Draws rectangle with rounded edges.
    #[inline]
    fn draw_rectangle_rounded(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        roundness: f32,
        segments: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleRounded(rec.into(), roundness, segments, color.into());
        }
    }

    /// Draws rectangle outline with rounded edges included.
    #[inline]
    fn draw_rectangle_rounded_lines(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        roundness: f32,
        segments: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleRoundedLines(rec.into(), roundness, segments, color.into());
        }
    }

    /// Draw rectangle with rounded edges outline
    #[inline]
    fn draw_rectangle_rounded_lines_ex(
        &mut self,
        rec: impl Into<ffi::Rectangle>,
        roundness: f32,
        segments: i32,
        line_thickness: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawRectangleRoundedLinesEx(
                rec.into(),
                roundness,
                segments,
                line_thickness,
                color.into(),
            )
        };
    }
    /// Draws a triangle.
    #[inline]
    fn draw_triangle(
        &mut self,
        v1: impl Into<Vector2>,
        v2: impl Into<Vector2>,
        v3: impl Into<Vector2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTriangle(v1.into(), v2.into(), v3.into(), color.into());
        }
    }

    /// Draws a triangle using lines.
    #[inline]
    fn draw_triangle_lines(
        &mut self,
        v1: impl Into<Vector2>,
        v2: impl Into<Vector2>,
        v3: impl Into<Vector2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTriangleLines(v1.into(), v2.into(), v3.into(), color.into());
        }
    }

    /// Draw a triangle fan defined by points.
    #[inline]
    fn draw_triangle_fan(&mut self, points: &[Vector2], color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawTriangleFan(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                color.into(),
            );
        }
    }

    /// Draw a triangle strip defined by points
    #[inline]
    fn draw_triangle_strip(&mut self, points: &[Vector2], color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawTriangleStrip(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                color.into(),
            );
        }
    }

    /// Draws a regular polygon of n sides (Vector version).
    #[inline]
    fn draw_poly(
        &mut self,
        center: impl Into<Vector2>,
        sides: i32,
        radius: f32,
        rotation: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawPoly(center.into(), sides, radius, rotation, color.into());
        }
    }

    /// Draws a regular polygon of n sides (Vector version).
    #[inline]
    fn draw_poly_lines(
        &mut self,
        center: impl Into<Vector2>,
        sides: i32,
        radius: f32,
        rotation: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawPolyLines(center.into(), sides, radius, rotation, color.into());
        }
    }

    /// Draws a `texture` using specified position and `tint` color.
    #[inline]
    fn draw_texture(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        x: i32,
        y: i32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTexture(*texture.as_ref(), x, y, tint.into());
        }
    }

    /// Draws a `texture` using specified `position` vector and `tint` color.
    #[inline]
    fn draw_texture_v(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        position: impl Into<Vector2>,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTextureV(*texture.as_ref(), position.into(), tint.into());
        }
    }

    /// Draws a `texture` with extended parameters.
    #[inline]
    fn draw_texture_ex(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        position: impl Into<Vector2>,
        rotation: f32,
        scale: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTextureEx(
                *texture.as_ref(),
                position.into(),
                rotation,
                scale,
                tint.into(),
            );
        }
    }

    /// Draws from a region of `texture` defined by the `source_rec` rectangle.
    #[inline]
    fn draw_texture_rec(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        source_rec: impl Into<ffi::Rectangle>,
        position: impl Into<Vector2>,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTextureRec(
                *texture.as_ref(),
                source_rec.into(),
                position.into(),
                tint.into(),
            );
        }
    }

    /// Draw from a region of `texture` defined by the `source_rec` rectangle with pro parameters.
    #[inline]
    fn draw_texture_pro(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        source_rec: impl Into<ffi::Rectangle>,
        dest_rec: impl Into<ffi::Rectangle>,
        origin: impl Into<Vector2>,
        rotation: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTexturePro(
                *texture.as_ref(),
                source_rec.into(),
                dest_rec.into(),
                origin.into(),
                rotation,
                tint.into(),
            );
        }
    }

    /// Draws a texture (or part of it) that stretches or shrinks nicely
    #[inline]
    fn draw_texture_n_patch(
        &mut self,
        texture: impl AsRef<ffi::Texture2D>,
        n_patch_info: impl Into<ffi::NPatchInfo>,
        dest_rec: impl Into<ffi::Rectangle>,
        origin: impl Into<Vector2>,
        rotation: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTextureNPatch(
                *texture.as_ref(),
                n_patch_info.into(),
                dest_rec.into(),
                origin.into(),
                rotation,
                tint.into(),
            );
        }
    }

    /// Shows current FPS.
    #[inline]
    fn draw_fps(&mut self, x: i32, y: i32) {
        unsafe {
            ffi::DrawFPS(x, y);
        }
    }

    /// Draws text (using default font).
    /// This does not support UTF-8. Use `[RaylibDrawHandle::draw_text_codepoints]` for that.
    #[inline]
    fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        font_size: i32,
        color: impl Into<ffi::Color>,
    ) {
        let c_text = CString::new(text).unwrap();

        unsafe {
            ffi::DrawText(c_text.as_ptr(), x, y, font_size, color.into());
        }
    }

    /// Draws text (using default font) with support for UTF-8.
    /// If you do not need UTF-8, use `[RaylibDrawHandle::draw_text]`.
    fn draw_text_codepoints(
        &mut self,
        font: impl AsRef<ffi::Font>,
        text: &str,
        position: impl Into<Vector2>,
        font_size: f32,
        spacing: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            let c_text = CString::new(text).unwrap();

            let mut len = 0;
            let u = ffi::LoadCodepoints(c_text.as_ptr(), &mut len);

            ffi::DrawTextCodepoints(
                *font.as_ref(),
                u,
                text.len() as i32,
                position.into(),
                font_size,
                spacing,
                tint.into(),
            )
        }
    }
    /// Draws text using `font` and additional parameters.
    #[inline]
    fn draw_text_ex(
        &mut self,
        font: impl AsRef<ffi::Font>,
        text: &str,
        position: impl Into<Vector2>,
        font_size: f32,
        spacing: f32,
        tint: impl Into<ffi::Color>,
    ) {
        let c_text = CString::new(text).unwrap();
        unsafe {
            ffi::DrawTextEx(
                *font.as_ref(),
                c_text.as_ptr(),
                position.into(),
                font_size,
                spacing,
                tint.into(),
            );
        }
    }

    /// Draw text using Font and pro parameters (rotation)
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn draw_text_pro(
        &mut self,
        font: impl AsRef<ffi::Font>,
        text: &str,
        position: impl Into<Vector2>,
        origin: impl Into<Vector2>,
        rotation: f32,
        font_size: f32,
        spacing: f32,
        tint: impl Into<ffi::Color>,
    ) {
        let c_text = CString::new(text).unwrap();
        unsafe {
            ffi::DrawTextPro(
                *font.as_ref(),
                c_text.as_ptr(),
                position.into(),
                origin.into(),
                rotation,
                font_size,
                spacing,
                tint.into(),
            );
        }
    }

    /// Draw one character (codepoint)
    #[inline]
    fn draw_text_codepoint(
        &mut self,
        font: impl AsRef<ffi::Font>,
        codepoint: i32,
        position: impl Into<Vector2>,
        scale: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTextCodepoint(
                *font.as_ref(),
                codepoint,
                position.into(),
                scale,
                tint.into(),
            );
        }
    }

    /// Enable waiting for events when the handle is dropped, no automatic event polling
    #[inline]
    fn enable_event_waiting(&self) {
        unsafe { ffi::EnableEventWaiting() }
    }

    /// Disable waiting for events when the handle is dropped, no automatic event polling
    #[inline]
    fn disable_event_waiting(&self) {
        unsafe { ffi::DisableEventWaiting() }
    }

    /// Draw a polygon outline of n sides with extended parameters
    #[inline]
    fn draw_poly_lines_ex(
        &mut self,
        center: impl Into<Vector2>,
        sides: i32,
        radius: f32,
        rotation: f32,
        line_thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawPolyLinesEx(
                center.into(),
                sides,
                radius,
                rotation,
                line_thick,
                color.into(),
            );
        }
    }
    /// Draw spline: Linear, minimum 2 points
    #[inline]
    fn draw_spline_linear(&mut self, points: &[Vector2], thick: f32, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawSplineLinear(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                thick,
                color.into(),
            )
        }
    }
    /// Draw spline: B-Spline, minimum 4 points
    #[inline]
    fn draw_spline_basis(&mut self, points: &[Vector2], thick: f32, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawSplineBasis(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                thick,
                color.into(),
            )
        }
    }
    /// Draw spline: Catmull-Rom, minimum 4 points
    #[inline]
    fn draw_spline_catmull_rom(
        &mut self,
        points: &[Vector2],
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineCatmullRom(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                thick,
                color.into(),
            )
        }
    }

    /// Draw spline: Quadratic Bezier, minimum 3 points (1 control point): [p1, c2, p3, c4...]
    #[inline]
    fn draw_spline_bezier_quadratic(
        &mut self,
        points: &[Vector2],
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineBezierQuadratic(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                thick,
                color.into(),
            )
        }
    }

    /// Draw spline: Cubic Bezier, minimum 4 points (2 control points): [p1, c2, c3, p4, c5, c6...]
    #[inline]
    fn draw_spline_bezier_cubic(
        &mut self,
        points: &[Vector2],
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineBezierCubic(
                points.as_ptr() as *mut ffi::Vector2,
                points.len() as i32,
                thick,
                color.into(),
            )
        }
    }

    /// Draw spline segment: Linear, 2 points
    #[inline]
    fn draw_spline_segment_linear(
        &mut self,
        p1: impl Into<Vector2>,
        p2: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe { ffi::DrawSplineSegmentLinear(p1.into(), p2.into(), thick, color.into()) }
    }

    /// Draw spline segment: B-Spline, 4 points
    #[inline]
    fn draw_spline_segment_basis(
        &mut self,
        p1: impl Into<Vector2>,
        p2: impl Into<Vector2>,
        p3: impl Into<Vector2>,
        p4: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineSegmentBasis(
                p1.into(),
                p2.into(),
                p3.into(),
                p4.into(),
                thick,
                color.into(),
            )
        }
    }

    /// Draw spline segment: Catmull-Rom, 4 points
    #[inline]
    fn draw_spline_segment_catmull_rom(
        &mut self,
        p1: impl Into<Vector2>,
        p2: impl Into<Vector2>,
        p3: impl Into<Vector2>,
        p4: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineSegmentCatmullRom(
                p1.into(),
                p2.into(),
                p3.into(),
                p4.into(),
                thick,
                color.into(),
            )
        }
    }

    /// Draw spline segment: Quadratic Bezier, 2 points, 1 control point
    #[inline]
    fn draw_spline_segment_bezier_quadratic(
        &mut self,
        p1: impl Into<Vector2>,
        c2: impl Into<Vector2>,
        p3: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineSegmentBezierQuadratic(
                p1.into(),
                c2.into(),
                p3.into(),
                thick,
                color.into(),
            )
        }
    }

    /// Draw spline segment: Cubic Bezier, 2 points, 2 control points
    #[inline]
    fn draw_spline_segment_bezier_cubic(
        &mut self,
        p1: impl Into<Vector2>,
        c2: impl Into<Vector2>,
        c3: impl Into<Vector2>,
        p4: impl Into<Vector2>,
        thick: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSplineSegmentBezierCubic(
                p1.into(),
                c2.into(),
                c3.into(),
                p4.into(),
                thick,
                color.into(),
            )
        }
    }

    /// Get (evaluate) spline point: Linear
    #[inline]
    #[must_use]
    fn get_spline_point_linear(
        &mut self,
        start_pos: impl Into<Vector2>,
        end_pos: impl Into<Vector2>,
        t: f32,
    ) -> Vector2 {
        unsafe { ffi::GetSplinePointLinear(start_pos.into(), end_pos.into(), t) }
    }

    /// Get (evaluate) spline point: B-Spline
    #[inline]
    #[must_use]
    fn get_spline_point_basis(
        &mut self,
        p1: impl Into<Vector2>,
        p2: impl Into<Vector2>,
        p3: impl Into<Vector2>,
        p4: impl Into<Vector2>,
        t: f32,
    ) -> Vector2 {
        unsafe { ffi::GetSplinePointBasis(p1.into(), p2.into(), p3.into(), p4.into(), t) }
    }

    /// Get (evaluate) spline point: Catmull-Rom
    #[inline]
    #[must_use]
    fn get_spline_point_catmull_rom(
        &mut self,
        p1: impl Into<Vector2>,
        p2: impl Into<Vector2>,
        p3: impl Into<Vector2>,
        p4: impl Into<Vector2>,
        t: f32,
    ) -> Vector2 {
        unsafe { ffi::GetSplinePointCatmullRom(p1.into(), p2.into(), p3.into(), p4.into(), t) }
    }

    /// Get (evaluate) spline point: Quadratic Bezier
    #[inline]
    #[must_use]
    fn get_spline_point_bezier_quad(
        &mut self,
        p1: impl Into<Vector2>,
        c2: impl Into<Vector2>,
        p3: impl Into<Vector2>,
        t: f32,
    ) -> Vector2 {
        unsafe { ffi::GetSplinePointBezierQuad(p1.into(), c2.into(), p3.into(), t) }
    }

    /// Get (evaluate) spline point: Cubic Bezier
    #[inline]
    #[must_use]
    fn get_spline_point_bezier_cubic(
        &mut self,
        p1: impl Into<Vector2>,
        c2: impl Into<Vector2>,
        c3: impl Into<Vector2>,
        p4: impl Into<Vector2>,
        t: f32,
    ) -> Vector2 {
        unsafe { ffi::GetSplinePointBezierCubic(p1.into(), c2.into(), c3.into(), p4.into(), t) }
    }
}

/// Drawing methods available within an active 3D camera mode block.
///
/// Implemented on guards entered through a 3D camera ([`RaylibMode3D`]) and on nested mode
/// guards that wrap a 3D-capable parent ([`RaylibShaderMode`], [`RaylibBlendMode`],
/// [`RaylibScissorMode`]). The methods on this trait submit 3D primitives — points, lines,
/// triangles, cubes, spheres, cylinders, meshes — into the camera's projected world space.
/// 2D draw methods from [`RaylibDraw`] remain callable through the guard's deref; they are
/// projected onto the near plane and so usually render as overlays.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(800, 600).title("demo").build();
/// let camera = Camera3D::perspective(
///     Vector3::new(4.0, 4.0, 4.0),
///     Vector3::new(0.0, 0.0, 0.0),
///     Vector3::new(0.0, 1.0, 0.0),
///     45.0,
/// );
/// while !rl.window_should_close() {
///     let mut d = rl.begin_drawing(&thread);
///     d.clear_background(Color::RAYWHITE);
///     let mut c = d.begin_mode3D(camera);
///     c.draw_cube(Vector3::zero(), 2.0, 2.0, 2.0, Color::RED);
///     c.draw_cube_wires(Vector3::zero(), 2.0, 2.0, 2.0, Color::MAROON);
///     c.draw_grid(10, 1.0);
/// }
/// ```
///
/// # See also
///
/// - [`RaylibMode3D`] — primary guard through which this trait is exposed.
/// - [`RaylibDraw`] — 2D drawing methods available on the same guard.
pub trait RaylibDraw3D {
    /// Draw a point in 3D space, actually a small line
    #[allow(non_snake_case)]
    #[inline]
    fn draw_point3D(&mut self, position: impl Into<Vector3>, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawPoint3D(position.into(), color.into());
        }
    }

    /// Draw a color-filled triangle (vertex in counter-clockwise order!)
    #[allow(non_snake_case)]
    #[inline]
    fn draw_triangle3D(
        &mut self,
        v1: impl Into<Vector3>,
        v2: impl Into<Vector3>,
        v3: impl Into<Vector3>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawTriangle3D(v1.into(), v2.into(), v3.into(), color.into());
        }
    }

    /// Draw a triangle strip defined by points
    #[allow(non_snake_case)]
    #[inline]
    fn draw_triangle_strip3D(&mut self, points: &[Vector3], color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawTriangleStrip3D(points.as_ptr() as *mut _, points.len() as i32, color.into());
        }
    }

    /// Draws a line in 3D world space.
    #[allow(non_snake_case)]
    #[inline]
    fn draw_line3D(
        &mut self,
        start_pos: impl Into<Vector3>,
        end_pos: impl Into<Vector3>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawLine3D(start_pos.into(), end_pos.into(), color.into());
        }
    }

    /// Draws a circle in 3D world space.
    #[allow(non_snake_case)]
    #[inline]
    fn draw_circle3D(
        &mut self,
        center: impl Into<Vector3>,
        radius: f32,
        rotation_axis: impl Into<Vector3>,
        rotation_angle: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCircle3D(
                center.into(),
                radius,
                rotation_axis.into(),
                rotation_angle,
                color.into(),
            );
        }
    }

    /// Draws a cube.
    #[inline]
    fn draw_cube(
        &mut self,
        position: impl Into<Vector3>,
        width: f32,
        height: f32,
        length: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCube(position.into(), width, height, length, color.into());
        }
    }

    /// Draws a cube (Vector version).
    #[inline]
    fn draw_cube_v(
        &mut self,
        position: impl Into<Vector3>,
        size: impl Into<Vector3>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCubeV(position.into(), size.into(), color.into());
        }
    }

    /// Draws a cube in wireframe.
    #[inline]
    fn draw_cube_wires(
        &mut self,
        position: impl Into<Vector3>,
        width: f32,
        height: f32,
        length: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCubeWires(position.into(), width, height, length, color.into());
        }
    }

    /// Draws a cube in wireframe. (Vector Version)
    #[inline]
    fn draw_cube_wires_v(
        &mut self,
        position: impl Into<Vector3>,
        size: impl Into<Vector3>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCubeWiresV(position.into(), size.into(), color.into());
        }
    }

    /// Draw a 3d mesh with material and transform
    #[inline]
    fn draw_mesh(
        &mut self,
        mesh: impl AsRef<ffi::Mesh>,
        material: WeakMaterial,
        transform: impl Into<ffi::Matrix>,
    ) {
        unsafe { ffi::DrawMesh(*mesh.as_ref(), material.0, transform.into()) }
    }

    /// Draw multiple mesh instances with material and different transforms
    #[inline]
    fn draw_mesh_instanced(
        &mut self,
        mesh: impl AsRef<ffi::Mesh>,
        material: WeakMaterial,
        transforms: &[Matrix],
    ) {
        unsafe {
            ffi::DrawMeshInstanced(
                *mesh.as_ref(),
                material.0,
                transforms.as_ptr(),
                transforms.len() as i32,
            )
        }
    }

    /// Draws a sphere.
    #[inline]
    fn draw_sphere(
        &mut self,
        center_pos: impl Into<Vector3>,
        radius: f32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSphere(center_pos.into(), radius, color.into());
        }
    }

    /// Draws a sphere with extended parameters.
    #[inline]
    fn draw_sphere_ex(
        &mut self,
        center_pos: impl Into<Vector3>,
        radius: f32,
        rings: i32,
        slices: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSphereEx(center_pos.into(), radius, rings, slices, color.into());
        }
    }

    /// Draws a sphere in wireframe.
    #[inline]
    fn draw_sphere_wires(
        &mut self,
        center_pos: impl Into<Vector3>,
        radius: f32,
        rings: i32,
        slices: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawSphereWires(center_pos.into(), radius, rings, slices, color.into());
        }
    }

    /// Draws a cylinder.
    #[inline]
    fn draw_cylinder(
        &mut self,
        position: impl Into<Vector3>,
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        slices: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCylinder(
                position.into(),
                radius_top,
                radius_bottom,
                height,
                slices,
                color.into(),
            );
        }
    }

    /// Draws a cylinder with extended parameters.
    #[inline]
    fn draw_cylinder_ex(
        &mut self,
        start_position: impl Into<Vector3>,
        end_position: impl Into<Vector3>,
        radius_start: f32,
        radius_end: f32,
        slices: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCylinderEx(
                start_position.into(),
                end_position.into(),
                radius_start,
                radius_end,
                slices,
                color.into(),
            );
        }
    }

    /// Draws a cylinder in wireframe.
    #[inline]
    fn draw_cylinder_wires(
        &mut self,
        position: impl Into<Vector3>,
        radius_top: f32,
        radius_bottom: f32,
        height: f32,
        slices: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCylinderWires(
                position.into(),
                radius_top,
                radius_bottom,
                height,
                slices,
                color.into(),
            );
        }
    }

    /// Draws a cylinder in wireframe with extended parameters.
    #[inline]
    fn draw_cylinder_wires_ex(
        &mut self,
        start_position: impl Into<Vector3>,
        end_position: impl Into<Vector3>,
        radius_start: f32,
        radius_end: f32,
        slices: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCylinderWiresEx(
                start_position.into(),
                end_position.into(),
                radius_start,
                radius_end,
                slices,
                color.into(),
            );
        }
    }

    /// Draw capsule with the center of its sphere caps at startPos and endPos
    #[inline]
    fn draw_capsule(
        &mut self,
        start_pos: impl Into<Vector3>,
        end_pos: impl Into<Vector3>,
        radius: f32,
        slices: i32,
        rings: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCapsule(
                start_pos.into(),
                end_pos.into(),
                radius,
                slices,
                rings,
                color.into(),
            )
        }
    }

    ///Draw capsule wireframe with the center of its sphere caps at startPos and endPos
    #[inline]
    fn draw_capsule_wires(
        &mut self,
        start_pos: impl Into<Vector3>,
        end_pos: impl Into<Vector3>,
        radius: f32,
        slices: i32,
        rings: i32,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawCapsuleWires(
                start_pos.into(),
                end_pos.into(),
                radius,
                slices,
                rings,
                color.into(),
            )
        }
    }

    /// Draws an X/Z plane.
    #[inline]
    fn draw_plane(
        &mut self,
        center_pos: impl Into<Vector3>,
        size: impl Into<Vector2>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawPlane(center_pos.into(), size.into(), color.into());
        }
    }

    /// Draws a ray line.
    #[inline]
    fn draw_ray(&mut self, ray: impl Into<ffi::Ray>, color: impl Into<ffi::Color>) {
        unsafe {
            ffi::DrawRay(ray.into(), color.into());
        }
    }

    /// Draws a grid (centered at (0, 0, 0)).
    #[inline]
    fn draw_grid(&mut self, slices: i32, spacing: f32) {
        unsafe {
            ffi::DrawGrid(slices, spacing);
        }
    }

    /// Draws a model (with texture if set).
    #[inline]
    fn draw_model(
        &mut self,
        model: impl AsRef<ffi::Model>,
        position: impl Into<Vector3>,
        scale: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawModel(*model.as_ref(), position.into(), scale, tint.into());
        }
    }

    /// Draws a model with extended parameters.
    #[inline]
    fn draw_model_ex(
        &mut self,
        model: impl AsRef<ffi::Model>,
        position: impl Into<Vector3>,
        rotation_axis: impl Into<Vector3>,
        rotation_angle: f32,
        scale: impl Into<Vector3>,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawModelEx(
                *model.as_ref(),
                position.into(),
                rotation_axis.into(),
                rotation_angle,
                scale.into(),
                tint.into(),
            );
        }
    }

    /// Draws a model with wires (with texture if set).
    #[inline]
    fn draw_model_wires(
        &mut self,
        model: impl AsRef<ffi::Model>,
        position: impl Into<Vector3>,
        scale: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawModelWires(*model.as_ref(), position.into(), scale, tint.into());
        }
    }

    /// Draws a model with wires.
    #[inline]
    fn draw_model_wires_ex(
        &mut self,
        model: impl AsRef<ffi::Model>,
        position: impl Into<Vector3>,
        rotation_axis: impl Into<Vector3>,
        rotation_angle: f32,
        scale: impl Into<Vector3>,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawModelWiresEx(
                *model.as_ref(),
                position.into(),
                rotation_axis.into(),
                rotation_angle,
                scale.into(),
                tint.into(),
            );
        }
    }

    /// Draws a bounding box (wires).
    #[inline]
    fn draw_bounding_box(
        &mut self,
        bbox: impl Into<ffi::BoundingBox>,
        color: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawBoundingBox(bbox.into(), color.into());
        }
    }

    /// Draws a billboard texture.
    #[inline]
    fn draw_billboard(
        &mut self,
        camera: impl Into<ffi::Camera3D>,
        texture: &Texture2D,
        center: impl Into<Vector3>,
        size: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawBillboard(camera.into(), texture.0, center.into(), size, tint.into());
        }
    }

    /// Draws a billboard texture defined by `source_rec`.
    #[inline]
    fn draw_billboard_rec(
        &mut self,
        camera: impl Into<ffi::Camera3D>,
        texture: &Texture2D,
        source_rec: impl Into<ffi::Rectangle>,
        center: impl Into<Vector3>,
        size: impl Into<Vector2>,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawBillboardRec(
                camera.into(),
                texture.0,
                source_rec.into(),
                center.into(),
                size.into(),
                tint.into(),
            );
        }
    }

    /// Draw a billboard texture defined by source and rotation
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn draw_billboard_pro(
        &mut self,
        camera: impl Into<ffi::Camera>,
        texture: impl Into<ffi::Texture2D>,
        source: impl Into<ffi::Rectangle>,
        position: impl Into<Vector3>,
        up: impl Into<Vector3>,
        size: impl Into<Vector2>,
        origin: impl Into<Vector2>,
        rotation: f32,
        tint: impl Into<ffi::Color>,
    ) {
        unsafe {
            ffi::DrawBillboardPro(
                camera.into(),
                texture.into(),
                source.into(),
                position.into(),
                up.into(),
                size.into(),
                origin.into(),
                rotation,
                tint.into(),
            )
        }
    }
}
