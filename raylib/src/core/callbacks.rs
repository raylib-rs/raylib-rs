#![allow(non_camel_case_types)]

use crate::core::error::SetCallbackError;
use crate::{RaylibHandle, ffi};
pub use raylib_sys::TraceLogLevel;
use std::{
    borrow::Cow,
    convert::TryInto,
    ffi::{CStr, CString, c_char, c_int, c_void},
    mem::{size_of, transmute},
    pin::Pin,
    ptr::null_mut,
    slice::from_raw_parts_mut,
    sync::atomic::{AtomicUsize, Ordering},
};
/// Low-level per-[`AudioStream`](super::audio::AudioStream) callback registration.
///
/// Wraps raylib's `SetAudioStreamCallback`: a single global callback slot per process feeds
/// raw PCM bytes (`u8` / `i16` / `f32` per the stream's sample size) into an
/// [`AudioStream`](super::audio::AudioStream) on raylib's audio thread. Prefer the
/// closure-based audio-mixed processor family ([`attach_audio_mixed_processor`]) for
/// effects on the global mix bus; use this module only when you need to **source** raw
/// samples for a specific stream.
///
/// Register with [`set_audio_stream_callback`](audio_stream_callback::set_audio_stream_callback)
/// and clear with [`unset_audio_stream_callback`](audio_stream_callback::unset_audio_stream_callback).
/// Only one callback may be set at a time — a second registration returns
/// [`UpdateAudioStreamError::CallbackSlotBusy`](crate::error::UpdateAudioStreamError::CallbackSlotBusy).
///
/// # See also
///
/// See the *Audio* chapter of the book.
pub mod audio_stream_callback;
mod stream_processor_with_user_data_wrapper;
use super::audio::{Music, RaylibAudio};
use stream_processor_with_user_data_wrapper::*;

type TraceLogCallback = unsafe extern "C" fn(*mut i8, *const i8, ...);
unsafe extern "C" {
    fn SetTraceLogCallback(cb: Option<TraceLogCallback>);
}

type RustTraceLogCallback = fn(TraceLogLevel, &str);
type RustSaveFileDataCallback = fn(&str, &[u8]) -> bool;
type RustLoadFileDataCallback = fn(&str) -> Vec<u8>;
type RustSaveFileTextCallback = fn(&str, &str) -> bool;
type RustLoadFileTextCallback = fn(&str) -> String;
static TRACE_LOG_CALLBACK: AtomicUsize = AtomicUsize::new(0);
static SAVE_FILE_DATA_CALLBACK: AtomicUsize = AtomicUsize::new(0);
static LOAD_FILE_DATA_CALLBACK: AtomicUsize = AtomicUsize::new(0);
static SAVE_FILE_TEXT_CALLBACK: AtomicUsize = AtomicUsize::new(0);
static LOAD_FILE_TEXT_CALLBACK: AtomicUsize = AtomicUsize::new(0);
fn trace_log_callback() -> Option<RustTraceLogCallback> {
    debug_assert!(size_of::<RustTraceLogCallback>() == size_of::<usize>());
    unsafe { transmute(TRACE_LOG_CALLBACK.load(Ordering::Relaxed)) }
}

fn save_file_data_callback() -> Option<RustSaveFileDataCallback> {
    debug_assert!(size_of::<RustSaveFileDataCallback>() == size_of::<usize>());
    unsafe { transmute(SAVE_FILE_DATA_CALLBACK.load(Ordering::Relaxed)) }
}

fn load_file_data_callback() -> Option<RustLoadFileDataCallback> {
    debug_assert!(size_of::<RustLoadFileDataCallback>() == size_of::<usize>());
    unsafe { transmute(LOAD_FILE_DATA_CALLBACK.load(Ordering::Relaxed)) }
}

fn save_file_text_callback() -> Option<RustSaveFileTextCallback> {
    debug_assert!(size_of::<RustSaveFileTextCallback>() == size_of::<usize>());
    unsafe { transmute(SAVE_FILE_TEXT_CALLBACK.load(Ordering::Relaxed)) }
}

