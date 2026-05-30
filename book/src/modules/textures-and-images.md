# Textures and images

raylib distinguishes two related types: **`Image`** is CPU-side pixel data (a
heap-allocated buffer you can read and modify without a GPU or a window), and
**`Texture2D`** is a GPU-side handle created by uploading an `Image`. The typical
flow is: generate or load an `Image`, manipulate it, then call
`load_texture_from_image` on `RaylibHandle` to push it to the GPU.

Both types are RAII resources — they clean up automatically on drop (see
[RAII and resources](../core-concepts/raii-and-resources.md)).

## API surface

- [`Image::load_image`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html#method.load_image) —
  load an image file from disk; returns `Result<Image, …>`.
- [`Image::gen_image_color`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html#method.gen_image_color) —
  generate a solid-color image in memory; no window needed.
- [`Image::gen_image_perlin_noise`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html#method.gen_image_perlin_noise) —
  procedural Perlin-noise image.
- [`Image::draw`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html#method.draw) —
  CPU-side blit of one image onto another.
- [`Image::get_color`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html#method.get_color) —
  read a single pixel's `Color` by `(x, y)`.
- [`RaylibHandle::load_texture_from_image`](https://docs.rs/raylib/latest/raylib/core/texture/trait.RaylibTexture.html#method.load_texture_from_image) —
  upload an `Image` to the GPU as a `Texture2D`; requires a window + thread token.
- [`RenderTexture2D`](https://docs.rs/raylib/latest/raylib/core/texture/struct.RenderTexture2D.html) —
  off-screen render target; used for render-to-texture workflows.
- [`RaylibDraw::draw_texture`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_texture) —
  draw a `Texture2D` at a position with a tint color.

## Example

CPU-side image operations (generation and pixel read-back) work without a window or
GPU. The example below generates a red image and asserts a pixel value — no display
required.

```rust
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    // Generate a 64×64 solid-red Image entirely in CPU memory.
    let img = Image::gen_image_color(64, 64, Color::RED);

    // Read back a pixel and verify its color.
    let pixel = img.get_color(32, 32);
    assert_eq!(pixel.r, 255);
    assert_eq!(pixel.g, 0);
    assert_eq!(pixel.b, 0);
    // `img` is dropped here; UnloadImage is called automatically.
}
```

Uploading to the GPU requires a window:

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Texture demo")
        .build();

    let img = Image::gen_image_color(64, 64, Color::BLUE);
    // load_texture_from_image uploads to GPU; img can be dropped afterwards.
    let tex = rl.load_texture_from_image(&thread, &img).unwrap();
    drop(img); // CPU buffer freed; GPU copy lives on in `tex`.

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_texture(&tex, 100, 100, Color::WHITE);
    }
}
```

## Gotchas

- **`Image` is CPU; `Texture2D` is GPU.** You cannot modify a `Texture2D` directly.
  If you need per-frame CPU edits, keep the `Image`, modify it, and re-upload with
  `update_texture`.
- **`RenderTexture2D` Y-flip on readback.** The WS4 headless harness reads the
  software-renderer framebuffer as BGRA with Y-inverted rows and then normalises it
  for you. If you use `load_image_from_screen` directly, be aware of the GL-style
  bottom-left origin. See `notes/ws4b-complete.md`.
- **Memory-leak fix in 6.0.** `export_image_to_memory` had a leak in 5.x
  (backlog #247). It was fixed by PR #250 in WS3 — the buffer is now wrapped as
  `ManuallyDrop<Box<[u8]>>` and freed correctly.

## See also

- [Software renderer](./software-renderer.md) — headless pixel-probe testing.
- [3D models](./3d-models.md) — textures applied to models.
- [`Image` docs.rs](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html) /
  [`Texture2D` docs.rs](https://docs.rs/raylib/latest/raylib/core/texture/struct.Texture2D.html)
