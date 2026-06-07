use crate::{Matrix, Quaternion, Vector2, Vector3, Vector4};

// ─── Vector2 ─────────────────────────────────────────────────────────────────

impl Vector2 {
    /// The zero vector `(0, 0)`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// The vector `(1, 1)`.
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    /// Construct a new Vector2.
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero vector. (raymath `Vector2Zero`)
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        // SAFETY: Vector2Zero is a pure value-out raymath fn; no preconditions.
        unsafe { crate::Vector2Zero() }
    }

    /// Vector with both components set to 1. (raymath `Vector2One`)
    #[inline]
    #[must_use]
    pub fn one() -> Self {
        // SAFETY: Vector2One is a pure value-out raymath fn; no preconditions.
        unsafe { crate::Vector2One() }
    }

    /// Add scalar to both components. (raymath `Vector2AddValue`)
    #[inline]
    #[must_use]
    pub fn add_value(self, add: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2AddValue(self, add) }
    }

    /// Subtract scalar from both components. (raymath `Vector2SubtractValue`)
    #[inline]
    #[must_use]
    pub fn sub_value(self, sub: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2SubtractValue(self, sub) }
    }

    /// Vector length. (raymath `Vector2Length`)
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Length(self) }
    }

    /// Squared vector length. (raymath `Vector2LengthSqr`)
    #[inline]
    #[must_use]
    pub fn length_sqr(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2LengthSqr(self) }
    }

    /// Dot product. (raymath `Vector2DotProduct`)
    #[inline]
    #[must_use]
    pub fn dot(self, other: Vector2) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2DotProduct(self, other) }
    }

    /// Cross product (scalar). (raymath `Vector2CrossProduct`)
    #[inline]
    #[must_use]
    pub fn cross(self, other: Vector2) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2CrossProduct(self, other) }
    }

    /// Distance between two vectors. (raymath `Vector2Distance`)
    #[inline]
    #[must_use]
    pub fn distance(self, other: Vector2) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Distance(self, other) }
    }

    /// Squared distance between two vectors. (raymath `Vector2DistanceSqr`)
    #[inline]
    #[must_use]
    pub fn distance_sqr(self, other: Vector2) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2DistanceSqr(self, other) }
    }

    /// Angle between two vectors (radians). (raymath `Vector2Angle`)
    #[inline]
    #[must_use]
    pub fn angle(self, other: Vector2) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Angle(self, other) }
    }

    /// Angle of line defined by two points. (raymath `Vector2LineAngle`)
    #[inline]
    #[must_use]
    pub fn line_angle(start: Vector2, end: Vector2) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2LineAngle(start, end) }
    }

    /// Scale vector by scalar. (raymath `Vector2Scale`)
    #[inline]
    #[must_use]
    pub fn scale(self, scale: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Scale(self, scale) }
    }

    /// Component-wise multiply. (raymath `Vector2Multiply`)
    #[inline]
    #[must_use]
    pub fn multiply(self, other: Vector2) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Multiply(self, other) }
    }

    /// Negate vector. (raymath `Vector2Negate`)
    #[inline]
    #[must_use]
    pub fn negate(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Negate(self) }
    }

    /// Component-wise divide. (raymath `Vector2Divide`)
    #[inline]
    #[must_use]
    pub fn divide(self, other: Vector2) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Divide(self, other) }
    }

    /// Normalize vector. (raymath `Vector2Normalize`)
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Normalize(self) }
    }

    /// Transform by matrix. (raymath `Vector2Transform`)
    #[inline]
    #[must_use]
    pub fn transform(self, mat: Matrix) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Transform(self, mat) }
    }

    /// Linear interpolation. (raymath `Vector2Lerp`)
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Vector2, amount: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Lerp(self, other, amount) }
    }

    /// Reflect vector about normal. (raymath `Vector2Reflect`)
    #[inline]
    #[must_use]
    pub fn reflect(self, normal: Vector2) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Reflect(self, normal) }
    }

    /// Component-wise minimum. (raymath `Vector2Min`)
    #[inline]
    #[must_use]
    pub fn min(self, other: Vector2) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Min(self, other) }
    }

    /// Component-wise maximum. (raymath `Vector2Max`)
    #[inline]
    #[must_use]
    pub fn max(self, other: Vector2) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Max(self, other) }
    }

    /// Rotate by angle (radians). (raymath `Vector2Rotate`)
    #[inline]
    #[must_use]
    pub fn rotate(self, angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Rotate(self, angle) }
    }

    /// Move towards target by at most maxDistance. (raymath `Vector2MoveTowards`)
    #[inline]
    #[must_use]
    pub fn move_towards(self, target: Vector2, max_distance: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2MoveTowards(self, target, max_distance) }
    }

    /// Invert components (1/x, 1/y). (raymath `Vector2Invert`)
    #[inline]
    #[must_use]
    pub fn invert(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Invert(self) }
    }

    /// Clamp components between min and max vectors. (raymath `Vector2Clamp`)
    #[inline]
    #[must_use]
    pub fn clamp(self, min: Vector2, max: Vector2) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Clamp(self, min, max) }
    }

    /// Clamp length between min and max scalars. (raymath `Vector2ClampValue`)
    #[inline]
    #[must_use]
    pub fn clamp_value(self, min: f32, max: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2ClampValue(self, min, max) }
    }

    /// Approximate equality (uses raymath epsilon). (raymath `Vector2Equals`)
    #[inline]
    #[must_use]
    pub fn equals(self, other: Vector2) -> bool {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Equals(self, other) != 0 }
    }

    /// Refract through surface with index ratio r. (raymath `Vector2Refract`)
    #[inline]
    #[must_use]
    pub fn refract(self, n: Vector2, r: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector2Refract(self, n, r) }
    }
}

