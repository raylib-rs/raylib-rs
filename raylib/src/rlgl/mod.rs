//! Safe wrappers for rlgl, raylib's immediate-mode OpenGL abstraction layer
//! (`rlgl.h`). rlgl sits directly beneath raylib's higher-level drawing
//! functions and supports OpenGL 3.3, OpenGL 2.1, OpenGL ES 2.0, and the
//! software-renderer backend — the same source compiles for all back-ends.
//!
//! This module exposes the surface most useful for custom rendering:
//!
//! - **Matrix stack** — [`rl_push_matrix`](RaylibRlgl::rl_push_matrix) returns
//!   an [`RlMatrix`] RAII guard; `Drop` pops the matrix, so the stack is always
//!   balanced even on early return.
//! - **Immediate-mode vertex streams** —
//!   [`rl_begin`](RaylibRlgl::rl_begin) / [`rl_draw`](RaylibRlgl::rl_draw) with
//!   an [`RlImmediate`] RAII guard that calls `rlEnd` on drop.
//! - **Render-state toggles** — depth test, back-face culling, etc.
//! - **Safe bind helpers** — [`rl_set_texture`](RaylibRlgl::rl_set_texture) and
//!   [`rl_enable_shader`](RaylibRlgl::rl_enable_shader) accept the crate's
//!   [`Texture2D`](crate::core::texture::Texture2D) /
//!   [`Shader`](crate::core::shaders::Shader) RAII handles instead of raw ids.
//!
//! GL-object *lifecycle* (create/destroy) stays with those safe types and raw
//! [`ffi`]; the full rlgl surface is still available there as a power-user
//! escape hatch.
//!
//! ## Coverage policy
//!
//! The safe surface deliberately covers the immediate-mode, matrix-stack, and
//! render-state slice of rlgl. The per-function disposition of the entire
//! rlgl FFI surface (wrapped / escape-hatch / future-work) is recorded in
//! `docs/superpowers/notes/databuf-mesh-testing-complete.md`, along with the
//! census script to regenerate it after a raylib bump.
//!
//! All entry points are on [`RaylibRlgl`], blanket-implemented for every draw
//! handle, so they are only callable inside a `begin_drawing` frame.

mod immediate;
mod matrix;

pub use immediate::{DrawMode, RlImmediate};
pub use matrix::{MatrixMode, RlMatrix};

use crate::core::drawing::RaylibDraw;
use crate::ffi;
use crate::math::Matrix;

/// Immediate-mode rlgl drawing, available on every draw handle. See the module docs.
pub trait RaylibRlgl: RaylibDraw + Sized {
    // ── Matrix stack ────────────────────────────────────────────────────────

