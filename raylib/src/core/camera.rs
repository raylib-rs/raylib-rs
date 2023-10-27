//! Utility code for using Raylib [`Camera3D`] and [`Camera2D`]
use crate::{
    core::RaylibHandle,
    ffi::{self, Camera3D, CameraMode, Vector3},
};

impl RaylibHandle {
    /// Updates camera position for selected mode.
    #[inline]
    pub fn update_camera(&self, camera: &mut Camera3D, mode: CameraMode) {
        unsafe { ffi::UpdateCamera(camera, mode as i32) }
    }

    pub fn update_camera_pro(
        &self,
        camera: &mut Camera3D,
        movement: impl Into<Vector3>,
        rotation: impl Into<Vector3>,
        zoom: f32,
    ) {
        unsafe { ffi::UpdateCameraPro(camera, movement.into(), rotation.into(), zoom) }
    }
}
