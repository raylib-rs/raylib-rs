//! Optional conversions between raylib-sys math types and [`glam`] types.
//!
//! Enabled with `features = ["glam"]`. All conversions are zero-cost field-wise assignments
//! between `#[repr(C)]` structs with compatible layout semantics.
//!
//! # Matrix column-major mapping
//!
//! Raylib's [`Matrix`] stores elements such that **semantically** (per the raymath conventions):
//!   - col0 = (m0,  m1,  m2,  m3)
//!   - col1 = (m4,  m5,  m6,  m7)
//!   - col2 = (m8,  m9,  m10, m11)
//!   - col3 = (m12, m13, m14, m15)
//!
//! `glam::Mat4::from_cols_array` / `to_cols_array` use the same column-major order, so the
//! mapping is direct: `a[0..4]` ↔ col0, `a[4..8]` ↔ col1, `a[8..12]` ↔ col2, `a[12..16]` ↔ col3.
//!
//! This mapping is verified by the semantic translation test in `tests/conversions.rs`: a
//! [`Matrix::translate(x, y, z)`] must produce the same transform as
//! [`glam::Mat4::from_translation`].
//!
//! # Quaternion mapping
//!
//! Both `glam::Quat` and raylib's [`Quaternion`] use flat `{ x, y, z, w }` fields (x,y,z
//! = vector part, w = scalar part).

use crate::{Matrix, Quaternion, Vector2, Vector3, Vector4};

// ── Vector2 ──────────────────────────────────────────────────────────────────

impl From<glam::Vec2> for Vector2 {
    #[inline]
    fn from(v: glam::Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vector2> for glam::Vec2 {
    #[inline]
    fn from(v: Vector2) -> Self {
        glam::Vec2::new(v.x, v.y)
    }
}

// ── Vector3 ──────────────────────────────────────────────────────────────────

impl From<glam::Vec3> for Vector3 {
    #[inline]
    fn from(v: glam::Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vector3> for glam::Vec3 {
    #[inline]
    fn from(v: Vector3) -> Self {
        glam::Vec3::new(v.x, v.y, v.z)
    }
}

// ── Vector4 ──────────────────────────────────────────────────────────────────

impl From<glam::Vec4> for Vector4 {
    #[inline]
    fn from(v: glam::Vec4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

impl From<Vector4> for glam::Vec4 {
    #[inline]
    fn from(v: Vector4) -> Self {
        glam::Vec4::new(v.x, v.y, v.z, v.w)
    }
}

// ── Quaternion ───────────────────────────────────────────────────────────────
//
// Both glam::Quat and raylib Quaternion use { x, y, z, w } (vector part first, scalar last).

impl From<glam::Quat> for Quaternion {
    #[inline]
    fn from(q: glam::Quat) -> Self {
        Self {
            x: q.x,
            y: q.y,
            z: q.z,
            w: q.w,
        }
    }
}

impl From<Quaternion> for glam::Quat {
    #[inline]
    fn from(q: Quaternion) -> Self {
        glam::Quat::from_xyzw(q.x, q.y, q.z, q.w)
    }
}

// ── Matrix ↔ glam::Mat4 ──────────────────────────────────────────────────────
//
// Raylib Matrix semantic column-major layout:
//   col0 = (m0,  m1,  m2,  m3)
//   col1 = (m4,  m5,  m6,  m7)
//   col2 = (m8,  m9,  m10, m11)
//   col3 = (m12, m13, m14, m15)
//
// glam::Mat4::from_cols_array takes [f32; 16] in column-major order:
//   indices 0..3  = col0, 4..7 = col1, 8..11 = col2, 12..15 = col3.
//
// The mapping is therefore 1:1; no transpose needed.

impl From<Matrix> for glam::Mat4 {
    #[inline]
    fn from(m: Matrix) -> Self {
        glam::Mat4::from_cols_array(&[
            m.m0, m.m1, m.m2, m.m3, m.m4, m.m5, m.m6, m.m7, m.m8, m.m9, m.m10, m.m11, m.m12, m.m13,
            m.m14, m.m15,
        ])
    }
}

impl From<glam::Mat4> for Matrix {
    #[inline]
    fn from(g: glam::Mat4) -> Self {
        let a = g.to_cols_array();
        Matrix {
            m0: a[0],
            m1: a[1],
            m2: a[2],
            m3: a[3],
            m4: a[4],
            m5: a[5],
            m6: a[6],
            m7: a[7],
            m8: a[8],
            m9: a[9],
            m10: a[10],
            m11: a[11],
            m12: a[12],
            m13: a[13],
            m14: a[14],
            m15: a[15],
        }
    }
}