    /// Push the current matrix onto the stack; the returned guard pops it on drop.
    /// Apply transforms (`rl_translatef`/`rl_rotatef`/`rl_scalef`) through the guard.
    #[inline]
    fn rl_push_matrix(&mut self) -> RlMatrix<'_, Self> {
        // SAFETY: the returned RlMatrix's Drop calls the matching rlPopMatrix.
        unsafe { ffi::rlPushMatrix() };
        RlMatrix::new(self)
    }

    /// Reset the current matrix to identity.
    #[inline]
    fn rl_load_identity(&mut self) {
        // SAFETY: rlLoadIdentity is an unconditional state mutation; no preconditions.
        unsafe { ffi::rlLoadIdentity() }
    }

    /// Multiply the current matrix by a translation.
    #[inline]
    fn rl_translatef(&mut self, x: f32, y: f32, z: f32) {
        // SAFETY: pure rlgl state call; no preconditions.
        unsafe { ffi::rlTranslatef(x, y, z) }
    }

    /// Multiply the current matrix by a rotation of `angle` degrees about (x,y,z).
    #[inline]
    fn rl_rotatef(&mut self, angle: f32, x: f32, y: f32, z: f32) {
        // SAFETY: pure rlgl state call; no preconditions.
        unsafe { ffi::rlRotatef(angle, x, y, z) }
    }

    /// Multiply the current matrix by a scale.
    #[inline]
    fn rl_scalef(&mut self, x: f32, y: f32, z: f32) {
        // SAFETY: pure rlgl state call; no preconditions.
        unsafe { ffi::rlScalef(x, y, z) }
    }

    /// Multiply the current matrix by `mat`.
    #[inline]
    fn rl_mult_matrixf(&mut self, mat: Matrix) {
        // SAFETY: ffi::Matrix is repr(C) with 16 contiguous f32 fields in the
        // column-major order rlMultMatrixf expects. Casting the Matrix address to
        // *const f32 is sound: the struct starts with m0 at offset 0 and is
        // exactly 16×f32 = 64 bytes (confirmed by bindgen size assertion).
        unsafe { ffi::rlMultMatrixf(&mat as *const Matrix as *const f32) }
    }

    /// Choose which matrix subsequent operations affect.
    #[inline]
    fn rl_matrix_mode(&mut self, mode: MatrixMode) {
        // SAFETY: pure rlgl state call; mode is guaranteed valid by the MatrixMode enum.
        unsafe { ffi::rlMatrixMode(mode as i32) }
    }

    /// Multiply the current matrix by an orthographic projection.
    #[inline]
    fn rl_ortho(&mut self, left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) {
        // SAFETY: pure rlgl state call; no preconditions.
        unsafe { ffi::rlOrtho(left, right, bottom, top, near, far) }
    }

    /// Set the projection matrix directly.
    #[inline]
    fn rl_set_matrix_projection(&mut self, proj: Matrix) {
        // SAFETY: crate::math::Matrix is re-exported from ffi::Matrix (same type);
        // passed by value to rlSetMatrixProjection which expects ffi::Matrix.
        unsafe { ffi::rlSetMatrixProjection(proj) }
    }

    /// Set the model-view matrix directly.
    #[inline]
    fn rl_set_matrix_modelview(&mut self, view: Matrix) {
        // SAFETY: same identity as rl_set_matrix_projection.
        unsafe { ffi::rlSetMatrixModelview(view) }
    }

    // ── Immediate-mode vertex stream ─────────────────────────────────────────

    /// Begin an immediate-mode vertex stream; the returned guard ends it on drop.
    /// Emit vertices/colors through the guard. Prefer [`rl_draw`](RaylibRlgl::rl_draw).
    #[inline]
    fn rl_begin(&mut self, mode: DrawMode) -> RlImmediate<'_, Self> {
        // SAFETY: the returned RlImmediate's Drop calls the matching rlEnd.
        unsafe { ffi::rlBegin(mode as i32) };
        RlImmediate::new(self)
    }

    /// Run `body` inside an immediate-mode block, ending it afterwards (closure form).
    ///
    /// # Example
    /// ```no_run
    /// use raylib::prelude::*;
    /// fn frame(d: &mut RaylibDrawHandle) {
    ///     d.rl_draw(DrawMode::Triangles, |v| {
    ///         v.color4ub(Color::RED);
    ///         v.vertex2f(0.0, 0.0);
    ///         v.vertex2f(100.0, 0.0);
    ///         v.vertex2f(50.0, 100.0);
    ///     });
    /// }
    /// ```
    #[inline]
    fn rl_draw(&mut self, mode: DrawMode, body: impl FnOnce(&mut RlImmediate<'_, Self>)) {
        let mut v = self.rl_begin(mode);
        body(&mut v);
    }

    // ── Render-state toggles ─────────────────────────────────────────────────

    /// Enable depth testing.
    #[inline]
    fn rl_enable_depth_test(&mut self) {
        // SAFETY: unconditional rlgl state toggle; no preconditions.
        unsafe { ffi::rlEnableDepthTest() }
    }

    /// Disable depth testing.
    #[inline]
    fn rl_disable_depth_test(&mut self) {
        // SAFETY: unconditional rlgl state toggle; no preconditions.
        unsafe { ffi::rlDisableDepthTest() }
    }

    /// Enable back-face culling.
    #[inline]
    fn rl_enable_backface_culling(&mut self) {
        // SAFETY: unconditional rlgl state toggle; no preconditions.
        unsafe { ffi::rlEnableBackfaceCulling() }
    }

    /// Disable back-face culling.
    #[inline]
    fn rl_disable_backface_culling(&mut self) {
        // SAFETY: unconditional rlgl state toggle; no preconditions.
        unsafe { ffi::rlDisableBackfaceCulling() }
    }

    // ── Bind-safe-handle methods ─────────────────────────────────────────────

    /// Set the active texture for subsequent immediate-mode drawing.
    ///
    /// # Example
    /// ```no_run
    /// use raylib::prelude::*;
    /// fn frame(d: &mut RaylibDrawHandle, tex: &Texture2D) {
    ///     d.rl_set_texture(tex);
    ///     d.rl_draw(DrawMode::Quads, |v| {
    ///         v.color4ub(Color::WHITE);
    ///         v.texcoord2f(0.0, 0.0); v.vertex2f(0.0, 0.0);
    ///         v.texcoord2f(1.0, 0.0); v.vertex2f(64.0, 0.0);
    ///         v.texcoord2f(1.0, 1.0); v.vertex2f(64.0, 64.0);
    ///         v.texcoord2f(0.0, 1.0); v.vertex2f(0.0, 64.0);
    ///     });
    ///     d.rl_disable_texture();
    /// }
    /// ```
    #[inline]
    fn rl_set_texture(&mut self, texture: &crate::core::texture::Texture2D) {
        // SAFETY: texture.id is a valid GPU texture id owned by the Texture2D RAII.
        unsafe { ffi::rlSetTexture(texture.id) }
    }

    /// Enable a texture (by binding the safe handle's GL id).
    #[inline]
    fn rl_enable_texture(&mut self, texture: &crate::core::texture::Texture2D) {
        // SAFETY: texture.id is a valid GPU texture id owned by the Texture2D RAII.
        unsafe { ffi::rlEnableTexture(texture.id) }
    }

    /// Disable the active texture.
    #[inline]
    fn rl_disable_texture(&mut self) {
        // SAFETY: unconditional rlgl state call; no preconditions.
        unsafe { ffi::rlDisableTexture() }
    }

    /// Select the active multitexture slot.
    #[inline]
    fn rl_active_texture_slot(&mut self, slot: i32) {
        // SAFETY: rlgl clamps out-of-range slots; no UB from an invalid slot value.
        unsafe { ffi::rlActiveTextureSlot(slot) }
    }

    /// Enable a shader program (by binding the safe handle's GL id).
    #[inline]
    fn rl_enable_shader(&mut self, shader: &crate::core::shaders::Shader) {
        // SAFETY: shader.id is a valid GPU shader id owned by the Shader RAII.
        unsafe { ffi::rlEnableShader(shader.id) }
    }

    /// Set the active shader and its uniform locations (from the safe handle).
    #[inline]
    fn rl_set_shader(&mut self, shader: &crate::core::shaders::Shader) {
        // SAFETY: shader.id is a valid GPU shader id; shader.locs is a non-null
        // pointer owned and initialized by the Shader RAII's load path.
        unsafe { ffi::rlSetShader(shader.id, shader.locs) }
    }
}

impl<D: RaylibDraw> RaylibRlgl for D {}
