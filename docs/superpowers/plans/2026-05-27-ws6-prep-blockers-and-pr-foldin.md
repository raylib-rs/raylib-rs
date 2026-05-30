# WS6-prep — Clear `-Dwarnings` blockers + fold in deferred quality PRs — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the safe crate clippy-`-Dwarnings`-clean and land the tracked-deferred quality PRs (idiom + soundness) with attribution, so the WS6a gate plan can flip the gates on a settled API surface.

**Architecture:** Two mechanical blocker fixes in `raylib-sys/build.rs` and `raylib/src/core/callbacks.rs`, then three small idiom PRs cherry-picked as-is, then a soundness pass that *adapts* (not blindly cherry-picks) PRs whose 5.x targets were largely rewritten by WS3 — "already covered" is an acceptable per-PR outcome, recorded with reasoning.

**Tech Stack:** Rust (edition 2024, MSRV 1.85), `cargo`, `cargo clippy`, `gh` CLI (against `raylib-rs/raylib-rs` for diffs), the WS4 `software_renderer` render-test harness.

**Spec:** `docs/superpowers/specs/2026-05-27-ws6-platform-cicd-design.md` §3 (WS6-prep), §2 (W6-4).

**Working model:** Branch `6.0-rc`. Build/test with the safe crate's curated set. Commits end with `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>`; cherry-picked PRs also credit the original author with a second `Co-Authored-By` trailer (GitHub handle → `<handle>@users.noreply.github.com`).

---

## File structure

