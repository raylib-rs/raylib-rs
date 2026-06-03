# Shapes

2D drawing primitives — rectangles, lines, circles, triangles, polygons, and
individual pixels — are all methods on the
[`RaylibDraw`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html)
trait. The trait is implemented by `RaylibDrawHandle`, so all calls live inside the
`begin_drawing` scope. No extra state or resource loading is required; shapes are
drawn immediately with a color and optional parameters for thickness, rounding, or
gradients.

## API surface

- [`draw_rectangle`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_rectangle) —
  filled rectangle by position + size.
- [`draw_rectangle_v`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_rectangle_v) —
  filled rectangle by `Vector2` position + size.
- [`draw_rectangle_rec`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_rectangle_rec) —
  filled rectangle from a `Rectangle` value.
- [`draw_circle`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_circle) —
  filled circle by center + radius.
- [`draw_circle_v`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_circle_v) —
  filled circle by `Vector2` center + radius.
- [`draw_line`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_line) —
  1-pixel line between two integer points.
- [`draw_line_ex`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_line_ex) —
  thick line between two `Vector2` points with a `thickness` parameter.
- [`draw_triangle`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_triangle) —
  filled triangle from three `Vector2` vertices (counter-clockwise winding).
- [`draw_poly`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_poly) —
  filled regular polygon (n sides).
- [`draw_pixel`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_pixel) —
  single pixel by integer coordinates.

## Example

The example opens a window and draws a rectangle and a circle every frame. A
software-renderer version of this is available as part of the WS9 showcase; the
live example here requires a display.

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Shapes demo")
        .vsync()
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Filled rectangle at (50, 50), 200×100, blue.
        d.draw_rectangle(50, 50, 200, 100, Color::BLUE);

        // Filled circle at (400, 200), radius 80, red.
        d.draw_circle(400, 200, 80.0, Color::RED);

        // 1-pixel line from (0, 0) to (640, 480), dark gray.
        d.draw_line(0, 0, 640, 480, Color::DARKGRAY);
    }
}
```

## Gotchas

Shapes are stable across raylib 5.x and 6.0; there are no breaking changes to the
2D primitive set. The only points worth noting:

- **Counter-clockwise winding for `draw_triangle`.** raylib expects vertices in
  counter-clockwise order; clockwise-wound triangles are culled.
- **Integer vs. vector variants.** Most primitives have both an integer overload
  (`draw_circle(cx: i32, cy: i32, …)`) and a `_v` overload
  (`draw_circle_v(center: Vector2, …)`). Prefer `_v` when your coordinates are
  already `Vector2`.

## See also

- [Window and drawing](./window-and-drawing.md) — the `begin_drawing` scope that all
  shape calls live inside.
- [Textures and images](./textures-and-images.md) — blitting pixel buffers and
  textures in the same draw scope.
- [Software renderer](./software-renderer.md) — running headless shape tests.

### Showcase examples

Showcase examples that exercise this module:

- [shapes_basic_shapes](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_basic_shapes.html)
- [shapes_lines_bezier](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_lines_bezier.html)
- [shapes_rectangle_advanced](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_rectangle_advanced.html)
- [shapes_splines_drawing](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_splines_drawing.html)
- [shapes_colors_palette](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_colors_palette.html)
- [shapes_logo_raylib](https://raylib-rs.github.io/raylib-rs/examples/shapes/shapes_logo_raylib.html)
