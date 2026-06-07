//! Code for the safe manipulation of shaders

use crate::consts::ShaderUniformDataType;
use crate::core::math::Matrix;
use crate::core::math::{Vector2, Vector3, Vector4};
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi;
use std::ffi::CString;
use std::os::raw::c_void;

fn no_drop<T>(_thing: T) {}
// SOUNDNESS: readonly — P1 (locs slice trusts the locs pointer) + P2 (Drop=UnloadShader frees locs).
make_thin_wrapper!(
    /// GLSL shader program (vertex + fragment).
    ///
    /// Load a shader from files with [`RaylibHandle::load_shader`] or from in-memory
    /// source strings with [`RaylibHandle::load_shader_from_memory`] (pass `None` for
    /// either stage to use the default raylib shader for that stage).
    ///
    /// Once loaded, the typical workflow is:
    /// 1. Query a uniform location by name with [`RaylibShader::get_shader_location`].
    /// 2. Upload a value each frame with [`RaylibShader::set_shader_value`] (or the
    ///    vector/matrix/texture variants).
    /// 3. Activate the shader inside a drawing scope with `begin_shader_mode`.
    ///
    /// Freed via `UnloadShader` on drop.
    ///
    /// # Examples
    ///
    /// Load a custom shader and set a `float` uniform each frame:
    ///
    /// ```rust,no_run
    /// use raylib::prelude::*;
    /// let (mut rl, thread) = raylib::init().size(640, 480).title("shader demo").build();
    /// let mut shader = rl.load_shader(
    ///     &thread,
    ///     Some("assets/vs.glsl"),
    ///     Some("assets/fs.glsl"),
    /// );
    /// let time_loc = shader.get_shader_location("uTime");
    /// let mut t: f32 = 0.0;
    /// while !rl.window_should_close() {
    ///     t += rl.get_frame_time();
    ///     shader.set_shader_value(time_loc, t);
    ///     let mut d = rl.begin_drawing(&thread);
    ///     d.clear_background(Color::BLACK);
    ///     let _sm = d.begin_shader_mode(&mut shader);
    ///     // draw geometry here — it will be shaded by `shader`
    /// }
    /// ```
    Shader,
    ffi::Shader,
    ffi::UnloadShader,
    readonly
);
// SOUNDNESS: readonly — P1 (same locs accessor); no_drop removes P2.
make_thin_wrapper!(WeakShader, ffi::Shader, no_drop, readonly);

// #[cfg(feature = "nightly")]
// impl !Send for Shader {}
// #[cfg(feature = "nightly")]
// unsafe impl Sync for Shader {}

impl RaylibHandle {
    #[must_use]
    /// Loads a custom shader and binds default locations.
    pub fn load_shader(
        &mut self,
        _: &RaylibThread,
        vs_filename: Option<&str>,
        fs_filename: Option<&str>,
    ) -> Shader {
        let c_vs_filename = vs_filename.map(|f| CString::new(f).unwrap());
        let c_fs_filename = fs_filename.map(|f| CString::new(f).unwrap());

        let vs = c_vs_filename
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());
        let fs = c_fs_filename
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());

        Shader(unsafe { ffi::LoadShader(vs, fs) })
    }

    #[must_use]
    /// Loads shader from code strings and binds default locations.
    pub fn load_shader_from_memory(
        &mut self,
        _: &RaylibThread,
        vs_code: Option<&str>,
        fs_code: Option<&str>,
    ) -> Shader {
        let c_vs_code = vs_code.map(|f| CString::new(f).unwrap());
        let c_fs_code = fs_code.map(|f| CString::new(f).unwrap());

        let vs = c_vs_code
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());
        let fs = c_fs_code
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());

        Shader(unsafe { ffi::LoadShaderFromMemory(vs, fs) })
    }

    /// Sets a custom projection matrix (replaces internal projection matrix).
    #[inline]
    pub fn set_matrix_projection(&mut self, _: &RaylibThread, proj: impl Into<Matrix>) {
        unsafe {
            ffi::rlSetMatrixProjection(proj.into());
        }
    }

    /// Sets a custom modelview matrix (replaces internal modelview matrix).
    #[inline]
    pub fn set_matrix_modelview(&mut self, _: &RaylibThread, view: impl Into<Matrix>) {
        unsafe {
            ffi::rlSetMatrixModelview(view.into());
        }
    }

    /// Gets internal modelview matrix.
    #[must_use]
    #[inline]
    pub fn get_matrix_modelview(&self) -> Matrix {
        unsafe { ffi::rlGetMatrixModelview() }
    }

    /// Gets internal projection matrix.
    #[inline]
    #[must_use]
    pub fn get_matrix_projection(&self) -> Matrix {
        unsafe { ffi::rlGetMatrixProjection() }
    }
    #[inline]
    #[must_use]
    /// Get default shader. Modifying it modifies everything that uses that shader
    pub fn get_shader_default() -> WeakShader {
        unsafe {
            WeakShader(ffi::Shader {
                id: ffi::rlGetShaderIdDefault(),
                locs: ffi::rlGetShaderLocsDefault(),
            })
        }
    }
}

