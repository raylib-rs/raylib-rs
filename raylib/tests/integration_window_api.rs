//! Tier-2: window-coupled API smoke tests batched into one test fn
//! (one InitWindow per process per file). Salvaged from
//! raylib-test/src/{misc,window}.rs.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::with_headless;

#[test]
fn window_api_smoke() {
    with_headless(64, 64, |rl, thread| {
        // Clipboard ops are SKIPPED under software_renderer: the Memory
        // platform doesn't wire SetClipboardText / GetClipboardText to a
        // real backend, so calling them causes a null-pointer dereference
        // (STATUS_ACCESS_VIOLATION on Windows) which Rust panic-unwind
        // can't catch. They are exercised under real GLFW elsewhere.
        eprintln!("SKIP: clipboard not wired under software_renderer Memory platform");

        // Screen-space conversions (don't panic).
        let cam =
            Camera::orthographic(Vector3::ZERO, Vector3::new(0.0, 0.0, 1.0), Vector3::Y, 90.0);
        let _ = rl.get_screen_to_world_ray(Vector2::ZERO, cam);
        let _ = rl.get_world_to_screen(Vector3::ZERO, cam);

        // Timing fns (don't panic).
        rl.set_target_fps(24);
        let _ = rl.get_fps();
        let _ = rl.get_frame_time();
        let _ = rl.get_time();

        // Cursor: double-show / double-hide / double-disable / double-enable
        // must be idempotent (raylib's C side guards against double-state).
        rl.hide_cursor();
        rl.hide_cursor();
        rl.show_cursor();
        rl.show_cursor();
        rl.disable_cursor();
        rl.disable_cursor();
        rl.enable_cursor();
        rl.enable_cursor();

        // Window title + dimensions (with_headless opened a 64x64 window).
        rl.set_window_title(thread, "raylib test");
        assert_eq!(
            rl.get_screen_width(),
            64,
            "screen width matches headless dim"
        );
        assert_eq!(
            rl.get_screen_height(),
            64,
            "screen height matches headless dim"
        );
    });
}
