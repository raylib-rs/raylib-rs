# Quickstart

This chapter gets you from zero to a working window in under 30 lines of Rust. If you have not installed the native build dependencies yet, see the install guide for your platform first.

## Add the dependency

In your `Cargo.toml`:

```toml
[dependencies]
raylib = "5.7"
```

> **Note:** The crate version in this book reflects the latest published release. The `6.0-rc` branch tracks the in-progress 6.0 upgrade; the crate version bumps to `6.0` when WS8 publishes to crates.io.

## Open a window

```rust,no_run
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, raylib-rs")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        d.draw_text("Hello, raylib-rs!", 20, 20, 20, Color::BLACK);
    }
}
```

Walking through the code:

- `raylib::init()` returns a `RaylibBuilder`. Chain `.size()` and `.title()` to configure the window before it opens.
- `.build()` opens the window and returns a pair: `RaylibHandle` (your main API handle) and `RaylibThread` (a token proving you are on the main thread).
- `rl.begin_drawing(&thread)` opens a drawing frame and returns a `RaylibDrawHandle`. All draw calls live inside this block; the frame is committed when `d` is dropped at the end of the loop body.
- `Color::WHITE` and `Color::BLACK` are constants in `raylib::prelude`. No import gymnastics needed.

## What's next

- **Install guides** — pick your platform: [Windows](./install-windows.md), [macOS](./install-macos.md), [Linux](./install-linux.md), or [Web/Wasm](./install-web.md).
- **Core Concepts** — the Handle/Thread ownership model, RAII resources, and how strings work.
- **Modules** — dive into [Window and drawing](), [Input](), [Shapes](), and beyond.