impl From<(f32, f32)> for Vector2 {
    /// Construct from an `(x, y)` tuple.
    #[inline]
    fn from((x, y): (f32, f32)) -> Self {
        Vector2 { x, y }
    }
}
impl From<[f32; 2]> for Vector2 {
    /// Construct from an `[x, y]` array.
    #[inline]
    fn from([x, y]: [f32; 2]) -> Self {
        Vector2 { x, y }
    }
}

impl core::ops::Add for Vector2 {
    type Output = Vector2;
    #[inline]
    fn add(self, rhs: Vector2) -> Vector2 {
        // SAFETY: Vector2Add is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector2Add(self, rhs) }
    }
}
impl core::ops::AddAssign for Vector2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector2) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for Vector2 {
    type Output = Vector2;
    #[inline]
    fn sub(self, rhs: Vector2) -> Vector2 {
        // SAFETY: Vector2Subtract is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector2Subtract(self, rhs) }
    }
}
impl core::ops::SubAssign for Vector2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector2) {
        *self = *self - rhs;
    }
}

impl core::ops::Mul<f32> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn mul(self, rhs: f32) -> Vector2 {
        // SAFETY: Vector2Scale is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector2Scale(self, rhs) }
    }
}
impl core::ops::MulAssign<f32> for Vector2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl core::ops::Mul<Vector2> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn mul(self, rhs: Vector2) -> Vector2 {
        // SAFETY: Vector2Multiply is a pure value-in/out raymath fn (component-wise); no preconditions.
        unsafe { crate::Vector2Multiply(self, rhs) }
    }
}

impl core::ops::Div<Vector2> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn div(self, rhs: Vector2) -> Vector2 {
        // SAFETY: Vector2Divide is a pure value-in/out raymath fn (component-wise); no preconditions.
        unsafe { crate::Vector2Divide(self, rhs) }
    }
}

impl core::ops::Neg for Vector2 {
    type Output = Vector2;
    #[inline]
    fn neg(self) -> Vector2 {
        // SAFETY: Vector2Negate is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector2Negate(self) }
    }
}

// ─── Vector3 ─────────────────────────────────────────────────────────────────

