//! WS4b Tier-2 (updated WS5-prep): software-rendered shapes land the expected
//! pixels. Headless, no GPU. Uses the normalized `render_frame`, so probes are in
//! natural top-left coordinates with the drawn colors (no BGRA / Y-flip
//! compensation — `render_frame` handles both).
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::{assert_pixel, render_frame, with_headless};

// One `#[test]` per file: `with_headless` calls `InitWindow`, which raylib permits
// only once per process, so every probe shares a single headless context.
#[test]
fn shapes_render_expected_pixels() {
    with_headless(64, 64, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_rectangle(8, 8, 16, 16, Color::RED); // solid red block, screen (8..23, 8..23)
            d.draw_circle(48, 48, 8.0, Color::GREEN); // green disc, screen centre (48,48)
            d.draw_line(0, 0, 63, 0, Color::BLUE); // top edge blue, screen y≈0
        });

        // Background: well outside every primitive.
        assert_pixel(&img, 40, 20, Color::BLACK, 8);

        // Rectangle interior reads back as the drawn red.
        assert_pixel(&img, 16, 16, Color::RED, 16);

        // Circle centre reads back as the drawn green.
        assert_pixel(&img, 48, 48, Color::GREEN, 16);

        // Top-edge line reads back as the drawn blue. rlsw places the topmost line
        // one row in, so probe screen y=1 rather than y=0.
        assert_pixel(&img, 32, 1, Color::BLUE, 16);
    });
}
