//! WS4b Tier-2: drawn text produces foreground pixels within its bounds. Headless.
#![cfg(feature = "software_renderer")]
use raylib::prelude::*;
use raylib::test_harness::{render_frame, with_headless};

#[test]
fn text_draws_foreground_pixels() {
    with_headless(120, 32, |rl, thread| {
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            d.draw_text("Hi", 4, 4, 20, Color::WHITE);
        });
        // Count near-white pixels in the text region; glyphs must mark some.
        let mut white = 0;
        for y in 0..img.height() {
            for x in 0..40.min(img.width()) {
                let c = img.get_color(x, y);
                if c.r > 180 && c.g > 180 && c.b > 180 {
                    white += 1;
                }
            }
        }
        assert!(
            white > 10,
            "expected drawn glyph pixels, found {white} near-white"
        );
    });
}
