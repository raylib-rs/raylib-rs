# thiserror Migration Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the last hand-rolled error type (`SetLogError`) with a thiserror struct and complete the `RaylibError` aggregate (8 missing `#[from]` variants + `#[non_exhaustive]`).

**Architecture:** Two breaking-but-pre-final API changes in the safe crate: (1) `core/callbacks.rs`'s `SetLogError<'a>` newtype + manual `Display`/`Error` impls become a `SetCallbackError(&'static str)` thiserror struct living in `core/error.rs` with the other error types; (2) `RaylibError` gains `#[from]` variants for the 6 error.rs leaf enums it's missing plus `PixelColorError` (from `core/pixel.rs`) and the new `SetCallbackError`, and becomes `#[non_exhaustive]`. Tests are Tier-1 (window-independent) unit tests inside `error.rs`'s existing `#[cfg(test)]` pattern.

**Tech Stack:** Rust 1.85 / edition 2024, `thiserror` 2.x (already a dependency), cargo-nextest.

**Spec:** `docs/superpowers/specs/2026-06-03-thiserror-migration-completion-design.md`
**Branch:** `chore/thiserror-migration` (already created off `origin/unstable` @ `1462ea1`)

---

## File structure

- Modify: `raylib/src/core/error.rs` — add `SetCallbackError` (+ tests), add 8 `RaylibError` variants + `#[non_exhaustive]`, update `RaylibError` doc example.
- Modify: `raylib/src/core/callbacks.rs` — delete `SetLogError` + impls; update macro, 5 free fns, 5 deprecated `RaylibHandle` methods, and the doc example.
- Modify: `CHANGELOG.md` — two entries under the existing `### Breaking` section of the `6.0.0-rc.2` block (that block becomes `6.0.0` final per its header note).

House style note for every new doc comment: each new variant/type gets a summary line, a **Cause:** paragraph, and a **Recovery:** paragraph, matching the existing entries in `error.rs` (see e.g. `AudioInitError::DoubleInit` at the top of the file). `deny(missing_docs)` is crate-wide — undocumented public items fail the build.

---

## Task 1: `SetCallbackError` replaces `SetLogError`

**Files:**
- Modify: `raylib/src/core/error.rs` (new type after `LoadStyleFromMemoryError`, ~line 1165; new test mod at the bottom)
- Modify: `raylib/src/core/callbacks.rs:141-194` (type + macro + free fns) and `:500-545` (RaylibHandle methods)

- [ ] **Step 1: Write the failing Display test**

At the bottom of `raylib/src/core/error.rs` (after the existing `load_icons_error_tests` mod):

```rust
#[cfg(test)]
mod set_callback_error_tests {
    use super::*;

    #[test]
    fn display_names_the_occupied_slot() {
        let e = SetCallbackError("save file data");
        assert_eq!(
            e.to_string(),
            "there is a save file data callback already set"
        );
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run (from repo root): `cargo nextest run -p raylib -E 'test(set_callback_error)'`
Expected: compile error — `cannot find ... SetCallbackError` (the type doesn't exist yet).

- [ ] **Step 3: Add `SetCallbackError` to `error.rs`**

Insert after `LoadStyleFromMemoryError`'s closing brace (~line 1165), before the `#[cfg(test)]` mods:

```rust
/// Error returned when installing a process-global callback whose slot is already occupied.
///
/// Each callback slot in [`crate::core::callbacks`] ([`set_save_file_data_callback`],
/// [`set_load_file_data_callback`], [`set_save_file_text_callback`],
/// [`set_load_file_text_callback`]) holds at most one function at a time. Calling a setter
/// while its slot is occupied returns this error rather than silently overwriting. The inner
/// `&'static str` names which callback kind was already set (e.g. `"save file data"`).
///
/// **Cause:** A previous call to the same setter installed a callback that has not been
/// removed.
///
/// **Recovery:** Keep a single registration site per callback kind, or remove the existing
/// callback (where an unset function exists) before installing a new one.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
/// use raylib::core::callbacks::set_save_file_data_callback;
///
/// fn writer(_path: &str, _bytes: &[u8]) -> bool { true }
/// set_save_file_data_callback(writer).expect("first install");
/// match set_save_file_data_callback(writer) {
///     Err(e) => eprintln!("{e}"), // "there is a save file data callback already set"
///     Ok(()) => unreachable!(),
/// }
/// ```
///
/// [`set_save_file_data_callback`]: crate::core::callbacks::set_save_file_data_callback
/// [`set_load_file_data_callback`]: crate::core::callbacks::set_load_file_data_callback
/// [`set_save_file_text_callback`]: crate::core::callbacks::set_save_file_text_callback
/// [`set_load_file_text_callback`]: crate::core::callbacks::set_load_file_text_callback
#[derive(Error, Debug)]
#[error("there is a {0} callback already set")]
pub struct SetCallbackError(pub(crate) &'static str);
```

Note: the message is lowercase without a trailing period (Rust error-message convention);
the old hand-rolled message was `"There is a {} callback already set."` — the doc comment in
callbacks.rs quoting it is updated in Step 4.

- [ ] **Step 4: Migrate `callbacks.rs`**

(a) Delete lines ~141–172: the `SetLogError` doc comment, `pub struct SetLogError<'a>(&'a str);`, the `impl std::fmt::Display`, and the `impl std::error::Error`. (Its doc example moves to `SetCallbackError` in Step 3, already done.)

(b) Add the import near the top of the file alongside the existing `use` items:

```rust
use crate::core::error::SetCallbackError;
```

(c) In `safe_callback_set_func!` change the error construction:

```rust
            Err(SetCallbackError($ty))
```

(d) Update the 5 free-fn signatures (drop the lifetime params — they exist only for the old error type):

```rust
pub fn set_trace_log_callback(cb: fn(TraceLogLevel, &str)) -> Result<(), SetCallbackError> {
```
```rust
pub fn set_save_file_data_callback(cb: fn(&str, &[u8]) -> bool) -> Result<(), SetCallbackError> {
```
```rust
pub fn set_load_file_data_callback(cb: fn(&str) -> Vec<u8>) -> Result<(), SetCallbackError> {
```
```rust
pub fn set_save_file_text_callback(cb: fn(&str, &str) -> bool) -> Result<(), SetCallbackError> {
```
```rust
pub fn set_load_file_text_callback(cb: fn(&str) -> String) -> Result<(), SetCallbackError> {
```

(`set_trace_log_callback` keeps its `Result` return for API uniformity even though its body
always returns `Ok` — spec decision; do not change its body.)

(e) Update the 5 deprecated `RaylibHandle` methods (~lines 500–545): each
`Result<(), SetLogError<'_>>` becomes `Result<(), SetCallbackError>`. The `&'_ mut self`
receivers stay as they are. Example for the first; apply identically to all five:

```rust
    pub fn set_trace_log_callback(
        &'_ mut self,
        cb: fn(TraceLogLevel, &str),
    ) -> Result<(), SetCallbackError> {
        set_trace_log_callback(cb)
    }
```

(f) Search the file for any remaining references: `grep -n "SetLogError" raylib/src/core/callbacks.rs` must return nothing.

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo nextest run -p raylib -E 'test(set_callback_error)'`
Expected: PASS (1 test).

- [ ] **Step 6: Verify the crate still compiles + no stragglers**

Run: `cargo check -p raylib` → clean.
Run: `grep -rn "SetLogError" raylib/src` → no matches.
Run: `grep -rnE "impl std::error::Error for|impl std::fmt::Display for" raylib/src` → no matches (definition-of-done check #1).

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add raylib/src/core/error.rs raylib/src/core/callbacks.rs
git commit -m "refactor(error): replace hand-rolled SetLogError with thiserror SetCallbackError

The last manual Display/Error impl in the safe crate. The artificial
lifetime parameter on the callback setters goes away with it.
BREAKING (pre-6.0.0-final): public type renamed, no alias.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 2: Complete the `RaylibError` aggregate

**Files:**
- Modify: `raylib/src/core/error.rs:919-1055` (doc example + enum) and the test mods at the bottom.

- [ ] **Step 1: Write the failing conversion tests**

Add at the bottom of `error.rs`:

```rust
#[cfg(test)]
mod raylib_error_from_tests {
    use super::*;
    use crate::consts::PixelFormat;
    use crate::core::pixel::PixelColorError;

    #[test]
    fn all_new_leaf_errors_convert_via_from() {
        assert!(matches!(
            RaylibError::from(UpdateAudioStreamError::CallbackSlotBusy),
            RaylibError::UpdateAudioStream(_)
        ));
        assert!(matches!(
            RaylibError::from(InvalidMeshError::TrianglePointMiscount),
            RaylibError::InvalidMesh(_)
        ));
        assert!(matches!(
            RaylibError::from(GenMeshError::InvalidMesh(
                InvalidMeshError::TrianglePointMiscount
            )),
            RaylibError::GenMesh(_)
        ));
        assert!(matches!(
            RaylibError::from(Base64Error::DecodeFailed),
            RaylibError::Base64(_)
        ));
        assert!(matches!(
            RaylibError::from(LoadIconsError::HeaderTruncated(3)),
            RaylibError::LoadIcons(_)
        ));
        assert!(matches!(
            RaylibError::from(LoadStyleFromMemoryError::LengthOverflow(5)),
            RaylibError::LoadStyleFromMemory(_)
        ));
        assert!(matches!(
            RaylibError::from(PixelColorError::CompressedFormat(
                PixelFormat::PIXELFORMAT_COMPRESSED_DXT1_RGB
            )),
            RaylibError::PixelColor(_)
        ));
        assert!(matches!(
            RaylibError::from(SetCallbackError("trace log")),
            RaylibError::SetCallback(_)
        ));
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo nextest run -p raylib -E 'test(raylib_error_from)'`
Expected: compile error — `no variant named UpdateAudioStream` (etc.).

- [ ] **Step 3: Add the 8 variants + `#[non_exhaustive]`**

(a) Add `#[non_exhaustive]` between `#[derive(Error, Debug)]` and `pub enum RaylibError {` (~line 944):

```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RaylibError {
```

(b) Append the 8 variants before the enum's closing brace (~line 1055), keeping the existing house style:

```rust
    /// Wraps an [`UpdateAudioStreamError`] surfaced through `?` from an audio-stream update
    /// or callback-install site.
    ///
    /// **Cause:** An `AudioStream::update` or `set_audio_stream_callback` call propagated up
    /// an [`UpdateAudioStreamError`].
    ///
    /// **Recovery:** Inspect the inner [`UpdateAudioStreamError`] variant and apply its
    /// recovery.
    #[error("audio stream update error")]
    UpdateAudioStream(#[from] UpdateAudioStreamError),
    /// Wraps an [`InvalidMeshError`] surfaced through `?` from a mesh-validation site.
    ///
    /// **Cause:** A mesh accessor or builder validated a [`crate::ffi::Mesh`] and found its
    /// buffers inconsistent.
    ///
    /// **Recovery:** Inspect the inner [`InvalidMeshError`] variant and apply its recovery.
    #[error("invalid mesh error")]
    InvalidMesh(#[from] InvalidMeshError),
    /// Wraps a [`GenMeshError`] surfaced through `?` from a mesh-generation site.
    ///
    /// **Cause:** A `MeshBuilder::build` (or other mesh-generation) call propagated up a
    /// [`GenMeshError`].
    ///
    /// **Recovery:** Inspect the inner [`GenMeshError`] variant and apply its recovery.
    #[error("mesh generation error")]
    GenMesh(#[from] GenMeshError),
    /// Wraps a [`Base64Error`] surfaced through `?` from a base64 encode/decode site.
    ///
    /// **Cause:** An `encode_data_base64` or `decode_data_base64` call propagated up a
    /// [`Base64Error`].
    ///
    /// **Recovery:** Inspect the inner [`Base64Error`] variant and apply its recovery.
    #[error("base64 error")]
    Base64(#[from] Base64Error),
    /// Wraps a [`LoadIconsError`] surfaced through `?` from a raygui icons-load site.
    ///
    /// **Cause:** A `gui_load_icons`/`gui_load_icons_from_memory` call propagated up a
    /// [`LoadIconsError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadIconsError`] variant and apply its recovery.
    #[error("icons loading error")]
    LoadIcons(#[from] LoadIconsError),
    /// Wraps a [`LoadStyleFromMemoryError`] surfaced through `?` from a raygui style-load site.
    ///
    /// **Cause:** A `gui_load_style_from_memory` call propagated up a
    /// [`LoadStyleFromMemoryError`].
    ///
    /// **Recovery:** Inspect the inner [`LoadStyleFromMemoryError`] variant and apply its
    /// recovery.
    #[error("style loading error")]
    LoadStyleFromMemory(#[from] LoadStyleFromMemoryError),
    /// Wraps a [`crate::core::pixel::PixelColorError`] surfaced through `?` from a
    /// pixel-level color read/write site.
    ///
    /// **Cause:** A `get_pixel_color`/`set_pixel_color` call propagated up a
    /// [`crate::core::pixel::PixelColorError`].
    ///
    /// **Recovery:** Inspect the inner error variant and apply its recovery.
    #[error("pixel color error")]
    PixelColor(#[from] crate::core::pixel::PixelColorError),
    /// Wraps a [`SetCallbackError`] surfaced through `?` from a callback-install site.
    ///
    /// **Cause:** A `set_*_callback` call found its global slot already occupied.
    ///
    /// **Recovery:** Keep a single registration site per callback kind.
    #[error("callback registration error")]
    SetCallback(#[from] SetCallbackError),
```

(c) Update the `RaylibError` rustdoc example (~lines 919–943): doctests compile as an
external crate, so `#[non_exhaustive]` makes a `_` arm mandatory. Replace the example with:

```rust
/// Top-level error type that aggregates all raylib-rs domain errors.
///
/// Marked `#[non_exhaustive]`: new domain errors may gain variants here in minor releases,
/// so matches must include a wildcard arm.
///
/// # Examples
///
/// ```no_run
/// use raylib::core::error::RaylibError;
///
/// fn handle(e: RaylibError) {
///     match e {
///         RaylibError::AudioInit(_) => eprintln!("audio device failed to initialize"),
///         RaylibError::LoadSound(_) => eprintln!("sound load failed"),
///         RaylibError::LoadModel(_) => eprintln!("model load failed"),
///         RaylibError::LoadFont(_) => eprintln!("font load failed"),
///         RaylibError::LoadTexture(_) => eprintln!("texture load failed"),
///         RaylibError::SetCallback(_) => eprintln!("callback slot already occupied"),
///         other => eprintln!("raylib error: {other}"),
///     }
/// }
/// ```
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo nextest run -p raylib -E 'test(raylib_error_from)'`
Expected: PASS (1 test).

- [ ] **Step 5: Definition-of-done coverage check**

Every `pub enum *Error`/`pub struct *Error` leaf in the safe crate must now have a `#[from]`
in `RaylibError`. Verify the counts line up:

```bash
grep -E "^pub (enum|struct) \w*Error" raylib/src/core/error.rs raylib/src/core/pixel.rs
grep -c "#\[from\]" <(awk '/^pub enum RaylibError/,/^\}/' raylib/src/core/error.rs)
```
Expected: **22 declarations** grepped (error.rs: 19 pre-existing leaf enums + `RaylibError`
itself + the new `SetCallbackError` struct = 21; pixel.rs: `PixelColorError` = 1). Minus
`RaylibError` itself → **21 leaves**, matching **21 `#[from]` lines** in the awk-extracted
`RaylibError` block (13 existing + 8 new). Note `GenMeshError`'s own two `#[from]` lines live
inside *that* enum — the awk extraction is scoped to `RaylibError` only, so they don't
inflate the count. (If the numbers disagree, list which leaf lacks a variant and add it.)

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add raylib/src/core/error.rs
git commit -m "feat(error): complete RaylibError aggregate + mark non_exhaustive

Adds #[from] variants for UpdateAudioStream/InvalidMesh/GenMesh/Base64/
LoadIcons/LoadStyleFromMemory/PixelColor/SetCallback so every leaf error
composes via ?. BREAKING (pre-6.0.0-final): exhaustive matches on
RaylibError stop compiling (new variants + non_exhaustive).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 3: CHANGELOG entries

**Files:**
- Modify: `CHANGELOG.md` (the `### Breaking` section of the `## 6.0.0-rc.2` block — per that block's header note, it describes what ships in `6.0.0` final)

- [ ] **Step 1: Add the two entries**

Append to the existing `### Breaking` bullet list:

```markdown
- `SetLogError` (core/callbacks) is replaced by `SetCallbackError` in `core::error` — a `thiserror` struct without the artificial lifetime parameter. The ~10 `set_*_callback` functions/methods now return `Result<(), SetCallbackError>`.
- `RaylibError` is now `#[non_exhaustive]` and gained `#[from]` variants for `UpdateAudioStreamError`, `InvalidMeshError`, `GenMeshError`, `Base64Error`, `LoadIconsError`, `LoadStyleFromMemoryError`, `PixelColorError`, and `SetCallbackError` — every leaf error now composes via `?`. Exhaustive matches on `RaylibError` need a wildcard arm.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): record SetCallbackError rename + RaylibError completion

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>"
```

---

## Task 4: Full verification + PR

- [ ] **Step 1: Full test suite**

Run from repo root:
```bash
cargo nextest run -p raylib
cargo test --doc -p raylib
```
Expected: all green. The doctests cover the updated `SetCallbackError` and `RaylibError`
examples. (Note: per CLAUDE.md, a couple of pre-existing software-renderer doctest failures
exist only under the SR feature set — the default-feature doc run here is expected clean.)

- [ ] **Step 2: Quality gates**

```bash
cargo fmt --all -- --check
cargo clippy -p raylib --all-targets -- -D warnings
```
Expected: both clean. (`deny(missing_docs)` is enforced at compile time, so Step 1 already
proved docs coverage.)

- [ ] **Step 3: Push + PR**

```bash
git push -u origin chore/thiserror-migration
gh pr create --base unstable --repo raylib-rs/raylib-rs \
  --title "refactor(error): finish the thiserror migration (SetCallbackError + complete RaylibError)" \
  --body "<summary of: scope finding (migration was ~95% done), the SetLogError->SetCallbackError rename, the 8 new RaylibError variants + non_exhaustive, the two CHANGELOG breaking entries, and the deliberate keep-as-is decision for window.rs std-error returns. Link the spec. End with the Claude Code attribution line.>"
```

- [ ] **Step 4: Mark flexible-queue item 12 complete**

After the PR merges, no separate done-note is needed (spec + plan suffice per the spec's
Deliverables); the merge itself closes flexible-queue item 12.

---

## Self-review notes

- **Spec coverage:** SetCallbackError shape + location (Task 1), lifetime removal + 10 signatures (Task 1 Step 4 d/e), uniform `Result` on `set_trace_log_callback` kept (Task 1 Step 4d note), 8 `#[from]` variants + non_exhaustive + doc example (Task 2), std-error returns untouched (no task — deliberate), Display + conversion tests (Tasks 1–2), CHANGELOG (Task 3), gates + PR (Task 4). Definition-of-done greps embedded in Task 1 Step 6 and Task 2 Step 5. ✓
- **Type consistency:** `SetCallbackError(pub(crate) &'static str)` constructed as `SetCallbackError($ty)` in the macro and `SetCallbackError("trace log")`/`("save file data")` in tests — consistent. Variant names in Task 2 Step 1 tests match Step 3 definitions. ✓
- **Doctest gotcha:** the `SetCallbackError` doc example can't construct the type directly (field is `pub(crate)`) — it goes through `set_save_file_data_callback`, which exists by the time the doctest compiles (same commit). ✓
