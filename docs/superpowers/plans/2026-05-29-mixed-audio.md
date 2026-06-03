# mixed-audio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wrap raylib's `AttachAudioMixedProcessor` / `DetachAudioMixedProcessor` with a safe RAII guard `MixedAudioProcessorCallback` that lives alongside the per-stream sibling in `callbacks.rs`, sharing its 30-slot trampoline pool.

**Architecture:** Two existing files modified. The slot-pool helper file (`stream_processor_with_user_data_wrapper.rs`) gets `pub(crate)` visibility on the three slot primitives + two new mixed-bus helpers wrapping the FFI calls. `callbacks.rs` gets the new `MixedAudioProcessorCallback` struct (mirrors `AudioStreamProcessorCallback`'s shape) + the `attach_audio_mixed_processor` free fn + two Tier-2 lifecycle tests via `test_harness::with_headless`.

**Tech Stack:** Rust 1.85 (edition 2024), reuses existing `paste` + `seq_macro` machinery (no new deps), `software_renderer` feature for Tier-2 tests, `RaylibAudio::init_audio_device()` for the audio thread in tests.

**Spec reference:** `docs/superpowers/specs/2026-05-29-mixed-audio-design.md`.

---

### Task 1: Bump slot primitives to `pub(crate)` + update header comment

**Files:**
- Modify: `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`

The slot primitives (`set_context` / `clear_context` / `get_callback`) are currently file-private. The new mixed-bus helpers need to call them, so widen the visibility to `pub(crate)`. Also update the file header comment to reflect the broader role (both per-stream and mixed-bus consumers).

- [ ] **Step 1: Read the existing header + locate the three primitives**

Run: `grep -nE '^(fn|pub fn) (set_context|clear_context|get_callback|attach_audio_stream|detach_audio_stream)' raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`

Expected: matches showing `set_context`, `clear_context`, `get_callback` at file-private visibility (no `pub`) — currently inside the `paste! { ... }` macro expansion. The `attach_audio_stream_processor_with_user_data` and `detach_audio_stream_processor_with_user_data` are the existing `pub fn` consumers.

- [ ] **Step 2: Bump `set_context` to `pub(crate)`**

Inside the `paste! { ... }` block, change the `set_context` declaration from `fn set_context(...)` to `pub(crate) fn set_context(...)`. Keep all other code unchanged.

- [ ] **Step 3: Bump `clear_context` to `pub(crate)`**

Same: change `fn clear_context(...)` to `pub(crate) fn clear_context(...)`.

- [ ] **Step 4: Bump `get_callback` to `pub(crate)`**

Same: change `fn get_callback(...)` to `pub(crate) fn get_callback(...)`.

- [ ] **Step 5: Replace the top-of-file header comment**

Replace the existing `// region: -- ...` comments at the top of the file (the section before the `RawAudioCallbackWithUserData` type alias) with this module-level doc comment:

```rust
//! Shared trampoline-slot pool for closure-driven audio callbacks.
//!
//! raylib's C-side audio processors (`AttachAudioStreamProcessor`,
//! `AttachAudioMixedProcessor`) accept only a function pointer — no
//! user-data parameter. To thread closure state through, we
//! pre-register **30 trampolines** (named `callback_0` through
//! `callback_29`), each with its own
//! `LazyLock<Mutex<AudioCallbackWithUserData>>` slot. A consumer
//! reserves a free slot via [`set_context`], the trampoline for that
//! slot looks up the closure context, and [`clear_context`] frees the
//! slot.
//!
//! Consumers:
//! - Per-stream: [`attach_audio_stream_processor_with_user_data`] /
//!   [`detach_audio_stream_processor_with_user_data`].
//! - Mixed bus: [`attach_audio_mixed_processor_with_user_data`] /
//!   [`detach_audio_mixed_processor_with_user_data`].
//!
//! Both consumers share the same 30-slot pool. The file is named
//! `stream_processor_with_user_data_wrapper.rs` for historical
//! reasons; it now covers both kinds of audio processors.
```

Insert this immediately after any `use` statements at the top of the file (the doc comment must be at module level, not preceded by other items).

- [ ] **Step 6: Verify the workspace still compiles**

Run: `cargo build -p raylib --features full 2>&1 | tail -5`
Expected: `Finished` line. The `pub(crate)` bumps add to surface but don't break anything; existing `pub fn` consumers still call the now-`pub(crate)` primitives identically.

- [ ] **Step 7: Commit Task 1**

Run via Bash tool:

```bash
git add raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs
git commit -m "$(cat <<'EOF'
refactor(audio-callbacks): bump slot primitives to pub(crate) + module-level doc

Widens visibility on `set_context` / `clear_context` / `get_callback`
inside the slot-pool macro expansion so the upcoming mixed-bus
consumer can call them. Also replaces the legacy region comments
with a proper module-level doc comment explaining the shared
30-slot pool serves both per-stream and mixed-bus audio processors.

No public API change; no functional change.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Add `attach_audio_mixed_processor_with_user_data` + detach helpers

**Files:**
- Modify: `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`

Append the two new `pub(crate)` helpers after the existing per-stream attach/detach functions. They share the slot pool but call the **mixed-bus** FFI functions instead of per-stream ones.

- [ ] **Step 1: Locate the insertion point**

Run: `grep -n 'pub fn detach_audio_stream_processor_with_user_data' raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`

Expected: one match (the existing per-stream detach helper). Insert the new helpers immediately after its closing `}`.

- [ ] **Step 2: Append the two new helpers**

After the closing `}` of `detach_audio_stream_processor_with_user_data`, append:

```rust

