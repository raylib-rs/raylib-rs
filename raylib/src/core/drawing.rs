//! Contains code related to drawing. Types that can be set as a surface to draw will implement the [`RaylibDraw`] trait

use crate::{
    core::{RaylibHandle, RaylibThread, texture::Texture2D, vr::VrStereoConfig},
    ffi,
    math::{Matrix, Rectangle, Vector2, Vector3},
    models::WeakMaterial,
    shaders::Shader,
};
use std::{
    ffi::CString,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// Seems like all draw commands must be issued from the main thread
impl RaylibHandle {
    /// Setup canvas (framebuffer) to start drawing.
    /// Prefer using the closure version, [`RaylibHandle::draw`]. This version returns a handle that calls [`ffi::EndDrawing`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    pub fn begin_drawing<'a>(&'a mut self, _: &RaylibThread) -> RaylibDrawHandle<'a> {
        // SAFETY: Borrowing RaylibHandle proves that Raylib, its window, and GL functions are successfully loaded and that none of them can be
        // released for the lifetime of `'a`. Borrowing RaylibThread, which can only be constructed *by* initializing Raylib and explicitly
        // denies `Send`/`Sync` proves we are on the thread that initialized Raylib.
        unsafe {
            ffi::BeginDrawing();
        };

        RaylibDrawHandle(self, PhantomData)
    }

    /// Setup canvas (framebuffer) to start drawing.
    // Every FnMut is a FnOnce, but not every FnOnce is a FnMut. The closure may possibly execute multiple times throughout the program, but not multiple times in a single call to this method.
    // Taking a FnOnce instead of a FnMut when the function only needs to be called once in this method makes the method slightly more versatile/less needlessly restrictive for no actual cost.
    pub fn draw<'a>(&'a mut self, _: &RaylibThread, func: impl FnOnce(RaylibDrawHandle<'a>)) {
        // SAFETY: Borrowing RaylibHandle proves that Raylib, its window, and GL functions are successfully loaded and that none of them can be
        // released for the lifetime of `'a`. Borrowing RaylibThread, which can only be constructed *by* initializing Raylib and explicitly
        // denies `Send`/`Sync` proves we are on the thread that initialized Raylib.
        unsafe {
            ffi::BeginDrawing();
        };
        func(RaylibDrawHandle(self, PhantomData));
        // Uncomment the following if RaylibDrawHandle has been changed to no longer call EndDrawing() in its drop implementation:
        // unsafe {
        //     ffi::EndDrawing();
        // }
    }
}

/// Handle returned by [`begin_drawing`](RaylibHandle::begin_drawing) to provide access to drawing functions.
///
/// Calls [`ffi::EndDrawing`] when dropped.
///
/// # Guarantees
///
/// [`RaylibDrawHandle`] guarantees [`ffi::BeginDrawing`] has been called this frame without the corresponding [`ffi::EndDrawing`] having been called yet.
/// Raylib, the window, and GL functions are guaranteed to have loaded successfully and will not be released for the duration of `'a`.
/// [`RaylibDrawHandle`] cannot be shared across threads, guaranteeing borrowers are on the thread that initiated drawing this frame.
pub struct RaylibDrawHandle<'a>(&'a mut RaylibHandle, PhantomData<*const ()>);

impl RaylibDrawHandle<'_> {
    #[deprecated = "Calling begin_drawing within RaylibDrawHandle will result in a runtime error."]
    #[doc(hidden)]
    pub fn begin_drawing(&mut self, _: &RaylibThread) -> RaylibDrawHandle<'_> {
        panic!("Nested begin_drawing call")
    }

    #[deprecated = "Calling draw within RaylibDrawHandle will result in a runtime error."]
    #[doc(hidden)]
    pub fn draw(&mut self, _: &RaylibThread, mut _func: impl FnMut(RaylibDrawHandle)) {
        panic!("Nested draw call")
    }
}

impl Drop for RaylibDrawHandle<'_> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: RaylibDrawHandle guarantees BeginDrawing has been called this frame without the corresponding EndDrawing having been called yet.
        unsafe {
            ffi::EndDrawing();
        }
    }
}

impl Deref for RaylibDrawHandle<'_> {
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for RaylibDrawHandle<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

// SAFETY: See `RaylibDrawHandle` guarantees.
unsafe impl RaylibDraw for RaylibDrawHandle<'_> {}

// Texture2D Stuff

/// Handle returned by [`begin_texture_mode`](RaylibTextureModeExt::begin_texture_mode) to provide access to drawing functions.
///
/// Calls [`ffi::EndTextureMode`] when dropped.
///
/// # Guarantees
///
/// [`RaylibTextureMode`] guarantees [`ffi::BeginTextureMode`] has been called this frame without the corresponding [`ffi::EndTextureMode`] having been called yet.
/// Raylib, the window, and GL functions are guaranteed to have loaded successfully and will not be released for the duration of `'a`.
/// [`RaylibTextureMode`] cannot be shared across threads, guaranteeing borrowers are on the thread that initiated drawing this frame.
// The texture does not need to be held exclusively for the duration of the *previous* draw mode, nor does the *previous* draw mode need to outlive the texture reference.
// The texture exclusivity and previous draw mode only need to outlive the *current* draw mode. So they can (and should) have separate lifetimes.
//
// Additionally: the texture is not actually *used* by this wrapper after construction, only the previous draw mode.
// Some space can be saved by using a phantom instead of copying the actual reference.
// The PhantomData will ensure that the borrow checker still analyzes as though the mutable texture reference was held, without physically storing it in the runtime memory.
pub struct RaylibTextureMode<'a, 'b, T: ?Sized + RaylibTextureModeExt + 'a>(
    &'a mut T,
    PhantomData<&'b mut ffi::RenderTexture2D>,
    PhantomData<*const ()>,
);

impl<'a, T: ?Sized + RaylibTextureModeExt + 'a> Drop for RaylibTextureMode<'a, '_, T> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: RaylibTextureMode guarantees BeginTextureMode has been called this frame without the corresponding EndTextureMode having been called yet.
        unsafe { ffi::EndTextureMode() }
    }
}

