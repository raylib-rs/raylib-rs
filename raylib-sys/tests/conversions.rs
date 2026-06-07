//! Round-trip conversion tests between raylib-sys math types and mint/glam interop types.
//! Also includes optional serde JSON round-trip tests (feature = "serde").

#[cfg(feature = "mint")]
mod mint_tests {
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
}

#[cfg(feature = "glam")]
mod glam_tests {
    use raylib_sys::{Matrix, Quaternion, Vector2, Vector3, Vector4};

    #[test]
    fn glam_vector2_roundtrip() {
        let v = Vector2 { x: 3.0, y: 4.0 };
        let g: glam::Vec2 = v.into();
        assert_eq!(g.x, 3.0);
        assert_eq!(g.y, 4.0);
        let back = Vector2::from(g);
        assert_eq!(back.x, v.x);
        assert_eq!(back.y, v.y);
    }

    #[test]
    fn glam_vector3_roundtrip() {
        let v = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let g: glam::Vec3 = v.into();
        assert_eq!(g.x, 1.0);
        assert_eq!(g.y, 2.0);
        assert_eq!(g.z, 3.0);
        let back = Vector3::from(g);
        assert_eq!(back.x, v.x);
        assert_eq!(back.y, v.y);
        assert_eq!(back.z, v.z);
    }

    #[test]
    fn glam_vector4_roundtrip() {
        let v = Vector4 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            w: 4.0,
        };
        let g: glam::Vec4 = v.into();
        assert_eq!(g.x, 1.0);
        assert_eq!(g.y, 2.0);
        assert_eq!(g.z, 3.0);
        assert_eq!(g.w, 4.0);
        let back = Vector4::from(g);
        assert_eq!(back.x, v.x);
        assert_eq!(back.y, v.y);
        assert_eq!(back.z, v.z);
        assert_eq!(back.w, v.w);
    }

    #[test]
    fn glam_quaternion_roundtrip() {
        // Both glam::Quat and raylib Quaternion are { x, y, z, w }.
        let q = Quaternion {
            x: 0.1,
            y: 0.2,
            z: 0.3,
            w: 0.9,
        };
        let g: glam::Quat = q.into();
        assert_eq!(g.x, q.x);
        assert_eq!(g.y, q.y);
        assert_eq!(g.z, q.z);
        assert_eq!(g.w, q.w);
        let back = Quaternion::from(g);
        assert_eq!(back.x, q.x);
        assert_eq!(back.y, q.y);
        assert_eq!(back.z, q.z);
        assert_eq!(back.w, q.w);
    }

    #[test]
    fn glam_matrix_roundtrip() {
        // All 16 components must survive the round-trip.
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
        let g: glam::Mat4 = mat.into();
        let back = Matrix::from(g);
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

    #[test]
    fn glam_matrix_translation_matches_semantically() {
        // raylib MatrixTranslate(1,2,3) must produce the same transform as
        // glam's from_translation(vec3(1,2,3)).
        //
        // For a translation matrix:
        //   col3 = (tx, ty, tz, 1.0), all other columns are identity columns.
        // In raylib field terms: m12=tx, m13=ty, m14=tz (col3 elements).
        let r: Matrix = Matrix::translate(1.0, 2.0, 3.0);
        let g: glam::Mat4 = r.into();
        let expected = glam::Mat4::from_translation(glam::vec3(1.0, 2.0, 3.0));
        assert!(
            g.abs_diff_eq(expected, 1e-5),
            "got {g:?} expected {expected:?}"
        );
    }
}

#[cfg(feature = "serde")]
mod serde_tests {
    use raylib_sys::{Color, Matrix, Quaternion, Rectangle, Vector2, Vector3, Vector4};

    #[test]
    fn serde_vector2_roundtrip() {
        let v = Vector2::new(1.0, 2.0);
        let j = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Vector2>(&j).unwrap(), v);
    }

    #[test]
    fn serde_vector3_roundtrip() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let j = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Vector3>(&j).unwrap(), v);
    }

    #[test]
    fn serde_vector4_roundtrip() {
        let v = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let j = serde_json::to_string(&v).unwrap();
        assert_eq!(serde_json::from_str::<Vector4>(&j).unwrap(), v);
    }

    #[test]
    fn serde_matrix_roundtrip() {
        let mat = Matrix {
            m0: 1.0,
            m1: 0.0,
            m2: 0.0,
            m3: 0.0,
            m4: 0.0,
            m5: 1.0,
            m6: 0.0,
            m7: 0.0,
            m8: 0.0,
            m9: 0.0,
            m10: 1.0,
            m11: 0.0,
            m12: 4.0,
            m13: 5.0,
            m14: 6.0,
            m15: 1.0,
        };
        let j = serde_json::to_string(&mat).unwrap();
        assert_eq!(serde_json::from_str::<Matrix>(&j).unwrap(), mat);
    }

    #[test]
    fn serde_quaternion_roundtrip() {
        let q = Quaternion::new(0.1, 0.2, 0.3, 0.9);
        let j = serde_json::to_string(&q).unwrap();
        assert_eq!(serde_json::from_str::<Quaternion>(&j).unwrap(), q);
    }

    #[test]
    fn serde_rectangle_roundtrip() {
        let r = Rectangle::new(10.0, 20.0, 100.0, 50.0);
        let j = serde_json::to_string(&r).unwrap();
        assert_eq!(serde_json::from_str::<Rectangle>(&j).unwrap(), r);
    }

    #[test]
    fn serde_color_roundtrip() {
        let c = Color::new(255, 128, 0, 255);
        let j = serde_json::to_string(&c).unwrap();
        assert_eq!(serde_json::from_str::<Color>(&j).unwrap(), c);
    }
}

/// Tuple/array `From` conversions for the vector types (no features needed).
mod tuple_conversions {
    use raylib_sys::{Vector2, Vector3, Vector4};

    #[test]
    fn vector2_from_tuple_and_array() {
        assert_eq!(Vector2::from((1.0, 2.0)), Vector2 { x: 1.0, y: 2.0 });
        assert_eq!(Vector2::from([1.0, 2.0]), Vector2 { x: 1.0, y: 2.0 });
    }

    #[test]
    fn vector3_from_tuple_and_array() {
        let expected = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        assert_eq!(Vector3::from((1.0, 2.0, 3.0)), expected);
        assert_eq!(Vector3::from([1.0, 2.0, 3.0]), expected);
    }

    #[test]
    fn vector4_from_tuple_and_array() {
        let expected = Vector4 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            w: 4.0,
        };
        assert_eq!(Vector4::from((1.0, 2.0, 3.0, 4.0)), expected);
        assert_eq!(Vector4::from([1.0, 2.0, 3.0, 4.0]), expected);
    }

    #[test]
    fn into_inference_at_call_sites() {
        // The point of the impls: `impl Into<Vector2>` params accept tuples/arrays.
        fn takes(v: impl Into<Vector2>) -> Vector2 {
            v.into()
        }
        assert_eq!(takes((5.0, 6.0)), Vector2 { x: 5.0, y: 6.0 });
        assert_eq!(takes([5.0, 6.0]), Vector2 { x: 5.0, y: 6.0 });
    }
}
