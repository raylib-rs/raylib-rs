/* raylib-rs
   raymath.rs - Structs and functions for game-related math and linear algebra

Copyright (c) 2018-2019 Paul Clement (@deltaphc)

This software is provided "as-is", without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

  2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

  3. This notice may not be removed or altered from any source distribution.
*/

//! Game-related math: the public `Vector*`/`Matrix`/`Quaternion` types are the
//! native `raylib-sys` types (own their `#[repr(C)]` layout + raymath methods).
pub use crate::ffi::{Matrix, Quaternion, Vector2, Vector3, Vector4};

use crate::ffi;
use crate::misc::AsF32;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

macro_rules! optional_serde_struct {
    ($def:item) => {
        #[repr(C)]
        #[derive(Default, Debug, Copy, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        $def
    };
}

/// A convenience function for linearly interpolating an `f32`.
#[inline]
#[must_use]
pub const fn lerp(v0: f32, v1: f32, amount: f32) -> f32 {
    v0 + amount * (v1 - v0)
}

/// A convenience function for making a new `Quaternion`.
#[inline]
#[must_use]
pub fn rquat<T1: AsF32, T2: AsF32, T3: AsF32, T4: AsF32>(x: T1, y: T2, z: T3, w: T4) -> Quaternion {
    Quaternion::new(x.as_f32(), y.as_f32(), z.as_f32(), w.as_f32())
}

optional_serde_struct! {
    /// Ray, ray for raycasting
    pub struct Ray {
        /// Ray position (origin)
        pub position: Vector3,
        /// Ray direction (normalized)
        pub direction: Vector3,
    }
}

impl From<ffi::Ray> for Ray {
    fn from(r: ffi::Ray) -> Ray {
        unsafe { std::mem::transmute(r) }
    }
}

impl From<Ray> for ffi::Ray {
    fn from(v: Ray) -> ffi::Ray {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&Ray> for ffi::Ray {
    fn from(v: &Ray) -> ffi::Ray {
        unsafe { std::mem::transmute(*v) }
    }
}

impl Ray {
    #[must_use]
    #[inline]
    pub const fn new(position: Vector3, direction: Vector3) -> Self {
        Self {
            position,
            direction,
        }
    }
}

pub type Rectangle = ffi::Rectangle;

optional_serde_struct! {
    /// BoundingBox
    pub struct BoundingBox {
        /// Minimum vertex box-corner
        pub min: Vector3,
        /// Maximum vertex box-corner
        pub max: Vector3,
    }
}

impl BoundingBox {
    #[must_use]
    #[inline]
    pub fn new(min: Vector3, max: Vector3) -> BoundingBox {
        BoundingBox { min, max }
    }
}

impl From<ffi::BoundingBox> for BoundingBox {
    fn from(r: ffi::BoundingBox) -> BoundingBox {
        unsafe { std::mem::transmute(r) }
    }
}

impl From<BoundingBox> for ffi::BoundingBox {
    fn from(v: BoundingBox) -> ffi::BoundingBox {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&BoundingBox> for ffi::BoundingBox {
    fn from(v: &BoundingBox) -> ffi::BoundingBox {
        unsafe { std::mem::transmute(*v) }
    }
}

impl BoundingBox {
    /// Detects collision between two boxes.
    #[inline]
    #[must_use]
    pub fn check_collision_boxes(&self, box2: BoundingBox) -> bool {
        unsafe { ffi::CheckCollisionBoxes(self.into(), box2.into()) }
    }

    /// Detects collision between box and sphere.
    #[inline]
    #[must_use]
    pub fn check_collision_box_sphere(
        &self,
        center_sphere: impl Into<crate::MintVec3>,
        radius_sphere: f32,
    ) -> bool {
        unsafe { ffi::CheckCollisionBoxSphere(self.into(), center_sphere.into(), radius_sphere) }
    }

    /// Detects collision between ray and box.
    #[inline]
    #[must_use]
    pub fn get_ray_collision_box(&self, ray: Ray) -> RayCollision {
        unsafe { ffi::GetRayCollisionBox(ray.into(), self.into()).into() }
    }
}

optional_serde_struct! {
    /// RayCollision, ray hit information
    pub struct RayCollision {
        /// Did the ray hit something?
        pub hit: bool,
        /// Distance to the nearest hit
        pub distance: f32,
        /// Point of the nearest hit
        pub point: Vector3,
        /// Surface normal of hit
        pub normal: Vector3,
    }
}

impl From<ffi::RayCollision> for RayCollision {
    fn from(r: ffi::RayCollision) -> RayCollision {
        unsafe { std::mem::transmute(r) }
    }
}

impl From<RayCollision> for ffi::RayCollision {
    fn from(v: RayCollision) -> ffi::RayCollision {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&RayCollision> for ffi::RayCollision {
    fn from(v: &RayCollision) -> ffi::RayCollision {
        unsafe { std::mem::transmute(*v) }
    }
}

optional_serde_struct! {
    /// Transform, vertex transformation data
    pub struct Transform {
        /// Translation
        pub translation: Vector3,
        /// Rotation
        pub rotation: Quaternion,
        /// Scale
        pub scale: Vector3,
    }
}

impl From<ffi::Transform> for Transform {
    fn from(r: ffi::Transform) -> Transform {
        unsafe { std::mem::transmute(r) }
    }
}

impl From<Transform> for ffi::Transform {
    fn from(v: Transform) -> ffi::Transform {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<&Transform> for ffi::Transform {
    fn from(v: &Transform) -> ffi::Transform {
        unsafe { std::mem::transmute(*v) }
    }
}

#[cfg(test)]
mod math_test {
    use super::{Ray, Vector2, Vector3, Vector4};
    use crate::{ffi, math::Matrix};

    #[test]
    fn test_into() {
        let v2: ffi::Vector2 = Vector2 { x: 1.0, y: 2.0 };
        assert!(v2.x == 1.0 && v2.y == 2.0, "bad memory transmutation");

        let v3: ffi::Vector3 = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        assert!(
            v3.x == 1.0 && v3.y == 2.0 && v3.z == 3.0,
            "bad memory transmutation"
        );

        let v4: ffi::Vector4 = Vector4::new(1.0, 2.0, 3.0, 4.0);
        assert!(
            v4.x == 1.0 && v4.y == 2.0 && v4.z == 3.0 && v4.w == 4.0,
            "bad memory transmutation"
        );

        let r: ffi::Ray = (Ray {
            position: v3,
            direction: Vector3 {
                x: 3.0,
                y: 2.0,
                z: 1.0,
            },
        })
        .into();
        assert!(
            r.position.x == 1.0
                && r.position.y == 2.0
                && r.position.z == 3.0
                && r.direction.x == 3.0
                && r.direction.y == 2.0
                && r.direction.z == 1.0,
            "bad memory transmutation"
        );
        // raylib's Matrix is 16 named floats (m0..m15), column-major; identity has
        // 1.0 on the diagonal (m0/m5/m10/m15) and 0.0 elsewhere.
        let identity_mat: ffi::Matrix = Matrix::identity();
        assert!(
            identity_mat.m0 == 1.0
                && identity_mat.m5 == 1.0
                && identity_mat.m10 == 1.0
                && identity_mat.m15 == 1.0
                && identity_mat.m1 == 0.0
                && identity_mat.m4 == 0.0,
            "identity matrix should be 1.0 on the diagonal, 0.0 elsewhere"
        );
    }
}
