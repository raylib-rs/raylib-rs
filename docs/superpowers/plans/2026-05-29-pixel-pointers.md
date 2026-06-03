# pixel-pointers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wrap raylib's `GetPixelColor` / `SetPixelColor` C functions with safe, ergonomic Rust signatures over `&[u8]` / `&mut [u8]`, plus a `bytes_per_pixel` helper, with full validation and Tier-1 tests.

**Architecture:** A new focused module `raylib/src/core/pixel.rs` exposing four public items — `PixelColorError` (thiserror), `bytes_per_pixel`, `get_pixel_color`, `set_pixel_color`. Re-exported through `core::*` and `prelude`. The exhaustive `bytes_per_pixel` match arm catches future enum additions at compile time; a private `validate_slice` helper centralizes length + compressed-format checks.

**Tech Stack:** Rust 1.85 (edition 2024), `thiserror` 2.0 (already a dep), `cargo test --lib` (no window needed — Tier-1 only).

**Spec reference:** `docs/superpowers/specs/2026-05-29-pixel-pointers-design.md`.

---

### Task 1: Scaffold the module with the error type and stub function signatures

**Files:**
- Create: `raylib/src/core/pixel.rs`
- Modify: `raylib/src/core/mod.rs`
- Modify: `raylib/src/prelude.rs`

This task gets the module compiling with stub bodies before any tests are written. Stubs return predictable errors so a compile is enough validation.

- [ ] **Step 1: Locate the existing `pub mod` declarations in `raylib/src/core/mod.rs`**

Run: `grep -nE '^pub mod ' raylib/src/core/mod.rs`
Expected output: a block of `pub mod window;`, `pub mod drawing;`, etc. Note the alphabetical convention (if any) so the new `pub mod pixel;` slots in correctly.

- [ ] **Step 2: Locate the existing `pub use` declarations in `raylib/src/prelude.rs`**

Run: `grep -nE '^pub use ' raylib/src/prelude.rs | head -20`
Expected: a series of `pub use crate::core::xxx::*;` re-exports. Note the convention so the new `pub use crate::core::pixel::*;` slots in correctly.

- [ ] **Step 3: Create `raylib/src/core/pixel.rs` with the error type and stubs**

Write the file with this exact content (full module skeleton — tests come in Task 3):

```rust
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
use crate::core::Color;
use crate::ffi;

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
pub fn get_pixel_color(
    bytes: &[u8],
    format: PixelFormat,
) -> Result<Color, PixelColorError> {
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
```

- [ ] **Step 4: Wire the module into `raylib/src/core/mod.rs`**

Find the existing `pub mod` block (per Step 1) and add a line `pub mod pixel;` in alphabetical order. Then add a corresponding `pub use pixel::*;` line (or follow whatever re-export convention you observed — some `core/` modules are `pub use crate::core::texture::*;` etc.).

- [ ] **Step 5: Add the prelude re-exports**

In `raylib/src/prelude.rs`, find the cluster of `pub use crate::core::*` lines and add:

```rust
pub use crate::core::pixel::{
    PixelColorError, bytes_per_pixel, get_pixel_color, set_pixel_color,
};
```

If the prelude pattern is "glob-only" (i.e., it does `pub use crate::core::texture::*;` without naming each item), use `pub use crate::core::pixel::*;` to match the existing convention instead.

- [ ] **Step 6: Verify the workspace compiles**

