//! Code for the safe manipulation of shaders

use crate::consts::ShaderUniformDataType;
use crate::core::math::Matrix;
use crate::core::math::{Vector2, Vector3, Vector4};
use crate::core::{RaylibHandle, RaylibThread};
use crate::{MintMatrix, ffi};
use std::ffi::CString;
use std::os::raw::c_void;

/// Maximum number of shader locations supported
/// (normally set by `config.h`, 32 by default)
pub const RL_MAX_SHADER_LOCATIONS: usize = 32;

fn no_drop<T>(_thing: T) {}
make_thin_wrapper!(
    /// Shader
    Shader,
    ffi::Shader,
    ffi::UnloadShader
);
make_thin_wrapper!(
    /// Unowned version of [`Shader`] that does not free the resource when dropped
    WeakShader,
    ffi::Shader,
    no_drop
);

// #[cfg(feature = "nightly")]
// impl !Send for Shader {}
// #[cfg(feature = "nightly")]
// unsafe impl Sync for Shader {}

impl RaylibHandle {
    /// Loads a custom shader and binds default locations.
    ///
    /// # Panics
    ///
    /// This method will panic if `vs_filename` or `fs_filename` contains an internal 0 byte.
    #[must_use]
    pub fn load_shader(
        &mut self,
        _: &RaylibThread,
        vs_filename: Option<&str>,
        fs_filename: Option<&str>,
    ) -> Shader {
        let vs_c_filename = vs_filename
            .map(|f| CString::new(f).expect("vs_filename should not contain an internal 0 byte"));
        let fs_c_filename = fs_filename
            .map(|f| CString::new(f).expect("fs_filename should not contain an internal 0 byte"));

        let vs = vs_c_filename
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());
        let fs = fs_c_filename
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());

        Shader(unsafe { ffi::LoadShader(vs, fs) })
    }

    /// Loads shader from code strings and binds default locations.
    ///
    /// # Panics
    ///
    /// This method will panic if `vs_filename` or `fs_filename` contains an internal 0 byte.
    #[must_use]
    pub fn load_shader_from_memory(
        &mut self,
        _: &RaylibThread,
        vs_code: Option<&str>,
        fs_code: Option<&str>,
    ) -> Shader {
        let vs_c_code = vs_code
            .map(|f| CString::new(f).expect("vs_code should not contain an internal 0 byte"));
        let fs_c_code = fs_code
            .map(|f| CString::new(f).expect("fs_code should not contain an internal 0 byte"));

        let vs = vs_c_code
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());
        let fs = fs_c_code
            .as_ref()
            .map_or_else(std::ptr::null, |s| s.as_ptr());

        Shader(unsafe { ffi::LoadShaderFromMemory(vs, fs) })
    }

    /// Sets a custom projection matrix (replaces internal projection matrix).
    #[inline]
    pub fn set_matrix_projection(&mut self, _: &RaylibThread, proj: impl Into<MintMatrix>) {
        unsafe {
            ffi::rlSetMatrixProjection(proj.into());
        }
    }

    /// Sets a custom modelview matrix (replaces internal modelview matrix).
    #[inline]
    pub fn set_matrix_modelview(&mut self, _: &RaylibThread, view: impl Into<MintMatrix>) {
        unsafe {
            ffi::rlSetMatrixModelview(view.into());
        }
    }

    /// Gets internal modelview matrix.
    #[inline]
    #[must_use]
    pub fn get_matrix_modelview(&self) -> Matrix {
        unsafe { ffi::rlGetMatrixModelview().into() }
    }

    /// Gets internal projection matrix.
    #[inline]
    #[must_use]
    pub fn get_matrix_projection(&self) -> Matrix {
        unsafe { ffi::rlGetMatrixProjection().into() }
    }

    /// Get default shader. Modifying it modifies everything that uses that shader
    #[inline]
    #[must_use]
    pub fn get_shader_default() -> WeakShader {
        WeakShader(ffi::Shader {
            id: unsafe { ffi::rlGetShaderIdDefault() },
            locs: unsafe { ffi::rlGetShaderLocsDefault() },
        })
    }
}

/// Types that can be sent to the GPU as uniform values for shaders.
///
/// # Safety
///
/// The implementor must ensure that [`ShaderV::UNIFORM_TYPE`] accurately describes how
/// `Self` should be interpreted by the GPU.
pub unsafe trait ShaderV: Copy {
    /// Enumerator describing the format [`ShaderV::value`]'s return can/should be interpreted by the GPU.
    /// This involves the size, alignment, and information stored by the type.
    const UNIFORM_TYPE: ShaderUniformDataType;

    /// Convert a reference to `self` into a void pointer that will have its data sent to the GPU.
    fn value(&self) -> &c_void;
}
#[allow(clippy::enum_glob_use, reason = "variants are prefixed")]
use ShaderUniformDataType::*;

