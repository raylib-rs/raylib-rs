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

use paste::paste;
use raylib_sys::{AttachAudioStreamProcessor, AudioStream, DetachAudioStreamProcessor};
use seq_macro::seq;
use std::sync::{LazyLock, Mutex};

// region: -- AudioCallbackWithUserData --

/// This is the callback we wish to get from raylib:
/// It contains `user_data` in order to plug in our
/// context (e.g. our closure).
type RawAudioCallbackWithUserData = extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    data_ptr: *mut ::std::os::raw::c_void,
    frames: u32,
) -> ();

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
}

impl Default for AudioCallbackWithUserData {
    fn default() -> Self {
        AudioCallbackWithUserData {
            user_data: std::ptr::null_mut(),
            callback: None,
        }
    }
}

// endregion: -- AudioCallbackWithUserData --

// region: -- raw callbacks and linkage
// raw callback and linkage to AudioCallbackWithUserData
// we only support a limited amount of callbacks - since
// we need a dedicated callback function for each
// callback or closure we plug in. This is caused by the
// absence of a `user_data` context in the callbacks
// supported by raylib.

macro_rules! generate_functions {
  ( $( $n:literal ),* ) => {
    paste! {
        $(
            /// For each supported callback the data for our context.
            /// (here we have N "slots" with context data)
            static [< CLOSURE_ $n >]:  LazyLock<Mutex<AudioCallbackWithUserData>> = LazyLock::new(|| Mutex::new(AudioCallbackWithUserData::default()));
        )*

          /// Function to set our context
          /// and returns the slot used to store the context.
          #[allow(unpredictable_function_pointer_comparisons)]
          pub(crate) fn set_context(audio_callback: AudioCallbackWithUserData) -> usize {
              $(
                  {
                      let mut guard = [< CLOSURE_ $n >].lock().unwrap();
                      if (*guard).callback == None {
                        *guard = audio_callback;
                        return $n;
                      }
                  }
              )*
              panic!("index out of bounds");
          }

          /// Function to clear our context given the slot of the context.
          #[allow(unpredictable_function_pointer_comparisons)]
          pub(crate) fn clear_context(index: usize) {
              $(
                  if index == $n {
                      let mut guard = [< CLOSURE_ $n >].lock().unwrap();
                      if (*guard).callback == None {
                          panic!(
                              "No callbacks registered under this number ({}).",
                              index
                          );
                      }
                      *guard = AudioCallbackWithUserData::default();
                      return;
                  }
              )*
              panic!("clear_context: index {} out of bounds", index);
          }

          $(
            /// The real callback passed to raylib.
            /// Each callback has a fixed association with
            /// a given context "slot".
            #[unsafe(no_mangle)]
            pub extern "C" fn [< callback_ $n >](data_ptr: *mut ::std::os::raw::c_void, frames: u32) -> () {
              let guard = [< CLOSURE_ $n >].lock().unwrap();
              let audio_callback = &(*guard);
              if let Some(callback) = audio_callback.callback {
                (callback)(audio_callback.user_data, data_ptr, frames);
              } else {
                  panic!("unexpected: no callback $n set")
              }
            }
          )*

          /// Function to get the callback for a given context
          /// given the slot of the context.
          pub(crate) fn get_callback(index: usize) -> extern "C" fn(data_ptr: *mut ::std::os::raw::c_void, frames: u32) {
            $(
                if index == $n {
                    return [< callback_ $n >];
                }
            )*
            panic!("get_callback: index out of bounds");
          }
        }
  }
}

// here, you can control how many callbacks are supported
seq!(I in 1..30 {
    generate_functions!( 0#(,I)* );
});

// endregion: -- raw callbacks and linkage

/// Here, we c
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
