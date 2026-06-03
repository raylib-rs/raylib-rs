# WS5b — Safe Immediate-Mode rlgl Module Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a safe, top-level `raylib/src/rlgl/` module covering rlgl's immediate-mode drawing surface — the matrix stack, immediate-mode vertex streams, the render-state toggles the examples use, and ergonomic bind-methods that accept the crate's safe `Texture2D`/`Shader` — render-verified with the WS4 harness.

**Architecture:** rlgl is the GL-abstraction layer beneath raylib's modules; it wraps a *separate* header (`rlgl.h`), so the safe module is **top-level** (sibling of `core/` and `rgui/`), mirroring `rgui/`. Drawing happens during a `BeginDrawing`/`EndDrawing` frame, so the API hangs off the draw-handle types via a `RaylibRlgl` extension trait blanket-implemented for `D: RaylibDraw` (the same pattern as `RaylibTextureModeExt`). Two RAII guards enforce balance: `RlMatrix` (auto-`rlPopMatrix` on drop) and `RlImmediate` (auto-`rlEnd` on drop). The GL-object *lifecycle* (create/destroy textures/shaders/framebuffers/VBOs) stays with the existing safe RAII and raw `ffi`; rlgl only *binds* handles the caller already owns.

**Tech Stack:** Rust (edition 2024, MSRV 1.85). rlgl FFI already exists in `raylib-sys` (161 `rl*` fns, generated from `rlgl.h`). `raylib::test_harness` for Tier-2 verification.

**Spec:** `docs/superpowers/specs/2026-05-26-ws5-raygui-rlgl-design.md` (§WS5b). **Depends on:** WS5-prep (normalized harness). Independent of WS5a, but lands after it.

**Backlog cherry-picks (attribute in commits):** issue #234 (safe OpenGL draw-call bindings), issue #179 (`rlPushMatrix` family) — per `inventory.md`.

---

## Key facts (verified against the tree — don't re-derive)

- rlgl FFI is in `raylib-sys` (bindgen from `binding/binding.h` → `#include "../raylib/src/rlgl.h"`). Confirmed available symbols + signatures:
  - Matrix: `rlPushMatrix()`, `rlPopMatrix()`, `rlLoadIdentity()`, `rlTranslatef(f32,f32,f32)`, `rlRotatef(angle:f32, x:f32,y:f32,z:f32)`, `rlScalef(f32,f32,f32)`, `rlMultMatrixf(*const f32)`, `rlMatrixMode(c_int)`, `rlOrtho(f64,f64,f64,f64,f64,f64)`, `rlFrustum(f64×6)`, `rlViewport(i32,i32,i32,i32)`, `rlSetMatrixModelview(Matrix)`, `rlSetMatrixProjection(Matrix)`.
  - Immediate: `rlBegin(c_int)`, `rlEnd()`, `rlVertex2f(f32,f32)`, `rlVertex2i(i32,i32)`, `rlVertex3f(f32,f32,f32)`, `rlTexCoord2f(f32,f32)`, `rlNormal3f(f32,f32,f32)`, `rlColor4ub(u8,u8,u8,u8)`, `rlColor3f(f32,f32,f32)`, `rlColor4f(f32,f32,f32,f32)`.
  - State: `rlEnableDepthTest()`, `rlDisableDepthTest()`, `rlEnableBackfaceCulling()`, `rlDisableBackfaceCulling()`.
  - Bind handles (raw GL ids): `rlSetTexture(id:u32)`, `rlEnableTexture(id:u32)`, `rlDisableTexture()`, `rlEnableShader(id:u32)`, `rlSetShader(id:u32, locs:*mut i32)`, `rlActiveTextureSlot(slot:i32)`.
