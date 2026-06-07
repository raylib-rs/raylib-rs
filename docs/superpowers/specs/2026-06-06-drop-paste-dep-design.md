# Drop the `paste` dependency — design

**Date:** 2026-06-06
**Status:** approved (maintainer picked Approach A this session; executed same-session)
**Queue:** post-release flexible-queue item 16 (`ws8e-checkpoint-review-feedback.md`, from WS8d Fold-in 2)
**Advisory:** RUSTSEC-2024-0436 (`paste` 1.0 unmaintained, repo archived 2024-10)

## Goal

Remove the `paste` proc-macro dependency from the `raylib` crate. Its only
consumer is the audio-callback trampoline pool
(`raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`),
where `paste!` builds `CLOSURE_N` / `callback_N` identifiers inside a
`generate_functions!` helper driven by `seq_macro::seq!`.

## Decision: Approach A — array + const-generic trampolines

- `const SLOTS: usize = 30` (same capacity); `static CLOSURES:
  [Mutex<AudioCallbackWithUserData>; SLOTS]` built with an inline-const
  initializer and a new `const fn empty()` (Default delegates to it).
  `LazyLock` is gone — `Mutex::new` is const.
- One generic `extern "C" fn trampoline<const N: usize>` replaces the 30
  generated fns (monomorphized generic fns are valid C fn pointers). Body
  unchanged: lock slot N, call or panic; guard held while the user callback
  runs (same re-entrancy/mutual-exclusion semantics as before).
- `static TRAMPOLINES: [RawAudioCallback; SLOTS]` built by the one remaining
  `seq!` literal. The array type makes a SLOTS/seq-range mismatch a compile
  error.
- `set_context` / `clear_context` / `get_callback` become ordinary
  loops/indexing instead of 30 unrolled if-chains; the
  `unpredictable_function_pointer_comparisons` allows disappear
  (`is_none()` instead of `== None`).
- The old trampolines were `#[unsafe(no_mangle)] pub` — 30 unmangled
  `callback_N` symbols in the global linker namespace (collision bait, e.g.
  with any C library defining `callback_0`). The module is private, so they
  were never reachable Rust API; the new trampoline is private and mangled.

**Rejected — B: seq!-only rewrite in place.** Also zero new deps (seq! does
`~N` identifier pasting), but keeps LazyLock, the unrolled if-chains, and the
no_mangle pollution.

**Rejected — C: swap to `pastey`.** Trades an unmaintained proc-macro dep for
another third-party proc-macro dep when an existing dependency covers the
need.

## Behavior preserved

Public surface of the module (`AudioCallbackWithUserData`,
`attach_/detach_audio_stream_processor_with_user_data`,
`attach_/detach_audio_mixed_processor_with_user_data`) and all semantics:
0-based reusable slot indices, first-free-wins reservation, panics on
exhaustion / double-clear / out-of-bounds, guard held during callback
invocation, detach-before-clear lifecycle ordering.

## Testing

New Tier-1 unit test in-file (the slot logic never touches FFI): fill all 30
slots → distinct in-bounds indices; per-slot trampoline pointer
distinctness (raylib's detach identity-matches the fn pointer); clear →
first-free-wins reuse. Deliberately a single test fn: the pool is a
process-global, so parallel test threads would race for slots and a
`should_panic` test would poison a slot mutex.

## Cleanup

- `raylib/Cargo.toml`: `paste = "1.0"` removed (`seq-macro` stays).
- `deny.toml`: RUSTSEC-2024-0436 ignore entry retired (comment breadcrumb
  left).
- CHANGELOG: Unreleased → Changed entry (dep removal + the no_mangle symbol
  change, which is linker-observable).

## Done criteria

- `cargo tree -i paste` finds no package.
- Quality gates green (clippy `-Dwarnings`, nextest, cargo-deny, fmt).
- PR to `unstable`; queue item 16 closed.
