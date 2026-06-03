# pixel-pointers — safe wrappers for `GetPixelColor` / `SetPixelColor`

**Status:** design approved 2026-05-29. First pre-WS9 workstream in the
owner-locked queue (per `docs/superpowers/notes/ws8e-checkpoint-review-feedback.md`).

The raylib 6.0 cheatsheet parity audit
(`docs/superpowers/notes/cheatsheet-parity-audit.md` §4 #1) called out
`GetPixelColor` / `SetPixelColor` as a gap that needs a small design
pass — the C signatures take a `void *` pointer + a `PixelFormat`,
which doesn't map cleanly to safe Rust without choosing how to express
the pointer. This spec picks the minimum-viable byte-slice approach
and ships it as a new focused module.

## 1. Goals

1. Wrap the two cheatsheet functions with safe, ergonomic Rust signatures.
2. Validate input at the safe-API boundary: wrong-length slices and
   compressed formats (which cannot be addressed pixel-by-pixel) must
   return typed errors, not panic and not invoke UB.
3. Expose a `bytes_per_pixel(PixelFormat) -> Option<usize>` helper so
   callers can size their own buffers correctly.
4. Ship Tier-1 unit tests covering every uncompressed format and every
   error path.

Done-criteria are in §9.

## 2. Non-goals

- **No method-on-`Image` ergonomic layer.** `Image::get_color(x, y)`
  already covers the "I have an `Image`, give me the pixel at (x, y)"
  case via a different C function (`GetImageColor`). The new free
  functions are for raw byte buffers, not `Image` shortcuts.
- **No `PixelData<F>` typed newtype.** A const-generic-over-format API
  was considered and rejected as too much machinery for minimum-viable
  cheatsheet coverage. Users who want strong typing can build it on
  top of the free functions.
- **No BGRA support.** raylib's `PixelFormat` enum has no
  `B8G8R8A8` variant; the rlsw test-harness BGRA-byte-order quirk
  remains expressed as a bespoke `bytes.swap(0, 2)` loop in
  `raylib/src/test_harness.rs` and cannot be routed through these
  wrappers. The new module's doc surfaces this limitation explicitly.
- **No new dependencies.** `thiserror` is already in
  `raylib/Cargo.toml` `[dependencies]` for the error type.

## 3. Locked decisions (owner-confirmed during brainstorm 2026-05-29)

| # | Decision | Resolution |
|---|----------|------------|
| D1 | Use case scope | Minimum-viable — free fn pair over `&[u8]` / `&mut [u8]` with format validation. Not method-on-Image; not typed newtype; not declined. |
| D2 | Error handling | `Result<T, PixelColorError>` using `thiserror`. Two variants: `InsufficientBytes { format, expected, actual }` and `CompressedFormat(PixelFormat)`. `PartialEq + Eq` derived for test assertions. |
| D3 | BGRA + test-harness integration | Leave test_harness alone. raylib has no BGRA format, so wrapping the rlsw swap through these fns wouldn't eliminate it. Document the limitation in the new module. |
| D4 | `bytes_per_pixel` match arms | Exhaustive — no wildcard. A future raylib enum addition causes a compile error at the match site, surfacing the change rather than silently returning `None`. |
| D5 | Trailing-bytes contract | "Ignores trailing bytes beyond `bytes_per_pixel(format)`". Looser than exact-length matching; documented in the function-level rustdoc. |

## 4. File structure

A new focused module:

```
raylib/src/core/
├── pixel.rs              # NEW — this spec
├── texture.rs            # untouched
├── ...
└── mod.rs                # add `pub mod pixel; pub use pixel::*;`
raylib/src/
├── prelude.rs            # add the 4 public items
```

Rationale: `texture.rs` is already 1380+ lines and centers on `Image` /
`Texture2D`. The new module is for raw-byte pixel access (no `Image`
coupling), so it earns its own file. Total module size after this spec
ships: ~150 lines + ~100 lines of tests in the same file.

## 5. Public API

```rust
// raylib::core::pixel

use crate::consts::PixelFormat;
use crate::core::Color;
use crate::ffi;

/// Errors from [`get_pixel_color`] / [`set_pixel_color`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PixelColorError {
    /// Slice is shorter than the format requires.
    #[error("pixel format {format:?} needs {expected} bytes per pixel, got {actual}")]
    InsufficientBytes {
        format: PixelFormat,
        expected: usize,
        actual: usize,
    },
    /// Compressed formats are block-addressed and cannot be read/written one
    /// pixel at a time. raylib's `GetPixelColor` does not support them either.
    #[error("compressed pixel format {0:?} cannot be addressed pixel-by-pixel")]
    CompressedFormat(PixelFormat),
}

/// Bytes per single pixel for an uncompressed [`PixelFormat`]. Returns
/// `None` for compressed variants — they're addressed by block, not pixel.
pub fn bytes_per_pixel(format: PixelFormat) -> Option<usize>;

/// Read a single pixel from `bytes` interpreted as `format`.
///
/// `bytes` must contain at least `bytes_per_pixel(format)?` bytes; any
/// trailing bytes are ignored. Returns [`PixelColorError::CompressedFormat`]
/// for block-compressed formats, or [`PixelColorError::InsufficientBytes`]
/// when the slice is too short.
pub fn get_pixel_color(
    bytes: &[u8],
    format: PixelFormat,
) -> Result<Color, PixelColorError>;

/// Write `color` into `bytes` encoded as `format`.
///
/// Same length / format-validity rules as [`get_pixel_color`]. Trailing
/// bytes beyond `bytes_per_pixel(format)?` are not touched.
pub fn set_pixel_color(
    bytes: &mut [u8],
    color: Color,
    format: PixelFormat,
) -> Result<(), PixelColorError>;
```

All four items are re-exported from `crate::core::pixel::*` via
`raylib/src/core/mod.rs` and surfaced in `raylib::prelude` so the
canonical `use raylib::prelude::*;` import path works.

### Implementation notes

A private helper centralizes validation for both read and write so the
two public functions stay one-liners:

```rust
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

pub fn get_pixel_color(
    bytes: &[u8],
    format: PixelFormat,
) -> Result<Color, PixelColorError> {
    validate_slice(bytes.len(), format)?;
    // SAFETY: validate_slice ensured `bytes.len() >= bytes_per_pixel(format)`,
    // so raylib reads only within the slice. GetPixelColor's signature takes
    // `void *` but the C body only reads from srcPtr (verified in
    // raylib-sys/raylib/src/rtextures.c:5174-5240), so casting `*const u8`
    // → `*mut u8` is sound.
    Ok(unsafe { ffi::GetPixelColor(bytes.as_ptr() as *mut _, format as i32) })
}

pub fn set_pixel_color(
    bytes: &mut [u8],
    color: Color,
    format: PixelFormat,
) -> Result<(), PixelColorError> {
    validate_slice(bytes.len(), format)?;
    // SAFETY: same length guarantee as get_pixel_color; SetPixelColor writes
    // exactly `bytes_per_pixel(format)` bytes starting at dstPtr.
    unsafe { ffi::SetPixelColor(bytes.as_mut_ptr() as *mut _, color, format as i32) };
    Ok(())
}
```

## 6. `bytes_per_pixel` mapping

Exhaustive match — no wildcard arm. Verified against raylib 6.0
`raylib-sys/raylib/src/raylib.h:852-875`:

| Variant | Bytes / pixel |
|---|---|
| `PIXELFORMAT_UNCOMPRESSED_GRAYSCALE` | 1 |
| `PIXELFORMAT_UNCOMPRESSED_GRAY_ALPHA` | 2 |
| `PIXELFORMAT_UNCOMPRESSED_R5G6B5` | 2 |
| `PIXELFORMAT_UNCOMPRESSED_R5G5B5A1` | 2 |
| `PIXELFORMAT_UNCOMPRESSED_R4G4B4A4` | 2 |
| `PIXELFORMAT_UNCOMPRESSED_R8G8B8` | 3 |
| `PIXELFORMAT_UNCOMPRESSED_R8G8B8A8` | 4 |
| `PIXELFORMAT_UNCOMPRESSED_R32` | 4 |
| `PIXELFORMAT_UNCOMPRESSED_R32G32B32` | 12 |
| `PIXELFORMAT_UNCOMPRESSED_R32G32B32A32` | 16 |
| `PIXELFORMAT_UNCOMPRESSED_R16` | 2 |
| `PIXELFORMAT_UNCOMPRESSED_R16G16B16` | 6 |
| `PIXELFORMAT_UNCOMPRESSED_R16G16B16A16` | 8 |
| `PIXELFORMAT_COMPRESSED_*` (11 variants) | `None` |

Validation: test §7.1 cross-checks each uncompressed entry against
`ffi::GetPixelDataSize(1, 1, F as i32)` so the table stays honest.

## 7. Testing strategy

All Tier-1 (window-independent), in a `#[cfg(test)] mod tests` block
at the bottom of `raylib/src/core/pixel.rs`.

### 7.1 `bytes_per_pixel` consistent with raylib

For each uncompressed variant: assert
`bytes_per_pixel(F).unwrap() == GetPixelDataSize(1, 1, F as i32) as usize`.

### 7.2 `bytes_per_pixel` returns `None` for every compressed variant

Exhaustive list — no `for` loop over a slice; each `PIXELFORMAT_COMPRESSED_*`
named explicitly so an enum addition trips the test.

### 7.3 Round-trip per uncompressed format

Two passes per format:

- **All-FF**: `Color::new(0xFF, 0xFF, 0xFF, 0xFF)` round-trips
  exactly (max values quantize cleanly even through 5/6/4-bit channels).
- **Channel-distinct**: `Color::new(0xC0, 0x80, 0x40, 0xFF)` round-trips
  with per-format tolerance:
  - 8-bit channels (`GRAYSCALE`, `GRAY_ALPHA`, `R8G8B8`, `R8G8B8A8`): exact.
  - 32-bit float channels (`R32`, `R32G32B32`, `R32G32B32A32`): ±1 LSB
    tolerance. The encode-decode round-trip is `u8 → f32 / 255.0 → f32 * 255.0 → u8`,
    and the truncating final cast can shave 1 unit (e.g. 192 → 0.7529… → 191.9999…
    → 191). The implementation should discover the precise tolerance and
    pin it; ±1 is the documented worst case.
  - 5/6/4-bit channels (`R5G6B5`, `R5G5B5A1`, `R4G4B4A4`): ±8 LSBs
    tolerance per channel.
  - 16-bit half-float channels (`R16`, `R16G16B16`, `R16G16B16A16`):
    ±1 LSB tolerance per channel.
  - `GRAYSCALE` / `GRAY_ALPHA`: alpha exact, RGB collapsed to luminance
    (post-round-trip R/G/B equal each other; assert the equality
    invariant rather than a specific value).

### 7.4 Error variants fire correctly

- `get_pixel_color(&[], R8G8B8A8)` → `InsufficientBytes { format, expected: 4, actual: 0 }`.
- `get_pixel_color(&[0; 2], R8G8B8A8)` → `InsufficientBytes { format, expected: 4, actual: 2 }`.
- `get_pixel_color(&[0; 4], DXT1_RGB)` → `CompressedFormat(DXT1_RGB)`.
- Same three cases mirrored for `set_pixel_color`.

### 7.5 Trailing bytes ignored

`get_pixel_color(&[0; 64], R8G8B8A8)` equals `get_pixel_color(&[0; 4], R8G8B8A8)`.

### Test infrastructure

A shared `const UNCOMPRESSED_FORMATS: &[(PixelFormat, usize)]` at the
top of the test module, listing every uncompressed variant with its
expected byte count. Tests §7.1, §7.3, §7.5 all iterate this table so
adding a new variant exercises every relevant test.

**Coverage estimate**: ~35 assertions across 7 `#[test]` functions,
~100 lines of test code.

## 8. Documentation

Module-level rustdoc at the top of `raylib/src/core/pixel.rs`:

- One-paragraph intro explaining when to reach for these vs.
  `Image::get_color(x, y)`.
- A "see also" pointer to `Image::get_color` and `get_pixel_data_size`.
- An explicit note about the BGRA-not-supported limitation (rlsw test
  harness quirk) — see `raylib/src/test_harness.rs` `normalize_readback`.

Function-level rustdoc on each public item — sketched in §5 above.

## 9. Done-criteria

WS pixel-pointers is complete when **all** of:

- [ ] `raylib/src/core/pixel.rs` exists with the four public items
      from §5 and the private `validate_slice` helper.
- [ ] `raylib/src/core/mod.rs` declares `pub mod pixel;` and re-exports.
- [ ] `raylib/src/prelude.rs` includes the four public items.
- [ ] `bytes_per_pixel` is exhaustive — no wildcard match arm.
- [ ] All Tier-1 tests from §7 pass under `cargo test -p raylib --lib`.
- [ ] `cargo build --workspace --features full` clean.
- [ ] `cargo clippy --workspace --features full -- -D warnings` clean.
- [ ] Documentation (§8) lands — module-level + per-fn rustdoc.
- [ ] Module-level doc references the BGRA-not-supported limitation.
- [ ] `cheatsheet-parity-audit.md` §3 + §4 updated: move `GetPixelColor` /
      `SetPixelColor` from the small-gap table into the "covered ✅"
      column with the new module's path.
- [ ] `CHANGELOG.md` `## 6.0.0-rc.1 (unreleased)` `### Added` gains the
      four new public items.
- [ ] Commit(s) on `6.0-rc` with the `Co-Authored-By: Claude Opus 4.7`
      trailer; pushed to `fork/6.0-rc` and `fork/unstable`.
- [ ] CLAUDE.md status line: pre-WS9 queue head flips from
      `pixel-pointers` to `hashes`.

## 10. Risks + mitigations

1. **Bytes-per-pixel table drift if raylib changes a format.**
   Mitigated by §7.1 cross-check against `GetPixelDataSize` — if raylib
   ever changes the byte count of an uncompressed variant, the test
   fails immediately.
2. **`*const u8 → *mut u8` cast for the read path.** raylib's signature
   is `void *` even though `GetPixelColor` only reads. The SAFETY
   comment cites `rtextures.c:5174-5240` and notes that we verified
   read-only behavior. If a future raylib version starts writing
   through `srcPtr`, the cast becomes unsound — but that would be a
   surprising C API breakage worth catching with a Miri run during
   the post-release UBSAN-through-FFI workstream.
3. **Lossy-format round-trip tests use tolerances.** The tolerance
   values (±8 LSB for 5-bit channels, etc.) are derived from the
   quantization step — verified against the C decoder math in
   `rtextures.c:5180-5240`. If the test ever drifts, fix by adjusting
   the tolerance to match the actual quantization step, not by
   loosening it generously.

## 11. Out-of-scope follow-ups (logged for later)

- `Image::pixel_color(x, y)` / `Image::set_pixel_color(x, y, color)`
  methods that internally use `image.data()` + the new free fns —
  shippable once a real consumer wants them. Not part of this WS.
- Pixel iterators over `Image` — same: ship when a consumer wants them.
- `PixelData<F>` typed newtype — likely never; the dynamic-format API
  has been enough for every real raylib consumer in the wild.
- Upstream `B8G8R8A8` `PixelFormat` variant or a fix to rlsw's BGRA
  framebuffer ordering — outside the safe-bindings layer; would let
  the test_harness shed its bespoke `bytes.swap(0, 2)` loop.
