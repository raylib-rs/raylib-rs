# Wrapper-Soundness Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the unsound `DerefMut`/`AsMut<ffi::T>` surface from wrappers whose safe APIs trust raw fields (issue #276), replacing it with an `unsafe` raw accessor + targeted safe setters — per the audit classification, with `// SOUNDNESS:` comments at every macro call site.

**Architecture:** New `AsRawMut<T>` trait (`unsafe fn as_raw_mut`) emitted by a new `readonly` macro mode that keeps `Deref`/`AsRef` but drops `DerefMut`/`AsMut`. The six accessor traits bounded on `AsMut<ffi::T>` over unsound types rebind to `AsRawMut<ffi::T>`; their default methods wrap raw access in `unsafe` blocks whose invariants they themselves uphold. Three new safe setters on `RaylibMaterial` absorb ~52 of the ~53 external write sites.

**Tech Stack:** Rust 1.88, macro_rules, compile_fail doctests, nextest + software_renderer Tier-2.

**Spec:** `docs/superpowers/specs/2026-06-06-wrapper-soundness-design.md` (classification table + audit evidence)
**Branch:** `feat/wrapper-soundness` (off `unstable`, spec committed). PR base: `unstable`, resolves #276, credit @AmityWilder (#277).

**Environment notes:**
- Windows host, PowerShell. `cargo fmt -p <crate>` before commits (not `--all` in worktrees). Stage files explicitly, never `git add -A`.
- If executing in the main checkout (current setup), branch is already checked out. If a subagent runs a task: verify `git branch --show-current` = `feat/wrapper-soundness` before committing (memory: subagent-worktree-cwd-trap).
- CI commands verbatim (memory: plan-clippy-vs-ci-command-divergence). The five clippy legs: see Task 7.
- Expect Tasks 2–3 to be compiler-driven: Task 2 intentionally breaks compilation; Task 3 fixes ALL of it by the stated rules before anything is committed (Tasks 2+3 land as one commit).

---

### Task 1: `AsRawMut` trait + `readonly` macro mode + rslice mut-half removal

**Files:**
- Modify: `raylib/src/core/macros.rs`
- Modify: `raylib/src/core/mod.rs` (re-export)

- [ ] **Step 1: Add the `AsRawMut` trait at the top of `macros.rs`** (before `make_thin_wrapper!`):

```rust
/// Escape-hatch mutable access to a wrapper's raw FFI value.
///
/// Implemented by every thin wrapper. For wrappers classified *sound*
/// (see the per-type `// SOUNDNESS:` comments at the `make_thin_wrapper!`
/// call sites) this duplicates `AsMut`; for *readonly* wrappers it is the
/// only mutable access to the raw struct — and it is `unsafe`, because
/// the wrapper's safe API trusts invariants of the raw fields (issue #276).
pub trait AsRawMut<T> {
    /// Mutable access to the wrapped raw FFI value.
    ///
    /// # Safety
    ///
    /// Callers must uphold the wrapper's invariants:
    /// - do **not** corrupt count/dimension/format fields that safe
    ///   accessors use to derive slice lengths or buffer sizes (e.g.
    ///   `Mesh::vertexCount`, `Image::width`/`height`/`format`), and
    /// - do **not** reassign pointer fields that are owned and freed by
    ///   `Drop` (e.g. `Model::meshes`, `Font::glyphs`), unless you take
    ///   over ownership of the previous allocation.
    unsafe fn as_raw_mut(&mut self) -> &mut T;
}
```

Note: `macros.rs` holds `macro_rules!` only today — if it is included via `#[macro_use] mod macros;` the trait can still live there, but it must be re-exported. Check `raylib/src/core/mod.rs` for how `macros` is declared; add `pub use macros::AsRawMut;` (or define the trait directly in `core/mod.rs` if `macros` is `#[macro_use]`-only without a normal module path — in that case put the trait in `core/mod.rs` right after the module declarations and adjust the macro bodies to name it as `crate::core::AsRawMut`). Also re-export it from `prelude` (find `raylib/src/prelude.rs` and add it alongside the other trait exports) so trait-method calls resolve for users.

