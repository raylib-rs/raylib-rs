# `log` Crate Bridge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Opt-in `log` cargo feature: `raylib::init().log_to_rust().build()` forwards raylib's C-side `TraceLog` output into the Rust `log` facade (target `"raylib"`), with the facade as the single level filter.

**Architecture:** A `fn(TraceLogLevel, &str)` bridge in `raylib/src/core/logging.rs` emits `log::log!`; it is installed through the *existing* single-slot trace-log callback path (`set_trace_log_callback` → `setLogCallbackWrapper` C shim) by `RaylibBuilder::build()` before `InitWindow`, which also sets raylib's C threshold to `LOG_ALL`. Spec: `docs/superpowers/specs/2026-06-03-log-crate-feature-design.md`.

**Tech Stack:** `log = "0.4"` (optional dep), existing callbacks infrastructure, cargo-nextest, mdBook.

**Build note (Windows worktrees):** prefix cargo commands with `CARGO_TARGET_DIR=C:/rt` — the deep worktree path overflows MAX_PATH inside raylib-sys's CMake scratch dirs otherwise. Tier-1/unit commands below assume it.

---

### Task 1: Feature plumbing (`Cargo.toml`)

**Files:**
- Modify: `raylib/Cargo.toml` (dependencies block ~line 18-26; `[features]` block ~line 34; `full` alias ~line 60)

- [ ] **Step 1: Add the optional dependency**

In `raylib/Cargo.toml` `[dependencies]`, after the `mint` line:

```toml
# Opt-in bridge from raylib's TraceLog into the Rust `log` facade.
log = { version = "0.4", optional = true }
```

- [ ] **Step 2: Add the feature + extend `full`**

In `[features]`, after `mint = [...]`:

```toml
log = ["dep:log"]
```

In the `full` list, extend the first line:

```toml
    "default", "raygui", "glam", "mint", "serde", "log",
```

- [ ] **Step 3: Verify it resolves**

Run: `CARGO_TARGET_DIR=C:/rt cargo check -p raylib --features log`
Expected: clean build (no code uses the dep yet).

- [ ] **Step 4: Commit**

```bash
git add raylib/Cargo.toml
git commit -m "feat(log): add opt-in log feature plumbing"
```

---

### Task 2: Bridge fn + installer in `logging.rs` (TDD)

**Files:**
- Modify: `raylib/src/core/logging.rs` (whole file is 23 lines today)

- [ ] **Step 1: Write the failing test**

Append to `raylib/src/core/logging.rs`:

```rust
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
```

- [ ] **Step 2: Run it to confirm it fails**

Run: `CARGO_TARGET_DIR=C:/rt cargo nextest run -p raylib --features log -E 'test(bridge_forwards_trace_log_to_log_facade)'`
Expected: COMPILE FAILURE — `install_log_bridge` / `log_bridge` not found.

- [ ] **Step 3: Implement bridge + installer**

In `raylib/src/core/logging.rs`: delete the line `// TODO: refactor this entire thing to use log` (line 2, resolved by this change) and add after `trace_log`:

```rust
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
```

If the `TraceLogLevel` variant set differs (compile error on the match), check `raylib-sys` bindgen output — the variants are `LOG_ALL, LOG_TRACE, LOG_DEBUG, LOG_INFO, LOG_WARNING, LOG_ERROR, LOG_FATAL, LOG_NONE`; the match must stay exhaustive without a `_` arm.

- [ ] **Step 4: Run the test to confirm it passes**

Run: `CARGO_TARGET_DIR=C:/rt cargo nextest run -p raylib --features log -E 'test(bridge_forwards_trace_log_to_log_facade)'`
Expected: PASS (1 test).

- [ ] **Step 5: Confirm feature-off still builds (cfg hygiene)**

Run: `CARGO_TARGET_DIR=C:/rt cargo check -p raylib`
Expected: clean — everything added is `#[cfg(feature = "log")]`-gated.

- [ ] **Step 6: Commit**

```bash
git add raylib/src/core/logging.rs
git commit -m "feat(log): TraceLog->log bridge fn + installer (Tier-1 tested)"
```

---

### Task 3: `RaylibBuilder::log_to_rust()` (TDD via Tier-2 headless)

**Files:**
- Modify: `raylib/src/core/mod.rs` (builder struct ~line 333-354; methods after `log_level` ~line 378; `build()` after the `SetTraceLogLevel` block ~line 555-557)
- Create: `raylib/tests/integration_log_bridge.rs`

- [ ] **Step 1: Write the failing Tier-2 test**

Create `raylib/tests/integration_log_bridge.rs`:

