# WS3a — Parity Checklist + Native Math-Type Adoption + `MintVec*` Sweep

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Stand up the authoritative raylib↔safe-binding **parity checklist** as the source of truth for what's left, then **adopt the WS2 native `raylib-sys` types as the safe crate's public math API** (`raylib::Vector2/3/4/Matrix/Quaternion` become the `raylib_sys` types), deprecate the `MintVec*` indirection, and make `glam`/`mint`/`serde` opt-in features that forward to `raylib-sys`.

**Architecture:** The safe crate currently aliases `Vector2 = glam::Vec2` and **hand-rolls** `Quaternion`/`Matrix` in `core/math.rs`. WS2 moved those types + their full raymath method set into `raylib-sys`. WS3a re-exports the sys types and **deletes the redundant hand-rolled Matrix/Quaternion**, reconciling the handful of internal method-name mismatches (safe code calls `.normalized()`/`from_vec3_pair`; sys exposes `.normalize()`/`from_vector3_to_vector3`). `MintVec2/3/4/MintMatrix/MintQuat` become `#[deprecated]` aliases (they are now *identical* to the native types — `MintVec2 == ffi::Vector2 == Vector2`); every **internal** `impl Into<MintVec2>` and bare `MintVec*` use is swept to the native name (169 lines across 9 files) so no deprecation warnings fire internally. The hard `glam` dep is dropped; `glam`/`mint`/`serde` become features forwarding to `raylib-sys/<feat>`.

**This plan does NOT make the safe crate compile.** The 37 captured FFI-change errors (model/font/draw/file/callback) are **WS3b's** worklist. WS3a's done-state is: parity checklist committed, and the safe crate's *remaining* compile errors are **exactly** the documented 37 FFI-change errors — i.e. the type flip + sweep + Cargo change introduce **no new error categories** (nothing referencing `Vector2`/`Matrix`/`Quaternion`/`glam`/`MintVec`).

**Tech Stack:** Rust 1.85 (edition 2024), `raylib-sys` 6.0 native types (`vector_math.rs`/`matrix_quat_math.rs`/`math.rs`), Python 3 (checklist tool). Branch `6.0-rc`; push to `fork` triggers the 3-OS CI.

**Reference:** spec D2 + §"math decoupling"; `docs/superpowers/notes/ws1-breakage-baseline.md` (the 37); `raylib/src/core/math.rs` (current aliases + hand-rolled Quaternion ~54–470 / Matrix ~451–960); `raylib/src/lib.rs:74-78` (`MintVec*` defs); `raylib-sys/src/{vector_math.rs,matrix_quat_math.rs,math.rs}` (the native methods to adopt); `find_unimplemented.py` + `checklist.md` (checklist seed); memory `skip-std-equivalent-fns`.

**Pre-flight:** `git rev-parse --abbrev-ref HEAD` → `6.0-rc`; `cargo build -p raylib-sys` green; `cargo test -p raylib-sys` green (WS2 math tests).

**Scope boundary (every task):** Edit only `raylib/src/core/math.rs`, `raylib/src/lib.rs`, the 9 sweep files, `raylib/Cargo.toml`, and the repo-root checklist tooling. Do **NOT** fix the 37 FFI-change errors here (WS3b). Do not touch `raylib-sys/**` (WS2 is done).

---

## File structure

