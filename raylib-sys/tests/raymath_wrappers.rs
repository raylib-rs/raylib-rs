#![allow(clippy::approx_constant)]
use raylib_sys::{
    Matrix, Quaternion, Vector2, Vector3, Vector4, matrix_decompose, quaternion_to_axis_angle,
    vector3_ortho_normalize as ortho_normalize,
};

// Helper: component-wise float epsilon check for vectors and scalars.
macro_rules! assert_f32_eq {
    ($got:expr, $exp:expr) => {{
        let g = $got;
        let e = $exp;
        assert!(
            (g - e).abs() < 1e-4,
            "assert_f32_eq! failed: got {}, expected {}",
            g,
            e
        );
    }};
}

macro_rules! assert_v2_eq {
    ($got:expr, $exp:expr) => {{
        let g = $got;
        let e = $exp;
        assert!(
            (g.x - e.x).abs() < 1e-4 && (g.y - e.y).abs() < 1e-4,
            "assert_v2_eq! failed: got ({}, {}), expected ({}, {})",
            g.x,
            g.y,
            e.x,
            e.y
        );
    }};
}

macro_rules! assert_v3_eq {
    ($got:expr, $exp:expr) => {{
        let g = $got;
        let e = $exp;
        assert!(
            (g.x - e.x).abs() < 1e-4 && (g.y - e.y).abs() < 1e-4 && (g.z - e.z).abs() < 1e-4,
            "assert_v3_eq! failed: got ({}, {}, {}), expected ({}, {}, {})",
            g.x,
            g.y,
            g.z,
            e.x,
            e.y,
            e.z
        );
    }};
}

macro_rules! assert_v4_eq {
    ($got:expr, $exp:expr) => {{
        let g = $got;
        let e = $exp;
        assert!(
            (g.x - e.x).abs() < 1e-4
                && (g.y - e.y).abs() < 1e-4
                && (g.z - e.z).abs() < 1e-4
                && (g.w - e.w).abs() < 1e-4,
            "assert_v4_eq! failed: got ({}, {}, {}, {}), expected ({}, {}, {}, {})",
            g.x,
            g.y,
            g.z,
            g.w,
            e.x,
            e.y,
            e.z,
            e.w
        );
    }};
}

// ─── Vector2 ─────────────────────────────────────────────────────────────────

#[test]
fn vector2_core_ops() {
    assert_eq!(Vector2::new(3.0, 4.0).length(), 5.0);
    let sum = Vector2::new(1.0, 0.0) + Vector2::new(0.0, 1.0);
    assert_eq!(sum, Vector2::new(1.0, 1.0));
    let scaled = Vector2::new(2.0, 3.0) * 2.0;
    assert_eq!(scaled, Vector2::new(4.0, 6.0));
    assert_eq!(-Vector2::new(1.0, -1.0), Vector2::new(-1.0, 1.0));
}

#[test]
fn vector2_arithmetic_wiring() {
    let a = Vector2::new(1.0, 2.0);
    let b = Vector2::new(4.0, 7.0);

    // add (commutative — distinct values check wiring)
    assert_v2_eq!(a + b, Vector2::new(5.0, 9.0));

    // subtract is NON-commutative: b - a vs a - b differ
    assert_v2_eq!(b - a, Vector2::new(3.0, 5.0));
    assert_v2_eq!(a - b, Vector2::new(-3.0, -5.0));

    // add_value / sub_value
    assert_v2_eq!(a.add_value(10.0), Vector2::new(11.0, 12.0));
    assert_v2_eq!(a.sub_value(0.5), Vector2::new(0.5, 1.5));

    // scale (operand order matters: v*s vs s*v — only one direction defined)
    assert_v2_eq!(a.scale(3.0), Vector2::new(3.0, 6.0));
    assert_v2_eq!(a * 3.0, Vector2::new(3.0, 6.0));

    // multiply component-wise (symmetric but distinct components catch component swap)
    assert_v2_eq!(a.multiply(b), Vector2::new(4.0, 14.0));
    assert_v2_eq!(a * b, Vector2::new(4.0, 14.0));

    // divide component-wise (NON-commutative)
    assert_v2_eq!(b.divide(a), Vector2::new(4.0, 3.5));
    assert_v2_eq!(a.divide(b), Vector2::new(0.25, 2.0 / 7.0));
    assert_v2_eq!(b / a, Vector2::new(4.0, 3.5));

    // negate
    assert_v2_eq!(a.negate(), Vector2::new(-1.0, -2.0));
    assert_v2_eq!(-a, Vector2::new(-1.0, -2.0));
}

#[test]
fn vector2_length_and_distance() {
    let a = Vector2::new(3.0, 4.0);
    // length
    assert_f32_eq!(a.length(), 5.0);
    // length_sqr: no sqrt
    assert_f32_eq!(a.length_sqr(), 25.0);

    let b = Vector2::new(6.0, 8.0); // distance = 5
    assert_f32_eq!(a.distance(b), 5.0);
    assert_f32_eq!(a.distance_sqr(b), 25.0);
}

#[test]
fn vector2_dot_and_cross() {
    let a = Vector2::new(2.0, 3.0);
    let b = Vector2::new(5.0, 7.0);

    // dot: 2*5 + 3*7 = 10 + 21 = 31
    assert_f32_eq!(a.dot(b), 31.0);
    // cross (scalar 2D): a.x*b.y - a.y*b.x = 14 - 15 = -1
    // Order matters: b.cross(a) should be +1
    assert_f32_eq!(a.cross(b), -1.0);
    assert_f32_eq!(b.cross(a), 1.0);
}

#[test]
fn vector2_normalize() {
    let v = Vector2::new(0.0, 5.0);
    let n = v.normalize();
    assert_v2_eq!(n, Vector2::new(0.0, 1.0));

    // non-trivial: (3,4) -> (0.6, 0.8)
    let v2 = Vector2::new(3.0, 4.0);
    let n2 = v2.normalize();
    assert_f32_eq!(n2.x, 0.6);
    assert_f32_eq!(n2.y, 0.8);
}