impl Vector3 {
    /// The zero vector `(0, 0, 0)`.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    /// The vector `(1, 1, 1)`.
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    /// The unit vector along +X `(1, 0, 0)`.
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    /// The unit vector along +Y `(0, 1, 0)`.
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    /// The unit vector along +Z `(0, 0, 1)`.
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    /// Construct a new Vector3.
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Zero vector. (raymath `Vector3Zero`)
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        // SAFETY: Vector3Zero is a pure value-out raymath fn; no preconditions.
        unsafe { crate::Vector3Zero() }
    }

    /// Vector with all components set to 1. (raymath `Vector3One`)
    #[inline]
    #[must_use]
    pub fn one() -> Self {
        // SAFETY: Vector3One is a pure value-out raymath fn; no preconditions.
        unsafe { crate::Vector3One() }
    }

    /// Add scalar to all components. (raymath `Vector3AddValue`)
    #[inline]
    #[must_use]
    pub fn add_value(self, add: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3AddValue(self, add) }
    }

    /// Subtract scalar from all components. (raymath `Vector3SubtractValue`)
    #[inline]
    #[must_use]
    pub fn sub_value(self, sub: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3SubtractValue(self, sub) }
    }

    /// Scale vector by scalar. (raymath `Vector3Scale`)
    #[inline]
    #[must_use]
    pub fn scale(self, scalar: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Scale(self, scalar) }
    }

    /// Component-wise multiply. (raymath `Vector3Multiply`)
    #[inline]
    #[must_use]
    pub fn multiply(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Multiply(self, other) }
    }

    /// Cross product. (raymath `Vector3CrossProduct`)
    #[inline]
    #[must_use]
    pub fn cross(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3CrossProduct(self, other) }
    }

    /// Perpendicular vector. (raymath `Vector3Perpendicular`)
    #[inline]
    #[must_use]
    pub fn perpendicular(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Perpendicular(self) }
    }

    /// Vector length. (raymath `Vector3Length`)
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Length(self) }
    }

    /// Squared vector length. (raymath `Vector3LengthSqr`)
    #[inline]
    #[must_use]
    pub fn length_sqr(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3LengthSqr(self) }
    }

    /// Dot product. (raymath `Vector3DotProduct`)
    #[inline]
    #[must_use]
    pub fn dot(self, other: Vector3) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3DotProduct(self, other) }
    }

    /// Distance between two vectors. (raymath `Vector3Distance`)
    #[inline]
    #[must_use]
    pub fn distance(self, other: Vector3) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Distance(self, other) }
    }

    /// Squared distance between two vectors. (raymath `Vector3DistanceSqr`)
    #[inline]
    #[must_use]
    pub fn distance_sqr(self, other: Vector3) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3DistanceSqr(self, other) }
    }

    /// Angle between two vectors (radians). (raymath `Vector3Angle`)
    #[inline]
    #[must_use]
    pub fn angle(self, other: Vector3) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Angle(self, other) }
    }

    /// Negate vector. (raymath `Vector3Negate`)
    #[inline]
    #[must_use]
    pub fn negate(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Negate(self) }
    }

    /// Component-wise divide. (raymath `Vector3Divide`)
    #[inline]
    #[must_use]
    pub fn divide(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Divide(self, other) }
    }

    /// Normalize vector. (raymath `Vector3Normalize`)
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Normalize(self) }
    }

    /// Project `self` onto `other`. (raymath `Vector3Project`)
    #[inline]
    #[must_use]
    pub fn project(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Project(self, other) }
    }

    /// Reject `self` from `other` (component perpendicular to `other`). (raymath `Vector3Reject`)
    #[inline]
    #[must_use]
    pub fn reject(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Reject(self, other) }
    }

    /// Transform by matrix. (raymath `Vector3Transform`)
    #[inline]
    #[must_use]
    pub fn transform(self, mat: Matrix) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Transform(self, mat) }
    }

    /// Rotate by quaternion. (raymath `Vector3RotateByQuaternion`)
    #[inline]
    #[must_use]
    pub fn rotate_by_quaternion(self, q: Quaternion) -> Self {
        // SAFETY: pure value-in/out; Quaternion is #[repr(C)] matching the FFI type.
        unsafe { crate::Vector3RotateByQuaternion(self, q) }
    }

    /// Rotate by axis and angle (radians). (raymath `Vector3RotateByAxisAngle`)
    #[inline]
    #[must_use]
    pub fn rotate_by_axis_angle(self, axis: Vector3, angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3RotateByAxisAngle(self, axis, angle) }
    }

    /// Move towards target by at most maxDistance. (raymath `Vector3MoveTowards`)
    #[inline]
    #[must_use]
    pub fn move_towards(self, target: Vector3, max_distance: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3MoveTowards(self, target, max_distance) }
    }

    /// Linear interpolation. (raymath `Vector3Lerp`)
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Vector3, amount: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Lerp(self, other, amount) }
    }

    /// Cubic Hermite interpolation. (raymath `Vector3CubicHermite`)
    #[inline]
    #[must_use]
    pub fn cubic_hermite(
        self,
        tangent1: Vector3,
        v2: Vector3,
        tangent2: Vector3,
        amount: f32,
    ) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3CubicHermite(self, tangent1, v2, tangent2, amount) }
    }

    /// Reflect vector about normal. (raymath `Vector3Reflect`)
    #[inline]
    #[must_use]
    pub fn reflect(self, normal: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Reflect(self, normal) }
    }

    /// Component-wise minimum. (raymath `Vector3Min`)
    #[inline]
    #[must_use]
    pub fn min(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Min(self, other) }
    }

    /// Component-wise maximum. (raymath `Vector3Max`)
    #[inline]
    #[must_use]
    pub fn max(self, other: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Max(self, other) }
    }

    /// Barycentric coordinates of point p in triangle (a, b, c). (raymath `Vector3Barycenter`)
    #[inline]
    #[must_use]
    pub fn barycenter(p: Vector3, a: Vector3, b: Vector3, c: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Barycenter(p, a, b, c) }
    }

    /// Unproject vector from screen space using projection and view matrices. (raymath `Vector3Unproject`)
    #[inline]
    #[must_use]
    pub fn unproject(self, projection: Matrix, view: Matrix) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Unproject(self, projection, view) }
    }

    /// Convert to array of floats. (raymath `Vector3ToFloatV`)
    #[inline]
    #[must_use]
    pub fn to_float_array(self) -> crate::float3 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3ToFloatV(self) }
    }

    /// Invert components (1/x, 1/y, 1/z). (raymath `Vector3Invert`)
    #[inline]
    #[must_use]
    pub fn invert(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Invert(self) }
    }

    /// Clamp components between min and max vectors. (raymath `Vector3Clamp`)
    #[inline]
    #[must_use]
    pub fn clamp(self, min: Vector3, max: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Clamp(self, min, max) }
    }

    /// Clamp length between min and max scalars. (raymath `Vector3ClampValue`)
    #[inline]
    #[must_use]
    pub fn clamp_value(self, min: f32, max: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3ClampValue(self, min, max) }
    }

    /// Approximate equality (uses raymath epsilon). (raymath `Vector3Equals`)
    #[inline]
    #[must_use]
    pub fn equals(self, other: Vector3) -> bool {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Equals(self, other) != 0 }
    }

    /// Refract through surface with index ratio r. (raymath `Vector3Refract`)
    #[inline]
    #[must_use]
    pub fn refract(self, n: Vector3, r: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector3Refract(self, n, r) }
    }
}

