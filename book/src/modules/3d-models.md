# 3D models

3D rendering in raylib-rs uses `Camera3D` for view/projection and a
`RaylibMode3D` scope (entered via `begin_mode3D`) that activates 3D draw calls.
A `Model` bundles one or more `Mesh` arrays and `Material` arrays into a single
RAII resource. Skeletal animation is exposed through the new 6.0 `ModelAnimations`
wrapper.

## API surface

- [`RaylibHandle::load_model`](https://docs.rs/raylib/latest/raylib/core/models/trait.RaylibModel.html#method.load_model) —
  load a model from a file (`.obj`, `.glb`, `.gltf`, `.iqm`, …); returns
  `Result<Model, …>`.
- [`Mesh`](https://docs.rs/raylib/latest/raylib/core/models/struct.Mesh.html) —
  vertex data; owned by `Model` — do not drop it separately.
- [`Material`](https://docs.rs/raylib/latest/raylib/core/models/struct.Material.html) —
  material parameters (textures, colours, shader); owned by `Model`.
- [`ModelAnimations`](https://docs.rs/raylib/latest/raylib/core/models/struct.ModelAnimations.html) —
  RAII wrapper (new in 6.0) for the skeletal-animation set returned by
  `load_model_animations`.
- [`Camera3D`](https://docs.rs/raylib/latest/raylib/core/camera/struct.Camera3D.html) —
  position, target, up-vector, FOV, and projection mode.
- [`RaylibMode3DExt::begin_mode3D`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibMode3DExt.html#method.begin_mode3D) —
  enters 3D mode; returns a `RaylibMode3D` guard that calls `EndMode3D` on drop.
- [`RaylibDraw3D::draw_model`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw3D.html#method.draw_model) —
  draw a model at a position with a scale and tint.

## Example

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("3D model demo")
        .vsync()
        .build();

    let model = rl.load_model(&thread, "assets/robot.glb").unwrap();

    let camera = Camera3D {
        position: Vector3::new(5.0, 5.0, 5.0),
        target:   Vector3::new(0.0, 0.0, 0.0),
        up:       Vector3::new(0.0, 1.0, 0.0),
        fovy:     45.0,
        projection: CameraProjection::CAMERA_PERSPECTIVE,
    };

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        {
            let mut d3 = d.begin_mode3D(camera);
            d3.draw_model(&model, Vector3::zero(), 1.0, Color::WHITE);
        }
    }
    // `model` is dropped here; UnloadModel is called automatically.
}
```

## Gotchas

- **`ModelAnimations` is the new RAII wrapper (6.0).** Prior to 6.0, you had to call
  `UnloadModelAnimations` manually; the new `ModelAnimations` struct handles this on
  drop. Load animations via `RaylibHandle::load_model_animations`.
- **`Model` owns its `Mesh` and `Material` arrays.** Do not call `drop()` on a `Mesh`
  or `Material` obtained from a `Model` — the `Model`'s own `Drop` impl handles
  everything. Dropping them separately is a double-free.
- **Mesh accessor soundness (WS6-prep).** PRs #257/#118/#256 (folded in WS6) changed
  the safe accessors to return `&[T]` slices whose length is guaranteed by the C
  struct. Out-of-bounds access is no longer possible through the safe API.
- **Tracked-deferred.** Issue #283 (improper index calculation) and #207 (broader 3D
  API improvements) are deferred to future workstreams.

## See also

- [Textures and images](./textures-and-images.md) — textures applied to model
  materials.
- [raymath](./raymath.md) — `Vector3`, `Matrix`, `Quaternion` used throughout the 3D
  API.
- [`Model` docs.rs](https://docs.rs/raylib/latest/raylib/core/models/struct.Model.html)

### Showcase examples

Showcase examples that exercise this module:

- [models_loading](https://raylib-rs.github.io/raylib-rs/examples/models/models_loading.html)
- [models_loading_gltf](https://raylib-rs.github.io/raylib-rs/examples/models/models_loading_gltf.html)
- [models_mesh_generation](https://raylib-rs.github.io/raylib-rs/examples/models/models_mesh_generation.html)
- [models_geometric_shapes](https://raylib-rs.github.io/raylib-rs/examples/models/models_geometric_shapes.html)
- [models_billboard_rendering](https://raylib-rs.github.io/raylib-rs/examples/models/models_billboard_rendering.html)
- [models_box_collisions](https://raylib-rs.github.io/raylib-rs/examples/models/models_box_collisions.html)
