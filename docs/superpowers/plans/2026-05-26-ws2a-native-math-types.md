# WS2a — Native Math Types in `raylib-sys` (via raymath C-shim)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Drop the `mint` type aliases in `raylib-sys` — let **bindgen generate** the POD `Vector2/3/4` and `Matrix` straight from the C headers, keep `Quaternion` as a distinct transparent `#[repr(C)]` newtype (C aliases it to `Vector4`, so bindgen can't give it its own method namespace), and give them all their math by **calling raylib's own raymath functions** through a C-shim — so the default build owns its math types with zero math-crate dependencies.

**Architecture (owner decision 2026-05-26 — "C-shim, reuse raylib's math"):** raymath.h's 146 functions are `inline` (no symbols). A one-TU shim `#define RAYMATH_IMPLEMENTATION` + `#include raymath.h` emits external definitions (confirmed: raymath.h lines 21/67/75). We compile that shim (mirroring `gen_rgui()`), add raymath.h to the bindgen header so the functions are generated, and write thin inherent-method/operator wrappers (in our own modules — legal since the types are local to `raylib-sys`) that call the FFI raymath fns. The types are `#[repr(C)]` and layout-compatible (guarded by the WS1 `layout_compat` test), so passing them by value to the C functions is sound.

**Type sourcing (owner decision 2026-05-26, clarified):** the hand-definitions were a *legacy of the mint aliasing* — blocklisting was only needed to alias the types to `mint`. With mint gone:
- **`Vector2/3/4` + `Matrix` → bindgen-generated** (un-blocklist them). They're plain `typedef struct`s; bindgen emits clean POD structs that track the header automatically (no drift risk). We add methods/operators/conversions via `impl` blocks.
- **`Quaternion` → distinct hand-defined `#[repr(C)] struct Quaternion { pub x, y, z, w: f32 }`** (stays blocklisted). Because C does `typedef Vector4 Quaternion`, bindgen would emit a bare alias with no separate method namespace; a distinct struct (public fields, layout-identical to `Vector4`) gives it clean ops and zero-cost `From`/`Into<Vector4>` at FFI boundaries. Mirrors what the safe crate already exposes.
- **`Rectangle` + `Color` → unchanged** (stay hand-defined/blocklisted — never about mint; they carry rich APIs like `Color::RED` and rectangle collision helpers).

**This is sys-only** — the safe crate keeps compiling against `ffi::*` as before; it *adopts* these types as its public `Vector2` in WS3. `mint`/`glam`/`serde` adapters are **WS2b** (serde on the bindgen-generated types is injected via a `build.rs` `add_derives` callback gated on `CARGO_FEATURE_SERDE`).

**Tech Stack:** Rust 1.85, `cc` (shim compile), `bindgen`, raylib 6.0 `raymath.h`. Branch `6.0-rc`; push to `fork` triggers the green 3-OS CI from WS1.

**Reference:** spec D2; `raylib-sys/src/math.rs` (current mint aliases), `raylib-sys/build.rs` (`gen_bindings` @259, `gen_rgui` @339), `raylib-sys/binding/binding.h`, `raylib/src/core/math.rs` (existing hand-rolled Matrix/Quaternion for reference). raymath.h is the enumeration source for all wrappers.