/// Attach a closure-driven processor to raylib's global mixed audio
/// bus. Reserves a slot from the shared 30-slot pool and calls the
/// C-side `AttachAudioMixedProcessor` with that slot's trampoline.
///
/// Returns the slot index — pass it to
/// [`detach_audio_mixed_processor_with_user_data`] when the consumer
/// drops.
pub(crate) fn attach_audio_mixed_processor_with_user_data(
    callback: AudioCallbackWithUserData,
) -> usize {
    let idx = set_context(callback);
    // SAFETY: get_callback(idx) returns the trampoline matching the
    // slot we just reserved; raylib copies the fn pointer into its
    // linked-list entry. We hold the slot until drop calls detach +
    // clear_context.
    unsafe {
        raylib_sys::AttachAudioMixedProcessor(Some(get_callback(idx)));
    }
    idx
}

/// Detach the closure-driven mixed-bus processor and clear the slot.
///
/// Calls the C-side `DetachAudioMixedProcessor` with the same
/// trampoline pointer registered for `index`, so raylib's internal
/// list-search matches and the entry is removed. The slot is only
/// cleared AFTER the C side has stopped iterating it — without this
/// the closure could still receive one more invocation against freed
/// state (same lifecycle ordering as the per-stream sibling fixed in
/// WS8e).
pub(crate) fn detach_audio_mixed_processor_with_user_data(index: usize) {
    let trampoline = get_callback(index);
    // SAFETY: trampoline is the same fn pointer raylib stored at
    // attach time; the C-side list-search matches and removes the
    // entry. clear_context runs AFTER detach completes so raylib has
    // stopped iterating before the closure context disappears.
    unsafe {
        raylib_sys::DetachAudioMixedProcessor(Some(trampoline));
    }
    clear_context(index);
}
```

- [ ] **Step 3: Verify the workspace compiles**

Run: `cargo build -p raylib --features full 2>&1 | tail -5`
Expected: `Finished` line, no errors.

If `raylib_sys::AttachAudioMixedProcessor` / `DetachAudioMixedProcessor` aren't resolvable, check the FFI binding name (`grep -n 'AttachAudioMixedProcessor' raylib-sys/src/lib.rs` or the bindgen output). They should be there — bindgen translates `RLAPI void AttachAudioMixedProcessor(AudioCallback)` directly.

- [ ] **Step 4: Verify clippy is clean**

Run: `cargo clippy -p raylib --features full -- -D warnings 2>&1 | tail -5`
Expected: no warnings.

- [ ] **Step 5: Commit Task 2**

```bash
git add raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs
git commit -m "$(cat <<'EOF'
feat(audio-callbacks): add mixed-bus attach/detach helpers (pub(crate))

Two new `pub(crate)` helpers in the slot-pool file that mirror the
per-stream attach/detach pattern but invoke
AttachAudioMixedProcessor / DetachAudioMixedProcessor instead of
the per-stream FFI calls.

Lifecycle correctness mirrors the WS8e per-stream fix: the detach
helper calls DetachAudioMixedProcessor BEFORE clear_context, so
raylib's internal list-walk has stopped iterating the slot before
the closure context disappears.

No public API yet — the `pub` user-facing entry point lands in
the next task.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Add `MixedAudioProcessorCallback` struct + impl + Drop

**Files:**
- Modify: `raylib/src/core/callbacks.rs`