| Path | Responsibility | Task |
|------|----------------|------|
| `find_unimplemented.py` | Extend → classify every `raylib.h` RLAPI fn as implemented / wont-impl(reason) / unimplemented; emit a real checklist + summary | 1 |
| `docs/superpowers/parity-checklist.md` | Generated authoritative checklist (source of truth for WS3b/c & future sessions) | 1 |
| `raylib/src/core/math.rs` | Re-export sys `Vector2/3/4/Matrix/Quaternion`; delete hand-rolled Matrix/Quaternion; keep `lerp`/`rquat`/`Transform` glue; reconcile internal method names | 2 |
| `raylib/src/lib.rs:74-78` | `MintVec*`/`MintMatrix`/`MintQuat` → `#[deprecated]` aliases of native types | 3 |
| `raylib/src/core/{drawing,models,shaders,texture,window,camera,math}.rs`, `raylib/src/rgui/safe.rs` | Sweep internal `MintVec*` → `Vector*`/`Matrix`/`Quaternion` (169 lines) | 3 |
| `raylib/Cargo.toml` | Drop hard `glam`; add optional `glam`/`mint`/`serde` features forwarding to `raylib-sys` | 4 |

---

## Task 1: Build the authoritative parity checklist (do this FIRST)

**Files:** Modify `find_unimplemented.py`; create `docs/superpowers/parity-checklist.md`.

Per memory `skip-std-equivalent-fns`: the checklist is the source of truth for what's left. The current script only *prints* unimplemented names and has a partial `wont_impl`. Extend it to classify **every** RLAPI function and emit a committed checklist.

- [ ] **Step 1: Read the current tool.** Read `find_unimplemented.py` (it scans `raylib-sys/raylib/src/raylib.h` for `RLAPI` lines, marks a fn implemented if `ffi::<Name>` appears in any `raylib/src/core/*` file, and has a `wont_impl` list already covering `Text*`, UTF-8 helpers, and file/path helpers).

- [ ] **Step 2: Extend `wont_impl` with explicit std-equivalent rationale.** Convert `wont_impl` from a flat list to a dict mapping name → reason. Keep every existing entry; ensure these std-equivalent families are present with reasons (per the locked "std-only line" decision — KEEP base64/compress/CRC/MD5/SHA1, they are NOT std):

```python
# Functions intentionally not wrapped: Rust std / core idioms do these as well or better.
# (Locked decision: std-only line — base64/DEFLATE/CRC/MD5/SHA1 are KEPT, they are not in std.)
wont_impl = {
    "SetTraceLogCallback": "implemented via C shim; not detected by the ffi:: scan",
    # UTF-8 — use Rust's native &str/String/char
    "GetCodepointNext": "Rust str/char iteration", "GetCodepointPrevious": "Rust str/char iteration",
    "CodepointToUTF8": "char::encode_utf8", "LoadUTF8": "String/str", "UnloadUTF8": "String/str",
    # Text — use Rust's str/String + format!/std
    "TextCopy": "String", "TextIsEqual": "str ==", "TextLength": "str::len",
    "TextFormat": "format!", "TextSubtext": "str slicing", "TextReplace": "str::replace",
    "TextInsert": "String::insert_str", "TextJoin": "[T]::join", "TextSplit": "str::split",
    "TextAppend": "String::push_str", "TextFindIndex": "str::find", "TextToUpper": "str::to_uppercase",
    "TextToLower": "str::to_lowercase", "TextToPascal": "Rust string ops", "TextToSnake": "Rust string ops",
    "TextToCamel": "Rust string ops", "TextToInteger": "str::parse", "TextToFloat": "str::parse",
    # Filesystem / paths — use std::fs / std::path
    "LoadFileData": "std::fs::read", "UnloadFileData": "Drop", "SaveFileData": "std::fs::write",
    "LoadFileText": "std::fs::read_to_string", "UnloadFileText": "Drop", "SaveFileText": "std::fs::write",
    "FileExists": "Path::exists", "DirectoryExists": "Path::is_dir",
    "GetFileExtension": "Path::extension", "GetFileName": "Path::file_name",
    "GetFileNameWithoutExt": "Path::file_stem", "GetDirectoryPath": "Path::parent",
    "GetPrevDirectoryPath": "Path ops", "GetWorkingDirectory": "std::env::current_dir",
    "MakeDirectory": "std::fs::create_dir_all", "ChangeDirectory": "std::env::set_current_dir",
    "IsFileNameValid": "Rust path validation", "GetFileModTime": "fs::Metadata::modified",
    "MemRealloc": "not needed",
}
```