/// A Rust value that can be uploaded as a shader uniform variable.
///
/// Implemented for the scalar / vector primitives raylib's GLSL bindings accept: `f32`,
/// `i32`, `Vector2`/`Vector3`/`Vector4`, `[f32; 2..=4]`, `[i32; 2..=4]`, and the borrowed
/// `&[i32]` flavour for `sampler2D`. Each impl pins its [`UNIFORM_TYPE`](Self::UNIFORM_TYPE)
/// constant to the matching [`ShaderUniformDataType`] variant so callers don't pass the type
/// tag separately. The trait powers the generic [`Shader::set_shader_value`] and
/// [`Shader::set_shader_value_v`] methods (plus the per-uniform fast paths on
/// [`RaylibShader`]).
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
///
/// let (mut rl, thread) = raylib::init().size(640, 480).title("shader").build();
/// let mut shader = rl.load_shader(&thread, None, Some("assets/tint.fs"));
/// let loc = shader.get_shader_location("u_tint");
/// shader.set_shader_value(loc, Vector3::new(1.0, 0.5, 0.0));
/// ```
///
/// # See also
///
/// - [`Shader::set_shader_value`] — single-value uniform upload.
/// - [`Shader::set_shader_value_v`] — array uniform upload.
/// - [`RaylibShader`] — extension methods on shader handles.
pub trait ShaderV {
    /// The raylib [`ShaderUniformDataType`] that corresponds to this Rust type.
    ///
    /// Read by [`Shader::set_shader_value`] (and the vector variant) so the GLSL-side type
    /// tag matches the Rust value's layout — e.g. [`Vector3`] pins this to
    /// [`ShaderUniformDataType::SHADER_UNIFORM_VEC3`], `i32` to
    /// [`ShaderUniformDataType::SHADER_UNIFORM_INT`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    /// use raylib::core::shaders::ShaderV;
    ///
    /// assert_eq!(<f32 as ShaderV>::UNIFORM_TYPE, ShaderUniformDataType::SHADER_UNIFORM_FLOAT);
    /// assert_eq!(<Vector3 as ShaderV>::UNIFORM_TYPE, ShaderUniformDataType::SHADER_UNIFORM_VEC3);
    /// ```
    const UNIFORM_TYPE: ShaderUniformDataType;
    /// Returns a raw pointer to the shader value for use in FFI calls.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid for the lifetime of `self`. The caller must not
    /// dereference or use the pointer after `self` is dropped.
    unsafe fn value(&self) -> *const c_void;
}

impl ShaderV for f32 {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_FLOAT;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self as *const f32 as *const c_void
    }
}

impl ShaderV for Vector2 {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_VEC2;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self as *const Vector2 as *const c_void
    }
}

