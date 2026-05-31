//! Utility code for using Raylib [`Camera3D`] and [`Camera2D`]
use raylib_sys::CameraMode;
use std::mem::transmute;

use crate::ffi::{self, CameraProjection};
use crate::math::{Vector2, Vector3};

use super::math::Matrix;

/// Camera2D, defines position/orientation in 2d space
#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Camera2D {
    /// Camera offset (displacement from target)
    pub offset: Vector2,
    /// Camera target (rotation and zoom origin)
    pub target: Vector2,
    /// Camera rotation in degrees
    pub rotation: f32,
    /// Camera zoom (scaling), should be 1.0 by default
    pub zoom: f32,
}
impl Camera2D {
    #[must_use]
    #[inline(always)]
    #[allow(dead_code)]
    fn get_camera_matrix_2d(camera: impl Into<ffi::Camera2D>) -> Matrix {
        unsafe { ffi::GetCameraMatrix2D(camera.into()) }
    }
}

impl From<ffi::Camera2D> for Camera2D {
    fn from(v: ffi::Camera2D) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<Camera2D> for ffi::Camera2D {
    fn from(v: Camera2D) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&Camera2D> for ffi::Camera2D {
    fn from(v: &Camera2D) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}

/// Camera, defines position/orientation in 3d space
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Camera3D {
    /// Camera position
    pub position: Vector3,
    /// Camera target it looks-at
    pub target: Vector3,
    /// Camera up vector (rotation over its axis)
    pub up: Vector3,
    /// Camera field-of-view aperture in Y (degrees) in perspective, used as near plane width in orthographic
    pub fovy: f32,
    /// Camera projection: CAMERA_PERSPECTIVE or CAMERA_ORTHOGRAPHIC
    pub projection: CameraProjection,
}
/// Camera type fallback, defaults to Camera3D
pub type Camera = Camera3D;

impl From<ffi::Camera3D> for Camera3D {
    fn from(v: ffi::Camera3D) -> Camera3D {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<Camera3D> for ffi::Camera3D {
    fn from(v: Camera3D) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&Camera3D> for ffi::Camera3D {
    fn from(v: &Camera3D) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}

impl From<&mut Camera3D> for ffi::Camera3D {
    fn from(v: &mut Camera3D) -> Self {
        unsafe { std::mem::transmute(*v) }
    }
}
impl From<&mut Camera3D> for *mut ffi::Camera3D {
    fn from(val: &mut Camera3D) -> Self {
        val as *mut Camera3D as *mut ffi::Camera3D
    }
}

impl Camera3D {
    /// Returns the camera projection type (perspective or orthographic).
    ///
    /// This mirrors the `projection` field; use the [`CameraProjection`]
    /// discriminant to branch between perspective and orthographic
    /// rendering paths.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let p = Camera3D::perspective(
    ///     Vector3::new(0.0, 10.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// assert_eq!(p.camera_type(), CameraProjection::CAMERA_PERSPECTIVE);
    ///
    /// let o = Camera3D::orthographic(
    ///     Vector3::new(0.0, 10.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     20.0,
    /// );
    /// assert_eq!(o.camera_type(), CameraProjection::CAMERA_ORTHOGRAPHIC);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::perspective`] — construct a perspective camera.
    /// - [`Camera3D::orthographic`] — construct an orthographic camera.
    #[must_use]
    #[inline(always)]
    pub fn camera_type(&self) -> CameraProjection {
        unsafe { transmute(self.projection as u32) }
    }

    /// Constructs a perspective `Camera3D` with the given position, target, up vector, and vertical field-of-view (degrees).
    ///
    /// The projection is set to [`CameraProjection::CAMERA_PERSPECTIVE`].
    /// Use [`Camera3D::orthographic`] if you want a parallel projection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 10.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// assert_eq!(cam.position, Vector3::new(0.0, 10.0, 10.0));
    /// assert_eq!(cam.target, Vector3::ZERO);
    /// assert_eq!(cam.up, Vector3::Y);
    /// assert_eq!(cam.fovy, 60.0);
    /// assert_eq!(cam.projection, CameraProjection::CAMERA_PERSPECTIVE);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::orthographic`] — orthographic counterpart.
    #[must_use]
    #[inline(always)]
    pub fn perspective(position: Vector3, target: Vector3, up: Vector3, fovy: f32) -> Camera3D {
        Camera3D {
            position,
            target,
            up,
            fovy,
            projection: CameraProjection::CAMERA_PERSPECTIVE,
        }
    }

    /// Constructs an orthographic `Camera3D` with the given position, target, up vector, and near-plane width.
    ///
    /// In orthographic mode the `fovy` parameter is reinterpreted as the
    /// near-plane width in world units (raylib's convention). Use
    /// [`Camera3D::perspective`] for a standard FOV-driven projection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let cam = Camera3D::orthographic(
    ///     Vector3::new(0.0, 10.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     20.0,
    /// );
    /// assert_eq!(cam.projection, CameraProjection::CAMERA_ORTHOGRAPHIC);
    /// assert_eq!(cam.fovy, 20.0);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::perspective`] — perspective counterpart.
    #[must_use]
    #[inline(always)]
    pub fn orthographic(position: Vector3, target: Vector3, up: Vector3, fovy: f32) -> Camera3D {
        let mut c = Self::perspective(position, target, up, fovy);
        c.projection = CameraProjection::CAMERA_ORTHOGRAPHIC;
        c
    }

    /// Returns the unit vector pointing from the camera position toward its target.
    ///
    /// Always normalized. Reflects the current `position`/`target` state,
    /// so calling it again after moving the camera returns the updated
    /// direction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let fwd = cam.forward();
    /// // Camera at +Z looking at origin → forward points along -Z.
    /// assert!((fwd.x).abs() < 1e-5);
    /// assert!((fwd.y).abs() < 1e-5);
    /// assert!((fwd.z + 1.0).abs() < 1e-5);
    /// // Forward is unit-length.
    /// let len = (fwd.x * fwd.x + fwd.y * fwd.y + fwd.z * fwd.z).sqrt();
    /// assert!((len - 1.0).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::up`] — companion up direction.
    /// - [`Camera3D::move_forward`] — translate along this axis.
    #[must_use]
    #[inline(always)]
    pub fn forward(&self) -> Vector3 {
        unsafe { ffi::GetCameraForward(self as *const _ as *mut _) }
    }

    /// Returns the camera's normalized up direction vector.
    ///
    /// This is the camera's local up axis (orthogonal to `forward`),
    /// recomputed from the current `up` field. Useful when you need the
    /// "true" up after a sequence of pitches/rolls.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let up = cam.up();
    /// // With world-up = Y and looking along -Z, up should be ~(0, 1, 0).
    /// assert!((up.x).abs() < 1e-5);
    /// assert!((up.y - 1.0).abs() < 1e-5);
    /// assert!((up.z).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::forward`] — companion forward direction.
    #[must_use]
    #[inline(always)]
    pub fn up(&self) -> Vector3 {
        unsafe { ffi::GetCameraUp(self as *const _ as *mut _) }
    }

    /// Moves the camera forward (or backward if `distance` is negative) along its look direction.
    ///
    /// If `in_world_plane` is `true`, the motion is projected onto the
    /// world XZ plane (useful for FPS-style cameras that should not
    /// "fly" when looking up or down).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_z = cam.position.z;
    /// cam.move_forward(1.0, false);
    /// // Forward points along -Z, so position.z should decrease.
    /// assert!(cam.position.z < initial_z);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::move_up`] / [`Camera3D::move_right`] — companion axes.
    /// - [`Camera3D::move_to_target`] — dolly along the look ray.
    #[inline(always)]
    pub fn move_forward(&mut self, distance: f32, in_world_plane: bool) {
        unsafe { ffi::CameraMoveForward(self.into(), distance, in_world_plane) }
    }

    /// Moves the camera upward (or downward if `distance` is negative) along its local up axis.
    ///
    /// Both `position` and `target` are translated by the same offset,
    /// so the look direction is preserved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_y = cam.position.y;
    /// cam.move_up(2.0);
    /// assert!((cam.position.y - (initial_y + 2.0)).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::move_forward`] / [`Camera3D::move_right`] — companion axes.
    #[inline(always)]
    pub fn move_up(&mut self, distance: f32) {
        unsafe { ffi::CameraMoveUp(self.into(), distance) }
    }

    /// Moves the camera right (or left if `distance` is negative) along its local right axis.
    ///
    /// If `in_world_plane` is `true`, motion is constrained to the world
    /// XZ plane (FPS-style strafe). Both `position` and `target` shift
    /// together so the heading is unchanged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_x = cam.position.x;
    /// cam.move_right(1.0, true);
    /// // Looking along -Z with up = +Y → right is +X.
    /// assert!(cam.position.x > initial_x);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::move_forward`] / [`Camera3D::move_up`] — companion axes.
    #[inline(always)]
    pub fn move_right(&mut self, distance: f32, in_world_plane: bool) {
        unsafe { ffi::CameraMoveRight(self.into(), distance, in_world_plane) }
    }

    /// Adjusts the position-to-target distance by `delta` along the look ray.
    ///
    /// Positive `delta` pushes the camera farther from the target;
    /// negative pulls it closer (clamped so distance stays > 0). The
    /// target itself is unchanged.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_dist = cam.position.z; // distance to origin along +Z.
    /// cam.move_to_target(2.0);
    /// // Positive delta pushes the camera farther from the target along +Z.
    /// assert!(cam.position.z > initial_dist);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::move_forward`] — translate without altering distance-to-target semantics.
    #[inline(always)]
    pub fn move_to_target(&mut self, delta: f32) {
        unsafe { ffi::CameraMoveToTarget(self.into(), delta) }
    }

    /// Rotates the camera around its up axis by `angle` radians (yaw).
    ///
    /// If `rotate_around_target` is `true`, the camera orbits the target
    /// (orbital-camera behavior); otherwise the target rotates around the
    /// camera position (FPS-look behavior).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_target_x = cam.target.x;
    /// cam.yaw(std::f32::consts::FRAC_PI_4, false);
    /// // Target should rotate around the camera; x component changes.
    /// assert!((cam.target.x - initial_target_x).abs() > 1e-3);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::pitch`] / [`Camera3D::roll`] — other rotation axes.
    #[inline(always)]
    pub fn yaw(&mut self, angle: f32, rotate_around_target: bool) {
        unsafe { ffi::CameraYaw(self.into(), angle, rotate_around_target) }
    }

    /// Rotates the camera around its right axis by `angle` radians (pitch).
    ///
    /// `lock_view` clamps pitch to ±~85° to prevent gimbal flip;
    /// `rotate_around_target` orbits the target rather than rotating the
    /// target around the camera; `rotate_up` propagates the rotation to
    /// the up vector itself (rare — typically `false`).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_target_y = cam.target.y;
    /// cam.pitch(0.3, true, false, false);
    /// // Pitching tilts the look ray; target.y should move.
    /// assert!((cam.target.y - initial_target_y).abs() > 1e-3);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::yaw`] / [`Camera3D::roll`] — other rotation axes.
    #[inline(always)]
    pub fn pitch(
        &mut self,
        angle: f32,
        lock_view: bool,
        rotate_around_target: bool,
        rotate_up: bool,
    ) {
        unsafe {
            ffi::CameraPitch(
                self.into(),
                angle,
                lock_view,
                rotate_around_target,
                rotate_up,
            )
        }
    }

    /// Rotates the camera around its forward axis by `angle` radians (roll).
    ///
    /// This tilts the horizon by rotating the camera's up vector while
    /// `position` and `target` stay put.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial_up = cam.up;
    /// cam.roll(std::f32::consts::FRAC_PI_4);
    /// // Up vector should rotate — its x component (was 0) becomes non-zero.
    /// assert!((cam.up.x - initial_up.x).abs() > 1e-3);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::yaw`] / [`Camera3D::pitch`] — other rotation axes.
    #[inline(always)]
    pub fn roll(&mut self, angle: f32) {
        unsafe { ffi::CameraRoll(self.into(), angle) }
    }

    /// Returns the camera's view (world-to-camera) matrix.
    ///
    /// Use this when feeding a shader uniform or composing your own MVP
    /// matrix outside of raylib's draw-mode helpers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let v = cam.view_matrix();
    /// // The view matrix should not be all-zero (a valid view exists).
    /// assert!(v.m0 != 0.0 || v.m5 != 0.0 || v.m10 != 0.0 || v.m15 != 0.0);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::projection_matrix`] — companion projection matrix.
    #[must_use]
    #[inline(always)]
    pub fn view_matrix(&self) -> Matrix {
        unsafe { ffi::GetCameraViewMatrix(self as *const _ as *mut _) }
    }

    /// Returns the camera's projection matrix for the given viewport `aspect` ratio.
    ///
    /// `aspect` is `width / height` of the target framebuffer. For a
    /// perspective camera the result encodes the `fovy`; for an
    /// orthographic camera it encodes the `fovy` as half-width in world
    /// units.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let p = cam.projection_matrix(16.0 / 9.0);
    /// // A perspective projection has non-zero diagonal entries.
    /// assert!(p.m0 != 0.0);
    /// assert!(p.m5 != 0.0);
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::view_matrix`] — companion view matrix.
    #[must_use]
    #[inline(always)]
    pub fn projection_matrix(&self, aspect: f32) -> Matrix {
        unsafe { ffi::GetCameraProjectionMatrix(self as *const _ as *mut _, aspect) }
    }

    /// Updates the camera for the selected built-in mode using current input state.
    ///
    /// Polls mouse delta and keyboard input internally, so this method
    /// requires an initialized raylib window. For pure programmatic
    /// updates without input dependence use [`Camera3D::update_camera_pro`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use raylib::prelude::*;
    ///
    /// let (mut rl, thread) = raylib::init().size(640, 480).title("camera demo").build();
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 10.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// while !rl.window_should_close() {
    ///     cam.update_camera(CameraMode::CAMERA_FREE);
    ///     let mut d = rl.begin_drawing(&thread);
    ///     d.clear_background(Color::RAYWHITE);
    /// }
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::update_camera_pro`] — input-free update with explicit deltas.
    #[inline(always)]
    pub fn update_camera(&mut self, mode: CameraMode) {
        unsafe { ffi::UpdateCamera(self.into(), mode as i32) }
    }

    /// Updates the camera with explicit movement, rotation, and zoom deltas (no input polling).
    ///
    /// `movement` is `(forward, right, up)` translation in world units;
    /// `rotation` is `(yaw, pitch, roll)` in degrees; `zoom` dollies
    /// toward the target. Suitable for headless updates, replays, or
    /// custom input mappings that bypass raylib's built-in modes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use raylib::prelude::*;
    ///
    /// let mut cam = Camera3D::perspective(
    ///     Vector3::new(0.0, 0.0, 10.0),
    ///     Vector3::ZERO,
    ///     Vector3::Y,
    ///     60.0,
    /// );
    /// let initial = cam.position;
    /// // Move forward 1 unit, no rotation or zoom.
    /// cam.update_camera_pro(Vector3::new(1.0, 0.0, 0.0), Vector3::ZERO, 0.0);
    /// // Position should have changed.
    /// assert!(
    ///     (cam.position.x - initial.x).abs()
    ///         + (cam.position.y - initial.y).abs()
    ///         + (cam.position.z - initial.z).abs()
    ///         > 1e-5
    /// );
    /// ```
    ///
    /// # See also
    ///
    /// - [`Camera3D::update_camera`] — input-driven update for built-in modes.
    #[inline(always)]
    pub fn update_camera_pro(
        &mut self,
        movement: impl Into<Vector3>,
        rotation: impl Into<Vector3>,
        zoom: f32,
    ) {
        unsafe { ffi::UpdateCameraPro(self.into(), movement.into(), rotation.into(), zoom) }
    }
}
