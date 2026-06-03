//! Optional conversions between raylib-sys math types and [`mint`] interoperability types.
//!
//! Enabled with `features = ["mint"]`. All conversions are zero-cost field-wise assignments
//! between `#[repr(C)]` structs with identical layout semantics.
//!
//! # Matrix column-major mapping
//!
//! Raylib's [`Matrix`] stores elements in column-major order: fields `m0..m3` form column 0,
//! `m4..m7` column 1, `m8..m11` column 2, and `m12..m15` column 3.
//! [`mint::ColumnMatrix4`] uses four [`mint::Vector4`] columns named `x/y/z/w`, so the
//! mapping is: `x ↔ (m0,m1,m2,m3)`, `y ↔ (m4,m5,m6,m7)`, `z ↔ (m8,m9,m10,m11)`,
//! `w ↔ (m12,m13,m14,m15)`.
//!
//! # Quaternion mapping
//!
//! [`mint::Quaternion`] uses `{ v: Vector3<T>, s: T }` where `v` is the vector part (x,y,z)
//! and `s` is the scalar part (w). Raylib's [`Quaternion`] uses flat `{ x, y, z, w }` fields.

use crate::{Matrix, Quaternion, Vector2, Vector3, Vector4};

// ── Vector2 ──────────────────────────────────────────────────────────────────

impl From<mint::Vector2<f32>> for Vector2 {
    #[inline]
    fn from(v: mint::Vector2<f32>) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vector2> for mint::Vector2<f32> {
    #[inline]
    fn from(v: Vector2) -> Self {
        mint::Vector2 { x: v.x, y: v.y }
    }
}

// ── Vector3 ──────────────────────────────────────────────────────────────────

impl From<mint::Vector3<f32>> for Vector3 {
    #[inline]
    fn from(v: mint::Vector3<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vector3> for mint::Vector3<f32> {
    #[inline]
    fn from(v: Vector3) -> Self {
        mint::Vector3 {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

// ── Vector4 ──────────────────────────────────────────────────────────────────

impl From<mint::Vector4<f32>> for Vector4 {
    #[inline]
    fn from(v: mint::Vector4<f32>) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

impl From<Vector4> for mint::Vector4<f32> {
    #[inline]
    fn from(v: Vector4) -> Self {
        mint::Vector4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

// ── Quaternion ───────────────────────────────────────────────────────────────
//
// mint::Quaternion<f32> layout: { v: Vector3<f32> { x, y, z }, s: f32 }
// raylib Quaternion layout:     { x, y, z, w }
// Mapping: v.x ↔ x, v.y ↔ y, v.z ↔ z, s ↔ w

impl From<mint::Quaternion<f32>> for Quaternion {
    #[inline]
    fn from(q: mint::Quaternion<f32>) -> Self {
        Self {
            x: q.v.x,
            y: q.v.y,
            z: q.v.z,
            w: q.s,
        }
    }
}

impl From<Quaternion> for mint::Quaternion<f32> {
    #[inline]
    fn from(q: Quaternion) -> Self {
        mint::Quaternion {
            v: mint::Vector3 {
                x: q.x,
                y: q.y,
                z: q.z,
            },
            s: q.w,
        }
    }
}

// ── Matrix ↔ ColumnMatrix4 ───────────────────────────────────────────────────
//
// Raylib Matrix is column-major; fields m0..m15 map to columns as follows:
//   column x (0): m0, m1, m2, m3
//   column y (1): m4, m5, m6, m7
//   column z (2): m8, m9, m10, m11
//   column w (3): m12, m13, m14, m15
//
// mint::ColumnMatrix4<f32> has four Vector4 columns named x, y, z, w.

impl From<mint::ColumnMatrix4<f32>> for Matrix {
    #[inline]
    fn from(m: mint::ColumnMatrix4<f32>) -> Self {
        Self {
            m0: m.x.x,
            m1: m.x.y,
            m2: m.x.z,
            m3: m.x.w,
            m4: m.y.x,
            m5: m.y.y,
            m6: m.y.z,
            m7: m.y.w,
            m8: m.z.x,
            m9: m.z.y,
            m10: m.z.z,
            m11: m.z.w,
            m12: m.w.x,
            m13: m.w.y,
            m14: m.w.z,
            m15: m.w.w,
        }
    }
}

impl From<Matrix> for mint::ColumnMatrix4<f32> {
    #[inline]
    fn from(m: Matrix) -> Self {
        mint::ColumnMatrix4 {
            x: mint::Vector4 {
                x: m.m0,
                y: m.m1,
                z: m.m2,
                w: m.m3,
            },
            y: mint::Vector4 {
                x: m.m4,
                y: m.m5,
                z: m.m6,
                w: m.m7,
            },
            z: mint::Vector4 {
                x: m.m8,
                y: m.m9,
                z: m.m10,
                w: m.m11,
            },
            w: mint::Vector4 {
                x: m.m12,
                y: m.m13,
                z: m.m14,
                w: m.m15,
            },
        }
    }
}
