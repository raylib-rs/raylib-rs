# Error handling

raylib-rs uses structured error types for all fallible operations.  The safe
crate defines a rich set of typed errors in
[`raylib::core::error`](https://docs.rs/raylib/latest/raylib/core/error/index.html) —
each domain (images, textures, fonts, audio, models, etc.) has its own error
enum.  A top-level
[`RaylibError`](https://docs.rs/raylib/latest/raylib/core/error/enum.RaylibError.html)
aggregates all of them via `#[from]` conversions for callers that don't need to
distinguish the source.

**Drawing operations are infallible** — there is no error path for
`draw_rectangle`, `draw_text`, etc.  Errors arise only at resource load time.

## API surface

- [`InvalidImageError`](https://docs.rs/raylib/latest/raylib/core/error/enum.InvalidImageError.html) —
  returned by `Image::load_image`, `Image::load_image_from_mem`, etc.
  Variants include `NullDataFromFile` (file not found / unsupported format),
  `ZeroWidth`/`ZeroHeight`, `UnsupportedFormat`.
- [`LoadTextureError`](https://docs.rs/raylib/latest/raylib/core/error/enum.LoadTextureError.html) —
  returned by `load_texture`, `load_texture_from_image`, `load_render_texture`.
- [`LoadFontError`](https://docs.rs/raylib/latest/raylib/core/error/enum.LoadFontError.html) —
  returned by `load_font`, `load_font_ex`.
- [`LoadSoundError`](https://docs.rs/raylib/latest/raylib/core/error/enum.LoadSoundError.html) —
  returned by `RaylibAudio::new_sound`, `new_wave`, `new_music`.
- [`LoadModelError`](https://docs.rs/raylib/latest/raylib/core/error/enum.LoadModelError.html) —
  returned by `load_model`, `load_model_from_mesh`.
- [`RaylibError`](https://docs.rs/raylib/latest/raylib/core/error/enum.RaylibError.html) —
  top-level enum aggregating all domain errors via `From` impls; useful when
  propagating with `?` through a function that loads multiple resource types.
- **`raylib::init().build()` panics** on a second call (window context is
  process-global).  All other APIs are `Result`-based.

## Example

```rust,no_run
# extern crate raylib;
use raylib::core::texture::Image;
use raylib::core::error::InvalidImageError;

fn main() {
    match Image::load_image("nonexistent.png") {
        Ok(img) => {
            println!("loaded {}x{} image", img.width(), img.height());
        }
        Err(InvalidImageError::NullDataFromFile) => {
            eprintln!("file not found or unsupported format");
        }
        Err(e) => {
            eprintln!("unexpected image error: {e}");
        }
    }
}
```

## Gotchas

- **Error variants are typed, not stringly-typed.**
  Unlike some thin C-binding layers, raylib-rs maps sentinel return values
  to named enum variants.  This makes `match` exhaustive and lets you
  distinguish "file not found" from "GPU upload failed" without parsing a
  message string.
- **Errors use `thiserror`.** Every error type implements `std::error::Error`
  and `Display`.  They compose naturally with `anyhow`, `eyre`, or any
  `Box<dyn Error>` handler.
- **`?` propagation works** if you use `RaylibError` as your function's error
  type — every domain error has a `From` impl into it.  Alternatively, the
  individual domain errors can be mapped with `.map_err(RaylibError::from)`.
- **`raylib::init().build()` panics, not `Err`.**
  Double-init is a programming error (not a runtime condition), so it panics
  rather than returning `Err`.

## See also

- [Safety](../core-concepts/safety.md) — how the safe wrapper prevents UB.
- [Textures and images](./textures-and-images.md) — `Image` and `Texture2D` load paths.
- [`RaylibError` docs.rs](https://docs.rs/raylib/latest/raylib/core/error/enum.RaylibError.html)
