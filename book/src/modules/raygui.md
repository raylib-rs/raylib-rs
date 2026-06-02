# raygui

[raygui](https://github.com/raysan5/raygui) is an immediate-mode GUI toolkit
bundled with raylib.  raylib-rs 6.0 exposes all 57 `RAYGUIAPI` functions at
parity with raygui 6.0.  Controls are grouped into sub-traits
(`RaylibGuiControls`, `RaylibGuiContainers`, `RaylibGuiState`,
`RaylibGuiAdvanced`, `RaylibGuiIcons`) and blanket-implemented for every draw
handle — just `use raylib::prelude::*` and the methods are available inside
a `begin_drawing` frame.

String parameters accept `impl AsRef<str>` (pass `&str`, `String`, or any
type that derefs to a string slice) — conversion goes through a thread-local
scratch buffer so there are no per-frame heap allocations.

> **Feature gate.** raygui is behind the `raygui` Cargo feature.  Add
> `features = ["raygui"]` (or `"full"`) to your `Cargo.toml` dependency.

## API surface

- [`RaylibGuiControls::gui_button`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiControls.html#method.gui_button) —
  button control; returns `true` when clicked.
- [`RaylibGuiControls::gui_label`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiControls.html#method.gui_label) —
  label control; displays text.
- [`RaylibGuiControls::gui_slider`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiControls.html#method.gui_slider) —
  horizontal slider; takes `&mut f32` for the current value.
- [`RaylibGuiControls::gui_combo_box`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiControls.html#method.gui_combo_box) —
  combo box; semicolon-separated option string, `&mut i32` active index.
- [`RaylibGuiContainers::gui_window_box`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiContainers.html#method.gui_window_box) —
  window box container; returns `true` when the close button is clicked.
- [`RaylibGuiState::gui_set_state`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiState.html#method.gui_set_state) —
  set global GUI state (`Normal`, `Focused`, `Pressed`, `Disabled`).
- [`RaylibGuiState::gui_load_style`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiState.html#method.gui_load_style) —
  load a `.rgs` style file to retheme all controls.
- [`RaylibDrawGui`](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibDrawGui.html) —
  umbrella supertrait re-exported for source compatibility.

## Example

raygui controls require an active drawing frame and a GPU context.
The example below is marked `ignore` because the `software_renderer` feature
is mutually exclusive with the default OpenGL build used by the book's CI.
The [WS9 showcase](https://github.com/raylib-rs/raylib-rs) will provide a
runnable raygui demo once the web gallery is deployed.

```rust,ignore
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(400, 300)
        .title("raygui demo")
        .build();

    let mut slider_val: f32 = 0.5;

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Label
        d.gui_label(
            Rectangle::new(20.0, 20.0, 200.0, 30.0),
            "Hello, raygui!",
        );

        // Button — returns true on click
        if d.gui_button(Rectangle::new(20.0, 60.0, 120.0, 30.0), "Click me") {
            println!("button clicked");
        }

        // Slider — mutates slider_val in place
        d.gui_slider(
            Rectangle::new(20.0, 110.0, 200.0, 20.0),
            "Min",
            "Max",
            &mut slider_val,
            0.0,
            1.0,
        );
    }
}
```

## Gotchas

- **String parameters are `impl AsRef<str>`.** Pass `&str`, `String`, or any
  type implementing `AsRef<str>`.  Under the hood, each call writes the string
  to a thread-local `CString` scratch buffer — safe and zero-alloc per frame.
- **GUI state is global.**  raygui is immediate-mode; all style and state
  (`gui_set_state`, `gui_load_style`) affects every control drawn this frame.
  Call ordering matters.
- **`gui_load_style_from_memory` is absent** from the vendored raygui header
  (backlog #296 — deferred until the vendored raygui advances).
- **`raygui` feature required.**  Without `features = ["raygui"]` the trait
  methods are not compiled in.

## See also

- [Window and drawing](./window-and-drawing.md) — the frame loop that hosts GUI calls.
- [Strings and allocations](../core-concepts/strings-and-allocs.md) — why raygui uses `impl AsRef<str>`.
- [`RaylibGuiControls` docs.rs](https://docs.rs/raylib/latest/raylib/rgui/trait.RaylibGuiControls.html)

### Showcase examples

Showcase examples that exercise this module:

- [controls_test_suite](https://dacode45.github.io/raylib-rs/examples/raygui/controls_test_suite.html)
- [floating_window](https://dacode45.github.io/raylib-rs/examples/raygui/floating_window.html)
- [property_list](https://dacode45.github.io/raylib-rs/examples/raygui/property_list.html)
- [scroll_panel](https://dacode45.github.io/raylib-rs/examples/raygui/scroll_panel.html)
- [style_selector](https://dacode45.github.io/raylib-rs/examples/raygui/style_selector.html)
- [custom_sliders](https://dacode45.github.io/raylib-rs/examples/raygui/custom_sliders.html)
