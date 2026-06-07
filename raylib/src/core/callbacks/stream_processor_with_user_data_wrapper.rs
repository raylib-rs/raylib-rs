//! Shared trampoline-slot pool for closure-driven audio callbacks.
//!
//! raylib's C-side audio processors (`AttachAudioStreamProcessor`,
//! `AttachAudioMixedProcessor`) accept only a function pointer — no
//! user-data parameter. To thread closure state through, we
//! pre-register **30 trampolines** (`trampoline::<0>` through
//! `trampoline::<29>`), each reading its own slot in the [`CLOSURES`]
//! pool. A consumer reserves a free slot via [`set_context`], the
//! trampoline for that slot looks up the closure context, and
//! [`clear_context`] frees the slot.
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

use raylib_sys::{AttachAudioStreamProcessor, AudioStream, DetachAudioStreamProcessor};
use seq_macro::seq;
use std::sync::Mutex;

// region: -- AudioCallbackWithUserData --

/// This is the callback we wish to get from raylib:
/// It contains `user_data` in order to plug in our
/// context (e.g. our closure).
type RawAudioCallbackWithUserData = extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    data_ptr: *mut ::std::os::raw::c_void,
    frames: u32,
) -> ();

/// The signature raylib actually accepts: no user-data parameter.
/// Each pool slot owns one trampoline of this shape.
type RawAudioCallback = extern "C" fn(data_ptr: *mut ::std::os::raw::c_void, frames: u32);

/// This is a tuple of `user_data` which represents
/// our context (see RawAudioCallbackWithUserData)
/// and the callback we wish to pass to our raylib
/// abstraction layer (wrapping the real raylib
/// callback to plug in our context).
pub struct AudioCallbackWithUserData {
    user_data: *mut ::std::os::raw::c_void,
    callback: Option<RawAudioCallbackWithUserData>,
}

unsafe impl Send for AudioCallbackWithUserData {} //??

impl AudioCallbackWithUserData {
    pub fn new(
        user_data: *mut ::std::os::raw::c_void,
        raw_callback: RawAudioCallbackWithUserData,
    ) -> Self {
        AudioCallbackWithUserData {
            user_data,
            callback: Some(raw_callback),
        }
    }

    /// An empty slot value: no callback registered, null user data.
    /// `const` so the slot pool can be built in a `static` initializer.
    const fn empty() -> Self {
        AudioCallbackWithUserData {
            user_data: std::ptr::null_mut(),
            callback: None,
        }
    }
}

impl Default for AudioCallbackWithUserData {
    fn default() -> Self {
        Self::empty()
    }
}

// endregion: -- AudioCallbackWithUserData --

// region: -- trampoline pool
// We only support a limited number of callbacks, since raylib's
// callback signatures carry no `user_data` pointer: every registered
// closure needs a dedicated `extern "C" fn` whose identity encodes
// which context slot to read.

/// Number of trampoline slots in the shared pool.
/// Keep the `seq!` range building [`TRAMPOLINES`] in sync — the array
/// type makes a mismatch a compile error.
const SLOTS: usize = 30;

/// One context slot per trampoline; `trampoline::<N>` reads `CLOSURES[N]`.
static CLOSURES: [Mutex<AudioCallbackWithUserData>; SLOTS] =
    [const { Mutex::new(AudioCallbackWithUserData::empty()) }; SLOTS];

/// The real callback passed to raylib. Each monomorphization has a
/// fixed association with one context slot. The slot guard is held
/// while the user callback runs, so `clear_context` cannot free a
/// context out from under an in-flight invocation.
///
/// (A fn pointer needs no `no_mangle`/`pub`: raylib stores and calls
/// the pointer we hand it at attach time.)
extern "C" fn trampoline<const N: usize>(data_ptr: *mut ::std::os::raw::c_void, frames: u32) {
    let guard = CLOSURES[N].lock().unwrap();
    if let Some(callback) = guard.callback {
        (callback)(guard.user_data, data_ptr, frames);
    } else {
        panic!("unexpected: no callback {N} set");
    }
}

/// Per-slot trampoline fn pointers, indexable by slot id.
static TRAMPOLINES: [RawAudioCallback; SLOTS] = seq!(N in 0..30 {
    [
        #(
            trampoline::<N>,
        )*
    ]
});

