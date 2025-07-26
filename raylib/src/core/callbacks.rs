#![allow(non_camel_case_types)]

mod stream_processor_with_user_data_wrapper;

use crate::{
    RaylibHandle,
    audio::{AudioStream, Music},
    ffi::{self, TraceLogLevel},
};
use std::{
    borrow::Cow,
    convert::TryInto,
    ffi::{CStr, CString, c_char, c_int, c_void},
    marker::PhantomData,
    pin::Pin,
    ptr::null_mut,
    slice::from_raw_parts_mut,
    sync::atomic::{AtomicUsize, Ordering},
};
use stream_processor_with_user_data_wrapper::{
    AudioCallbackWithUserData, attach_audio_stream_processor_with_user_data,
    detach_audio_stream_processor_with_user_data,
};

type TraceLogCallback = unsafe extern "C" fn(*mut i8, *const i8, ...);
unsafe extern "C" {
    fn SetTraceLogCallback(cb: Option<TraceLogCallback>);
}

mod sealed {
    use super::{
        AtomicFn, AtomicUsize, RustAudioStreamCallback, RustLoadFileDataCallback,
        RustLoadFileTextCallback, RustSaveFileDataCallback, RustSaveFileTextCallback,
        RustTraceLogCallback,
    };
    use std::num::NonZeroUsize;

    /// # Safety
    ///
    /// `Self` and `NonZeroUsize` must be safe to convert between and call across threads.
    pub unsafe trait AtomicFnExt: Sized + Send + Sync {
        #[must_use]
        fn into_nonzero(self) -> NonZeroUsize;

        #[must_use]
        fn from_nonzero(val: NonZeroUsize) -> Self;

        #[must_use]
        fn into_usize(val: Option<Self>) -> usize;

        #[must_use]
        fn from_usize(val: usize) -> Option<Self>;
    }

    macro_rules! impl_atomic_fn_ext {
        ($(unsafe impl AtomicFnExt for $Callback:ident {})*) => {$(
            const _: () = {
                assert!(std::mem::size_of::<$Callback>() == std::mem::size_of::<NonZeroUsize>());
                assert!(std::mem::align_of::<$Callback>() == std::mem::align_of::<NonZeroUsize>());
                assert!(std::mem::size_of::<Option<$Callback>>() == std::mem::size_of::<$Callback>());
                assert!(std::mem::size_of::<AtomicFn<$Callback>>() == std::mem::size_of::<AtomicUsize>());
            };

            unsafe impl AtomicFnExt for $Callback {
                #[inline(always)]
                fn into_nonzero(self) -> NonZeroUsize {
                    // SAFETY: Implementor must uphold trait safety contract
                    unsafe { std::mem::transmute::<$Callback, NonZeroUsize>(self) }
                }

                #[inline(always)]
                fn from_nonzero(val: NonZeroUsize) -> $Callback {
                    // SAFETY: Implementor must uphold trait safety contract
                    unsafe { std::mem::transmute::<NonZeroUsize, $Callback>(val) }
                }

                #[inline(always)]
                fn into_usize(val: Option<$Callback>) -> usize {
                    // SAFETY: Implementor must uphold trait safety contract
                    unsafe { std::mem::transmute::<Option<$Callback>, usize>(val) }
                }

                #[inline(always)]
                fn from_usize(val: usize) -> Option<$Callback> {
                    // SAFETY: Implementor must uphold trait safety contract
                    unsafe { std::mem::transmute::<usize, Option<$Callback>>(val) }
                }
            }
        )*};
    }

    impl_atomic_fn_ext! {
        unsafe impl AtomicFnExt for RustTraceLogCallback {}
        unsafe impl AtomicFnExt for RustSaveFileDataCallback {}
        unsafe impl AtomicFnExt for RustLoadFileDataCallback {}
        unsafe impl AtomicFnExt for RustSaveFileTextCallback {}
        unsafe impl AtomicFnExt for RustLoadFileTextCallback {}
        unsafe impl AtomicFnExt for RustAudioStreamCallback {}
    }
}
use sealed::AtomicFnExt;

