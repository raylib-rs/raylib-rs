use crate::{Matrix, Quaternion, Vector3};

// ─── Matrix ──────────────────────────────────────────────────────────────────

impl Matrix {
    /// Identity matrix. (raymath `MatrixIdentity`)
    #[inline]
    #[must_use]
    pub fn identity() -> Self {
        // SAFETY: MatrixIdentity is a pure value-out raymath fn; no preconditions.
        unsafe { crate::MatrixIdentity() }
    }

    /// Matrix determinant. (raymath `MatrixDeterminant`)
    #[inline]
    #[must_use]
    pub fn determinant(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixDeterminant(self) }
    }

    /// Trace of the matrix (sum of diagonal values). (raymath `MatrixTrace`)
    #[inline]
    #[must_use]
    pub fn trace(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixTrace(self) }
    }

    /// Transpose matrix. (raymath `MatrixTranspose`)
    #[inline]
    #[must_use]
    pub fn transpose(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixTranspose(self) }
    }

    /// Invert matrix. (raymath `MatrixInvert`)
    #[inline]
    #[must_use]
    pub fn invert(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixInvert(self) }
    }

    /// Multiply matrix by scalar. (raymath `MatrixMultiplyValue`)
    #[inline]
    #[must_use]
    pub fn mul_value(self, value: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixMultiplyValue(self, value) }
    }

    /// Translate matrix. (raymath `MatrixTranslate`)
    #[inline]
    #[must_use]
    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixTranslate(x, y, z) }
    }

    /// Rotate matrix by axis and angle (radians). (raymath `MatrixRotate`)
    #[inline]
    #[must_use]
    pub fn rotate(axis: Vector3, angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixRotate(axis, angle) }
    }

    /// Rotate around X axis (radians). (raymath `MatrixRotateX`)
    #[inline]
    #[must_use]
    pub fn rotate_x(angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixRotateX(angle) }
    }

    /// Rotate around Y axis (radians). (raymath `MatrixRotateY`)
    #[inline]
    #[must_use]
    pub fn rotate_y(angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixRotateY(angle) }
    }

    /// Rotate around Z axis (radians). (raymath `MatrixRotateZ`)
    #[inline]
    #[must_use]
    pub fn rotate_z(angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixRotateZ(angle) }
    }

    /// Rotate around XYZ axes (radians, intrinsic order). (raymath `MatrixRotateXYZ`)
    #[inline]
    #[must_use]
    pub fn rotate_xyz(angle: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixRotateXYZ(angle) }
    }

    /// Rotate around ZYX axes (radians, intrinsic order). (raymath `MatrixRotateZYX`)
    #[inline]
    #[must_use]
    pub fn rotate_zyx(angle: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixRotateZYX(angle) }
    }

    /// Scale matrix. (raymath `MatrixScale`)
    #[inline]
    #[must_use]
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixScale(x, y, z) }
    }

    /// Frustum projection matrix. (raymath `MatrixFrustum`)
    #[inline]
    #[must_use]
    pub fn frustum(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixFrustum(left, right, bottom, top, near, far) }
    }

    /// Perspective projection matrix. (raymath `MatrixPerspective`)
    #[inline]
    #[must_use]
    pub fn perspective(fov_y: f64, aspect: f64, near: f64, far: f64) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixPerspective(fov_y, aspect, near, far) }
    }

    /// Orthographic projection matrix. (raymath `MatrixOrtho`)
    #[inline]
    #[must_use]
    pub fn ortho(left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixOrtho(left, right, bottom, top, near, far) }
    }

    /// Look-at view matrix. (raymath `MatrixLookAt`)
    #[inline]
    #[must_use]
    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixLookAt(eye, target, up) }
    }

    /// Convert matrix to float array. (raymath `MatrixToFloatV`)
    #[inline]
    #[must_use]
    pub fn to_float_array(self) -> crate::float16 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixToFloatV(self) }
    }

    /// Compose a matrix from translation, rotation quaternion, and scale. (raymath `MatrixCompose`)
    #[inline]
    #[must_use]
    pub fn compose(translation: Vector3, rotation: Quaternion, scale: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::MatrixCompose(translation, rotation, scale) }
    }
}