Append the new struct after the closing `// endregion: -- AudioStreamProcessorCallback --` comment. The shape mirrors `AudioStreamProcessorCallback` but without the per-stream `stream` field.

- [ ] **Step 1: Locate the insertion point**

Run: `grep -n 'endregion: -- AudioStreamProcessorCallback' raylib/src/core/callbacks.rs`

Expected: one match. Insert the new code immediately after that closing comment line, before the `pub fn attach_audio_stream_processor_to_music` definition.

- [ ] **Step 2: Verify the existing imports cover what we need**

Run: `head -25 raylib/src/core/callbacks.rs`

Expected: the file already imports `std::pin::Pin`, `std::os::raw::*`, `crate::core::audio::Music`, `raylib_sys`. We need `crate::core::audio::RaylibAudio` (a sibling). Check the existing import line for `audio::`:

Run: `grep -n 'use crate::core::audio' raylib/src/core/callbacks.rs`

Expected: an existing line importing `Music`. Extend it to also import `RaylibAudio` (or add a new `use` line if the existing pattern is `use ...::Music;` only).

- [ ] **Step 3: Add the `RaylibAudio` import**

If the existing import is `use crate::core::audio::Music;`, replace it with:

```rust
use crate::core::audio::{Music, RaylibAudio};
```

If it's a multi-line `use crate::core::audio::{ ... }`, add `RaylibAudio` to the brace list alphabetically.

If there's no existing import (Music is imported some other way), add a fresh line:

```rust
use crate::core::audio::RaylibAudio;
```

- [ ] **Step 4: Append the struct + impl + Drop block**

After the `// endregion: -- AudioStreamProcessorCallback --` line, append:

```rust

// region: -- MixedAudioProcessorCallback --

/// Closure-driven processor attached to raylib's **global mixed audio
/// bus**.
///
/// The processor receives every stereo frame after raylib has mixed
/// all playing streams. Multiple `MixedAudioProcessorCallback`
/// instances can be attached simultaneously; they form a chain in
/// raylib's internal linked list and run in attach-order on each
/// frame.
///
/// This is a guard type — drop it to detach. Users obtain one via
/// [`attach_audio_mixed_processor`]; direct construction is not
/// supported (no public constructor).
pub struct MixedAudioProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32) + Send + 'static,
{
    rust_callback: &'a mut F,
    /// Set after `attach_audio_mixed_processor_with_user_data` succeeds.
    /// `None` only during the construction window before attach.
    callback_index: Option<usize>,
}

impl<'a, F> MixedAudioProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32) + Send + 'static,
{
    fn new(closure: &'a mut F) -> Self {
        Self {
            rust_callback: closure,
            callback_index: None,
        }
    }

    fn get_as_user_data(&mut self) -> *mut ::std::os::raw::c_void {
        self as *mut Self as *mut ::std::os::raw::c_void
    }

    fn get_c_callback(
        &mut self,
    ) -> extern "C" fn(
        *mut ::std::os::raw::c_void,
        *mut ::std::os::raw::c_void,
        ::std::os::raw::c_uint,
    ) -> () {
        Self::c_callback
    }

    extern "C" fn c_callback(
        user_data: *mut ::std::os::raw::c_void,
        data_ptr: *mut ::std::os::raw::c_void,
        frame_count: ::std::os::raw::c_uint,
    ) {
        // SAFETY: user_data is `*mut Self` per get_as_user_data; the
        // wrapping Pin<Box<Self>> keeps it stable for the guard's
        // lifetime, which outlives every callback firing per the
        // &'a RaylibAudio borrow. data_ptr is `frame_count * 2 *
        // sizeof(f32)` interleaved stereo bytes (mixed bus is always
        // stereo per raylib.h:1736).
        unsafe {
            let cb: &mut Self = user_data.cast::<Self>().as_mut().unwrap();
            let data = std::slice::from_raw_parts_mut(
                data_ptr as *mut f32,
                frame_count as usize * 2,
            );
            (cb.rust_callback)(data, 2);
        }
    }
}

impl<F> Drop for MixedAudioProcessorCallback<'_, F>
where
    F: FnMut(&mut [f32], u32) + Send + 'static,
{
    fn drop(&mut self) {
        if let Some(idx) = self.callback_index {
            detach_audio_mixed_processor_with_user_data(idx);
        }
    }
}