- [ ] **Step 2: Add the `readonly` arm to `make_thin_wrapper!`**

After the existing `true` arm, add:

```rust
    ($(#[$attrs:meta])* $name:ident, $t:ty, $dropfunc:expr, readonly) => {
        $(#[$attrs])*
        #[repr(transparent)]
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name(pub(crate) $t);

        impl_wrapper!($name, $t, $dropfunc, 0);
        readonly_deref_impl_wrapper!($name, $t, $dropfunc, 0);
        gen_from_raw_wrapper!($name, $t, $dropfunc, 0);
    };
```

And the matching arm in `make_thin_wrapper_lifetime!` after its `true` arm:

```rust
    ($(#[$attrs:meta])* $name:ident, $t1:ty, $t2:ty, $dropfunc:expr, readonly) => {
        $(#[$attrs])*
        #[derive(Debug)]
        #[allow(missing_docs)]
        pub struct $name<'a>(pub(crate) $t1, &'a $t2);

        impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
        readonly_deref_impl_wrapper!($name<'a>, $t1, $dropfunc, 0);
    };
```

- [ ] **Step 3: Add `readonly_deref_impl_wrapper!`** (next to `deref_impl_wrapper!`):

```rust
/// Internal helper. Read half of the deref family only (`Deref` +
/// `AsRef`), plus the `unsafe` [`AsRawMut`] escape hatch. Used by the
/// `readonly` wrapper mode for types whose safe API trusts raw fields
/// (issue #276) — see the `// SOUNDNESS:` comment at each call site.
macro_rules! readonly_deref_impl_wrapper {
    ($name:ident$(<$lifetime:tt>)?, $t:ty, $dropfunc:expr, $rawfield:tt) => {
        impl$(<$lifetime>)? std::convert::AsRef<$t> for $name$(<$lifetime>)? {
            fn as_ref(&self) -> &$t {
                &self.$rawfield
            }
        }

        impl$(<$lifetime>)? std::ops::Deref for $name$(<$lifetime>)? {
            type Target = $t;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$rawfield
            }
        }

        impl$(<$lifetime>)? crate::core::AsRawMut<$t> for $name$(<$lifetime>)? {
            unsafe fn as_raw_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }
    };
}
```

- [ ] **Step 4: Emit `AsRawMut` from the full mode too** (uniformity — sound types get it as a redundant-but-harmless alias). Append to `deref_impl_wrapper!`'s expansion:

```rust
        impl$(<$lifetime>)? crate::core::AsRawMut<$t> for $name$(<$lifetime>)? {
            unsafe fn as_raw_mut(&mut self) -> &mut $t {
                &mut self.$rawfield
            }
        }
```

- [ ] **Step 5: rslice mut-half removal.** In `impl_rslice!`, delete the `AsMut<$t>` and `DerefMut` impls and add a safe element accessor:

```rust
        impl $name {
            /// Mutable access to the elements. The `Box` itself is not
            /// exposed (replacing it would free the raylib-owned buffer
            /// through the global allocator — see issue #276).
            #[inline]
            pub fn as_mut_slice(&mut self) -> &mut [Color] {
                &mut self.$rawfield
            }
        }