**Pre-flight:** `git rev-parse --abbrev-ref HEAD` → `6.0-rc`; `cargo build -p raylib-sys` green; `cargo test -p raylib-sys --test layout_compat` passes (this test MUST keep passing through every task — it's the layout guard).

**Scope boundary (every task):** modify only `raylib-sys/**`. Do NOT edit `raylib/src/**` (safe crate = WS3). The WS1 `layout_compat` + `symbol_presence` tests must stay green.

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `raylib-sys/build.rs` | Un-blocklist `Vector2/3/4` + `Matrix` (let bindgen generate); trim `TypeOverrideCallback` to Rectangle/Color | 1 |
| `raylib-sys/src/math.rs` | Drop mint aliases; keep distinct `#[repr(C)]` `Quaternion` (pub x/y/z/w) + `From`/`Into<Vector4>`; keep `Rectangle` | 1 |
| `raylib-sys/binding/raymath_shim.c` | `#define RAYMATH_IMPLEMENTATION` + include raymath.h (create) | 2 |
| `raylib-sys/binding/binding.h` | add `#include "../raylib/src/raymath.h"` so bindgen generates raymath fns (modify) | 2 |
| `raylib-sys/build.rs` | `gen_raymath()` compiles the shim (mirror `gen_rgui`); call it (modify) | 2 |
| `raylib-sys/src/vector_math.rs` | Inherent methods + operator traits on Vector2/3/4 wrapping `ffi::VectorN*` (create) | 3 |
| `raylib-sys/src/matrix_quat_math.rs` | Inherent methods + operators on Matrix/Quaternion wrapping raymath (create) | 4 |
| `raylib-sys/Cargo.toml` | drop the hard `mint` dependency (modify) | 5 |
| `raylib-sys/tests/raymath_wrappers.rs` | Tier-1 known-value tests for the wrappers (create across tasks 3–4) | 3,4 |

---

## Task 1: Un-blocklist the POD types; keep a distinct `Quaternion`

**Files:** Modify `raylib-sys/build.rs` (blocklist + `TypeOverrideCallback`) and `raylib-sys/src/math.rs`.

The hand-definitions were a legacy of the mint aliasing. Let bindgen generate `Vector2/3/4` + `Matrix`; keep `Quaternion` distinct (C aliases it to `Vector4`); leave `Rectangle`/`Color` alone.

- [ ] **Step 1: The layout guard is the contract.** Run `cargo test -p raylib-sys --test layout_compat` → PASS now. It must still PASS after this task (it now also guards that the *bindgen-generated* types have the expected layout).

- [ ] **Step 2: Un-blocklist `Vector2/3/4` + `Matrix` in `raylib-sys/build.rs`.** In `gen_bindings()` (~line 298), delete the four lines `.blocklist_type("Vector2")`, `"Vector3"`, `"Vector4"`, `"Matrix"`. **Keep** `.blocklist_type("Quaternion")`, `.blocklist_type("Rectangle")`, `.blocklist_type("Color")`. In the `TypeOverrideCallback` `overridden_types` array (~line 61), remove `"Vector2"`, `"Vector3"`, `"Vector4"`, `"Matrix"`; keep `"Quaternion"`, `"Rectangle"`, `"Color"` (bindgen now derives Copy/Debug/PartialEq on the generated Vector*/Matrix itself).

- [ ] **Step 3: Rewrite the type section of `raylib-sys/src/math.rs`.** Remove `pub use mint;` and all five `pub type X = mint::...` aliases. bindgen now supplies `Vector2/3/4` and `Matrix`. Add a **distinct** transparent Quaternion (public fields, layout = `Vector4`) and its conversions; keep the existing `Rectangle` struct + collision methods unchanged:

```rust
/// Quaternion. C aliases this to Vector4 (`typedef Vector4 Quaternion`); we use a
/// distinct, layout-identical #[repr(C)] struct so it has its own method namespace.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Quaternion { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

impl Quaternion {
    #[inline] pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self { Self { x, y, z, w } }
}
// Zero-cost interchange with the FFI Vector4 that raymath's Quaternion* fns actually take/return.
impl From<crate::Vector4> for Quaternion {
    #[inline] fn from(v: crate::Vector4) -> Self { Self { x: v.x, y: v.y, z: v.z, w: v.w } }
}
impl From<Quaternion> for crate::Vector4 {
    #[inline] fn from(q: Quaternion) -> Self { crate::Vector4 { x: q.x, y: q.y, z: q.z, w: q.w } }
}
```

- [ ] **Step 4: Build + guards.** `cargo build -p raylib-sys` → success (bindgen generates Vector*/Matrix; we define Quaternion). Then `cargo test -p raylib-sys --test layout_compat --test symbol_presence` → all PASS (layout unchanged: Vector2=8, Vector3=12, Vector4=16, Matrix=64, Quaternion=16). If `layout_compat` fails on a *generated* type, that's a real signal bindgen produced an unexpected layout — investigate before proceeding. `cargo fmt --all`.

- [ ] **Step 5: Commit.**
```bash
git add raylib-sys/build.rs raylib-sys/src/math.rs
git commit -m "$(printf 'refactor(ws2a)!: bindgen-generate Vector*/Matrix; distinct Quaternion\n\nUn-blocklist Vector2/3/4 + Matrix (the hand-defs were only there to alias\nto mint); trim TypeOverrideCallback. Keep Quaternion as a distinct\n#[repr(C)] newtype (C aliases it to Vector4) with From/Into<Vector4>.\nRectangle/Color unchanged. Layout still guarded by layout_compat.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: raymath C-shim + bindgen wiring

**Files:** Create `raylib-sys/binding/raymath_shim.c`; modify `raylib-sys/binding/binding.h`, `raylib-sys/build.rs`.

- [ ] **Step 1: Create the shim** `raylib-sys/binding/raymath_shim.c`:
```c
/* Emit external (out-of-line) symbols for raymath's otherwise-inline functions,
   so raylib-rs can call raylib's own math through FFI. See raymath.h (RAYMATH_IMPLEMENTATION). */
#define RAYMATH_IMPLEMENTATION
#include "../raylib/src/raymath.h"
```

- [ ] **Step 2: Make bindgen generate the raymath functions.** In `raylib-sys/binding/binding.h`, add near the other includes:
```c
#include "../raylib/src/raymath.h"
```
(Same relative-path style as the existing `#include "../raylib/src/rlgl.h"`.) bindgen already blocklists Vector2/3/4/Matrix/Quaternion, so the generated raymath fn signatures will reference our native types.

- [ ] **Step 3: Compile + link the shim.** In `raylib-sys/build.rs`, add a function mirroring `gen_rgui()` (around line 339):
```rust
fn gen_raymath() {
    cc::Build::new()
        .files(vec!["binding/raymath_shim.c"])
        .include("binding")
        .warnings(false)
        .extra_warnings(false)
        .compile("raymath_shim");
}
```
and call `gen_raymath();` wherever `gen_rgui()` / `gen_utils()` are called (find those call sites in `main`). Guard it with `#[cfg(not(feature = "nobuild"))]` consistent with the other gen_* calls if applicable.

- [ ] **Step 4: Build sys.** `cargo build -p raylib-sys`. Expected: success, the shim compiles and links. If the linker reports duplicate symbols for raymath fns, raylib's own TUs may also be emitting them — in that case ensure ONLY the shim defines RAYMATH_IMPLEMENTATION (raylib's modules include raymath.h as plain `inline`, which per C99 does not emit external symbols, so this should not happen; if it does, investigate which TU double-defines).

