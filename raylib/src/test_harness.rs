//! Headless render-test harness (software_renderer / Memory platform).
//!
//! raylib is single-init per process, so a Tier-2 test file inits **once** via
//! [`with_headless`], draws inside the closure, then probes the framebuffer.
//!
//! # Example
//!
//! ```no_run
//! use raylib::test_harness::*;
//!
//! with_headless(320, 240, |rl, thread| {
//!     let img = render_frame(rl, thread, |d| {
//!         d.clear_background(raylib::prelude::Color::BLACK);
//!     });
//!     assert_pixel(&img, 0, 0, raylib::prelude::Color::BLACK, 0);
//! });
//! ```

use crate::core::drawing::RaylibDrawHandle;
use crate::prelude::*;

/// Initialise a windowless, software-rendered context of `w`×`h`, run
/// `body` with the handle and thread token, then tear down.
///
/// **Call at most once per test process.** raylib is single-init per process;
/// calling this more than once will panic at the raylib level.
pub fn with_headless<F: FnOnce(&mut RaylibHandle, &RaylibThread)>(w: i32, h: i32, body: F) {
    let (mut rl, thread) = crate::init().size(w, h).title("headless").build();
    body(&mut rl, &thread);
    // RaylibHandle's Drop calls CloseWindow.
}

/// Draw one frame via the `draw` closure, then read the software framebuffer
/// back as an [`Image`].
///
/// `EndDrawing`-on-drop flushes rlsw into the memory framebuffer before
/// [`load_image_from_screen`](RaylibHandle::load_image_from_screen) reads it.
#[must_use]
pub fn render_frame<F: FnOnce(&mut RaylibDrawHandle<'_>)>(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    draw: F,
) -> Image {
    {
        let mut d = rl.begin_drawing(thread);
        draw(&mut d);
    } // Drop here calls EndDrawing, flushing rlsw into the memory framebuffer.
    rl.load_image_from_screen(thread)
}

/// A single RGBA pixel read from a framebuffer [`Image`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Px {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Read the pixel at `(x, y)` from `img` as a [`Px`].
#[inline]
#[must_use]
pub fn pixel_at(img: &Image, x: i32, y: i32) -> Px {
    let c = img.get_color(x, y);
    Px {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

/// Assert that pixel `(x, y)` in `img` matches `expected` within a
/// per-channel tolerance of `tol`.
///
/// Panics with a descriptive message showing the actual vs expected pixel and
/// position on failure.
#[inline]
pub fn assert_pixel(img: &Image, x: i32, y: i32, expected: Color, tol: u8) {
    let actual = pixel_at(img, x, y);
    let close = |a: u8, b: u8| a.abs_diff(b) <= tol;
    if !close(actual.r, expected.r)
        || !close(actual.g, expected.g)
        || !close(actual.b, expected.b)
        || !close(actual.a, expected.a)
    {
        panic!(
            "pixel ({x}, {y}): expected ({r}, {g}, {b}, {a}) ± {tol}, \
             got ({ar}, {ag}, {ab}, {aa})",
            r = expected.r,
            g = expected.g,
            b = expected.b,
            a = expected.a,
            ar = actual.r,
            ag = actual.g,
            ab = actual.b,
            aa = actual.a,
        );
    }
}