Run: `cargo build -p raylib --features full 2>&1 | tail -10`
Expected: `Finished` line, no `error` lines. Compilation warnings about unused `validate_slice` are acceptable (it's used by the public fns).

- [ ] **Step 7: Verify clippy is clean on the new module**

Run: `cargo clippy -p raylib --features full -- -D warnings 2>&1 | tail -10`
Expected: no errors. If clippy flags the cast `bytes.as_ptr() as *mut _`, add `#[allow(clippy::cast_ref_to_mut)]` on the `get_pixel_color` function with a comment explaining the FFI signature mismatch — but try without the allow first; clippy may not complain about this specific pattern.

- [ ] **Step 8: Commit Task 1**

Run via the Bash tool (heredoc syntax for the multi-line message):

```bash
git add raylib/src/core/pixel.rs raylib/src/core/mod.rs raylib/src/prelude.rs
git commit -m "$(cat <<'EOF'
feat(pixel): scaffold pixel-pointers module with stubs + error type

New raylib/src/core/pixel.rs surfaces:
- PixelColorError (thiserror with InsufficientBytes + CompressedFormat)
- bytes_per_pixel(PixelFormat) -> Option<usize>     [exhaustive match]
- get_pixel_color(&[u8], PixelFormat) -> Result<Color, _>
- set_pixel_color(&mut [u8], Color, PixelFormat) -> Result<(), _>
- private validate_slice helper

Wired into raylib::core via `pub mod pixel;` + re-export, and into
raylib::prelude so `use raylib::prelude::*;` brings the four public
items into scope.

Implementation is complete (FFI calls + validation); tests follow in
the next tasks. Module-level rustdoc explains when to reach for these
vs Image::get_color, and documents the BGRA-not-supported limitation
that the test_harness BGRA quirk lives outside raylib's PixelFormat
model and cannot be wrapped through these fns.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Add the test infrastructure (shared format table)

**Files:**
- Modify: `raylib/src/core/pixel.rs`

The shared `UNCOMPRESSED_FORMATS` table is referenced by Tasks 3, 4, 5, and 6. Adding it first means subsequent tasks have one fewer "set up boilerplate" step.

- [ ] **Step 1: Append a `#[cfg(test)] mod tests` block to `raylib/src/core/pixel.rs`**

After the closing `}` of `set_pixel_color`, append:

```rust

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
}
```

- [ ] **Step 2: Verify the test module compiles** (no test functions yet, just the const)

Run: `cargo build -p raylib --tests --features full 2>&1 | tail -5`
Expected: `Finished` with no errors.

- [ ] **Step 3: Commit Task 2**

```bash
git add raylib/src/core/pixel.rs
git commit -m "$(cat <<'EOF'
test(pixel): add shared UNCOMPRESSED_FORMATS table for the test module

The const table pairs each of the 13 uncompressed PixelFormat variants
with its bytes-per-pixel count. Used by upcoming round-trip,
bytes_per_pixel cross-check, and trailing-bytes tests so each test is
data-driven and a future enum addition only needs the table updated.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: Tests for `bytes_per_pixel` — cross-check raylib + None for compressed

**Files:**
- Modify: `raylib/src/core/pixel.rs` (the `mod tests` block)

These tests verify the existing implementation; they should pass immediately. The cross-check is the most important — if raylib ever changes a format's byte count, this fires.

- [ ] **Step 1: Add the cross-check test inside `mod tests`**

After the `UNCOMPRESSED_FORMATS` const, add:

```rust
    #[test]
    fn bytes_per_pixel_agrees_with_raylib() {
        for &(format, expected) in UNCOMPRESSED_FORMATS {
            let bpp = bytes_per_pixel(format)
                .unwrap_or_else(|| panic!("{format:?} should be uncompressed"));
            assert_eq!(bpp, expected, "{format:?}: table says {expected}, fn says {bpp}");

            // raylib computes the same byte count via GetPixelDataSize(1, 1, ...).
            let raylib_bpp = unsafe {
                crate::ffi::GetPixelDataSize(1, 1, format as i32)
            } as usize;
            assert_eq!(
                bpp, raylib_bpp,
                "{format:?}: rust says {bpp}, raylib's GetPixelDataSize(1,1,...) says {raylib_bpp}"
            );
        }
    }
```

- [ ] **Step 2: Add the compressed-returns-None test**

Append (still inside `mod tests`):

```rust
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
```

- [ ] **Step 3: Run the two new tests**

Run: `cargo test -p raylib --lib --features full bytes_per_pixel 2>&1 | tail -15`
Expected: `test result: ok. 2 passed; 0 failed`. If `bytes_per_pixel_agrees_with_raylib` fails, the diagnostic message names the offending format — fix the byte count in the match arm in the main `bytes_per_pixel` function.

- [ ] **Step 4: Commit Task 3**

```bash
git add raylib/src/core/pixel.rs
git commit -m "$(cat <<'EOF'
test(pixel): bytes_per_pixel cross-check vs raylib + None-for-compressed

Two tests:
- bytes_per_pixel_agrees_with_raylib iterates UNCOMPRESSED_FORMATS and
  checks each entry against raylib's own GetPixelDataSize(1, 1, F).
  A future raylib format-layout change trips here immediately.
- bytes_per_pixel_none_for_every_compressed_variant lists the 11
  compressed variants explicitly (no wildcard) so an enum addition
  surfaces as a missing entry in the test.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: Error-variant tests (insufficient bytes + compressed format)

**Files:**
- Modify: `raylib/src/core/pixel.rs` (the `mod tests` block)

Six assertions covering the two error variants from both `get_pixel_color` and `set_pixel_color`. Tests pass against the existing implementation.

- [ ] **Step 1: Add the insufficient-bytes test**

Append to `mod tests`:

```rust
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
```

- [ ] **Step 2: Add the compressed-format test**

Append:

```rust
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
```

- [ ] **Step 3: Run the four new tests**

Run: `cargo test -p raylib --lib --features full pixel:: 2>&1 | tail -15`
Expected: 6 tests pass (the 2 from Task 3 + the 4 here).

- [ ] **Step 4: Commit Task 4**

```bash
git add raylib/src/core/pixel.rs
git commit -m "$(cat <<'EOF'
test(pixel): error-variant coverage for get/set_pixel_color

Four #[test] functions covering the two error variants:
- get/set_returns_insufficient_bytes_on_short_slice: empty + 2-byte
  slice against R8G8B8A8 (which needs 4 bytes).
- get/set_returns_compressed_format_for_compressed_input: 8-byte slice
  fed to DXT1_RGB rejects with CompressedFormat regardless of slice
  length — the format validation runs first.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 5: All-FF round-trip — every uncompressed format

**Files:**
- Modify: `raylib/src/core/pixel.rs` (the `mod tests` block)

The "all channels at max" pixel is the easiest round-trip — every uncompressed format quantizes `0xFF` cleanly to its max representable value and back.

- [ ] **Step 1: Add the all-FF round-trip test**

Append to `mod tests`:

```rust
    #[test]
    fn round_trip_all_ff_for_every_uncompressed_format() {
        let input = Color::new(0xFF, 0xFF, 0xFF, 0xFF);
        for &(format, bpp) in UNCOMPRESSED_FORMATS {
            let mut bytes = vec![0u8; bpp];
            set_pixel_color(&mut bytes, input, format).unwrap_or_else(|e| {
                panic!("set_pixel_color({format:?}) errored: {e}")
            });
            let got = get_pixel_color(&bytes, format).unwrap_or_else(|e| {
                panic!("get_pixel_color({format:?}) errored: {e}")
            });
            assert_eq!(
                got, input,
                "{format:?}: all-FF round-trip diverged (got {got:?}, expected {input:?})"
            );
        }
    }
```

- [ ] **Step 2: Run the all-FF test**

Run: `cargo test -p raylib --lib --features full round_trip_all_ff 2>&1 | tail -10`
Expected: `test result: ok. 1 passed`.

If any format fails, the diagnostic names it. The most likely culprit is a misread of the C encoding: open `raylib-sys/raylib/src/rtextures.c` and search for `SetPixelColor` to verify what that variant does with `Color::new(0xFF, 0xFF, 0xFF, 0xFF)`. The all-FF case is structured to round-trip exactly for every variant; a failure indicates either a real raylib bug or a misunderstanding of which channels participate (e.g. `GRAYSCALE` only stores `r` — but `r=0xFF` decodes back as `(0xFF, 0xFF, 0xFF, 0xFF)`, so it's still all-FF).

- [ ] **Step 3: Commit Task 5**

```bash
git add raylib/src/core/pixel.rs
git commit -m "$(cat <<'EOF'
test(pixel): all-FF round-trip for every uncompressed format

For each of the 13 uncompressed PixelFormat variants, encode
Color::new(0xFF, 0xFF, 0xFF, 0xFF) via set_pixel_color and decode via
get_pixel_color; assert exact equality. Max values quantize cleanly
through every variant including the lossy 5/6/4-bit channels (31 → 255
for 5-bit, 63 → 255 for 6-bit, 15 → 255 for 4-bit).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 6: Channel-distinct round-trip with per-format tolerance

**Files:**
- Modify: `raylib/src/core/pixel.rs` (the `mod tests` block)

The channel-distinct pixel `Color::new(0xC0, 0x80, 0x40, 0xFF)` exercises encoding precision. Tolerance varies per format-family. The plan locks specific tolerance values; the implementer should run the test once and tighten any tolerance that's loose.

- [ ] **Step 1: Add a tolerance helper and the channel-distinct test**

Append to `mod tests`:

```rust
    /// Per-channel tolerance for a format's lossy quantization.
    /// Returns (rgb_tol, alpha_tol) where each is the maximum allowed
    /// `|got - expected|` for the given channel.
    fn tolerance(format: PixelFormat) -> (u8, u8) {
        use PixelFormat::*;
        match format {
            // 8-bit channels: exact except for the lossy GRAYSCALE collapse.
            PIXELFORMAT_UNCOMPRESSED_R8G8B8 => (0, 0),
            PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 => (0, 0),
            // 32-bit float channels: ±1 LSB from the `u8 → f32 / 255 → * 255 → u8`
            // truncation (e.g. 192 → 0.7529... → 191.999... → 191).
            PIXELFORMAT_UNCOMPRESSED_R32 => (1, 0),
            PIXELFORMAT_UNCOMPRESSED_R32G32B32 => (1, 0),
            PIXELFORMAT_UNCOMPRESSED_R32G32B32A32 => (1, 1),
            // 16-bit half-float: ±1 LSB from half-float precision.
            PIXELFORMAT_UNCOMPRESSED_R16 => (1, 0),
            PIXELFORMAT_UNCOMPRESSED_R16G16B16 => (1, 0),
            PIXELFORMAT_UNCOMPRESSED_R16G16B16A16 => (1, 1),
            // 5/6/4-bit channels: quantization step roughly 255/(2^k - 1):
            // 5-bit -> step 8 -> tolerance 4 (half-step). Allow 8 to absorb
            // any rounding choice the C code makes.
            PIXELFORMAT_UNCOMPRESSED_R5G6B5 => (8, 0),
            PIXELFORMAT_UNCOMPRESSED_R5G5B5A1 => (8, 0), // alpha is 0 or 255 only
            PIXELFORMAT_UNCOMPRESSED_R4G4B4A4 => (16, 16), // 4-bit step ~17.
            // Grayscale: collapses R/G/B to a single channel; the round-trip
            // produces three identical channels. We don't compare per-channel
            // tolerance for these — they get a separate assertion path.
            PIXELFORMAT_UNCOMPRESSED_GRAYSCALE => (0, 0),
            PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA => (0, 0),
            // Compressed formats can't reach this code path.
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
                        got.r, input.r
                    );
                    assert!(
                        within_tolerance(got.g, input.g, rgb_tol),
                        "{format:?}: G got {} expected {} (tol {rgb_tol})",
                        got.g, input.g
                    );
                    assert!(
                        within_tolerance(got.b, input.b, rgb_tol),
                        "{format:?}: B got {} expected {} (tol {rgb_tol})",
                        got.b, input.b
                    );
                    assert!(
                        within_tolerance(got.a, input.a, a_tol),
                        "{format:?}: A got {} expected {} (tol {a_tol})",
                        got.a, input.a
                    );
                }
            }
        }
    }
```

- [ ] **Step 2: Run the channel-distinct test**

Run: `cargo test -p raylib --lib --features full round_trip_channel_distinct 2>&1 | tail -15`
Expected: `test result: ok. 1 passed`.

If a tolerance is too tight (e.g. some 5-bit format diverges by 9 with the listed tolerance of 8), the diagnostic shows the exact channel + got/expected values. Investigate by reading the corresponding C branch in `raylib-sys/raylib/src/rtextures.c` (search for the variant name; SetPixelColor cases are below the GetPixelColor cases starting around line 5350). Adjust the tolerance to match the actual quantization step. **Don't loosen tolerances generously** — pin them to the minimum value that passes, with a comment if the value is surprising.

If a 16-bit half-float branch is missing in raylib (the half-precision encoding may not have an `SetPixelColor` arm in older raylib versions), the test will fail with "no change" — `get_pixel_color` returns black or zero. In that case, mark the half-float formats as skipped with a `// TODO(raylib upstream): half-float SetPixelColor not implemented` and move on; the all-FF test (Task 5) is still valid because raylib defaults to no-op on missing arms.

- [ ] **Step 3: Commit Task 6**

```bash
git add raylib/src/core/pixel.rs
git commit -m "$(cat <<'EOF'
test(pixel): channel-distinct round-trip with per-format tolerance

Encode Color::new(0xC0, 0x80, 0x40, 0xFF) into every uncompressed
PixelFormat and verify the decoded color matches within a
quantization-step-aware tolerance:

- 8-bit RGB(A) formats: exact (tol 0).
- 32-bit / 16-bit float formats: ±1 LSB (float rounding).
- 5/6-bit channels: ±8 LSBs (5-bit step ~8.2).
- 4-bit channels: ±16 LSBs (4-bit step ~17).
- GRAYSCALE / GRAY_ALPHA: assert the channel-collapse invariant
  (R==G==B post-round-trip) and alpha handling, not specific values.

Tolerances are pinned to the minimum that passes; if any future raylib
quantization change tightens or relaxes them, update the table.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 7: Trailing-bytes-ignored test

**Files:**
- Modify: `raylib/src/core/pixel.rs` (the `mod tests` block)

Quick test to document the "trailing bytes ignored" contract.

- [ ] **Step 1: Add the trailing-bytes test**

Append to `mod tests`:

```rust
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

        let got_exact = get_pixel_color(
            &exact,
            PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        )
        .unwrap();
        let got_long = get_pixel_color(
            &long,
            PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8,
        )
        .unwrap();
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
            assert_eq!(b, sentinel, "byte {} past the pixel must be untouched", i + 4);
        }
    }
