/*******************************************************************************************
*
*   raylib [text] example - writing anim
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.4, last time updated with raylib 1.4
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [text] example - writing anim")
        .build();

    let message = "This sample illustrates a text writing\nanimation effect! Check it out! ;)";

    let mut frames_counter: i32 = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            frames_counter += 8;
        } else {
            frames_counter += 1;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            frames_counter = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // idiomatic: upstream uses TextSubtext(message, 0, framesCounter/10); Rust slices on byte
        // boundaries — clamp to message length so we don't panic, and stay on a char boundary.
        let take = (frames_counter / 10).max(0) as usize;
        let take = take.min(message.len());
        let mut take_safe = take;
        while take_safe > 0 && !message.is_char_boundary(take_safe) {
            take_safe -= 1;
        }
        d.draw_text(&message[..take_safe], 210, 160, 20, Color::MAROON);

        d.draw_text("PRESS [ENTER] to RESTART!", 240, 260, 20, Color::LIGHTGRAY);
        d.draw_text("HOLD [SPACE] to SPEED UP!", 239, 300, 20, Color::LIGHTGRAY);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