impl<'a, T: ?Sized + RaylibTextureModeExt + 'a> Deref for RaylibTextureMode<'a, '_, T>
where
    T: Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: ?Sized + RaylibTextureModeExt + 'a> DerefMut for RaylibTextureMode<'a, '_, T>
where
    T: DerefMut<Target = RaylibHandle>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}
// framebuffer: &'a mut ffi::RenderTexture2D,

/// Ability to begin drawing to a [`ffi::RenderTexture2D`].
///
/// # Safety
///
/// Implementors must guarantee that borrowing `Self` proves that Raylib, its window, and GL functions are
/// successfully loaded and that none of them will be released for the lifetime of the borrow.
///
/// It must also be guaranteed that [`ffi::BeginTextureMode`] is safe to call given the current render mode.
pub unsafe trait RaylibTextureModeExt {
    /// Begin drawing to render texture.
    ///
    /// Prefer using the closure version, [`RaylibTextureModeExt::draw_texture_mode`].
    /// This version returns a handle that calls [`ffi::EndTextureMode`] at the end of the scope and is provided as a fallback incase you run into issues with closures (such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    fn begin_texture_mode<'a, 'b>(
        &'a mut self,
        _: &RaylibThread,
        framebuffer: &'b mut ffi::RenderTexture2D,
    ) -> RaylibTextureMode<'a, 'b, Self> {
        // SAFETY: Implementor must uphold the trait safety contract.
        unsafe { ffi::BeginTextureMode(*framebuffer) }
        RaylibTextureMode(self, PhantomData, PhantomData)
    }

    /// Begin drawing to render texture.
    fn draw_texture_mode<'a, 'b>(
        &'a mut self,
        _: &RaylibThread,
        framebuffer: &'b mut ffi::RenderTexture2D,
        func: impl FnOnce(RaylibTextureMode<'a, 'b, Self>),
    ) {
        // SAFETY: Implementor must uphold the trait safety contract.
        unsafe { ffi::BeginTextureMode(*framebuffer) }
        func(RaylibTextureMode(self, PhantomData, PhantomData));
        // Uncomment the following if RaylibTextureMode has been changed to no longer call EndTextureMode() in its drop implementation:
        // unsafe { ffi::EndTextureMode(); }
    }
}

// Only the `DrawHandle` and the `RaylibHandle` can start a texture

// SAFETY: `RaylibHandle` inherently proves that Raylib, its window, and GL functions are
// successfully loaded and that none of them will be released for its lifetime.
// It is safe to initialize Raylib texture mode while drawing is not active so long as the
// window and GL functions are ready.
unsafe impl RaylibTextureModeExt for RaylibHandle {}

// SAFETY: `RaylibDrawHandle` borrows `RaylibHandle`, proving that Raylib, its window, and GL
// functions are successfully loaded and that none of them will be released for its lifetime.
// It is safe to initialize Raylib texture mode while drawing is active when no additional
// draw modes are active.
unsafe impl RaylibTextureModeExt for RaylibDrawHandle<'_> {}

// SAFETY: see `RaylibTextureMode` guarantees.
unsafe impl<'a, T: ?Sized + RaylibTextureModeExt + 'a> RaylibDraw for RaylibTextureMode<'a, '_, T> {}

// VR Stuff

/// Handle returned by [`begin_vr_stereo_mode`](RaylibVRModeExt::begin_vr_stereo_mode) to provide access to drawing functions.
///
/// Calls [`ffi::EndVrStereoMode`] when dropped.
///
/// # Guarantees
///
/// [`RaylibVRMode`] guarantees [`ffi::BeginVrStereoMode`] has been called this frame without the corresponding [`ffi::EndVrStereoMode`] having been called yet.
// Lifetime 'a is duplicatively stored in a PhantomData so that the borrow checker knows T is being held *exclusively* for the lifetime of the mode, without giving the false impression that it can actually be mutated by the library.
pub struct RaylibVRMode<'a, 'b, T: ?Sized + RaylibVRModeExt + 'a>(
    &'a T,
    PhantomData<&'a mut T>,
    PhantomData<&'b mut VrStereoConfig>,
);

impl<'a, T: ?Sized + RaylibVRModeExt + 'a> Drop for RaylibVRMode<'a, '_, T> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: RaylibVRMode guarantees BeginVrStereoMode has been called this frame without the corresponding EndVrStereoMode having been called yet.
        unsafe {
            ffi::EndVrStereoMode();
        }
    }
}

impl<'a, T: 'a> Deref for RaylibVRMode<'a, '_, T>
where
    T: ?Sized + RaylibVRModeExt + Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// Ability to begin drawing in VR stereo mode.
pub trait RaylibVRModeExt: RaylibDraw {
    /// Begin stereo rendering (requires VR simulator).
    ///
    /// Prefer using the closure version, [`RaylibVRModeExt::draw_vr_stereo_mode`].
    ///
    /// This version returns a handle that calls [`ffi::EndVrStereoMode`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    fn begin_vr_stereo_mode<'a, 'b>(
        &'a mut self,
        _: &RaylibThread,
        vr_config: &'b mut VrStereoConfig,
    ) -> RaylibVRMode<'a, 'b, Self> {
        // SAFETY: Implementors of RaylibDraw are required to ensure draw functions are safe to call for the duration of their lifetime.
        unsafe { ffi::BeginVrStereoMode(*vr_config.as_ref()) }
        RaylibVRMode(self, PhantomData, PhantomData)
    }

    /// Begin stereo rendering (requires VR simulator).
    fn draw_vr_stereo_mode<'a, 'b>(
        &'a mut self,
        vr_config: &'b mut VrStereoConfig,
        func: impl FnOnce(RaylibVRMode<'a, 'b, Self>),
    ) {
        // SAFETY: Implementors of RaylibDraw are required to ensure draw functions are safe to call for the duration of their lifetime.
        unsafe { ffi::BeginVrStereoMode(*vr_config.as_ref()) }
        func(RaylibVRMode(self, PhantomData, PhantomData));
        // Uncomment the following if RaylibVRMode has been changed to no longer call EndTextureMode() in its drop implementation:
        // unsafe { ffi::EndVrStereoMode(); }
    }
}

