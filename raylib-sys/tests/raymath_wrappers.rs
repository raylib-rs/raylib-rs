#![allow(clippy::approx_constant)]
use raylib_sys::{Matrix, Quaternion, Vector2, Vector3, Vector4};

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
fn vector2_core_ops() {
    assert_eq!(Vector2::new(3.0, 4.0).length(), 5.0);
    let sum = Vector2::new(1.0, 0.0) + Vector2::new(0.0, 1.0);
    assert_eq!(sum, Vector2::new(1.0, 1.0));
    let scaled = Vector2::new(2.0, 3.0) * 2.0;
    assert_eq!(scaled, Vector2::new(4.0, 6.0));
    assert_eq!(-Vector2::new(1.0, -1.0), Vector2::new(-1.0, 1.0));
}

#[test]
fn vector4_core_ops() {
    let a = Vector4::new(1.0, 0.0, 0.0, 0.0);
    assert!((a.length() - 1.0).abs() < 1e-6);
    let sum = Vector4::new(1.0, 2.0, 3.0, 4.0) + Vector4::new(1.0, 1.0, 1.0, 1.0);
    assert_eq!(sum, Vector4::new(2.0, 3.0, 4.0, 5.0));
}

#[test]
fn matrix_identity_is_neutral() {
    let id = Matrix::identity();
    let m = Matrix::translate(1.0, 2.0, 3.0);
    assert_eq!(id * m, m);
    assert_eq!(id.determinant(), 1.0);
}

#[test]
fn quaternion_identity_normalizes() {
    let q = Quaternion::identity();
    let n = q.normalize();
    // identity quaternion is already unit length
    assert!((q.length() - 1.0).abs() < 1e-6);
    assert_eq!(n, q);
}
