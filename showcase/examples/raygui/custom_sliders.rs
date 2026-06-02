/*******************************************************************************************
*
*   raygui - custom sliders
*
*   DEPENDENCIES:
*       raylib 4.0  - Windowing/input management and drawing.
*       raygui 3.0  - Immediate-mode GUI controls.
*
*   COMPILATION (Windows - MinGW):
*       gcc -o $(NAME_PART).exe $(FILE_NAME) -I../../src -lraylib -lopengl32 -lgdi32 -std=c99
*
*   LICENSE: zlib/libpng
*
*   Copyright (c) 2016-2024 Ramon Santamaria (@raysan5)
*
**********************************************************************************************/

// SIMPLIFIED: The C original defines six custom slider controls (vertical/horizontal × normal/bar/pro
// × owning) that reach into raygui internals (`guiState`, `guiLocked`, `guiAlpha`, `GuiDrawRectangle`,
// `GuiDrawText`, `GuiGetTextWidth`) that aren't exposed through `raylib::ffi`. We mirror the same
// layout (two groups: "STANDARD" and "OWNING") with the safe horizontal slider/slider-bar wrappers
// so the visual intent — split groups, shared value, edit-mode toggle on the owning side — survives.

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //---------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raygui - custom sliders")
        .build();

    let mut value: f32 = 0.5;
    let mut slider_edit_mode = false;
    let mut v_slider_edit_mode = false;
    let mut v_slider_bar_edit_mode = false;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // TODO: Implement required update logic
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        let bg_color: Color = Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR) as u32,
        );
        d.clear_background(bg_color);

        if v_slider_edit_mode || v_slider_bar_edit_mode {
            d.gui_lock();
        } else {
            d.gui_unlock();
        }

        // raygui: controls drawing
        //----------------------------------------------------------------------------------
        d.gui_group_box(Rectangle::new(66.0, 24.0, 276.0, 312.0), "STANDARD");
        d.gui_slider(
            Rectangle::new(96.0, 48.0, 216.0, 16.0),
            format!("{:0.2}", value),
            "",
            &mut value,
            0.0,
            1.0,
        );
        // SIMPLIFIED: vertical-slider variants do not exist in the safe API; use horizontal
        // sliders inside the same group so the slider count and value sharing are preserved.
        d.gui_slider(
            Rectangle::new(96.0, 120.0, 216.0, 24.0),
            format!("{:0.2}", value),
            "",
            &mut value,
            0.0,
            1.0,
        );
        d.gui_slider_bar(
            Rectangle::new(96.0, 200.0, 216.0, 24.0),
            format!("{:0.2}", value),
            "",
            &mut value,
            0.0,
            1.0,
        );

        d.gui_group_box(Rectangle::new(378.0, 24.0, 276.0, 312.0), "OWNING");
        if d.gui_slider(
            Rectangle::new(408.0, 48.0, 216.0, 16.0),
            "",
            format!("{:0.2}", value),
            &mut value,
            0.0,
            1.0,
        ) {
            slider_edit_mode = !slider_edit_mode;
        }
        // SIMPLIFIED: the owning vertical variants would track edit-mode like the C source; we
        // mirror that with the safe horizontal slider, toggling on each interaction.
        if d.gui_slider(
            Rectangle::new(408.0, 120.0, 216.0, 24.0),
            "",
            format!("{:0.2}", value),
            &mut value,
            0.0,
            1.0,
        ) {
            v_slider_edit_mode = !v_slider_edit_mode;
        }
        if d.gui_slider_bar(
            Rectangle::new(408.0, 200.0, 216.0, 24.0),
            "",
            format!("{:0.2}", value),
            &mut value,
            0.0,
            1.0,
        ) {
            v_slider_bar_edit_mode = !v_slider_bar_edit_mode;
        }
        //----------------------------------------------------------------------------------

        // Suppress unused warning on `slider_edit_mode` — it is wired symmetrically with the others.
        let _ = slider_edit_mode;

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