#[test]
fn vector2_lerp() {
    let a = Vector2::new(0.0, 0.0);
    let b = Vector2::new(10.0, 20.0);
    // lerp(a, b, 0.3) = (3, 6)
    assert_v2_eq!(a.lerp(b, 0.3), Vector2::new(3.0, 6.0));
    // lerp is NON-commutative: b.lerp(a, 0.3) = (7, 14)
    assert_v2_eq!(b.lerp(a, 0.3), Vector2::new(7.0, 14.0));
}

#[test]
fn vector2_reflect() {
    // Reflect (1, -1) off horizontal surface (normal = (0,1))
    // result = v - 2*(v.n)*n = (1,-1) - 2*(-1)*(0,1) = (1,1)
    let v = Vector2::new(1.0, -1.0);
    let n = Vector2::new(0.0, 1.0);
    assert_v2_eq!(v.reflect(n), Vector2::new(1.0, 1.0));
    // Order matters: n.reflect(v) is completely different
    // n=(0,1) reflected off v=(1,-1) (not normalised, but check it differs)
    let reflected_swapped = n.reflect(v);
    assert!(
        (reflected_swapped.x - 1.0).abs() > 0.1 || (reflected_swapped.y - 1.0).abs() > 0.1,
        "reflect operand order appears not to matter (mis-wire?)"
    );
}

#[test]
fn vector2_min_max_clamp() {
    let a = Vector2::new(2.0, 7.0);
    let b = Vector2::new(5.0, 3.0);
    assert_v2_eq!(a.min(b), Vector2::new(2.0, 3.0));
    assert_v2_eq!(a.max(b), Vector2::new(5.0, 7.0));

    // clamp: v clamped between (0,0) and (3,6) -> (2,6) [y was 7, clamped to 6]
    let lo = Vector2::new(0.0, 0.0);
    let hi = Vector2::new(3.0, 6.0);
    assert_v2_eq!(a.clamp(lo, hi), Vector2::new(2.0, 6.0));

    // clamp_value: length clamped to [0, 4] — a=(2,7), len≈7.28, so scale to len=4
    let clamped = a.clamp_value(0.0, 4.0);
    assert_f32_eq!(clamped.length(), 4.0);
}

#[test]
fn vector2_rotate_and_move_towards() {
    // Rotate (1,0) by 90 degrees (pi/2) -> (0,1)
    use std::f32::consts::FRAC_PI_2;
    let v = Vector2::new(1.0, 0.0);
    let r = v.rotate(FRAC_PI_2);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 1.0);

    // move_towards: from (0,0) towards (10,0) by 3 -> (3,0)
    let from = Vector2::new(0.0, 0.0);
    let to = Vector2::new(10.0, 0.0);
    assert_v2_eq!(from.move_towards(to, 3.0), Vector2::new(3.0, 0.0));
    // move_towards is NON-commutative
    let back = to.move_towards(from, 3.0);
    assert_v2_eq!(back, Vector2::new(7.0, 0.0));
}

#[test]
fn vector2_invert_and_zero_one() {
    // invert: 1/x, 1/y
    let v = Vector2::new(2.0, 4.0);
    assert_v2_eq!(v.invert(), Vector2::new(0.5, 0.25));

    // zero / one
    assert_v2_eq!(Vector2::zero(), Vector2::new(0.0, 0.0));
    assert_v2_eq!(Vector2::one(), Vector2::new(1.0, 1.0));
}

#[test]
fn vector2_equals_and_angle() {
    let a = Vector2::new(1.0, 0.0);
    let b = Vector2::new(0.0, 1.0);
    assert!(!a.equals(b));
    assert!(a.equals(Vector2::new(1.0, 0.0)));

    // angle between (1,0) and (0,1) = 90 degrees = pi/2
    use std::f32::consts::FRAC_PI_2;
    assert_f32_eq!(a.angle(b), FRAC_PI_2);
}

#[test]
fn vector2_transform() {
    // Transform (1,0) by a 90-degree Z-rotation matrix
    // MatrixRotateZ(pi/2): (1,0) -> (0,1)
    use std::f32::consts::FRAC_PI_2;
    let m = Matrix::rotate_z(FRAC_PI_2);
    let v = Vector2::new(1.0, 0.0);
    let t = v.transform(m);
    assert_f32_eq!(t.x, 0.0);
    assert_f32_eq!(t.y, 1.0);
}

// ─── Vector3 ─────────────────────────────────────────────────────────────────

#[test]
fn vector3_core_ops() {
    let a = Vector3::new(1.0, 2.0, 2.0);
    assert_eq!(a.length(), 3.0); // sqrt(1+4+4)
    assert_eq!(
        Vector3::new(1.0, 0.0, 0.0).dot(Vector3::new(0.0, 1.0, 0.0)),
        0.0
    );
    let c = Vector3::new(1.0, 0.0, 0.0).cross(Vector3::new(0.0, 1.0, 0.0));
    assert_eq!(c, Vector3::new(0.0, 0.0, 1.0));
    assert_eq!(
        Vector3::new(1.0, 1.0, 1.0) + Vector3::new(1.0, 2.0, 3.0),
        Vector3::new(2.0, 3.0, 4.0)
    );
    // normalize
    let n = Vector3::new(3.0, 0.0, 0.0).normalize();
    assert_eq!(n, Vector3::new(1.0, 0.0, 0.0));
    // lerp
    let mid = Vector3::new(0.0, 0.0, 0.0).lerp(Vector3::new(2.0, 2.0, 2.0), 0.5);
    assert_eq!(mid, Vector3::new(1.0, 1.0, 1.0));
}

#[test]
fn vector3_arithmetic_wiring() {
    let a = Vector3::new(1.0, 2.0, 3.0);
    let b = Vector3::new(4.0, 5.0, 6.0);

    // add
    assert_v3_eq!(a + b, Vector3::new(5.0, 7.0, 9.0));
    // subtract NON-commutative
    assert_v3_eq!(b - a, Vector3::new(3.0, 3.0, 3.0));
    assert_v3_eq!(a - b, Vector3::new(-3.0, -3.0, -3.0));

    // add_value / sub_value
    assert_v3_eq!(a.add_value(10.0), Vector3::new(11.0, 12.0, 13.0));
    assert_v3_eq!(a.sub_value(0.5), Vector3::new(0.5, 1.5, 2.5));

    // scale
    assert_v3_eq!(a.scale(2.0), Vector3::new(2.0, 4.0, 6.0));
    assert_v3_eq!(a * 2.0, Vector3::new(2.0, 4.0, 6.0));

    // multiply component-wise (distinct components check swap)
    assert_v3_eq!(a.multiply(b), Vector3::new(4.0, 10.0, 18.0));
    assert_v3_eq!(a * b, Vector3::new(4.0, 10.0, 18.0));

    // divide (NON-commutative)
    assert_v3_eq!(b.divide(a), Vector3::new(4.0, 2.5, 2.0));
    assert_v3_eq!(b / a, Vector3::new(4.0, 2.5, 2.0));

    // negate
    assert_v3_eq!(a.negate(), Vector3::new(-1.0, -2.0, -3.0));
    assert_v3_eq!(-a, Vector3::new(-1.0, -2.0, -3.0));
}

