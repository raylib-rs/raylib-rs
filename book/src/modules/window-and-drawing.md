# Window and drawing

The window is opened by calling `raylib::init().build()`. The builder returns a
`(RaylibHandle, RaylibThread)` pair that controls the entire lifecycle: the handle
exposes every per-frame operation, and the thread token is the proof that you are on
the correct OS thread. Drawing happens between `begin_drawing` and the end of the
returned guard's lifetime — all draw primitives live on the `RaylibDraw` trait
implemented by `RaylibDrawHandle`.

## API surface

- [`RaylibBuilder`](https://docs.rs/raylib/latest/raylib/core/struct.RaylibBuilder.html) —
  fluent builder returned by `raylib::init()`; call `.build()` to open the window.
  Key builder methods: `.size(w, h)`, `.title(s)`, `.vsync()`, `.msaa_4x()`,
  `.undecorated()`.
- [`RaylibHandle`](https://docs.rs/raylib/latest/raylib/core/struct.RaylibHandle.html) —
  the main handle; owns the window and is the receiver for most API calls.
- [`RaylibHandle::set_target_fps`](https://docs.rs/raylib/latest/raylib/core/struct.RaylibHandle.html#method.set_target_fps) —
  cap the frame rate. `set_target_fps(0)` disables the limiter (busy-spin).
- [`RaylibHandle::begin_drawing`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDrawHandle.html) —
  returns a scoped `RaylibDrawHandle`; `EndDrawing` fires automatically on drop.
- [`RaylibDraw`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html) —
  trait with all 2D drawing primitives: `clear_background`, `draw_text`,
  `draw_rectangle`, `draw_line`, `draw_circle`, `draw_texture`, and more.
- [`RaylibHandle::window_should_close`](https://docs.rs/raylib/latest/raylib/core/window/trait.RaylibWindow.html) —
  returns `true` when the OS close event has been sent.

## Example

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, raylib-rs!")
        .vsync()
        .build();

    rl.set_target_fps(60);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::DARKGRAY);
    }
    // RaylibHandle is dropped here; CloseWindow() is called automatically.
}
```

## Gotchas

- **Draw only inside the guard.** All methods on `RaylibDraw` are only accessible on
  the `RaylibDrawHandle` scope. You cannot call `draw_text` directly on
  `RaylibHandle` — you must enter `begin_drawing` first.
- **FPS pacing.** `set_target_fps(0)` removes the frame limiter entirely; raylib will
  busy-spin. Omitting the call defaults to the raylib default (60 on most platforms).
- **Window state queries.** `is_window_minimized`, `is_window_resized`, and related
  helpers live on the
  [`RaylibWindow`](https://docs.rs/raylib/latest/raylib/core/window/trait.RaylibWindow.html)
  trait — check the trait's documentation for the current set of methods.
- **`RaylibThread` is `!Send`.** Never put it in a `Mutex`, `Arc`, or hand it to
  another thread. See the [Handle and thread](../core-concepts/handle-and-thread.md)
  chapter for why.

## See also

- [Handle and thread](../core-concepts/handle-and-thread.md) — lifecycle details and
  the `!Send` invariant.
- [Input](./input.md) — reading keyboard, mouse, and gamepad inside the loop.
- [Shapes](./shapes.md) — the full 2D primitive catalogue.