impl ShaderV for Vector3 {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_VEC3;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self as *const Vector3 as *const c_void
    }
}

impl ShaderV for Vector4 {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_VEC4;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self as *const Vector4 as *const c_void
    }
}

impl ShaderV for i32 {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_INT;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self as *const i32 as *const c_void
    }
}

impl ShaderV for [i32; 2] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_IVEC2;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl ShaderV for [i32; 3] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_IVEC3;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl ShaderV for [i32; 4] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_IVEC4;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl ShaderV for [f32; 2] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_VEC2;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl ShaderV for [f32; 3] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_VEC3;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl ShaderV for [f32; 4] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_VEC4;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl ShaderV for &[i32] {
    const UNIFORM_TYPE: ShaderUniformDataType = ShaderUniformDataType::SHADER_UNIFORM_SAMPLER2D;
    #[inline]
    unsafe fn value(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

impl Shader {
    /// # Safety
    ///
    /// The caller becomes responsible for ensuring the underlying `ffi::Shader` is eventually
    /// unloaded. The returned `WeakShader` does not call `UnloadShader` on drop.
    #[inline]
    #[must_use]
    pub unsafe fn make_weak(self) -> WeakShader {
        let m = WeakShader(self.0);
        std::mem::forget(self);
        m
    }

    /// Check if shader is valid
    #[inline]
    #[must_use]
    pub fn is_shader_valid(&self) -> bool {
        unsafe { ffi::IsShaderValid(self.0) }
    }

    /// Sets shader uniform value
    #[inline]
    pub fn set_shader_value<S: ShaderV>(&mut self, uniform_loc: i32, value: S) {
        unsafe {
            ffi::SetShaderValue(
                self.0,
                uniform_loc,
                value.value(),
                (S::UNIFORM_TYPE as u32) as i32,
            );
        }
    }

    /// Set shader uniform value vector
    #[inline]
    pub fn set_shader_value_v<S: ShaderV>(&mut self, uniform_loc: i32, value: &[S]) {
        unsafe {
            ffi::SetShaderValueV(
                self.0,
                uniform_loc,
                value.as_ptr() as *const ::std::os::raw::c_void,
                (S::UNIFORM_TYPE as u32) as i32,
                value.len() as i32,
            );
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    pub fn set_shader_value_matrix(&mut self, uniform_loc: i32, mat: impl Into<Matrix>) {
        unsafe {
            ffi::SetShaderValueMatrix(self.0, uniform_loc, mat.into());
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    pub fn set_shader_value_texture(
        &mut self,
        uniform_loc: i32,
        texture: impl AsRef<ffi::Texture2D>,
    ) {
        unsafe {
            ffi::SetShaderValueTexture(self.0, uniform_loc, *texture.as_ref());
        }
    }
}

impl RaylibShader for WeakShader {}
impl RaylibShader for Shader {}

/// Extension methods for types that wrap a raylib `Shader` (both owned [`Shader`] and [`WeakShader`]).
///
/// Implemented for [`Shader`] (RAII-owned) and [`WeakShader`] (no-drop alias from
/// [`Shader::make_weak`]). Provides the field accessors ([`locs`](Self::locs),
/// [`locs_mut`](Self::locs_mut)) plus uniform-location lookup
/// ([`get_shader_location`](Self::get_shader_location),
/// [`get_shader_location_attribute`](Self::get_shader_location_attribute)) and the matching
/// setter family. Uniform-value setters that move into [`Shader`]'s `impl` block
/// ([`set_shader_value`](Shader::set_shader_value),
/// [`set_shader_value_v`](Shader::set_shader_value_v),
/// [`set_shader_value_matrix`](Shader::set_shader_value_matrix),
/// [`set_shader_value_texture`](Shader::set_shader_value_texture)) require `&mut Shader`
/// directly.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
/// use raylib::core::shaders::RaylibShader;
///
/// let (mut rl, thread) = raylib::init().size(640, 480).title("shader").build();
/// let mut shader = rl.load_shader(&thread, None, Some("assets/tint.fs"));
/// let loc = shader.get_shader_location("u_tint");
/// shader.set_shader_value(loc, Vector3::new(1.0, 0.5, 0.0));
/// ```
///
/// # See also
///
/// - [`Shader`] — owning shader handle.
/// - [`ShaderV`] — values that can be uploaded as uniforms.
pub trait RaylibShader: AsRef<ffi::Shader> + crate::core::AsRawMut<ffi::Shader> {
    /// Shader locations array (RL_MAX_SHADER_LOCATIONS)
    #[inline]
    #[must_use]
    fn locs(&self) -> &[i32] {
        unsafe { std::slice::from_raw_parts(self.as_ref().locs, 32) }
    }

    /// Shader locations array (RL_MAX_SHADER_LOCATIONS)
    #[inline]
    #[must_use]
    fn locs_mut(&mut self) -> &mut [i32] {
        // SAFETY: raw mut access: this method upholds the wrapper invariants itself
        // (reads the locs pointer to build a fixed-length slice — does not corrupt the pointer).
        unsafe { std::slice::from_raw_parts_mut(self.as_raw_mut().locs, 32) }
    }

    /// Gets shader uniform location by name.
    #[inline]
    #[must_use]
    fn get_shader_location(&self, uniform_name: &str) -> i32 {
        let c_uniform_name = CString::new(uniform_name).unwrap();
        unsafe { ffi::GetShaderLocation(*self.as_ref(), c_uniform_name.as_ptr()) }
    }

    /// Gets shader attribute location by name.
    #[inline]
    #[must_use]
    fn get_shader_location_attribute(&self, attribute_name: &str) -> i32 {
        let c_attribute_name = CString::new(attribute_name).unwrap();
        unsafe { ffi::GetShaderLocationAttrib(*self.as_ref(), c_attribute_name.as_ptr()) }
    }

    /// Sets shader uniform value
    #[inline]
    fn set_shader_value<S: ShaderV>(&mut self, uniform_loc: i32, value: S) {
        // SAFETY: SetShaderValue uploads a uniform; does not corrupt the locs pointer
        // (the only trusted field) — invariants upheld.
        unsafe {
            ffi::SetShaderValue(
                *self.as_raw_mut(),
                uniform_loc,
                value.value(),
                (S::UNIFORM_TYPE as u32) as i32,
            );
        }
    }

    /// Set shader uniform value vector
    #[inline]
    fn set_shader_value_v<S: ShaderV>(&mut self, uniform_loc: i32, value: &[S]) {
        // SAFETY: SetShaderValueV uploads a uniform array; does not corrupt the locs pointer
        // (the only trusted field) — invariants upheld.
        unsafe {
            ffi::SetShaderValueV(
                *self.as_raw_mut(),
                uniform_loc,
                value.as_ptr() as *const ::std::os::raw::c_void,
                (S::UNIFORM_TYPE as u32) as i32,
                value.len() as i32,
            );
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    fn set_shader_value_matrix(&mut self, uniform_loc: i32, mat: impl Into<Matrix>) {
        // SAFETY: SetShaderValueMatrix uploads a matrix uniform; does not corrupt the locs pointer
        // (the only trusted field) — invariants upheld.
        unsafe {
            ffi::SetShaderValueMatrix(*self.as_raw_mut(), uniform_loc, mat.into());
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    fn set_shader_value_texture(&mut self, uniform_loc: i32, texture: impl AsRef<ffi::Texture2D>) {
        // SAFETY: SetShaderValueTexture uploads a texture uniform; does not corrupt the locs pointer
        // (the only trusted field) — invariants upheld.
        unsafe {
            ffi::SetShaderValueTexture(*self.as_raw_mut(), uniform_loc, *texture.as_ref());
        }
    }
}