#[test]
fn vector3_dot_cross_wiring() {
    let a = Vector3::new(1.0, 2.0, 3.0);
    let b = Vector3::new(4.0, 5.0, 6.0);

    // dot: 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    assert_f32_eq!(a.dot(b), 32.0);

    // cross is NON-commutative
    // a x b = (2*6-3*5, 3*4-1*6, 1*5-2*4) = (12-15, 12-6, 5-8) = (-3, 6, -3)
    assert_v3_eq!(a.cross(b), Vector3::new(-3.0, 6.0, -3.0));
    // b x a = -( a x b ) = (3, -6, 3)
    assert_v3_eq!(b.cross(a), Vector3::new(3.0, -6.0, 3.0));
}

#[test]
fn vector3_length_and_distance() {
    let a = Vector3::new(1.0, 2.0, 3.0);
    // length = sqrt(1+4+9) = sqrt(14)
    assert_f32_eq!(a.length(), 14.0_f32.sqrt());
    // length_sqr = 14
    assert_f32_eq!(a.length_sqr(), 14.0);

    let b = Vector3::new(2.0, 4.0, 6.0);
    // distance(a,b) = |b-a| = |(1,2,3)| = sqrt(14)
    assert_f32_eq!(a.distance(b), 14.0_f32.sqrt());
    assert_f32_eq!(a.distance_sqr(b), 14.0);
}

#[test]
fn vector3_normalize() {
    let v = Vector3::new(2.0, 0.0, 0.0);
    assert_v3_eq!(v.normalize(), Vector3::new(1.0, 0.0, 0.0));

    // non-trivial: (1,2,2) -> length=3, so (1/3, 2/3, 2/3)
    let v2 = Vector3::new(1.0, 2.0, 2.0);
    let n = v2.normalize();
    assert_f32_eq!(n.x, 1.0 / 3.0);
    assert_f32_eq!(n.y, 2.0 / 3.0);
    assert_f32_eq!(n.z, 2.0 / 3.0);
}

#[test]
fn vector3_angle() {
    // angle between (1,0,0) and (0,1,0) = pi/2
    use std::f32::consts::FRAC_PI_2;
    let a = Vector3::new(1.0, 0.0, 0.0);
    let b = Vector3::new(0.0, 1.0, 0.0);
    assert_f32_eq!(a.angle(b), FRAC_PI_2);
}

#[test]
fn vector3_lerp_non_commutative() {
    let a = Vector3::new(0.0, 0.0, 0.0);
    let b = Vector3::new(10.0, 20.0, 30.0);
    assert_v3_eq!(a.lerp(b, 0.3), Vector3::new(3.0, 6.0, 9.0));
    // b.lerp(a, 0.3) = b + 0.3*(a-b) = (7, 14, 21)
    assert_v3_eq!(b.lerp(a, 0.3), Vector3::new(7.0, 14.0, 21.0));
}

#[test]
fn vector3_reflect() {
    // v = (1,-1,0) reflected off normal (0,1,0) -> (1,1,0)
    let v = Vector3::new(1.0, -1.0, 0.0);
    let n = Vector3::new(0.0, 1.0, 0.0);
    assert_v3_eq!(v.reflect(n), Vector3::new(1.0, 1.0, 0.0));
    // NON-commutative check
    let r2 = n.reflect(v);
    // n=(0,1,0) reflected off v=(1,-1,0): different result
    assert!(
        (r2.x - 1.0).abs() > 0.1 || (r2.y - 1.0).abs() > 0.1,
        "reflect operand order appears symmetric (mis-wire?)"
    );
}

#[test]
fn vector3_project_and_reject() {
    // project (1,2,0) onto x-axis (1,0,0) -> (1,0,0)
    let v = Vector3::new(1.0, 2.0, 0.0);
    let axis = Vector3::new(1.0, 0.0, 0.0);
    assert_v3_eq!(v.project(axis), Vector3::new(1.0, 0.0, 0.0));
    // reject should be perpendicular component: (0,2,0)
    assert_v3_eq!(v.reject(axis), Vector3::new(0.0, 2.0, 0.0));
    // NON-commutative: project is order-sensitive
    assert_v3_eq!(axis.project(v), Vector3::new(1.0 / 5.0, 2.0 / 5.0, 0.0));
}

#[test]
fn vector3_min_max_clamp() {
    let a = Vector3::new(2.0, 7.0, 1.0);
    let b = Vector3::new(5.0, 3.0, 8.0);
    assert_v3_eq!(a.min(b), Vector3::new(2.0, 3.0, 1.0));
    assert_v3_eq!(a.max(b), Vector3::new(5.0, 7.0, 8.0));

    let lo = Vector3::new(0.0, 0.0, 0.0);
    let hi = Vector3::new(3.0, 6.0, 5.0);
    // a = (2,7,1), clamped to (3,6,5) hi -> (2,6,1)
    assert_v3_eq!(a.clamp(lo, hi), Vector3::new(2.0, 6.0, 1.0));

    // clamp_value: length of (2,7,1) ≈ 7.35, clamped to max=5
    let cv = a.clamp_value(0.0, 5.0);
    assert_f32_eq!(cv.length(), 5.0);
}

#[test]
fn vector3_rotate_by_quaternion() {
    // Rotate (1,0,0) by 90-degree rotation around Z -> (0,1,0)
    use std::f32::consts::FRAC_PI_2;
    let q = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);
    let v = Vector3::new(1.0, 0.0, 0.0);
    let r = v.rotate_by_quaternion(q);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 1.0);
    assert_f32_eq!(r.z, 0.0);
}

