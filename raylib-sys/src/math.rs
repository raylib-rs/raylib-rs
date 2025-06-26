// pub use glam;
// pub type Vector2 = glam::f32::Vec2;
// pub type Vector3 = glam::f32::Vec3;
// pub type Vector4 = glam::f32::Vec4;
// glam Matrix and Quat are not align compat with raylib so we write our own in math.rs
// pub type Matrix = glam::Mat4;
// pub type Quaternion = glam::Quat;
pub use mint;
pub type Vector2 = mint::Vector2<f32>;
pub type Vector3 = mint::Vector3<f32>;
pub type Vector4 = mint::Vector4<f32>;
pub type Matrix = mint::RowMatrix4<f32>;
pub type Quaternion = mint::Vector4<f32>; // raylib does this same alias so we match it
