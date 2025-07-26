//! Functions to change the behavior of raylib logging.

// TODO: refactor this entire thing to use log
use crate::consts::TraceLogLevel;
use crate::ffi;
use std::ffi::CString;

/// Set the current threshold (minimum) log level
#[inline]
pub fn set_trace_log(types: TraceLogLevel) {
    unsafe {
        ffi::SetTraceLogLevel(types as i32);
    }
}

/// Writes a trace log message ([`LOG_INFO`](TraceLogLevel::LOG_INFO), [`LOG_WARNING`](TraceLogLevel::LOG_WARNING), [`LOG_ERROR`](TraceLogLevel::LOG_ERROR), [`LOG_DEBUG`](TraceLogLevel::LOG_DEBUG)).
///
/// # Panics
///
/// This function will panic if `text` contains an internal 0 byte.
#[inline]
pub fn trace_log(msg_type: TraceLogLevel, text: &str) {
    unsafe {
        let text = CString::new(text).expect("text should not contain an internal 0 byte");
        ffi::TraceLog(msg_type as i32, text.as_ptr());
    }
}
