//! Common collision handling code
use crate::core::math::Vector2;

use crate::ffi;
use crate::math::{Matrix, RayCollision};
use crate::models::Mesh;

/// Check if circle collides with a line created between two points [p1] and [p2]
#[inline]
#[must_use]
pub fn check_collision_circle_line(
    center: impl Into<ffi::Vector2>,
    radius: f32,
    p1: impl Into<ffi::Vector2>,
    p2: impl Into<ffi::Vector2>,
) -> bool {
    // SAFETY: CheckCollisionCircleLine is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::CheckCollisionCircleLine(center.into(), radius, p1.into(), p2.into()) }
}

// Collision Handling
/// Checks collision between two circles.
#[inline]
#[must_use]
pub fn check_collision_circles(
    center1: impl Into<ffi::Vector2>,
    radius1: f32,
    center2: impl Into<ffi::Vector2>,
    radius2: f32,
) -> bool {
    // SAFETY: CheckCollisionCircles is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::CheckCollisionCircles(center1.into(), radius1, center2.into(), radius2) }
}

/// Checks if point is inside circle.
#[inline]
#[must_use]
pub fn check_collision_point_circle(
    point: impl Into<ffi::Vector2>,
    center: impl Into<ffi::Vector2>,
    radius: f32,
) -> bool {
    // SAFETY: CheckCollisionPointCircle is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::CheckCollisionPointCircle(point.into(), center.into(), radius) }
}

/// Check if point is within a polygon described by array of vertices
///
/// # Panics
///
/// This function will panic if `points` has a length greater than [`i32::MAX`] elements.
#[inline]
#[must_use]
pub fn check_collision_point_poly(point: impl Into<ffi::Vector2>, points: &[Vector2]) -> bool {
    // SAFETY: `points` is guaranteed to be safe to dereference because it is a reference, and `pointCount` is guaranteed
    // to accurately describe the number of valid elements in the array because it is the length of the slice.
    unsafe {
        ffi::CheckCollisionPointPoly(
            point.into(),
            points.as_ptr().cast(),
            points
                .len()
                .try_into()
                .expect("points should not exceed i32::MAX elements"),
        )
    }
}

/// Check if point belongs to line created between two points [p1] and [p2] with defined margin in pixels [threshold]
#[inline]
#[must_use]
pub fn check_collision_point_line(
    point: impl Into<ffi::Vector2>,
    p1: impl Into<ffi::Vector2>,
    p2: impl Into<ffi::Vector2>,
    threshold: i32,
) -> bool {
    // SAFETY: CheckCollisionPointLine is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::CheckCollisionPointLine(point.into(), p1.into(), p2.into(), threshold) }
}

/// Checks if point is inside a triangle.
#[inline]
#[must_use]
pub fn check_collision_point_triangle(
    point: impl Into<ffi::Vector2>,
    p1: impl Into<ffi::Vector2>,
    p2: impl Into<ffi::Vector2>,
    p3: impl Into<ffi::Vector2>,
) -> bool {
    // SAFETY: CheckCollisionPointTriangle is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::CheckCollisionPointTriangle(point.into(), p1.into(), p2.into(), p3.into()) }
}

/// Check the collision between two lines defined by two points each, returns collision point by reference
#[inline]
#[must_use]
pub fn check_collision_lines(
    start_pos1: impl Into<ffi::Vector2>,
    end_pos1: impl Into<ffi::Vector2>,
    start_pos2: impl Into<ffi::Vector2>,
    end_pos2: impl Into<ffi::Vector2>,
) -> Option<Vector2> {
    let mut out = ffi::Vector2 { x: 0.0, y: 0.0 };

    // SAFETY: `out` is guaranteed to be safe to dereference and write to because it is a mutable reference
    // to a valid local variable.
    let collision = unsafe {
        ffi::CheckCollisionLines(
            start_pos1.into(),
            end_pos1.into(),
            start_pos2.into(),
            end_pos2.into(),
            &raw mut out,
        )
    };
    collision.then(|| out.into())
}