impl<D: ?Sized + RaylibDraw> RaylibVRModeExt for D {}
unsafe impl<'a, T: ?Sized + RaylibVRModeExt + 'a> RaylibDraw for RaylibVRMode<'a, '_, T> {}

// 2D Mode

/// Handle returned by [`begin_mode2D`](RaylibMode2DExt::begin_mode2D) to provide access to 2D drawing functions.
///
/// Calls [`ffi::EndMode2D`] when dropped.
///
/// # Guarantees
///
/// [`RaylibMode2D`] guarantees [`ffi::BeginMode2D`] has been called this frame without the corresponding [`ffi::EndMode2D`] having been called yet.
pub struct RaylibMode2D<'a, T: 'a>(&'a mut T);

impl<'a, T: 'a> Drop for RaylibMode2D<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndMode2D() }
    }
}

impl<'a, T: 'a> Deref for RaylibMode2D<'a, T>
where
    T: Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: 'a> DerefMut for RaylibMode2D<'a, T>
where
    T: DerefMut<Target = RaylibHandle>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Ability to begin drawing in 2D.
pub trait RaylibMode2DExt
where
    Self: Sized,
{
    /// Begin 2D mode with custom camera (2D).
    ///
    /// Prefer using the closure version, [`RaylibMode2DExt::draw_mode2D`].
    /// This version returns a handle that calls [`ffi::EndMode2D`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[allow(non_snake_case, reason = "consistent style")]
    #[inline]
    #[must_use]
    fn begin_mode2D(&mut self, camera: impl Into<ffi::Camera2D>) -> RaylibMode2D<'_, Self> {
        unsafe {
            ffi::BeginMode2D(camera.into());
        }
        RaylibMode2D(self)
    }

    /// Begin 2D mode with custom camera (2D).
    #[allow(non_snake_case, reason = "consistent style")]
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
unsafe impl<'a, T: 'a> RaylibDraw for RaylibMode2D<'a, T> {}

// 3D Mode

/// Handle returned by [`begin_mode3D`](RaylibMode3DExt::begin_mode3D) to provide access to 3D drawing functions.
///
/// Calls [`ffi::EndMode3D`] when dropped.
pub struct RaylibMode3D<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibMode3D<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndMode3D() }
    }
}
impl<'a, T: 'a> Deref for RaylibMode3D<'a, T>
where
    T: Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> DerefMut for RaylibMode3D<'a, T>
where
    T: DerefMut<Target = RaylibHandle>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Ability to begin drawing in 3D.
pub trait RaylibMode3DExt
where
    Self: Sized,
{
    /// Begin 3D mode with custom camera (3D).
    ///
    /// Prefer using the closure version, [`RaylibMode3DExt::draw_mode3D`].
    /// This version returns a handle that calls [`ffi::EndMode3D`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[allow(non_snake_case, reason = "consistent style")]
    #[inline]
    #[must_use]
    fn begin_mode3D(&mut self, camera: impl Into<ffi::Camera3D>) -> RaylibMode3D<'_, Self> {
        unsafe {
            ffi::BeginMode3D(camera.into());
        }
        RaylibMode3D(self)
    }

    /// Begin 3D mode with custom camera (3D).
    #[allow(non_snake_case, reason = "consistent style")]
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
unsafe impl<'a, T: 'a> RaylibDraw for RaylibMode3D<'a, T> {}
unsafe impl<'a, T: 'a> RaylibDraw3D for RaylibMode3D<'a, T> {}

// shader Mode

/// Handle returned by [`begin_shader_mode`](RaylibShaderModeExt::begin_shader_mode) to provide access to drawing functions.
///
/// Calls [`ffi::EndShaderMode`] when dropped.
pub struct RaylibShaderMode<'a, 'b, T: 'a>(&'a mut T, PhantomData<&'b mut Shader>);

impl<'a, T: 'a> Drop for RaylibShaderMode<'a, '_, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndShaderMode() }
    }
}
impl<'a, T: 'a> Deref for RaylibShaderMode<'a, '_, T>
where
    T: Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> DerefMut for RaylibShaderMode<'a, '_, T>
where
    T: DerefMut<Target = RaylibHandle>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Ability to begin drawing with a shader applied.
pub trait RaylibShaderModeExt
where
    Self: Sized,
{
    /// Begin custom shader drawing.
    ///
    /// Prefer using the closure version, [`RaylibShaderModeExt::draw_shader_mode`].
    /// This version returns a handle that calls [`ffi::EndShaderMode`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
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
unsafe impl<'a, T: 'a> RaylibDraw for RaylibShaderMode<'a, '_, T> {}
unsafe impl<'a, T: 'a> RaylibDraw3D for RaylibShaderMode<'a, '_, T> {}

// Blend Mode

/// Handle returned by [`begin_blend_mode`](RaylibBlendModeExt::begin_blend_mode) to provide access to drawing functions.
///
/// Calls [`ffi::EndBlendMode`] when dropped.
pub struct RaylibBlendMode<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibBlendMode<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndBlendMode() }
    }
}
impl<'a, T: 'a> Deref for RaylibBlendMode<'a, T>
where
    T: Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> DerefMut for RaylibBlendMode<'a, T>
where
    T: DerefMut<Target = RaylibHandle>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Ability to begin drawing with a blend mode applied.
pub trait RaylibBlendModeExt
where
    Self: Sized,
{
    /// Begin blending mode (alpha, additive, multiplied, subtract, custom).
    ///
    /// Prefer using the closure version, [`RaylibBlendModeExt::draw_blend_mode`].
    /// This version returns a handle that calls [`ffi::EndBlendMode`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
    fn begin_blend_mode(
        &mut self,
        blend_mode: crate::consts::BlendMode,
    ) -> RaylibBlendMode<'_, Self> {
        unsafe { ffi::BeginBlendMode(blend_mode as i32) }
        RaylibBlendMode(self)
    }

    /// Begin blending mode (alpha, additive, multiplied, subtract, custom).
    fn draw_blend_mode<'a>(
        &'a mut self,
        blend_mode: crate::consts::BlendMode,
        func: impl FnOnce(RaylibBlendMode<'a, Self>),
    ) {
        unsafe { ffi::BeginBlendMode(blend_mode as i32) }
        func(RaylibBlendMode(self));
        // Uncomment the following if RaylibBlendMode has been changed to no longer call EndBlendMode() in its drop implementation:
        // unsafe { ffi::EndBlendMode(); }
    }
}

