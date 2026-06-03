#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A unit quaternion used for 3D rotations.
///
/// C aliases `Quaternion` to `Vector4` (`typedef Vector4 Quaternion`); raylib-rs
/// uses a **distinct** `#[repr(C)]` struct so the two types have their own method
/// namespaces. The layout is identical (`x`, `y`, `z`, `w` as `f32`), and
/// zero-cost conversions are provided:
/// - `Quaternion::from(v: Vector4)` — convert from a raw four-component vector.
/// - `Vector4::from(q: Quaternion)` — convert back.
///
/// Build a `Quaternion` with one of the constructor functions:
/// [`from_axis_angle`](Quaternion::from_axis_angle),
/// [`from_euler`](Quaternion::from_euler),
/// [`from_matrix`](Quaternion::from_matrix),
/// or [`identity`](Quaternion::identity).
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

/// An axis-aligned rectangle defined by its top-left corner and dimensions.
///
/// `Rectangle` is a `#[repr(C)]` struct with four `f32` fields: `x` and `y` for the top-left
/// corner position, and `width` / `height` for the extents. It maps directly to raylib's
/// `Rectangle` type in C.
///
/// Collision and containment helpers are available as methods:
/// [`check_collision_recs`](Rectangle::check_collision_recs),
/// [`check_collision_point_rec`](Rectangle::check_collision_point_rec),
/// [`get_collision_rec`](Rectangle::get_collision_rec),
/// [`check_collision_circle_rec`](Rectangle::check_collision_circle_rec).
///
/// # Examples
///
/// ```rust
/// use raylib_sys::Rectangle;
///
/// let r1 = Rectangle::new(0.0, 0.0, 10.0, 10.0);
/// let r2 = Rectangle::new(5.0, 5.0, 10.0, 10.0);
/// let r3 = Rectangle::new(20.0, 20.0, 10.0, 10.0);
///
/// assert!(r1.check_collision_recs(r2));   // overlapping
/// assert!(!r1.check_collision_recs(r3));  // non-overlapping
/// ```
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
        // SAFETY: CheckCollisionRecs is a pure value-in/out raylib fn; both args are
        // #[repr(C)] Copy structs with no pointer aliasing or preconditions.
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
        // SAFETY: CheckCollisionCircleRec is a pure value-in/out raylib fn; all args are
        // #[repr(C)] Copy values (Vector2, f32, Rectangle) with no pointer aliasing or preconditions.
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
        self.check_collision_recs(other).then(|| {
            // SAFETY: GetCollisionRec is a pure value-in/out raylib fn; both args are
            // #[repr(C)] Copy structs with no pointer aliasing or preconditions.
            unsafe { crate::GetCollisionRec(*self, other) }
        })
    }

    /// Checks if point is inside rectangle.
    #[inline]
    #[must_use]
    pub fn check_collision_point_rec(&self, point: impl Into<crate::Vector2>) -> bool {
        // SAFETY: CheckCollisionPointRec is a pure value-in/out raylib fn; both args are
        // #[repr(C)] Copy values (Vector2, Rectangle) with no pointer aliasing or preconditions.
        unsafe { crate::CheckCollisionPointRec(point.into(), *self) }
    }
}
