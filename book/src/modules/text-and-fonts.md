# Text and fonts

raylib bundles a default bitmap font that is available as soon as the window is open.
Custom fonts are loaded via `RaylibHandle::load_font` (from a file) or
`RaylibHandle::load_font_ex` (with an explicit codepoint set for preloaded glyph
atlases). Both return a RAII `Font` that is automatically unloaded on drop.
Text drawing — including measuring — hangs off `RaylibHandle` for the default font
and off the `Font` struct or `RaylibDraw` trait for custom fonts.

## API surface

- [`RaylibHandle::load_font`](https://docs.rs/raylib/latest/raylib/core/text/trait.RaylibFont.html#method.load_font) —
  load a font from a file; returns `Result<Font, …>`.
- [`RaylibHandle::load_font_ex`](https://docs.rs/raylib/latest/raylib/core/text/trait.RaylibFont.html#method.load_font_ex) —
  load a font with explicit font size and a codepoint array; useful for preloading a
  specific non-ASCII character set.
- [`RaylibHandle::measure_text`](https://docs.rs/raylib/latest/raylib/core/struct.RaylibHandle.html#method.measure_text) —
  measure the pixel width of a string at a given size using the default font.
- [`Font::measure_text`](https://docs.rs/raylib/latest/raylib/core/text/struct.Font.html#method.measure_text) —
  measure a string with a custom font, returning a `Vector2` (width × height).
- [`RaylibDraw::draw_text`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_text) —
  draw a `&str` using the default font.
- [`RaylibDraw::draw_text_ex`](https://docs.rs/raylib/latest/raylib/core/drawing/trait.RaylibDraw.html#method.draw_text_ex) —
  draw a `&str` using a custom `Font`, with position, size, spacing, and tint.

## Example

Drawing text requires an open window. The example below opens a 640×480 window, draws
a greeting with the default font each frame, and measures its width for centering.

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Text demo")
        .vsync()
        .build();

    let message = "Hello, raylib-rs!";
    let font_size = 24;

    while !rl.window_should_close() {
        // Measure text width for the default font.
        let width = rl.measure_text(message, font_size);

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Centered draw using the default font.
        d.draw_text(
            message,
            (640 - width) / 2,
            (480 - font_size) / 2,
            font_size,
            Color::DARKGRAY,
        );
    }
}
```

## Gotchas

- **Custom fonts need a window.** `load_font` and `load_font_ex` call into raylib's
  GPU atlas-building path, so they must be called after `raylib::init().build()`.
  Font data files (`.ttf`, `.otf`, `.fnt`, `.bdf`) must be on disk at the given path.
- **Codepoint preloading.** `load_font_ex` takes an optional `&[i32]` codepoint list.
  If `None`, raylib loads the default Latin set (32–127). Pass an explicit list if
  your application draws CJK, emoji, or other extended Unicode ranges.
- **raylib 6.0 new text symbols.** The 6.0 release added more than 30 new text utility
  functions in the C library. The Rust parity checklist (`docs/superpowers/plans/`)
  tracks which are exposed; check the
  [`text` module rustdoc](https://docs.rs/raylib/latest/raylib/core/text/) for the
  current set.

## See also

- [Window and drawing](./window-and-drawing.md) — the draw handle that text methods
  live on.
- [`Font` docs.rs](https://docs.rs/raylib/latest/raylib/core/text/struct.Font.html) —
  full rustdoc for custom font operations.

### Showcase examples

Showcase examples that exercise this module:

- [text_font_loading](https://raylib-rs.github.io/raylib-rs/examples/text/text_font_loading.html)
- [text_font_sdf](https://raylib-rs.github.io/raylib-rs/examples/text/text_font_sdf.html)
- [text_format_text](https://raylib-rs.github.io/raylib-rs/examples/text/text_format_text.html)
- [text_input_box](https://raylib-rs.github.io/raylib-rs/examples/text/text_input_box.html)
- [text_rectangle_bounds](https://raylib-rs.github.io/raylib-rs/examples/text/text_rectangle_bounds.html)
- [text_unicode_emojis](https://raylib-rs.github.io/raylib-rs/examples/text/text_unicode_emojis.html)