- [ ] **Step 5: Prove the raymath symbols are bound — extend the guard test.** Append to `raylib-sys/tests/symbol_presence.rs` a test:
```rust
#[test]
fn raymath_functions_are_bound() {
    let _ = Vector3DotProduct as *const ();
    let _ = Vector3CrossProduct as *const ();
    let _ = MatrixMultiply as *const ();
    let _ = QuaternionNormalize as *const ();
}
```
Run `cargo test -p raylib-sys --test symbol_presence` → PASS (compiles, links → symbols resolve). If a name doesn't resolve, grep raymath.h for the exact 6.0 name.

- [ ] **Step 6: Commit.**
```bash
git add raylib-sys/binding/raymath_shim.c raylib-sys/binding/binding.h raylib-sys/build.rs raylib-sys/tests/symbol_presence.rs
git commit -m "$(printf 'build(ws2a): emit + bind raymath via C-shim\n\nRAYMATH_IMPLEMENTATION shim emits external symbols for raymath inline fns;\nbinding.h includes raymath.h so bindgen generates them; gen_raymath()\ncompiles+links the shim. Guard test references key raymath symbols.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 3: Vector2/3/4 methods + operators (FFI wrappers)

**Files:** Create `raylib-sys/src/vector_math.rs` (and `mod vector_math;` in `lib.rs`); create/extend `raylib-sys/tests/raymath_wrappers.rs`.

This is bulk wrapper work. **Source of truth:** every `VectorN*` function in `raylib-sys/raylib/src/raymath.h`. Each becomes a method (or operator-trait impl) on the matching native type, delegating to the FFI fn.

- [ ] **Step 1: Enumerate.** `grep -oE "^RMAPI [A-Za-z0-9_ \*]+ Vector[234][A-Za-z]+" raylib-sys/raylib/src/raymath.h | sort` → the full list (e.g. `Vector2Add`, `Vector2Length`, `Vector3DotProduct`, `Vector3CrossProduct`, `Vector3Normalize`, `Vector3Lerp`, …). Cover all of them.

- [ ] **Step 2: Write the wrappers** following this exact pattern (every FFI call gets a `// SAFETY:` per CONTRIBUTE.md — raymath fns are pure value-in/value-out, no preconditions):
```rust
use crate::{Vector2, Vector3, Vector4};

impl Vector3 {
    /// Dot product. (raymath `Vector3DotProduct`)
    #[inline]
    pub fn dot(self, other: Vector3) -> f32 {
        // SAFETY: Vector3DotProduct is pure (value in/out), no preconditions.
        unsafe { crate::Vector3DotProduct(self, other) }
    }
    #[inline]
    pub fn cross(self, other: Vector3) -> Vector3 {
        // SAFETY: pure value-in/value-out.
        unsafe { crate::Vector3CrossProduct(self, other) }
    }
    #[inline]
    pub fn length(self) -> f32 {
        // SAFETY: pure value-in/value-out.
        unsafe { crate::Vector3Length(self) }
    }
    // ... one method per Vector3* raymath fn
}

impl std::ops::Add for Vector3 {
    type Output = Vector3;
    #[inline]
    fn add(self, rhs: Vector3) -> Vector3 {
        // SAFETY: pure value-in/value-out.
        unsafe { crate::Vector3Add(self, rhs) }
    }
}
// + Sub (Vector3Subtract), Mul<f32> (Vector3Scale), Neg (Vector3Negate), AddAssign/SubAssign, etc.
```
Map the common operators: `Add`→`*Add`, `Sub`→`*Subtract`, `Neg`→`*Negate`, `Mul<f32>`→`*Scale`. Name idiomatic methods for the rest (`length`, `length_sqr`, `normalize`(→`*Normalize`), `distance`, `lerp`, `reflect`, `rotate`, `clamp`, …). Keep the C name in a doc comment for traceability. Do Vector2, Vector3, Vector4 in that order in one file.