unsafe impl ShaderV for f32 {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_FLOAT;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

unsafe impl ShaderV for Vector2 {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_VEC2;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

unsafe impl ShaderV for Vector3 {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_VEC3;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

unsafe impl ShaderV for Vector4 {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_VEC4;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

unsafe impl ShaderV for i32 {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_INT;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

unsafe impl ShaderV for [i32; 2] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_IVEC2;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

unsafe impl ShaderV for [i32; 3] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_IVEC3;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

unsafe impl ShaderV for [i32; 4] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_IVEC4;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

unsafe impl ShaderV for [f32; 2] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_VEC2;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

unsafe impl ShaderV for [f32; 3] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_VEC3;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

unsafe impl ShaderV for [f32; 4] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_VEC4;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

unsafe impl ShaderV for &[i32] {
    const UNIFORM_TYPE: ShaderUniformDataType = SHADER_UNIFORM_SAMPLER2D;
    #[inline]
    fn value(&self) -> &c_void {
        // SAFETY: A reference cast to a pointer is still a reference
        unsafe { &*self.as_ptr().cast() }
    }
}

impl Shader {
    /// Convert `self` to its weak form, allowing it to be shared by multiple containers on the condition
    /// that it is only unloaded once, manually.
    ///
    /// # Safety
    ///
    /// Must manually free memory by calling the proper unload function.
    /// Even if the return implements [`Copy`], exactly one instance should be unloaded to avoid double-free,
    /// and copies must not be used after being unloaded to avoid use-after-free.
    #[inline]
    #[must_use]
    pub const unsafe fn make_weak(self) -> WeakShader {
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
                std::ptr::from_ref(value.value()),
                S::UNIFORM_TYPE as i32,
            );
        }
    }

    /// Set shader uniform value vector
    ///
    /// # Panics
    ///
    /// This method will panic if `value` has a length greater than [`i32::MAX`].
    #[inline]
    pub fn set_shader_value_v<S: ShaderV>(&mut self, uniform_loc: i32, value: &[S]) {
        unsafe {
            ffi::SetShaderValueV(
                self.0,
                uniform_loc,
                value.as_ptr().cast(),
                S::UNIFORM_TYPE as i32,
                value
                    .len()
                    .try_into()
                    .expect("value should not exceed i32::MAX elements"),
            );
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    pub fn set_shader_value_matrix(&mut self, uniform_loc: i32, mat: impl Into<MintMatrix>) {
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

/// [`Shader`] accessors and helper methods.
pub trait RaylibShader {
    /// Shader locations array ([`RL_MAX_SHADER_LOCATIONS`])
    #[inline]
    #[must_use]
    fn locs(&self) -> &[i32; RL_MAX_SHADER_LOCATIONS]
    where
        Self: AsRef<ffi::Shader>,
    {
        unsafe { &*self.as_ref().locs.cast() }
    }

    /// Shader locations array ([`RL_MAX_SHADER_LOCATIONS`])
    #[inline]
    #[must_use]
    fn locs_mut(&mut self) -> &mut [i32; RL_MAX_SHADER_LOCATIONS]
    where
        Self: AsMut<ffi::Shader>,
    {
        unsafe { &mut *self.as_mut().locs.cast() }
    }

    /// Gets shader uniform location by name.
    ///
    /// # Panics
    ///
    /// This method will panic if `uniform_name` contains an internal 0 byte.
    #[inline]
    #[must_use]
    fn get_shader_location(&self, uniform_name: &str) -> i32
    where
        Self: AsRef<ffi::Shader>,
    {
        let c_uniform_name =
            CString::new(uniform_name).expect("uniform_name should not contain an internal 0 byte");
        unsafe { ffi::GetShaderLocation(*self.as_ref(), c_uniform_name.as_ptr()) }
    }

    /// Gets shader attribute location by name.
    ///
    /// # Panics
    ///
    /// This method will panic if `attribute_name` contains an internal 0 byte.
    #[inline]
    #[must_use]
    fn get_shader_location_attribute(&self, attribute_name: &str) -> i32
    where
        Self: AsRef<ffi::Shader>,
    {
        let c_attribute_name = CString::new(attribute_name)
            .expect("attribute_name should not contain an internal 0 byte");
        unsafe { ffi::GetShaderLocationAttrib(*self.as_ref(), c_attribute_name.as_ptr()) }
    }

    /// Sets shader uniform value
    #[inline]
    fn set_shader_value<S: ShaderV>(&mut self, uniform_loc: i32, value: S)
    where
        Self: AsMut<ffi::Shader>,
    {
        unsafe {
            ffi::SetShaderValue(
                *self.as_mut(),
                uniform_loc,
                value.value(),
                S::UNIFORM_TYPE as i32,
            );
        }
    }

    /// Set shader uniform value vector
    ///
    /// # Panics
    ///
    /// This method will panic if `value` has a length greater than [`i32::MAX`].
    #[inline]
    fn set_shader_value_v<S: ShaderV>(&mut self, uniform_loc: i32, value: &[S])
    where
        Self: AsMut<ffi::Shader>,
    {
        unsafe {
            ffi::SetShaderValueV(
                *self.as_mut(),
                uniform_loc,
                value.as_ptr().cast(),
                S::UNIFORM_TYPE as i32,
                value
                    .len()
                    .try_into()
                    .expect("value should not exceed i32::MAX elements"),
            );
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    fn set_shader_value_matrix(&mut self, uniform_loc: i32, mat: impl Into<MintMatrix>)
    where
        Self: AsMut<ffi::Shader>,
    {
        unsafe {
            ffi::SetShaderValueMatrix(*self.as_mut(), uniform_loc, mat.into());
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    fn set_shader_value_texture(&mut self, uniform_loc: i32, texture: impl AsRef<ffi::Texture2D>)
    where
        Self: AsMut<ffi::Shader>,
    {
        unsafe {
            ffi::SetShaderValueTexture(*self.as_mut(), uniform_loc, *texture.as_ref());
        }
    }
}
