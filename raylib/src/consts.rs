//! Various constant enums to use with raylib
pub use crate::ffi;

pub use ffi::BlendMode;
pub use ffi::CameraMode;
pub use ffi::CameraProjection;
pub use ffi::ConfigFlags;
pub use ffi::CubemapLayout;
pub use ffi::DEG2RAD;
pub use ffi::GamepadAxis;
pub use ffi::GamepadButton;
pub use ffi::Gesture;
pub use ffi::KeyboardKey;
pub use ffi::MaterialMapIndex;
pub use ffi::MouseButton;
pub use ffi::NPatchLayout;
pub use ffi::PixelFormat;
pub use ffi::ShaderLocationIndex;
pub use ffi::ShaderUniformDataType;
pub use ffi::TextureFilter;
pub use ffi::TextureWrap;
pub use ffi::TraceLogLevel;
// TODO Fix when rlgl bindings are in
/// Compile-time upper bound on the number of material maps a [`Material`](crate::core::models::Material) can hold.
///
/// Mirrors raylib's `MAX_MATERIAL_MAPS` (`config.h`) and indexes the `Material::maps` array. The
/// stock entries are the diffuse/specular/normal/etc. slots enumerated by
/// [`MaterialMapIndex`]; the remaining slots are free for user-defined channels.
///
/// # Examples
///
/// ```rust
/// use raylib::consts::MAX_MATERIAL_MAPS;
/// assert_eq!(MAX_MATERIAL_MAPS, 12);
/// ```
pub const MAX_MATERIAL_MAPS: u32 = 12;
/// Compile-time upper bound on the number of cached uniform/attribute locations per [`Shader`](crate::core::shaders::Shader).
///
/// Mirrors raylib's `RL_MAX_SHADER_LOCATIONS` (`rlgl.h`) and sizes the `Shader::locs` array
/// indexed by [`ShaderLocationIndex`]. Custom uniforms past the built-in slots may be stored
/// up to this limit before raylib begins overwriting earlier entries.
///
/// # Examples
///
/// ```rust
/// use raylib::consts::MAX_SHADER_LOCATIONS;
/// assert_eq!(MAX_SHADER_LOCATIONS, 32);
/// ```
pub const MAX_SHADER_LOCATIONS: u32 = 32;

pub use ffi::MouseCursor;
pub use ffi::PI;
pub use ffi::RAD2DEG;
#[cfg(not(feature = "nobuild"))]
pub use ffi::{
    GuiCheckBoxProperty, GuiColorPickerProperty, GuiComboBoxProperty, GuiControl,
    GuiControlProperty, GuiDefaultProperty, GuiDropdownBoxProperty, GuiIconName,
    GuiListViewProperty, GuiProgressBarProperty, GuiScrollBarProperty, GuiSliderProperty, GuiState,
    GuiTextAlignment, GuiTextAlignmentVertical, GuiTextBoxProperty, GuiTextWrapMode,
    GuiToggleProperty, GuiValueBoxProperty,
};