```rust
//! Tier-2: `.log_to_rust()` end-to-end under the headless software renderer.
//! Builder installs the bridge BEFORE InitWindow, so raylib's init logging
//! must arrive through the `log` facade with target "raylib".
#![cfg(all(feature = "software_renderer", feature = "log"))]

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

// One #[test] per file: InitWindow is once-per-process (same rule as the
// other Tier-2 files; nextest gives per-test process isolation).
#[test]
fn builder_log_to_rust_bridges_init_logs() {
    log::set_logger(&LOGGER).expect("logger installs once");
    log::set_max_level(log::LevelFilter::Trace);

    let (_rl, _thread) = raylib::init()
        .size(64, 64)
        .title("log-bridge")
        .log_to_rust()
        .build();

    let records = LOGGER.records.lock().unwrap();
    assert!(
        records
            .iter()
            .any(|(_, target, msg)| target == "raylib" && msg.contains("Initializing raylib")),
        "expected raylib's init log line via the bridge; got {} records: {:?}",
        records.len(),
        records.iter().take(5).collect::<Vec<_>>(),
    );
}
```

- [ ] **Step 2: Run it to confirm it fails**

Run: `CARGO_TARGET_DIR=C:/rt cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,log -E 'binary(integration_log_bridge)'`
Expected: COMPILE FAILURE — no method `log_to_rust` on `RaylibBuilder`.

- [ ] **Step 3: Implement the builder flag + method + build() wiring**

In `raylib/src/core/mod.rs`, add the field at the end of the `RaylibBuilder` struct (after `title: &'a str,`):

```rust
    #[cfg(feature = "log")]
    bridge_log: bool,
```

Add the method directly after `log_level()` (~line 378):

```rust
    /// Forwards raylib's `TraceLog` output into the [`log`] crate facade
    /// (target `"raylib"`), so the application's logger (`env_logger`,
    /// `tracing-log`, …) receives raylib's logs under standard
    /// `RUST_LOG`-style filtering.
    ///
    /// Installed during [`build`](Self::build) *before* `InitWindow`, so
    /// raylib's init logging is captured too. Makes the `log` facade the
    /// single level filter by setting raylib's own threshold to `LOG_ALL`
    /// — overriding any [`log_level`](Self::log_level) — and claims the
    /// same single callback slot as
    /// [`set_trace_log_callback`](crate::core::callbacks::set_trace_log_callback)
    /// (mutually exclusive; last writer wins).
    ///
    /// The bridge only emits into the facade; install a logger yourself
    /// (e.g. `env_logger::init()`) or the messages are silently dropped.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // env_logger::init();   // any `log`-compatible logger
    /// let (mut rl, thread) = raylib::init()
    ///     .size(800, 450)
    ///     .title("game")
    ///     .log_to_rust()
    ///     .build();
    /// log::info!("app and raylib logs share one pipeline now");
    /// ```
    #[cfg(feature = "log")]
    pub const fn log_to_rust(&mut self) -> &mut Self {
        self.bridge_log = true;
        self
    }
```

In `build()`, directly after the existing `SetTraceLogLevel` block (`unsafe { ffi::SetTraceLogLevel(self.log_level as i32); }`, ~line 555) and before `let rl = init_window(...)`:

```rust
        // The log bridge claims the trace-log callback slot and makes the
        // `log` facade the single filter — installed before InitWindow so
        // init logging is captured, and after the SetTraceLogLevel above
        // so its LOG_ALL override wins over `.log_level()`.
        #[cfg(feature = "log")]
        if self.bridge_log {
            crate::core::logging::install_log_bridge();
        }
```

- [ ] **Step 4: Run the Tier-2 test to confirm it passes**

Run: the same command as Step 2.
Expected: PASS (1 test). If the assertion fails because raylib's init line phrasing changed, check the captured records printed in the panic message and match on a stable substring of the actual init output (e.g. `"raylib"` version banner) — keep the `target == "raylib"` half of the assertion as-is.

- [ ] **Step 5: Re-run Tier-1 + feature-off checks**

Run: `CARGO_TARGET_DIR=C:/rt cargo nextest run -p raylib --features log -E 'test(bridge_forwards_trace_log_to_log_facade)'`
Expected: PASS.
Run: `CARGO_TARGET_DIR=C:/rt cargo check -p raylib`
Expected: clean (field + method + wiring all cfg-gated).

- [ ] **Step 6: Commit**

```bash
git add raylib/src/core/mod.rs raylib/tests/integration_log_bridge.rs
git commit -m "feat(log): RaylibBuilder::log_to_rust() builder bridge (Tier-2 tested)"
```

---

### Task 4: CI wiring (`test.yml` software-render leg)

**Files:**
- Modify: `.github/workflows/test.yml` (software-render job, after the "Tier-2 render tests (rlgl)" step)

- [ ] **Step 1: Add a Tier-2 step running the new binary**

Insert after the "Tier-2 render tests (rlgl)" step:

```yaml
      - name: Tier-2 log-bridge test
        run: cargo nextest run -p raylib --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,log -E 'binary(integration_log_bridge)'
```

(The Tier-1 unit test needs no wiring: `log` is now in `full`, so the existing `unit` matrix `features: full` leg compiles and runs it.)

- [ ] **Step 2: Sanity-check the YAML**

Run: `git diff .github/workflows/test.yml` and confirm indentation matches the sibling steps (6 spaces before `- name:`).

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/test.yml
git commit -m "ci(test): run the Tier-2 log-bridge test on the software-render leg"
```

---

### Task 5: Book section

**Files:**
- Modify: `book/src/modules/callbacks-and-logging.md`