#[test]
fn vector3_rotate_by_axis_angle() {
    use std::f32::consts::FRAC_PI_2;
    // Rotate (1,0,0) around Z axis by 90 degrees -> (0,1,0)
    let v = Vector3::new(1.0, 0.0, 0.0);
    let axis = Vector3::new(0.0, 0.0, 1.0);
    let r = v.rotate_by_axis_angle(axis, FRAC_PI_2);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 1.0);
    assert_f32_eq!(r.z, 0.0);
    // NON-commutative (axis swap would change result)
    let r2 = v.rotate_by_axis_angle(Vector3::new(0.0, 1.0, 0.0), FRAC_PI_2);
    assert_f32_eq!(r2.x, 0.0);
    assert_f32_eq!(r2.y, 0.0);
    assert_f32_eq!(r2.z, -1.0);
}

#[test]
fn vector3_move_towards() {
    let from = Vector3::new(0.0, 0.0, 0.0);
    let to = Vector3::new(10.0, 0.0, 0.0);
    assert_v3_eq!(from.move_towards(to, 3.0), Vector3::new(3.0, 0.0, 0.0));
    // NON-commutative
    assert_v3_eq!(to.move_towards(from, 3.0), Vector3::new(7.0, 0.0, 0.0));
}

#[test]
fn vector3_invert_and_zero_one() {
    assert_v3_eq!(Vector3::zero(), Vector3::new(0.0, 0.0, 0.0));
    assert_v3_eq!(Vector3::one(), Vector3::new(1.0, 1.0, 1.0));
    assert_v3_eq!(
        Vector3::new(2.0, 4.0, 8.0).invert(),
        Vector3::new(0.5, 0.25, 0.125)
    );
}

#[test]
fn vector3_transform_by_matrix() {
    // translate (1,0,0) by (5,0,0) -> (6,0,0)
    let m = Matrix::translate(5.0, 0.0, 0.0);
    let v = Vector3::new(1.0, 0.0, 0.0);
    let t = v.transform(m);
    assert_f32_eq!(t.x, 6.0);
    assert_f32_eq!(t.y, 0.0);
    assert_f32_eq!(t.z, 0.0);
}

#[test]
fn vector3_equals() {
    assert!(Vector3::new(1.0, 2.0, 3.0).equals(Vector3::new(1.0, 2.0, 3.0)));
    assert!(!Vector3::new(1.0, 2.0, 3.0).equals(Vector3::new(1.0, 2.0, 4.0)));
}

#[test]
fn vector3_to_float_array() {
    let v = Vector3::new(7.0, 8.0, 9.0);
    let arr = v.to_float_array();
    assert_f32_eq!(arr.v[0], 7.0);
    assert_f32_eq!(arr.v[1], 8.0);
    assert_f32_eq!(arr.v[2], 9.0);
}

#[test]
fn vector3_ortho_normalize_test() {
    let mut v1 = Vector3::new(1.0, 1.0, 0.0);
    let mut v2 = Vector3::new(0.0, 1.0, 0.0);
    ortho_normalize(&mut v1, &mut v2);
    // v1 should be normalized
    assert_f32_eq!(v1.length(), 1.0);
    // v2 should be normalized
    assert_f32_eq!(v2.length(), 1.0);
    // v1 and v2 should be orthogonal
    assert_f32_eq!(v1.dot(v2), 0.0);
}

#[test]
fn vector3_cubic_hermite() {
    // At t=0 should return v0 (self), at t=1 should return v1
    let v0 = Vector3::new(0.0, 0.0, 0.0);
    let t0 = Vector3::new(1.0, 0.0, 0.0); // tangent at start
    let v1 = Vector3::new(1.0, 0.0, 0.0);
    let t1 = Vector3::new(1.0, 0.0, 0.0); // tangent at end
    let at_start = v0.cubic_hermite(t0, v1, t1, 0.0);
    assert_v3_eq!(at_start, Vector3::new(0.0, 0.0, 0.0));
    let at_end = v0.cubic_hermite(t0, v1, t1, 1.0);
    assert_v3_eq!(at_end, Vector3::new(1.0, 0.0, 0.0));
}

#[test]
fn vector3_barycenter() {
    // Centroid of a triangle = barycenter of midpoint
    let a = Vector3::new(0.0, 0.0, 0.0);
    let b = Vector3::new(3.0, 0.0, 0.0);
    let c = Vector3::new(0.0, 3.0, 0.0);
    // centroid = (1, 1, 0)
    let centroid = Vector3::barycenter(Vector3::new(1.0, 1.0, 0.0), a, b, c);
    // barycentric coords should sum to ~1 and each be ~1/3
    assert_f32_eq!(centroid.x + centroid.y + centroid.z, 1.0);
    assert_f32_eq!(centroid.x, 1.0 / 3.0);
    assert_f32_eq!(centroid.y, 1.0 / 3.0);
    assert_f32_eq!(centroid.z, 1.0 / 3.0);
}

// ─── Vector4 ─────────────────────────────────────────────────────────────────

#[test]
fn vector4_core_ops() {
    let a = Vector4::new(1.0, 0.0, 0.0, 0.0);
    assert!((a.length() - 1.0).abs() < 1e-6);
    let sum = Vector4::new(1.0, 2.0, 3.0, 4.0) + Vector4::new(1.0, 1.0, 1.0, 1.0);
    assert_eq!(sum, Vector4::new(2.0, 3.0, 4.0, 5.0));
}

