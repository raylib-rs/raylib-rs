//! Per-pixel reads and writes into raw byte buffers in a given [`PixelFormat`].
//!
//! These wrap raylib's `GetPixelColor` / `SetPixelColor` C functions and are
//! the right tools when you have a byte buffer (e.g. one you're about to
//! upload to a [`Texture2D`](crate::core::texture::Texture2D), or a CPU
//! framebuffer you generated yourself) and want to read or write a single
//! pixel value while staying format-aware.
//!
//! If you instead have an [`Image`](crate::core::texture::Image) and want
//! the pixel at coordinates `(x, y)`, use
//! [`Image::get_color`](crate::core::texture::Image::get_color) — that goes
//! through raylib's separate `GetImageColor` C function and handles the row
//! stride for you.
//!
//! # BGRA limitation
//!
//! raylib's [`PixelFormat`] enum has no `B8G8R8A8` variant. The rlsw
//! software-renderer test harness stores its framebuffer as BGRA bytes
//! while labeling them with `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8`, and the
//! channel swap is expressed as a bespoke byte-swap loop in
//! `raylib::test_harness::normalize_readback`. These functions cannot
//! eliminate that loop — feeding them BGRA bytes labeled as
//! `R8G8B8A8` returns a `Color` with R and B swapped, which is the same
//! problem.

use crate::consts::PixelFormat;
use crate::ffi;
use crate::ffi::Color;

/// Errors returned by [`get_pixel_color`] / [`set_pixel_color`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PixelColorError {
    /// The byte slice is shorter than the format requires.
    #[error("pixel format {format:?} needs {expected} bytes per pixel, got {actual}")]
    InsufficientBytes {
        /// The format the call was made against.
        format: PixelFormat,
        /// Bytes required for one pixel in this format.
        expected: usize,
        /// Bytes actually provided.
        actual: usize,
    },
    /// Compressed formats are block-addressed; they cannot be read or
    /// written one pixel at a time. raylib's `GetPixelColor` does not
    /// support them either.
    #[error("compressed pixel format {0:?} cannot be addressed pixel-by-pixel")]
    CompressedFormat(PixelFormat),
}

/// Bytes per single pixel for an uncompressed [`PixelFormat`].
///
/// Returns `None` for compressed variants — they are addressed by
/// block, not pixel.
///
/// The match is exhaustive (no wildcard). If raylib adds a new
/// `PixelFormat` variant in a future version, this function will fail
/// to compile, surfacing the change rather than silently returning
/// `None`.
pub fn bytes_per_pixel(format: PixelFormat) -> Option<usize> {
    use PixelFormat::*;
    Some(match format {
        PIXELFORMAT_UNCOMPRESSED_GRAYSCALE => 1,
        PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA => 2,
        PIXELFORMAT_UNCOMPRESSED_R5G6B5 => 2,
        PIXELFORMAT_UNCOMPRESSED_R5G5B5A1 => 2,
        PIXELFORMAT_UNCOMPRESSED_R4G4B4A4 => 2,
        PIXELFORMAT_UNCOMPRESSED_R8G8B8 => 3,
        PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 => 4,
        PIXELFORMAT_UNCOMPRESSED_R32 => 4,
        PIXELFORMAT_UNCOMPRESSED_R32G32B32 => 12,
        PIXELFORMAT_UNCOMPRESSED_R32G32B32A32 => 16,
        PIXELFORMAT_UNCOMPRESSED_R16 => 2,
        PIXELFORMAT_UNCOMPRESSED_R16G16B16 => 6,
        PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 => 8,
        PIXELFORMAT_COMPRESSED_DXT1_RGB
        | PIXELFORMAT_COMPRESSED_DXT1_RGBA
        | PIXELFORMAT_COMPRESSED_DXT3_RGBA
        | PIXELFORMAT_COMPRESSED_DXT5_RGBA
        | PIXELFORMAT_COMPRESSED_ETC1_RGB
        | PIXELFORMAT_COMPRESSED_ETC2_RGB
        | PIXELFORMAT_COMPRESSED_ETC2_EAC_RGBA
        | PIXELFORMAT_COMPRESSED_PVRT_RGB
        | PIXELFORMAT_COMPRESSED_PVRT_RGBA
        | PIXELFORMAT_COMPRESSED_ASTC_4x4_RGBA
        | PIXELFORMAT_COMPRESSED_ASTC_8x8_RGBA => return None,
    })
}

