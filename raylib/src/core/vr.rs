//! VR related functions
use crate::core::{RaylibHandle, RaylibThread};
use crate::ffi;

make_thin_wrapper!(
    /// [`VrStereoConfig`], VR stereo rendering configuration for simulator
    VrStereoConfig,
    ffi::VrStereoConfig,
    ffi::UnloadVrStereoConfig
);

/// [`VrDeviceInfo`], Head-Mounted-Display device parameters
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VrDeviceInfo {
    /// Horizontal resolution in pixels
    pub h_resolution: i32,
    /// Vertical resolution in pixels
    pub v_resolution: i32,
    /// Horizontal size in meters
    pub h_screen_size: f32,
    /// Vertical size in meters
    pub v_screen_size: f32,
    /// Distance between eye and display in meters
    pub eye_to_screen_distance: f32,
    /// Lens separation distance in meters
    pub lens_separation_distance: f32,
    /// IPD (distance between pupils) in meters
    pub interpupillary_distance: f32,
    /// Lens distortion constant parameters
    pub lens_distortion_values: [f32; 4],
    /// Chromatic aberration correction parameters
    pub chroma_ab_correction: [f32; 4],
}

impl From<ffi::VrDeviceInfo> for VrDeviceInfo {
    fn from(v: ffi::VrDeviceInfo) -> VrDeviceInfo {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<VrDeviceInfo> for ffi::VrDeviceInfo {
    fn from(v: VrDeviceInfo) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&VrDeviceInfo> for ffi::VrDeviceInfo {
    fn from(v: &VrDeviceInfo) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}

impl RaylibHandle {
    /// Load VR stereo config for VR simulator device parameters
    #[inline]
    #[must_use]
    pub fn load_vr_stereo_config(
        &mut self,
        _: &RaylibThread,
        device: impl Into<ffi::VrDeviceInfo>,
    ) -> VrStereoConfig {
        VrStereoConfig(unsafe { ffi::LoadVrStereoConfig(device.into()) })
    }
}