#[test]
fn vector4_arithmetic_wiring() {
    let a = Vector4::new(1.0, 2.0, 3.0, 4.0);
    let b = Vector4::new(5.0, 6.0, 7.0, 8.0);

    // add
    assert_v4_eq!(a + b, Vector4::new(6.0, 8.0, 10.0, 12.0));
    // subtract NON-commutative
    assert_v4_eq!(b - a, Vector4::new(4.0, 4.0, 4.0, 4.0));
    assert_v4_eq!(a - b, Vector4::new(-4.0, -4.0, -4.0, -4.0));

    // add_value / sub_value
    assert_v4_eq!(a.add_value(10.0), Vector4::new(11.0, 12.0, 13.0, 14.0));
    assert_v4_eq!(a.sub_value(0.5), Vector4::new(0.5, 1.5, 2.5, 3.5));

    // scale
    assert_v4_eq!(a.scale(3.0), Vector4::new(3.0, 6.0, 9.0, 12.0));
    assert_v4_eq!(a * 3.0, Vector4::new(3.0, 6.0, 9.0, 12.0));

    // multiply component-wise
    assert_v4_eq!(a.multiply(b), Vector4::new(5.0, 12.0, 21.0, 32.0));
    assert_v4_eq!(a * b, Vector4::new(5.0, 12.0, 21.0, 32.0));

    // divide component-wise (NON-commutative)
    assert_v4_eq!(b.divide(a), Vector4::new(5.0, 3.0, 7.0 / 3.0, 2.0));
    assert_v4_eq!(b / a, Vector4::new(5.0, 3.0, 7.0 / 3.0, 2.0));

    // negate
    assert_v4_eq!(a.negate(), Vector4::new(-1.0, -2.0, -3.0, -4.0));
    assert_v4_eq!(-a, Vector4::new(-1.0, -2.0, -3.0, -4.0));
}

#[test]
fn vector4_length_and_distance() {
    let a = Vector4::new(1.0, 2.0, 3.0, 4.0);
    // length = sqrt(1+4+9+16) = sqrt(30)
    assert_f32_eq!(a.length(), 30.0_f32.sqrt());
    assert_f32_eq!(a.length_sqr(), 30.0);

    let b = Vector4::new(2.0, 4.0, 6.0, 8.0);
    // distance = |(1,2,3,4)| = sqrt(30)
    assert_f32_eq!(a.distance(b), 30.0_f32.sqrt());
    assert_f32_eq!(a.distance_sqr(b), 30.0);
}

#[test]
fn vector4_dot() {
    let a = Vector4::new(1.0, 2.0, 3.0, 4.0);
    let b = Vector4::new(2.0, 3.0, 4.0, 5.0);
    // dot = 1*2 + 2*3 + 3*4 + 4*5 = 2+6+12+20 = 40
    assert_f32_eq!(a.dot(b), 40.0);
}

#[test]
fn vector4_normalize() {
    let v = Vector4::new(2.0, 0.0, 0.0, 0.0);
    assert_v4_eq!(v.normalize(), Vector4::new(1.0, 0.0, 0.0, 0.0));
}

#[test]
fn vector4_min_max() {
    let a = Vector4::new(1.0, 5.0, 2.0, 8.0);
    let b = Vector4::new(3.0, 2.0, 7.0, 4.0);
    assert_v4_eq!(a.min(b), Vector4::new(1.0, 2.0, 2.0, 4.0));
    assert_v4_eq!(a.max(b), Vector4::new(3.0, 5.0, 7.0, 8.0));
}

#[test]
fn vector4_lerp() {
    let a = Vector4::new(0.0, 0.0, 0.0, 0.0);
    let b = Vector4::new(10.0, 20.0, 30.0, 40.0);
    assert_v4_eq!(a.lerp(b, 0.5), Vector4::new(5.0, 10.0, 15.0, 20.0));
    // NON-commutative
    assert_v4_eq!(b.lerp(a, 0.5), Vector4::new(5.0, 10.0, 15.0, 20.0)); // symmetric at 0.5
    assert_v4_eq!(a.lerp(b, 0.25), Vector4::new(2.5, 5.0, 7.5, 10.0));
    assert_v4_eq!(b.lerp(a, 0.25), Vector4::new(7.5, 15.0, 22.5, 30.0)); // non-commutative at other t
}

#[test]
fn vector4_move_towards() {
    let from = Vector4::new(0.0, 0.0, 0.0, 0.0);
    let to = Vector4::new(10.0, 0.0, 0.0, 0.0);
    let r = from.move_towards(to, 3.0);
    assert_f32_eq!(r.x, 3.0);
    // NON-commutative
    let r2 = to.move_towards(from, 3.0);
    assert_f32_eq!(r2.x, 7.0);
}

#[test]
fn vector4_invert_and_zero_one() {
    assert_v4_eq!(Vector4::zero(), Vector4::new(0.0, 0.0, 0.0, 0.0));
    assert_v4_eq!(Vector4::one(), Vector4::new(1.0, 1.0, 1.0, 1.0));
    assert_v4_eq!(
        Vector4::new(2.0, 4.0, 8.0, 16.0).invert(),
        Vector4::new(0.5, 0.25, 0.125, 0.0625)
    );
}

#[test]
fn vector4_equals() {
    let a = Vector4::new(1.0, 2.0, 3.0, 4.0);
    assert!(a.equals(Vector4::new(1.0, 2.0, 3.0, 4.0)));
    assert!(!a.equals(Vector4::new(1.0, 2.0, 3.0, 5.0)));
}

// ─── Matrix ──────────────────────────────────────────────────────────────────

#[test]
fn matrix_identity_is_neutral() {
    let id = Matrix::identity();
    let m = Matrix::translate(1.0, 2.0, 3.0);
    assert_eq!(id * m, m);
    assert_eq!(id.determinant(), 1.0);
}

#[test]
fn matrix_translate_wiring() {
    // MatrixTranslate(x,y,z): C struct field order is {m0,m4,m8,m12, m1,m5,m9,m13, m2,m6,m10,m14, m3,m7,m11,m15}
    // Init: {1,0,0,x, 0,1,0,y, 0,0,1,z, 0,0,0,1}
    // So: m12=x, m13=y, m14=z
    let t = Matrix::translate(7.0, 13.0, 17.0);
    // Distinct values: x=7, y=13, z=17 — any component swap would fail
    assert_f32_eq!(t.m12, 7.0);
    assert_f32_eq!(t.m13, 13.0);
    assert_f32_eq!(t.m14, 17.0);
    // diagonal should be 1
    assert_f32_eq!(t.m0, 1.0);
    assert_f32_eq!(t.m5, 1.0);
    assert_f32_eq!(t.m10, 1.0);
    assert_f32_eq!(t.m15, 1.0);
    // translation slots m3, m7, m11 should be 0
    assert_f32_eq!(t.m3, 0.0);
    assert_f32_eq!(t.m7, 0.0);
    assert_f32_eq!(t.m11, 0.0);
}