- [ ] **Step 3: Make the tool emit a checklist file with status + summary.** Rewrite the bottom of `find_unimplemented.py` so that, in addition to (or instead of) printing, it writes `docs/superpowers/parity-checklist.md`. For each RLAPI fn, classify as: `[x]` implemented (`ffi::<Name>` found), `[~]` won't-implement (in `wont_impl`, with the reason inline), or `[ ]` unimplemented. Group by the comment-section headers already present in `raylib.h` (e.g. `// Window-related functions`). Emit a header summary line with counts. Sketch:

```python
def classify(func_name, src_files):
    if func_name in wont_impl:
        return ("~", wont_impl[func_name])
    for f in src_files:
        if "ffi::" + func_name in f:
            return ("x", None)
    return (" ", None)

# ...accumulate rows per section, then:
with open("docs/superpowers/parity-checklist.md", "w") as out:
    out.write("# raylib 6.0 ↔ safe-binding parity checklist\n\n")
    out.write(f"_Generated by find_unimplemented.py. {n_impl} implemented · "
              f"{n_skip} wont-impl (std) · {n_todo} TODO of {n_total} RLAPI fns._\n\n")
    out.write("Legend: [x] wrapped · [~] intentionally skipped (Rust std) · [ ] TODO\n\n")
    # ...write sections with `- [x] FuncName` / `- [~] FuncName — reason` / `- [ ] FuncName`
```

- [ ] **Step 4: Run it and sanity-check.**