- Mode constants in the bindings: `ffi::RL_LINES = 1`, `ffi::RL_TRIANGLES = 4`, `ffi::RL_QUADS = 7`, `ffi::RL_MODELVIEW = 5888`, `ffi::RL_PROJECTION = 5889`, `ffi::RL_TEXTURE = 5890`.
- Safe wrappers `Deref` to their ffi struct (`make_thin_wrapper!`), so `Texture2D` exposes `.id: u32` and `Shader` exposes `.id: u32` + `.locs: *mut i32` directly. (Confirmed: `core/macros.rs` `deref_impl_wrapper!`.)
- Draw-handle RAII pattern to mirror: `RaylibTextureMode<'a,'b,T>` in `core/drawing.rs` — a `&'a mut T` newtype with `Drop` calling the End fn and `Deref`/`DerefMut` to `T`; entry via the `RaylibTextureModeExt` trait (`begin_*` returns the guard; closure `draw_*` form too). `RaylibDraw` is at `core/drawing.rs:492`; `RaylibDrawHandle` implements it.
- `Matrix`, `Vector2`, `Vector3`, `Color`, `Texture2D`, `Shader` are in the prelude.
- The four rlgl showcase examples (`shapes_rlgl_triangle`, `shapes_rlgl_color_wheel`, `models_rlgl_solar_system`) use only matrix-stack + immediate-mode + color (no texture/shader binding); `rlgl_standalone` bypasses `RaylibHandle` and stays a raw-ffi example (out of scope).
- Tier-2 test/CI feature set: `software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION` (rlgl needs no extra feature; immediate-mode draws work under rlsw — that's how the harness renders).
- Commit trailer: `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`. Branch `6.0-rc`; no worktree, no merge.

---

## File Structure (target)

- `raylib/src/rlgl/mod.rs` — **new.** Module doc; declares submodules; re-exports `RaylibRlgl`, `RlMatrix`, `RlImmediate`, `DrawMode`, `MatrixMode`; defines `RaylibRlgl` ext trait + blanket `impl<D: RaylibDraw> RaylibRlgl for D {}`.
- `raylib/src/rlgl/matrix.rs` — **new.** `MatrixMode` enum + `RlMatrix` guard + matrix methods.
- `raylib/src/rlgl/immediate.rs` — **new.** `DrawMode` enum + `RlImmediate` guard + vertex/color/texcoord/normal methods.
- `raylib/src/rlgl/state.rs` — **new.** Render-state toggles + bind-safe-handle methods.
- `raylib/src/lib.rs` — **modify.** Add `pub mod rlgl;`.
- `raylib/src/prelude.rs` — **modify.** Add `pub use crate::rlgl::*;`.
- `raylib/tests/render_rlgl.rs` — **new.** Tier-2 render test.
- `.github/workflows/baseline.yml` — **modify.** Add the rlgl render-test step.

The `RaylibRlgl` ext trait holds the entry points; `RlMatrix`/`RlImmediate` are the guards. Methods that are only valid between `rlBegin`/`rlEnd` live on `RlImmediate` (not the trait), so they cannot be called outside an immediate-mode block.

---

## Task 1: Module skeleton — enums, ext trait, wiring

**Files:**
- Create: `raylib/src/rlgl/mod.rs`, `raylib/src/rlgl/matrix.rs`, `raylib/src/rlgl/immediate.rs`, `raylib/src/rlgl/state.rs`
- Modify: `raylib/src/lib.rs`, `raylib/src/prelude.rs`

- [ ] **Step 1: Create the enums (in their files)**

`raylib/src/rlgl/immediate.rs`:
```rust
use crate::core::drawing::RaylibDraw;
use crate::ffi;
use crate::ffi::Color;

/// Primitive type for an immediate-mode vertex stream (`rlBegin`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawMode {
    /// Line list (`RL_LINES`).
    Lines = ffi::RL_LINES as i32,
    /// Triangle list (`RL_TRIANGLES`).
    Triangles = ffi::RL_TRIANGLES as i32,
    /// Quad list (`RL_QUADS`).
    Quads = ffi::RL_QUADS as i32,
}
```

`raylib/src/rlgl/matrix.rs`:
```rust
use crate::core::drawing::RaylibDraw;
use crate::ffi;
use crate::math::Matrix;

/// Target matrix for `rl_matrix_mode`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixMode {
    /// Model-view matrix (`RL_MODELVIEW`).
    ModelView = ffi::RL_MODELVIEW as i32,
    /// Projection matrix (`RL_PROJECTION`).
    Projection = ffi::RL_PROJECTION as i32,
    /// Texture matrix (`RL_TEXTURE`).
    Texture = ffi::RL_TEXTURE as i32,
}
```

`raylib/src/rlgl/state.rs`: start with just `use` lines:
```rust
use crate::core::drawing::RaylibDraw;
use crate::core::shaders::Shader;
use crate::core::texture::Texture2D;
use crate::ffi;
```

- [ ] **Step 2: Create `mod.rs` with the ext trait (entry points are stubbed to compile; filled in Tasks 2–4)**

```rust
//! Safe wrappers for rlgl's immediate-mode drawing layer (the GL abstraction
//! beneath raylib's modules; wraps `rlgl.h`). Covers the matrix stack, immediate-
//! mode vertex streams, a few render-state toggles, and ergonomic methods that
//! bind the crate's safe [`Texture2D`](crate::core::texture::Texture2D) /
//! [`Shader`](crate::core::shaders::Shader) handles. GL-object *lifecycle*
//! (create/destroy) stays with those safe types and raw [`ffi`](crate::ffi); the
//! full 161-fn rlgl surface remains available there as a power-user escape hatch.
//!
//! All entry points hang off the draw-handle types (via [`RaylibRlgl`]), so they
//! are only callable inside a `begin_drawing` frame.

mod immediate;
mod matrix;
mod state;

pub use immediate::{DrawMode, RlImmediate};
pub use matrix::{MatrixMode, RlMatrix};

use crate::core::drawing::RaylibDraw;

/// Immediate-mode rlgl drawing, available on every draw handle. See the module docs.
pub trait RaylibRlgl: RaylibDraw + Sized {
    // Filled in Tasks 2–4.
}

impl<D: RaylibDraw> RaylibRlgl for D {}
```

- [ ] **Step 3: Wire `lib.rs` and `prelude.rs`**

In `raylib/src/lib.rs`, add near the other `pub mod` lines (after `pub mod rgui;` if present, else near `pub mod core;`):
```rust
pub mod rlgl;
```
In `raylib/src/prelude.rs`, add after the `pub use crate::rgui::*;` line:
```rust
pub use crate::rlgl::*;
```

- [ ] **Step 4: Add placeholder guards so the module compiles**

In `immediate.rs`, add:
```rust
/// RAII guard for an immediate-mode vertex stream: `rlBegin` on creation,
/// `rlEnd` on drop. Vertex/color methods (Task 3) live here so they cannot be
/// called outside a `rlBegin`/`rlEnd` block.
pub struct RlImmediate<'a, T: RaylibDraw>(&'a mut T);

impl<'a, T: RaylibDraw> Drop for RlImmediate<'a, T> {
    fn drop(&mut self) {
        // SAFETY: paired with the `rlBegin` that created this guard; raylib's
        // rlgl tolerates rlEnd after rlBegin on the active batch.
        unsafe { ffi::rlEnd() }
    }
}
```

In `matrix.rs`, add:
```rust
/// RAII guard for a pushed matrix: `rlPushMatrix` on creation, `rlPopMatrix` on
/// drop. Deref's to the draw handle so you can keep drawing within the pushed
/// transform. Transform methods (Task 2) are on the [`RaylibRlgl`] trait.
pub struct RlMatrix<'a, T: crate::core::drawing::RaylibDraw>(&'a mut T);

impl<'a, T: crate::core::drawing::RaylibDraw> Drop for RlMatrix<'a, T> {
    fn drop(&mut self) {
        // SAFETY: paired with the `rlPushMatrix` that created this guard.
        unsafe { ffi::rlPopMatrix() }
    }
}

impl<'a, T: crate::core::drawing::RaylibDraw> std::ops::Deref for RlMatrix<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}
impl<'a, T: crate::core::drawing::RaylibDraw> std::ops::DerefMut for RlMatrix<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}
```

- [ ] **Step 5: Build**

```
cargo build -p raylib
```
Expected: clean (the `&mut T` fields are unused-warning-free because `Drop`/`Deref` use them; if `RlImmediate`'s field triggers dead-code until Task 3, add `#[allow(dead_code)]` temporarily and remove it in Task 3). `DrawMode`/`MatrixMode`/`RaylibRlgl` may be unused until later tasks — that is fine for a `pub` API (no dead-code warning on pub items).

- [ ] **Step 6: Commit**

```bash
git add raylib/src/rlgl raylib/src/lib.rs raylib/src/prelude.rs
git commit -m "feat(ws5b): rlgl module skeleton — DrawMode/MatrixMode, RaylibRlgl, guards

Refs #234, #179 (rlgl safe bindings).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: Matrix stack (`matrix.rs` + `RaylibRlgl` methods)

**Files:**
- Modify: `raylib/src/rlgl/matrix.rs`, `raylib/src/rlgl/mod.rs`

- [ ] **Step 1: Add the matrix entry points to the `RaylibRlgl` trait** (in `mod.rs`, inside the trait body)

```rust
    /// Push the current matrix onto the stack; the returned guard pops it on drop.
    /// Apply transforms (`rl_translatef`/`rl_rotatef`/`rl_scalef`) through the guard.
    #[inline]
    fn rl_push_matrix(&mut self) -> RlMatrix<'_, Self> {
        // SAFETY: the returned RlMatrix's Drop calls the matching rlPopMatrix.
        unsafe { ffi::rlPushMatrix() };
        RlMatrix::new(self)
    }
    /// Reset the current matrix to identity.
    #[inline]
    fn rl_load_identity(&mut self) {
        unsafe { ffi::rlLoadIdentity() }
    }
    /// Multiply the current matrix by a translation.
    #[inline]
    fn rl_translatef(&mut self, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlTranslatef(x, y, z) }
    }
    /// Multiply the current matrix by a rotation of `angle` degrees about (x,y,z).
    #[inline]
    fn rl_rotatef(&mut self, angle: f32, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlRotatef(angle, x, y, z) }
    }
    /// Multiply the current matrix by a scale.
    #[inline]
    fn rl_scalef(&mut self, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlScalef(x, y, z) }
    }
    /// Multiply the current matrix by `mat`.
    #[inline]
    fn rl_mult_matrixf(&mut self, mat: crate::math::Matrix) {
        let m: [f32; 16] = mat.to_array();
        unsafe { ffi::rlMultMatrixf(m.as_ptr()) }
    }
    /// Choose which matrix subsequent operations affect.
    #[inline]
    fn rl_matrix_mode(&mut self, mode: MatrixMode) {
        unsafe { ffi::rlMatrixMode(mode as i32) }
    }
    /// Multiply the current matrix by an orthographic projection.
    #[inline]
    fn rl_ortho(&mut self, left: f64, right: f64, bottom: f64, top: f64, near: f64, far: f64) {
        unsafe { ffi::rlOrtho(left, right, bottom, top, near, far) }
    }
    /// Set the projection matrix directly.
    #[inline]
    fn rl_set_matrix_projection(&mut self, proj: crate::math::Matrix) {
        unsafe { ffi::rlSetMatrixProjection(proj.into()) }
    }
    /// Set the model-view matrix directly.
    #[inline]
    fn rl_set_matrix_modelview(&mut self, view: crate::math::Matrix) {
        unsafe { ffi::rlSetMatrixModelview(view.into()) }
    }
