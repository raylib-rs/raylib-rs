/*******************************************************************************************
*
*   raylib [core] example - window should close
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2013-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [core] example - window should close")
        .build();

    rl.set_exit_key(None); // Disable KEY_ESCAPE to close window, X-button still works

    let mut exit_window_requested = false; // Flag to request window to exit
    let mut exit_window = false; // Flag to set window to exit

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !exit_window {
        // Update
        //----------------------------------------------------------------------------------
        // Detect if X-button or KEY_ESCAPE have been pressed to close window
        if rl.window_should_close() || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            exit_window_requested = true;
        }

        if exit_window_requested {
            // A request for close window has been issued, we can save data before closing
            // or just show a message asking for confirmation

            if rl.is_key_pressed(KeyboardKey::KEY_Y) {
                exit_window = true;
            } else if rl.is_key_pressed(KeyboardKey::KEY_N) {
                exit_window_requested = false;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if exit_window_requested {
            d.draw_rectangle(0, 100, screen_width, 200, Color::BLACK);
            d.draw_text(
                "Are you sure you want to exit program? [Y/N]",
                40,
                180,
                30,
                Color::WHITE,
            );
        } else {
            d.draw_text(
                "Try to close the window to get confirmation message!",
                120,
                200,
                20,
                Color::LIGHTGRAY,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
