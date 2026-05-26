//! Round-trip conversion tests between raylib-sys math types and mint interop types.
//! All tests are gated on the `mint` feature.
#![cfg(feature = "mint")]

use raylib_sys::{Matrix, Quaternion, Vector2, Vector3, Vector4};

#[test]
fn mint_vector2_roundtrip() {
    let v = Vector2 { x: 1.0, y: 2.0 };
    let m: mint::Vector2<f32> = v.into();
    assert_eq!(m.x, 1.0);
    assert_eq!(m.y, 2.0);
    let back = Vector2::from(m);
    assert_eq!(back, v);
}

#[test]
fn mint_vector3_roundtrip() {
    let v = Vector3 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    };
    let m: mint::Vector3<f32> = v.into();
    assert_eq!(m.x, 1.0);
    assert_eq!(m.y, 2.0);
    assert_eq!(m.z, 3.0);
    let back = Vector3::from(m);
    assert_eq!(back, v);
}

#[test]
fn mint_vector4_roundtrip() {
    let v = Vector4 {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
    };
    let m: mint::Vector4<f32> = v.into();
    assert_eq!(m.x, 1.0);
    assert_eq!(m.y, 2.0);
    assert_eq!(m.z, 3.0);
    assert_eq!(m.w, 4.0);
    let back = Vector4::from(m);
    assert_eq!(back, v);
}

#[test]
fn mint_quaternion_roundtrip() {
    // All four components must survive the round-trip.
    let q = Quaternion {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        w: 4.0,
    };
    let m: mint::Quaternion<f32> = q.into();
    // mint::Quaternion is { v: Vector3 { x, y, z }, s: w }
    assert_eq!(m.v.x, 1.0);
    assert_eq!(m.v.y, 2.0);
    assert_eq!(m.v.z, 3.0);
    assert_eq!(m.s, 4.0);
    let back = Quaternion::from(m);
    assert_eq!(back.x, q.x);
    assert_eq!(back.y, q.y);
    assert_eq!(back.z, q.z);
    assert_eq!(back.w, q.w);
}

#[test]
fn mint_matrix_roundtrip() {
    // All 16 components must survive the round-trip.
    // Raylib Matrix is column-major: m0..m3 = col0, m4..m7 = col1, m8..m11 = col2, m12..m15 = col3.
    let mat = Matrix {
        m0: 0.0,
        m1: 1.0,
        m2: 2.0,
        m3: 3.0,
        m4: 4.0,
        m5: 5.0,
        m6: 6.0,
        m7: 7.0,
        m8: 8.0,
        m9: 9.0,
        m10: 10.0,
        m11: 11.0,
        m12: 12.0,
        m13: 13.0,
        m14: 14.0,
        m15: 15.0,
    };
    let m: mint::ColumnMatrix4<f32> = mat.into();
    // Column 0 (x)
    assert_eq!(m.x.x, 0.0);
    assert_eq!(m.x.y, 1.0);
    assert_eq!(m.x.z, 2.0);
    assert_eq!(m.x.w, 3.0);
    // Column 1 (y)
    assert_eq!(m.y.x, 4.0);
    assert_eq!(m.y.y, 5.0);
    assert_eq!(m.y.z, 6.0);
    assert_eq!(m.y.w, 7.0);
    // Column 2 (z)
    assert_eq!(m.z.x, 8.0);
    assert_eq!(m.z.y, 9.0);
    assert_eq!(m.z.z, 10.0);
    assert_eq!(m.z.w, 11.0);
    // Column 3 (w)
    assert_eq!(m.w.x, 12.0);
    assert_eq!(m.w.y, 13.0);
    assert_eq!(m.w.z, 14.0);
    assert_eq!(m.w.w, 15.0);
    // Back to Matrix
    let back = Matrix::from(m);
    assert_eq!(back.m0, mat.m0);
    assert_eq!(back.m1, mat.m1);
    assert_eq!(back.m2, mat.m2);
    assert_eq!(back.m3, mat.m3);
    assert_eq!(back.m4, mat.m4);
    assert_eq!(back.m5, mat.m5);
    assert_eq!(back.m6, mat.m6);
    assert_eq!(back.m7, mat.m7);
    assert_eq!(back.m8, mat.m8);
    assert_eq!(back.m9, mat.m9);
    assert_eq!(back.m10, mat.m10);
    assert_eq!(back.m11, mat.m11);
    assert_eq!(back.m12, mat.m12);
    assert_eq!(back.m13, mat.m13);
    assert_eq!(back.m14, mat.m14);
    assert_eq!(back.m15, mat.m15);
}
