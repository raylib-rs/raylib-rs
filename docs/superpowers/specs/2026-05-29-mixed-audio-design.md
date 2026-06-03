# mixed-audio — safe wrappers for `AttachAudioMixedProcessor` / `DetachAudioMixedProcessor`

**Status:** design approved 2026-05-29. Third pre-WS9 workstream in
the owner-locked queue (after the just-shipped `pixel-pointers` and
`hashes`).

raylib 6.0's `Attach`/`DetachAudioMixedProcessor` wire a closure into
raylib's global mixed audio bus — post-processing every stereo frame
after all playing streams are mixed. The cheatsheet parity audit
(`docs/superpowers/notes/cheatsheet-parity-audit.md` §4 #3) called these
out as a gap that needs a real RAII handle distinct from the per-stream
variant, since the lifetime story is different: the mixed-bus
processor is global to the audio device, not tied to a single
`Music` or `AudioStream`. This spec mirrors the per-stream wrapper's
shape (just hardened in WS8e) and shares its 30-slot trampoline
machinery.

## 1. Goals

1. Wrap `AttachAudioMixedProcessor` / `DetachAudioMixedProcessor` with
   a safe RAII guard (`MixedAudioProcessorCallback`) that:
   - Accepts a `FnMut(&mut [f32], u32) + Send + 'static` closure
     (matches the per-stream signature so the existing trampoline
     pool just works).
   - Borrows `&'a RaylibAudio` so the audio device cannot close while
     a processor is still attached.
   - Detaches on `Drop` — calling `DetachAudioMixedProcessor` BEFORE
     clearing the slot, mirroring the WS8e per-stream soundness fix.
2. Share the existing 30-slot trampoline pool from
   `stream_processor_with_user_data_wrapper.rs` with the new mixed-bus
   consumer.
3. Document the shared-pool exhaustion ceiling (31st attach panics)
   so users know when to drop existing processors.
4. Tier-2 lifecycle tests via `test_harness::with_headless` +
   explicit `RaylibAudio::init_audio_device()` cover attach + drop +
   multiple-attach without panicking or leaking slots.

Done-criteria are in §10.

## 2. Non-goals

- **No "did the callback fire" assertion in tests.** Mixed-bus
  callbacks fire from raylib's internal audio thread, asynchronously,
  only when there's audio to mix. A firing-test requires real audio
  playback + cross-thread counters + a sleep window, which is fragile
  on CI. The lifecycle tests cover the attach/detach correctness;
  firing-correctness is deferred to a future window-opening
  integration test (or a manual `#[ignore]`'d test).
- **No dedicated 30-slot pool for mixed-audio.** The shared pool is
  the cheap path; users running into the 31-slot ceiling can drop
  unused processors to free slots. A future workstream can add a
  larger pool or a dynamic-allocation pool if real users hit the
  ceiling.
- **No method form** (`audio.attach_mixed_processor(...)`). Free fn
  matches the per-stream convention.
- **No new dependencies.** Reuses existing `paste` + `seq_macro`
  machinery in `stream_processor_with_user_data_wrapper.rs`.
- **No file rename.** The slot-pool file
  (`stream_processor_with_user_data_wrapper.rs`) keeps its legacy
  name; a header comment notes that it serves both per-stream and
  mixed-bus consumers now. Rename is incidental scope creep best left
  to a future cleanup.

## 3. Locked decisions (owner-confirmed during brainstorm 2026-05-29)

| # | Decision | Resolution |
|---|----------|------------|
| D1 | User-facing API shape | **Free function** `attach_audio_mixed_processor(audio, processor)` mirroring `attach_audio_stream_processor_to_music`. Lives in `raylib/src/core/callbacks.rs` alongside the per-stream sibling. |
| D2 | Closure signature | `F: FnMut(&mut [f32], u32) + Send + 'static` — matches per-stream verbatim so the existing trampoline pool works without modification. The `u32` channel count is always `2` for mixed-audio; documented but redundant. |
| D3 | Slot pool | **Shared** with the per-stream pool. Mixed-audio attaches reserve one of the existing 30 slots; the 31st attach (per-stream + mixed combined) panics. Documented in rustdoc. |
| D4 | File rename | **No rename.** Slot-pool file stays at `stream_processor_with_user_data_wrapper.rs` with an updated header comment explaining the broader role. |
| D5 | Slot-primitive visibility | `pub(crate)` for `set_context` / `clear_context` / `get_callback`. Mixed-audio's new attach/detach helpers stay `pub(crate)` too (only the user-facing `attach_audio_mixed_processor` is `pub`). |
| D6 | `&RaylibThread` parameter | **Not required.** raylib's `AttachAudioMixedProcessor` / `DetachAudioMixedProcessor` are mutex-guarded internally (`ma_mutex_lock(&AUDIO.System.lock)`); concurrent calls from arbitrary threads are safe at the C level. |
| D7 | Test strategy | Lifecycle-only Tier-2 tests via `test_harness::with_headless` + `RaylibAudio::init_audio_device()`. Two tests: single attach-and-drop, multiple-attach-with-out-of-order drops. No firing-correctness test. |
| D8 | Slot-pool exhaustion handling | Panic with clear message on the 31st attach. The existing pool already panics via `panic!("index out of bounds")`; rustdoc on `attach_audio_mixed_processor` calls out the 30-slot ceiling. |

## 4. File structure

```
raylib/src/core/
├── callbacks.rs                                 # MODIFY — add MixedAudioProcessorCallback struct + attach_audio_mixed_processor fn alongside AudioStreamProcessorCallback
└── callbacks/
    └── stream_processor_with_user_data_wrapper.rs  # MODIFY — bump slot-primitives to pub(crate), add attach_audio_mixed_processor_with_user_data / detach_audio_mixed_processor_with_user_data helpers, add header comment about the broader role
```

No new files. Two existing files modified.

Total LOC added: ~80 in `callbacks.rs` (struct + impl + Drop + attach helper) + ~25 in the slot-pool file (two helpers + a header comment + visibility tweaks). Tests in a `#[cfg(test)] mod` at the bottom of `callbacks.rs` (or a `#[cfg(test)] mod mixed_audio_tests` to keep the existing test module clean): ~50 lines.

## 5. Public API

```rust
// raylib::core::callbacks  (additions to the existing file)

use std::os::raw::{c_uint, c_void};
use std::pin::Pin;
use crate::core::RaylibAudio;

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
        Self { rust_callback: closure, callback_index: None }
    }

    fn get_as_user_data(&mut self) -> *mut c_void {
        self as *mut Self as *mut c_void
    }

    fn get_c_callback(
        &mut self,
    ) -> extern "C" fn(*mut c_void, *mut c_void, c_uint) -> () {
        Self::c_callback
    }

    extern "C" fn c_callback(
        user_data: *mut c_void,
        data_ptr: *mut c_void,
        frame_count: c_uint,
    ) {
        // SAFETY: user_data is `*mut Self` per get_as_user_data; the wrapping
        // Pin<Box<Self>> keeps it stable for the guard's lifetime, which
        // outlives every callback firing per the &'a RaylibAudio borrow.
        // data_ptr is `frame_count * 2 * sizeof(f32)` interleaved stereo bytes
        // (mixed bus is always stereo per raylib.h:1736).
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
///   interleaved stereo samples plus the channel count (`2`).
///   Modifying the slice in-place applies the effect to the mixed
///   bus.
///
/// # Slot pool exhaustion
///
/// The crate supports up to **30 simultaneous** audio-processor
/// closures (shared pool across per-stream and mixed-bus
/// consumers — see
/// [`attach_audio_stream_processor_to_music`]). Attaching a 31st
/// processor panics. Detach existing processors (drop their guards)
/// to free slots.
///
/// # Thread safety
///
/// raylib's `AttachAudioMixedProcessor` / `DetachAudioMixedProcessor`
/// are internally mutex-guarded; attach and drop are safe to call
/// from any thread.
pub fn attach_audio_mixed_processor<'a, F>(
    _audio: &'a RaylibAudio,
    processor: &'a mut F,
) -> Pin<Box<MixedAudioProcessorCallback<'a, F>>>
where
    F: FnMut(&mut [f32], u32) + Send + 'static,
{
    let mut cb = Box::new(MixedAudioProcessorCallback::<'a, F>::new(processor));
    let idx = attach_audio_mixed_processor_with_user_data(
        AudioCallbackWithUserData::new(
            cb.get_as_user_data(),
            cb.get_c_callback(),
        ),
    );
    cb.callback_index = Some(idx);
    Box::into_pin(cb)
}
```

## 6. Slot-pool helper additions

In `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`:

### Header comment update

Replace the existing top-of-file comments with:

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

### Visibility bumps

`set_context` / `clear_context` / `get_callback` change from private to `pub(crate)`.

### New helpers

```rust
pub(crate) fn attach_audio_mixed_processor_with_user_data(
    callback: AudioCallbackWithUserData,
) -> usize {
    let idx = set_context(callback);
    // SAFETY: get_callback(idx) returns the trampoline matching the slot
    // we just reserved; raylib copies the fn pointer into its linked-list
    // entry. We hold the slot until drop calls detach + clear_context.
    unsafe {
        raylib_sys::AttachAudioMixedProcessor(Some(get_callback(idx)));
    }
    idx
}

pub(crate) fn detach_audio_mixed_processor_with_user_data(index: usize) {
    let trampoline = get_callback(index);
    // SAFETY: trampoline is the same fn pointer raylib stored at attach
    // time; the C-side list-search matches and removes the entry.
    // clear_context runs AFTER detach completes so raylib has stopped
    // iterating before the closure context disappears.
    unsafe {
        raylib_sys::DetachAudioMixedProcessor(Some(trampoline));
    }
    clear_context(index);
}
```

The pattern mirrors the existing per-stream helpers verbatim — only the FFI calls differ (no `AudioStream` argument; mixed-bus is global).

## 7. Lifecycle correctness

The detach-before-clear ordering is load-bearing for soundness (same as the WS8e per-stream fix):

1. `Drop` runs on `MixedAudioProcessorCallback`.
2. `detach_audio_mixed_processor_with_user_data(idx)` calls
   `ffi::DetachAudioMixedProcessor(Some(trampoline))` first.
3. Inside that FFI call, raylib's mutex-guarded list-walk removes
   the entry. After this returns, raylib has stopped iterating the
   trampoline.
4. THEN `clear_context(idx)` empties the slot.

If the order were reversed (clear first, detach second), there's a
window where raylib could iterate the now-empty slot and the
trampoline would `panic!("unexpected: no callback set")` (per the
existing `stream_processor_with_user_data_wrapper.rs:115`).

## 8. Testing strategy

Two Tier-2 tests (gated on `feature = "software_renderer"`) in a new
`#[cfg(test)] mod mixed_audio_tests` block at the bottom of
`callbacks.rs`. Both open a 1×1 software-renderer window via
`test_harness::with_headless` and explicitly call
`RaylibAudio::init_audio_device()` to get an `AudioHandle`.

### Test 1: `attach_then_drop_does_not_panic`

```rust
#[cfg(feature = "software_renderer")]
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
```

**Proves**: attach reserves a slot, drop calls `DetachAudioMixedProcessor` + `clear_context` without panicking.

### Test 2: `multiple_processors_attach_and_drop_independently`

```rust
#[cfg(feature = "software_renderer")]
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
```

**Proves**: multiple simultaneous attachments + out-of-order drops don't corrupt the slot pool or raylib's linked list.

### Why no firing test

Validating "the callback fired" requires:
- Loading or generating real audio (Wave/Music/AudioStream).
- Playing it through the mixed bus.
- Synchronizing with raylib's audio thread (likely an `Arc<AtomicUsize>` counter shared into the closure).
- A controlled sleep window.

This is brittle on CI (audio device may be virtual or absent), so the firing-correctness test belongs in a future window-opening integration suite (likely the `raylib-test` crate after its delete-or-fix workstream lands). Tracked in §11.

### Test infrastructure

Tests need `--test-threads=1` to avoid raylib's single-init lock — the same constraint that applies to `hashes`' Tier-2 tests. The CI invocation already passes this for the `software_renderer` test matrix; no new infrastructure needed.

## 9. Documentation

### Module-level

`raylib/src/core/callbacks.rs` already has prose for `AudioStreamProcessorCallback`. Extend the module-level doc with a "Mixed audio bus" subsection that:
- Distinguishes per-stream vs. mixed-bus use cases (per-stream = effect on one Music/AudioStream; mixed-bus = effect on the entire mix after raylib combines all streams).
- Points at `MixedAudioProcessorCallback` + `attach_audio_mixed_processor`.
- Notes the shared 30-slot pool and the chain semantics for multiple mixed-bus processors.

### Per-function

`attach_audio_mixed_processor` rustdoc per §5 above:
- "Closure constraints" (Send + 'static + the channel count contract).
- "Slot pool exhaustion" (30-slot ceiling, shared with per-stream).
- "Thread safety" (C side is mutex-guarded; attach/drop are thread-safe).

`MixedAudioProcessorCallback` struct doc per §5 (it's a guard type; users obtain one via the attach fn, not by direct construction).

## 10. Done-criteria

WS mixed-audio is complete when **all** of:

- [ ] `raylib/src/core/callbacks.rs` gains the `MixedAudioProcessorCallback`
      struct + its `impl` + `Drop` + the `attach_audio_mixed_processor`
      free function from §5.
- [ ] `raylib/src/core/callbacks/stream_processor_with_user_data_wrapper.rs`
      gains the two `pub(crate)` helpers
      (`attach_audio_mixed_processor_with_user_data`,
      `detach_audio_mixed_processor_with_user_data`), the visibility
      bumps for the slot primitives, and the updated header comment.
- [ ] Drop calls `DetachAudioMixedProcessor` BEFORE `clear_context`
      (lifecycle correctness per §7).
- [ ] `attach_audio_mixed_processor` borrows `&'a RaylibAudio` —
      the audio device cannot drop while a processor is attached.
- [ ] Two Tier-2 tests pass under
      `cargo test -p raylib --no-default-features --features software_renderer,...`
      with `--test-threads=1`:
      `attach_then_drop_does_not_panic` and
      `multiple_processors_attach_and_drop_independently`.
- [ ] `cargo build --workspace --features full` clean.
- [ ] `cargo clippy --workspace --features full -- -D warnings` clean.
- [ ] `cargo fmt --check` clean.
- [ ] `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps` clean.
- [ ] Module-level rustdoc + per-fn rustdoc per §9.
- [ ] `cheatsheet-parity-audit.md` reconciled: move
      `AttachAudioMixedProcessor` + `DetachAudioMixedProcessor` from
      🟥 GAP / §4 workstream to ✅ in §1; decrement §0 gap counts
      (7 → 5); remove the `mixed-audio` entry from §4 (or move to the
      "Done" subsection).
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Added` gains
      the new public items (`MixedAudioProcessorCallback`,
      `attach_audio_mixed_processor`).
- [ ] `CLAUDE.md` status line: pre-WS9 queue head flips from
      `mixed-audio` to `raylib-test`.
- [ ] Commits on `6.0-rc` with the `Co-Authored-By: Claude Opus 4.7`
      trailer; pushed to `fork/6.0-rc` and `fork/unstable`.

## 11. Risks + mitigations

1. **Slot-pool exhaustion** in tests or user code. Mitigated by the
   30-slot ceiling being documented + the slot-pool being shared with
   per-stream consumers. If a user hits 30, they get a clear panic.
   A future workstream can lift the ceiling if real users hit it.
2. **Audio device unavailable on CI** when
   `RaylibAudio::init_audio_device()` is called in tests. The init
   call may fail on a headless runner without an audio output device.
   Mitigation: the tests use `.expect("audio init")`; if the
   environment can't init audio, the tests fail loudly rather than
   silently passing. If this is a problem on CI, the tests can be
   gated further (e.g. `#[cfg(target_os = "...")]`) — discovered
   during execution.
3. **The mixed-bus C-side list-walk** in `DetachAudioMixedProcessor`
   could fail to match if the trampoline pointer raylib stored
   differs from the one our `get_callback(index)` returns. The
   30-slot trampolines are `#[no_mangle]` static fns whose addresses
   are program-lifetime-stable, so this should hold. If it ever
   breaks, the symptom is "detach doesn't actually remove the entry"
   — the test cases would catch it because subsequent attaches would
   pile up indefinitely.
4. **Pinning correctness**: the `Pin<Box<MixedAudioProcessorCallback>>`
   stores `*mut Self` as the trampoline's user-data. Moving the
   struct would invalidate the pointer. `Box::into_pin` + the lack
   of `Unpin` for `Self` enforces this at compile time; the per-stream
   variant uses the same pattern (verified safe in WS8e).

## 12. Out-of-scope follow-ups (logged for later)

- **Firing-correctness test** for the mixed-bus callback. Requires
  real audio playback + cross-thread counter + a tracked sleep
  window. Most natural home is the `raylib-test` crate after its
  pending delete-or-fix workstream lands.
- **Larger slot pool** — current 30-slot ceiling is fine for typical
  use but a future workstream could lift it (e.g. dynamic
  allocation, or a separate pool per consumer type).
- **File rename** for `stream_processor_with_user_data_wrapper.rs` to
  reflect its broader role (now covers both per-stream and mixed-bus
  consumers). Incidental cleanup; tracked-deferred.
- **Method-form API mirrors** for both per-stream and mixed-bus
  attach functions (`music.attach_processor(closure)`,
  `audio.attach_mixed_processor(closure)`). Would be a small
  ergonomic win; out of scope here.
- **RaylibThread audit follow-through** — the `RaylibThread` audit
  workstream (TodoList #27, queued post-mixed-audio) should
  re-evaluate whether the mixed-bus attach/detach helpers benefit
  from a thread witness despite the C-side mutex (defense-in-depth
  argument). Currently not required per D6; revisit if the audit
  surfaces new patterns.