fn load_file_text_callback() -> Option<RustLoadFileTextCallback> {
    debug_assert!(size_of::<RustLoadFileTextCallback>() == size_of::<usize>());
    unsafe { transmute(LOAD_FILE_TEXT_CALLBACK.load(Ordering::Relaxed)) }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `text` must be a valid C string pointer or null.
pub unsafe extern "C" fn custom_trace_log_callback(level: TraceLogLevel, text: *const c_char) {
    if let Some(trace_log) = trace_log_callback() {
        let text = if text.is_null() {
            Cow::Borrowed("(MESSAGE WAS NULL)")
        } else {
            unsafe { CStr::from_ptr(text).to_string_lossy() }
        };

        trace_log(level, &text)
    }
}

extern "C" fn custom_save_file_data_callback(
    path: *const c_char,
    buffer: *mut c_void,
    size: c_int,
) -> bool {
    let save_file_data = save_file_data_callback().expect("no callback");
    let path = unsafe { CStr::from_ptr(path) };
    let buffer = unsafe { from_raw_parts_mut(buffer as *mut u8, size as usize) };

    save_file_data(path.to_str().expect("path is non utf-8"), buffer)
}

extern "C" fn custom_load_file_data_callback(path: *const c_char, size: *mut c_int) -> *mut u8 {
    let load_file_data = load_file_data_callback().expect("no callback");

    if let Some(size) = unsafe { size.as_mut() } {
        let path = unsafe { CStr::from_ptr(path) };
        let buffer = load_file_data(path.to_str().expect("path is non utf-8"));
        *size = buffer.len().try_into().expect("out of range buffer size");

        // Copy everything to the raylib world
        unsafe {
            let buffer_ffi =
                ffi::MemAlloc((*size).try_into().expect("non representable buffer size"))
                    as *mut u8;
            buffer_ffi.copy_from_nonoverlapping(buffer.as_ptr(), buffer.len());

            buffer_ffi
        }
    } else {
        null_mut()
    }
}

extern "C" fn custom_save_file_text_callback(a: *const c_char, b: *const c_char) -> bool {
    let save_file_text = save_file_text_callback().unwrap();
    let a = unsafe { CStr::from_ptr(a) };
    let b = unsafe { CStr::from_ptr(b) };
    save_file_text(a.to_str().unwrap(), b.to_str().unwrap())
}
extern "C" fn custom_load_file_text_callback(a: *const c_char) -> *mut c_char {
    let load_file_text = load_file_text_callback().unwrap();
    let a = unsafe { CStr::from_ptr(a) };
    let st = load_file_text(a.to_str().unwrap());
    let oh = Box::leak(Box::new(CString::new(st).unwrap()));
    oh.as_ptr() as *mut c_char
}

macro_rules! safe_callback_set_func {
    ($cb:expr, $target_cb:expr, $rawsetter:expr, $ogfunc:expr, $ty:literal) => {
        if $target_cb.load(Ordering::Acquire) == 0 {
            $target_cb.store($cb as usize, Ordering::Release);
            unsafe { $rawsetter(Some($ogfunc)) };
            Ok(())
        } else {
            Err(SetCallbackError($ty))
        }
    };
}

/// Set custom trace log
pub fn set_trace_log_callback(cb: fn(TraceLogLevel, &str)) -> Result<(), SetCallbackError> {
    TRACE_LOG_CALLBACK.store(cb as usize, Ordering::Relaxed);
    #[cfg(not(feature = "nobuild"))]
    unsafe {
        ffi::setLogCallbackWrapper()
    };
    Ok(())
}
/// Set custom file binary data saver
pub fn set_save_file_data_callback(cb: fn(&str, &[u8]) -> bool) -> Result<(), SetCallbackError> {
    safe_callback_set_func!(
        cb,
        SAVE_FILE_DATA_CALLBACK,
        ffi::SetSaveFileDataCallback,
        custom_save_file_data_callback,
        "save file data"
    )
}
/// Set custom file binary data loader
///
/// Whatever you return from your callback will be intentionally leaked as Raylib is relied on to free it.
pub fn set_load_file_data_callback(cb: fn(&str) -> Vec<u8>) -> Result<(), SetCallbackError> {
    safe_callback_set_func!(
        cb,
        LOAD_FILE_DATA_CALLBACK,
        ffi::SetLoadFileDataCallback,
        custom_load_file_data_callback,
        "load file data"
    )
}
/// Set custom file text data saver
pub fn set_save_file_text_callback(cb: fn(&str, &str) -> bool) -> Result<(), SetCallbackError> {
    safe_callback_set_func!(
        cb,
        SAVE_FILE_TEXT_CALLBACK,
        ffi::SetSaveFileTextCallback,
        custom_save_file_text_callback,
        "load file data"
    )
}
/// Set custom file text data loader
///
/// Whatever you return from your callback will be intentionally leaked as Raylib is relied on to free it.
pub fn set_load_file_text_callback(cb: fn(&str) -> String) -> Result<(), SetCallbackError> {
    safe_callback_set_func!(
        cb,
        LOAD_FILE_TEXT_CALLBACK,
        ffi::SetLoadFileTextCallback,
        custom_load_file_text_callback,
        "load file text"
    )
}

// region: -- AudioStreamProcessorCallback --

/// This struct encapsulates a rust callback
/// and guarantees the lifetime to be long enough ('a)
/// (once `get_as_user_data` is called, it the struct
/// should not be moved again! -> use Pin<..>)
pub struct AudioStreamProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32),
{
    rust_callback: &'a mut F,
    nb_channels: u32,
    /// The `AudioStream` the trampoline was attached to. Stored so `Drop` can
    /// call `DetachAudioStreamProcessor` with the matching stream + trampoline
    /// pointer, ensuring raylib actually removes the entry from its processor
    /// list (the slot-clear alone leaves a dangling iteration on the C side).
    stream: Option<raylib_sys::AudioStream>,
    callback_index: Option<usize>,
}