- [ ] **Step 3: Tier-1 tests** in `raylib-sys/tests/raymath_wrappers.rs` — known-value checks (these call through to raylib's math, so they verify our wrapper calls the RIGHT fn):
```rust
use raylib_sys::{Vector2, Vector3};
#[test]
fn vector3_core_ops() {
    let a = Vector3::new(1.0, 2.0, 2.0);
    assert_eq!(a.length(), 3.0);                       // sqrt(1+4+4)
    assert_eq!(Vector3::new(1.0,0.0,0.0).dot(Vector3::new(0.0,1.0,0.0)), 0.0);
    let c = Vector3::new(1.0,0.0,0.0).cross(Vector3::new(0.0,1.0,0.0));
    assert_eq!(c, Vector3::new(0.0,0.0,1.0));
    assert_eq!(Vector3::new(1.0,1.0,1.0) + Vector3::new(1.0,2.0,3.0), Vector3::new(2.0,3.0,4.0));
}
#[test]
fn vector2_core_ops() {
    assert_eq!(Vector2::new(3.0,4.0).length(), 5.0);
}
```
Add a handful per type covering the common ops (add/sub/scale/dot/cross/length/normalize/lerp). Run `cargo test -p raylib-sys --test raymath_wrappers` → PASS.

- [ ] **Step 4: Build + guards.** `cargo build -p raylib-sys` and `cargo test -p raylib-sys` → green; `cargo fmt --all`.

- [ ] **Step 5: Commit.**
```bash
git add raylib-sys/src/vector_math.rs raylib-sys/src/lib.rs raylib-sys/tests/raymath_wrappers.rs
git commit -m "$(printf 'feat(ws2a): Vector2/3/4 methods + operators via raymath\n\nInherent methods and std operator impls delegating to raylib raymath fns,\ncovering every VectorN* function. Tier-1 known-value tests.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 4: Matrix + Quaternion methods + operators

**Files:** Create `raylib-sys/src/matrix_quat_math.rs` (+ `mod` in `lib.rs`); extend `raylib-sys/tests/raymath_wrappers.rs`.

Same pattern as Task 3. **Source of truth:** every `Matrix*` and `Quaternion*` fn in raymath.h (`grep -oE "^RMAPI [A-Za-z0-9_ \*]+ (Matrix|Quaternion)[A-Za-z]+" raylib-sys/raylib/src/raymath.h | sort`). Reference `raylib/src/core/math.rs` for the idiomatic method NAMES the safe crate previously used (e.g. `Matrix::identity`, `translate`, `rotate_xyz`, `determinant`; `Quaternion::normalize`, `slerp`, `from_axis_angle`) so WS3 adoption is smooth — but delegate the bodies to the FFI raymath fns rather than reimplementing.

- [ ] **Step 1: Enumerate** the Matrix*/Quaternion* fns (grep above).
- [ ] **Step 2: Wrappers** — inherent methods + operators (`Mul for Matrix`→`MatrixMultiply`, `Add`/`Sub`→`MatrixAdd`/`MatrixSubtract`; `Mul for Quaternion`→`QuaternionMultiply`), each `unsafe { crate::MatrixXxx(...) }` with a `// SAFETY:` comment. Provide `Matrix::identity()` (→`MatrixIdentity`) and `Quaternion::identity()` (→`QuaternionIdentity`).
  - **Quaternion conversion note:** because C does `typedef Vector4 Quaternion`, bindgen resolves the blocklisted alias to `Vector4`, so the generated `Quaternion*` fns take/return `crate::Vector4`. Quaternion methods therefore convert at the call: `pub fn normalize(self) -> Self { /* SAFETY: pure, layout-identical */ unsafe { crate::QuaternionNormalize(self.into()) }.into() }`. The `From`/`Into<Vector4>` impls from Task 1 make this zero-cost. **Verify the generated signatures** (`Vector4` vs `Quaternion`) early in this task and adjust the conversion accordingly — if bindgen unexpectedly keeps the `Quaternion` name, no conversion is needed.
- [ ] **Step 3: Tier-1 tests** — e.g. `Matrix::identity()` is the identity; `MatrixMultiply(identity, m) == m`; `Quaternion::identity().normalize()` stays identity; determinant of identity == 1.
```rust
use raylib_sys::{Matrix, Quaternion};
#[test]
fn matrix_identity_is_neutral() {
    let id = Matrix::identity();
    let m = Matrix::translate(1.0, 2.0, 3.0); // -> MatrixTranslate
    assert_eq!(id * m, m);
    assert_eq!(id.determinant(), 1.0);        // -> MatrixDeterminant
}
```
- [ ] **Step 4: Build + tests + fmt** green.
- [ ] **Step 5: Commit** (`feat(ws2a): Matrix + Quaternion methods + operators via raymath`, with the Claude trailer).

---

## Task 5: Drop the hard `mint` dependency; verify zero-dep default

**Files:** Modify `raylib-sys/Cargo.toml`.

- [ ] **Step 1: Remove the `mint` dependency.** Delete `mint = { version = "0.5" }` from `[dependencies]` in `raylib-sys/Cargo.toml`. (The `serde = ["dep:serde", "mint/serde"]` feature line references mint — temporarily change it to just `serde = ["dep:serde"]`; full mint/glam/serde adapters are WS2b. Leave a `# TODO(ws2b): re-add mint/glam/serde as optional` comment.)

- [ ] **Step 2: Verify zero math deps in the default build.**
```bash
cargo build -p raylib-sys
cargo tree -p raylib-sys -e no-dev | grep -iE "mint|glam" && echo "FOUND_MATH_DEP (bad)" || echo "NO_MATH_DEPS (good)"
```
Expected: `NO_MATH_DEPS (good)`.

- [ ] **Step 3: Full sys verification.** `cargo test -p raylib-sys` (symbol_presence, layout_compat, raymath_wrappers all green); `cargo fmt --all --check`.

- [ ] **Step 4: Commit + push to fork (triggers the WS1 3-OS CI).**
```bash
git add raylib-sys/Cargo.toml
git commit -m "$(printf 'feat(ws2a)!: drop hard mint dependency from raylib-sys\n\nDefault build now pulls zero math crates; native types own their math via\nthe raymath shim. mint/glam/serde return as optional features in WS2b.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
git push fork 6.0-rc
```
- [ ] **Step 5: Watch CI** (`gh run list --repo Dacode45/ms-raylib-rs --branch 6.0-rc --limit 1`, then watch) → the 3-OS `build-sys` matrix + fmt green. Fix any platform-specific shim issues (controller-orchestrated) until green.

---

## Done criteria (WS2a)

- [ ] `Vector2/3/4` + `Matrix` are bindgen-generated (un-blocklisted); `Quaternion` is a distinct hand-defined `#[repr(C)]` newtype with `From`/`Into<Vector4>`; no `mint` aliases; `layout_compat` still green.
- [ ] raymath shim compiles + links; raymath fns bound (symbol guard green).
- [ ] Vector2/3/4 + Matrix + Quaternion expose methods/operators for the raymath fns; Tier-1 wrapper tests green.
- [ ] Default `raylib-sys` build pulls **zero** math crates (`cargo tree` clean).
- [ ] 3-OS CI green on the fork. Safe crate untouched (still WS3); `raylib/src/**` unchanged.

## Self-review (against spec)

- **D2 coverage:** native repr(C) types owned by sys ✓; raymath provided (via the C-shim the owner chose — a deliberate, recorded variant of D2's "port raymath," trading per-call FFI cost for reuse of raylib's tested math) ✓; zero-dep default ✓. **mint/glam/serde optionality is explicitly WS2b**, not this plan.
- **Soundness:** every wrapper's `unsafe` FFI call has a `// SAFETY:` note (raymath fns are pure value-in/value-out); native types are layout-verified, so by-value passing to C is sound.
- **Scope:** sys-only; safe crate (and the 37 WS0-captured errors + adopting these types as public `Vector2`) is WS3. The `layout_compat`/`symbol_presence` guards from WS1 ride along as regression protection.
- **Bulk tasks (3,4):** enumerated from raymath.h with an exact pattern + representative code + tests, since spelling out ~146 wrappers verbatim is impractical and raymath.h is the authoritative list.