impl core::ops::Mul for Matrix {
    type Output = Matrix;
    #[inline]
    fn mul(self, rhs: Matrix) -> Matrix {
        // SAFETY: MatrixMultiply is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::MatrixMultiply(self, rhs) }
    }
}
impl core::ops::MulAssign for Matrix {
    #[inline]
    fn mul_assign(&mut self, rhs: Matrix) {
        *self = *self * rhs;
    }
}

impl core::ops::Add for Matrix {
    type Output = Matrix;
    #[inline]
    fn add(self, rhs: Matrix) -> Matrix {
        // SAFETY: MatrixAdd is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::MatrixAdd(self, rhs) }
    }
}
impl core::ops::AddAssign for Matrix {
    #[inline]
    fn add_assign(&mut self, rhs: Matrix) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for Matrix {
    type Output = Matrix;
    #[inline]
    fn sub(self, rhs: Matrix) -> Matrix {
        // SAFETY: MatrixSubtract is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::MatrixSubtract(self, rhs) }
    }
}
impl core::ops::SubAssign for Matrix {
    #[inline]
    fn sub_assign(&mut self, rhs: Matrix) {
        *self = *self - rhs;
    }
}

/// Decompose a matrix into translation, rotation quaternion, and scale.
/// (raymath `MatrixDecompose`)
pub fn matrix_decompose(mat: Matrix) -> (Vector3, Quaternion, Vector3) {
    let mut translation = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut rotation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    let mut scale = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    // SAFETY: MatrixDecompose writes through non-null, properly aligned out-ptrs;
    // all three are distinct stack allocations with no aliasing.
    unsafe { crate::MatrixDecompose(mat, &mut translation, &mut rotation, &mut scale) };
    (translation, rotation, scale)
}

// ─── Quaternion ──────────────────────────────────────────────────────────────

impl Quaternion {
    /// Identity quaternion (0, 0, 0, 1). (raymath `QuaternionIdentity`)
    #[inline]
    #[must_use]
    pub fn identity() -> Self {
        // SAFETY: QuaternionIdentity is a pure value-out raymath fn; no preconditions.
        unsafe { crate::QuaternionIdentity() }
    }

    /// Add scalar to each component. (raymath `QuaternionAddValue`)
    #[inline]
    #[must_use]
    pub fn add_value(self, add: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionAddValue(self, add) }
    }

    /// Subtract scalar from each component. (raymath `QuaternionSubtractValue`)
    #[inline]
    #[must_use]
    pub fn sub_value(self, sub: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionSubtractValue(self, sub) }
    }

    /// Quaternion length. (raymath `QuaternionLength`)
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionLength(self) }
    }

    /// Normalize quaternion. (raymath `QuaternionNormalize`)
    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionNormalize(self) }
    }

    /// Invert quaternion. (raymath `QuaternionInvert`)
    #[inline]
    #[must_use]
    pub fn invert(self) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionInvert(self) }
    }

    /// Linear interpolation. (raymath `QuaternionLerp`)
    #[inline]
    #[must_use]
    pub fn lerp(self, other: Quaternion, amount: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionLerp(self, other, amount) }
    }

    /// Normalized linear interpolation. (raymath `QuaternionNlerp`)
    #[inline]
    #[must_use]
    pub fn nlerp(self, other: Quaternion, amount: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionNlerp(self, other, amount) }
    }

    /// Spherical linear interpolation. (raymath `QuaternionSlerp`)
    #[inline]
    #[must_use]
    pub fn slerp(self, other: Quaternion, amount: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionSlerp(self, other, amount) }
    }

    /// Cubic Hermite spline interpolation. (raymath `QuaternionCubicHermiteSpline`)
    #[inline]
    #[must_use]
    pub fn cubic_hermite_spline(
        self,
        out_tangent1: Quaternion,
        q2: Quaternion,
        in_tangent2: Quaternion,
        t: f32,
    ) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionCubicHermiteSpline(self, out_tangent1, q2, in_tangent2, t) }
    }

    /// Quaternion for rotation from one vector to another. (raymath `QuaternionFromVector3ToVector3`)
    #[inline]
    #[must_use]
    pub fn from_vector3_to_vector3(from: Vector3, to: Vector3) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionFromVector3ToVector3(from, to) }
    }

    /// Quaternion from rotation matrix. (raymath `QuaternionFromMatrix`)
    #[inline]
    #[must_use]
    pub fn from_matrix(mat: Matrix) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionFromMatrix(mat) }
    }

    /// Rotation matrix from quaternion. (raymath `QuaternionToMatrix`)
    #[inline]
    #[must_use]
    pub fn to_matrix(self) -> Matrix {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionToMatrix(self) }
    }

    /// Quaternion from axis and angle (radians). (raymath `QuaternionFromAxisAngle`)
    #[inline]
    #[must_use]
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionFromAxisAngle(axis, angle) }
    }

    /// Quaternion from Euler angles (pitch, yaw, roll in radians). (raymath `QuaternionFromEuler`)
    #[inline]
    #[must_use]
    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionFromEuler(pitch, yaw, roll) }
    }

    /// Convert quaternion to Euler angles (roll, pitch, yaw in radians). (raymath `QuaternionToEuler`)
    #[inline]
    #[must_use]
    pub fn to_euler(self) -> Vector3 {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionToEuler(self) }
    }

    /// Transform quaternion by matrix. (raymath `QuaternionTransform`)
    #[inline]
    #[must_use]
    pub fn transform(self, mat: Matrix) -> Self {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionTransform(self, mat) }
    }

    /// Approximate equality (uses raymath epsilon). (raymath `QuaternionEquals`)
    #[inline]
    #[must_use]
    pub fn equals(self, other: Quaternion) -> bool {
        // SAFETY: pure value-in/out, no preconditions.
        unsafe { crate::QuaternionEquals(self, other) != 0 }
    }
}