impl<'a, F> AudioStreamProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32),
{
    fn new(closure: &'a mut F, nb_channels_from_music: u32) -> Self {
        Self {
            rust_callback: closure,
            nb_channels: nb_channels_from_music,
            stream: None,
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
        unsafe {
            let stream_processor_callback: &mut Self = user_data.cast::<Self>().as_mut().unwrap();
            let f32_ptr = data_ptr as *mut f32;
            let data = {
                std::slice::from_raw_parts_mut(
                    f32_ptr,
                    frame_count as usize * stream_processor_callback.nb_channels as usize,
                )
            };
            (stream_processor_callback.rust_callback)(data, stream_processor_callback.nb_channels);
        }
    }
}

impl<F> Drop for AudioStreamProcessorCallback<'_, F>
where
    F: FnMut(&mut [f32], u32),
{
    fn drop(&mut self) {
        if let (Some(stream), Some(index)) = (self.stream, self.callback_index) {
            detach_audio_stream_processor_with_user_data(stream, index);
        }
    }
}

// endregion: -- AudioStreamProcessorCallback --

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
            let data =
                std::slice::from_raw_parts_mut(data_ptr as *mut f32, frame_count as usize * 2);
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

/// Attach an audio stream processor closure to a [`Music`] stream, returning a pinned guard that detaches on drop.
///
/// The closure fires on raylib's audio thread for every decoded frame of the music stream,
/// receiving a mutable slice of `frame_count * channels` interleaved `f32` samples. Modify
/// the slice in place to apply an effect. Drop the returned `Pin<Box<...>>` to detach via
/// `DetachAudioStreamProcessor`. Multiple processors can be attached to the same music
/// stream and run in attach-order on each frame.
///
/// # Closure constraints
///
/// `F: FnMut(&mut [f32], u32) + Send + 'static`. `Send + 'static` is required because the
/// callback runs on raylib's internal audio thread.
///
/// # Examples
///
/// ```no_run
/// use raylib::prelude::*;
/// use raylib::core::callbacks::attach_audio_stream_processor_to_music;
///
/// let audio = RaylibAudio::init_audio_device().expect("audio init");
/// let music = audio.new_music("assets/track.ogg").expect("music load");
/// let mut gain = |samples: &mut [f32], _channels: u32| {
///     for s in samples { *s *= 0.5; }
/// };
/// let _guard = attach_audio_stream_processor_to_music(&music, &mut gain);
/// // `_guard` drops at end of scope → detach from raylib's processor list.
/// ```
///
/// # See also
///
/// - [`attach_audio_mixed_processor`] — apply a processor to the global mix bus.
/// - [`Music`] — the audio stream this attaches to.
pub fn attach_audio_stream_processor_to_music<'a, F>(
    music: &'a Music<'a>,
    processor: &'a mut F,
) -> Pin<Box<AudioStreamProcessorCallback<'a, F>>>
where
    F: FnMut(&mut [f32], u32) + Send + 'static, // static because the function is executed in another thread
{
    let mut stream_processor_callback =
        Box::new(AudioStreamProcessorCallback::<'a, F>::new(processor, 2));
    stream_processor_callback.stream = Some(music.stream);
    stream_processor_callback.callback_index = Some(attach_audio_stream_processor_with_user_data(
        music.stream,
        AudioCallbackWithUserData::new(
            stream_processor_callback.get_as_user_data(), // pass the address of the stream_processor_callback as void*
            stream_processor_callback.get_c_callback(),
        ),
    ));
    assert!(stream_processor_callback.callback_index.is_some());
    Box::into_pin(stream_processor_callback)
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

impl RaylibHandle {
    /// Set custom trace log
    #[deprecated = "Decoupled from RaylibHandle. Use [set_trace_log_callback](core::callbacks::set_trace_log_callback) instead."]
    pub fn set_trace_log_callback(
        &'_ mut self,
        cb: fn(TraceLogLevel, &str),
    ) -> Result<(), SetCallbackError> {
        set_trace_log_callback(cb)
    }
    /// Set custom file binary data saver
    #[deprecated = "Decoupled from RaylibHandle. Use [set_save_file_data_callback](core::callbacks::set_save_file_data_callback) instead."]
    pub fn set_save_file_data_callback(
        &'_ mut self,
        cb: fn(&str, &[u8]) -> bool,
    ) -> Result<(), SetCallbackError> {
        set_save_file_data_callback(cb)
    }
    /// Set custom file binary data loader
    ///
    /// Whatever you return from your callback will be intentionally leaked as Raylib is relied on to free it.
    #[deprecated = "Decoupled from RaylibHandle. Use [set_load_file_data_callback](core::callbacks::set_load_file_data_callback) instead."]
    pub fn set_load_file_data_callback(
        &'_ mut self,
        cb: fn(&str) -> Vec<u8>,
    ) -> Result<(), SetCallbackError> {
        set_load_file_data_callback(cb)
    }
    /// Set custom file text data saver
    #[deprecated = "Decoupled from RaylibHandle. Use [set_save_file_text_callback](core::callbacks::set_save_file_text_callback) instead."]
    pub fn set_save_file_text_callback(
        &'_ mut self,
        cb: fn(&str, &str) -> bool,
    ) -> Result<(), SetCallbackError> {
        set_save_file_text_callback(cb)
    }
    /// Set custom file text data loader
    ///
    /// Whatever you return from your callback will be intentionally leaked as Raylib is relied on to free it.
    #[deprecated = "Decoupled from RaylibHandle. Use [set_load_file_text_callback](core::callbacks::set_load_file_text_callback) instead."]
    pub fn set_load_file_text_callback(
        &'_ mut self,
        cb: fn(&str) -> String,
    ) -> Result<(), SetCallbackError> {
        set_load_file_text_callback(cb)
    }
}

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
