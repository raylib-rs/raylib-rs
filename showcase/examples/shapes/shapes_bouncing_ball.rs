/*******************************************************************************************
*
*   raylib [shapes] example - bouncing ball
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
*
*   Example contributed by Ramon Santamaria (@raysan5), reviewed by Jopestpe (@jopestpe)
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
    //---------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shapes] example - bouncing ball")
        .msaa_4x()
        .build();

    let mut ball_position = Vector2::new(
        rl.get_screen_width() as f32 / 2.0,
        rl.get_screen_height() as f32 / 2.0,
    );
    let mut ball_speed = Vector2::new(5.0, 4.0);
    let ball_radius: i32 = 20;
    let gravity: f32 = 0.2;

    let mut use_gravity = true;
    let mut pause = false;
    let mut frames_counter = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //-----------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_G) {
            use_gravity = !use_gravity;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            pause = !pause;
        }

        if !pause {
            ball_position.x += ball_speed.x;
            ball_position.y += ball_speed.y;

            if use_gravity {
                ball_speed.y += gravity;
            }

            // Check walls collision for bouncing
            if (ball_position.x >= (rl.get_screen_width() - ball_radius) as f32)
                || (ball_position.x <= ball_radius as f32)
            {
                ball_speed.x *= -1.0;
            }
            if (ball_position.y >= (rl.get_screen_height() - ball_radius) as f32)
                || (ball_position.y <= ball_radius as f32)
            {
                ball_speed.y *= -0.95;
            }
        } else {
            frames_counter += 1;
        }
        viewer.update(&mut rl, &thread);
        //-----------------------------------------------------

        // Draw
        //-----------------------------------------------------
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_circle_v(ball_position, ball_radius as f32, Color::MAROON);
        d.draw_text(
            "PRESS SPACE to PAUSE BALL MOVEMENT",
            10,
            screen_h - 25,
            20,
            Color::LIGHTGRAY,
        );

        if use_gravity {
            d.draw_text(
                "GRAVITY: ON (Press G to disable)",
                10,
                screen_h - 50,
                20,
                Color::DARKGREEN,
            );
        } else {
            d.draw_text(
                "GRAVITY: OFF (Press G to enable)",
                10,
                screen_h - 50,
                20,
                Color::RED,
            );
        }

        // On pause, we draw a blinking message
        if pause && ((frames_counter / 30) % 2) != 0 {
            d.draw_text("PAUSED", 350, 200, 30, Color::GRAY);
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //-----------------------------------------------------
    }

    // De-Initialization
    //---------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //----------------------------------------------------------
}
