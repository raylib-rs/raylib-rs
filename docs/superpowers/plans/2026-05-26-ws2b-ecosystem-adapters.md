# WS2b — Optional Ecosystem Adapters (mint / glam / serde) + Wrapper Tests

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Re-introduce `mint`, `glam`, and `serde` as **optional, off-by-default** features on `raylib-sys`'s native math types via `From`/`Into` conversions and (for serde) feature-gated derives — completing the "nothing but raylib by default, ecosystems opt-in" story (spec D2) — and harden the raymath wrappers with mis-wiring tests.

**Architecture:** `raylib-sys` owns `Vector2/3/4`, `Matrix` (bindgen-generated), `Quaternion` (distinct struct). WS2b adds feature-gated adapter modules and serde derive injection — **sys-only**; the safe crate is still red and is WS3's job. Each feature is additive and independently composable; the default build stays math-dep-free (verified).

**Tech Stack:** Rust 1.85, `mint 0.5`, `glam 0.30`, `serde 1`, bindgen `add_derives`. Branch `6.0-rc`; push to `fork` for the 3-OS CI.

**Reference:** spec D2; WS2a plan + result (commits `006eb49`..`97e23d1`); `raylib-sys/src/{math.rs,vector_math.rs,matrix_quat_math.rs}`; `docs/superpowers/notes/ws1-breakage-baseline.md`. The current `raylib-sys/Cargo.toml` has only `serde` (optional) — `mint`/`glam` are gone (WS2a).

**Pre-flight:** on branch `6.0-rc`; `cargo build -p raylib-sys` green; `cargo test -p raylib-sys` green; `cargo tree -p raylib-sys -e no-dev` shows no mint/glam.

**Scope boundary (every task):** modify only `raylib-sys/**`. Do NOT touch `raylib/src/**` (safe crate = WS3). The default build must remain math-dep-free; existing tests stay green.

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `raylib-sys/Cargo.toml` | add optional `mint`/`glam` deps; `mint`/`glam` features; serde feature wiring | 1,2,3 |
| `raylib-sys/src/mint_conv.rs` | `#[cfg(feature="mint")]` From/Into native ↔ `mint::*` (+ tests) | 1 |
| `raylib-sys/src/glam_conv.rs` | `#[cfg(feature="glam")]` From/Into native ↔ `glam::*` (+ tests) | 2 |
| `raylib-sys/build.rs` | inject serde derives on bindgen-generated Vector*/Matrix when `CARGO_FEATURE_SERDE` | 3 |
| `raylib-sys/src/math.rs` | ensure serde `cfg_attr` on hand-defined Rectangle/Color/Quaternion | 3 |
| `raylib-sys/tests/raymath_wrappers.rs` | expand mis-wiring tests across all wrapper groups | 4 |
| `raylib-sys/tests/conversions.rs` | round-trip tests behind each feature (create) | 1,2,3 |

---

## Task 1: `mint` optional adapter

**Files:** `raylib-sys/Cargo.toml`, create `raylib-sys/src/mint_conv.rs` + `mod` in `lib.rs`, `raylib-sys/tests/conversions.rs`.

- [ ] **Step 1: Cargo.toml.** Add to `[dependencies]`: `mint = { version = "0.5", optional = true }`. Add to `[features]`: `mint = ["dep:mint"]`.
- [ ] **Step 2: Adapters** `raylib-sys/src/mint_conv.rs` (gate the whole module `#![cfg(feature = "mint")]` or `#[cfg(feature="mint")] mod mint_conv;` in lib.rs). `mint`'s types are plain `#[repr(C)]`, so conversions are field-wise:
```rust
use crate::{Vector2, Vector3, Vector4, Quaternion, Matrix};

impl From<mint::Vector2<f32>> for Vector2 { fn from(v: mint::Vector2<f32>) -> Self { Self { x: v.x, y: v.y } } }
impl From<Vector2> for mint::Vector2<f32> { fn from(v: Vector2) -> Self { mint::Vector2 { x: v.x, y: v.y } } }
// ... Vector3 <-> mint::Vector3<f32>, Vector4 <-> mint::Vector4<f32>, Quaternion <-> mint::Quaternion<f32>
// Matrix <-> mint::ColumnMatrix4<f32> (raylib Matrix is column-major; map fields to mint's column-major array carefully).
```
Cover Vector2/3/4 + Quaternion (trivial) + Matrix (mind column-major mapping). Add `impl From<(f32,f32)> for Vector2` etc. only if cheap — keep focused on mint here.
- [ ] **Step 3: Round-trip tests** in `raylib-sys/tests/conversions.rs` under `#[cfg(feature = "mint")]`:
```rust
#[test] fn mint_vector_roundtrip() {
    let v = raylib_sys::Vector3::new(1.0, 2.0, 3.0);
    let m: mint::Vector3<f32> = v.into();
    assert_eq!(raylib_sys::Vector3::from(m), v);
}
```
Add one per type. For Matrix, assert a round-trip preserves all 16 elements.
- [ ] **Step 4: Verify.** `cargo build -p raylib-sys --features mint`; `cargo test -p raylib-sys --features mint`; `cargo tree -p raylib-sys -e no-dev` (default, no `--features`) → still NO mint. `cargo fmt --all`.
- [ ] **Step 5: Commit** (`feat(ws2b): optional mint conversions`, Claude trailer).

