# Collision

The collision module provides pure-data helpers for detecting overlap between
2D and 3D shapes.  All functions are free functions (or methods on geometry
types) — no window, no GPU, no handle required.  They map directly to raylib's
`CheckCollision*` and `GetRayCollision*` family.

## API surface

- [`Rectangle::check_collision_recs`](https://docs.rs/raylib/latest/raylib/ffi/struct.Rectangle.html#method.check_collision_recs) —
  check overlap between two `Rectangle`s.
- [`Rectangle::check_collision_point_rec`](https://docs.rs/raylib/latest/raylib/ffi/struct.Rectangle.html#method.check_collision_point_rec) —
  check if a point is inside a rectangle.
- [`Rectangle::check_collision_circle_rec`](https://docs.rs/raylib/latest/raylib/ffi/struct.Rectangle.html#method.check_collision_circle_rec) —
  check if a circle overlaps a rectangle.
- [`Rectangle::get_collision_rec`](https://docs.rs/raylib/latest/raylib/ffi/struct.Rectangle.html#method.get_collision_rec) —
  compute the intersection rectangle of two colliding rectangles (`None` if
  they don't overlap).
- [`check_collision_circles`](https://docs.rs/raylib/latest/raylib/core/collision/fn.check_collision_circles.html) —
  check overlap between two circles.
- [`check_collision_point_circle`](https://docs.rs/raylib/latest/raylib/core/collision/fn.check_collision_point_circle.html) —
  check if a point is inside a circle.
- [`check_collision_point_triangle`](https://docs.rs/raylib/latest/raylib/core/collision/fn.check_collision_point_triangle.html) —
  check if a point is inside a triangle.
- [`check_collision_point_line`](https://docs.rs/raylib/latest/raylib/core/collision/fn.check_collision_point_line.html) —
  check if a point is within a threshold distance of a line segment.
- [`check_collision_lines`](https://docs.rs/raylib/latest/raylib/core/collision/fn.check_collision_lines.html) —
  check if two line segments intersect; returns `Option<Vector2>` with the
  intersection point.
- [`check_collision_spheres`](https://docs.rs/raylib/latest/raylib/core/collision/fn.check_collision_spheres.html) —
  3D sphere–sphere overlap check.
- [`BoundingBox::check_collision_boxes`](https://docs.rs/raylib/latest/raylib/math/struct.BoundingBox.html#method.check_collision_boxes) —
  3D axis-aligned bounding box overlap check.

## Example

Rectangle and circle collision — pure Rust, no window needed.

```rust
# extern crate raylib;
use raylib::math::Rectangle;
use raylib::core::collision::{check_collision_circles, check_collision_lines};
use raylib::math::Vector2;

// Two overlapping rectangles
let r1 = Rectangle::new(0.0, 0.0, 10.0, 10.0);
let r2 = Rectangle::new(5.0, 5.0, 10.0, 10.0);
assert!(r1.check_collision_recs(r2), "overlapping rects should collide");

// A separated rectangle pair
let r3 = Rectangle::new(20.0, 20.0, 5.0, 5.0);
assert!(!r1.check_collision_recs(r3), "non-overlapping rects should not collide");

// Point inside rectangle
assert!(r1.check_collision_point_rec(Vector2::new(3.0, 3.0)));
assert!(!r1.check_collision_point_rec(Vector2::new(15.0, 15.0)));

// Two overlapping circles (centers 1 apart, radii sum = 4)
assert!(check_collision_circles(
    Vector2::new(0.0, 0.0), 2.0,
    Vector2::new(1.0, 0.0), 2.0,
));

// Line intersection
let hit = check_collision_lines(
    Vector2::new(-1.0, -1.0), Vector2::new(1.0, 1.0),
    Vector2::new(-1.0,  1.0), Vector2::new(1.0, -1.0),
);
assert!(hit.is_some(), "diagonal lines should intersect");
let pt = hit.unwrap();
assert!((pt.x).abs() < 1e-4 && (pt.y).abs() < 1e-4);
```

## Gotchas

- **`check_collision_recs` is a method on `Rectangle`**, not a free function.
  Call it as `rect.check_collision_recs(other)`.  Similarly,
  `check_collision_point_rec`, `check_collision_circle_rec`, and
  `get_collision_rec` are methods on `Rectangle`.
- **`check_collision_lines` returns `Option<Vector2>`**, not `bool`.  Use
  `.is_some()` to test for collision and `.unwrap()` to read the intersection
  point.
- **Touch counts as collision.**  Circles whose edges exactly touch (distance
  equals the sum of radii) are considered colliding; this matches raylib's C
  behaviour.

## See also

- [raymath](./raymath.md) — `Vector2`/`Vector3`/`BoundingBox` types used here.
- [Shapes](./shapes.md) — drawing the same geometry you're testing.
- [`check_collision_recs` docs.rs](https://docs.rs/raylib/latest/raylib/ffi/struct.Rectangle.html#method.check_collision_recs)

### Showcase examples

Showcase examples that exercise this module:

- [shapes_collision_area](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_collision_area.html)
- [shapes_ellipse_collision](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_ellipse_collision.html)
- [shapes_ball_physics](https://dacode45.github.io/raylib-rs/examples/shapes/shapes_ball_physics.html)
