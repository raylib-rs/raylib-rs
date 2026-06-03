/*******************************************************************************************
*
*   raylib [shapes] example - logo raylib anim
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
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
    let screen_width: i32 = 800;
    let screen_height: i32 = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shapes] example - logo raylib anim")
        .build();

    let logo_position_x = screen_width / 2 - 128;
    let logo_position_y = screen_height / 2 - 128;

    let mut frames_counter: i32 = 0;
    let mut letters_count: i32 = 0;

    let mut top_side_rec_width: i32 = 16;
    let mut left_side_rec_height: i32 = 16;

    let mut bottom_side_rec_width: i32 = 16;
    let mut right_side_rec_height: i32 = 16;

    let mut state: i32 = 0; // Tracking animation states (State Machine)
    let mut alpha: f32 = 1.0; // Useful for fading

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if state == 0
        // State 0: Small box blinking
        {
            frames_counter += 1;

            if frames_counter == 120 {
                state = 1;
                frames_counter = 0; // Reset counter... will be used later...
            }
        } else if state == 1
        // State 1: Top and left bars growing
        {
            top_side_rec_width += 4;
            left_side_rec_height += 4;

            if top_side_rec_width == 256 {
                state = 2;
            }
        } else if state == 2
        // State 2: Bottom and right bars growing
        {
            bottom_side_rec_width += 4;
            right_side_rec_height += 4;

            if bottom_side_rec_width == 256 {
                state = 3;
            }
        } else if state == 3
        // State 3: Letters appearing (one by one)
        {
            frames_counter += 1;

            if frames_counter / 12 != 0
            // Every 12 frames, one more letter!
            {
                letters_count += 1;
                frames_counter = 0;
            }

            if letters_count >= 10
            // When all letters have appeared, just fade out everything
            {
                alpha -= 0.02;

                if alpha <= 0.0 {
                    alpha = 0.0;
                    state = 4;
                }
            }
        } else if state == 4
        // State 4: Reset and Replay
        {
            if rl.is_key_pressed(KeyboardKey::KEY_R) {
                frames_counter = 0;
                letters_count = 0;

                top_side_rec_width = 16;
                left_side_rec_height = 16;

                bottom_side_rec_width = 16;
                right_side_rec_height = 16;

                alpha = 1.0;
                state = 0; // Return to State 0
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if state == 0 {
            if (frames_counter / 15) % 2 != 0 {
                d.draw_rectangle(logo_position_x, logo_position_y, 16, 16, Color::BLACK);
            }
        } else if state == 1 {
            d.draw_rectangle(
                logo_position_x,
                logo_position_y,
                top_side_rec_width,
                16,
                Color::BLACK,
            );
            d.draw_rectangle(
                logo_position_x,
                logo_position_y,
                16,
                left_side_rec_height,
                Color::BLACK,
            );
        } else if state == 2 {
            d.draw_rectangle(
                logo_position_x,
                logo_position_y,
                top_side_rec_width,
                16,
                Color::BLACK,
            );
            d.draw_rectangle(
                logo_position_x,
                logo_position_y,
                16,
                left_side_rec_height,
                Color::BLACK,
            );

            d.draw_rectangle(
                logo_position_x + 240,
                logo_position_y,
                16,
                right_side_rec_height,
                Color::BLACK,
            );
            d.draw_rectangle(
                logo_position_x,
                logo_position_y + 240,
                bottom_side_rec_width,
                16,
                Color::BLACK,
            );
        } else if state == 3 {
            d.draw_rectangle(
                logo_position_x,
                logo_position_y,
                top_side_rec_width,
                16,
                Color::BLACK.alpha(alpha),
            );
            d.draw_rectangle(
                logo_position_x,
                logo_position_y + 16,
                16,
                left_side_rec_height - 32,
                Color::BLACK.alpha(alpha),
            );

            d.draw_rectangle(
                logo_position_x + 240,
                logo_position_y + 16,
                16,
                right_side_rec_height - 32,
                Color::BLACK.alpha(alpha),
            );
            d.draw_rectangle(
                logo_position_x,
                logo_position_y + 240,
                bottom_side_rec_width,
                16,
                Color::BLACK.alpha(alpha),
            );

            d.draw_rectangle(
                screen_w / 2 - 112,
                screen_h / 2 - 112,
                224,
                224,
                Color::RAYWHITE.alpha(alpha),
            );

            d.draw_text(
                &"raylib"[..(letters_count.max(0) as usize).min(6)],
                screen_w / 2 - 44,
                screen_h / 2 + 48,
                50,
                Color::BLACK.alpha(alpha),
            );
        } else if state == 4 {
            d.draw_text("[R] REPLAY", 340, 200, 20, Color::GRAY);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
