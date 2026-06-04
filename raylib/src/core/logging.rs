//! Functions to change the behavior of raylib logging.
use crate::consts::TraceLogLevel;
use crate::ffi;
use std::ffi::CString;

/// Set the current threshold (minimum) log level
#[inline]
pub fn set_trace_log(types: TraceLogLevel) {
    unsafe {
        ffi::SetTraceLogLevel((types as u32) as i32);
    }
}

/// Writes a trace log message (`Log::INFO`, `Log::WARNING`, `Log::ERROR`, `Log::DEBUG`).
#[inline]
pub fn trace_log(msg_type: TraceLogLevel, text: &str) {
    unsafe {
        let text = CString::new(text).unwrap();
        ffi::TraceLog((msg_type as u32) as i32, text.as_ptr());
    }
}

/// Forwards one raylib trace-log message into the [`log`] facade under
/// target `"raylib"`.
///
/// `LOG_FATAL` maps to [`log::Level::Error`] (the facade has no fatal
/// level; raylib still aborts after the callback returns, unchanged).
/// `LOG_NONE` and `LOG_ALL` are threshold markers, never message levels —
/// they are not emitted.
#[cfg(feature = "log")]
fn log_bridge(level: TraceLogLevel, text: &str) {
    use log::Level;
    let level = match level {
        TraceLogLevel::LOG_TRACE => Level::Trace,
        TraceLogLevel::LOG_DEBUG => Level::Debug,
        TraceLogLevel::LOG_INFO => Level::Info,
        TraceLogLevel::LOG_WARNING => Level::Warn,
        TraceLogLevel::LOG_ERROR | TraceLogLevel::LOG_FATAL => Level::Error,
        TraceLogLevel::LOG_NONE | TraceLogLevel::LOG_ALL => return,
    };
    log::log!(target: "raylib", level, "{text}");
}

/// Claims the single trace-log callback slot with [`log_bridge`] and drops
/// raylib's own threshold to `LOG_ALL`, making the `log` facade the single
/// level filter. Called by `RaylibBuilder::build` when
/// `.log_to_rust()` was requested.
#[cfg(feature = "log")]
pub(crate) fn install_log_bridge() {
    // The trace-log slot intentionally overwrites (last writer wins); the
    // Result is always Ok today, the API just reserves an error.
    let _ = crate::core::callbacks::set_trace_log_callback(log_bridge);
    set_trace_log(TraceLogLevel::LOG_ALL);
}

#[cfg(all(test, feature = "log"))]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct CaptureLogger {
        records: Mutex<Vec<(log::Level, String, String)>>,
    }

    impl log::Log for CaptureLogger {
        fn enabled(&self, _: &log::Metadata<'_>) -> bool {
            true
        }
        fn log(&self, record: &log::Record<'_>) {
            self.records.lock().unwrap().push((
                record.level(),
                record.target().to_string(),
                record.args().to_string(),
            ));
        }
        fn flush(&self) {}
    }

    static LOGGER: CaptureLogger = CaptureLogger {
        records: Mutex::new(Vec::new()),
    };

    // One test fn: `log::set_logger` is once-per-process, and under plain
    // `cargo test` all unit tests share the process.
    #[test]
    fn bridge_forwards_trace_log_to_log_facade() {
        log::set_logger(&LOGGER).expect("logger installs once");
        log::set_max_level(log::LevelFilter::Trace);
        install_log_bridge();

        // One probe through the real C path (TraceLog → varargs shim →
        // callback slot → bridge) proves the FFI wiring; TraceLog works
        // without InitWindow.
        trace_log(TraceLogLevel::LOG_WARNING, "bridge-ffi-probe");

        // Level-mapping matrix via the bridge fn directly — deterministic,
        // and LOG_FATAL must NOT go through real TraceLog (raylib aborts
        // the process after a fatal log).
        log_bridge(TraceLogLevel::LOG_TRACE, "map-trace");
        log_bridge(TraceLogLevel::LOG_DEBUG, "map-debug");
        log_bridge(TraceLogLevel::LOG_INFO, "map-info");
        log_bridge(TraceLogLevel::LOG_WARNING, "map-warn");
        log_bridge(TraceLogLevel::LOG_ERROR, "map-error");
        log_bridge(TraceLogLevel::LOG_FATAL, "map-fatal");
        log_bridge(TraceLogLevel::LOG_NONE, "map-none");
        log_bridge(TraceLogLevel::LOG_ALL, "map-all");

        let records = LOGGER.records.lock().unwrap();
        let find = |needle: &str| {
            records
                .iter()
                .find(|(_, _, msg)| msg.contains(needle))
                .cloned()
        };

        let (lvl, target, _) = find("bridge-ffi-probe").expect("ffi probe forwarded");
        assert_eq!(lvl, log::Level::Warn);
        assert_eq!(target, "raylib");

        assert_eq!(find("map-trace").unwrap().0, log::Level::Trace);
        assert_eq!(find("map-debug").unwrap().0, log::Level::Debug);
        assert_eq!(find("map-info").unwrap().0, log::Level::Info);
        assert_eq!(find("map-warn").unwrap().0, log::Level::Warn);
        assert_eq!(find("map-error").unwrap().0, log::Level::Error);
        assert_eq!(find("map-fatal").unwrap().0, log::Level::Error);
        assert!(find("map-none").is_none(), "LOG_NONE must not emit");
        assert!(find("map-all").is_none(), "LOG_ALL must not emit");
        assert!(records.iter().all(|(_, t, _)| t == "raylib"));
    }
}