/// Orthonormalize two vectors in-place. (raymath `Vector3OrthoNormalize`)
pub fn vector3_ortho_normalize(v1: &mut Vector3, v2: &mut Vector3) {
    // SAFETY: Vector3OrthoNormalize writes through the given pointers, which are valid,
    // non-null, properly aligned &mut references. v1 and v2 are distinct borrows so
    // there is no aliasing.
    unsafe { crate::Vector3OrthoNormalize(v1 as *mut _, v2 as *mut _) }
}

impl From<(f32, f32, f32)> for Vector3 {
    /// Construct from an `(x, y, z)` tuple.
    #[inline]
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Vector3 { x, y, z }
    }
}
impl From<[f32; 3]> for Vector3 {
    /// Construct from an `[x, y, z]` array.
    #[inline]
    fn from([x, y, z]: [f32; 3]) -> Self {
        Vector3 { x, y, z }
    }
}

impl core::ops::Add for Vector3 {
    type Output = Vector3;
    #[inline]
    fn add(self, rhs: Vector3) -> Vector3 {
        // SAFETY: Vector3Add is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector3Add(self, rhs) }
    }
}
impl core::ops::AddAssign for Vector3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector3) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for Vector3 {
    type Output = Vector3;
    #[inline]
    fn sub(self, rhs: Vector3) -> Vector3 {
        // SAFETY: Vector3Subtract is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector3Subtract(self, rhs) }
    }
}
impl core::ops::SubAssign for Vector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector3) {
        *self = *self - rhs;
    }
}