#[test]
fn matrix_scale_wiring() {
    // MatrixScale(x,y,z): init {x,0,0,0, 0,y,0,0, 0,0,z,0, 0,0,0,1}
    // In struct field order {m0,m4,m8,m12, m1,m5,m9,m13, m2,m6,m10,m14, m3,m7,m11,m15}:
    // m0=x, m5=y, m10=z, m15=1
    let s = Matrix::scale(2.0, 3.0, 5.0);
    // Distinct values ensure no component swap is silent
    assert_f32_eq!(s.m0, 2.0);
    assert_f32_eq!(s.m5, 3.0);
    assert_f32_eq!(s.m10, 5.0);
    assert_f32_eq!(s.m15, 1.0);
    // translation slots should be 0
    assert_f32_eq!(s.m12, 0.0);
    assert_f32_eq!(s.m13, 0.0);
    assert_f32_eq!(s.m14, 0.0);
}

#[test]
fn matrix_multiply_non_commutative() {
    // translate(1,0,0) * scale(2,2,2) != scale(2,2,2) * translate(1,0,0)
    // Translation is in m12 (x-component)
    let t = Matrix::translate(1.0, 0.0, 0.0);
    let s = Matrix::scale(2.0, 2.0, 2.0);
    let ts = t * s;
    let st = s * t;
    // They must differ in the translation slot m12
    assert!(
        (ts.m12 - st.m12).abs() > 0.1,
        "MatrixMultiply appears commutative (mis-wire?): ts.m12={} st.m12={}",
        ts.m12,
        st.m12
    );
    // Verify the specific values via vector transform:
    // t*s applied to (1,0,0): result.x = (1)*ts.m0 + ts.m12 = 2 + 2 = 4
    // s*t applied to (1,0,0): result.x = (1)*st.m0 + st.m12 = 2 + 1 = 3
    let v = Vector3::new(1.0, 0.0, 0.0);
    let r_ts = v.transform(ts);
    let r_st = v.transform(st);
    assert_f32_eq!(r_ts.x, 4.0); // scale 2 then translate by 1*scale = 2: 2+2
    assert_f32_eq!(r_st.x, 3.0); // translate by 1 then scale: 1+2=3? actually s*t: translate(1) then scale(2) = 2*(1+1)=?
    // These should differ
    assert!(
        (r_ts.x - r_st.x).abs() > 0.1,
        "ts and st produce same transform (mis-wire?)"
    );
}

#[test]
fn matrix_transpose() {
    // Translate matrix has values in m12=x, m13=y, m14=z (the 4th column in column-major)
    // Transpose swaps: m12 <-> m3, m13 <-> m7, m14 <-> m11
    let t = Matrix::translate(7.0, 13.0, 17.0);
    let tr = t.transpose();
    // After transpose, the values that were in the 4th column (m12,m13,m14) are now in 4th row (m3,m7,m11)
    assert_f32_eq!(tr.m3, 7.0);
    assert_f32_eq!(tr.m7, 13.0);
    assert_f32_eq!(tr.m11, 17.0);
    // And the original m3,m7,m11 slots (were 0) should still be 0 after transpose for these positions
    assert_f32_eq!(tr.m12, 0.0);
}

#[test]
fn matrix_determinant() {
    // identity det = 1
    assert_f32_eq!(Matrix::identity().determinant(), 1.0);
    // scale(2,3,5) det = 2*3*5 = 30
    assert_f32_eq!(Matrix::scale(2.0, 3.0, 5.0).determinant(), 30.0);
}

#[test]
fn matrix_trace() {
    // trace = sum of diagonal = m0 + m5 + m10 + m15
    // identity: 4.0
    assert_f32_eq!(Matrix::identity().trace(), 4.0);
    // scale(2,3,5): 2+3+5+1 = 11
    assert_f32_eq!(Matrix::scale(2.0, 3.0, 5.0).trace(), 11.0);
}

#[test]
fn matrix_invert() {
    // Inverse of translate(1,2,3) = translate(-1,-2,-3)
    let t = Matrix::translate(1.0, 2.0, 3.0);
    let inv = t.invert();
    // t * inv = identity
    let result = t * inv;
    assert_f32_eq!(result.m0, 1.0);
    assert_f32_eq!(result.m5, 1.0);
    assert_f32_eq!(result.m10, 1.0);
    assert_f32_eq!(result.m15, 1.0);
    // Translation slots (m12,m13,m14) should be 0 in the result
    assert_f32_eq!(result.m12, 0.0);
    assert_f32_eq!(result.m13, 0.0);
    assert_f32_eq!(result.m14, 0.0);
}

#[test]
fn matrix_mul_value() {
    // scale matrix * 2.0: each component doubled
    let s = Matrix::scale(2.0, 3.0, 5.0);
    let s2 = s.mul_value(2.0);
    assert_f32_eq!(s2.m0, 4.0);
    assert_f32_eq!(s2.m5, 6.0);
    assert_f32_eq!(s2.m10, 10.0);
}

#[test]
fn matrix_add_subtract() {
    let a = Matrix::translate(1.0, 2.0, 3.0);
    let b = Matrix::translate(4.0, 5.0, 6.0);
    let sum = a + b;
    // translation slots are m12, m13, m14
    assert_f32_eq!(sum.m12, 5.0);
    assert_f32_eq!(sum.m13, 7.0);
    assert_f32_eq!(sum.m14, 9.0);
    // diagonal adds (1+1=2)
    assert_f32_eq!(sum.m0, 2.0);

    let diff = b - a;
    assert_f32_eq!(diff.m12, 3.0);
    assert_f32_eq!(diff.m13, 3.0);
    assert_f32_eq!(diff.m14, 3.0);
    // NON-commutative subtract
    let diff2 = a - b;
    assert_f32_eq!(diff2.m12, -3.0);
}

#[test]
fn matrix_rotate_x_wiring() {
    // Rotate (0,1,0) around X by 90 degrees -> (0,0,1)
    use std::f32::consts::FRAC_PI_2;
    let m = Matrix::rotate_x(FRAC_PI_2);
    let v = Vector3::new(0.0, 1.0, 0.0);
    let r = v.transform(m);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 0.0);
    assert_f32_eq!(r.z, 1.0);
}