#[repr(transparent)]
struct AtomicFn<F: AtomicFnExt>(AtomicUsize, PhantomData<F>);

impl<F: AtomicFnExt> AtomicFn<F> {
    /// Construct a null function pointer
    #[inline]
    pub const fn null() -> Self {
        Self(AtomicUsize::new(0), PhantomData)
    }

    /// Stores a value into the atomic function pointer.
    #[inline]
    pub fn store(&self, val: Option<F>, order: Ordering) {
        self.0.store(AtomicFnExt::into_usize(val), order);
    }

    /// Loads a value from the atomic function pointer.
    #[inline]
    pub fn load(&self, order: Ordering) -> Option<F> {
        AtomicFnExt::from_usize(self.0.load(order))
    }
}

type RustTraceLogCallback = fn(TraceLogLevel, &str);
type RustSaveFileDataCallback = fn(&str, &[u8]) -> bool;
type RustLoadFileDataCallback = fn(&str) -> Vec<u8>;
type RustSaveFileTextCallback = fn(&str, &str) -> bool;
type RustLoadFileTextCallback = fn(&str) -> String;
type RustAudioStreamCallback = fn(&[u8]);

const _: () = {};

static TRACE_LOG_CALLBACK: AtomicFn<RustTraceLogCallback> = AtomicFn::null();
static SAVE_FILE_DATA_CALLBACK: AtomicFn<RustSaveFileDataCallback> = AtomicFn::null();
static LOAD_FILE_DATA_CALLBACK: AtomicFn<RustLoadFileDataCallback> = AtomicFn::null();
static SAVE_FILE_TEXT_CALLBACK: AtomicFn<RustSaveFileTextCallback> = AtomicFn::null();
static LOAD_FILE_TEXT_CALLBACK: AtomicFn<RustLoadFileTextCallback> = AtomicFn::null();
static AUDIO_STREAM_CALLBACK: AtomicFn<RustAudioStreamCallback> = AtomicFn::null();

fn trace_log_callback() -> Option<RustTraceLogCallback> {
    TRACE_LOG_CALLBACK.load(Ordering::Relaxed)
}

fn save_file_data_callback() -> Option<RustSaveFileDataCallback> {
    SAVE_FILE_DATA_CALLBACK.load(Ordering::Relaxed)
}

fn load_file_data_callback() -> Option<RustLoadFileDataCallback> {
    LOAD_FILE_DATA_CALLBACK.load(Ordering::Relaxed)
}

fn save_file_text_callback() -> Option<RustSaveFileTextCallback> {
    SAVE_FILE_TEXT_CALLBACK.load(Ordering::Relaxed)
}

fn load_file_text_callback() -> Option<RustLoadFileTextCallback> {
    LOAD_FILE_TEXT_CALLBACK.load(Ordering::Relaxed)
}

fn audio_stream_callback() -> Option<RustAudioStreamCallback> {
    AUDIO_STREAM_CALLBACK.load(Ordering::Relaxed)
}

/// # Safety
///
/// This method converts `text` to a [`CStr`] without checks. It is the caller's responsibility to ensure that `text` meets the safety requirements of [`CStr::from_ptr`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn custom_trace_log_callback(level: TraceLogLevel, text: *const c_char) {
    if let Some(trace_log) = trace_log_callback() {
        let text = if text.is_null() {
            Cow::Borrowed("(MESSAGE WAS NULL)")
        } else {
            unsafe { CStr::from_ptr(text).to_string_lossy() }
        };

        trace_log(level, &text);
    }
}

extern "C" fn custom_save_file_data_callback(
    path: *const c_char,
    buffer: *mut c_void,
    size: c_int,
) -> bool {
    let save_file_data = save_file_data_callback().expect("no callback");
    let path = unsafe { CStr::from_ptr(path) };
    let buffer = unsafe {
        from_raw_parts_mut(
            buffer.cast::<u8>(),
            size.try_into().expect("size should not be negative"),
        )
    };

    save_file_data(path.to_str().expect("path should be non utf-8"), buffer)
}

