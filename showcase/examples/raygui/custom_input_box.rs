/*******************************************************************************************
*
*   raygui - basic calculator app with custom input box for float values
*
*   DEPENDENCIES:
*       raylib 4.5  - Windowing/input management and drawing.
*       raygui 3.5  - Immediate-mode GUI controls.
*
*   COMPILATION (Windows - MinGW):
*       gcc -o $(NAME_PART).exe $(FILE_NAME) -I../../src -lraylib -lopengl32 -lgdi32 -std=c99
*
**********************************************************************************************/

// SIMPLIFIED: The C original defines a custom `GuiFloatBox` control that reaches into raygui's
// internal state (`guiState`, `guiLocked`, `guiAlpha`, `guiFloatingPointIndex`, and the private
// `GuiDrawRectangle`/`GuiDrawText`/`GetTextBounds` helpers). None of those are part of raygui's
// public C API and so they are not exposed through `raylib::ffi`. We mirror the calculator's
// visual layout using the safe `gui_value_box_float` wrapper (which internally calls
// `GuiValueBoxFloat`); the resulting widgets accept signed floats and have the same bounds,
// labels, and arithmetic-button placement as the C source.

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(250, 100)
        .title("Basic calculator")
        .build();

    // General variables
    rl.set_target_fps(60);

    let mut variable_a: f32 = 0.0;
    let mut variable_b: f32 = 0.0;
    let mut result: f32 = 0.0;
    let mut operation: char = '+';

    // idiomatic: the edit-text buffers used by `gui_value_box_float`. Reserve at least
    // 32 bytes (RAYGUI_VALUEBOX_MAX_CHARS + 1) so raygui can edit in place.
    let mut variable_a_text = String::with_capacity(33);
    let mut variable_b_text = String::with_capacity(33);
    let mut result_text = String::with_capacity(33);

    let mut variable_a_mode = false;
    let mut variable_b_mode = false;
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close() {
        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if d.gui_value_box_float(
            Rectangle::new(10.0, 10.0, 100.0, 20.0),
            "",
            &mut variable_a_text,
            &mut variable_a,
            variable_a_mode,
        ) {
            variable_a_mode = !variable_a_mode;
        }
        if d.gui_value_box_float(
            Rectangle::new(140.0, 10.0, 100.0, 20.0),
            "",
            &mut variable_b_text,
            &mut variable_b,
            variable_b_mode,
        ) {
            variable_b_mode = !variable_b_mode;
        }

        if d.gui_button(Rectangle::new(10.0, 70.0, 50.0, 20.0), "+") {
            result = variable_a + variable_b;
            operation = '+';
        }
        if d.gui_button(Rectangle::new(70.0, 70.0, 50.0, 20.0), "-") {
            result = variable_a - variable_b;
            operation = '-';
        }
        if d.gui_button(Rectangle::new(130.0, 70.0, 50.0, 20.0), "*") {
            result = variable_a * variable_b;
            operation = '*';
        }
        if d.gui_button(Rectangle::new(190.0, 70.0, 50.0, 20.0), "/") {
            result = variable_a / variable_b;
            operation = '/';
        }

        d.draw_text(&operation.to_string(), 123, 15, 10, Color::DARKGRAY);

        d.gui_value_box_float(
            Rectangle::new(55.0, 40.0, 135.0, 20.0),
            "= ",
            &mut result_text,
            &mut result,
            false,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