Run: `python find_unimplemented.py` (from repo root).
Expected: `docs/superpowers/parity-checklist.md` created; summary counts printed; the 37-error symbols (e.g. `DrawModelPoints`, `UpdateModelAnimationBones`) show `[ ]` or absent (they'll be handled/removed in WS3b); the std families show `[~]`.

- [ ] **Step 5: Spot-verify against the worklist.** Open `docs/superpowers/parity-checklist.md`; confirm `DrawModelPoints`/`DrawModelPointsEx` appear as TODO (WS3b removes them), and that `Text*`/file helpers are `[~]` with reasons. This file is what WS3b/WS3c and future sessions consult.

- [ ] **Step 6: Commit.**
```bash
git add find_unimplemented.py docs/superpowers/parity-checklist.md
git commit -m "$(printf 'chore(ws3a): authoritative raylib 6.0 parity checklist\n\nExtend find_unimplemented.py to classify every RLAPI fn as\nimplemented / wont-impl(std-equivalent, with reason) / TODO and emit\ndocs/superpowers/parity-checklist.md as the source of truth for what\nremains. Locks the std-only skip line (base64/compress/hash KEPT).\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 2: Flip `core/math.rs` to re-export the native types

**Files:** Modify `raylib/src/core/math.rs`.

Replace the glam aliases + hand-rolled Matrix/Quaternion with re-exports of the WS2 sys types. Keep the safe-only glue: `lerp`, `rquat`, and the `Transform`/`Ray`/`BoundingBox`/`RayCollision` wrappers that live elsewhere are untouched here — this task is only the type sourcing in `math.rs`.

- [ ] **Step 1: Capture the baseline error fingerprint.** Run `cargo build -p raylib 2>&1 | rg "^error" | sort | uniq -c | tee /tmp/ws3a-baseline-errors.txt`. These are the pre-WS3a errors (the 37 FFI-change errors). Keep this file; Task 2–4 must not add error categories outside it.

- [ ] **Step 2: Replace the type-sourcing head of `math.rs`.** Delete the glam aliasing block (lines ~24–28: `pub use glam;` + `pub type Vector2/3/4 = glam::...` + the note) and the entire hand-rolled `Quaternion` struct+impls and `Matrix` struct+impls. Replace the head with re-exports of the native types (these now carry the full raymath method set + operators from WS2):

```rust
//! Game-related math: the public `Vector*`/`Matrix`/`Quaternion` types are the
//! native `raylib-sys` types (own their `#[repr(C)]` layout + raymath methods).
pub use crate::ffi::{Matrix, Quaternion, Vector2, Vector3, Vector4};

use crate::misc::AsF32;
```

Keep `lerp` and `rquat` (rewrite `rquat` to use `Quaternion::new`, which exists on the sys type). Keep any `Vector2/3/4` convenience constructors **only if** the sys type lacks them — the sys types already provide `::new`, `::zero`, `::one`, ops, `dot`, `cross`, `normalize`, `length`, `lerp`, etc. (see `raylib-sys/src/vector_math.rs`), so delete safe-crate duplicates rather than shadow them.

- [ ] **Step 3: Reconcile internal method-name mismatches.** Build and fix every error that references a math method the sys type names differently. Known mismatches (safe → sys): `.normalized()` → `.normalize()`; `.length_squared()` → `.length_sqr()`; `Quaternion::from_vec3_pair(a,b)` → `Quaternion::from_vector3_to_vector3(a,b)`; `mat.to_matrix()`/`from_matrix` already match. Find them:

Run: `rg -n "\.normalized\(\)|\.length_squared\(\)|from_vec3_pair" raylib/src`
For each hit, replace with the sys method name. (If a safe call has no sys equivalent, STOP and surface it — do not invent a method.)

- [ ] **Step 4: Verify no math/type/glam errors remain.** Run `cargo build -p raylib 2>&1 | rg "^error" | sort | uniq -c > /tmp/ws3a-after-task2.txt`; then `diff /tmp/ws3a-baseline-errors.txt /tmp/ws3a-after-task2.txt`. Expected: the error set is a **subset** of baseline (math/type errors gone or unchanged); **no NEW** errors mentioning `Vector`, `Matrix`, `Quaternion`, `glam`, or `E0599`/`E0308` on math types. If a new math error appears, fix it in this task before proceeding.

- [ ] **Step 5: `cargo fmt --all` and commit.**
```bash
git add raylib/src/core/math.rs
git commit -m "$(printf 'refactor(ws3a)!: adopt native raylib-sys math types in safe crate\n\nRe-export ffi::{Vector2,Vector3,Vector4,Matrix,Quaternion} as the public\nmath API; delete the hand-rolled safe Matrix/Quaternion (the sys types\nnow carry the raymath methods). Reconcile internal method names\n(.normalized->.normalize, from_vec3_pair->from_vector3_to_vector3).\nRetires glam from the safe public surface.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 3: Deprecate `MintVec*` and sweep internal call sites

**Files:** Modify `raylib/src/lib.rs:74-78`; sweep `raylib/src/core/{drawing,models,shaders,texture,window,camera,math}.rs` and `raylib/src/rgui/safe.rs`.

`MintVec2` is now *identical* to `Vector2`. Per the locked decision, keep the aliases as a one-release courtesy bridge but mark them `#[deprecated]`, and sweep **all internal uses** to the native names (so internal code emits no deprecation warnings — important for the WS6 `-Dwarnings` gate).

- [ ] **Step 1: Mark the aliases deprecated in `lib.rs`.** Replace lines 74–78:

```rust
/// Deprecated alias for [`Vector2`]; the public type is now the native `raylib-sys` type.
#[deprecated(since = "6.0.0", note = "use `Vector2` (the native type); `MintVec2` is now identical")]
pub type MintVec2 = ffi::Vector2;
/// Deprecated alias for [`Vector3`].
#[deprecated(since = "6.0.0", note = "use `Vector3`; `MintVec3` is now identical")]
pub type MintVec3 = ffi::Vector3;
/// Deprecated alias for [`Vector4`].
#[deprecated(since = "6.0.0", note = "use `Vector4`; `MintVec4` is now identical")]
pub type MintVec4 = ffi::Vector4;
/// Deprecated alias for [`Matrix`].
#[deprecated(since = "6.0.0", note = "use `Matrix`; `MintMatrix` is now identical")]
pub type MintMatrix = ffi::Matrix;
/// Deprecated alias for [`Quaternion`].
#[deprecated(since = "6.0.0", note = "use `Quaternion`; `MintQuat` is now identical")]
pub type MintQuat = ffi::Quaternion;
```

- [ ] **Step 2: Enumerate the sweep.** Run `rg -n "MintVec2|MintVec3|MintVec4|MintMatrix|MintQuat" raylib/src` and confirm ~169 lines across the 9 files (105 `MintVec2`, 57 `MintVec3`, 8 `MintMatrix`, 1 `MintVec4`, plus any `MintQuat`).

- [ ] **Step 3: Apply the mechanical rename in every file EXCEPT `lib.rs`.** Transformation rules (apply per file; `lib.rs` keeps the deprecated alias definitions):
  - `MintVec2` → `Vector2`
  - `MintVec3` → `Vector3`
  - `MintVec4` → `Vector4`
  - `MintMatrix` → `Matrix`
  - `MintQuat` → `Quaternion`

This covers both `impl Into<MintVec2>` → `impl Into<Vector2>` and any bare type uses. Do it file-by-file, e.g. per file:

Run (PowerShell, one file shown — repeat for each):
`(Get-Content raylib/src/core/drawing.rs) -replace 'MintVec2','Vector2' -replace 'MintVec3','Vector3' -replace 'MintVec4','Vector4' -replace 'MintMatrix','Matrix' -replace 'MintQuat','Quaternion' | Set-Content raylib/src/core/drawing.rs`

(Bash equivalent: `sed -i 's/MintVec2/Vector2/g; s/MintVec3/Vector3/g; s/MintVec4/Vector4/g; s/MintMatrix/Matrix/g; s/MintQuat/Quaternion/g' <file>`.) Files: `core/drawing.rs`, `core/models.rs`, `core/shaders.rs`, `core/texture.rs`, `core/window.rs`, `core/camera.rs`, `core/math.rs`, `rgui/safe.rs`.

- [ ] **Step 4: Fix the now-broken imports.** Many files `use crate::{MintVec2, ...}` or `use crate::core::math::Vector2`. After the rename, ensure each swept file imports the native `Vector2/3/4`/`Matrix`/`Quaternion` (from `crate::core::math::*` or `crate::ffi`). Run `cargo build -p raylib 2>&1 | rg "MintVec|MintMatrix|MintQuat|unresolved import"` → expect **no** Mint references and no new unresolved-import errors. Fix imports until clean.

- [ ] **Step 5: Verify the error fingerprint is still a subset of baseline.** Run `cargo build -p raylib 2>&1 | rg "^error" | sort | uniq -c > /tmp/ws3a-after-task3.txt`; `diff /tmp/ws3a-baseline-errors.txt /tmp/ws3a-after-task3.txt`. Expected: no new error categories beyond the 37 FFI-change errors. (`rg deprecated raylib/src` warning check: `cargo build -p raylib 2>&1 | rg "use of deprecated"` → expect **none** from internal code.)

- [ ] **Step 6: `cargo fmt --all` and commit.**
```bash
git add raylib/src/lib.rs raylib/src/core raylib/src/rgui/safe.rs
git commit -m "$(printf 'refactor(ws3a)!: deprecate MintVec* aliases; sweep to native types\n\nMintVec2/3/4, MintMatrix, MintQuat are now #[deprecated] aliases of the\nidentical native types and kept one release as a courtesy bridge. All\n169 internal call sites across 9 files swept to Vector2/3/4/Matrix/\nQuaternion so no deprecation warnings fire internally.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## Task 4: Drop hard `glam`; make `glam`/`mint`/`serde` opt-in

**Files:** Modify `raylib/Cargo.toml`.

The safe crate must pull zero math deps by default; `glam`/`mint`/`serde` forward to the `raylib-sys` adapters WS2b built.

- [ ] **Step 1: Remove the mandatory `glam` dependency.** In `[dependencies]`, delete the line `glam = { version = "0.30", features = ["mint"] }`. Add optional deps (the conversions live in `raylib-sys`; the safe crate only needs the feature toggles to forward):

```toml
[dependencies]
raylib-sys = { version = "5.7.0", path = "../raylib-sys", default-features = false }
serde = { version = "1.0.125", features = ["derive"], optional = true }
serde_json = { version = "1.0.64", optional = true }
glam = { version = "0.30", optional = true }
mint = { version = "0.5", optional = true }

thiserror = "2.0.12"
paste = "1.0"
seq-macro = "0.3.5"
```

- [ ] **Step 2: Wire the feature → sys-adapter forwarding.** In `[features]`, replace the `serde` line and add `glam`/`mint`:

```toml
serde = ["dep:serde", "dep:serde_json", "raylib-sys/serde"]
glam = ["dep:glam", "raylib-sys/glam"]
mint = ["dep:mint", "raylib-sys/mint"]
```

(Confirm against `raylib-sys/Cargo.toml` that `serde`/`glam`/`mint` features exist there from WS2b; if `serde_json` was not previously gated, keep prior behavior — only add `dep:serde_json` if the safe crate uses it under `serde`.)

- [ ] **Step 3: Default build pulls no math deps.** Run `cargo tree -p raylib --no-default-features -e normal 2>/dev/null | rg -i "glam|mint" ` → expect **empty**. Run `cargo tree -p raylib -e normal | rg -i "glam|mint"` → expect **empty** (default features no longer include them).

- [ ] **Step 4: Feature builds resolve (compile of deps, not the safe crate yet).** The safe crate still has the 37 errors, so full build fails — but the *dependency graph* must resolve. Run `cargo build -p raylib --features glam,mint,serde 2>&1 | rg "^error: (failed to select|feature)" ` → expect **empty** (no feature/resolution errors; only the known code errors remain).

- [ ] **Step 5: Final WS3a fingerprint check.** Run `cargo build -p raylib 2>&1 | rg "^error" | sort | uniq -c > /tmp/ws3a-final.txt`; `diff /tmp/ws3a-baseline-errors.txt /tmp/ws3a-final.txt`. **Done-gate for WS3a:** the remaining errors are exactly the 37 FFI-change errors (model/font/draw/file/callback/data) — nothing referencing math types, glam, mint, or MintVec. Record `/tmp/ws3a-final.txt` contents in the WS3a completion note for WS3b's handoff.

- [ ] **Step 6: Commit.**
```bash
git add raylib/Cargo.toml
git commit -m "$(printf 'build(ws3a)!: drop hard glam dep; glam/mint/serde now opt-in\n\nDefault safe-crate build pulls zero math/serialization deps. glam, mint,\nserde become features forwarding to raylib-sys adapters (WS2b). Verified\ncargo tree shows no glam/mint by default.\n\nCo-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>')"
```

---

## WS3a done criteria

- `docs/superpowers/parity-checklist.md` committed and regenerable via `python find_unimplemented.py`.
- `raylib::Vector2/3/4/Matrix/Quaternion` ARE the native `raylib-sys` types; hand-rolled safe Matrix/Quaternion deleted.
- `MintVec*` are `#[deprecated]` aliases; zero internal Mint references; no internal deprecation warnings.
- `cargo tree -p raylib` (default) shows no `glam`/`mint`; `--features glam,mint,serde` resolves.
- `cargo build -p raylib` fails with **only** the documented 37 FFI-change errors (handed to WS3b). `raylib-sys` still builds + tests green.
