/*******************************************************************************************
*
*   raylib [shapes] example - easings ball
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ease;
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
        .title("raylib [shapes] example - easings ball")
        .build();

    // Ball variable value to be animated with easings
    let mut ball_position_x: i32 = -100;
    let mut ball_radius: i32 = 20;
    let mut ball_alpha: f32 = 0.0;

    let mut state = 0;
    let mut frames_counter = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        #[expect(clippy::collapsible_if, reason = "C-parity: C nests the conditionals")]
        if state == 0
        // Move ball position X with easing
        {
            frames_counter += 1;
            ball_position_x = ease::elastic_out(
                frames_counter as f32,
                -100.0,
                screen_width as f32 / 2.0 + 100.0,
                120.0,
            ) as i32;

            if frames_counter >= 120 {
                frames_counter = 0;
                state = 1;
            }
        } else if state == 1
        // Increase ball radius with easing
        {
            frames_counter += 1;
            ball_radius = ease::elastic_in(frames_counter as f32, 20.0, 500.0, 200.0) as i32;

            if frames_counter >= 200 {
                frames_counter = 0;
                state = 2;
            }
        } else if state == 2
        // Change ball alpha with easing (background color blending)
        {
            frames_counter += 1;
            ball_alpha = ease::cubic_out(frames_counter as f32, 0.0, 1.0, 200.0);

            if frames_counter >= 200 {
                frames_counter = 0;
                state = 3;
            }
        } else if state == 3
        // Reset state to play again
        {
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                // Reset required variables to play again
                ball_position_x = -100;
                ball_radius = 20;
                ball_alpha = 0.0;
                state = 0;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            frames_counter = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if state >= 2 {
            d.draw_rectangle(0, 0, screen_width, screen_height, Color::GREEN);
        }
        d.draw_circle(
            ball_position_x,
            200,
            ball_radius as f32,
            Color::RED.alpha(1.0 - ball_alpha),
        );

        if state == 3 {
            d.draw_text("PRESS [ENTER] TO PLAY AGAIN!", 240, 200, 20, Color::BLACK);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
