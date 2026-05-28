# RAII and resources

Every raylib resource in raylib-rs is a Rust struct that cleans up on drop. The `Unload*` family of
C functions is never exposed in the public API — `Drop` implementations call them automatically
when the resource leaves scope. This eliminates the class of bug where a resource is forgotten or
freed twice.

The pattern applies uniformly: `Image`, `Texture2D`, `RenderTexture2D`, `Font`, `Mesh`, `Shader`,
`Material`, and `Model` all implement `Drop`. You allocate with `load_*` or `gen_image_*`; you free
by letting the value go out of scope (or by calling `drop()` explicitly when order matters).

## API surface

- [`Image`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Image.html) — CPU-side pixel
  buffer; dropped by calling `UnloadImage`.
- [`Texture2D`](https://docs.rs/raylib/latest/raylib/core/texture/struct.Texture2D.html) — GPU
  texture handle; dropped by calling `UnloadTexture`.
- [`RenderTexture2D`](https://docs.rs/raylib/latest/raylib/core/texture/struct.RenderTexture2D.html) —
  off-screen render target; dropped by `UnloadRenderTexture`.
- [`Font`](https://docs.rs/raylib/latest/raylib/core/text/struct.Font.html) — loaded font;
  dropped by `UnloadFont`.
- [`Mesh`](https://docs.rs/raylib/latest/raylib/core/models/struct.Mesh.html) — vertex data;
  owned by `Model` (do not drop separately).
- [`Shader`](https://docs.rs/raylib/latest/raylib/core/shaders/struct.Shader.html) — GPU shader
  program; dropped by `UnloadShader`.
- [`Material`](https://docs.rs/raylib/latest/raylib/core/models/struct.Material.html) — material
  parameters; owned by `Model`.
- [`Model`](https://docs.rs/raylib/latest/raylib/core/models/struct.Model.html) — 3D model
  including meshes and materials; dropped by `UnloadModel`.
- [`ModelAnimations`](https://docs.rs/raylib/latest/raylib/core/models/struct.ModelAnimations.html) —
  new in 6.0, RAII wrapper for the skeletal-animation animation set.

## Example

The example below shows resource loading and explicit teardown order. `image` is dropped first;
`font` is kept alive until the end of the outer scope. Because Drop is deterministic in Rust, the
sequence is predictable and correct.

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("RAII demo")
        .build();

    // gen_image_color returns an Image that owns its pixel buffer.
    let image = Image::gen_image_color(64, 64, Color::RED);

    // load_texture_from_image uploads to the GPU.
    let texture = rl.load_texture_from_image(&thread, &image).unwrap();

    // Explicit drop: image is freed here (UnloadImage called).
    // texture remains alive.
    drop(image);

    // texture is freed here when it leaves scope (UnloadTexture called).
    drop(texture);

    // RaylibHandle (rl) is freed here, tearing down the window.
}
```

## Gotchas

- **Lifetime-bound audio resources** — `Wave`, `Sound`, `Music`, and `AudioStream` are bound by
  lifetime to an `AudioHandle`. They cannot outlive the audio device. See the
  [audio chapter](../modules/audio.md).
- **raylib-allocated buffers** — functions that return raylib-managed byte slices (e.g., exported
  image data) wrap them as `ManuallyDrop<Box<[T]>>` and free via the matching `Unload*` or
  `MemFree` call — never `libc::free`. This stays correct under custom raylib allocators. See
  `DECISIONS.md` for the rationale.
- **`Model` owns its `Mesh` and `Material` arrays** — do not call `drop()` on a `Mesh` or
  `Material` that came from a `Model`. Dropping the `Model` handles everything.

## See also

- [`core-concepts/handle-and-thread.md`](./handle-and-thread.md) — why the handle matters for
  resource loading.
- [`modules/textures-and-images.md`](../modules/textures-and-images.md) — `Image` vs. `Texture2D`
  in depth.
- [`modules/audio.md`](../modules/audio.md) — lifetime-bound audio resources.
