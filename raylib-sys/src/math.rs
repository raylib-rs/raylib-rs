#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Quaternion. C aliases this to Vector4 (`typedef Vector4 Quaternion`); we use a
/// distinct, layout-identical #[repr(C)] struct so it has its own method namespace.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

// Zero-cost interchange with the FFI Vector4 that raymath's Quaternion* fns actually take/return.
impl From<crate::Vector4> for Quaternion {
    #[inline]
    fn from(v: crate::Vector4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}
impl From<Quaternion> for crate::Vector4 {
    #[inline]
    fn from(q: Quaternion) -> Self {
        crate::Vector4 {
            x: q.x,
            y: q.y,
            z: q.z,
            w: q.w,
        }
    }
}

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
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
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
    pub fn check_collision_circle_rec(
        &self,
        center: impl Into<crate::Vector2>,
        radius: f32,
    ) -> bool {
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
    pub fn check_collision_point_rec(&self, point: impl Into<crate::Vector2>) -> bool {
        unsafe { crate::CheckCollisionPointRec(point.into(), *self) }
    }
}
