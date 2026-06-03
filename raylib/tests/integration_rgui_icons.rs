//! Tier-2: prove the safe `gui_get_icons_mut` accessor aliases raygui's
//! live icon buffer — a mutation made via the safe accessor is observable
//! by a subsequent `gui_draw_icon` call.

#![cfg(all(feature = "software_renderer", feature = "raygui"))]

use raylib::prelude::*;
use raylib::rgui::{RAYGUI_ICON_DATA_ELEMENTS, RAYGUI_ICON_MAX_ICONS};
use raylib::test_harness::{pixel_at, render_frame, with_headless};

#[test]
fn icon_buffer_mutations_are_drawn() {
    const W: i32 = 64;
    const H: i32 = 64;
    // Use an unassigned slot (234..=255 are all-zero in raygui's default
    // guiIcons table; ICON_COLLISION = 233 is the last populated icon).
    const SLOT: usize = 240;

    with_headless(W, H, |rl, thread| {
        // Compile-time consts cross-check.
        assert_eq!(RAYGUI_ICON_MAX_ICONS, 256);
        assert_eq!(RAYGUI_ICON_DATA_ELEMENTS, 8);

        // Set a known custom bitmap: all 256 bits = 1, so the entire 16×16
        // icon area is painted.
        {
            let icons = rl.gui_get_icons_mut();
            icons[SLOT] = [0xFFFF_FFFF; RAYGUI_ICON_DATA_ELEMENTS];
        }

        // Draw the icon at (8, 8), pixel_size=1, in WHITE on BLACK background.
        // `SLOT` is an arbitrary index past the populated ICON_* enum values,
        // so we call the FFI directly rather than constructing a GuiIconName.
        let img = render_frame(rl, thread, |d| {
            d.clear_background(Color::BLACK);
            // SAFETY: SLOT = 240 fits in i32 (no overflow). GuiDrawIcon takes
            // a raw icon-index integer, not a typed enum, so any value in
            // 0..RAYGUI_ICON_MAX_ICONS is valid. The Color literal below uses
            // the raw FFI struct because the draw-handle closure's `d` type
            // does not expose a free-standing gui_draw_icon with an integer
            // index (only a GuiIconName enum variant). The pointer written by
            // gui_get_icons_mut is not accessed here (the borrow has already
            // ended), so there is no aliasing.
            unsafe {
                raylib::ffi::GuiDrawIcon(
                    SLOT as i32,
                    8,
                    8,
                    1,
                    raylib::ffi::Color {
                        r: 255,
                        g: 255,
                        b: 255,
                        a: 255,
                    },
                );
            }
        });

        // Sample inside the icon's drawn area: at least one pixel must be white.
        let mut found_white = false;
        'outer: for y in 8..(8 + 16) {
            for x in 8..(8 + 16) {
                let p = pixel_at(&img, x, y);
                if p.r == 255 && p.g == 255 && p.b == 255 {
                    found_white = true;
                    break 'outer;
                }
            }
        }
        assert!(
            found_white,
            "expected at least one white pixel in the drawn 16×16 icon region \
             after mutating the icon buffer via gui_get_icons_mut"
        );
    });
}