extern "C" fn custom_load_file_data_callback(path: *const c_char, size: *mut c_int) -> *mut u8 {
    let load_file_data = load_file_data_callback().expect("no callback");

    if let Some(size) = unsafe { size.as_mut() } {
        let path = unsafe { CStr::from_ptr(path) };
        let buffer = load_file_data(path.to_str().expect("path is non utf-8"));
        *size = buffer.len().try_into().expect("out of range buffer size");

        // Copy everything to the raylib world
        let buffer_ffi =
            unsafe { ffi::MemAlloc((*size).try_into().expect("non representable buffer size")) }
                .cast::<u8>();
        unsafe {
            buffer_ffi.copy_from_nonoverlapping(buffer.as_ptr(), buffer.len());
        }

        buffer_ffi
    } else {
        null_mut()
    }
}

extern "C" fn custom_save_file_text_callback(a: *const c_char, b: *mut c_char) -> bool {
    let save_file_text = save_file_text_callback().expect("callback should be initialized");
    let a = unsafe { CStr::from_ptr(a) };
    let b = unsafe { CStr::from_ptr(b) };
    save_file_text(
        a.to_str().expect("string should be utf-8"),
        b.to_str().expect("string should be utf-8"),
    )
}
extern "C" fn custom_load_file_text_callback(a: *const c_char) -> *mut c_char {
    let load_file_text = load_file_text_callback().expect("callback should be initialized");
    let a = unsafe { CStr::from_ptr(a) };
    let st = load_file_text(a.to_str().expect("string should be utf-8"));
    let oh = Box::leak(Box::new(
        CString::new(st).expect("string should not contain an internal 0 byte"),
    ));
    oh.as_ptr().cast_mut().cast::<c_char>()
}

extern "C" fn custom_audio_stream_callback(a: *mut c_void, b: u32) {
    let audio_stream = audio_stream_callback().expect("callback should be initialized");
    let a = unsafe { std::slice::from_raw_parts(a.cast::<u8>(), b as usize) };
    audio_stream(a);
}
#[derive(Debug)]
pub struct SetCallbackError(&'static str);

impl std::fmt::Display for SetCallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("There is a {} callback already set.", self.0))
    }
}

impl std::error::Error for SetCallbackError {}

macro_rules! safe_callback_set_func {
    ($cb:expr, $target_cb:expr, $rawsetter:expr, $ogfunc:expr, $ty:literal) => {
        if $target_cb.load(Ordering::Acquire).is_none() {
            $target_cb.store(Some($cb), Ordering::Release);
            unsafe { $rawsetter(Some($ogfunc)) };
            Ok(())
        } else {
            Err(SetCallbackError($ty))
        }
    };
}

