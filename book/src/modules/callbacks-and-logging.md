# Callbacks and logging

raylib exposes hooks for several behaviours: trace-log output, custom file I/O
loaders/savers, and audio stream processing.  raylib-rs wraps the trace-log
hook as a plain Rust function pointer so you can redirect raylib's diagnostic
output to your own logger.

The logging free functions (`trace_log`, `set_trace_log`) live in
`raylib::core::logging` and are re-exported through `raylib::prelude`.

## API surface

- [`set_trace_log_callback`](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_trace_log_callback.html) —
  install a callback `fn(TraceLogLevel, &str)` that receives every raylib log
  message.  Returns `Ok(())` immediately; the old callback is silently replaced.
- [`TraceLogLevel`](https://docs.rs/raylib/latest/raylib/consts/enum.TraceLogLevel.html) —
  enum: `LOG_ALL`, `LOG_TRACE`, `LOG_DEBUG`, `LOG_INFO`, `LOG_WARNING`,
  `LOG_ERROR`, `LOG_FATAL`, `LOG_NONE`.
- [`set_trace_log`](https://docs.rs/raylib/latest/raylib/core/logging/fn.set_trace_log.html) —
  set the minimum log level; messages below it are suppressed.
- [`trace_log`](https://docs.rs/raylib/latest/raylib/core/logging/fn.trace_log.html) —
  emit a log message at the given level.
- [`set_save_file_data_callback`](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_save_file_data_callback.html) /
  [`set_load_file_data_callback`](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_load_file_data_callback.html) /
  [`set_save_file_text_callback`](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_save_file_text_callback.html) /
  [`set_load_file_text_callback`](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_load_file_text_callback.html) —
  replace raylib's file I/O with your own Rust closures (e.g., to read assets
  from an archive).
- [`attach_audio_stream_processor_to_music`](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.attach_audio_stream_processor_to_music.html) —
  attach an `FnMut(&mut [f32], u32)` audio processor to a `Music` stream;
  returns a pinned guard that detaches on drop.

## Example

Install a trace-log callback and emit one message.  No window needed to install
the callback, but `trace_log` calls into `ffi::TraceLog` which does require an
active raylib context, so this example is `no_run`.

```rust,no_run
# extern crate raylib;
use raylib::core::callbacks::set_trace_log_callback;
use raylib::core::logging::trace_log;
use raylib::consts::TraceLogLevel;

fn my_logger(level: TraceLogLevel, msg: &str) {
    eprintln!("[{level:?}] {msg}");
}

fn main() {
    // Install the callback before init so we capture startup messages too.
    set_trace_log_callback(my_logger).expect("callback already set");

    let (_rl, _thread) = raylib::init()
        .size(1, 1)
        .title("callback demo")
        .build();

    // Emit a custom message through the same callback.
    trace_log(TraceLogLevel::LOG_INFO, "Hello from the custom logger");
}
```

## Gotchas

- **Callbacks are global state.** Installing a new trace-log callback replaces
  the previous one.  `set_trace_log_callback` always returns `Ok(())`.  The old
  methods on `RaylibHandle` (e.g., `rl.set_trace_log_callback(...)`) are
  `#[deprecated]` — use the free function form instead.
- **The audio callback was removed in 6.0.** The old per-sample audio callback
  API is gone entirely.  Use `AudioStream` with `is_processed` /
  `update_audio_stream` for custom PCM generation, or attach a processor via
  `attach_audio_stream_processor_to_music`.
- **File I/O callbacks replace raylib's I/O for the lifetime of the process.**
  There is no "unset" for `set_load_file_data_callback` etc. — they can only be
  set once per callback slot.

## See also

- [Audio](./audio.md) — `AudioStream` and the audio processor callback.
- [`set_trace_log_callback` docs.rs](https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_trace_log_callback.html)
