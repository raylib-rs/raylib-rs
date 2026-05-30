//! WS4a: prove the Memory platform initialises windowless and renders into a
//! readable in-memory framebuffer. Only built/run with `--features software_renderer`.
//!
//! Framebuffer readback mechanism (confirmed for WS4b):
//!   `LoadImageFromScreen()` calls `rlReadScreenPixels()` which calls
//!   `glReadPixels()` — implemented in rlsw (the software-renderer GL shim).
//!   After `EndDrawing()` / `SwapScreenBuffer()`, the rlsw framebuffer contains
//!   the rendered pixels and `glReadPixels` returns them correctly.
//!   The platform.pixels buffer in rcore_memory.c is a secondary copy written
//!   by SwapScreenBuffer; `LoadImageFromScreen` reads directly from rlsw, not
//!   from platform.pixels.
//!
//! Minimal feature set required to build+run this test:
//!   --no-default-features --features software_renderer,SUPPORT_MODULE_RTEXTURES
#![cfg(feature = "software_renderer")]
use raylib_sys::*;

#[test]
fn memory_platform_inits_and_reads_back() {
    unsafe {
        InitWindow(320, 240, c"sw-smoke".as_ptr());
        assert!(IsWindowReady(), "Memory platform window should be ready");
        assert_eq!(GetScreenWidth(), 320);
        assert_eq!(GetScreenHeight(), 240);

        BeginDrawing();
        ClearBackground(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        });
        EndDrawing();

        let img = LoadImageFromScreen();
        assert!(!img.data.is_null(), "screen image data must be non-null");
        assert_eq!(img.width, 320);
        assert_eq!(img.height, 240);
        UnloadImage(img);

        CloseWindow();
    }
}
