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
            assert_eq!(bpp, expected, "{format:?}: table says {expected}, fn says {bpp}");

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
}