impl core::ops::Add for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn add(self, rhs: Quaternion) -> Quaternion {
        // SAFETY: QuaternionAdd is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::QuaternionAdd(self, rhs) }
    }
}
impl core::ops::AddAssign for Quaternion {
    #[inline]
    fn add_assign(&mut self, rhs: Quaternion) {
        *self = *self + rhs;
    }
}

impl core::ops::Sub for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn sub(self, rhs: Quaternion) -> Quaternion {
        // SAFETY: QuaternionSubtract is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::QuaternionSubtract(self, rhs) }
    }
}
impl core::ops::SubAssign for Quaternion {
    #[inline]
    fn sub_assign(&mut self, rhs: Quaternion) {
        *self = *self - rhs;
    }
}

impl core::ops::Mul for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn mul(self, rhs: Quaternion) -> Quaternion {
        // SAFETY: QuaternionMultiply is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::QuaternionMultiply(self, rhs) }
    }
}
impl core::ops::MulAssign for Quaternion {
    #[inline]
    fn mul_assign(&mut self, rhs: Quaternion) {
        *self = *self * rhs;
    }
}

impl core::ops::Mul<f32> for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn mul(self, rhs: f32) -> Quaternion {
        // SAFETY: QuaternionScale is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::QuaternionScale(self, rhs) }
    }
}
impl core::ops::MulAssign<f32> for Quaternion {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl core::ops::Div for Quaternion {
    type Output = Quaternion;
    #[inline]
    fn div(self, rhs: Quaternion) -> Quaternion {
        // SAFETY: QuaternionDivide is a pure value-in/out raymath fn; no preconditions.
        unsafe { crate::QuaternionDivide(self, rhs) }
    }
}
impl core::ops::DivAssign for Quaternion {
    #[inline]
    fn div_assign(&mut self, rhs: Quaternion) {
        *self = *self / rhs;
    }
}

/// Convert quaternion to axis-angle representation.
/// Returns `(axis, angle_radians)`. (raymath `QuaternionToAxisAngle`)
pub fn quaternion_to_axis_angle(q: Quaternion) -> (Vector3, f32) {
    let mut out_axis = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let mut out_angle: f32 = 0.0;
    // SAFETY: QuaternionToAxisAngle writes through non-null, properly aligned out-ptrs;
    // both are distinct stack allocations with no aliasing.
    unsafe { crate::QuaternionToAxisAngle(q, &mut out_axis, &mut out_angle) };
    (out_axis, out_angle)
}
