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
    unsafe { ffi::CheckCollisionPointCircle(point.into(), center.into(), radius) }
}

/// Check if point is within a polygon described by array of vertices
#[inline]
#[must_use]
pub fn check_collision_point_poly(point: impl Into<ffi::Vector2>, points: &[Vector2]) -> bool {
    unsafe {
        ffi::CheckCollisionPointPoly(
            point.into(),
            points.as_ptr(),
            points.len() as i32,
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

    let collision = unsafe {
        ffi::CheckCollisionLines(
            start_pos1.into(),
            end_pos1.into(),
            start_pos2.into(),
            end_pos2.into(),
            &mut out,
        )
    };
    if collision {
        Some(out)
    } else {
        None
    }
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
    unsafe { ffi::GetRayCollisionSphere(ray.into(), sphere_position.into(), sphere_radius).into() }
}

/// Gets collision info between ray and model.
#[inline]
#[must_use]
pub fn get_ray_collision_model(
    ray: impl Into<ffi::Ray>,
    model: &Mesh,
    transform: &Matrix,
) -> RayCollision {
    unsafe { ffi::GetRayCollisionMesh(ray.into(), model.0, *transform).into() }
}

/// Gets collision info between ray and triangle.
#[inline]
#[must_use]
pub fn get_ray_collision_triangle(
    ray: impl Into<ffi::Ray>,
    p1: impl Into<ffi::Vector3>,
    p2: impl Into<ffi::Vector3>,
    p3: impl Into<ffi::Vector3>,
) -> RayCollision {
    unsafe { ffi::GetRayCollisionTriangle(ray.into(), p1.into(), p2.into(), p3.into()).into() }
}

/// Gets collision info between ray and model.
#[inline]
#[must_use]
pub fn get_ray_collision_quad(
    ray: impl Into<ffi::Ray>,
    p1: impl Into<ffi::Vector3>,
    p2: impl Into<ffi::Vector3>,
    p3: impl Into<ffi::Vector3>,
    p4: impl Into<ffi::Vector3>,
) -> RayCollision {
    unsafe {
        ffi::GetRayCollisionQuad(ray.into(), p1.into(), p2.into(), p3.into(), p4.into()).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::{Vector2, Vector3};

    // --- check_collision_circles ---

    #[test]
    fn circles_overlap() {
        // centers 1 apart, radii sum = 4 → overlapping
        assert!(check_collision_circles(
            Vector2::new(0.0, 0.0),
            2.0,
            Vector2::new(1.0, 0.0),
            2.0
        ));
    }

    #[test]
    fn circles_separate() {
        // centers 10 apart, radii sum = 2 → no collision
        assert!(!check_collision_circles(
            Vector2::new(0.0, 0.0),
            1.0,
            Vector2::new(10.0, 0.0),
            1.0
        ));
    }

    #[test]
    fn circles_touching_boundary() {
        // circles just touching (distance == sum of radii); raylib treats this as collision
        assert!(check_collision_circles(
            Vector2::new(0.0, 0.0),
            1.0,
            Vector2::new(2.0, 0.0),
            1.0
        ));
    }

    // --- check_collision_circle_line ---

    #[test]
    fn circle_line_intersects() {
        // horizontal line y=0 from (-5,0) to (5,0), circle at (0,0.5) radius 1 → intersects
        assert!(check_collision_circle_line(
            Vector2::new(0.0, 0.5),
            1.0,
            Vector2::new(-5.0, 0.0),
            Vector2::new(5.0, 0.0),
        ));
    }

    #[test]
    fn circle_line_misses() {
        // horizontal line y=0, circle at (0,5) radius 1 → no intersection
        assert!(!check_collision_circle_line(
            Vector2::new(0.0, 5.0),
            1.0,
            Vector2::new(-5.0, 0.0),
            Vector2::new(5.0, 0.0),
        ));
    }

    // --- check_collision_point_circle ---

    #[test]
    fn point_inside_circle() {
        assert!(check_collision_point_circle(
            Vector2::new(0.5, 0.5),
            Vector2::new(0.0, 0.0),
            2.0,
        ));
    }

    #[test]
    fn point_outside_circle() {
        assert!(!check_collision_point_circle(
            Vector2::new(10.0, 10.0),
            Vector2::new(0.0, 0.0),
            1.0,
        ));
    }

    // --- check_collision_point_triangle ---

    #[test]
    fn point_inside_triangle() {
        // right triangle with vertices (0,0),(4,0),(0,4); centroid (4/3,4/3) is inside
        assert!(check_collision_point_triangle(
            Vector2::new(1.0, 1.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(0.0, 4.0),
        ));
    }

    #[test]
    fn point_outside_triangle() {
        assert!(!check_collision_point_triangle(
            Vector2::new(5.0, 5.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(4.0, 0.0),
            Vector2::new(0.0, 4.0),
        ));
    }

    // --- check_collision_point_poly ---

    #[test]
    fn point_inside_square_poly() {
        // unit square CCW: (0,0),(1,0),(1,1),(0,1)
        let square = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(0.0, 1.0),
        ];
        assert!(check_collision_point_poly(Vector2::new(0.5, 0.5), &square));
    }

    #[test]
    fn point_outside_square_poly() {
        let square = vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(0.0, 1.0),
        ];
        assert!(!check_collision_point_poly(Vector2::new(2.0, 2.0), &square));
    }

    // --- check_collision_point_line ---

    #[test]
    fn point_on_line_within_threshold() {
        // point (1,0) lies exactly on line from (0,0) to (10,0), threshold 1
        assert!(check_collision_point_line(
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            1,
        ));
    }

    #[test]
    fn point_far_from_line() {
        // point (5,5) is 5 units from line y=0, threshold 1 → no collision
        assert!(!check_collision_point_line(
            Vector2::new(5.0, 5.0),
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            1,
        ));
    }

    // --- check_collision_lines ---

    #[test]
    fn lines_cross() {
        // diagonal lines that cross at (0,0)
        let result = check_collision_lines(
            Vector2::new(-1.0, -1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(-1.0, 1.0),
            Vector2::new(1.0, -1.0),
        );
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!(
            (pt.x).abs() < 1e-4,
            "intersection x should be ~0, got {}",
            pt.x
        );
        assert!(
            (pt.y).abs() < 1e-4,
            "intersection y should be ~0, got {}",
            pt.y
        );
    }

    #[test]
    fn lines_parallel_no_collision() {
        // two parallel horizontal lines → None
        let result = check_collision_lines(
            Vector2::new(0.0, 0.0),
            Vector2::new(10.0, 0.0),
            Vector2::new(0.0, 1.0),
            Vector2::new(10.0, 1.0),
        );
        assert!(result.is_none());
    }

    // --- check_collision_spheres ---

    #[test]
    fn spheres_overlap_3d() {
        assert!(check_collision_spheres(
            Vector3::new(0.0, 0.0, 0.0),
            2.0,
            Vector3::new(1.0, 0.0, 0.0),
            2.0,
        ));
    }

    #[test]
    fn spheres_separate_3d() {
        assert!(!check_collision_spheres(
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
            Vector3::new(10.0, 0.0, 0.0),
            1.0,
        ));
    }
}
