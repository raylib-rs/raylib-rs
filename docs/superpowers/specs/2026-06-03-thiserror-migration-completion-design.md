# thiserror migration completion — design

**Date:** 2026-06-03
**Branch:** `chore/thiserror-migration` (off canonical `unstable` @ `1462ea1`)
**Status:** approved, pre-implementation
**Origin:** flexible-queue item 12 (`ws8e-checkpoint-review-feedback.md` §7 — owner: "TODO: Use this_error for all error types.")

## Scope finding (why this is small)

The review comment that spawned this workstream is largely stale: `raylib/src/core/error.rs`
already holds ~19 leaf error enums, all `#[derive(thiserror::Error)]` with house-style
Cause/Recovery variant docs, plus a `RaylibError` aggregate with `#[from]` composition.
`audio_stream_callback.rs` (the commented file) already returns
`Result<(), UpdateAudioStreamError>`. A crate-wide sweep found exactly one hand-rolled
error type and one incomplete aggregate. This spec completes the migration rather than
restarting it.

## Goals

1. **Zero hand-rolled error impls in the safe crate.** The single remaining manual
   `impl Display` + `impl std::error::Error` (`SetLogError` in `core/callbacks.rs`) is
   replaced by a thiserror type.
2. **Complete `?` composition.** Every public leaf error type converts into `RaylibError`
   via `#[from]`.
3. **Future-proof the aggregate.** `RaylibError` becomes `#[non_exhaustive]` so adding
   leaf errors later is not a semver-major event.

## Non-goals

- No renaming or restructuring of the existing 19 leaf error enums.
- No wrapping of the two std-error returns in `core/window.rs`
  (`get_monitor_name`/`get_monitor_info` → `std::ffi::IntoStringError`,
  `get_clipboard_text` → `std::str::Utf8Error`). These are precise std types for genuinely
  std failures; wrapping adds a layer for no gain. **Deliberate decision, recorded here.**
- No `source()` re-plumbing or error-message rewording of existing types.
- Leaf enums stay exhaustive (matching leaf variants is the documented pattern);
  `#[non_exhaustive]` applies to the `RaylibError` aggregate only.

## Design

### 1. `SetCallbackError` replaces `SetLogError`

New type in `raylib/src/core/error.rs`, alongside the other error types:

```rust
/// Error returned when installing a callback whose global slot is already occupied.
/// (house-style docs: Cause / Recovery / # Examples)
#[derive(Error, Debug)]
#[error("there is a {0} callback already set")]
pub struct SetCallbackError(pub(crate) &'static str);
```

Rationale for a struct over an enum: the error has exactly one cause ("slot already
occupied"); the only payload is *which* slot, and the call site already determines that.
Per-kind enum variants would add ~7 documented variants for no matchability gain.

Changes in `raylib/src/core/callbacks.rs`:
- Delete `SetLogError<'a>` and its manual `Display`/`Error` impls (lines ~163–172).
- The artificial lifetime disappears: all ~10 setter signatures change from
  `Result<(), SetLogError<'a>>` to `Result<(), SetCallbackError>` (the `'a`/`'b` lifetime
  params on those functions go away entirely).
- `safe_callback_set_func!` macro body: `Err(SetLogError($ty))` → `Err(SetCallbackError($ty))`.
  The `&'static str` kind arguments ("save file data", …) are unchanged.
- `set_trace_log_callback` keeps its `Result` signature for API uniformity even though it
  currently always returns `Ok` (it overwrites rather than guards its slot).
- Doc examples referencing `SetLogError` (e.g. the doctest above the type) updated.

**Breaking change** (allowed pre-6.0.0-final): public type `SetLogError` is renamed; no
deprecated alias (the lifetime parameter makes an alias awkward and we are pre-final).
CHANGELOG entry required.

### 2. `RaylibError` aggregate completion

Add the eight missing `#[from]` variants, each with house-style Cause/Recovery docs:

| New variant | Wraps | Defined in |
|---|---|---|
| `UpdateAudioStream` | `UpdateAudioStreamError` | error.rs |
| `InvalidMesh` | `InvalidMeshError` | error.rs |
| `GenMesh` | `GenMeshError` | error.rs |
| `Base64` | `Base64Error` | error.rs |
| `LoadIcons` | `LoadIconsError` | error.rs |
| `LoadStyleFromMemory` | `LoadStyleFromMemoryError` | error.rs |
| `PixelColor` | `PixelColorError` | core/pixel.rs |
| `SetCallback` | `SetCallbackError` | error.rs (new) |

Plus:
- `#[non_exhaustive]` on `RaylibError`.
- Its rustdoc match example gains a `_` arm (required by `non_exhaustive` anyway) and a
  sentence noting the enum is non-exhaustive.

**Breaking change**: exhaustive matches on `RaylibError` stop compiling (both from the new
variants and from `non_exhaustive`). CHANGELOG entry required.

### 3. Tests (Tier-1, window-independent)

- `SetCallbackError` Display output matches `"there is a save file data callback already set"`.
- A conversion test exercising `RaylibError::from(...)`/`?` for each of the 8 new variants
  (compile-time guarantee that `#[from]` is wired; runtime asserts on variant identity).
- Existing gates do the rest: `deny(missing_docs)` forces docs on every new variant,
  clippy `-D warnings`, doctests (the updated examples in callbacks.rs + error.rs must pass).

### 4. Deliverables

- One PR to canonical `unstable` from `chore/thiserror-migration`.
- `CHANGELOG.md` entries under the unreleased/6.0.0 section: the `SetLogError` →
  `SetCallbackError` rename and the `RaylibError` `non_exhaustive` + new variants.
- No done-note needed beyond this spec + the plan (small workstream); the flexible-queue
  tracking note can mark item 12 complete when the PR merges.

## Definition of done

- `grep` finds no manual `impl std::error::Error`/`impl Display for *Error` in `raylib/src`.
- Every `pub enum *Error`/`pub struct *Error` leaf in the safe crate has a `#[from]` variant
  in `RaylibError` (std error types in window.rs deliberately excluded).
- `RaylibError` is `#[non_exhaustive]`.
- `cargo nextest run` + `cargo test --doc` green from `raylib/`; fmt + clippy `-Dwarnings` clean.
- CHANGELOG updated.