---

## Task 2: `glam` optional adapter

**Files:** `raylib-sys/Cargo.toml`, create `raylib-sys/src/glam_conv.rs` + `mod`, extend `conversions.rs`.

- [ ] **Step 1: Cargo.toml.** `glam = { version = "0.30", optional = true }`; feature `glam = ["dep:glam"]`.
- [ ] **Step 2: Adapters** `raylib-sys/src/glam_conv.rs` (`#[cfg(feature="glam")]`):
```rust
use crate::{Vector2, Vector3, Vector4, Quaternion, Matrix};
impl From<glam::Vec2> for Vector2 { fn from(v: glam::Vec2) -> Self { Self { x: v.x, y: v.y } } }
impl From<Vector2> for glam::Vec2 { fn from(v: Vector2) -> Self { glam::vec2(v.x, v.y) } }
// Vector3<->Vec3, Vector4<->Vec4, Quaternion<->glam::Quat (both x,y,z,w).
```
- [ ] **Step 3: Matrix conversion — handle layout carefully.** raylib `Matrix` is **column-major** with fields `m0,m4,m8,m12` = first row but stored column-major; `glam::Mat4` is column-major via `from_cols_array(&[16])`. Map correctly:
```rust
impl From<Matrix> for glam::Mat4 {
    fn from(m: Matrix) -> Self {
        // raylib stores column-major: column0 = (m0,m1,m2,m3), column1 = (m4,m5,m6,m7), ...
        glam::Mat4::from_cols_array(&[
            m.m0, m.m1, m.m2, m.m3,
            m.m4, m.m5, m.m6, m.m7,
            m.m8, m.m9, m.m10, m.m11,
            m.m12, m.m13, m.m14, m.m15,
        ])
    }
}
// and the inverse From<glam::Mat4> for Matrix using to_cols_array().
```
**Verify the column order against `raylib-sys/raylib/src/raymath.h`** (look at how `MatrixToFloatV`/`MatrixIdentity` lay out the fields) before trusting the mapping — this is the one error-prone conversion.
- [ ] **Step 4: Tests** under `#[cfg(feature="glam")]` — vector/quat round-trips, PLUS a **semantic** matrix test (not just round-trip): build a translation `Matrix::translate(1,2,3)`, convert to `glam::Mat4`, and assert it transforms a point the same way raylib does (or that the converted Mat4 equals `glam::Mat4::from_translation(vec3(1,2,3))`). This catches a wrong column mapping that a round-trip alone would hide.
- [ ] **Step 5: Verify** (`--features glam`, default still no glam, fmt) and **commit**.

---

## Task 3: serde across all math types

**Files:** `raylib-sys/build.rs`, `raylib-sys/src/math.rs`, `raylib-sys/Cargo.toml`, extend `conversions.rs`.

bindgen-generated `Vector*`/`Matrix` can't carry a `cfg_attr` derive, so inject serde derives at generation time when the feature is on.

- [ ] **Step 1: Conditional derive injection in `build.rs`.** In `gen_bindings()`, when `env::var("CARGO_FEATURE_SERDE").is_ok()`, add a `ParseCallbacks`/`add_derives` that returns `["serde::Serialize", "serde::Deserialize"]` **only for the math type names** (`Vector2/3/4`, `Matrix` — match on the type name in the callback). Example:
```rust
#[derive(Debug)]
struct SerdeOnMath;
impl bindgen::callbacks::ParseCallbacks for SerdeOnMath {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo) -> Vec<String> {
        match info.name {
            "Vector2" | "Vector3" | "Vector4" | "Matrix" =>
                vec!["serde::Serialize".into(), "serde::Deserialize".into()],
            _ => vec![],
        }
    }
}
// ...later: if std::env::var("CARGO_FEATURE_SERDE").is_ok() { builder = builder.parse_callbacks(Box::new(SerdeOnMath)); }
```
- [ ] **Step 2: Hand-defined types.** Confirm `Quaternion` already has `#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]` (WS2a added it); add the same to `Rectangle` and `Color` if missing (check `math.rs`/`color.rs`). The `serde` feature already pulls `dep:serde`.
- [ ] **Step 3: Tests** under `#[cfg(feature="serde")]` in `conversions.rs` — JSON round-trip each math type:
```rust
#[test] fn serde_vector3_roundtrip() {
    let v = raylib_sys::Vector3::new(1.0, 2.0, 3.0);
    let j = serde_json::to_string(&v).unwrap();
    assert_eq!(serde_json::from_str::<raylib_sys::Vector3>(&j).unwrap(), v);
}
```
(Add `serde_json` as a dev-dependency if not present.) Cover Vector2/3/4, Matrix, Quaternion, Rectangle, Color.
- [ ] **Step 4: Verify.** `cargo build -p raylib-sys --features serde`; `cargo test -p raylib-sys --features serde`; default build still serde-free; fmt. **Commit.**