```

- [ ] **Step 2: Run the test**

Run: `cargo test -p raylib --lib --features full trailing_bytes 2>&1 | tail -10`
Expected: `test result: ok. 1 passed`.

- [ ] **Step 3: Commit Task 7**

```bash
git add raylib/src/core/pixel.rs
git commit -m "$(cat <<'EOF'
test(pixel): trailing-bytes-ignored contract

set/get_pixel_color must touch only the first bytes_per_pixel(format)
bytes of the slice. The test:
- Confirms get_pixel_color on a 16-byte buffer matches the result on a
  4-byte buffer with the same prefix.
- Confirms set_pixel_color leaves bytes past the pixel untouched
  (verified via a sentinel pattern).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 8: Run the full lib test suite + clippy + final build

**Files:** none (verification only)

- [ ] **Step 1: Full lib tests pass**

Run: `cargo test -p raylib --lib --features full 2>&1 | tail -10`
Expected: every test passes. The pixel module contributes 7 new tests; existing tests should be unaffected. The full count should be 58 (existing pre-pixel) + 7 = 65 or thereabouts. If the previous count was different, just confirm `failed; 0`.

- [ ] **Step 2: Clippy clean**

Run: `cargo clippy -p raylib --features full -- -D warnings 2>&1 | tail -10`
Expected: no warnings. Lints to watch for:
- `clippy::cast_ref_to_mut` on the `as *mut _` casts — if it fires, add `#[allow(clippy::cast_ref_to_mut)]` on the offending function with a 1-line comment pointing at the SAFETY note.
- `clippy::missing_panics_doc` on `unwrap`s in test code — tests are exempt by configuration; ignore unless it fires.

