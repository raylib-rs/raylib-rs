//! Headless render-test harness (`software_renderer` / `Platform::Memory`
//! backend).
//!
//! Tier-2 tests use this module to render frames without opening a window. The
//! `software_renderer` feature activates the rlsw memory platform, which
//! writes pixels into a CPU-side framebuffer instead of an on-screen surface.
//! Because raylib is single-init per process, a test file initialises the
//! context **once** via [`with_headless`], executes draws inside the closure,
//! then probes pixel values with [`pixel_at`] / [`assert_pixel`].
//!
//! ## Readback format
//!
//! [`render_frame`] returns a **normalised** top-left RGBA image: the harness
//! corrects the raw rlsw readback (which is BGRA bytes and Y-inverted) so
//! callers use natural draw coordinates and colours — a `Color::RED` rectangle
//! drawn at screen `(x, y)` reads back as red at `(x, y)`. Use
//! [`render_frame_raw`] if you specifically need the un-normalised BGRA +
//! Y-inverted buffer.
//!
//! ## All-modules link requirement
//!
//! The harness links all five raylib content modules:
//! `SUPPORT_MODULE_RTEXTURES`, `SUPPORT_MODULE_RSHAPES`,
//! `SUPPORT_MODULE_RTEXT`, `SUPPORT_MODULE_RMODELS`, and
//! `SUPPORT_MODULE_RAUDIO`. Omitting any of them — or omitting
//! `SUPPORT_IMAGE_GENERATION` (required on MSVC) — produces link errors.
//! The `software_renderer` feature in `raylib-sys` enables the correct set;
//! do not mix `software_renderer` with individual module-disable flags.
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
/// **Call at most once per test process.**
///
/// # Panics
///
/// Panics if raylib has already been initialised in this process; raylib is
/// single-init per process (enforced by `RaylibHandle`).
pub fn with_headless<F: FnOnce(&mut RaylibHandle, &RaylibThread)>(w: i32, h: i32, body: F) {
    let (mut rl, thread) = crate::init().size(w, h).title("headless").build();
    body(&mut rl, &thread);
    // RaylibHandle's Drop calls CloseWindow.
}

/// Draw one frame via the `draw` closure and read the software framebuffer back
/// **without normalization** — the returned [`Image`] is in raw rlsw readback
/// form: bytes are BGRA (R and B swapped vs. what was drawn) and rows are
/// Y-inverted (`y_img = (h - 1) - y_screen`). Prefer [`render_frame`] unless you
/// specifically need the raw readback; see `notes/ws4b-complete.md`.
///
/// Uses the scoped [`begin_drawing`](RaylibHandle::begin_drawing) handle so the
/// `EndDrawing` flush on drop is guaranteed to complete (flushing rlsw into the
/// memory framebuffer) before
/// [`load_image_from_screen`](RaylibHandle::load_image_from_screen) reads it.
#[inline]
#[must_use]
pub fn render_frame_raw<F: FnOnce(&mut RaylibDrawHandle<'_>)>(
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

/// Draw one frame via the `draw` closure, then read the software framebuffer
/// back as a **normalized** top-left RGBA [`Image`].
///
/// The raw rlsw readback is BGRA + Y-inverted (deterministic on every OS; see
/// `notes/ws4b-complete.md`). This function corrects both so callers use natural
/// draw coordinates and colors: a `Color::RED` rectangle reads back as red at the
/// screen `(x, y)` it was drawn at. Use [`render_frame_raw`] for the un-normalized
/// buffer.
#[inline]
#[must_use]
pub fn render_frame<F: FnOnce(&mut RaylibDrawHandle<'_>)>(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    draw: F,
) -> Image {
    let mut img = render_frame_raw(rl, thread, draw);
    normalize_readback(&mut img);
    img
}

/// Correct the rlsw readback in place: flip vertically and correct the
/// BGRA-as-RGBA byte order, yielding a true top-left RGBA image.
fn normalize_readback(img: &mut Image) {
    // Fix Y-inversion using the existing safe image op. NOTE: `flip_vertical`
    // (ImageFlipVertical) replaces the underlying data buffer, so the raw pointer
    // must be taken *after* this call — which it is, below.
    img.flip_vertical();

    // Correct the BGRA-as-RGBA byte order by swapping byte 0 (R) and byte 2 (B)
    // of every 4-byte pixel. The readback is always
    // PIXELFORMAT_UNCOMPRESSED_R8G8B8A8; the channel swap is only sound for that
    // 4-bytes-per-pixel layout (asserted below).
    debug_assert_eq!(
        img.format(),
        crate::consts::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        "normalize_readback assumes 32-bit RGBA readback"
    );
    let len = (img.width() * img.height()) as usize * 4;
    if len == 0 {
        // A 0×0 image leaves `data` unallocated (possibly null); skip — and avoid
        // `from_raw_parts_mut(null, 0)`, which is UB.
        return;
    }
    // SAFETY: `img.data()` is taken after `flip_vertical`, so it points at the
    // current, freshly allocated buffer, valid for `img`'s lifetime. The readback
    // image is PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 (debug-asserted above), so the
    // buffer is exactly `width * height * 4 == len` valid, initialized bytes, and
    // `len > 0` here so the pointer is non-null. `img` is not accessed through its
    // `&mut` while `bytes` is live, so the slice is the only handle to the data
    // (no aliasing). We only swap two in-bounds bytes per 4-byte chunk.
    unsafe {
        let bytes = std::slice::from_raw_parts_mut(img.data() as *mut u8, len);
        for px in bytes.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }
}

/// A single RGBA pixel read from a framebuffer [`Image`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Px {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha as reported by the framebuffer. On the Memory platform this may
    /// not match the drawn `Color`'s alpha, so [`assert_pixel`] ignores it;
    /// don't assert on this field directly.
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

/// Assert that pixel `(x, y)` in `img` matches `expected` within a per-channel
/// tolerance of `tol`.
///
/// Only the R, G, and B channels are compared. Alpha is intentionally ignored:
/// the Memory-platform framebuffer readback can report alpha values that don't
/// match an opaque `Color`'s `a`, which would make probes fail spuriously.
///
/// Panics with a descriptive message showing the actual vs expected pixel and
/// position on failure.
#[inline]
pub fn assert_pixel(img: &Image, x: i32, y: i32, expected: Color, tol: u8) {
    let actual = pixel_at(img, x, y);
    let close = |a: u8, b: u8| a.abs_diff(b) <= tol;
    if !close(actual.r, expected.r) || !close(actual.g, expected.g) || !close(actual.b, expected.b)
    {
        panic!(
            "pixel ({x}, {y}): expected ~({r}, {g}, {b}) ± {tol}, \
             got ({ar}, {ag}, {ab})",
            r = expected.r,
            g = expected.g,
            b = expected.b,
            ar = actual.r,
            ag = actual.g,
            ab = actual.b,
        );
    }
}
