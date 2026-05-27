//! WS4b Tier-2: software-rendered shapes land the expected pixels. Headless, no GPU.
//!
//! # rlsw readback notes (deterministic on every OS — these are compile-time, not platform, effects)
//!
//! Two characteristics of the software-renderer readback shape the probes below.
//! Both are baked into rlsw's pure-C source, so they are identical on Linux,
//! macOS, and Windows — do NOT special-case them per platform.
//!
//! 1. **Channel order:** rlsw is built with its upstream default
//!    `SW_FRAMEBUFFER_OUTPUT_BGRA = true` (`external/rlsw.h`), so the color
//!    buffer is emitted BGRA. `rlReadScreenPixels` (`rlgl.h`) labels it
//!    `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8` without reordering, so the R and B
//!    channels read back swapped relative to what was drawn.
//! 2. **Y orientation:** `rlReadScreenPixels` flips vertically unconditionally,
//!    which combined with rlsw's buffer origin leaves the image y-axis inverted
//!    relative to screen space: `y_img = (h - 1) - y_screen`.
//!
//! Probe coordinates and expected colors below reflect these confirmed behaviours.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::{assert_pixel, render_frame, with_headless};

// One `#[test]` per file: `with_headless` calls `InitWindow`, which raylib permits
// only once per process, so every probe shares a single headless context.
#[test]
fn shapes_render_expected_pixels() {
    // Canvas is 64×64. rlsw image-space: y_img = 63 - y_screen; R↔B channels swapped.
    with_headless(64, 64, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_rectangle(8, 8, 16, 16, Color::RED); // solid red block (screen y=8..23)
            d.draw_circle(48, 48, 8.0, Color::GREEN); // green disc (screen centre 48,48)
            d.draw_line(0, 0, 63, 0, Color::BLUE); // top edge blue (screen y=0)
        });

        // Background: screen (40,20) → image (40,43). Well outside all primitives.
        assert_pixel(&img, 40, 43, Color::BLACK, 8);

        // Rectangle interior: screen (8..23, 8..23) → image (8..23, 40..55).
        // rlsw R↔B swap: drawn Color::RED {255,0,0} → image pixel {0,0,255} = Color::BLUE.
        assert_pixel(&img, 16, 47, Color::BLUE, 16);

        // Circle centre: screen (48,48) → image (48,15).
        // Color::GREEN is {0,128,0} in this crate (not raylib's {0,228,48}); R=B=0,
        // so the R↔B swap is a no-op and it stays {0,128,0}.
        assert_pixel(&img, 48, 15, Color::GREEN, 16);

        // Line near top: drawn at screen y=0 but reads back at image y=62, not the
        // formula's 63 — rlsw places the topmost line one row in, and image y=63 reads
        // empty. rlsw R↔B swap: drawn Color::BLUE {0,0,255} → image {255,0,0} = Color::RED.
        assert_pixel(&img, 32, 62, Color::RED, 16);
    });
}