- [ ] **Step 3: Rustdoc clean**

Run: `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps 2>&1 | tail -10`
Expected: clean build. Broken intradoc-links to `Image`, `Image::get_color`, etc. will fail here. If they do, double-check the import paths in the module-level doc.

- [ ] **Step 4: Mdbook still builds (the module isn't referenced from any book chapter, but doctests run from `target/debug/deps`)**

Run: `mdbook build book 2>&1 | tail -5`
Expected: clean (no errors). No book-chapter rewrites needed; pixel-pointers is an internal module, not a user-facing core concept.

---

### Task 9: Update the cheatsheet parity audit + CHANGELOG

**Files:**
- Modify: `docs/superpowers/notes/cheatsheet-parity-audit.md`
- Modify: `CHANGELOG.md`

Now that the gap is filled, move it from §3 (small-gaps table) to §1 (covered list) in the audit, and add CHANGELOG entries.

- [ ] **Step 1: Find the entries for `GetPixelColor` / `SetPixelColor` in the audit**

Run: `grep -n 'GetPixelColor\|SetPixelColor' docs/superpowers/notes/cheatsheet-parity-audit.md | head -10`
Expected: matches in §1's "Image manipulation" or "Color/pixel related" subsection (status 🟥 GAP), in §3's small-fixes table, and in §4's future-workstream list. Each needs a touch.

- [ ] **Step 2: Update §1 entries — flip 🟥 to ✅**

For each `GetPixelColor` / `SetPixelColor` line in §1, change the marker from `🟥` to `✅` and the action from "GAP" to a path reference. The pattern matches the rest of §1's ✅ rows:

Example transformation (the exact line will differ slightly):

```
- 🟥 `GetPixelColor` — GAP. Action: small fix-now (cheatsheet audit §3).
```

→

```
- ✅ `GetPixelColor` — wrapped at `raylib/src/core/pixel.rs::get_pixel_color`.
```

Same pattern for `SetPixelColor`.

- [ ] **Step 3: Update the summary counts in §0**

The §0 summary opens with totals like "Covered: 486" / "Gaps: 13" / "Small fixes: 5". Decrement the gap counts by 2 and bump the covered count by 2:
- Covered: 486 → 488.
- Genuine gaps: 13 → 11.
- Small fixes: 5 → 3.

Also adjust the headline "13, not 49" if the total in-scope gap count changes — re-derive: 13 - 2 = 11.

- [ ] **Step 4: Update §3 (small fixes table)**

Remove the `GetPixelColor` / `SetPixelColor` rows from the small-fixes table at §3. Update the counter at the bottom (was "Five small gaps, ~27 lines total" — those two were ~10 lines; new line: "Three small gaps, ~17 lines total" or similar).

Wait — these were in §4 (future workstream), NOT §3. Re-check by grep. If they were in §4, remove the `pixel-pointers` workstream from §4 entirely and note in §5 reconciliation that this workstream is now ✅ done.

- [ ] **Step 5: Update §4 (future workstreams)**

The §4 entry for `pixel-pointers` (containing `GetPixelColor` / `SetPixelColor`) is now done. Either:
- Remove the §4 #1 entry entirely and renumber `hashes` to #1 and `mixed-audio` to #2, OR
- Keep the entry but prepend `**DONE (commit <SHA>):**` to mark it complete.

Pick whichever is more consistent with the rest of the doc's style. Default to "remove + renumber" if there's no clear precedent.

- [ ] **Step 6: Append CHANGELOG entries**

In `CHANGELOG.md`, under `## 6.0.0-rc.1 (unreleased)` → `### Added`, append:

```markdown
- New module `raylib::core::pixel` with safe wrappers over raylib's
  pixel-pointer C functions:
  - `get_pixel_color(bytes: &[u8], format: PixelFormat) -> Result<Color, PixelColorError>`
  - `set_pixel_color(bytes: &mut [u8], color: Color, format: PixelFormat) -> Result<(), PixelColorError>`
  - `bytes_per_pixel(format: PixelFormat) -> Option<usize>` helper
  - `PixelColorError` (thiserror) with `InsufficientBytes` and `CompressedFormat` variants
  All four are re-exported through `raylib::prelude`.
```

- [ ] **Step 7: Verify the doc/CHANGELOG edits are sane**

Run: `grep -n 'GetPixelColor\|SetPixelColor' docs/superpowers/notes/cheatsheet-parity-audit.md`
Expected: every remaining match is in the ✅ list (§1) or §5 reconciliation notes; no `🟥` rows left.

Run: `grep -n 'pixel::' CHANGELOG.md | head -3`
Expected: the new bullet shows in the 6.0.0-rc.1 block.

- [ ] **Step 8: Commit Task 9**

```bash
git add docs/superpowers/notes/cheatsheet-parity-audit.md CHANGELOG.md
git commit -m "$(cat <<'EOF'
docs(pixel-pointers): mark GetPixelColor / SetPixelColor as covered

The pixel-pointers workstream from cheatsheet-parity-audit.md §4 #1 is
done; reconcile the audit + CHANGELOG accordingly.

- cheatsheet-parity-audit.md:
  - §1 entries for GetPixelColor + SetPixelColor flip 🟥 -> ✅ pointing
    at raylib/src/core/pixel.rs.
  - §0 totals: Covered 486 -> 488, gaps 13 -> 11, small fixes 5 -> 3.
  - §4 workstream #1 (pixel-pointers) removed; remaining items
    renumbered (hashes -> #1, mixed-audio -> #2).
- CHANGELOG.md ## 6.0.0-rc.1 (unreleased) ### Added: lists the new
  raylib::core::pixel module with the four public items.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
```

---

### Task 10: Final push to fork

**Files:** none (push only)

- [ ] **Step 1: Confirm clean working tree before push**

Run: `git status --short`
Expected: only the three permitted untracked files (`TODO.md`, `prompt.md`, `next-session-prompt.md`); no `M` lines.

- [ ] **Step 2: Push to `fork/6.0-rc`**

Run: `git push fork 6.0-rc 2>&1 | tail -3`
Expected: a successful update like `<old-SHA>..<new-SHA>  6.0-rc -> 6.0-rc`.

- [ ] **Step 3: Push to `fork/unstable`**

Run: `git push fork 6.0-rc:unstable 2>&1 | tail -3`
Expected: same SHA range pushed to `unstable`.

- [ ] **Step 4: Update CLAUDE.md status line (pre-WS9 queue head: pixel-pointers ✅, hashes ← NEXT)**

Find the status line (`grep -n 'pixel-pointers' CLAUDE.md` — should match the **Pre-WS9 queue ← NEXT** block). Replace the existing `pixel-pointers → hashes → mixed-audio` prefix with `pixel-pointers ✅ → hashes ← NEXT → mixed-audio → ...`.

Commit + push:

```bash
git add CLAUDE.md
git commit -m "$(cat <<'EOF'
docs(roadmap): pixel-pointers complete; hashes is next

CLAUDE.md status line: pre-WS9 queue head flips from pixel-pointers to
hashes. See docs/superpowers/notes/cheatsheet-parity-audit.md for the
updated cheatsheet coverage (488 / 500 in-scope).

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
EOF
)"
git push fork 6.0-rc
git push fork 6.0-rc:unstable
```

---

## pixel-pointers complete when

- [ ] `raylib/src/core/pixel.rs` exists with the four public items + private `validate_slice` + module-level rustdoc explaining the BGRA limitation.
- [ ] `raylib/src/core/mod.rs` declares `pub mod pixel;` and re-exports.
- [ ] `raylib/src/prelude.rs` includes the four public items.
- [ ] `bytes_per_pixel` uses an exhaustive match (no wildcard).
- [ ] 7 new tests pass under `cargo test -p raylib --lib --features full`.
- [ ] `cargo build --workspace --features full` clean.
- [ ] `cargo clippy -p raylib --features full -- -D warnings` clean.
- [ ] `RUSTDOCFLAGS="-Dwarnings" cargo doc -p raylib --features full --no-deps` clean.
- [ ] `mdbook build book` clean.
- [ ] `cheatsheet-parity-audit.md` reconciled: §0 totals, §1 ✅ rows, §4 workstream removed.
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Added` lists the new module + its four public items.
- [ ] `CLAUDE.md` status line: pre-WS9 queue head flips `pixel-pointers ✅ → hashes ← NEXT`.
- [ ] All commits include the `Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>` trailer.
- [ ] Pushed to both `fork/6.0-rc` and `fork/unstable`.

Next workstream: **`hashes`** (cheatsheet audit §4 #1 after renumbering — `Compute{CRC32,MD5,SHA1,SHA256}`). Brainstorm starts when the owner is ready.