// endregion: -- MixedAudioProcessorCallback --
```

- [ ] **Step 5: Add the `detach_audio_mixed_processor_with_user_data` import if missing**

`callbacks.rs` already imports the per-stream helpers from `stream_processor_with_user_data_wrapper`. Find that import:

Run: `grep -nE 'use.*stream_processor_with_user_data_wrapper' raylib/src/core/callbacks.rs`

Expected: one line bringing in the per-stream helpers (likely `attach_audio_stream_processor_with_user_data` + `detach_audio_stream_processor_with_user_data` + `AudioCallbackWithUserData`).

Extend the brace list to also include `attach_audio_mixed_processor_with_user_data` and `detach_audio_mixed_processor_with_user_data`. The result should be e.g.:

```rust
use super::callbacks::stream_processor_with_user_data_wrapper::{
    AudioCallbackWithUserData,
    attach_audio_mixed_processor_with_user_data,
    attach_audio_stream_processor_with_user_data,
    detach_audio_mixed_processor_with_user_data,
    detach_audio_stream_processor_with_user_data,
};
```

(Adapt the path to whatever the existing import uses — it might be `mod` declared in a nearby file rather than via `super::`.)

- [ ] **Step 6: Verify the workspace compiles**

Run: `cargo build -p raylib --features full 2>&1 | tail -10`
Expected: `Finished` line, no errors. If the trampoline `c_callback` complains about not being callable from `extern "C"` context, double-check the function signature matches the per-stream pattern verbatim — they should be identical.

- [ ] **Step 7: Verify clippy is clean**

Run: `cargo clippy -p raylib --features full -- -D warnings 2>&1 | tail -5`
Expected: no warnings.

- [ ] **Step 8: Commit Task 3**

```bash
git add raylib/src/core/callbacks.rs
git commit -m "$(cat <<'EOF'
feat(audio-callbacks): add MixedAudioProcessorCallback struct + impl + Drop

Mirrors AudioStreamProcessorCallback's shape but for raylib's global
mixed audio bus. Differences from the per-stream sibling:
- No `stream` field (mixed bus is global, not per-stream).
- Drop calls detach_audio_mixed_processor_with_user_data with just
  the slot index (no stream argument needed by the FFI).
- c_callback hardcodes `nb_channels = 2` (mixed bus is always
  stereo per raylib.h:1736).

The struct uses the existing 30-slot trampoline pool, so a
MixedAudioProcessorCallback + a per-stream
AudioStreamProcessorCallback compete for the same 30 slots.

No public attach fn yet — that lands in the next task.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Add the public `attach_audio_mixed_processor` free function

**Files:**
- Modify: `raylib/src/core/callbacks.rs`

The user-facing entry point. Mirrors `attach_audio_stream_processor_to_music`'s shape; borrows `&'a RaylibAudio` so the audio device cannot drop while a processor is attached.

- [ ] **Step 1: Locate the insertion point**

Run: `grep -n 'pub fn attach_audio_stream_processor_to_music' raylib/src/core/callbacks.rs`

Expected: one match (the existing per-stream public attach fn). Find its closing `}` and insert the new fn immediately after.

- [ ] **Step 2: Append the free function**

After the closing `}` of `attach_audio_stream_processor_to_music`, append:

```rust