/// Validate that `len` bytes is enough for one pixel in `format` and
/// that `format` is uncompressed. Returns the required byte count on
/// success.
fn validate_slice(len: usize, format: PixelFormat) -> Result<usize, PixelColorError> {
    match bytes_per_pixel(format) {
        None => Err(PixelColorError::CompressedFormat(format)),
        Some(expected) if len < expected => Err(PixelColorError::InsufficientBytes {
            format,
            expected,
            actual: len,
        }),
        Some(expected) => Ok(expected),
    }
}

/// Read a single pixel from `bytes` interpreted as `format`.
///
/// `bytes` must contain at least `bytes_per_pixel(format)?` bytes; any
/// trailing bytes are ignored. Returns
/// [`PixelColorError::CompressedFormat`] for block-compressed formats,
/// or [`PixelColorError::InsufficientBytes`] when the slice is too
/// short.
pub fn get_pixel_color(bytes: &[u8], format: PixelFormat) -> Result<Color, PixelColorError> {
    validate_slice(bytes.len(), format)?;
    // SAFETY: validate_slice ensured `bytes.len() >= bytes_per_pixel(format)`,
    // so raylib reads only within the slice. GetPixelColor's signature is
    // `void *` even though the C body only reads from srcPtr (verified in
    // raylib-sys/raylib/src/rtextures.c around line 5174), so casting
    // `*const u8` to `*mut u8` is sound.
    Ok(unsafe { ffi::GetPixelColor(bytes.as_ptr() as *mut _, format as i32) })
}