```

Add the imports `MatrixMode`, `RlMatrix` to `mod.rs` (already `pub use`d) and `use crate::ffi;` at the top of `mod.rs`.

Note on `rl_mult_matrixf` / `rl_set_matrix_*`: confirm the safe `Matrix` → `[f32;16]` / `ffi::Matrix` conversions. `Matrix::to_array()` may or may not exist; if not, build the array field-by-field, or use `Into<ffi::Matrix>` for the `rlSetMatrix*` calls (those take `Matrix` by value — `proj.into()` where `Matrix: Into<ffi::Matrix>`). For `rlMultMatrixf` (needs `*const f32`), use whatever column-major accessor the math module provides; if none, `rl_mult_matrixf` may be dropped from this task and noted as deferred (it is not used by any showcase example). Decide by what compiles; do not invent an accessor.

- [ ] **Step 2: Add the `RlMatrix::new` constructor** in `matrix.rs`:

```rust
impl<'a, T: crate::core::drawing::RaylibDraw> RlMatrix<'a, T> {
    pub(crate) fn new(parent: &'a mut T) -> Self {
        RlMatrix(parent)
    }
}
```

- [ ] **Step 3: Build**

```
cargo build -p raylib
```
Expected: clean. If `Matrix` conversions don't exist as assumed, adjust per the Step 1 note and rebuild.

- [ ] **Step 4: Commit**

```bash
git add raylib/src/rlgl
git commit -m "feat(ws5b): rlgl matrix stack — RlMatrix guard + transform/projection methods

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: Immediate-mode vertex stream (`immediate.rs` + entry point)