---

## Task 4: Harden the raymath wrappers with mis-wiring tests

**Files:** extend `raylib-sys/tests/raymath_wrappers.rs`.

WS2a wrapped ~129 raymath fns with only 5 tests. Since each wrapper delegates to raylib's tested C, the real risk is **mis-wiring** (wrong fn or wrong argument order). Add focused tests that would fail if a method called the wrong raymath fn.

- [ ] **Step 1: Cover every wrapper group with discriminating known-value tests.** For each type, pick inputs whose results differ per operation so a mis-wire is caught:
```rust
#[test] fn vector3_ops_are_wired_correctly() {
    let a = raylib_sys::Vector3::new(1.0, 2.0, 3.0);
    let b = raylib_sys::Vector3::new(4.0, 5.0, 6.0);
    assert_eq!(a + b, raylib_sys::Vector3::new(5.0, 7.0, 9.0));      // Add
    assert_eq!(b - a, raylib_sys::Vector3::new(3.0, 3.0, 3.0));      // Subtract (order matters!)
    assert_eq!(a.dot(b), 32.0);                                       // 4+10+18
    assert_eq!(a.cross(b), raylib_sys::Vector3::new(-3.0, 6.0, -3.0));// cross (order matters!)
    assert!((a.distance(b) - 27.0_f32.sqrt()).abs() < 1e-5);
    // ... one assertion per remaining Vector3 method, asymmetric inputs to catch arg-swaps
}
```
Do the same for Vector2, Vector4, Matrix (e.g. non-commutative `MatrixMultiply` order; `MatrixTranslate` puts values in the right cells; `MatrixDeterminant` of a known matrix), and Quaternion (multiply order, `from_axis_angle` known rotation). **Aim to touch every public method/operator at least once** — use the method list from `vector_math.rs`/`matrix_quat_math.rs`.
- [ ] **Step 2: Subtraction/order traps explicitly.** Include asymmetric operands for every non-commutative op (`Subtract`, `cross`, `MatrixMultiply`, `QuaternionMultiply`, `distance` is symmetric so pair it with a directional op) so an `a,b`↔`b,a` swap fails.
- [ ] **Step 3: Run.** `cargo test -p raylib-sys --test raymath_wrappers` → all PASS. If any fails, you've found a real mis-wire in WS2a — fix the wrapper in `vector_math.rs`/`matrix_quat_math.rs` and note it. `cargo fmt --all`.
- [ ] **Step 4: Commit** (`test(ws2b): mis-wiring coverage for raymath wrappers`, Claude trailer).

---

## Task 5: Feature-matrix verification + push to fork

**Files:** none (verification); maybe `raylib-sys/Cargo.toml` tidy.

- [ ] **Step 1: Verify every feature combo builds + tests.**
```bash
cargo test -p raylib-sys                                  # default (no math deps)
cargo test -p raylib-sys --features mint
cargo test -p raylib-sys --features glam
cargo test -p raylib-sys --features serde
cargo test -p raylib-sys --features "mint,glam,serde"     # all together
```
All green. Then confirm default purity: `cargo tree -p raylib-sys -e no-dev | grep -iE "mint|glam|serde" || echo CLEAN_DEFAULT`.
- [ ] **Step 2: fmt + clippy.** `cargo fmt --all --check`; `cargo clippy -p raylib-sys --all-features -- -D warnings` (fix any lint in the new modules).
- [ ] **Step 3: Push + watch CI.** `git push fork 6.0-rc`; watch the 3-OS `baseline` run → green. (CI builds default features; the feature combos are verified locally here — a feature-combo CI matrix is WS6.)

---

## Done criteria (WS2b)

- [ ] `mint` + `glam` are optional deps with `From`/`Into` adapters for Vector2/3/4 + Matrix + Quaternion; round-trip (and the glam-Matrix *semantic*) tests pass.
- [ ] serde derives present on all math types when `--features serde`; JSON round-trip tests pass; absent by default.
- [ ] Default build still pulls **zero** math/serde deps.
- [ ] raymath wrappers covered by discriminating mis-wiring tests (every public method touched; non-commutative ops use asymmetric operands); all green.
- [ ] All feature combos build + test; fmt + clippy clean; fork 3-OS CI green. Safe crate untouched.

## Self-review (against spec)

- **D2 completion:** mint/glam/serde now opt-in adapters on the native types; default stays dep-free → "nothing but raylib by default, ecosystems opt-in" is fully realized at the sys layer. The safe crate's public-type adoption + retiring its hard glam dep is WS3.
- **Risk flagged:** the glam `Matrix` column-major mapping is the one error-prone conversion — covered by a semantic (not just round-trip) test.
- **Testing (D10 + owner request):** Task 4 closes the WS2a wrapper-test gap with mis-wiring-focused coverage.
- **Scope:** sys-only; `MintVec*` deprecation aliases live in the safe crate and are handled in WS3.
