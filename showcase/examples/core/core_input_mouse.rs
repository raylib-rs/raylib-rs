/*******************************************************************************************
*
*   raylib [core] example - input mouse
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 5.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - input mouse")
        .build();

    let mut ball_position = Vector2::new(-100.0, -100.0);
    let mut ball_color = Color::DARKBLUE;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_H) {
            if rl.is_cursor_hidden() {
                rl.show_cursor();
            } else {
                rl.hide_cursor();
            }
        }

        ball_position = rl.get_mouse_position();

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            ball_color = Color::MAROON;
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE) {
            ball_color = Color::LIME;
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            ball_color = Color::DARKBLUE;
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_SIDE) {
            ball_color = Color::PURPLE;
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_EXTRA) {
            ball_color = Color::YELLOW;
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_FORWARD) {
            ball_color = Color::ORANGE;
        } else if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_BACK) {
            ball_color = Color::BEIGE;
        }
        let cursor_hidden = rl.is_cursor_hidden();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_circle_v(ball_position, 40.0, ball_color);

        d.draw_text(
            "move ball with mouse and click mouse button to change color",
            10,
            10,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Press 'H' to toggle cursor visibility",
            10,
            30,
            20,
            Color::DARKGRAY,
        );

        if cursor_hidden {
            d.draw_text("CURSOR HIDDEN", 20, 60, 20, Color::RED);
        } else {
            d.draw_text("CURSOR VISIBLE", 20, 60, 20, Color::LIME);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