**Files:**
- Modify: `raylib/src/rlgl/immediate.rs`, `raylib/src/rlgl/mod.rs`

- [ ] **Step 1: Add the immediate-mode entry points to `RaylibRlgl`** (in `mod.rs`)

```rust
    /// Begin an immediate-mode vertex stream; the returned guard ends it on drop.
    /// Emit vertices/colors through the guard. Prefer [`rl_draw`](RaylibRlgl::rl_draw).
    #[inline]
    fn rl_begin(&mut self, mode: DrawMode) -> RlImmediate<'_, Self> {
        // SAFETY: the returned RlImmediate's Drop calls the matching rlEnd.
        unsafe { ffi::rlBegin(mode as i32) };
        RlImmediate::new(self)
    }
    /// Run `body` inside an immediate-mode block, ending it afterwards (closure form).
    #[inline]
    fn rl_draw(&mut self, mode: DrawMode, body: impl FnOnce(&mut RlImmediate<'_, Self>)) {
        let mut v = self.rl_begin(mode);
        body(&mut v);
    }
```

- [ ] **Step 2: Add `RlImmediate::new` + the vertex/color/texcoord/normal methods** in `immediate.rs`:

```rust
impl<'a, T: RaylibDraw> RlImmediate<'a, T> {
    pub(crate) fn new(parent: &'a mut T) -> Self {
        RlImmediate(parent)
    }
    /// Emit a 2D vertex.
    #[inline]
    pub fn vertex2f(&mut self, x: f32, y: f32) {
        unsafe { ffi::rlVertex2f(x, y) }
    }
    /// Emit a 3D vertex.
    #[inline]
    pub fn vertex3f(&mut self, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlVertex3f(x, y, z) }
    }
    /// Set the current vertex color (8-bit RGBA).
    #[inline]
    pub fn color4ub(&mut self, color: impl Into<Color>) {
        let c = color.into();
        unsafe { ffi::rlColor4ub(c.r, c.g, c.b, c.a) }
    }
    /// Set the current vertex color (float RGB).
    #[inline]
    pub fn color3f(&mut self, r: f32, g: f32, b: f32) {
        unsafe { ffi::rlColor3f(r, g, b) }
    }
    /// Set the current vertex color (float RGBA).
    #[inline]
    pub fn color4f(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe { ffi::rlColor4f(r, g, b, a) }
    }
    /// Set the current texture coordinate.
    #[inline]
    pub fn texcoord2f(&mut self, x: f32, y: f32) {
        unsafe { ffi::rlTexCoord2f(x, y) }
    }
    /// Set the current normal vector.
    #[inline]
    pub fn normal3f(&mut self, x: f32, y: f32, z: f32) {
        unsafe { ffi::rlNormal3f(x, y, z) }
    }
}
```

