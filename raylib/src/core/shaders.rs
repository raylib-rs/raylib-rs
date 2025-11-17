//! Code for the safe manipulation of shaders

use crate::consts::ShaderUniformDataType;
use crate::core::math::Matrix;
use crate::core::math::{Vector2, Vector3, Vector4};
use crate::core::{RaylibHandle, RaylibThread};
use crate::{MintMatrix, ffi};
use std::ffi::CString;
use std::os::raw::c_void;

fn no_drop<T>(_thing: T) {}
make_thick_wrapper! {
    pub struct Shader {
        id: u32,
        locs: *mut i32,
    }
    weak = WeakShader,
    raw = ffi::Shader,
    drop = ffi::UnloadShader
}

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

        unsafe { Shader::from_raw_unchecked(ffi::LoadShader(vs, fs)) }
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

        unsafe { Shader::from_raw_unchecked(ffi::LoadShaderFromMemory(vs, fs)) }
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
    #[must_use]
    #[inline]
    pub fn get_matrix_modelview(&self) -> Matrix {
        unsafe { ffi::rlGetMatrixModelview().into() }
    }

    /// Gets internal projection matrix.
    #[inline]
    #[must_use]
    pub fn get_matrix_projection(&self) -> Matrix {
        unsafe { ffi::rlGetMatrixProjection().into() }
    }
    #[inline]
    #[must_use]
    /// Get default shader. Modifying it modifies everything that uses that shader
    pub fn get_shader_default() -> WeakShader {
        unsafe {
            Shader::from_raw_unchecked(ffi::Shader {
                id: ffi::rlGetShaderIdDefault(),
                locs: ffi::rlGetShaderLocsDefault(),
            })
            .make_weak()
        }
    }
}

pub trait ShaderV {
    const UNIFORM_TYPE: ShaderUniformDataType;
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
    /// Check if shader is valid
    #[inline]
    #[must_use]
    pub fn is_shader_valid(&self) -> bool {
        unsafe { ffi::IsShaderValid(self.clone_raw()) }
    }

    /// Sets shader uniform value
    #[inline]
    pub fn set_shader_value<S: ShaderV>(&mut self, uniform_loc: i32, value: S) {
        unsafe {
            ffi::SetShaderValue(
                self.clone_raw(),
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
                self.clone_raw(),
                uniform_loc,
                value.as_ptr() as *const ::std::os::raw::c_void,
                (S::UNIFORM_TYPE as u32) as i32,
                value.len() as i32,
            );
        }
    }

    /// Sets shader uniform value (matrix 4x4).
    #[inline]
    pub fn set_shader_value_matrix(&mut self, uniform_loc: i32, mat: impl Into<MintMatrix>) {
        unsafe {
            ffi::SetShaderValueMatrix(self.clone_raw(), uniform_loc, mat.into());
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
            ffi::SetShaderValueTexture(self.clone_raw(), uniform_loc, *texture.as_ref());
        }
    }

    /// Shader locations array (RL_MAX_SHADER_LOCATIONS)
    #[inline]
    #[must_use]
    fn locs(&self) -> &[i32] {
        unsafe { std::slice::from_raw_parts(self.locs, 32) }
    }

    /// Shader locations array (RL_MAX_SHADER_LOCATIONS)
    #[inline]
    #[must_use]
    fn locs_mut(&mut self) -> &mut [i32] {
        unsafe { std::slice::from_raw_parts_mut(self.locs, 32) }
    }

    /// Gets shader uniform location by name.
    #[inline]
    #[must_use]
    fn get_shader_location(&self, uniform_name: &str) -> i32 {
        let c_uniform_name = CString::new(uniform_name).unwrap();
        unsafe { ffi::GetShaderLocation(self.clone_raw(), c_uniform_name.as_ptr()) }
    }

    /// Gets shader attribute location by name.
    #[inline]
    #[must_use]
    fn get_shader_location_attribute(&self, attribute_name: &str) -> i32 {
        let c_attribute_name = CString::new(attribute_name).unwrap();
        unsafe { ffi::GetShaderLocationAttrib(self.clone_raw(), c_attribute_name.as_ptr()) }
    }
}