/// Attach a closure to raylib's **global mixed audio bus**, returning
/// a pinned guard that detaches on drop.
///
/// The callback fires from raylib's internal audio thread, receiving
/// **interleaved stereo** frames (`channels == 2` always — raylib
/// mixes all playing streams down to stereo before invoking the
/// processor). Multiple `MixedAudioProcessorCallback` instances can
/// be attached simultaneously and run in attach-order.
///
/// # Closure constraints
///
/// `F: FnMut(&mut [f32], u32) + Send + 'static`:
/// - `Send + 'static` because the callback runs on raylib's audio
///   thread.
/// - The closure receives a mutable slice of `frame_count * 2`
///   interleaved stereo samples plus the channel count (always `2`).
///   Modifying the slice in-place applies the effect to the mixed
///   bus.
///
/// # Slot pool exhaustion
///
/// The crate supports up to **30 simultaneous** audio-processor
/// closures (shared pool across per-stream and mixed-bus consumers).
/// Attaching a 31st processor panics with `"index out of bounds"`.
/// Detach existing processors (drop their guards) to free slots.
///
/// # Thread safety
///
/// raylib's `AttachAudioMixedProcessor` /
/// `DetachAudioMixedProcessor` are internally mutex-guarded; attach
/// and drop are safe to call from any thread.
pub fn attach_audio_mixed_processor<'a, F>(
    _audio: &'a RaylibAudio,
    processor: &'a mut F,
) -> Pin<Box<MixedAudioProcessorCallback<'a, F>>>
where
    F: FnMut(&mut [f32], u32) + Send + 'static, // static because the function is executed in another thread
{
    let mut cb = Box::new(MixedAudioProcessorCallback::<'a, F>::new(processor));
    let idx = attach_audio_mixed_processor_with_user_data(AudioCallbackWithUserData::new(
        cb.get_as_user_data(),
        cb.get_c_callback(),
    ));
    cb.callback_index = Some(idx);
    Box::into_pin(cb)
}
```

- [ ] **Step 3: Verify the workspace compiles**

Run: `cargo build -p raylib --features full 2>&1 | tail -10`
Expected: `Finished` line, no errors.

- [ ] **Step 4: Verify clippy is clean**

Run: `cargo clippy -p raylib --features full -- -D warnings 2>&1 | tail -5`
Expected: no warnings.

- [ ] **Step 5: Verify rustdoc is clean**

Run: `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps 2>&1 | tail -10`
Expected: clean build. Intradoc-links to `MixedAudioProcessorCallback` and `RaylibAudio` resolve.

- [ ] **Step 6: Commit Task 4**

```bash
git add raylib/src/core/callbacks.rs
git commit -m "$(cat <<'EOF'
feat(audio-callbacks): public attach_audio_mixed_processor free fn

User-facing entry point. Mirrors attach_audio_stream_processor_to_music
but takes `&'a RaylibAudio` instead of `&'a Music` — the mixed bus
is global to the audio device, not tied to a single playing stream.

Returns Pin<Box<MixedAudioProcessorCallback>> as a guard. Drop calls
DetachAudioMixedProcessor + clear_context (same lifecycle ordering
as the WS8e per-stream fix).

Rustdoc covers the closure constraints (Send + 'static; stereo-only
buffer), the 30-slot shared-pool exhaustion ceiling, and the C-side
thread-safety story (mutex-guarded attach/detach).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: Lifecycle tests (attach + drop, multiple-attach)

**Files:**
- Modify: `raylib/src/core/callbacks.rs` (append a `#[cfg(test)] mod mixed_audio_tests` block)

Two Tier-2 tests gated on `feature = "software_renderer"`. Both open a 1×1 software-renderer window via `test_harness::with_headless`, init the audio device, attach + drop processors, and verify no panic.

- [ ] **Step 1: Locate the insertion point**

Run: `tail -20 raylib/src/core/callbacks.rs`

Expected: the file ends after the last public fn (likely `attach_audio_mixed_processor` from Task 4). Append the test module at the very bottom of the file.

- [ ] **Step 2: Append the test module**

At the end of `raylib/src/core/callbacks.rs`, append:

```rust

#[cfg(test)]
#[cfg(feature = "software_renderer")]
mod mixed_audio_tests {
    use super::*;
    use crate::core::audio::RaylibAudio;

    /// Attaching a mixed-bus processor and immediately dropping the
    /// guard must not panic. Proves the slot pool + Drop's
    /// DetachAudioMixedProcessor + clear_context all work together.
    #[test]
    fn attach_then_drop_does_not_panic() {
        crate::test_harness::with_headless(1, 1, |_rl, _thread| {
            let audio = RaylibAudio::init_audio_device().expect("audio init");
            let mut closure = |_samples: &mut [f32], _ch: u32| {};
            {
                let _guard = attach_audio_mixed_processor(&audio, &mut closure);
                // Guard dropped at end of scope — detach + slot clear.
            }
            // If we got here without panic, attach + drop both worked.
        });
    }

    /// Multiple simultaneous processors plus out-of-order drops must
    /// not corrupt the slot pool or raylib's linked list.
    #[test]
    fn multiple_processors_attach_and_drop_independently() {
        crate::test_harness::with_headless(1, 1, |_rl, _thread| {
            let audio = RaylibAudio::init_audio_device().expect("audio init");
            let mut closure_a = |_s: &mut [f32], _c: u32| {};
            let mut closure_b = |_s: &mut [f32], _c: u32| {};
            let guard_a = attach_audio_mixed_processor(&audio, &mut closure_a);
            let guard_b = attach_audio_mixed_processor(&audio, &mut closure_b);
            // Drop in reverse order; both should clean up.
            drop(guard_b);
            drop(guard_a);
        });
    }
}
```

- [ ] **Step 3: Run the tests under the software_renderer feature**

The Tier-2 software_renderer feature is mutually exclusive with the OpenGL backends, so it doesn't run under `--features full`. Use the same invocation the WS4/WS5 CI uses:

Run:
```bash
cargo test -p raylib --lib --no-default-features --features software_renderer,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RAUDIO,SUPPORT_FILEFORMAT_PNG mixed_audio_tests -- --test-threads=1 2>&1 | tail -10
```

Expected: `test result: ok. 2 passed; 0 failed`. The `--test-threads=1` flag is required because raylib enforces single-init via a lock (`raylib/src/core/mod.rs:415`).

If `RaylibAudio::init_audio_device()` fails on the CI runner (no audio output device), the test fails with the `.expect("audio init")` panic. If this is a problem in practice, the test may need to be gated further (e.g., `#[cfg_attr(target_os = "...", ignore)]`). Discover during execution; only escalate if it actually fires.

- [ ] **Step 4: Verify rustfmt is clean**

Run: `cargo fmt --check 2>&1 | tail -3`
Expected: zero output, exit 0. If rustfmt wants any line-collapses, run `cargo fmt` and re-verify before committing.

- [ ] **Step 5: Commit Task 5**

```bash
git add raylib/src/core/callbacks.rs
git commit -m "$(cat <<'EOF'
test(mixed-audio): Tier-2 lifecycle tests for attach + drop

Two #[test] fns in a new mixed_audio_tests module at the bottom of
callbacks.rs, gated on feature = "software_renderer":

- attach_then_drop_does_not_panic: single attach + scope-exit drop
  through with_headless + init_audio_device. Proves slot pool +
  Drop's DetachAudioMixedProcessor + clear_context all work together.
- multiple_processors_attach_and_drop_independently: two
  simultaneous attaches + out-of-order drops. Proves the slot pool
  + raylib's linked list survive multi-consumer churn.

Tests require `-- --test-threads=1` per raylib's single-init lock —
matches WS4/WS5 conventions in test.yml.

Firing-correctness ("did the closure actually get invoked?") is
deferred to a future window-opening integration suite per spec §11.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: Full verification — build, clippy, fmt, rustdoc, mdbook

**Files:** none (verification only)

- [ ] **Step 1: Full lib tests pass (Tier-1)**

Run: `cargo test -p raylib --lib --features full 2>&1 | tail -10`
Expected: every existing test still passes. Mixed-audio's Tier-2 tests are gated on `software_renderer`, so they don't show up here — expect the same count as before this workstream started (72 from the hashes workstream).

- [ ] **Step 2: Full lib tests pass (Tier-2 software_renderer)**

Run:
```bash
cargo test -p raylib --lib --no-default-features --features software_renderer,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RAUDIO,SUPPORT_FILEFORMAT_PNG -- --test-threads=1 2>&1 | tail -10
```
Expected: Tier-2 test count goes up by 2 (`attach_then_drop_does_not_panic` + `multiple_processors_attach_and_drop_independently`).

- [ ] **Step 3: Clippy clean**

Run: `cargo clippy -p raylib --features full -- -D warnings 2>&1 | tail -5`
Expected: no warnings.

Also verify under software_renderer feature:

Run: `cargo clippy -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RAUDIO,SUPPORT_FILEFORMAT_PNG --tests -- -D warnings 2>&1 | tail -5`
Expected: no warnings on the test code either.

- [ ] **Step 4: Rustfmt clean**

Run: `cargo fmt --check 2>&1 | tail -3`
Expected: zero output, exit 0.

- [ ] **Step 5: Rustdoc clean**

Run: `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps 2>&1 | tail -10`
Expected: clean build.

- [ ] **Step 6: Mdbook still builds**

Run: `mdbook build book 2>&1 | tail -5`
Expected: clean (no errors). The mixed-audio module isn't referenced from any book chapter; this just confirms nothing leaked.

---

### Task 7: Reconcile cheatsheet-parity-audit + CHANGELOG

**Files:**
- Modify: `docs/superpowers/notes/cheatsheet-parity-audit.md`
- Modify: `CHANGELOG.md`

The audit doc's §1 and §4 reference `AttachAudioMixedProcessor` and `DetachAudioMixedProcessor` as 🟥 GAP entries belonging to the `mixed-audio` workstream. Flip them to ✅ pointing at the new code paths; decrement §0 counts; remove the workstream entry from §4.

- [ ] **Step 1: Find the two entries in §1**

Run: `grep -nE 'AttachAudioMixedProcessor|DetachAudioMixedProcessor' docs/superpowers/notes/cheatsheet-parity-audit.md | head -10`

Expected: 🟥 GAP entries in §1 (currently pointing at the `mixed-audio` workstream), plus references in §4 (the workstream entry itself) and possibly the §0 summary or §4 "Done" subsection.

- [ ] **Step 2: Flip the §1 entries from 🟥 to ✅**

For each `Attach`/`DetachAudioMixedProcessor` line in §1, change the marker from `🟥` to `✅` and the action from "GAP. Action: future-workstream-mixed-audio." to a path reference. The pattern matches the just-shipped pixel-pointers + hashes ✅ rows:

```
- ✅ `AttachAudioMixedProcessor` — wrapped at `raylib/src/core/callbacks.rs::attach_audio_mixed_processor`.
- ✅ `DetachAudioMixedProcessor` — wrapped via `MixedAudioProcessorCallback::Drop` in `raylib/src/core/callbacks.rs`.
```

The exact original wording may differ; adapt to whatever ✅-style is used elsewhere in §1.

- [ ] **Step 3: Update §0 summary counts**

§0 currently shows (post-hashes): "Covered: 492 / Gaps: 7 / Medium workstreams: 2 (mixed-audio + Done section)". Decrement:
- Covered: 492 → 494 (+2).
- Genuine gaps: 7 → 5 (−2).
- Medium workstreams: 2 → 1 (mixed-audio removed; only the Done section remains).

Adjust the "X, not 49" headline accordingly: 7 − 2 = 5.

- [ ] **Step 4: Remove or move the `mixed-audio` workstream from §4**

The §4 entry for `mixed-audio` is done. Match whichever pattern the previous workstreams used (per pixel-pointers/hashes precedent: either remove + renumber OR move to the "Done (this audit)" subsection with a brief commit-range note). Default to "move to Done" since that's what hashes did.

If the §4 entry has a subsection like "Done (this audit)", append a new bullet under it:

```markdown
- **mixed-audio** ✅ — `MixedAudioProcessorCallback` + `attach_audio_mixed_processor` at `raylib/src/core/callbacks.rs`. RAII guard borrows `&'a RaylibAudio`; Drop detaches before clearing the slot. Tier-2 lifecycle tests under `software_renderer`.
```

And remove the active `mixed-audio` workstream entry above.

- [ ] **Step 5: Append CHANGELOG entries**

In `CHANGELOG.md`, under `## 6.0.0-rc.1 (unreleased)` → `### Added`, append:

```markdown
- **Mixed audio bus:** new public API for closure-driven processors on
  raylib's global mixed audio bus:
  - `attach_audio_mixed_processor(audio: &RaylibAudio, processor: &mut F) -> Pin<Box<MixedAudioProcessorCallback<'_, F>>>`
    where `F: FnMut(&mut [f32], u32) + Send + 'static`.
  - `MixedAudioProcessorCallback` — the RAII guard type. Dropping it
    calls `DetachAudioMixedProcessor` and frees the closure slot,
    mirroring the WS8e per-stream-processor soundness fix.
  - Multiple mixed-bus processors can be attached simultaneously
    (they chain in raylib's internal linked list). They share the
    same 30-slot trampoline pool with the per-stream processors.
  - Re-exported via `raylib::prelude`. Closes the `mixed-audio`
    workstream from the cheatsheet-parity audit.

  Also updates the "post-WS8 workstreams" note at the top of the
  6.0.0-rc.1 block: mixed-audio drops out of the list.
```

Then update the lead-paragraph note at the top of the `## 6.0.0-rc.1 (unreleased)` block:

Find the line that lists `(pixel-pointers, hashes, mixed-audio, WS9 showcase)` and remove `mixed-audio` from it. Result:

```
> Release candidate 1 for the 6.0 line — the published crate versions are
> `6.0.0-rc.1` while the post-WS8 workstreams (WS9 showcase ...) settle.
```

Or whatever phrasing keeps the remaining list grammatical (the three earlier workstreams are now all done; the lead note can just point at the remaining WS9 + final-release work).

- [ ] **Step 6: Spot-check the doc edits**

Run: `grep -nE 'AttachAudioMixedProcessor|DetachAudioMixedProcessor' docs/superpowers/notes/cheatsheet-parity-audit.md`
Expected: every match is ✅ (in §1 or §4 Done subsection); no 🟥 rows.

Run: `grep -n 'attach_audio_mixed_processor\|MixedAudioProcessorCallback' CHANGELOG.md | head -5`
Expected: the new bullet shows in the 6.0.0-rc.1 block.

- [ ] **Step 7: Commit Task 7**

```bash
git add docs/superpowers/notes/cheatsheet-parity-audit.md CHANGELOG.md
git commit -m "$(cat <<'EOF'
docs(mixed-audio): mark AttachAudioMixedProcessor / DetachAudioMixedProcessor as covered

The mixed-audio workstream from cheatsheet-parity-audit.md §4 is
done; reconcile the audit + CHANGELOG accordingly.

- cheatsheet-parity-audit.md:
  - §1 entries for AttachAudioMixedProcessor + DetachAudioMixedProcessor
    flip 🟥 -> ✅ pointing at raylib/src/core/callbacks.rs (the attach
    fn + MixedAudioProcessorCallback's Drop respectively).
  - §0 totals: Covered 492 -> 494, gaps 7 -> 5, medium workstreams 2 -> 1.
  - §4 mixed-audio workstream moved to the "Done (this audit)"
    subsection.
- CHANGELOG.md ## 6.0.0-rc.1 (unreleased) ### Added: lists the new
  attach_audio_mixed_processor + MixedAudioProcessorCallback with a
  reference to the WS8e per-stream-processor soundness pattern.
  Lead-paragraph note drops mixed-audio from the post-WS8 workstreams
  list (now empty for this group; only WS9 + final-release remain).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 8: CLAUDE.md status flip + final push

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Confirm clean working tree**

Run: `git status --short`
Expected: only the three permitted untracked files (`TODO.md`, `prompt.md`, `next-session-prompt.md`); no `M` lines.

- [ ] **Step 2: Update CLAUDE.md status line (pre-WS9 queue head: mixed-audio ✅, raylib-test ← NEXT)**

Find the status line:

Run: `grep -n 'mixed-audio ← NEXT\|hashes ✅' CLAUDE.md`

Expected: the current line shows `pixel-pointers ✅ → hashes ✅ → mixed-audio ← NEXT → raylib-test → ...`. Replace with:

```
pixel-pointers ✅ → hashes ✅ → mixed-audio ✅ → raylib-test ← NEXT → ...
```

(Preserve the rest of the chain — `UBSAN → rustdoc rewrite → safe abstractions → WS9 showcase → final-release`.)

- [ ] **Step 3: Commit + push**

```bash
git add CLAUDE.md
git commit -m "$(cat <<'EOF'
docs(roadmap): mixed-audio complete; raylib-test is next

CLAUDE.md status line: pre-WS9 queue head flips from mixed-audio to
raylib-test. See docs/superpowers/notes/cheatsheet-parity-audit.md
for the updated cheatsheet coverage (494 / 500 in-scope; 5 gaps
remain in the rtext/rfilesystem out-of-scope sections).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc 2>&1 | tail -3
git push fork 6.0-rc:unstable 2>&1 | tail -3
```

Expected: both pushes succeed with the same SHA range.

---

## mixed-audio complete when

- [ ] `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs` has `pub(crate)` on the three slot primitives + the two new `pub(crate)` mixed-bus helpers + the updated module-level doc comment.
- [ ] `raylib/src/core/callbacks.rs` has the `MixedAudioProcessorCallback` struct + impl + Drop + `attach_audio_mixed_processor` free fn + the two Tier-2 tests in `mod mixed_audio_tests`.
- [ ] `RaylibAudio` is imported in `callbacks.rs`.
- [ ] The two mixed-bus `pub(crate)` helpers are imported from the slot-pool file into `callbacks.rs`.
- [ ] Drop calls `DetachAudioMixedProcessor` BEFORE `clear_context` (lifecycle correctness).
- [ ] Tier-1 tests pass under `cargo test -p raylib --lib --features full`.
- [ ] Tier-2 tests pass under `--features software_renderer,...` with `-- --test-threads=1`.
- [ ] `cargo build --workspace --features full` clean.
- [ ] `cargo clippy --workspace --features full -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps` clean.
- [ ] `mdbook build book` clean.
- [ ] `cheatsheet-parity-audit.md` reconciled: §0 totals (494 covered / 5 gaps), §1 ✅ rows, §4 mixed-audio moved to "Done (this audit)".
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Added` lists the new APIs + lead-paragraph note updated.
- [ ] `CLAUDE.md` status line: pre-WS9 queue head flips `mixed-audio ✅ → raylib-test ← NEXT`.
- [ ] All commits include the `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>` trailer.
- [ ] Pushed to both `fork/6.0-rc` and `fork/unstable`.

Next workstream: **`raylib-test` delete-or-fix** (promoted from long-tail by owner directive 2026-05-29). Decide whether to fix the window-opening integration tests under xvfb or delete the crate entirely. Brainstorm starts when the owner is ready.
