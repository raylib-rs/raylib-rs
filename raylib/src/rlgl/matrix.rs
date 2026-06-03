use crate::core::drawing::RaylibDraw;
use crate::ffi;

/// Target matrix for `rl_matrix_mode`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixMode {
    /// Model-view matrix (`RL_MODELVIEW`).
    ModelView = ffi::RL_MODELVIEW as i32,
    /// Projection matrix (`RL_PROJECTION`).
    Projection = ffi::RL_PROJECTION as i32,
    /// Texture matrix (`RL_TEXTURE`).
    Texture = ffi::RL_TEXTURE as i32,
}

/// RAII guard for a pushed matrix: `rlPushMatrix` on creation, `rlPopMatrix` on
/// drop. Deref's to the draw handle so you can keep drawing within the pushed
/// transform. Transform methods are on the
/// [`RaylibRlgl`](crate::rlgl::RaylibRlgl) trait.
pub struct RlMatrix<'a, T: RaylibDraw>(&'a mut T);

impl<T: RaylibDraw> Drop for RlMatrix<'_, T> {
    fn drop(&mut self) {
        // SAFETY: paired with the `rlPushMatrix` that created this guard.
        unsafe { ffi::rlPopMatrix() }
    }
}

impl<T: RaylibDraw> std::ops::Deref for RlMatrix<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: RaylibDraw> std::ops::DerefMut for RlMatrix<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T: RaylibDraw> RlMatrix<'a, T> {
    pub(crate) fn new(parent: &'a mut T) -> Self {
        RlMatrix(parent)
    }
}
