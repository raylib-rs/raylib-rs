# Input

Input in raylib-rs is queried per-frame off `RaylibHandle` — there is no callback or
event queue. Every call snapshots the state that raylib collected during the previous
`EndDrawing` call. The API covers keyboard, mouse, gamepad, and touch in a uniform
style: `is_*_down` for level queries and `is_*_pressed`/`is_*_released` for
edge-triggered queries.

## API surface

- [`RaylibHandle::is_key_down`](https://docs.rs/raylib/latest/raylib/core/input/trait.RaylibKeyboard.html#method.is_key_down) —
  `true` while a key is held.
- [`RaylibHandle::is_key_pressed`](https://docs.rs/raylib/latest/raylib/core/input/trait.RaylibKeyboard.html#method.is_key_pressed) —
  `true` on the first frame a key transitions to down (edge-triggered).
- [`RaylibHandle::get_mouse_position`](https://docs.rs/raylib/latest/raylib/core/input/trait.RaylibMouse.html#method.get_mouse_position) —
  returns a `Vector2` in window pixels, Y-down.
- [`RaylibHandle::get_mouse_wheel_move`](https://docs.rs/raylib/latest/raylib/core/input/trait.RaylibMouse.html#method.get_mouse_wheel_move) —
  returns scroll delta for the current frame.
- [`RaylibHandle::is_gamepad_available`](https://docs.rs/raylib/latest/raylib/core/input/trait.RaylibGamepad.html#method.is_gamepad_available) —
  checks whether a gamepad index is connected.
- [`RaylibHandle::get_gamepad_button_pressed`](https://docs.rs/raylib/latest/raylib/core/input/trait.RaylibGamepad.html#method.get_gamepad_button_pressed) —
  returns `Option<GamepadButton>` for the most recently pressed button.
- [`KeyboardKey`](https://docs.rs/raylib/latest/raylib/consts/enum.KeyboardKey.html) —
  enum of all keyboard scancodes (e.g., `KeyboardKey::KEY_SPACE`).
- [`MouseButton`](https://docs.rs/raylib/latest/raylib/consts/enum.MouseButton.html) —
  left / right / middle and extended buttons.
- [`GamepadButton`](https://docs.rs/raylib/latest/raylib/consts/enum.GamepadButton.html) —
  cross-platform gamepad button enum.

## Example

```rust,no_run
# extern crate raylib;
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Input demo")
        .build();

    while !rl.window_should_close() {
        // Level query — true every frame the key is held.
        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            println!("SPACE held");
        }

        // Edge query — true only on the first frame of a press.
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            println!("ENTER pressed");
        }

        let mouse = rl.get_mouse_position();
        let scroll = rl.get_mouse_wheel_move();

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);
        d.draw_text(
            &format!("mouse: ({:.0}, {:.0})  scroll: {:.1}", mouse.x, mouse.y, scroll),
            10, 10, 20, Color::DARKGRAY,
        );
    }
}
```

## Gotchas

- **`is_key_pressed` is per-frame edge-triggered.** It returns `true` only on the
  single frame where the key transitions from up to down. If you poll it every frame
  and hold the key, you will see exactly one `true` unless you also check
  `is_key_pressed_repeat` for held-down auto-repeat.
- **Mouse coordinates are Y-down.** `get_mouse_position().y` increases downward, which
  matches window pixel conventions but is the opposite of mathematical Y-up.
- **`get_gamepad_button_pressed` transmute UB (tracked-deferred).** The current
  implementation performs a `transmute::<u32, GamepadButton>` on the raw integer
  returned by raylib. If raylib returns a value outside the known `GamepadButton`
  variants, this is undefined behaviour. A soundness fix is tracked for a future PR.
  For now, rely only on `is_gamepad_button_pressed` / `is_gamepad_button_down` with
  explicit `GamepadButton` variants.

## See also

- [Window and drawing](./window-and-drawing.md) — the frame loop that input is
  polled inside.
- [`KeyboardKey` docs.rs](https://docs.rs/raylib/latest/raylib/consts/enum.KeyboardKey.html) —
  full list of key constants.

### Showcase examples

Showcase examples that exercise this module:

- [core_input_keys](https://dacode45.github.io/raylib-rs/examples/core/core_input_keys.html)
- [core_input_mouse](https://dacode45.github.io/raylib-rs/examples/core/core_input_mouse.html)
- [core_input_mouse_wheel](https://dacode45.github.io/raylib-rs/examples/core/core_input_mouse_wheel.html)
- [core_input_gamepad](https://dacode45.github.io/raylib-rs/examples/core/core_input_gamepad.html)
- [core_input_gestures](https://dacode45.github.io/raylib-rs/examples/core/core_input_gestures.html)
- [core_keyboard_testbed](https://dacode45.github.io/raylib-rs/examples/core/core_keyboard_testbed.html)
