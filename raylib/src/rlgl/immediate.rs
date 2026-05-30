use crate::core::drawing::RaylibDraw;
use crate::ffi;
use crate::ffi::Color;

/// Primitive type for an immediate-mode vertex stream (`rlBegin`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawMode {
    /// Line list (`RL_LINES`).
    Lines = ffi::RL_LINES as i32,
    /// Triangle list (`RL_TRIANGLES`).
    Triangles = ffi::RL_TRIANGLES as i32,
    /// Quad list (`RL_QUADS`).
    Quads = ffi::RL_QUADS as i32,
}

/// RAII guard for an immediate-mode vertex stream: `rlBegin` on creation,
/// `rlEnd` on drop. Vertex/color methods (Task 3) live here so they cannot be
/// called outside a `rlBegin`/`rlEnd` block.
pub struct RlImmediate<'a, T: RaylibDraw>(#[allow(dead_code)] &'a mut T);

impl<T: RaylibDraw> Drop for RlImmediate<'_, T> {
    fn drop(&mut self) {
        // SAFETY: paired with the `rlBegin` that created this guard; raylib's
        // rlgl tolerates rlEnd after rlBegin on the active batch.
        unsafe { ffi::rlEnd() }
    }
}

impl<'a, T: RaylibDraw> RlImmediate<'a, T> {
    pub(crate) fn new(parent: &'a mut T) -> Self {
        RlImmediate(parent)
    }

    /// Emit a 2D vertex.
    #[inline]
    pub fn vertex2f(&mut self, x: f32, y: f32) {
        unsafe { ffi::rlVertex2f(x, y) }
    }

    /// Emit a 3D vertex.
    #[inline]
    pub fn vertex3f(&mut self, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlVertex3f(x, y, z) }
    }

    /// Set the current vertex color (8-bit RGBA).
    #[inline]
    pub fn color4ub(&mut self, color: impl Into<Color>) {
        let c = color.into();
        unsafe { ffi::rlColor4ub(c.r, c.g, c.b, c.a) }
    }

    /// Set the current vertex color (float RGB).
    #[inline]
    pub fn color3f(&mut self, r: f32, g: f32, b: f32) {
        unsafe { ffi::rlColor3f(r, g, b) }
    }

    /// Set the current vertex color (float RGBA).
    #[inline]
    pub fn color4f(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { ffi::rlColor4f(r, g, b, a) }
    }

    /// Set the current texture coordinate.
    #[inline]
    pub fn texcoord2f(&mut self, x: f32, y: f32) {
        unsafe { ffi::rlTexCoord2f(x, y) }
    }

    /// Set the current normal vector.
    #[inline]
    pub fn normal3f(&mut self, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlNormal3f(x, y, z) }
    }
}