/// Reserve the first free slot, store `audio_callback` there, and
/// return the slot index. Panics when all [`SLOTS`] slots are taken.
pub(crate) fn set_context(audio_callback: AudioCallbackWithUserData) -> usize {
    for (index, slot) in CLOSURES.iter().enumerate() {
        let mut guard = slot.lock().unwrap();
        if guard.callback.is_none() {
            *guard = audio_callback;
            return index;
        }
    }
    panic!("no free audio callback slot (max {SLOTS})");
}

/// Clear the context stored at `index`, freeing the slot.
/// Panics if the index is out of bounds or the slot is already empty.
pub(crate) fn clear_context(index: usize) {
    let Some(slot) = CLOSURES.get(index) else {
        panic!("clear_context: index {index} out of bounds");
    };
    let mut guard = slot.lock().unwrap();
    if guard.callback.is_none() {
        panic!("No callbacks registered under this number ({index}).");
    }
    *guard = AudioCallbackWithUserData::empty();
}

/// The trampoline associated with the context slot `index`.
/// Panics if the index is out of bounds.
pub(crate) fn get_callback(index: usize) -> RawAudioCallback {
    let Some(&trampoline) = TRAMPOLINES.get(index) else {
        panic!("get_callback: index {index} out of bounds");
    };
    trampoline
}

// endregion: -- trampoline pool

/// Attach a closure-driven processor to `stream`. Reserves a slot from
/// the shared pool and calls the C-side `AttachAudioStreamProcessor`
/// with that slot's trampoline. Returns the slot index — pass it to
/// [`detach_audio_stream_processor_with_user_data`] when done.
pub fn attach_audio_stream_processor_with_user_data(
    stream: AudioStream,
    callback: AudioCallbackWithUserData,
) -> usize {
    let idx = set_context(callback);
    unsafe {
        AttachAudioStreamProcessor(stream, Some(get_callback(idx)));
    }
    idx
}

/// Detach the closure-driven stream processor and clear the slot.
///
/// Calls the C-side `DetachAudioStreamProcessor` with the same trampoline
/// pointer that was registered for `index`, so raylib's internal processor
/// list-search matches and the entry is actually removed. The slot is only
/// cleared after the C side has stopped iterating it — without this the
/// closure could still receive one more invocation against freed state.
pub fn detach_audio_stream_processor_with_user_data(stream: AudioStream, index: usize) {
    let trampoline = get_callback(index);
    unsafe {
        DetachAudioStreamProcessor(stream, Some(trampoline));
    }
    clear_context(index);
}

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

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn test_cb(
        _user_data: *mut ::std::os::raw::c_void,
        _data_ptr: *mut ::std::os::raw::c_void,
        _frames: u32,
    ) {
    }

    /// One combined test on purpose: the pool is a process-global
    /// shared by every test in this binary, so parallel tests would
    /// race for slots (and a `should_panic` test would poison a slot
    /// mutex for everyone else).
    #[test]
    fn slot_pool_reserve_reuse_and_distinct_trampolines() {
        // Fill every slot; indices must be distinct and in-bounds.
        let indices: Vec<usize> = (0..SLOTS)
            .map(|_| {
                set_context(AudioCallbackWithUserData::new(
                    std::ptr::null_mut(),
                    test_cb,
                ))
            })
            .collect();
        let mut sorted = indices.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), SLOTS, "slot indices must be distinct");
        assert!(sorted.iter().all(|&i| i < SLOTS));

        // Each slot must map to its own trampoline (raylib's detach
        // does identity-matching on the fn pointer).
        let mut ptrs: Vec<usize> = (0..SLOTS).map(|i| get_callback(i) as usize).collect();
        ptrs.sort_unstable();
        ptrs.dedup();
        assert_eq!(ptrs.len(), SLOTS, "each slot needs a distinct trampoline");

        // Clearing a middle slot makes it the next one reserved
        // (first-free-wins ordering).
        clear_context(7);
        assert_eq!(
            set_context(AudioCallbackWithUserData::new(
                std::ptr::null_mut(),
                test_cb
            )),
            7
        );

        // Leave the pool empty for any future test in this binary.
        for i in 0..SLOTS {
            clear_context(i);
        }
    }
}