#[test]
fn matrix_rotate_y_wiring() {
    // Rotate (1,0,0) around Y by 90 degrees -> (0,0,-1)
    use std::f32::consts::FRAC_PI_2;
    let m = Matrix::rotate_y(FRAC_PI_2);
    let v = Vector3::new(1.0, 0.0, 0.0);
    let r = v.transform(m);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 0.0);
    assert_f32_eq!(r.z, -1.0);
}

#[test]
fn matrix_rotate_z_wiring() {
    // Rotate (1,0,0) around Z by 90 degrees -> (0,1,0)
    use std::f32::consts::FRAC_PI_2;
    let m = Matrix::rotate_z(FRAC_PI_2);
    let v = Vector3::new(1.0, 0.0, 0.0);
    let r = v.transform(m);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 1.0);
    assert_f32_eq!(r.z, 0.0);
}

#[test]
fn matrix_rotate_xyz_vs_rotate_zyx() {
    use std::f32::consts::FRAC_PI_2;
    // rotate_xyz and rotate_zyx with different angles should produce different results
    let angle = Vector3::new(FRAC_PI_2, 0.1, 0.2);
    let xyz = Matrix::rotate_xyz(angle);
    let zyx = Matrix::rotate_zyx(angle);
    // They should differ (different rotation order)
    assert!(
        (xyz.m0 - zyx.m0).abs() > 1e-4
            || (xyz.m5 - zyx.m5).abs() > 1e-4
            || (xyz.m10 - zyx.m10).abs() > 1e-4,
        "rotate_xyz and rotate_zyx produce identical results (possible mis-wire?)"
    );
}

#[test]
fn matrix_compose_decompose() {
    let translation = Vector3::new(1.0, 2.0, 3.0);
    let rotation = Quaternion::identity();
    let scale = Vector3::new(1.0, 1.0, 1.0);
    let m = Matrix::compose(translation, rotation, scale);
    let (t_out, _r_out, s_out) = matrix_decompose(m);
    assert_v3_eq!(t_out, translation);
    assert_v3_eq!(s_out, scale);
}

#[test]
fn matrix_to_float_array() {
    let id = Matrix::identity();
    let arr = id.to_float_array();
    // Identity: m0=1, m5=1, m10=1, m15=1, rest 0
    assert_f32_eq!(arr.v[0], 1.0); // m0
    assert_f32_eq!(arr.v[5], 1.0); // m5
    assert_f32_eq!(arr.v[10], 1.0); // m10
    assert_f32_eq!(arr.v[15], 1.0); // m15
    assert_f32_eq!(arr.v[1], 0.0); // m1
}

// ─── Quaternion ──────────────────────────────────────────────────────────────

#[test]
fn quaternion_identity_normalizes() {
    let q = Quaternion::identity();
    let n = q.normalize();
    // identity quaternion is already unit length
    assert!((q.length() - 1.0).abs() < 1e-6);
    assert_eq!(n, q);
}

#[test]
fn quaternion_identity_wiring() {
    // Identity quaternion = (0, 0, 0, 1)
    let q = Quaternion::identity();
    assert_f32_eq!(q.x, 0.0);
    assert_f32_eq!(q.y, 0.0);
    assert_f32_eq!(q.z, 0.0);
    assert_f32_eq!(q.w, 1.0);
}

#[test]
fn quaternion_arithmetic_wiring() {
    let a = Quaternion {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
    };
    let b = Quaternion {
        x: 5.0,
        y: 6.0,
        z: 7.0,
        w: 8.0,
    };

    // add (component-wise) - Quaternion has same fields as Vector4
    let sum = a + b;
    assert_f32_eq!(sum.x, 6.0);
    assert_f32_eq!(sum.y, 8.0);
    assert_f32_eq!(sum.z, 10.0);
    assert_f32_eq!(sum.w, 12.0);
    // subtract (NON-commutative)
    let d = b - a;
    assert_f32_eq!(d.x, 4.0);
    assert_f32_eq!(d.y, 4.0);
    assert_f32_eq!(d.z, 4.0);
    assert_f32_eq!(d.w, 4.0);
    let d2 = a - b;
    assert_f32_eq!(d2.x, -4.0);

    // add_value / sub_value
    let av = a.add_value(10.0);
    assert_f32_eq!(av.x, 11.0);
    assert_f32_eq!(av.w, 14.0);
    let sv = a.sub_value(0.5);
    assert_f32_eq!(sv.x, 0.5);
    assert_f32_eq!(sv.w, 3.5);

    // scale (Mul<f32>)
    let scaled = a * 3.0;
    assert_f32_eq!(scaled.x, 3.0);
    assert_f32_eq!(scaled.w, 12.0);

    // divide component-wise (NON-commutative)
    let div = b / a;
    assert_f32_eq!(div.x, 5.0);
    assert_f32_eq!(div.y, 3.0);
    assert_f32_eq!(div.z, 7.0 / 3.0);
    assert_f32_eq!(div.w, 2.0);
}

#[test]
fn quaternion_multiply_non_commutative() {
    // q1 * q2 != q2 * q1 (generally)
    // Use 90-degree rotations around different axes
    use std::f32::consts::FRAC_PI_2;
    let qx = Quaternion::from_axis_angle(Vector3::new(1.0, 0.0, 0.0), FRAC_PI_2);
    let qz = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);

    let qxz = qx * qz;
    let qzx = qz * qx;

    // These should differ
    assert!(
        (qxz.x - qzx.x).abs() > 1e-4
            || (qxz.y - qzx.y).abs() > 1e-4
            || (qxz.z - qzx.z).abs() > 1e-4
            || (qxz.w - qzx.w).abs() > 1e-4,
        "QuaternionMultiply appears commutative (mis-wire?)"
    );

    // Verify qx*qz: rotate Z-first (qz), then X (qx) applied to (1,0,0)
    // qz rotates (1,0,0) -> (0,1,0)
    // qx rotates (0,1,0) -> (0,0,1)
    let v = Vector3::new(1.0, 0.0, 0.0);
    let r = v.rotate_by_quaternion(qxz);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 0.0);
    assert_f32_eq!(r.z, 1.0);
}

