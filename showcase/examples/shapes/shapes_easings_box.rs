/*******************************************************************************************
*
*   raylib [shapes] example - easings box
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
        .title("raylib [shapes] example - easings box")
        .build();

    // Box variables to be animated with easings
    let mut rec = Rectangle::new(rl.get_screen_width() as f32 / 2.0, -100.0, 100.0, 100.0);
    let mut rotation: f32 = 0.0;
    let mut alpha: f32 = 1.0;

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
        match state {
            0 => {
                // Move box down to center of screen
                frames_counter += 1;

                // NOTE: Remember that 3rd parameter of easing function refers to
                // desired value variation, do not confuse it with expected final value!
                rec.y = ease::elastic_out(
                    frames_counter as f32,
                    -100.0,
                    rl.get_screen_height() as f32 / 2.0 + 100.0,
                    120.0,
                );

                if frames_counter >= 120 {
                    frames_counter = 0;
                    state = 1;
                }
            }
            1 => {
                // Scale box to an horizontal bar
                frames_counter += 1;
                rec.height = ease::bounce_out(frames_counter as f32, 100.0, -90.0, 120.0);
                rec.width = ease::bounce_out(
                    frames_counter as f32,
                    100.0,
                    rl.get_screen_width() as f32,
                    120.0,
                );

                if frames_counter >= 120 {
                    frames_counter = 0;
                    state = 2;
                }
            }
            2 => {
                // Rotate horizontal bar rectangle
                frames_counter += 1;
                rotation = ease::quad_out(frames_counter as f32, 0.0, 270.0, 240.0);

                if frames_counter >= 240 {
                    frames_counter = 0;
                    state = 3;
                }
            }
            3 => {
                // Increase bar size to fill all screen
                frames_counter += 1;
                rec.height = ease::circ_out(
                    frames_counter as f32,
                    10.0,
                    rl.get_screen_width() as f32,
                    120.0,
                );

                if frames_counter >= 120 {
                    frames_counter = 0;
                    state = 4;
                }
            }
            4 => {
                // Fade out animation
                frames_counter += 1;
                alpha = ease::sine_out(frames_counter as f32, 1.0, -1.0, 160.0);

                if frames_counter >= 160 {
                    frames_counter = 0;
                    state = 5;
                }
            }
            _ => {}
        }

        // Reset animation at any moment
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            rec = Rectangle::new(rl.get_screen_width() as f32 / 2.0, -100.0, 100.0, 100.0);
            rotation = 0.0;
            alpha = 1.0;
            state = 0;
            frames_counter = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_rectangle_pro(
            rec,
            Vector2::new(rec.width / 2.0, rec.height / 2.0),
            rotation,
            Color::BLACK.alpha(alpha),
        );

        d.draw_text(
            "PRESS [SPACE] TO RESET BOX ANIMATION!",
            10,
            screen_h - 25,
            20,
            Color::LIGHTGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
