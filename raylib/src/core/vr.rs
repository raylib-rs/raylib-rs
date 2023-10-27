//! Vr related functions
use crate::{core::RaylibHandle, ffi, make_thin_wrapper};

make_thin_wrapper!(
    VrStereoConfig,
    ffi::VrStereoConfig,
    ffi::UnloadVrStereoConfig
);

impl RaylibHandle {
    pub fn load_vr_stereo_config(&self, device: ffi::VrDeviceInfo) -> VrStereoConfig {
        VrStereoConfig(unsafe { ffi::LoadVrStereoConfig(device) })
    }
}
