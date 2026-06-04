# `log` feature: bridge raylib's TraceLog into the Rust `log` facade

**Date:** 2026-06-03
**Status:** approved (design dialogue 2026-06-03)
**Resolves:** the `// TODO: refactor this entire thing to use log` note in
`raylib/src/core/logging.rs`.

## Goal

An opt-in `log` cargo feature on the `raylib` crate that forwards raylib's
C-side `TraceLog` output into the [`log`](https://docs.rs/log) facade, so the
application's chosen logger (`env_logger`, `tracing-log`, …) receives raylib's
logs alongside the app's own, under standard `RUST_LOG`-style filtering.

Direction is **raylib → log only**. A `log::Log` backend writing Rust logs
*into* raylib's TraceLog was considered and rejected (different use-case;
feedback-loop guarding; YAGNI).

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Direction | raylib → `log` | The asked-for use-case: unified app logging. |
| Activation | `RaylibBuilder` method | Owner call ("should really be part of the window builder"). Installing during `build()` **before** `InitWindow` captures raylib's init logging too. |
| Filtering | `log` facade is the single filter | The builder method sets raylib's C-side threshold to `ALL`; otherwise raylib's default `INFO` threshold silently hides `RUST_LOG=debug` output. Users can re-raise the C threshold afterwards to save formatting cost. |
| Feature name | `log = ["dep:log"]` | Matches the dep, consistent with `serde`/`glam`/`mint`. Added to the `full` alias so WS6 quality gates cover it. |
| Log target | `"raylib"` | `RUST_LOG=raylib=trace` scoping. |

## API

```rust
// #[cfg(feature = "log")] on RaylibBuilder:
let (mut rl, thread) = raylib::init()
    .size(800, 450)
    .title("game")
    .log_to_rust()   // forwards TraceLog into `log` from this point on
    .build();
```

`log_to_rust()` sets a builder flag. `build()` honors it before `InitWindow`:

1. Registers the bridge through the **existing** single-slot trace-log
   callback path (`set_trace_log_callback` internals → `setLogCallbackWrapper`
   C shim; varargs formatting stays C-side).
2. Calls `SetTraceLogLevel(LOG_ALL)`.

## The bridge

A plain `fn(TraceLogLevel, &str)` (fits the existing `AtomicUsize` callback
slot) that emits `log::log!(target: "raylib", level, "{text}")`.

Level mapping:

| raylib | log |
|---|---|
| `LOG_TRACE` | `Trace` |
| `LOG_DEBUG` | `Debug` |
| `LOG_INFO` | `Info` |
| `LOG_WARNING` | `Warn` |
| `LOG_ERROR` | `Error` |
| `LOG_FATAL` | `Error` (no fatal in `log`; raylib still aborts C-side after the callback returns — unchanged) |
| `LOG_NONE` / unknown | not emitted |

## Interactions

- **Single callback slot:** `log_to_rust()` claims the same slot as
  `set_trace_log_callback`; mutually exclusive, last writer wins (today's
  semantics). Documented on both.
- **No logger installed:** the bridge emits into the facade only; with no
  logger set, messages drop silently — standard `log`-using-library behavior.
  The bridge never installs/initializes a logger itself.
- **`set_trace_log()`** after `build()` still works to raise the C-side
  threshold (cuts C-side formatting for filtered-out levels).

## Testing

- **Tier-1 (no window):** install the bridge internals + a capturing test
  logger, call `ffi::TraceLog` directly, assert target/level/text for each
  mapped level. Runs under `--features log`.
- **Tier-2 (headless):** `software_renderer,log` + `with_headless` exercising
  the builder flag end-to-end (build with `.log_to_rust()`, assert init logs
  were captured).
- CI: the `full`-alias inclusion puts the feature under the existing
  clippy/docs/test legs; no new workflow.

## Documentation

- Rustdoc on `log_to_rust()` (+ `# Examples` with `env_logger`).
- **Book:** the `callbacks-and-logging` chapter gains a "Routing raylib logs
  through the `log` crate" section (feature flag, builder call, RUST_LOG
  example, level-mapping table, slot-exclusivity note).
- `CHANGELOG.md` entry under the next release.
- Remove the resolved `// TODO: refactor this entire thing to use log` from
  `logging.rs`.

## Out of scope

- `log → raylib` direction (TraceLog as a `log::Log` sink).
- `tracing` native support (apps get it via `tracing-log`).
- Changing default logging behavior when the feature is off or the builder
  method isn't called — zero behavior change without explicit opt-in.