/// Detects collision between two spheres.
#[inline]
#[must_use]
pub fn check_collision_spheres(
    center_a: impl Into<ffi::Vector3>,
    radius_a: f32,
    center_b: impl Into<ffi::Vector3>,
    radius_b: f32,
) -> bool {
    // SAFETY: CheckCollisionSpheres is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::CheckCollisionSpheres(center_a.into(), radius_a, center_b.into(), radius_b) }
}

/// Detects collision between ray and sphere.
#[inline]
#[must_use]
pub fn get_ray_collision_sphere(
    ray: impl Into<ffi::Ray>,
    sphere_position: impl Into<ffi::Vector3>,
    sphere_radius: f32,
) -> RayCollision {
    // SAFETY: GetRayCollisionSphere is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::GetRayCollisionSphere(ray.into(), sphere_position.into(), sphere_radius).into() }
}

/// Gets collision info between ray and mesh.
///
/// # Safety
///
/// If `mesh.vertices` is null, this function does nothing dangerous and returns [`RayCollision::default()`].
///
/// If `mesh.vertices` is *not* null, it must point to valid, initialized data that is safe to dereference
/// for reading when cast to [`*const ffi::Vector3`](ffi::Vector3).
///
/// If `mesh.indices` is null, `mesh.vertices` must point to an array of at least `mesh.triangles` valid,
/// initialized [`ffi::Vector3`] **triplets** (`[ffi::Vector3; mesh.triangles * 3]`).
///
/// If `mesh.indices` is *not* null, `mesh.indices` must point to valid, initialized data that is safe to
/// dereference for reading at least `mesh.triangles` **triplets** (`[u16; mesh.triangles * 3]`).
/// In this case, every element in `mesh.indices` must be a valid index that is in-bounds of `mesh.vertices`.
/// `mesh.vertices` is not required to have `mesh.triangles` elements, it is only required to have at least as
/// many elements as 1 + the greatest index present in `mesh.indices`.
#[inline]
#[must_use]
pub unsafe fn get_ray_collision_mesh(
    ray: impl Into<ffi::Ray>,
    mesh: &Mesh,
    transform: &Matrix,
) -> RayCollision {
    // SAFETY: Caller must uphold safety contract
    unsafe { ffi::GetRayCollisionMesh(ray.into(), mesh.0, transform.into()).into() }
}

/// Gets collision info between ray and model.
///
/// # Safety
///
/// See [`get_ray_collision_mesh`]
#[inline]
#[must_use]
#[deprecated = "renamed to `get_ray_collision_mesh` to reflect the name of the Raylib function it calls"]
pub unsafe fn get_ray_collision_model(
    ray: impl Into<ffi::Ray>,
    model: &Mesh,
    transform: &Matrix,
) -> RayCollision {
    // SAFETY: Caller must uphold safety contract
    unsafe { get_ray_collision_mesh(ray, model, transform) }
}

/// Get collision info between ray and triangle
///
/// NOTE: The points are expected to be in counter-clockwise winding
///
/// NOTE: Based on https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
#[inline]
#[must_use]
pub fn get_ray_collision_triangle(
    ray: impl Into<ffi::Ray>,
    p1: impl Into<ffi::Vector3>,
    p2: impl Into<ffi::Vector3>,
    p3: impl Into<ffi::Vector3>,
) -> RayCollision {
    // SAFETY: GetRayCollisionTriangle is a pure math function with no preconditions and is trivially safe
    unsafe { ffi::GetRayCollisionTriangle(ray.into(), p1.into(), p2.into(), p3.into()).into() }
}

/// Gets collision info between ray and model.
///
/// NOTE: The points are expected to be in counter-clockwise winding
#[inline]
#[must_use]
pub fn get_ray_collision_quad(
    ray: impl Into<ffi::Ray>,
    p1: impl Into<ffi::Vector3>,
    p2: impl Into<ffi::Vector3>,
    p3: impl Into<ffi::Vector3>,
    p4: impl Into<ffi::Vector3>,
) -> RayCollision {
    // SAFETY: GetRayCollisionQuad is a pure math function with no preconditions and is trivially safe
    unsafe {
        ffi::GetRayCollisionQuad(ray.into(), p1.into(), p2.into(), p3.into(), p4.into()).into()
    }
}
