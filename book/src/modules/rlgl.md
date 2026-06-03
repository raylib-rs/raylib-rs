# rlgl

`rlgl` is raylib's thin OpenGL abstraction layer — the layer below raylib's
2D/3D rendering modules that maps draw calls to OpenGL 3.3, OpenGL ES 2.0, or
the software renderer depending on the active backend.

raylib-rs 6.0 exposes a safe wrapper for the **immediate-mode subset** of
`rlgl.h`: a matrix stack, immediate-mode vertex streams, render-state toggles,
and ergonomic methods that bind the crate's safe [`Texture2D`] and [`Shader`]
handles.  The full 161-function rlgl surface remains available as `ffi`
(power-user escape hatch) — the safe layer covers the everyday drawing needs.

All entry points hang off the [`RaylibRlgl`] trait, which is blanket-implemented
for every draw handle, so they're only callable inside a `begin_drawing` frame.

## API surface

- [`RaylibRlgl::rl_push_matrix`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_push_matrix) —
  push the current matrix; returns an [`RlMatrix`] guard that auto-pops on drop.
- [`RaylibRlgl::rl_translatef`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_translatef) /
  [`rl_rotatef`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_rotatef) /
  [`rl_scalef`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_scalef) —
  multiply the current matrix by a translation / rotation / scale.
- [`RaylibRlgl::rl_ortho`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_ortho) —
  multiply the current matrix by an orthographic projection.
- [`RaylibRlgl::rl_begin`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_begin) —
  begin a vertex stream; returns an [`RlImmediate`] guard that ends it on drop.
- [`RaylibRlgl::rl_draw`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_draw) —
  closure form: begin a stream, call `body`, end automatically.
- [`RaylibRlgl::rl_set_texture`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_set_texture) /
  [`rl_enable_texture`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_enable_texture) /
  [`rl_disable_texture`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_disable_texture) —
  bind/unbind a [`Texture2D`] for subsequent immediate-mode draws.
- [`RaylibRlgl::rl_enable_shader`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_enable_shader) /
  [`rl_set_shader`](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html#method.rl_set_shader) —
  bind a [`Shader`] by its safe handle.
- [`DrawMode`](https://docs.rs/raylib/latest/raylib/rlgl/enum.DrawMode.html) —
  `Lines`, `Triangles`, `Quads` (maps to `RL_LINES` / `RL_TRIANGLES` / `RL_QUADS`).
- [`RlMatrix`](https://docs.rs/raylib/latest/raylib/rlgl/struct.RlMatrix.html) /
  [`RlImmediate`](https://docs.rs/raylib/latest/raylib/rlgl/struct.RlImmediate.html) —
  RAII guards for the matrix stack and vertex stream.

## Example

The example requires a live GPU context and so cannot run in the book's CI
build (the `software_renderer` feature is mutually exclusive with the default
OpenGL build).  The [WS9 showcase](https://github.com/raylib-rs/raylib-rs) will
provide a runnable demo.

```rust,ignore
# extern crate raylib;
use raylib::prelude::*;
use raylib::rlgl::{DrawMode, RaylibRlgl};

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("rlgl demo")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Push a matrix, translate, draw an immediate-mode triangle, pop.
        {
            let mut m = d.rl_push_matrix();
            m.rl_translatef(320.0, 240.0, 0.0);

            m.rl_draw(DrawMode::Triangles, |v| {
                v.color4ub(Color::RED);
                v.vertex2f(-50.0, -50.0);
                v.color4ub(Color::GREEN);
                v.vertex2f( 50.0, -50.0);
                v.color4ub(Color::BLUE);
                v.vertex2f(  0.0,  50.0);
            });
        } // RlMatrix drops here → rlPopMatrix called automatically
    }
}
```

## Gotchas

- **`RlMatrix` auto-pops on drop.** Do not call `rl_pop_matrix` manually after
  using `rl_push_matrix` — the RAII guard handles it.  The guard derefs to the
  parent draw handle so you can call other draw methods through it.
- **Immediate-mode vertices need texcoords under the software renderer.**
  The rlsw backend samples the shapes texture; without `texcoord2f` calls the
  default `(0,0)` coordinate hits a transparent texel and emits zero pixels.
  Always emit `texcoord2f` alongside `vertex2f`/`vertex3f` when testing with the
  software renderer.
- **GL-object lifecycle stays with RAII.**  `rl_set_texture` / `rl_enable_shader`
  bind the existing RAII handles; they do **not** create or destroy GPU objects.
  Lifecycle management remains with `Texture2D::Drop` / `Shader::Drop` and the
  raw `ffi` create/destroy functions.

## See also

- [Window and drawing](./window-and-drawing.md) — the draw handle that hosts rlgl calls.
- [Software renderer](./software-renderer.md) — the headless backend used in render tests.
- [`RaylibRlgl` docs.rs](https://docs.rs/raylib/latest/raylib/rlgl/trait.RaylibRlgl.html)

### Showcase examples

Showcase examples that exercise this module:

- [rlgl_standalone](https://raylib-rs.github.io/raylib-rs/examples/others/rlgl_standalone.html) — desktop only
- [shapes_rlgl_color_wheel](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_rlgl_color_wheel.html)
- [shapes_rlgl_triangle](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_rlgl_triangle.html)
- [models_rlgl_solar_system](https://raylib-rs.github.io/raylib-rs/examples/models/models_rlgl_solar_system.html)