```

NOTE: `$t` in `impl_rslice!` is `Box<[Color]>`; `&mut self.0` where `self.0: ManuallyDrop<Box<[Color]>>` derefs to `&mut [Color]` via auto-deref — if inference complains, use `&mut self.$rawfield[..]`. The element type is always `Color` for both current rslices; if a future rslice uses another element type, generalize to `&mut [$elem]` by threading the element type through `make_rslice!` (it already receives it as `$t` there — pass it down).

- [ ] **Step 6: Compile check** — Run: `cargo check -p raylib`
Expected: errors are acceptable ONLY if they come from `ImageColors`/`ImagePalette` users of the removed mut surface (audit says zero) — i.e. expected: PASS. The `readonly` arms are not yet used by any call site.

- [ ] **Step 7: Commit**

```powershell
cargo fmt -p raylib
git add raylib/src/core/macros.rs raylib/src/core/mod.rs raylib/src/prelude.rs
git commit -m @'
feat(raylib): AsRawMut trait + readonly wrapper mode (issue #276 groundwork)

readonly emits Deref/AsRef + an unsafe as_raw_mut escape hatch instead
of the full deref family; rslices drop the Box-exposing mut half (which
allowed mem::take -> double free) in favor of a safe as_mut_slice.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 2: Reclassify every call site with `// SOUNDNESS:` comments

**Files:** `raylib/src/core/{models,texture,text,shaders,file,audio,vr,automation}.rs`

For each row, change the macro's trailing mode and add the `// SOUNDNESS:` comment line **directly above the macro invocation**. Sites without a trailing arg currently default to `true` — readonly sites must gain an explicit `, readonly`, sound sites stay as-is (add `, true` only if you must touch the line anyway; otherwise leave). The classification and the comments (verbatim):

| Site | New mode | `// SOUNDNESS:` comment |
|---|---|---|
| `models.rs:23` `Model` | `readonly` | `// SOUNDNESS: readonly — P1 (meshes/materials/bones slices trust meshCount/materialCount/boneCount) + P2 (Drop=UnloadModel frees meshes/materials/skeleton arrays).` |
| `models.rs:59` `WeakModel` | `readonly` | `// SOUNDNESS: readonly — P1 (same RaylibModel slice accessors); no_drop removes P2.` |
| `models.rs:60` `Mesh` | `readonly` | `// SOUNDNESS: readonly — P1 (vertices/normals/etc. slices trust vertexCount/triangleCount) + P2 (Drop=UnloadMesh frees every pointer field).` |
| `models.rs:92` `WeakMesh` | `readonly` | `// SOUNDNESS: readonly — P1 (same RaylibMesh slice accessors); no_drop removes P2.` |
| `models.rs:105` `WeakMaterial` | `readonly` | `// SOUNDNESS: readonly — P1 (maps slice trusts the maps pointer); no_drop removes P2.` |
| `models.rs:106` `Material` | `readonly` | `// SOUNDNESS: readonly — P1 (maps slice trusts the maps pointer) + P2 (Drop=UnloadMaterial frees maps).` |
| `models.rs:112` (`BoneInfo`) | keep `true` | `// SOUNDNESS: full deref — inline name[32]/parent only; no trusted counts, no owned pointers.` |
| `models.rs:118` `WeakModelAnimation` | `readonly` | `// SOUNDNESS: readonly — P1 (keyframe accessors trust framePoses/frameCount); no_drop removes P2.` |
| `models.rs:119` `ModelAnimation` | `readonly` | `// SOUNDNESS: readonly — P1 (keyframe accessors trust framePoses/frameCount); freed by the owning ModelAnimations, not this wrapper.` |
| `texture.rs:61` (`Image`) | `readonly` | `// SOUNDNESS: readonly — P1 (pixel readers trust width/height/format to size data) + P2 (Drop=UnloadImage frees data).` |
| `texture.rs:92` (`Texture2D`) | keep `true` | `// SOUNDNESS: full deref — inline id/width/height/mipmaps/format; corrupting them confuses raylib C-side only, no Rust-side UB.` |
| `texture.rs:119` `WeakTexture2D` | keep `true` | `// SOUNDNESS: full deref — same inline-only fields as Texture2D.` |
| `texture.rs:126` (`RenderTexture2D`) | keep `true` | `// SOUNDNESS: full deref — id + two inline Texture structs; no trusted counts, no owned CPU pointers.` |
| `texture.rs:161` `WeakRenderTexture2D` | keep `true` | `// SOUNDNESS: full deref — same inline-only fields as RenderTexture2D.` |
| `text.rs:16` (`Font`) | `readonly` | `// SOUNDNESS: readonly — P1 (chars slice trusts glyphs×glyphCount) + P2 (Drop=UnloadFont frees recs/glyphs).` |
| `text.rs:53` `WeakFont` | `readonly` | `// SOUNDNESS: readonly — P1 (same RaylibFont accessors); no_drop removes P2.` |
| `text.rs:54` (`GlyphInfo`) | keep `true` | `// SOUNDNESS: full deref — inline scalars + inline Image value; this wrapper neither sizes nor frees the image data.` |
| `shaders.rs:12` (`Shader`) | `readonly` | `// SOUNDNESS: readonly — P1 (locs slice trusts the locs pointer) + P2 (Drop=UnloadShader frees locs).` |
| `shaders.rs:54` `WeakShader` | `readonly` | `// SOUNDNESS: readonly — P1 (same locs accessor); no_drop removes P2.` |
| `file.rs:142` `FilePathList` | `readonly` | `// SOUNDNESS: readonly — P1 (paths slice trusts paths×count) + P2 (Drop=UnloadDirectoryFiles frees paths and each string).` |
| `file.rs:143` `DroppedFilePathList` | `readonly` | `// SOUNDNESS: readonly — P1/P2 like FilePathList (Drop=UnloadDroppedFiles).` |
| `audio.rs:13` (`Wave`) | `readonly` | `// SOUNDNESS: readonly — P2 (Drop=UnloadWave frees data); no Rust-side slice trusts frameCount.` |
| `audio.rs:27` (`Sound`) | `readonly` | `// SOUNDNESS: readonly — P2 (Drop=UnloadSound frees stream.buffer/processor). Closes the gap #277 originally targeted.` |
| `audio.rs:54` (`Music`) | `readonly` | `// SOUNDNESS: readonly — P2 (Drop=UnloadMusicStream frees stream + ctxData).` |
| `audio.rs:69` (`AudioStream`) | `readonly` | `// SOUNDNESS: readonly — P2 (Drop=UnloadAudioStream frees buffer/processor).` |
| `vr.rs:5` (`VrStereoConfig`) | keep `true` | `// SOUNDNESS: full deref — all fields inline Matrix/f32 arrays; nothing to invalid-free, no trusted counts.` |
| `automation.rs:94/136` | keep `false` | `// SOUNDNESS: opted out (false) — hand-written accessor surface; nothing raw leaks.` |
| `texture.rs:14/15` rslices | (Task 1 handled) | `// SOUNDNESS: rslice — Box mut-half removed (mem::take would double-free); element access via as_mut_slice.` |

(For wrappers where the macro spans multiple lines with doc attrs, the mode token goes where `true`/`false` would: after the drop fn. For `MaterialMap` — if it exists as a separate call site inside models.rs not listed above, classify `true` with the inline-fields comment; the audit found it SOUND.)

- [ ] **Step 1:** Apply the table.
- [ ] **Step 2:** Run `cargo check -p raylib 2>&1 | Select-String 'error\[' | Measure-Object -Line` — expect a sizeable error count (trait bounds + internal `as_mut()` uses). This is the intentionally-broken midpoint. **Do not commit** — Task 3 completes the change.

---

### Task 3: Trait-bound migration + internal sweep (completes Task 2's commit)

**Files:** `raylib/src/core/{models,text,shaders,texture,databuf}.rs` (+ any file the compiler flags)

- [ ] **Step 1: Rebind the six unsound-type traits.** Pattern (worked example, `models.rs:982`):

```rust
// before
pub trait RaylibMaterial: AsRef<ffi::Material> + AsMut<ffi::Material> {
// after
pub trait RaylibMaterial: AsRef<ffi::Material> + crate::core::AsRawMut<ffi::Material> {
```

Apply identically to: `RaylibModel` (models.rs:384), `RaylibMesh` (models.rs:608), `RaylibModelAnimation` (models.rs:1283), `RaylibFont` (text.rs:370), `RaylibShader` (shaders.rs:396). **Leave `RaylibTexture2D` and `RaylibRenderTexture2D` alone** (sound types keep `AsMut`).

- [ ] **Step 2: Convert default-method bodies.** Rule for every `self.as_mut()` inside the six rebound traits (and any inherent impl on a readonly wrapper):
  - If already inside an `unsafe` block/fn: change to `self.as_raw_mut()` and extend the existing `// SAFETY:` comment (or add one) with: `raw mut access: this method upholds the wrapper invariants itself (no count/pointer corruption).`
  - If in safe code (e.g. `Model::set_transform` at models.rs:413): use direct field access `self.0` for inherent impls on the concrete wrapper, or wrap in `unsafe { self.as_raw_mut() }` with the SAFETY comment for trait default methods. Worked examples:

```rust
// shader_mut (models.rs:992) — already unsafe-transmute body:
fn shader_mut(&mut self) -> &mut crate::shaders::WeakShader {
    // SAFETY: transmuting the inline shader field; raw mut access cannot
    // corrupt maps (the only trusted pointer) — invariants upheld.
    unsafe { std::mem::transmute(&mut self.as_raw_mut().shader) }
}

// set_material_texture (models.rs:1020):
fn set_material_texture(
    &mut self,
    map_type: crate::consts::MaterialMapIndex,
    texture: impl AsRef<ffi::Texture2D>,
) {
    // SAFETY: SetMaterialTexture writes one map's texture handle; counts
    // and owned pointers are untouched.
    unsafe {
        ffi::SetMaterialTexture(self.as_raw_mut(), (map_type as u32) as i32, *texture.as_ref())
    }
}

// Model::set_transform (models.rs:413) — inherent on the trait; keep safe:
fn set_transform(&mut self, mat: &Matrix) {
    // SAFETY: transform is an inline Matrix — not a trusted count or owned pointer.
    unsafe { self.as_raw_mut().transform = (*mat).into(); }
}
```

(Adjust the exact bodies to the existing code — the *rule* is fixed: every raw-mut touch sits in `unsafe` with a SAFETY line asserting which invariant class it cannot violate.)

- [ ] **Step 3: Compiler-driven sweep to green.** Iterate `cargo check -p raylib` and apply the Step-2 rule to every remaining error (31 known `self.as_mut()` sites across models/text/texture/shaders/databuf; plus any `&mut *wrapper` deref-mut uses the compiler flags). HARD RULE: never restore `AsMut`/`DerefMut` on a readonly type; never add `allow` attributes; if a site genuinely cannot satisfy the rule, STOP and report BLOCKED.
Run: `cargo check -p raylib` → expected PASS.
Run: `cargo nextest run -p raylib` → expected PASS (audit: no test write-sites).

- [ ] **Step 4: Commit (Tasks 2+3 together — the tree is only green now):**

```powershell
cargo fmt -p raylib
git add raylib/src/core/models.rs raylib/src/core/texture.rs raylib/src/core/text.rs raylib/src/core/shaders.rs raylib/src/core/file.rs raylib/src/core/audio.rs raylib/src/core/vr.rs raylib/src/core/automation.rs raylib/src/core/databuf.rs
git commit -m @'
fix(raylib)!: remove safe &mut ffi access where wrapper invariants forbid it

Resolves #276; supersedes PR #277 (analysis by AmityWilder). Per-type
audit: wrappers whose safe accessors trust count/dim/pointer fields
(P1) or whose Drop frees pointer fields (P2) move to the readonly mode
(Deref/AsRef + unsafe as_raw_mut); inline-data/GPU-id wrappers keep the
full deref family. Every macro call site carries a SOUNDNESS comment
recording its classification. The six accessor traits over unsound
types rebind AsMut -> AsRawMut; their methods uphold the invariants
inside documented unsafe blocks.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 4: `RaylibMaterial` safe setters + tests

**Files:**
- Modify: `raylib/src/core/models.rs` (inside `trait RaylibMaterial`)
- Test: `raylib/tests/integration_models.rs` (or a new `raylib/tests/material_setters.rs` following the existing Tier-2 `with_headless` pattern — copy the harness usage from `raylib/tests/render_shapes.rs`)

- [ ] **Step 1: Add the setters** (after `maps_mut` in the trait):

```rust
    /// Replace the material's shader.
    ///
    /// Copies the shader's inline FFI struct into the material — the same
    /// semantics as raylib C's `material.shader = shader`. The [`Shader`]
    /// wrapper keeps ownership of the shader (and frees it on drop), so it
    /// must outlive every use of this material.
    ///
    /// [`Shader`]: crate::shaders::Shader
    #[inline]
    fn set_shader(&mut self, shader: impl AsRef<ffi::Shader>) {
        // SAFETY: shader is an inline value field — writing it cannot
        // corrupt the maps pointer (the only field the safe API trusts).
        unsafe {
            self.as_raw_mut().shader = *shader.as_ref();
        }
    }

    /// Set one material map's color (e.g. albedo tint).
    ///
    /// # Panics
    /// Never — `index` is an enum bounded by `MAX_MATERIAL_MAPS`.
    #[inline]
    fn set_map_color(&mut self, index: crate::consts::MaterialMapIndex, color: impl Into<ffi::Color>) {
        self.maps_mut()[index as usize].color = color.into();
    }

    /// Set one material map's scalar value (e.g. metalness/roughness).
    #[inline]
    fn set_map_value(&mut self, index: crate::consts::MaterialMapIndex, value: f32) {
        self.maps_mut()[index as usize].value = value;
    }
```

(`MaterialMap` keeps full deref — sound — so field assignment through `maps_mut()`'s `&mut [MaterialMap]` compiles. If `color` field types disagree (`Color` wrapper vs `ffi::Color`), adapt with the crate's existing conversion — both are `repr(C)`-identical; check how neighboring code assigns map colors.)

- [ ] **Step 2: Tier-2 test** (software_renderer; follow the `with_headless` pattern used by existing Tier-2 tests — check `raylib/src/test_harness.rs` usage in `render_shapes.rs`):

```rust
//! Tier-2: Material safe-setter round-trips (issue #276 follow-up).
#![cfg(feature = "software_renderer")]

// (copy the exact harness imports/setup from raylib/tests/render_shapes.rs)

#[test]
fn material_setters_roundtrip() {
    with_headless(|rl, thread| {
        let mut mat = rl.load_material_default(thread);
        let shader_before = mat.shader().id;

        mat.set_map_color(
            raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO,
            raylib::prelude::Color::RED,
        );
        mat.set_map_value(raylib::consts::MaterialMapIndex::MATERIAL_MAP_METALNESS, 0.5);

        let maps = mat.maps();
        assert_eq!(maps[raylib::consts::MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize].color.r, 255);
        assert_eq!(maps[raylib::consts::MaterialMapIndex::MATERIAL_MAP_METALNESS as usize].value, 0.5);
        // set_shader with the material's own current shader is identity-safe:
        let weak = *mat.shader();
        mat.set_shader(&weak);
        assert_eq!(mat.shader().id, shader_before);
    });
}
```

(Adapt names — `load_material_default`'s exact receiver/signature, `with_headless`'s shape, and field paths must match the real harness; the assertions are the contract. If `set_shader(&weak)` trips a type mismatch, use the form the trait accepts: anything `AsRef<ffi::Shader>` — `WeakShader` implements it.)

- [ ] **Step 3: Run** the Tier-2 command verbatim:
`cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_MESH_GENERATION -E 'binary(material_setters)'`
Expected: PASS (and confirm the test RAN, not filtered out — pitfall: feature-gated silent skip).

- [ ] **Step 4: Commit**

```powershell
cargo fmt -p raylib
git add raylib/src/core/models.rs raylib/tests/material_setters.rs
git commit -m @'
feat(raylib): RaylibMaterial::set_shader / set_map_color / set_map_value

Safe setters for the legitimate material mutations the readonly mode
removed: shader assignment copies the inline FFI struct (documented
aliasing: the Shader wrapper keeps ownership), map color/value write
through the bounds-safe maps_mut slice. Tier-2 round-trip test.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

---

### Task 5: Showcase conversion sweep

**Files:** the ~30 showcase example files listed in the audit (see spec; canonical list below).

Conversion rules:
1. `<material>.as_mut().shader = *<shader>` / `<material>.shader = …` / `materials_mut()[i].as_mut().shader = …` → `materials_mut()[i].set_shader(&<shader>)` (or `.set_shader(<shader>.as_ref())` where a `&ffi::Shader` is in hand). Sites: models_loading_vox.rs:201, models_skybox_rendering.rs:79, models_animation_blend_custom.rs:355, models_animation_gpu_skinning.rs:68, shaders_basic_pbr.rs:256/324/603/607, shaders_cel_shading.rs:145/203/205/263/268/270/327, shaders_deferred_rendering.rs:286/287/630/634, shaders_fog_rendering.rs:202-204/302/306/310, shaders_mesh_instancing.rs:184/270, shaders_normalmap_rendering.rs:103, shaders_shadowmap_rendering.rs:204/210/367/373, shaders_simple_mask.rs:132/133, shaders_texture_tiling.rs:84, shaders_vertex_displacement.rs:86/138.
2. `unsafe { (*mat.maps.offset(i)).color = c }`-style raw map writes → `mat.set_map_color(<index enum>, c)` / `.set_map_value(<index>, v)`, dropping the `unsafe` block if nothing else needs it. Sites: shaders_basic_pbr.rs:261/299/328/362/615/636, shaders_custom_uniform.rs:65/169, shaders_fog_rendering.rs:142/147/152/318, shaders_model_shader.rs:75/133, shaders_normalmap_rendering.rs:261, shaders_simple_mask.rs:213, shaders_texture_tiling.rs:140.
3. The mesh pointer install (shaders_lightmap_rendering.rs:69): keep the existing `unsafe` block, change the access to `mesh.as_raw_mut().texcoords2 = …` (needs `use raylib::core::AsRawMut;` or the prelude import).
4. RenderTexture2D `target.texture.width = …` writes (shaders_depth_rendering.rs:64-73, shaders_depth_writing.rs:65+): NO change — RenderTexture2D keeps DerefMut.
5. Raw `ffi::Mesh` construction (models_mesh_generation.rs etc.): NO change — they mutate the raw struct before wrapping.
6. Line numbers are from the 2026-06-06 audit on `unstable` — verify each with the compiler: the authoritative worklist is whatever `cargo clippy -p raylib-showcase --examples -- -D warnings` flags after Tasks 1–4.

**Style note (memory: showcase-c-rust-port-style):** ports mirror the C example line-for-line; keep the conversion minimal — same line position, same surrounding comments. `set_shader(&shader)` replacing `materials[0].shader = shader` is in the C spirit.

- [ ] **Step 1:** Apply rules until both showcase clippy legs are green (verbatim):
`cargo clippy -p raylib-showcase --examples -- -D warnings`
`cargo clippy -p raylib-showcase --examples --features SUPPORT_CUSTOM_FRAME_CONTROL -- -D warnings`
Expected: PASS, both. (Remember the second leg compiles an extra example.)

- [ ] **Step 2:** `cargo nextest run -p raylib-showcase` → PASS.

- [ ] **Step 3: Commit**

```powershell
cargo fmt -p raylib-showcase
git add showcase/examples
git commit -m @'
refactor(showcase): material writes via the new safe setters

set_shader / set_map_color / set_map_value replace as_mut() field
writes and raw maps.offset() pokes; the one genuine raw-pointer install
(lightmap texcoords2) uses the unsafe as_raw_mut escape hatch.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

(`git add showcase/examples` adds a directory of tracked modifications — acceptable here as the change is scoped to exactly that tree; verify with `git status --porcelain` that nothing unexpected is staged.)

---

### Task 6: compile_fail doctests, book sweep, CHANGELOG

**Files:**
- Modify: `raylib/src/core/models.rs` (Mesh call-site doc attr), `raylib/src/core/audio.rs` (Sound doc attr)
- Modify: `book/src/**` (grep-driven), `CHANGELOG.md`

- [ ] **Step 1: compile_fail doctests.** Add to the `Mesh` macro call site's doc comment (models.rs:60, inside the existing `///` block):

```rust
    /// Mutable access to the raw FFI struct is `unsafe` — corrupting the
    /// counts the slice accessors trust is no longer possible in safe code:
    ///
    /// ```compile_fail
    /// # use raylib::prelude::*;
    /// fn corrupt(mesh: &mut Mesh) {
    ///     mesh.vertexCount += 5; // no DerefMut on Mesh (issue #276)
    /// }
    /// ```
```

And to `Sound`'s doc block (audio.rs:27):

```rust
    /// ```compile_fail
    /// # use raylib::prelude::*;
    /// fn corrupt(sound: &mut Sound) {
    ///     sound.stream.buffer = std::ptr::null_mut(); // no DerefMut on Sound (issue #276)
    /// }
    /// ```
```

(If `Sound<'_>`'s lifetime makes the signature awkward, `fn corrupt(sound: &mut Sound<'_>)`.)
Run: `cargo test -p raylib --doc --features full` → expected PASS with the two compile_fail tests listed as passing.

- [ ] **Step 2: Book sweep.** `Grep -r "as_mut" book/src` — for each hit on a readonly type, rewrite the snippet using the new setters or `unsafe as_raw_mut` with a sentence explaining why it's unsafe. If zero hits: state that in the commit message.

- [ ] **Step 3: CHANGELOG** — under `## Unreleased`, add to `### Breaking` (create the subsection if absent, before `### Changed`):

```markdown
### Breaking

- **Wrapper soundness (resolves [#276](https://github.com/raylib-rs/raylib-rs/issues/276), supersedes #277 by @AmityWilder):**
  `DerefMut`/`AsMut<ffi::T>` are removed from wrappers whose safe API trusts
  raw fields — `Image`, `Mesh`, `Model`, `Material`, `Font`, `Shader`,
  `ModelAnimation`, their `Weak*` variants, `FilePathList`,
  `DroppedFilePathList`, `Wave`, `Sound`, `Music`, `AudioStream` (read access
  via `Deref`/`AsRef` is unchanged). Raw mutation is now
  `unsafe fn as_raw_mut()` (new `AsRawMut` trait, in the prelude). New safe
  setters cover the common cases: `RaylibMaterial::set_shader`,
  `set_map_color`, `set_map_value`. `ImageColors`/`ImagePalette` no longer
  expose `&mut Box<[Color]>` (use `as_mut_slice`). `Texture2D`,
  `RenderTexture2D`, `VrStereoConfig` and other inline-data wrappers keep the
  full deref family — every classification is documented as a `// SOUNDNESS:`
  comment at its `make_thin_wrapper!` call site.
```

- [ ] **Step 4: Commit**

```powershell
cargo fmt -p raylib
git add raylib/src/core/models.rs raylib/src/core/audio.rs CHANGELOG.md
git commit -m @'
docs: compile_fail proofs + CHANGELOG for wrapper soundness

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
'@
```

(Add any changed book files to the same commit.)

---

### Task 7: Full gates, push, PR

- [ ] **Step 1: The five clippy legs + tests + doctests (verbatim):**

```powershell
cargo clippy -p raylib --lib --bins --features full -- -D warnings
cargo clippy -p raylib-sys --features full -- -D warnings
cargo clippy -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,SUPPORT_MESH_GENERATION,raygui -- -D warnings
cargo clippy -p raylib-showcase --examples -- -D warnings
cargo clippy -p raylib-showcase --examples --features SUPPORT_CUSTOM_FRAME_CONTROL -- -D warnings
cargo nextest run -p raylib -p raylib-sys -p raylib-showcase
cargo test -p raylib --doc --features full
cargo fmt --all --check
```

All expected: exit 0.

- [ ] **Step 2: Push + PR.** Write the body to `$env:TEMP\wrapper-soundness-pr.md` (mention: resolves #276, supersedes #277 with credit to @AmityWilder, the P1/P2 rule, the classification table pointer to the spec, the setters, breaking-change framing "rides 6.0.0 final"), then:

```powershell
git push -u origin feat/wrapper-soundness
gh pr create --base unstable --title "fix(raylib)!: wrapper soundness - remove safe &mut ffi access (resolves #276)" --body-file "$env:TEMP\wrapper-soundness-pr.md"
gh pr checks --watch
```

Expected: all 28 checks green.

- [ ] **Step 3:** Done-note `docs/superpowers/notes/wrapper-soundness-complete.md`: the classification table as-landed, the trait-rebind pattern, fallout counts, the stale-WS6-Sound-claim correction, and any new future-work seeded. Commit to the PR branch before merge or as a follow-up docs commit.

---

## Done criteria (from the spec)

- [ ] Readonly wrappers expose no safe `&mut ffi::T` (compile_fail-proven)
- [ ] Every macro call site has a `// SOUNDNESS:` comment
- [ ] All five clippy legs + tests + doctests green; CI 28/28
- [ ] CHANGELOG Breaking entry; #276 closeable; @AmityWilder credited
