//! Tier-2: `.log_to_rust()` end-to-end under the headless software renderer.
//! Builder installs the bridge BEFORE InitWindow, so raylib's init logging
//! must arrive through the `log` facade with target "raylib".
#![cfg(all(
    feature = "software_renderer",
    feature = "log",
    feature = "SUPPORT_TRACELOG"
))]

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