Remove any temporary `#[allow(dead_code)]` added in Task 1 Step 4 (the field is now used).

- [ ] **Step 3: Build + a doctest on `rl_draw`**

Add a doctest to the `rl_draw` method doc (in `mod.rs`):
```rust
    /// # Example
    /// ```no_run
    /// use raylib::prelude::*;
    /// fn frame(d: &mut RaylibDrawHandle) {
    ///     d.rl_draw(DrawMode::Triangles, |v| {
    ///         v.color4ub(Color::RED);
    ///         v.vertex2f(0.0, 0.0);
    ///         v.vertex2f(100.0, 0.0);
    ///         v.vertex2f(50.0, 100.0);
    ///     });
    /// }
    /// ```
```

Run:
```
cargo build -p raylib
cargo test -p raylib --doc rlgl
```
Expected: clean; doctest compiles (`no_run`).

- [ ] **Step 4: Commit**

```bash
git add raylib/src/rlgl
git commit -m "feat(ws5b): rlgl immediate mode — RlImmediate guard + rl_begin/rl_draw

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: Render-state toggles + bind-safe-handle methods (`state.rs`)

**Files:**
- Modify: `raylib/src/rlgl/state.rs`, `raylib/src/rlgl/mod.rs`

These go on `RaylibRlgl` (they affect global rlgl state, valid during drawing). Put the bodies in a second blanket-impl-backed helper, or simply add them to the `RaylibRlgl` trait in `mod.rs` (keep all trait methods in `mod.rs` for cohesion; `state.rs` holds only the imports/any helpers). For this task, add them to the `RaylibRlgl` trait body in `mod.rs` and delete the now-empty `state.rs` if it holds nothing — OR keep `state.rs` and move ALL trait methods there as `RaylibRlglState`. **Chosen:** keep them on the single `RaylibRlgl` trait in `mod.rs` (the trait is still small); remove `state.rs` and its `mod state;` if it ends up empty. (If you prefer the file, that's acceptable — but keep one trait.)

- [ ] **Step 1: Add render-state + bind methods to `RaylibRlgl`** (in `mod.rs`)

```rust
    /// Enable depth testing.
    #[inline]
    fn rl_enable_depth_test(&mut self) {
        unsafe { ffi::rlEnableDepthTest() }
    }
    /// Disable depth testing.
    #[inline]
    fn rl_disable_depth_test(&mut self) {
        unsafe { ffi::rlDisableDepthTest() }
    }
    /// Enable back-face culling.
    #[inline]
    fn rl_enable_backface_culling(&mut self) {
        unsafe { ffi::rlEnableBackfaceCulling() }
    }
    /// Disable back-face culling.
    #[inline]
    fn rl_disable_backface_culling(&mut self) {
        unsafe { ffi::rlDisableBackfaceCulling() }
    }
    /// Set the active texture for subsequent immediate-mode drawing.
    #[inline]
    fn rl_set_texture(&mut self, texture: &crate::core::texture::Texture2D) {
        unsafe { ffi::rlSetTexture(texture.id) }
    }
    /// Enable a texture (by binding the safe handle's GL id).
    #[inline]
    fn rl_enable_texture(&mut self, texture: &crate::core::texture::Texture2D) {
        unsafe { ffi::rlEnableTexture(texture.id) }
    }
    /// Disable the active texture.
    #[inline]
    fn rl_disable_texture(&mut self) {
        unsafe { ffi::rlDisableTexture() }
    }
    /// Select the active multitexture slot.
    #[inline]
    fn rl_active_texture_slot(&mut self, slot: i32) {
        unsafe { ffi::rlActiveTextureSlot(slot) }
    }
    /// Enable a shader program (by binding the safe handle's GL id).
    #[inline]
    fn rl_enable_shader(&mut self, shader: &crate::core::shaders::Shader) {
        unsafe { ffi::rlEnableShader(shader.id) }
    }
    /// Set the active shader and its uniform locations (from the safe handle).
    #[inline]
    fn rl_set_shader(&mut self, shader: &crate::core::shaders::Shader) {
        unsafe { ffi::rlSetShader(shader.id, shader.locs) }
    }
```

Confirm `Texture2D` derefs expose `.id` (it does — `make_thin_wrapper!` → `Deref<Target = ffi::Texture>`, `id: u32`) and `Shader` exposes `.id` + `.locs` (`Deref<Target = ffi::Shader>`). If `state.rs` ends up unused, remove it and its `mod state;`.

- [ ] **Step 2: Add a doctest showing textured immediate-mode drawing** (on `rl_set_texture`)

```rust
    /// # Example
    /// ```no_run
    /// use raylib::prelude::*;
    /// fn frame(d: &mut RaylibDrawHandle, tex: &Texture2D) {
    ///     d.rl_set_texture(tex);
    ///     d.rl_draw(DrawMode::Quads, |v| {
    ///         v.color4ub(Color::WHITE);
    ///         v.texcoord2f(0.0, 0.0); v.vertex2f(0.0, 0.0);
    ///         v.texcoord2f(1.0, 0.0); v.vertex2f(64.0, 0.0);
    ///         v.texcoord2f(1.0, 1.0); v.vertex2f(64.0, 64.0);
    ///         v.texcoord2f(0.0, 1.0); v.vertex2f(0.0, 64.0);
    ///     });
    ///     d.rl_disable_texture();
    /// }
    /// ```
```

- [ ] **Step 3: Build + clippy + doctests**

```
cargo build -p raylib
cargo clippy -p raylib -- -D warnings 2>&1 | grep -E "rlgl[/\\\\]|rlgl::" ; echo "(empty = no rlgl clippy hits)"
cargo test -p raylib --doc rlgl
```
Expected: clean; doctests compile.

- [ ] **Step 4: Commit**

```bash
git add raylib/src/rlgl
git commit -m "feat(ws5b): rlgl render-state toggles + bind &Texture2D/&Shader handles

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: Tier-2 render test (matrix-transformed immediate triangle)

**Files:**
- Create: `raylib/tests/render_rlgl.rs`
- Modify: `.github/workflows/baseline.yml`

- [ ] **Step 1: Write the render test** (normalized harness, natural coords/colors)

```rust
//! WS5b Tier-2: rlgl immediate-mode + matrix stack draw into the software
//! framebuffer. Headless. Uses the WS5-prep normalized `render_frame`.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;

#[test]
fn rlgl_immediate_triangle_renders() {
    raylib::test_harness::with_headless(100, 100, |rl, thread| {
        let img = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            // Draw a red triangle in screen space via immediate mode, shifted right
            // by +30px using the matrix stack. raygui/rlsw use a top-left ortho
            // projection in 2D, so screen coords map directly.
            let mut m = d.rl_push_matrix();
            m.rl_translatef(30.0, 0.0, 0.0);
            m.rl_draw(DrawMode::Triangles, |v| {
                v.color4ub(Color::RED);
                v.vertex2f(20.0, 20.0);
                v.vertex2f(60.0, 20.0);
                v.vertex2f(40.0, 70.0);
            });
            // m drops here -> rlPopMatrix
        });

        // Centroid of the (translated) triangle: screen ~ (40+30, 36) = (70, 36).
        // Count red pixels in the triangle's translated bounding box to confirm fill.
        let mut red = 0u32;
        for y in 20..70 {
            for x in 50..90 {
                let p = raylib::test_harness::pixel_at(&img, x, y);
                if p.r > 150 && p.g < 90 && p.b < 90 {
                    red += 1;
                }
            }
        }
        assert!(red > 200, "expected a filled red triangle, got {red} red px");
    });
}
```

- [ ] **Step 2: Run the test**

```
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test render_rlgl -- --test-threads=1 --nocapture
```
Expected: PASS. If the triangle lands elsewhere (the 2D projection origin/orientation under rlsw may differ from the assumption), print the red-pixel count and the bounding box, then adjust the probe window (and/or drop the translate) so the test still distinguishes "triangle drew" (red > 200) from "blank" (0). The color check (red, not blue) also verifies normalization is active. Document the observed counts in a comment.

- [ ] **Step 3: Add the CI step** (`.github/workflows/baseline.yml`, after the raygui render step)

```yaml
      - name: Tier-2 rlgl render tests
        run: cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION --test render_rlgl -- --test-threads=1
```

- [ ] **Step 4: Commit**

```bash
git add raylib/tests/render_rlgl.rs .github/workflows/baseline.yml
git commit -m "test(ws5b): Tier-2 render test for rlgl immediate mode + matrix stack

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: Docs note + inventory attribution + final verification

**Files:**
- Modify: `docs/superpowers/inventory.md`
- Create: `docs/superpowers/notes/ws5-complete.md` (combined WS5a + WS5b completion note)

- [ ] **Step 1: Mark #234 and #179 addressed in `inventory.md`**

Update the rows for #234 (safe OpenGL draw calls) and #179 (rlPushMatrix family): note they are addressed by the WS5b immediate-mode rlgl module (matrix stack + immediate-mode verts + bind handles), with the GL-object lifecycle/full state surface intentionally left to existing RAII + raw `ffi`.

- [ ] **Step 2: Write `docs/superpowers/notes/ws5-complete.md`**

Summarize WS5: prep (harness normalization + the gen_image_* MSVC fix), WS5a (raygui broad rework — scratch-buffer `impl AsRef<str>` convention, module split, soundness fixes, 57/57 parity, #296 deferred), WS5b (immediate-mode rlgl). List the render tests (`render_shapes`, `render_text`, `render_gui`, `render_rlgl`) and the CI feature set. Note tracked-deferred items carried to WS6 (idiom/soundness PRs, the `custom_audio_stream_callback` deprecation warning, the raw `gui_get_icons`/`gui_load_icons` safe abstraction, `rl_mult_matrixf` if it was deferred).

- [ ] **Step 3: Full local verification (the WS5b done-gate)**

```
cargo fmt --all -- --check
cargo build -p raylib
cargo build -p raylib --features raygui
cargo clippy -p raylib -- -D warnings 2>&1 | grep -E "rlgl[/\\\\]|rgui[/\\\\]" ; echo "(empty = no rlgl/rgui clippy hits)"
cargo test -p raylib --doc
cargo test -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui --test render_shapes --test render_text --test render_gui --test render_rlgl -- --test-threads=1
```
Expected: all green.

- [ ] **Step 4: Commit, push, watch CI**

```bash
git add docs/superpowers/inventory.md docs/superpowers/notes/ws5-complete.md
git commit -m "docs(ws5): WS5 completion note; attribute #234/#179 to rlgl module

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
```
Then watch the run to green (make `gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status` the LAST command in its invocation):
```bash
gh run watch <run-id> -R Dacode45/ms-raylib-rs --exit-status
```

---

## Self-Review

**Spec coverage (§WS5b):**
- Matrix stack RAII + transforms/ortho/matrix-mode/set-matrix → Task 2.
- Immediate-mode vertex stream (begin/end guard + closure) → Task 3.
- Render-state toggles (backface culling, depth test) → Task 4.
- Bind-safe-handle methods (`&Texture2D`/`&Shader`) + textured doctest → Task 4.
- GL-object lifecycle left to existing RAII + raw ffi → stated in module doc (Task 1); no lifecycle wrappers added.
- Top-level `raylib/src/rlgl/` module → Task 1.
- Tier-2 render test → Task 5.
- #234/#179 attribution → Task 1 commit + Task 6.
- rlsw behavior note → module doc + the render test runs under `software_renderer`.

**Placeholders:** the two genuinely empirical/uncertain points have explicit decision procedures: the `Matrix` conversion accessors in Task 2 (use what compiles; defer `rl_mult_matrixf` if no `*const f32` accessor exists) and the triangle probe window in Task 5 (print + adjust, but must distinguish drew-vs-blank and red-vs-blue).

**Type consistency:** `DrawMode`/`MatrixMode` (`#[repr(i32)]`, from `ffi::RL_*`) defined in Task 1, used in Tasks 2–3, 5. `RlMatrix::new`/`RlImmediate::new` are `pub(crate)`, defined in Tasks 1–2/1–3, called from the `RaylibRlgl` entry points. The single `RaylibRlgl` trait accretes methods across Tasks 2–4; `RlImmediate` accretes its vertex methods in Task 3. `texture.id` / `shader.id`/`shader.locs` reached via the confirmed `Deref` to the ffi structs.

**Assumption to verify early (Task 2 Step 1):** the safe `Matrix` → `ffi::Matrix` / `[f32;16]` conversions. If absent, follow the Task 2 note (use `Into<ffi::Matrix>` for `rlSetMatrix*`; defer `rl_mult_matrixf`). Don't fabricate an accessor.