#[test]
fn quaternion_from_axis_angle_wiring() {
    use std::f32::consts::FRAC_PI_2;
    // 90-degree rotation around Z axis: q = (0, 0, sin(45°), cos(45°))
    let q = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);
    let sin45 = (FRAC_PI_2 / 2.0).sin();
    let cos45 = (FRAC_PI_2 / 2.0).cos();
    assert_f32_eq!(q.x, 0.0);
    assert_f32_eq!(q.y, 0.0);
    assert_f32_eq!(q.z, sin45);
    assert_f32_eq!(q.w, cos45);
}

#[test]
fn quaternion_to_axis_angle_roundtrip() {
    use std::f32::consts::FRAC_PI_2;
    let original_angle = FRAC_PI_2;
    let original_axis = Vector3::new(0.0, 0.0, 1.0);
    let q = Quaternion::from_axis_angle(original_axis, original_angle);
    let (axis_out, angle_out) = quaternion_to_axis_angle(q);
    assert_f32_eq!(axis_out.z, 1.0);
    assert_f32_eq!(axis_out.x, 0.0);
    assert_f32_eq!(axis_out.y, 0.0);
    assert_f32_eq!(angle_out, original_angle);
}

#[test]
fn quaternion_invert() {
    use std::f32::consts::FRAC_PI_2;
    let q = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);
    let inv = q.invert();
    // q * q_inv = identity
    let prod = q * inv;
    assert_f32_eq!(prod.x, 0.0);
    assert_f32_eq!(prod.y, 0.0);
    assert_f32_eq!(prod.z, 0.0);
    assert_f32_eq!(prod.w, 1.0);
}

#[test]
fn quaternion_normalize() {
    let q = Quaternion {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
    };
    let n = q.normalize();
    // length should be 1
    let len = (n.x * n.x + n.y * n.y + n.z * n.z + n.w * n.w).sqrt();
    assert_f32_eq!(len, 1.0);
    // direction preserved: components proportional to originals
    let orig_len = (1.0_f32 + 4.0 + 9.0 + 16.0).sqrt();
    assert_f32_eq!(n.x, 1.0 / orig_len);
    assert_f32_eq!(n.w, 4.0 / orig_len);
}

#[test]
fn quaternion_lerp_non_commutative() {
    use std::f32::consts::FRAC_PI_2;
    let q1 = Quaternion::identity();
    let q2 = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);
    // lerp(q1, q2, 0.3) should differ from lerp(q2, q1, 0.3)
    let l1 = q1.lerp(q2, 0.3);
    let l2 = q2.lerp(q1, 0.3);
    assert!(
        (l1.x - l2.x).abs() > 1e-4
            || (l1.y - l2.y).abs() > 1e-4
            || (l1.z - l2.z).abs() > 1e-4
            || (l1.w - l2.w).abs() > 1e-4,
        "QuaternionLerp appears commutative (mis-wire?)"
    );
}

#[test]
fn quaternion_nlerp_and_slerp() {
    use std::f32::consts::FRAC_PI_2;
    let q1 = Quaternion::identity();
    let q2 = Quaternion::from_axis_angle(Vector3::new(0.0, 0.0, 1.0), FRAC_PI_2);

    // nlerp at t=0 should be near q1, at t=1 near q2
    let n0 = q1.nlerp(q2, 0.0);
    assert_f32_eq!(n0.w, q1.w);
    let n1 = q1.nlerp(q2, 1.0);
    assert_f32_eq!(n1.z, q2.z);
    assert_f32_eq!(n1.w, q2.w);

    // slerp at t=0 should be near q1, at t=1 near q2
    let s0 = q1.slerp(q2, 0.0);
    assert_f32_eq!(s0.w, q1.w);
    let s1 = q1.slerp(q2, 1.0);
    assert_f32_eq!(s1.z, q2.z);
    assert_f32_eq!(s1.w, q2.w);
}

#[test]
fn quaternion_from_matrix_and_to_matrix() {
    use std::f32::consts::FRAC_PI_2;
    // Build a rotation matrix, convert to quaternion, back to matrix
    let m = Matrix::rotate_z(FRAC_PI_2);
    let q = Quaternion::from_matrix(m);
    let m2 = q.to_matrix();

    // The matrices should be approximately equal
    assert_f32_eq!(m.m0, m2.m0);
    assert_f32_eq!(m.m5, m2.m5);
    assert_f32_eq!(m.m10, m2.m10);
}

#[test]
fn quaternion_from_vector3_to_vector3() {
    // Rotation from (1,0,0) to (0,1,0)
    let from = Vector3::new(1.0, 0.0, 0.0);
    let to = Vector3::new(0.0, 1.0, 0.0);
    let q = Quaternion::from_vector3_to_vector3(from, to);
    // Applying this quaternion to `from` should give ~`to`
    let r = from.rotate_by_quaternion(q);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 1.0);
    assert_f32_eq!(r.z, 0.0);
}

#[test]
fn quaternion_from_euler_and_to_euler() {
    // from_euler(0, 0, pi/2) = rotation around Z by 90 degrees
    use std::f32::consts::FRAC_PI_2;
    let q = Quaternion::from_euler(0.0, 0.0, FRAC_PI_2);
    // Rotating (1,0,0) should give ~(0,1,0)
    let v = Vector3::new(1.0, 0.0, 0.0);
    let r = v.rotate_by_quaternion(q);
    assert_f32_eq!(r.x, 0.0);
    assert_f32_eq!(r.y, 1.0);
    assert_f32_eq!(r.z, 0.0);

    // to_euler roundtrip (note: euler conventions differ)
    let euler = q.to_euler();
    // The specific axis should reflect the Z rotation
    let q2 = Quaternion::from_euler(euler.x, euler.y, euler.z);
    let r2 = v.rotate_by_quaternion(q2);
    assert_f32_eq!(r2.x, 0.0);
    assert_f32_eq!(r2.y, 1.0);
    assert_f32_eq!(r2.z, 0.0);
}

#[test]
fn quaternion_transform() {
    // QuaternionTransform: rotate quaternion by a matrix
    let q = Quaternion::identity();
    let m = Matrix::identity();
    let r = q.transform(m);
    // Identity transform of identity quaternion should remain identity-like
    assert_f32_eq!(r.w, 1.0);
}

#[test]
fn quaternion_equals() {
    let a = Quaternion {
        x: 1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    assert!(a.equals(a));
    let b = Quaternion {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };
    assert!(!a.equals(b));
}
