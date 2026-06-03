# Handle and thread

`RaylibHandle` is the entry point for almost every raylib operation. Together with `RaylibThread`, it
is returned by `raylib::init().build()`. The two-value return enforces two invariants at the type
level: there can only be one active window at a time (single-init), and drawing can only happen on
the thread that owns the handle.

`RaylibHandle` owns the window and the OpenGL context. `RaylibThread` is a zero-sized marker token
that proves you are on the main thread. You pass `&thread` into `begin_drawing` and other
thread-sensitive calls; the borrow checker ensures they are never called from a background thread.

## API surface

- [`RaylibHandle`](https://docs.rs/raylib/latest/raylib/struct.RaylibHandle.html) — the main API
  handle; returned alongside `RaylibThread` from `.build()`.
- [`RaylibThread`](https://docs.rs/raylib/latest/raylib/struct.RaylibThread.html) — main-thread
  proof token; `!Send`.
- [`RaylibBuilder`](https://docs.rs/raylib/latest/raylib/struct.RaylibBuilder.html) — builder for
  the window; obtained from `raylib::init()`.
- [`raylib::init()`](https://docs.rs/raylib/latest/raylib/fn.init.html) — the only entry point;
  returns a `RaylibBuilder`.
- [`RaylibHandle::window_should_close`](https://docs.rs/raylib/latest/raylib/struct.RaylibHandle.html#method.window_should_close) —
  returns `true` when the user requests quit (window X, `Escape`, etc.).
- [`RaylibHandle::begin_drawing`](https://docs.rs/raylib/latest/raylib/struct.RaylibHandle.html#method.begin_drawing) —
  opens a drawing frame; returns a `RaylibDrawHandle` guard that commits the frame on drop.

## Example

The following shows the canonical init-loop-draw pattern, highlighting where the handle and thread
token appear:

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    // init() returns a RaylibBuilder; .build() opens the window.
    // The two return values are the handle (rl) and the thread token (thread).
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Handle and thread demo")
        .build();

    while !rl.window_should_close() {
        // begin_drawing takes &thread — the borrow checker ensures this is
        // the main thread.  The returned guard commits the frame on drop.
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text("RaylibHandle in action", 20, 20, 20, Color::DARKGRAY);
    }
    // rl is dropped here, tearing down the window and GL context.
}
```

## Gotchas

- **`RaylibThread` is `!Send`** — never put it in a `Mutex`, `Arc`, or `thread::spawn` payload. The
  compiler will reject the attempt; this is intentional.
- **Single-init** — calling `raylib::init().build()` while an existing `RaylibHandle` is live will
  panic. raylib's C layer maintains global window state; there is no way to have two windows in one
  process.
- **Teardown order** — window teardown happens when `RaylibHandle` is dropped. If you `mem::forget`
  the handle, raylib's GL context leaks. Let Rust's normal drop order handle cleanup; do not try to
  tear down the window manually.

## See also

- [`core-concepts/raii-and-resources.md`](./raii-and-resources.md) — how all raylib resources
  follow RAII semantics.
- [`modules/window-and-drawing.md`](../modules/window-and-drawing.md) — the full window
  configuration and drawing API.
- [docs.rs `RaylibHandle`](https://docs.rs/raylib/latest/raylib/struct.RaylibHandle.html)