impl core::ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn mul(self, rhs: f32) -> Vector3 {
        // SAFETY: Vector3Scale is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector3Scale(self, rhs) }
    }
}
impl core::ops::MulAssign<f32> for Vector3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl core::ops::Mul<Vector3> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn mul(self, rhs: Vector3) -> Vector3 {
        // SAFETY: Vector3Multiply is a pure value-in/out raymath fn (component-wise); no preconditions.
        unsafe { crate::Vector3Multiply(self, rhs) }
    }
}

impl core::ops::Div<Vector3> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn div(self, rhs: Vector3) -> Vector3 {
        // SAFETY: Vector3Divide is a pure value-in/out raymath fn (component-wise); no preconditions.
        unsafe { crate::Vector3Divide(self, rhs) }
    }
}

impl core::ops::Neg for Vector3 {
    type Output = Vector3;
    #[inline]
    fn neg(self) -> Vector3 {
        // SAFETY: Vector3Negate is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector3Negate(self) }
    }
}

// ─── Vector4 ─────────────────────────────────────────────────────────────────

impl Vector4 {
    /// Construct a new Vector4.
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Zero vector. (raymath `Vector4Zero`)
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        // SAFETY: Vector4Zero is a pure value-out raymath fn; no preconditions.
        unsafe { crate::Vector4Zero() }
    }

    /// Vector with all components set to 1. (raymath `Vector4One`)
    #[inline]
    #[must_use]
    pub fn one() -> Self {
        // SAFETY: Vector4One is a pure value-out raymath fn; no preconditions.
        unsafe { crate::Vector4One() }
    }

    /// Add scalar to all components. (raymath `Vector4AddValue`)
    #[inline]
    #[must_use]
    pub fn add_value(self, add: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4AddValue(self, add) }
    }

    /// Subtract scalar from all components. (raymath `Vector4SubtractValue`)
    #[inline]
    #[must_use]
    pub fn sub_value(self, sub: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4SubtractValue(self, sub) }
    }

    /// Vector length. (raymath `Vector4Length`)
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Length(self) }
    }

    /// Squared vector length. (raymath `Vector4LengthSqr`)
    #[inline]
    #[must_use]
    pub fn length_sqr(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4LengthSqr(self) }
    }

    /// Dot product. (raymath `Vector4DotProduct`)
    #[inline]
    #[must_use]
    pub fn dot(self, other: Vector4) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4DotProduct(self, other) }
    }

    /// Distance between two vectors. (raymath `Vector4Distance`)
    #[inline]
    #[must_use]
    pub fn distance(self, other: Vector4) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Distance(self, other) }
    }

    /// Squared distance between two vectors. (raymath `Vector4DistanceSqr`)
    #[inline]
    #[must_use]
    pub fn distance_sqr(self, other: Vector4) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4DistanceSqr(self, other) }
    }

    /// Scale vector by scalar. (raymath `Vector4Scale`)
    #[inline]
    #[must_use]
    pub fn scale(self, scale: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Scale(self, scale) }
    }

    /// Component-wise multiply. (raymath `Vector4Multiply`)
    #[inline]
    #[must_use]
    pub fn multiply(self, other: Vector4) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Multiply(self, other) }
    }

    /// Negate vector. (raymath `Vector4Negate`)
    #[inline]
    #[must_use]
    pub fn negate(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Negate(self) }
    }

    /// Component-wise divide. (raymath `Vector4Divide`)
    #[inline]
    #[must_use]
    pub fn divide(self, other: Vector4) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Divide(self, other) }
    }

    /// Normalize vector. (raymath `Vector4Normalize`)
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Normalize(self) }
    }

    /// Component-wise minimum. (raymath `Vector4Min`)
    #[inline]
    #[must_use]
    pub fn min(self, other: Vector4) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Min(self, other) }
    }

    /// Component-wise maximum. (raymath `Vector4Max`)
    #[inline]
    #[must_use]
    pub fn max(self, other: Vector4) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Max(self, other) }
    }

    /// Linear interpolation. (raymath `Vector4Lerp`)
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Vector4, amount: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Lerp(self, other, amount) }
    }

    /// Move towards target by at most maxDistance. (raymath `Vector4MoveTowards`)
    #[inline]
    #[must_use]
    pub fn move_towards(self, target: Vector4, max_distance: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4MoveTowards(self, target, max_distance) }
    }

    /// Invert components (1/x, 1/y, 1/z, 1/w). (raymath `Vector4Invert`)
    #[inline]
    #[must_use]
    pub fn invert(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Invert(self) }
    }

    /// Approximate equality (uses raymath epsilon). (raymath `Vector4Equals`)
    #[inline]
    #[must_use]
    pub fn equals(self, other: Vector4) -> bool {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::Vector4Equals(self, other) != 0 }
    }
}