impl<D: RaylibDraw> RaylibBlendModeExt for D {}
unsafe impl<'a, T: 'a> RaylibDraw for RaylibBlendMode<'a, T> {}
unsafe impl<'a, T: 'a> RaylibDraw3D for RaylibBlendMode<'a, T> {}

// Scissor Mode stuff

/// Handle returned by [`begin_scissor_mode`](RaylibScissorModeExt::begin_scissor_mode) to provide access to drawing functions.
///
/// Calls [`ffi::EndScissorMode`] when dropped.
pub struct RaylibScissorMode<'a, T: 'a>(&'a mut T);
impl<'a, T: 'a> Drop for RaylibScissorMode<'a, T> {
    fn drop(&mut self) {
        unsafe { ffi::EndScissorMode() }
    }
}
impl<'a, T: 'a> Deref for RaylibScissorMode<'a, T>
where
    T: Deref<Target = RaylibHandle>,
{
    type Target = RaylibHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl<'a, T: 'a> DerefMut for RaylibScissorMode<'a, T>
where
    T: DerefMut<Target = RaylibHandle>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Ability to begin drawing in scissor mode.
pub trait RaylibScissorModeExt
where
    Self: Sized,
{
    /// Begin scissor mode (define screen area for following drawing).
    ///
    /// Prefer using the closure version, [`RaylibScissorModeExt::draw_scissor_mode`].
    /// This version returns a handle that calls [`ffi::EndScissorMode`] at the end of the scope and is provided as a fallback incase you run into issues with closures(such as lifetime or performance reasons)
    #[inline]
    #[must_use]
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
unsafe impl<'a, T: 'a> RaylibDraw for RaylibScissorMode<'a, T> {}
unsafe impl<'a, T: 'a + RaylibDraw3D> RaylibDraw3D for RaylibScissorMode<'a, T> {}

// Actual drawing functions

/// Permits external implementation of [`RaylibDraw`].
///
/// Implementing this trait on your type will automatically impl the default
/// definition of [`RaylibDraw`] for it.
///
/// # Safety
///
/// This trait implements [`RaylibDraw`] for your type. Implementors of
/// [`RaylibDrawImpl`] must uphold the safety contract of [`RaylibDraw`].
///
/// # Example
/// ```
/// use raylib::prelude::*;
///
/// struct Foo<'a, D>(&'a mut D);
///
/// // SAFETY: `Foo` stores a mutable `RaylibDraw` reference without creating
/// // unbalanced draw modes, proving it is safe to call draw functions.
/// unsafe impl<D: RaylibDraw> RaylibDrawImpl for Foo<'_, D> {}
///
/// fn foo_draw(foo: &mut Foo<'_, impl RaylibDraw>) {
///     foo.clear_background(Color::RAYWHITE);
/// }
///
/// fn foo_draw_caller(d: &mut impl RaylibDraw) {
///     foo_draw(&mut Foo(d));
/// }
/// ```
pub unsafe trait RaylibDrawImpl {}

/// Permits external implementation of [`RaylibDraw3D`].
///
/// Implementing this trait on your type will automatically impl the default
/// definition of [`RaylibDraw3D`] for it.
///
/// # Safety
///
/// This trait implements [`RaylibDraw3D`] for your type. Implementors of
/// [`RaylibDraw3DImpl`] must uphold the safety contract of [`RaylibDraw3D`].
///
/// # Example
/// ```
/// use raylib::prelude::*;
///
/// struct Bar<'a, D>(&'a mut D);
///
/// // SAFETY: `Bar` stores a mutable `RaylibDraw3D` reference without creating
/// // unbalanced draw modes, proving it is safe to call draw functions.
/// unsafe impl<D: RaylibDraw3D> RaylibDraw3DImpl for Bar<'_, D> {}
///
/// fn bar_draw(bar: &mut Bar<'_, impl RaylibDraw3D>) {
///     bar.draw_point3D(Vector3::new(1.0, 5.0, -4.0), Color::RED);
/// }
///
/// fn bar_draw_caller(d: &mut impl RaylibDraw3D) {
///     bar_draw(&mut Bar(d));
/// }
/// ```
pub unsafe trait RaylibDraw3DImpl {}

mod sealed {
    //! External impl of RaylibDraw/RaylibDraw3D is permitted, but redefinition is not.
    use super::*;

    // SAFETY: Implementors of `RaylibDrawImpl` must uphold `RaylibDraw`'s safety contract.
    unsafe impl<D: ?Sized + RaylibDrawImpl> RaylibDraw for D {}

    // SAFETY: Implementors of `RaylibDraw3DImpl` must uphold `RaylibDraw3D`'s safety contract.
    unsafe impl<D: ?Sized + RaylibDraw3DImpl> RaylibDraw3D for D {}

    /// Types through which it is safe to call standard Raylib drawing functions.
    ///
    /// # Safety
    ///
    /// The default implementation of the draw functions provided by this trait (which generally should never be overridden)
    /// access global static memory without locking, perform calls with function pointers that may not have been loaded,
    /// and expect for there to be a window and buffer to target--all without any checks.
    ///
    /// Implementors must guarantee that their type can only exist if Raylib has been successfully initialized with a window,
    /// has successfully loaded any necessary GL libraries, the calling thread is the same one that initialized Raylib, and
    /// [`ffi::BeginDrawing`] or [`ffi::BeginTextureMode`] has been called this frame without the corresponding
    /// [`ffi::EndDrawing`]/[`ffi::EndTextureMode`] having been called yet.
    pub unsafe trait RaylibDraw {
        /// Sets background color (framebuffer clear `color.into()`).
        #[inline]
        fn clear_background(&mut self, color: impl Into<ffi::Color>) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::ClearBackground(color.into());
            }
        }

        /// Get texture that is used for shapes drawing
        #[inline]
        #[must_use]
        fn get_shapes_texture(&self) -> Texture2D {
            // SAFETY: Implementor must uphold trait safety contract.
            Texture2D(unsafe { ffi::GetShapesTexture() })
        }

        /// Get texture source rectangle that is used for shapes drawing
        #[inline]
        #[must_use]
        fn get_shapes_texture_rectangle(&self) -> Rectangle {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe { ffi::GetShapesTextureRectangle() }
        }

        /// Define default texture used to draw shapes
        #[inline]
        fn set_shapes_texture(
            &mut self,
            texture: impl AsRef<ffi::Texture2D>,
            source: impl Into<ffi::Rectangle>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawPixel(x, y, color.into());
            }
        }

        /// Draws a pixel (Vector version).
        #[inline]
        fn draw_pixel_v(
            &mut self,
            position: impl Into<ffi::Vector2>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawLine(start_pos_x, start_pos_y, end_pos_x, end_pos_y, color.into());
            }
        }

        /// Draws a line (Vector version).
        #[inline]
        fn draw_line_v(
            &mut self,
            start_pos: impl Into<ffi::Vector2>,
            end_pos: impl Into<ffi::Vector2>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawLineV(start_pos.into(), end_pos.into(), color.into());
            }
        }

        /// Draws a line with thickness.
        #[inline]
        fn draw_line_ex(
            &mut self,
            start_pos: impl Into<ffi::Vector2>,
            end_pos: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawLineEx(start_pos.into(), end_pos.into(), thick, color.into());
            }
        }

        /// Draws a line using cubic-bezier curves in-out.
        #[inline]
        fn draw_line_bezier(
            &mut self,
            start_pos: impl Into<ffi::Vector2>,
            end_pos: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawLineBezier(start_pos.into(), end_pos.into(), thick, color.into());
            }
        }

        /// Draw lines sequence
        #[inline]
        fn draw_line_strip(&mut self, points: &[Vector2], color: impl Into<ffi::Color>) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawLineStrip(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCircle(center_x, center_y, radius, color.into());
            }
        }

        /// Draw a piece of a circle
        #[inline]
        fn draw_circle_sector(
            &mut self,
            center: impl Into<ffi::Vector2>,
            radius: f32,
            start_angle: f32,
            end_angle: f32,
            segments: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            center: impl Into<ffi::Vector2>,
            radius: f32,
            start_angle: f32,
            end_angle: f32,
            segments: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            color1: impl Into<ffi::Color>,
            color2: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCircleGradient(center_x, center_y, radius, color1.into(), color2.into());
            }
        }

        /// Draws a color-filled circle (Vector version).
        #[inline]
        fn draw_circle_v(
            &mut self,
            center: impl Into<ffi::Vector2>,
            radius: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCircleLines(center_x, center_y, radius, color.into());
            }
        }

        /// Draws circle outline. (Vector Version)
        #[inline]
        fn draw_circle_lines_v(
            &mut self,
            center: impl Into<ffi::Vector2>,
            radius: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawEllipse(center_x, center_y, radius_h, radius_v, color.into());
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawEllipseLines(center_x, center_y, radius_h, radius_v, color.into());
            }
        }

        /// Draw ring
        #[allow(clippy::too_many_arguments, reason = "consistency with Raylib")]
        #[inline]
        fn draw_ring(
            &mut self,
            center: impl Into<ffi::Vector2>,
            inner_radius: f32,
            outer_radius: f32,
            start_angle: f32,
            end_angle: f32,
            segments: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
        #[allow(clippy::too_many_arguments, reason = "consistency with Raylib")]
        #[inline]
        fn draw_ring_lines(
            &mut self,
            center: impl Into<ffi::Vector2>,
            inner_radius: f32,
            outer_radius: f32,
            start_angle: f32,
            end_angle: f32,
            segments: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawRectangle(x, y, width, height, color.into());
            }
        }

        /// Draws a color-filled rectangle (Vector version).
        #[inline]
        fn draw_rectangle_v(
            &mut self,
            position: impl Into<ffi::Vector2>,
            size: impl Into<ffi::Vector2>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawRectangleV(position.into(), size.into(), color.into());
            }
        }

        /// Draws a color-filled rectangle from `rec`.
        #[inline]
        fn draw_rectangle_rec(
            &mut self,
            rec: impl Into<ffi::Rectangle>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawRectangleRec(rec.into(), color.into());
            }
        }

        /// Draws a color-filled rectangle with pro parameters.
        #[inline]
        fn draw_rectangle_pro(
            &mut self,
            rec: impl Into<ffi::Rectangle>,
            origin: impl Into<ffi::Vector2>,
            rotation: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawRectangleRoundedLinesEx(
                    rec.into(),
                    roundness,
                    segments,
                    line_thickness,
                    color.into(),
                );
            }
        }

        /// Draws a triangle.
        #[inline]
        fn draw_triangle(
            &mut self,
            v1: impl Into<ffi::Vector2>,
            v2: impl Into<ffi::Vector2>,
            v3: impl Into<ffi::Vector2>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTriangle(v1.into(), v2.into(), v3.into(), color.into());
            }
        }

        /// Draws a triangle using lines.
        #[inline]
        fn draw_triangle_lines(
            &mut self,
            v1: impl Into<ffi::Vector2>,
            v2: impl Into<ffi::Vector2>,
            v3: impl Into<ffi::Vector2>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTriangleLines(v1.into(), v2.into(), v3.into(), color.into());
            }
        }

        /// Draw a triangle fan defined by points.
        #[inline]
        fn draw_triangle_fan(&mut self, points: &[Vector2], color: impl Into<ffi::Color>) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTriangleFan(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    color.into(),
                );
            }
        }

        /// Draw a triangle strip defined by points
        #[inline]
        fn draw_triangle_strip(&mut self, points: &[Vector2], color: impl Into<ffi::Color>) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTriangleStrip(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    color.into(),
                );
            }
        }

        /// Draws a regular polygon of n sides (Vector version).
        #[inline]
        fn draw_poly(
            &mut self,
            center: impl Into<ffi::Vector2>,
            sides: i32,
            radius: f32,
            rotation: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawPoly(center.into(), sides, radius, rotation, color.into());
            }
        }

        /// Draws a regular polygon of n sides (Vector version).
        #[inline]
        fn draw_poly_lines(
            &mut self,
            center: impl Into<ffi::Vector2>,
            sides: i32,
            radius: f32,
            rotation: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTexture(*texture.as_ref(), x, y, tint.into());
            }
        }

        /// Draws a `texture` using specified `position` vector and `tint` color.
        #[inline]
        fn draw_texture_v(
            &mut self,
            texture: impl AsRef<ffi::Texture2D>,
            position: impl Into<ffi::Vector2>,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTextureV(*texture.as_ref(), position.into(), tint.into());
            }
        }

        /// Draws a `texture` with extended parameters.
        #[inline]
        fn draw_texture_ex(
            &mut self,
            texture: impl AsRef<ffi::Texture2D>,
            position: impl Into<ffi::Vector2>,
            rotation: f32,
            scale: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            position: impl Into<ffi::Vector2>,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            origin: impl Into<ffi::Vector2>,
            rotation: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            origin: impl Into<ffi::Vector2>,
            rotation: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            let c_text = CString::new(text).expect("text should not contain an internal 0 byte");

            // SAFETY: Implementor must uphold trait safety contract.
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
            position: impl Into<ffi::Vector2>,
            font_size: f32,
            spacing: f32,
            tint: impl Into<ffi::Color>,
        ) {
            let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
            let mut len = 0;
            // SAFETY: Implementor must uphold trait safety contract.
            let u = unsafe { ffi::LoadCodepoints(c_text.as_ptr(), &raw mut len) };

            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTextCodepoints(
                    *font.as_ref(),
                    u,
                    text.len()
                        .try_into()
                        .expect("text should not exceed i32::MAX elements"),
                    position.into(),
                    font_size,
                    spacing,
                    tint.into(),
                );
            }
        }

        /// Draws text using `font` and additional parameters.
        #[inline]
        fn draw_text_ex(
            &mut self,
            font: impl AsRef<ffi::Font>,
            text: &str,
            position: impl Into<ffi::Vector2>,
            font_size: f32,
            spacing: f32,
            tint: impl Into<ffi::Color>,
        ) {
            let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
            // SAFETY: Implementor must uphold trait safety contract.
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
        #[allow(clippy::too_many_arguments, reason = "consistency with Raylib")]
        #[inline]
        fn draw_text_pro(
            &mut self,
            font: impl AsRef<ffi::Font>,
            text: &str,
            position: impl Into<ffi::Vector2>,
            origin: impl Into<ffi::Vector2>,
            rotation: f32,
            font_size: f32,
            spacing: f32,
            tint: impl Into<ffi::Color>,
        ) {
            let c_text = CString::new(text).expect("text should not contain an internal 0 byte");
            // SAFETY: Implementor must uphold trait safety contract.
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
            position: impl Into<ffi::Vector2>,
            scale: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe { ffi::EnableEventWaiting() }
        }

        /// Disable waiting for events when the handle is dropped, no automatic event polling
        #[inline]
        fn disable_event_waiting(&self) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe { ffi::DisableEventWaiting() }
        }

        /// Draw a polygon outline of n sides with extended parameters
        #[inline]
        fn draw_poly_lines_ex(
            &mut self,
            center: impl Into<ffi::Vector2>,
            sides: i32,
            radius: f32,
            rotation: f32,
            line_thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
        fn draw_spline_linear(
            &mut self,
            points: &[Vector2],
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineLinear(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    thick,
                    color.into(),
                );
            }
        }

        /// Draw spline: B-Spline, minimum 4 points
        #[inline]
        fn draw_spline_basis(
            &mut self,
            points: &[Vector2],
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineBasis(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    thick,
                    color.into(),
                );
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineCatmullRom(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    thick,
                    color.into(),
                );
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineBezierQuadratic(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    thick,
                    color.into(),
                );
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineBezierCubic(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    thick,
                    color.into(),
                );
            }
        }

        /// Draw spline segment: Linear, 2 points
        #[inline]
        fn draw_spline_segment_linear(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            p2: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe { ffi::DrawSplineSegmentLinear(p1.into(), p2.into(), thick, color.into()) }
        }

        /// Draw spline segment: B-Spline, 4 points
        #[inline]
        fn draw_spline_segment_basis(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            p2: impl Into<ffi::Vector2>,
            p3: impl Into<ffi::Vector2>,
            p4: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineSegmentBasis(
                    p1.into(),
                    p2.into(),
                    p3.into(),
                    p4.into(),
                    thick,
                    color.into(),
                );
            }
        }

        /// Draw spline segment: Catmull-Rom, 4 points
        #[inline]
        fn draw_spline_segment_catmull_rom(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            p2: impl Into<ffi::Vector2>,
            p3: impl Into<ffi::Vector2>,
            p4: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineSegmentCatmullRom(
                    p1.into(),
                    p2.into(),
                    p3.into(),
                    p4.into(),
                    thick,
                    color.into(),
                );
            }
        }

        /// Draw spline segment: Quadratic Bezier, 2 points, 1 control point
        #[inline]
        fn draw_spline_segment_bezier_quadratic(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            c2: impl Into<ffi::Vector2>,
            p3: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineSegmentBezierQuadratic(
                    p1.into(),
                    c2.into(),
                    p3.into(),
                    thick,
                    color.into(),
                );
            }
        }

        /// Draw spline segment: Cubic Bezier, 2 points, 2 control points
        #[inline]
        fn draw_spline_segment_bezier_cubic(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            c2: impl Into<ffi::Vector2>,
            c3: impl Into<ffi::Vector2>,
            p4: impl Into<ffi::Vector2>,
            thick: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSplineSegmentBezierCubic(
                    p1.into(),
                    c2.into(),
                    c3.into(),
                    p4.into(),
                    thick,
                    color.into(),
                );
            }
        }

        /// Get (evaluate) spline point: Linear
        #[inline]
        #[must_use]
        fn get_spline_point_linear(
            &mut self,
            start_pos: impl Into<ffi::Vector2>,
            end_pos: impl Into<ffi::Vector2>,
            t: f32,
        ) -> Vector2 {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe { ffi::GetSplinePointLinear(start_pos.into(), end_pos.into(), t).into() }
        }

        /// Get (evaluate) spline point: B-Spline
        #[inline]
        #[must_use]
        fn get_spline_point_basis(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            p2: impl Into<ffi::Vector2>,
            p3: impl Into<ffi::Vector2>,
            p4: impl Into<ffi::Vector2>,
            t: f32,
        ) -> Vector2 {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::GetSplinePointBasis(p1.into(), p2.into(), p3.into(), p4.into(), t).into()
            }
        }

        /// Get (evaluate) spline point: Catmull-Rom
        #[inline]
        #[must_use]
        fn get_spline_point_catmull_rom(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            p2: impl Into<ffi::Vector2>,
            p3: impl Into<ffi::Vector2>,
            p4: impl Into<ffi::Vector2>,
            t: f32,
        ) -> Vector2 {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::GetSplinePointCatmullRom(p1.into(), p2.into(), p3.into(), p4.into(), t).into()
            }
        }

        /// Get (evaluate) spline point: Quadratic Bezier
        #[inline]
        #[must_use]
        fn get_spline_point_bezier_quad(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            c2: impl Into<ffi::Vector2>,
            p3: impl Into<ffi::Vector2>,
            t: f32,
        ) -> Vector2 {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe { ffi::GetSplinePointBezierQuad(p1.into(), c2.into(), p3.into(), t).into() }
        }

        /// Get (evaluate) spline point: Cubic Bezier
        #[inline]
        #[must_use]
        fn get_spline_point_bezier_cubic(
            &mut self,
            p1: impl Into<ffi::Vector2>,
            c2: impl Into<ffi::Vector2>,
            c3: impl Into<ffi::Vector2>,
            p4: impl Into<ffi::Vector2>,
            t: f32,
        ) -> Vector2 {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::GetSplinePointBezierCubic(p1.into(), c2.into(), c3.into(), p4.into(), t).into()
            }
        }
    }

    /// Types through which it is safe to call 3D Raylib drawing functions.
    ///
    /// # Safety
    ///
    /// The default implementation of the draw functions provided by this trait (which generally should never be overridden)
    /// access global static memory without locking, perform calls with function pointers that may not have been loaded,
    /// and expect for there to be a window, buffer, & projection matrix to target--all without any checks.
    ///
    /// Implementors must guarantee that their type can only exist if Raylib has been successfully initialized with a window,
    /// has successfully loaded any necessary GL libraries, the calling thread is the same one that initialized Raylib, and
    /// [`ffi::BeginMode3D`] has been called this frame without the corresponding [`ffi::EndMode3D`] having been called yet.
    pub unsafe trait RaylibDraw3D {
        /// Draw a point in 3D space, actually a small line
        #[allow(non_snake_case, reason = "consistent style")]
        #[inline]
        fn draw_point3D(
            &mut self,
            position: impl Into<ffi::Vector3>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawPoint3D(position.into(), color.into());
            }
        }

        /// Draw a color-filled triangle (vertex in counter-clockwise order!)
        #[allow(non_snake_case, reason = "consistent style")]
        #[inline]
        fn draw_triangle3D(
            &mut self,
            v1: impl Into<ffi::Vector3>,
            v2: impl Into<ffi::Vector3>,
            v3: impl Into<ffi::Vector3>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTriangle3D(v1.into(), v2.into(), v3.into(), color.into());
            }
        }

        /// Draw a triangle strip defined by points
        #[allow(non_snake_case, reason = "consistent style")]
        #[inline]
        fn draw_triangle_strip3D(&mut self, points: &[Vector3], color: impl Into<ffi::Color>) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawTriangleStrip3D(
                    points.as_ptr().cast(),
                    points
                        .len()
                        .try_into()
                        .expect("points should not exceed i32::MAX elements"),
                    color.into(),
                );
            }
        }

        /// Draws a line in 3D world space.
        #[allow(non_snake_case, reason = "consistent style")]
        #[inline]
        fn draw_line3D(
            &mut self,
            start_pos: impl Into<ffi::Vector3>,
            end_pos: impl Into<ffi::Vector3>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawLine3D(start_pos.into(), end_pos.into(), color.into());
            }
        }

        /// Draws a circle in 3D world space.
        #[allow(non_snake_case, reason = "consistent style")]
        #[inline]
        fn draw_circle3D(
            &mut self,
            center: impl Into<ffi::Vector3>,
            radius: f32,
            rotation_axis: impl Into<ffi::Vector3>,
            rotation_angle: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            position: impl Into<ffi::Vector3>,
            width: f32,
            height: f32,
            length: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCube(position.into(), width, height, length, color.into());
            }
        }

        /// Draws a cube (Vector version).
        #[inline]
        fn draw_cube_v(
            &mut self,
            position: impl Into<ffi::Vector3>,
            size: impl Into<ffi::Vector3>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCubeV(position.into(), size.into(), color.into());
            }
        }

        /// Draws a cube in wireframe.
        #[inline]
        fn draw_cube_wires(
            &mut self,
            position: impl Into<ffi::Vector3>,
            width: f32,
            height: f32,
            length: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCubeWires(position.into(), width, height, length, color.into());
            }
        }

        /// Draws a cube in wireframe. (Vector Version)
        #[inline]
        fn draw_cube_wires_v(
            &mut self,
            position: impl Into<ffi::Vector3>,
            size: impl Into<ffi::Vector3>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawMeshInstanced(
                    *mesh.as_ref(),
                    material.0,
                    transforms.as_ptr().cast(),
                    transforms
                        .len()
                        .try_into()
                        .expect("transforms should not exceed i32::MAX elements"),
                );
            }
        }

        /// Draws a sphere.
        #[inline]
        fn draw_sphere(
            &mut self,
            center_pos: impl Into<ffi::Vector3>,
            radius: f32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSphere(center_pos.into(), radius, color.into());
            }
        }

        /// Draws a sphere with extended parameters.
        #[inline]
        fn draw_sphere_ex(
            &mut self,
            center_pos: impl Into<ffi::Vector3>,
            radius: f32,
            rings: i32,
            slices: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSphereEx(center_pos.into(), radius, rings, slices, color.into());
            }
        }

        /// Draws a sphere in wireframe.
        #[inline]
        fn draw_sphere_wires(
            &mut self,
            center_pos: impl Into<ffi::Vector3>,
            radius: f32,
            rings: i32,
            slices: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawSphereWires(center_pos.into(), radius, rings, slices, color.into());
            }
        }

        /// Draws a cylinder.
        #[inline]
        fn draw_cylinder(
            &mut self,
            position: impl Into<ffi::Vector3>,
            radius_top: f32,
            radius_bottom: f32,
            height: f32,
            slices: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            start_position: impl Into<ffi::Vector3>,
            end_position: impl Into<ffi::Vector3>,
            radius_start: f32,
            radius_end: f32,
            slices: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            position: impl Into<ffi::Vector3>,
            radius_top: f32,
            radius_bottom: f32,
            height: f32,
            slices: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            start_position: impl Into<ffi::Vector3>,
            end_position: impl Into<ffi::Vector3>,
            radius_start: f32,
            radius_end: f32,
            slices: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            start_pos: impl Into<ffi::Vector3>,
            end_pos: impl Into<ffi::Vector3>,
            radius: f32,
            slices: i32,
            rings: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCapsule(
                    start_pos.into(),
                    end_pos.into(),
                    radius,
                    slices,
                    rings,
                    color.into(),
                );
            }
        }

        ///Draw capsule wireframe with the center of its sphere caps at startPos and endPos
        #[inline]
        fn draw_capsule_wires(
            &mut self,
            start_pos: impl Into<ffi::Vector3>,
            end_pos: impl Into<ffi::Vector3>,
            radius: f32,
            slices: i32,
            rings: i32,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawCapsuleWires(
                    start_pos.into(),
                    end_pos.into(),
                    radius,
                    slices,
                    rings,
                    color.into(),
                );
            }
        }

        /// Draws an X/Z plane.
        #[inline]
        fn draw_plane(
            &mut self,
            center_pos: impl Into<ffi::Vector3>,
            size: impl Into<ffi::Vector2>,
            color: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawPlane(center_pos.into(), size.into(), color.into());
            }
        }

        /// Draws a ray line.
        #[inline]
        fn draw_ray(&mut self, ray: impl Into<ffi::Ray>, color: impl Into<ffi::Color>) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawRay(ray.into(), color.into());
            }
        }

        /// Draws a grid (centered at (0, 0, 0)).
        #[inline]
        fn draw_grid(&mut self, slices: i32, spacing: f32) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawGrid(slices, spacing);
            }
        }

        /// Draws a model (with texture if set).
        #[inline]
        fn draw_model(
            &mut self,
            model: impl AsRef<ffi::Model>,
            position: impl Into<ffi::Vector3>,
            scale: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawModel(*model.as_ref(), position.into(), scale, tint.into());
            }
        }

        /// Draws a model with extended parameters.
        #[inline]
        fn draw_model_ex(
            &mut self,
            model: impl AsRef<ffi::Model>,
            position: impl Into<ffi::Vector3>,
            rotation_axis: impl Into<ffi::Vector3>,
            rotation_angle: f32,
            scale: impl Into<ffi::Vector3>,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            position: impl Into<ffi::Vector3>,
            scale: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawModelWires(*model.as_ref(), position.into(), scale, tint.into());
            }
        }

        /// Draws a model with wires.
        #[inline]
        fn draw_model_wires_ex(
            &mut self,
            model: impl AsRef<ffi::Model>,
            position: impl Into<ffi::Vector3>,
            rotation_axis: impl Into<ffi::Vector3>,
            rotation_angle: f32,
            scale: impl Into<ffi::Vector3>,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            // SAFETY: Implementor must uphold trait safety contract.
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
            center: impl Into<ffi::Vector3>,
            size: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
            center: impl Into<ffi::Vector3>,
            size: impl Into<ffi::Vector2>,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
        #[allow(clippy::too_many_arguments, reason = "consistency with Raylib")]
        #[inline]
        fn draw_billboard_pro(
            &mut self,
            camera: impl Into<ffi::Camera>,
            texture: impl Into<ffi::Texture2D>,
            source: impl Into<ffi::Rectangle>,
            position: impl Into<ffi::Vector3>,
            up: impl Into<ffi::Vector3>,
            size: impl Into<ffi::Vector2>,
            origin: impl Into<ffi::Vector2>,
            rotation: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
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
                );
            }
        }

        /// Draw a model as points
        #[inline]
        fn draw_model_points(
            &mut self,
            model: impl Into<ffi::Model>,
            position: impl Into<ffi::Vector3>,
            scale: f32,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawModelPoints(model.into(), position.into(), scale, tint.into());
            }
        }

        /// Draw a model as points with extended parameters
        #[inline]
        fn draw_model_points_ex(
            &mut self,
            model: impl Into<ffi::Model>,
            position: impl Into<ffi::Vector3>,
            rotation_axis: impl Into<ffi::Vector3>,
            angle: f32,
            scale: impl Into<ffi::Vector3>,
            tint: impl Into<ffi::Color>,
        ) {
            // SAFETY: Implementor must uphold trait safety contract.
            unsafe {
                ffi::DrawModelPointsEx(
                    model.into(),
                    position.into(),
                    rotation_axis.into(),
                    angle,
                    scale.into(),
                    tint.into(),
                );
            }
        }
    }
}
pub use sealed::{RaylibDraw, RaylibDraw3D};