| File | Responsibility | Tasks |
|------|----------------|-------|
| `raylib-sys/build.rs` | Rename `Platform`/`PlatformOS` acronym variants (clippy blocker) | 1 |
| `raylib/src/core/callbacks.rs` | Remove dead deprecated audio-stream trampoline + method (warning blocker) | 2 |
| `raylib/src/core/{audio,file}.rs` | PR #272 — `c"..."` literals | 3 |
| `raylib-sys/src/color.rs`, `raylib/src/core/{camera,texture,vr}.rs` | PR #268 — `Into`→`From` | 4 |
| `raylib/src/core/audio.rs` | PR #266 — seal `AudioSample` | 5 |
| `raylib/src/core/{models,texture,...}.rs` | PR #277 — remove unsound wrapper trait impls (adapt) | 6 |
| `raylib/src/core/models.rs` | Mesh-accessor soundness pass (#257/#118/#256, adapt) | 7 |
| `docs/superpowers/notes/ws6-prep-complete.md` | Completion note + per-PR disposition | 8 |

**Reference commands used throughout:**
- Safe-crate build: `cargo build -p raylib`
- Safe-crate clippy (the gate target): `cargo clippy -p raylib --all-targets --features SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO -- -D warnings` (the `full` alias does not exist yet — that lands in WS6a; until then lint the default + module set).
- Render tests (must stay green): the four Tier-2 commands from `.github/workflows/baseline.yml` `software-render` job.

---

## Task 1: Rename `build.rs` acronym enum variants (clippy `upper_case_acronyms`)

**Files:**
- Modify: `raylib-sys/build.rs` (enum defs at ~606-622; match arms at ~200-201, 291-292, 439, 451, 455, 571-575, 579)

`clippy::upper_case_acronyms` flags `Platform::DRM`, `Platform::RPI`, `PlatformOS::BSD`, `PlatformOS::OSX`. Rename to `Drm`, `Rpi`, `Bsd`, `Osx` (the idiomatic fix; `build.rs` is not public API). Feature names (`drm`, `legacy_rpi`) are unaffected — only the Rust enum variants change.

- [ ] **Step 1: Establish the blocker exists**

Run: `cargo clippy -p raylib-sys 2>&1 | rg "upper_case_acronyms" -n`
Expected: warnings naming `DRM`, `RPI`, `BSD`, `OSX`.

- [ ] **Step 2: Rename the enum definitions**

In `raylib-sys/build.rs`, change:
```rust
enum Platform {
    Web,
    Desktop,
    Android,
    Drm,
    Rpi,    // legacy raspberry pi
    Memory, // raylib 6.0 windowless software-render platform (PLATFORM=Memory)
}
```
and
```rust
enum PlatformOS {
    Windows,
    Linux,
    Bsd,
    Osx,
    Unknown,
}
```

- [ ] **Step 3: Update every match arm / reference**

Replace these exact occurrences (use Edit per site; there are eight):
- `Platform::DRM => conf.define("PLATFORM", "DRM"),` → `Platform::Drm => conf.define("PLATFORM", "DRM"),` (the **string** `"DRM"` stays — it's the cmake `-DPLATFORM` value)
- `Platform::RPI => conf.define("PLATFORM", "Raspberry Pi"),` → `Platform::Rpi => ...` (string unchanged)
- `Platform::DRM => "-DPLATFORM_DRM",` → `Platform::Drm => "-DPLATFORM_DRM",`
- `Platform::RPI => "-DPLATFORM_RPI",` → `Platform::Rpi => "-DPLATFORM_RPI",`
- `PlatformOS::OSX => {` → `PlatformOS::Osx => {`
- `} else if platform == Platform::DRM {` → `... == Platform::Drm {`
- `} else if platform == Platform::RPI {` → `... == Platform::Rpi {`
- `"FreeBSD" => PlatformOS::BSD,` / `"OpenBSD" => PlatformOS::BSD,` / `"NetBSD" => PlatformOS::BSD,` / `"DragonFly" => PlatformOS::BSD,` → `... => PlatformOS::Bsd,` (four arms)
- `"Darwin" => PlatformOS::OSX,` → `"Darwin" => PlatformOS::Osx,`
- `} else if cfg!(feature = "drm") {\n        Platform::DRM` → `Platform::Drm`
- `} else if cfg!(feature = "legacy_rpi") {\n        Platform::RPI` → `Platform::Rpi`
- `} else if matches!(platform, Platform::DRM | Platform::RPI | Platform::Android) {` → `matches!(platform, Platform::Drm | Platform::Rpi | Platform::Android)`

- [ ] **Step 4: Verify the rename compiles and clippy is clean on these**

Run: `cargo clippy -p raylib-sys 2>&1 | rg "upper_case_acronyms" -n; echo "exit=$?"`
Expected: no `upper_case_acronyms` lines (the `rg` exit is 1 = no matches, which is what we want).
Run: `cargo build -p raylib-sys`
Expected: builds (no `cannot find variant` errors — proves all arms were updated).

- [ ] **Step 5: Commit**

```bash
git add raylib-sys/build.rs
git commit -m "fix(ws6): rename build.rs Platform acronym variants (clippy upper_case_acronyms)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: Remove the dead deprecated audio-stream callback (self-deprecation warning)

**Files:**
- Modify: `raylib/src/core/callbacks.rs` (remove fn at 130-136; remove method at 353-367)

`custom_audio_stream_callback` (130-136) is `#[deprecated]` yet invoked at 362 inside the deprecated `RaylibHandle::set_audio_stream_callback`, producing a self-deprecation warning. The live replacement is the generic `set_audio_stream_callback` in `core/callbacks/audio_stream_callback.rs`. Verified callers (`raylib-test/src/callbacks.rs`, `samples/audio_raw_stream_callback.rs`) use the **free** functions, not this deprecated method — removal is safe. (The other deprecated `RaylibHandle` methods at 309-351 do not warn and are left as-is; full finalization of the `callbacks.rs:130` TODO is out of WS6-prep scope.)

- [ ] **Step 1: Confirm the warning is present**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RAUDIO 2>&1 | rg "custom_audio_stream_callback|deprecated" -n`
Expected: a `use of deprecated function ... custom_audio_stream_callback` warning.

- [ ] **Step 2: Confirm no caller uses the deprecated method**

Run: `rg -n "\.set_audio_stream_callback\(" --glob '!raylib/src/core/callbacks.rs'`
Expected: no hits in `raylib-test`/`samples`/`showcase` for the **method** form on a handle (the new generic free fn `set_audio_stream_callback::<...>` is a different symbol and is fine).

- [ ] **Step 3: Delete the deprecated trampoline fn**

Remove these lines from `callbacks.rs` (including the stale TODO):
```rust
//TODO: before any merge find out what the status of the deprecated functions are and how to best finalize removing them
#[deprecated = "use [set_audio_stream_callback](core::callbacks::audio_stream::set_audio_stream_callback) and its trampoline instead."]
extern "C" fn custom_audio_stream_callback(a: *mut c_void, b: u32) {
    let audio_stream = audio_stream_callback().unwrap();
    let a = unsafe { std::slice::from_raw_parts(a as *mut u8, b as usize) };
    audio_stream(a);
}
```

- [ ] **Step 4: Delete the deprecated method that used it**

Remove the `set_audio_stream_callback` method block (353-367):
```rust
    /// Audio thread callback to request new data
    #[deprecated = "Decoupled from RaylibHandle. Use [set_audio_stream_callback](core::callbacks::set_audio_stream_callback) instead."]
    pub fn set_audio_stream_callback(
        &'_ mut self,
        stream: AudioStream,
        cb: fn(&[u8]),
    ) -> Result<(), SetLogError<'_>> {
        if AUDIO_STREAM_CALLBACK.load(Ordering::Acquire) == 0 {
            AUDIO_STREAM_CALLBACK.store(cb as _, Ordering::Release);
            unsafe { ffi::SetAudioStreamCallback(stream.0, Some(custom_audio_stream_callback)) }
            Ok(())
        } else {
            Err(SetLogError("audio stream"))
        }
    }
```

- [ ] **Step 5: Check for now-unused items**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RAUDIO 2>&1 | rg "warning|error" -n`
Expected: no warnings/errors. If `audio_stream_callback()` (the accessor) or `AUDIO_STREAM_CALLBACK` is now unused, confirm whether the live module still uses it (`rg -n "AUDIO_STREAM_CALLBACK|fn audio_stream_callback" raylib/src/core/callbacks*`); only remove genuinely-dead items, and keep anything the live `audio_stream_callback.rs` references.

- [ ] **Step 6: Verify build clean + render tests green**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RAUDIO`
Expected: clean (the targeted deprecation warning is gone).

- [ ] **Step 7: Commit**

```bash
git add raylib/src/core/callbacks.rs
git commit -m "refactor(ws6): remove dead deprecated custom_audio_stream_callback

The trampoline was #[deprecated] yet invoked internally, tripping clippy
-Dwarnings. The live path is the generic set_audio_stream_callback in
core/callbacks/audio_stream_callback. No in-workspace caller used the
removed RaylibHandle method.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: PR #272 — `c"..."` literals (idiom, cherry-pick as-is)

**Files:**
- Modify: `raylib/src/core/audio.rs`, `raylib/src/core/file.rs`

Replaces `CStr::from_bytes_with_nul(b"...\0").unwrap()` (and similar) with C-string literals `c"..."`. Mechanical, no logic change. Author: **AmityWilder**.

- [ ] **Step 1: Read the PR diff to see the exact sites**

Run: `gh pr diff 272 -R raylib-rs/raylib-rs`
Note each `from_bytes_with_nul`/`from_bytes_until_nul` literal it converts.

- [ ] **Step 2: Apply the conversions in the current files**

For each site the PR identifies, replace the verbose form with the `c"..."` literal. Example shape:
```rust
// before
let s = CStr::from_bytes_with_nul(b"some text\0").unwrap();
// after
let s = c"some text";
```
The PR targets 5.x; apply the same conversion wherever the equivalent literal exists in the current `audio.rs`/`file.rs`. If a site no longer exists, skip it (record in the completion note).

- [ ] **Step 3: Verify**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RAUDIO && cargo test -p raylib --doc`
Expected: builds; doctests pass.

- [ ] **Step 4: Commit (with attribution)**

```bash
git add raylib/src/core/audio.rs raylib/src/core/file.rs
git commit -m "refactor(ws6): use c\"...\" literals over CStr::from_bytes_with_nul (#272)

Cherry-picked from raylib-rs#272.

Co-Authored-By: AmityWilder <AmityWilder@users.noreply.github.com>
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: PR #268 — `Into<T> for U` → `From<U> for T` (idiom)

**Files:**
- Modify: `raylib-sys/src/color.rs`, `raylib/src/core/camera.rs`, `raylib/src/core/texture.rs`, `raylib/src/core/vr.rs`

Idiomatic-Rust convention fix: implementing `From` gives the `Into` for free; the reverse is discouraged (`clippy::from_over_into`). Author: **AmityWilder**.

- [ ] **Step 1: Read the PR diff**

Run: `gh pr diff 268 -R raylib-rs/raylib-rs`

- [ ] **Step 2: Locate current `impl Into<...> for ...` blocks**

Run: `rg -n "impl +Into<" raylib-sys/src/color.rs raylib/src/core/camera.rs raylib/src/core/texture.rs raylib/src/core/vr.rs`

- [ ] **Step 3: Convert each to `From`**

For each, rewrite (the WS2 native-type move may have changed the concrete types — match the current ones):
```rust
// before
impl Into<ffi::Color> for Color {
    fn into(self) -> ffi::Color { ffi::Color { r: self.r, g: self.g, b: self.b, a: self.a } }
}
// after
impl From<Color> for ffi::Color {
    fn from(c: Color) -> ffi::Color { ffi::Color { r: c.r, g: c.g, b: c.b, a: c.a } }
}
```
Keep any call sites that relied on `.into()` working — they still compile (`From` supplies `Into`).

- [ ] **Step 4: Verify**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RTEXTURES && cargo clippy -p raylib-sys 2>&1 | rg "from_over_into" -n; echo done`
Expected: builds; no `from_over_into` warnings remain.

- [ ] **Step 5: Commit (with attribution)**

```bash
git add raylib-sys/src/color.rs raylib/src/core/camera.rs raylib/src/core/texture.rs raylib/src/core/vr.rs
git commit -m "refactor(ws6): impl From over Into (#268, closes #267)

Cherry-picked from raylib-rs#268.

Co-Authored-By: AmityWilder <AmityWilder@users.noreply.github.com>
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 5: PR #266 — Seal `AudioSample` (API hygiene)

**Files:**
- Modify: `raylib/src/core/audio.rs`

Seals the `AudioSample` trait so downstream crates can't implement it (closes #213). Author: **AmityWilder**.

- [ ] **Step 1: Read the PR diff**

Run: `gh pr diff 266 -R raylib-rs/raylib-rs`

- [ ] **Step 2: Apply the sealed-trait pattern to the current `AudioSample`**

Add a private sealing supertrait and bound `AudioSample` on it (match the current `AudioSample` definition in `audio.rs`):
```rust
mod private {
    pub trait Sealed {}
}
/// ... existing AudioSample docs ...
pub trait AudioSample: private::Sealed {
    // ... existing items ...
}
// for each concrete impl, also:
impl private::Sealed for i16 {}
impl private::Sealed for f32 {}
impl private::Sealed for u8 {}
```
Use exactly the set of types the current crate implements `AudioSample` for (find them: `rg -n "impl AudioSample for" raylib/src/core/audio.rs`).

- [ ] **Step 3: Verify**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RAUDIO && cargo test -p raylib --doc`
Expected: builds; doctests pass.

- [ ] **Step 4: Commit (with attribution)**

```bash
git add raylib/src/core/audio.rs
git commit -m "refactor(ws6): seal AudioSample trait (#266, closes #213)

Cherry-picked from raylib-rs#266.

Co-Authored-By: AmityWilder <AmityWilder@users.noreply.github.com>
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 6: PR #277 — Remove unsound wrapper trait impls (ADAPT, may be mostly-covered)

**Files:**
- Investigate: `raylib/src/core/{models,texture,shaders,file,drawing,text,automation,misc,collision}.rs`, `raylib/src/core/macros.rs`, `raylib/src/rgui/safe.rs`

PR #277 removes `AsRef`/`AsMut`/`Deref` impls on thin wrappers that hold raw pointers (these hand out references with unbounded lifetimes — closes #276). **The WS3 rewrite already redesigned these wrappers**; an initial grep shows `models.rs` only uses `impl AsRef<...>` as *parameter bounds*, not as unsound trait impls. So this is an audit: find any remaining unsound `impl AsRef/AsMut/Deref for <Weak*/thin wrapper>` and remove it; if none remain, record "already covered by WS3". Author: **AmityWilder**.

- [ ] **Step 1: Read the PR to understand exactly which impls it removed and why**

Run: `gh pr diff 277 -R raylib-rs/raylib-rs | rg -n "^-.*impl (AsRef|AsMut|Deref|DerefMut)" `
This lists the unsound impls the PR deleted (the `-` lines).

- [ ] **Step 2: Search the current tree for surviving instances of those impls**

Run: `rg -n "impl +(AsRef|AsMut|Deref|DerefMut)<[^>]*> +for +(Weak|[A-Z][A-Za-z]*)" raylib/src/core raylib/src/rgui`
For each hit, decide: is it a thin wrapper over a raw-pointer-bearing FFI struct where handing out `&T`/`&mut T` could outlive the backing allocation? (The PR's reasoning in its description is the rubric.)

- [ ] **Step 3: Remove each surviving unsound impl; adjust call sites**

For each unsound impl found, delete it and replace internal uses with an explicit accessor method or `&self.0` field access (whatever the current code needs to keep compiling). If a removed impl was load-bearing for ergonomics, add a named method (e.g. `fn as_ffi(&self) -> &ffi::X`) instead of the blanket trait. If **none** survive, make no code change.

- [ ] **Step 4: Verify**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RTEXT && cargo test -p raylib`
Expected: builds; tests pass.

- [ ] **Step 5: Commit (with attribution) or record "already covered"**

If changes were made:
```bash
git add raylib/src/core raylib/src/rgui
git commit -m "fix(ws6): drop remaining unsound wrapper trait impls (#277, closes #276)

Adapted from raylib-rs#277 onto the WS3-rewritten surface.

Co-Authored-By: AmityWilder <AmityWilder@users.noreply.github.com>
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```
If nothing remained: add a line to the completion note (Task 8) — "#277: already covered by the WS3 wrapper redesign (no unsound AsRef/AsMut/Deref impls remain; verified by grep on <date>)" — and make no commit.

---

## Task 7: Mesh-accessor soundness pass — #257 + #118 + #256 (ADAPT, one coherent pass)

**Files:**
- Investigate/modify: `raylib/src/core/models.rs` (primary); possibly `raylib/src/core/error.rs`, `raylib/src/core/texture.rs`
- Test: add a unit test under `raylib/src/core/models.rs` (`#[cfg(test)]`) or `raylib/tests/`

Three overlapping mesh PRs are applied as one pass (spec §3):
- **#257** — `RaylibMesh` accessors form slices from possibly-null pointers without checking (closes #262). Author: **AmityWilder**.
- **#118** — add a safe `texcoords` accessor for `Mesh`. Author: **strizhkindenis**.
- **#256** — broader mesh safety/soundness for 3D (large; targets 6.0 per author). Author: **meisei4**.

All three target the pre-6.0 `Mesh`. The goal is **the soundness intent on the current 6.0 `Mesh`**: every pointer-to-slice accessor must null-check (return `&[]` or `Option<&[T]>` rather than constructing a slice from a null `data` pointer), and texcoords must be exposed safely.

- [ ] **Step 1: Read all three diffs to extract the soundness intent**

Run: `gh pr diff 257 -R raylib-rs/raylib-rs` ; `gh pr diff 118 -R raylib-rs/raylib-rs` ; `gh pr diff 256 -R raylib-rs/raylib-rs`
Capture: which accessors they null-guard, the chosen return convention (slice-vs-Option), and the texcoords accessor signature.

- [ ] **Step 2: Inventory the current `Mesh` accessors**

Run: `rg -n "from_raw_parts|\.vertices|\.texcoords|\.normals|\.indices|\.colors|fn (vertices|normals|texcoords|indices|tangents|colors)" raylib/src/core/models.rs`
List every accessor that builds a slice from a raw `Mesh` field pointer.

- [ ] **Step 3: Write a failing test for the null-pointer case**

Add to `models.rs` test module (a `Mesh` with a null field should yield an empty slice, not UB):
```rust
#[cfg(test)]
mod mesh_soundness {
    use super::*;
    #[test]
    fn null_field_accessors_are_empty_not_ub() {
        // SAFETY: zeroed Mesh has null data pointers and zero counts — the
        // accessors must treat null+0 as an empty slice rather than calling
        // slice::from_raw_parts on a null pointer (UB).
        let mesh: ffi::Mesh = unsafe { std::mem::zeroed() };
        let m = WeakMesh::from_raw(mesh); // use the current constructor for a borrowed/weak Mesh
        assert!(m.vertices().is_empty());
        assert!(m.texcoords().is_empty());
        assert!(m.normals().is_empty());
        std::mem::forget(m); // do not run Drop on a zeroed/non-owning Mesh
    }
}
```
Adjust `WeakMesh::from_raw`/the borrowed-Mesh constructor name to whatever the current code provides (find it: `rg -n "struct (Weak)?Mesh|impl .*Mesh" raylib/src/core/models.rs`). If no borrowing constructor exists, construct via the existing public path the tests already use.

- [ ] **Step 4: Run the test — expect failure (or a miri/UB risk)**

Run: `cargo test -p raylib --features SUPPORT_MODULE_RMODELS mesh_soundness -- --nocapture`
Expected: FAIL (panic / non-empty / segfault) if accessors don't null-check; this proves the hole.

- [ ] **Step 5: Make each accessor null-safe**

For every slice accessor, guard the pointer (apply the #257/#256 pattern to the current types):
```rust
pub fn vertices(&self) -> &[Vector3] {
    let m = self.as_ffi(); // or &self.0
    if m.vertices.is_null() || m.vertexCount == 0 {
        return &[];
    }
    // SAFETY: vertices points to vertexCount * 3 f32 (== vertexCount Vector3),
    // allocated by raylib and valid for the lifetime of this Mesh borrow.
    unsafe { std::slice::from_raw_parts(m.vertices as *const Vector3, m.vertexCount as usize) }
}
```
Add the safe `texcoords` accessor (#118) the same way (`m.texcoords` → `&[Vector2]`, length `vertexCount`). Mirror for `normals`, `indices` (`&[u16]`, length `triangleCount * 3`), `colors`, `tangents` as the PRs cover them.

- [ ] **Step 6: Run the test — expect pass**

Run: `cargo test -p raylib --features SUPPORT_MODULE_RMODELS mesh_soundness`
Expected: PASS.

- [ ] **Step 7: Full models build + existing tests + render tests**

Run: `cargo build -p raylib --features SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RTEXT && cargo test -p raylib --features SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RAUDIO`
Expected: green.

- [ ] **Step 8: Commit (with multi-author attribution)**

```bash
git add raylib/src/core/models.rs
git commit -m "fix(ws6): null-safe Mesh accessors + safe texcoords (#257/#118/#256)

Adapted the mesh soundness intent of raylib-rs#257, #118, and #256 onto
the WS3-rewritten 6.0 Mesh: pointer-to-slice accessors null-guard, and
texcoords are exposed safely. Closes #262.

Co-Authored-By: AmityWilder <AmityWilder@users.noreply.github.com>
Co-Authored-By: strizhkindenis <strizhkindenis@users.noreply.github.com>
Co-Authored-By: meisei4 <meisei4@users.noreply.github.com>
Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 8: WS6-prep verification + completion note

**Files:**
- Create: `docs/superpowers/notes/ws6-prep-complete.md`

- [ ] **Step 1: Confirm the safe crate is clippy-clean**

Run: `cargo clippy -p raylib --all-targets --features SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO -- -D warnings`
Expected: **exit 0, no warnings.** (This is the WS6a gate target minus the `full` alias, which doesn't exist yet.)

- [ ] **Step 2: Confirm the four Tier-2 render tests still pass**

Run the four `software-render` commands from `.github/workflows/baseline.yml` (render_shapes/render_text, render_gui, render_rlgl).
Expected: all green.

- [ ] **Step 3: Write the completion note**

Record per-PR disposition (applied / adapted / already-covered, with the reason), the blocker fixes, and the resulting clippy-clean state. Link the spec.

- [ ] **Step 4: Commit + push to fork to confirm CI**

```bash
git add docs/superpowers/notes/ws6-prep-complete.md
git commit -m "docs(ws6): WS6-prep completion note (blockers cleared, deferred PRs folded in)

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
git push fork 6.0-rc
```
Then watch the existing `baseline.yml` run (it must stay green):
`gh run watch <id> -R Dacode45/ms-raylib-rs --exit-status`

---

## Self-review notes

- **Spec coverage:** §3 WS6-prep blockers → Tasks 1-2; idiom PRs #272/#268/#266 → Tasks 3-5; soundness PRs #277/#257/#256/#118 → Tasks 6-7; "API surface settled before doc pass" → the whole plan precedes WS6a. ✓
- **Adapt-not-cherry-pick:** Tasks 6-7 explicitly target the WS3-rewritten surface and allow "already covered" outcomes — matching the spec's re-validation requirement. ✓
- **Attribution:** every PR fold-in commit carries the original author's `Co-Authored-By` plus Claude's. ✓
- **Type consistency:** accessor names (`vertices`/`texcoords`/`normals`/`indices`) used in Task 7 match the `Mesh` field set; the `full` alias is explicitly deferred to WS6a (this plan lints against the module set instead). ✓