/// Write `color` into `bytes` encoded as `format`.
///
/// Same length / format-validity rules as [`get_pixel_color`].
/// Trailing bytes beyond `bytes_per_pixel(format)?` are not touched.
pub fn set_pixel_color(
    bytes: &mut [u8],
    color: Color,
    format: PixelFormat,
) -> Result<(), PixelColorError> {
    validate_slice(bytes.len(), format)?;
    // SAFETY: validate_slice ensured `bytes.len() >= bytes_per_pixel(format)`,
    // so raylib writes only within the slice. SetPixelColor writes exactly
    // `bytes_per_pixel(format)` bytes starting at dstPtr.
    unsafe {
        ffi::SetPixelColor(bytes.as_mut_ptr() as *mut _, color, format as i32);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::PixelFormat;

    /// Every uncompressed PixelFormat paired with its bytes-per-pixel
    /// count. Used by the round-trip tests, the bytes_per_pixel
    /// cross-check, and the trailing-bytes test.
    const UNCOMPRESSED_FORMATS: &[(PixelFormat, usize)] = &[
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAYSCALE, 1),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA, 2),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R5G6B5, 2),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R5G5B5A1, 2),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R4G4B4A4, 2),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8, 3),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8, 4),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32, 4),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32G32B32, 12),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R32G32B32A32, 16),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16, 2),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16, 6),
        (PixelFormat::PIXELFORMAT_UNCOMPRESSED_R16G16B16A16, 8),
    ];

    #[test]
    fn bytes_per_pixel_agrees_with_raylib() {
        for &(format, expected) in UNCOMPRESSED_FORMATS {
            let bpp = bytes_per_pixel(format)
                .unwrap_or_else(|| panic!("{format:?} should be uncompressed"));
            assert_eq!(
                bpp, expected,
                "{format:?}: table says {expected}, fn says {bpp}"
            );

            // raylib computes the same byte count via GetPixelDataSize(1, 1, ...).
            let raylib_bpp = unsafe { crate::ffi::GetPixelDataSize(1, 1, format as i32) } as usize;
            assert_eq!(
                bpp, raylib_bpp,
                "{format:?}: rust says {bpp}, raylib's GetPixelDataSize(1,1,...) says {raylib_bpp}"
            );
        }
    }

    #[test]
    fn bytes_per_pixel_none_for_every_compressed_variant() {
        // Listed exhaustively (no for-loop over a slice) so an enum addition
        // surfaces as a missing arm here, matching the exhaustive-match
        // pattern in bytes_per_pixel itself.
        use PixelFormat::*;
        for format in [
            PIXELFORMAT_COMPRESSED_DXT1_RGB,
            PIXELFORMAT_COMPRESSED_DXT1_RGBA,
            PIXELFORMAT_COMPRESSED_DXT3_RGBA,
            PIXELFORMAT_COMPRESSED_DXT5_RGBA,
            PIXELFORMAT_COMPRESSED_ETC1_RGB,
            PIXELFORMAT_COMPRESSED_ETC2_RGB,
            PIXELFORMAT_COMPRESSED_ETC2_EAC_RGBA,
            PIXELFORMAT_COMPRESSED_PVRT_RGB,
            PIXELFORMAT_COMPRESSED_PVRT_RGBA,
            PIXELFORMAT_COMPRESSED_ASTC_4x4_RGBA,
            PIXELFORMAT_COMPRESSED_ASTC_8x8_RGBA,
        ] {
            assert_eq!(
                bytes_per_pixel(format),
                None,
                "{format:?} should return None"
            );
        }
    }

    #[test]
    fn get_returns_insufficient_bytes_on_short_slice() {
        // R8G8B8A8 needs 4 bytes.
        let empty = &[] as &[u8];
        assert_eq!(
            get_pixel_color(empty, PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8),
            Err(PixelColorError::InsufficientBytes {
                format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
                expected: 4,
                actual: 0,
            }),
        );

        let two_bytes = [0u8; 2];
        assert_eq!(
            get_pixel_color(&two_bytes, PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8),
            Err(PixelColorError::InsufficientBytes {
                format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
                expected: 4,
                actual: 2,
            }),
        );
    }

    #[test]
    fn set_returns_insufficient_bytes_on_short_slice() {
        let mut empty: Vec<u8> = vec![];
        assert_eq!(
            set_pixel_color(
                &mut empty,
                Color::new(255, 0, 0, 255),
                PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
            ),
            Err(PixelColorError::InsufficientBytes {
                format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
                expected: 4,
                actual: 0,
            }),
        );

        let mut two_bytes = [0u8; 2];
        assert_eq!(
            set_pixel_color(
                &mut two_bytes,
                Color::new(255, 0, 0, 255),
                PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
            ),
            Err(PixelColorError::InsufficientBytes {
                format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
                expected: 4,
                actual: 2,
            }),
        );
    }

    #[test]
    fn get_returns_compressed_format_for_compressed_input() {
        let bytes = [0u8; 8];
        assert_eq!(
            get_pixel_color(&bytes, PixelFormat::PIXELFORMAT_COMPRESSED_DXT1_RGB),
            Err(PixelColorError::CompressedFormat(
                PixelFormat::PIXELFORMAT_COMPRESSED_DXT1_RGB
            )),
        );
    }

    #[test]
    fn set_returns_compressed_format_for_compressed_input() {
        let mut bytes = [0u8; 8];
        assert_eq!(
            set_pixel_color(
                &mut bytes,
                Color::new(0, 0, 0, 0),
                PixelFormat::PIXELFORMAT_COMPRESSED_DXT1_RGB,
            ),
            Err(PixelColorError::CompressedFormat(
                PixelFormat::PIXELFORMAT_COMPRESSED_DXT1_RGB
            )),
        );
    }

    /// True for PixelFormat variants where raylib's `SetPixelColor` has no
    /// `case` branch (the function falls through to `default: break` and
    /// leaves the destination unchanged). Verified against
    /// `raylib-sys/raylib/src/rtextures.c` around line 5269: the switch
    /// only covers GRAYSCALE, GRAY_ALPHA, R5G6B5, R5G5B5A1, R4G4B4A4,
    /// R8G8B8, and R8G8B8A8. The 32-bit float and 16-bit half-float
    /// formats are read-only — `GetPixelColor` handles them but
    /// `SetPixelColor` does not. Round-trip tests must skip these.
    fn set_pixel_color_is_unimplemented(format: PixelFormat) -> bool {
        use PixelFormat::*;
        matches!(
            format,
            PIXELFORMAT_UNCOMPRESSED_R32
                | PIXELFORMAT_UNCOMPRESSED_R32G32B32
                | PIXELFORMAT_UNCOMPRESSED_R32G32B32A32
                | PIXELFORMAT_UNCOMPRESSED_R16
                | PIXELFORMAT_UNCOMPRESSED_R16G16B16
                | PIXELFORMAT_UNCOMPRESSED_R16G16B16A16
        )
    }

    #[test]
    fn round_trip_all_ff_for_every_uncompressed_format() {
        let input = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
        for &(format, bpp) in UNCOMPRESSED_FORMATS {
            // TODO(raylib upstream): SetPixelColor has no branch for the
            // 32-bit float / 16-bit half-float formats — see
            // set_pixel_color_is_unimplemented for the source reference.
            // Skipping avoids a false-positive "round-trip diverged" failure;
            // the error-variant tests still cover these formats.
            if set_pixel_color_is_unimplemented(format) {
                continue;
            }
            let mut bytes = vec![0u8; bpp];
            set_pixel_color(&mut bytes, input, format)
                .unwrap_or_else(|e| panic!("set_pixel_color({format:?}) errored: {e}"));
            let got = get_pixel_color(&bytes, format)
                .unwrap_or_else(|e| panic!("get_pixel_color({format:?}) errored: {e}"));
            assert_eq!(
                got, input,
                "{format:?}: all-FF round-trip diverged (got {got:?}, expected {input:?})"
            );
        }
    }

    /// Per-channel tolerance for a format's lossy quantization.
    /// Returns (rgb_tol, alpha_tol) where each is the maximum allowed
    /// `|got - expected|` for the given channel.
    fn tolerance(format: PixelFormat) -> (u8, u8) {
        use PixelFormat::*;
        match format {
            // 8-bit channels: exact (no quantization).
            PIXELFORMAT_UNCOMPRESSED_R8G8B8 => (0, 0),
            PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 => (0, 0),
            // 5/6-bit channels: SetPixelColor uses round() then GetPixelColor
            // uses integer / 255/31 division — round-trip for 0xC0 (192) lands
            // at 189 (delta 3), 0x80 (128) lands at 132 (delta 4 — 6-bit),
            // 0x40 (64) lands at 66 (delta 2). Tolerance 4 covers both
            // channels. Alpha is 0/255-only for R5G5B5A1.
            PIXELFORMAT_UNCOMPRESSED_R5G6B5 => (4, 0),
            // R5G5B5A1: raylib's GetPixelColor decodes blue with the mask
            // `val & 0x1F` (rtextures.c GetPixelColor case PIXELFORMAT_UNCOMPRESSED_R5G5B5A1),
            // which includes the alpha bit at position 0. When alpha=1,
            // blue's decoded value is `((b_quantized << 1) | 1) * 255 / 31`
            // instead of `b_quantized * 255 / 31` — roughly doubling the
            // decoded blue. Worst case is non-saturated blue with alpha=1:
            // input b=0x40 (64) decodes to 139 (delta 75). The tolerance
            // here absorbs that raylib quirk; tightening it would require
            // either an upstream fix or special-casing per channel.
            PIXELFORMAT_UNCOMPRESSED_R5G5B5A1 => (80, 0),
            // 4-bit channels: step ~17. Round-trip 0xC0 -> 204 (delta 12),
            // 0x80 -> 136 (delta 8), 0x40 -> 68 (delta 4), 0xFF -> 255 (0).
            PIXELFORMAT_UNCOMPRESSED_R4G4B4A4 => (12, 0),
            // Grayscale: collapses R/G/B to a single channel; the round-trip
            // produces three identical channels. We don't compare per-channel
            // tolerance for these — they get a separate assertion path.
            PIXELFORMAT_UNCOMPRESSED_GRAYSCALE => (0, 0),
            PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA => (0, 0),
            // Float / half-float formats are unreachable here — they have no
            // SetPixelColor branch, see set_pixel_color_is_unimplemented.
            // Compressed formats can't reach this code path either.
            f => unreachable!("tolerance() called with non-uncompressed format {f:?}"),
        }
    }

    fn within_tolerance(got: u8, expected: u8, tol: u8) -> bool {
        got.abs_diff(expected) <= tol
    }

    #[test]
    fn round_trip_channel_distinct_with_tolerance() {
        let input = Color::new(0xC0, 0x80, 0x40, 0xFF);
        for &(format, bpp) in UNCOMPRESSED_FORMATS {
            // TODO(raylib upstream): SetPixelColor has no branch for the
            // 32-bit float / 16-bit half-float formats — skip them here.
            if set_pixel_color_is_unimplemented(format) {
                continue;
            }
            let mut bytes = vec![0u8; bpp];
            set_pixel_color(&mut bytes, input, format).unwrap();
            let got = get_pixel_color(&bytes, format).unwrap();

            use PixelFormat::*;
            match format {
                // Grayscale collapses R/G/B; the round-trip's three RGB
                // channels are identical to each other, and alpha is fixed
                // to 255 (GRAYSCALE) or input.a (GRAY_ALPHA).
                PIXELFORMAT_UNCOMPRESSED_GRAYSCALE => {
                    assert_eq!(got.r, got.g, "{format:?}: R==G after collapse");
                    assert_eq!(got.g, got.b, "{format:?}: G==B after collapse");
                    assert_eq!(got.a, 255, "{format:?}: alpha forced to 255");
                }
                PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA => {
                    assert_eq!(got.r, got.g, "{format:?}: R==G after collapse");
                    assert_eq!(got.g, got.b, "{format:?}: G==B after collapse");
                    assert_eq!(got.a, input.a, "{format:?}: alpha exact");
                }
                _ => {
                    let (rgb_tol, a_tol) = tolerance(format);
                    assert!(
                        within_tolerance(got.r, input.r, rgb_tol),
                        "{format:?}: R got {} expected {} (tol {rgb_tol})",
                        got.r,
                        input.r
                    );
                    assert!(
                        within_tolerance(got.g, input.g, rgb_tol),
                        "{format:?}: G got {} expected {} (tol {rgb_tol})",
                        got.g,
                        input.g
                    );
                    assert!(
                        within_tolerance(got.b, input.b, rgb_tol),
                        "{format:?}: B got {} expected {} (tol {rgb_tol})",
                        got.b,
                        input.b
                    );
                    assert!(
                        within_tolerance(got.a, input.a, a_tol),
                        "{format:?}: A got {} expected {} (tol {a_tol})",
                        got.a,
                        input.a
                    );
                }
            }
        }
    }

    #[test]
    fn trailing_bytes_are_ignored() {
        // 64-byte buffer fed to a 4-byte format must produce the same result
        // as exactly 4 bytes.
        let exact = [0x11, 0x22, 0x33, 0x44];
        let long = [
            0x11, 0x22, 0x33, 0x44, // first pixel
            0xAA, 0xBB, 0xCC, 0xDD, // these bytes must be ignored
            0xEE, 0xFF, 0x00, 0x11, //
            0x22, 0x33, 0x44, 0x55, //
        ];

        let got_exact =
            get_pixel_color(&exact, PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8).unwrap();
        let got_long =
            get_pixel_color(&long, PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8).unwrap();
        assert_eq!(got_exact, got_long, "trailing bytes must not affect read");

        // Symmetric check for set: writing to a too-long buffer touches
        // only the first N bytes.
        let mut exact_out = [0u8; 4];
        let mut long_out = [0u8; 16];
        let sentinel = 0xA5;
        long_out[4..].fill(sentinel);
        let color = Color::new(0x11, 0x22, 0x33, 0x44);

        set_pixel_color(
            &mut exact_out,
            color,
            PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        )
        .unwrap();
        set_pixel_color(
            &mut long_out,
            color,
            PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        )
        .unwrap();

        assert_eq!(&long_out[..4], &exact_out, "first 4 bytes must match");
        for (i, &b) in long_out[4..].iter().enumerate() {
            assert_eq!(
                b,
                sentinel,
                "byte {} past the pixel must be untouched",
                i + 4
            );
        }
    }
}