/// Set custom trace log
///
/// # Errors
///
/// This function does not error currently.
pub fn set_trace_log_callback(cb: RustTraceLogCallback) -> Result<(), SetCallbackError> {
    TRACE_LOG_CALLBACK.store(Some(cb), Ordering::Relaxed);
    unsafe { ffi::setLogCallbackWrapper() };
    Ok(())
}
/// Set custom file binary data saver
///
/// # Errors
///
/// This function returns [`SetCallbackError`] if a custom `save_file_data` callback is already set.
pub fn set_save_file_data_callback(cb: RustSaveFileDataCallback) -> Result<(), SetCallbackError> {
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
///
/// # Errors
///
/// This function returns [`SetCallbackError`] if a custom `load_file_data` callback is already set.
pub fn set_load_file_data_callback(cb: RustLoadFileDataCallback) -> Result<(), SetCallbackError> {
    safe_callback_set_func!(
        cb,
        LOAD_FILE_DATA_CALLBACK,
        ffi::SetLoadFileDataCallback,
        custom_load_file_data_callback,
        "load file data"
    )
}
/// Set custom file text data saver
///
/// # Errors
///
/// This function returns [`SetCallbackError`] if a custom `save_file_text` callback is already set.
pub fn set_save_file_text_callback(cb: RustSaveFileTextCallback) -> Result<(), SetCallbackError> {
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
///
/// # Errors
///
/// This function returns [`SetCallbackError`] if a custom `load_file_text` callback is already set.
pub fn set_load_file_text_callback(cb: RustLoadFileTextCallback) -> Result<(), SetCallbackError> {
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
/// and guarantees the lifetime to be long enough (`'a`)
/// (once [`Self::get_as_user_data`] is called, it the struct
/// should not be moved again! -> use [`Pin<..>`])
pub struct AudioStreamProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32),
{
    rust_callback: &'a mut F,
    nb_channels: u32,
    callback_index: Option<usize>,
}

impl<'a, F> AudioStreamProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32),
{
    const fn new(closure: &'a mut F, nb_channels_from_music: u32) -> Self {
        Self {
            rust_callback: closure,
            nb_channels: nb_channels_from_music,
            callback_index: None,
        }
    }

    const fn get_as_user_data(&mut self) -> *mut ::std::os::raw::c_void {
        std::ptr::from_mut(self).cast::<::std::os::raw::c_void>()
    }

    fn get_c_callback(
        &mut self,
    ) -> extern "C" fn(
        *mut ::std::os::raw::c_void,
        *mut ::std::os::raw::c_void,
        ::std::os::raw::c_uint,
    ) {
        _ = self;
        Self::c_callback
    }

    extern "C" fn c_callback(
        user_data: *mut ::std::os::raw::c_void,
        data_ptr: *mut ::std::os::raw::c_void,
        frame_count: ::std::os::raw::c_uint,
    ) {
        let stream_processor_callback =
            unsafe { user_data.cast::<Self>().as_mut() }.expect("user_data should not be null");
        let f32_ptr = data_ptr.cast();
        let data = unsafe {
            std::slice::from_raw_parts_mut(
                f32_ptr,
                frame_count as usize * stream_processor_callback.nb_channels as usize,
            )
        };
        (stream_processor_callback.rust_callback)(data, stream_processor_callback.nb_channels);
    }
}

impl<'a, F> Drop for AudioStreamProcessorCallback<'a, F>
where
    F: FnMut(&mut [f32], u32) + 'a,
{
    fn drop(&mut self) {
        if let Some(index) = self.callback_index {
            detach_audio_stream_processor_with_user_data(index);
        }
    }
}

// endregion: -- AudioStreamProcessorCallback --

/// # Panics
///
/// This method will panic if `music`'s stream buffer is null.
pub fn attach_audio_stream_processor_to_music<'a, F>(
    music: &'a Music<'a>,
    processor: &'a mut F,
) -> Pin<Box<AudioStreamProcessorCallback<'a, F>>>
where
    F: FnMut(&mut [f32], u32) + Send + 'static, // static because the function is executed in another thread
{
    assert!(
        !music.stream.buffer.is_null(),
        "music stream buffer should not be null"
    );
    let mut stream_processor_callback =
        Box::new(AudioStreamProcessorCallback::<'a, F>::new(processor, 2));
    // SAFETY: Checked `music.stream.buffer` and it is not null.
    // TODO: How can we ensure `music`'s stream does not have any copies
    stream_processor_callback.callback_index = Some(unsafe {
        attach_audio_stream_processor_with_user_data(
            music.stream,
            AudioCallbackWithUserData::new(
                stream_processor_callback.get_as_user_data(), // pass the address of the stream_processor_callback as void*
                stream_processor_callback.get_c_callback(),
            ),
        )
    });
    assert!(stream_processor_callback.callback_index.is_some());
    Box::into_pin(stream_processor_callback)
}

/// Audio thread callback to request new data
///
/// # Errors
///
/// This function returns [`SetCallbackError`] if a custom `audio_stream` callback is already set.
pub fn set_audio_stream_callback(
    stream: AudioStream,
    cb: RustAudioStreamCallback,
) -> Result<(), SetCallbackError> {
    if AUDIO_STREAM_CALLBACK.load(Ordering::Acquire).is_none() {
        AUDIO_STREAM_CALLBACK.store(Some(cb), Ordering::Release);
        unsafe { ffi::SetAudioStreamCallback(stream.0, Some(custom_audio_stream_callback)) }
        Ok(())
    } else {
        Err(SetCallbackError("audio stream"))
    }
}

impl RaylibHandle {
    /// Set custom trace log
    ///
    /// # Errors
    ///
    /// This function returns [`SetCallbackError`] if a custom `trace_log` callback is already set.
    #[deprecated = "Decoupled from RaylibHandle. Use [`set_trace_log_callback`](core::callbacks::set_trace_log_callback) instead."]
    pub fn set_trace_log_callback(
        &mut self,
        cb: RustTraceLogCallback,
    ) -> Result<(), SetCallbackError> {
        set_trace_log_callback(cb)
    }
    /// Set custom file binary data saver
    ///
    /// # Errors
    ///
    /// This function returns [`SetCallbackError`] if a custom `save_file_data` callback is already set.
    #[deprecated = "Decoupled from RaylibHandle. Use [`set_save_file_data_callback`](core::callbacks::set_save_file_data_callback) instead."]
    pub fn set_save_file_data_callback(
        &mut self,
        cb: RustSaveFileDataCallback,
    ) -> Result<(), SetCallbackError> {
        set_save_file_data_callback(cb)
    }
    /// Set custom file binary data loader
    ///
    /// Whatever you return from your callback will be intentionally leaked as Raylib is relied on to free it.
    ///
    /// # Errors
    ///
    /// This function returns [`SetCallbackError`] if a custom `load_file_data` callback is already set.
    #[deprecated = "Decoupled from RaylibHandle. Use [`set_load_file_data_callback`](core::callbacks::set_load_file_data_callback) instead."]
    pub fn set_load_file_data_callback(
        &mut self,
        cb: RustLoadFileDataCallback,
    ) -> Result<(), SetCallbackError> {
        set_load_file_data_callback(cb)
    }
    /// Set custom file text data saver
    ///
    /// # Errors
    ///
    /// This function returns [`SetCallbackError`] if a custom `save_file_text` callback is already set.
    #[deprecated = "Decoupled from RaylibHandle. Use [`set_save_file_text_callback`](core::callbacks::set_save_file_text_callback) instead."]
    pub fn set_save_file_text_callback(
        &mut self,
        cb: RustSaveFileTextCallback,
    ) -> Result<(), SetCallbackError> {
        set_save_file_text_callback(cb)
    }
    /// Set custom file text data loader
    ///
    /// Whatever you return from your callback will be intentionally leaked as Raylib is relied on to free it.
    ///
    /// # Errors
    ///
    /// This function returns [`SetCallbackError`] if a custom `load_file_text` callback is already set.
    #[deprecated = "Decoupled from RaylibHandle. Use [`set_load_file_text_callback`](core::callbacks::set_load_file_text_callback) instead."]
    pub fn set_load_file_text_callback(
        &mut self,
        cb: RustLoadFileTextCallback,
    ) -> Result<(), SetCallbackError> {
        set_load_file_text_callback(cb)
    }

    /// Audio thread callback to request new data
    ///
    /// # Errors
    ///
    /// This function returns [`SetCallbackError`] if a custom `audio_stream` callback is already set.
    #[deprecated = "Decoupled from RaylibHandle. Use [`set_audio_stream_callback`](core::callbacks::set_audio_stream_callback) instead."]
    pub fn set_audio_stream_callback(
        &mut self,
        stream: AudioStream,
        cb: RustAudioStreamCallback,
    ) -> Result<(), SetCallbackError> {
        if AUDIO_STREAM_CALLBACK.load(Ordering::Acquire).is_none() {
            AUDIO_STREAM_CALLBACK.store(Some(cb), Ordering::Release);
            unsafe { ffi::SetAudioStreamCallback(stream.0, Some(custom_audio_stream_callback)) }
            Ok(())
        } else {
            Err(SetCallbackError("audio stream"))
        }
    }
}