impl From<(f32, f32, f32, f32)> for Vector4 {
    /// Construct from an `(x, y, z, w)` tuple.
    #[inline]
    fn from((x, y, z, w): (f32, f32, f32, f32)) -> Self {
        Vector4 { x, y, z, w }
    }
}
impl From<[f32; 4]> for Vector4 {
    /// Construct from an `[x, y, z, w]` array.
    #[inline]
    fn from([x, y, z, w]: [f32; 4]) -> Self {
        Vector4 { x, y, z, w }
    }
}

impl core::ops::Add for Vector4 {
    type Output = Vector4;
    #[inline]
    fn add(self, rhs: Vector4) -> Vector4 {
        // SAFETY: Vector4Add is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector4Add(self, rhs) }
    }
}
impl core::ops::AddAssign for Vector4 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector4) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for Vector4 {
    type Output = Vector4;
    #[inline]
    fn sub(self, rhs: Vector4) -> Vector4 {
        // SAFETY: Vector4Subtract is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector4Subtract(self, rhs) }
    }
}
impl core::ops::SubAssign for Vector4 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector4) {
        *self = *self - rhs;
    }
}

impl core::ops::Mul<f32> for Vector4 {
    type Output = Vector4;
    #[inline]
    fn mul(self, rhs: f32) -> Vector4 {
        // SAFETY: Vector4Scale is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector4Scale(self, rhs) }
    }
}
impl core::ops::MulAssign<f32> for Vector4 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl core::ops::Mul<Vector4> for Vector4 {
    type Output = Vector4;
    #[inline]
    fn mul(self, rhs: Vector4) -> Vector4 {
        // SAFETY: Vector4Multiply is a pure value-in/out raymath fn (component-wise); no preconditions.
        unsafe { crate::Vector4Multiply(self, rhs) }
    }
}

impl core::ops::Div<Vector4> for Vector4 {
    type Output = Vector4;
    #[inline]
    fn div(self, rhs: Vector4) -> Vector4 {
        // SAFETY: Vector4Divide is a pure value-in/out raymath fn (component-wise); no preconditions.
        unsafe { crate::Vector4Divide(self, rhs) }
    }
}

impl core::ops::Neg for Vector4 {
    type Output = Vector4;
    #[inline]
    fn neg(self) -> Vector4 {
        // SAFETY: Vector4Negate is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::Vector4Negate(self) }
    }
}
