#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rectangle {
    /// Creates a new rectangle from position and size.
    #[must_use]
    #[inline]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a rectangle from position and size vectors.
    #[must_use]
    #[inline]
    pub fn from_v2(pos: impl Into<Vector2>, dims: impl Into<Vector2>) -> Self {
        let pos: Vector2 = pos.into();
        let dims: Vector2 = dims.into();
        Self {
            x: pos.x,
            y: pos.y,
            width: dims.x,
            height: dims.y,
        }
    }

    /// Returns the position as a `Vector2`.
    /// Convert it to your math library types with .into()
    #[must_use]
    #[inline]
    pub fn pos(self) -> Vector2 {
        Vector2 {
            x: self.x,
            y: self.y,
        }
    }

    /// Returns the size as a `Vector2`.
    /// Convert it to your math library types with .into()
    #[must_use]
    #[inline]
    pub fn dims(self) -> Vector2 {
        Vector2 {
            x: self.width,
            y: self.height,
        }
    }

    /// Returns a copy with the given `x` and `y`.
    #[must_use]
    #[inline]
    pub fn with_xy(self, x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            width: self.width,
            height: self.height,
        }
    }

    /// Returns a copy with the given `width` and `height`.
    #[must_use]
    #[inline]
    pub fn with_wh(self, width: f32, height: f32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            width,
            height,
        }
    }
    /// Check collision between two rectangles
    #[inline]
    #[must_use]
    pub fn check_collision_recs(&self, other: Rectangle) -> bool {
        unsafe { crate::CheckCollisionRecs(*self, other) }
    }

    /// Checks collision between circle and rectangle.
    #[inline]
    #[must_use]
    pub fn check_collision_circle_rec(&self, center: impl Into<Vector2>, radius: f32) -> bool {
        unsafe { crate::CheckCollisionCircleRec(center.into(), radius, *self) }
    }

    /// Gets the overlap between two colliding rectangles.
    /// ```rust
    /// use raylib_sys::Rectangle;
    ///
    /// let r1 = Rectangle::new(0.0, 0.0, 10.0, 10.0);
    /// let r2 = Rectangle::new(20.0, 20.0, 10.0, 10.0);
    /// assert_eq!(None, r1.get_collision_rec(r2));
    /// assert_eq!(Some(r1), r1.get_collision_rec(r1));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_collision_rec(&self, other: Rectangle) -> Option<Rectangle> {
        self.check_collision_recs(other)
            .then(|| unsafe { crate::GetCollisionRec(*self, other) })
    }

    /// Checks if point is inside rectangle.
    #[inline]
    #[must_use]
    pub fn check_collision_point_rec(&self, point: impl Into<Vector2>) -> bool {
        unsafe { crate::CheckCollisionPointRec(point.into(), *self) }
    }
}

/// Trait for types that can be lossily converted to `f32`.
pub trait AsF32: Copy {
    fn as_f32(self) -> f32;
}

impl AsF32 for f32 {
    #[inline]
    fn as_f32(self) -> f32 {
        self
    }
}

impl AsF32 for f64 {
    #[inline]
    fn as_f32(self) -> f32 {
        self as f32
    }
}

impl AsF32 for i32 {
    #[inline]
    fn as_f32(self) -> f32 {
        self as f32
    }
}

/// Creates a `Rectangle` from values convertible to `f32`.
#[must_use]
#[inline]
pub fn rrect<T: AsF32>(x: T, y: T, width: T, height: T) -> Rectangle {
    Rectangle::new(x.as_f32(), y.as_f32(), width.as_f32(), height.as_f32())
}
