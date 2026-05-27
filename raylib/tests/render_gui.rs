//! WS5a Tier-2: raygui controls render into the software framebuffer. Headless.
//! Uses the WS5-prep normalized `render_frame` (natural top-left coords/colors).
#![cfg(all(feature = "software_renderer", feature = "raygui"))]
use raylib::prelude::*;

#[test]
fn gui_controls_render_pixels() {
    raylib::test_harness::with_headless(200, 120, |rl, thread| {
        // Pin a known state so probes don't depend on prior style mutations.
        rl.gui_set_state(raylib::consts::GuiState::STATE_NORMAL);

        let img = raylib::test_harness::render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            // A button draws a filled rect + border + centered label text.
            let _ = d.gui_button(Rectangle::new(20.0, 20.0, 160.0, 40.0), "OK");
            // A label draws text only.
            let _ = d.gui_label(Rectangle::new(20.0, 80.0, 160.0, 20.0), "hello");
        });

        // The button fill differs from the black background. Count non-black pixels
        // inside the button region to confirm it actually drew (a blank frame = 0).
        let mut drawn = 0u32;
        for y in 20..60 {
            for x in 20..180 {
                let p = raylib::test_harness::pixel_at(&img, x, y);
                if p.r as u16 + p.g as u16 + p.b as u16 > 30 {
                    drawn += 1;
                }
            }
        }
        // Observed locally: the raygui default-style button fill covers the large
        // majority of its 160x40 = 6400-pixel region. Require well above a
        // degenerate handful but safely under the full area.
        assert!(
            drawn > 1000,
            "expected the button to fill many pixels, got {drawn}"
        );
    });
}
