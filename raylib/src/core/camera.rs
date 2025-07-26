//! Utility code for using Raylib [`Camera3D`] and [`Camera2D`]

use crate::MintVec3;
use crate::ffi;
use crate::math::{Vector2, Vector3};

use super::math::Matrix;

/// [`Camera2D`], defines position/orientation in 2d space
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Camera2D {
    /// Camera offset (displacement from target)
    pub offset: Vector2,
    /// Camera target (rotation and zoom origin)
    pub target: Vector2,
    /// Camera rotation in degrees
    pub rotation: f32,
    /// Camera zoom (scaling), should be 1.0 by default
    pub zoom: f32,
}
impl Camera2D {
    #[must_use]
    #[inline]
    #[allow(dead_code)]
    fn get_camera_matrix_2d(camera: impl Into<ffi::Camera2D>) -> Matrix {
        unsafe { ffi::GetCameraMatrix2D(camera.into()).into() }
    }
}

impl From<ffi::Camera2D> for Camera2D {
    fn from(v: ffi::Camera2D) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<Camera2D> for ffi::Camera2D {
    fn from(v: Camera2D) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&Camera2D> for ffi::Camera2D {
    fn from(v: &Camera2D) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}

/// Camera, defines position/orientation in 3d space
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Camera3D {
    /// Camera position
    pub position: Vector3,
    /// Camera target it looks-at
    pub target: Vector3,
    /// Camera up vector (rotation over its axis)
    pub up: Vector3,
    /// Camera field-of-view aperture in Y (degrees) in perspective, used as near plane width in orthographic
    pub fovy: f32,
    /// Camera projection: [`ffi::CameraProjection::CAMERA_PERSPECTIVE`] or [`ffi::CameraProjection::CAMERA_ORTHOGRAPHIC`]
    pub projection: ffi::CameraProjection,
}
/// [`Camera`] type fallback, defaults to [`Camera3D`]
pub type Camera = Camera3D;

impl From<ffi::Camera3D> for Camera3D {
    fn from(v: ffi::Camera3D) -> Camera3D {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<Camera3D> for ffi::Camera3D {
    fn from(v: Camera3D) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&Camera3D> for ffi::Camera3D {
    fn from(v: &Camera3D) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}

impl From<&mut Camera3D> for ffi::Camera3D {
    fn from(v: &mut Camera3D) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}
impl From<&mut Camera3D> for *mut ffi::Camera3D {
    fn from(val: &mut Camera3D) -> Self {
        std::ptr::from_mut(val).cast()
    }
}

impl Camera3D {
    #[must_use]
    #[inline]
    pub const fn camera_type(&self) -> ffi::CameraProjection {
        self.projection
    }

    /// Create a perspective camera.
    /// fovy is in degrees
    #[must_use]
    #[inline]
    pub const fn perspective(
        position: Vector3,
        target: Vector3,
        up: Vector3,
        fovy: f32,
    ) -> Camera3D {
        Camera3D {
            position,
            target,
            up,
            fovy,
            projection: ffi::CameraProjection::CAMERA_PERSPECTIVE,
        }
    }

    /// Create a orthographic camera.
    /// fovy is in degrees
    #[must_use]
    #[inline]
    pub const fn orthographic(
        position: Vector3,
        target: Vector3,
        up: Vector3,
        fovy: f32,
    ) -> Camera3D {
        let mut c = Self::perspective(position, target, up, fovy);
        c.projection = ffi::CameraProjection::CAMERA_ORTHOGRAPHIC;
        c
    }

    #[must_use]
    #[inline]
    pub fn forward(&self) -> Vector3 {
        unsafe { ffi::GetCameraForward(std::ptr::from_ref(self).cast_mut().cast()).into() }
    }

    #[must_use]
    #[inline]
    pub fn up(&self) -> Vector3 {
        unsafe { ffi::GetCameraUp(std::ptr::from_ref(self).cast_mut().cast()).into() }
    }

    #[inline]
    pub fn move_forward(&mut self, distance: f32, in_world_plane: bool) {
        unsafe { ffi::CameraMoveForward(self.into(), distance, in_world_plane) }
    }

    #[inline]
    pub fn move_up(&mut self, distance: f32) {
        unsafe { ffi::CameraMoveUp(self.into(), distance) }
    }

    #[inline]
    pub fn move_right(&mut self, distance: f32, in_world_plane: bool) {
        unsafe { ffi::CameraMoveRight(self.into(), distance, in_world_plane) }
    }

    #[inline]
    pub fn move_to_target(&mut self, delta: f32) {
        unsafe { ffi::CameraMoveToTarget(self.into(), delta) }
    }

    #[inline]
    pub fn yaw(&mut self, angle: f32, rotate_around_target: bool) {
        unsafe { ffi::CameraYaw(self.into(), angle, rotate_around_target) }
    }

    #[inline]
    pub fn pitch(
        &mut self,
        angle: f32,
        lock_view: bool,
        rotate_around_target: bool,
        rotate_up: bool,
    ) {
        unsafe {
            ffi::CameraPitch(
                self.into(),
                angle,
                lock_view,
                rotate_around_target,
                rotate_up,
            );
        }
    }

    #[inline]
    pub fn roll(&mut self, angle: f32) {
        unsafe { ffi::CameraRoll(self.into(), angle) }
    }

    #[must_use]
    #[inline]
    pub fn view_matrix(&self) -> Matrix {
        unsafe { ffi::GetCameraViewMatrix(std::ptr::from_ref(self).cast_mut().cast()).into() }
    }
    #[must_use]
    #[inline]
    pub fn projection_matrix(&self, aspect: f32) -> Matrix {
        unsafe {
            ffi::GetCameraProjectionMatrix(std::ptr::from_ref(self).cast_mut().cast(), aspect)
                .into()
        }
    }
    /// Updates camera position for selected mode.
    #[inline]
    pub fn update_camera(&mut self, mode: ffi::CameraMode) {
        unsafe { ffi::UpdateCamera(self.into(), mode as i32) }
    }
    #[inline]
    pub fn update_camera_pro(
        &mut self,
        movement: impl Into<MintVec3>,
        rotation: impl Into<MintVec3>,
        zoom: f32,
    ) {
        unsafe { ffi::UpdateCameraPro(self.into(), movement.into(), rotation.into(), zoom) }
    }
}