- [ ] **Step 1: Find the insertion point**

Run: `grep -n "^## " book/src/modules/callbacks-and-logging.md`
Insert the new section before the `## See also` heading (or at the end of the chapter if there is none).

- [ ] **Step 2: Add the section**

````markdown
## Routing raylib logs through the `log` crate

With the opt-in `log` feature, the window builder can forward raylib's
`TraceLog` output into the [`log`](https://docs.rs/log) facade, so your
application's logger (`env_logger`, `tracing-log`, …) receives raylib's
logs alongside your own:

```toml
[dependencies]
raylib = { version = "6", features = ["log"] }
log = "0.4"
env_logger = "0.11"
```

```rust,no_run
env_logger::init(); // any `log`-compatible logger

let (mut rl, thread) = raylib::init()
    .size(800, 450)
    .title("game")
    .log_to_rust()
    .build();

log::info!("app and raylib logs share one pipeline now");
```

Messages arrive under the target `"raylib"`, so `RUST_LOG=raylib=debug`
scopes raylib's output independently of your app's.

The bridge makes the `log` facade the single level filter: it sets
raylib's own threshold to `LOG_ALL` (overriding `.log_level()`), so what
you see is controlled entirely by your logger's configuration. Raylib
levels map `TRACE→trace`, `DEBUG→debug`, `INFO→info`, `WARNING→warn`,
and both `ERROR` and `FATAL` to `error` (the facade has no fatal level;
raylib still aborts after a fatal log, unchanged).

Two caveats:

- The bridge claims the same single callback slot as
  [`set_trace_log_callback`] — they are mutually exclusive, last writer
  wins.
- The bridge only *emits* into the facade. Without a logger installed,
  the messages are silently dropped, like any library using `log`.

[`set_trace_log_callback`]: https://docs.rs/raylib/latest/raylib/core/callbacks/fn.set_trace_log_callback.html
````

- [ ] **Step 3: Build the book**

Run: `mdbook build book` (if mdbook is installed locally; otherwise rely on the `book.yml` CI leg)
Expected: clean build.

- [ ] **Step 4: Commit**

```bash
git add book/src/modules/callbacks-and-logging.md
git commit -m "docs(book): log-crate bridge section in callbacks-and-logging"
```

---

### Task 6: CHANGELOG

**Files:**
- Modify: `CHANGELOG.md` (top of file, above the `## 6.0.0-rc.2` heading)

- [ ] **Step 1: Add an Unreleased section**

Insert between the `# raylib-rs Changelog` title and the `## 6.0.0-rc.2` heading:

```markdown
## Unreleased

### Added

- Opt-in `log` feature: `RaylibBuilder::log_to_rust()` forwards raylib's
  `TraceLog` output into the [`log`](https://docs.rs/log) facade (target
  `"raylib"`), making `RUST_LOG`-style filtering the single source of
  truth. Levels map `TRACE/DEBUG/INFO/WARNING→trace/debug/info/warn`,
  `ERROR`+`FATAL→error`. See the *Callbacks and logging* book chapter.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): log-crate bridge feature entry"
```

---

### Task 7: Full verification sweep

**Files:** none (verification only)

- [ ] **Step 1: fmt**

Run: `rustfmt --edition 2024 --check raylib/src/core/logging.rs raylib/src/core/mod.rs raylib/tests/integration_log_bridge.rs`
Expected: no output, exit 0. (Repo-wide `cargo fmt --all --check` overflows MAX_PATH in deep worktrees on Windows.)

- [ ] **Step 2: clippy (the gates CI runs)**

Run: `CARGO_TARGET_DIR=C:/rt cargo clippy -p raylib --lib --bins --features full -- -D warnings`
Expected: clean.
Run: `CARGO_TARGET_DIR=C:/rt cargo clippy -p raylib --tests --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES,SUPPORT_MODULE_RSHAPES,SUPPORT_MODULE_RTEXT,SUPPORT_MODULE_RMODELS,SUPPORT_MODULE_RAUDIO,SUPPORT_IMAGE_GENERATION,raygui,log -- -D warnings`
Expected: clean.

- [ ] **Step 3: docs (deny missing_docs + links)**

Run: `RUSTDOCFLAGS="-D warnings" CARGO_TARGET_DIR=C:/rt cargo doc -p raylib --no-deps --features full`
Expected: clean — `log_to_rust`'s intra-doc links resolve.

- [ ] **Step 4: doctests**

Run: `CARGO_TARGET_DIR=C:/rt cargo test -p raylib --doc --features full`
Expected: PASS (the `log_to_rust` example is `no_run`; it must still compile).

- [ ] **Step 5: both test tiers once more**

Run: `CARGO_TARGET_DIR=C:/rt cargo nextest run -p raylib --features full -E 'test(bridge_forwards_trace_log_to_log_facade)'`
Expected: PASS (proves the `full`-alias path CI uses).
Run: the Task 3 Step 2 Tier-2 command.
Expected: PASS.

- [ ] **Step 6: Commit anything the sweep touched (usually nothing)**

```bash
git status --short   # expect empty
```
