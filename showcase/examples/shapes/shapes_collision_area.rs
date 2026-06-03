/*******************************************************************************************
*
*   raylib [shapes] example - collision area
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 2.5
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
        .title("raylib [shapes] example - collision area")
        .build();

    // Box A: Moving box
    let mut box_a = Rectangle::new(
        10.0,
        rl.get_screen_height() as f32 / 2.0 - 50.0,
        200.0,
        100.0,
    );
    let mut box_a_speed_x: i32 = 4;

    // Box B: Mouse moved box
    let mut box_b = Rectangle::new(
        rl.get_screen_width() as f32 / 2.0 - 30.0,
        rl.get_screen_height() as f32 / 2.0 - 30.0,
        60.0,
        60.0,
    );

    let mut box_collision = Rectangle::new(0.0, 0.0, 0.0, 0.0); // Collision rectangle

    let screen_upper_limit = 40; // Top menu limits

    let mut pause = false; // Movement pause
    let mut collision; // Collision detection

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //----------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //-----------------------------------------------------
        // Move box if not paused
        if !pause {
            box_a.x += box_a_speed_x as f32;
        }

        // Bounce box on x screen limits
        if ((box_a.x + box_a.width) >= rl.get_screen_width() as f32) || (box_a.x <= 0.0) {
            box_a_speed_x *= -1;
        }

        // Update player-controlled-box (box02)
        box_b.x = rl.get_mouse_x() as f32 - box_b.width / 2.0;
        box_b.y = rl.get_mouse_y() as f32 - box_b.height / 2.0;

        // Make sure Box B does not go out of move area limits
        if (box_b.x + box_b.width) >= rl.get_screen_width() as f32 {
            box_b.x = rl.get_screen_width() as f32 - box_b.width;
        } else if box_b.x <= 0.0 {
            box_b.x = 0.0;
        }

        if (box_b.y + box_b.height) >= rl.get_screen_height() as f32 {
            box_b.y = rl.get_screen_height() as f32 - box_b.height;
        } else if box_b.y <= screen_upper_limit as f32 {
            box_b.y = screen_upper_limit as f32;
        }

        // Check boxes collision
        collision = box_a.check_collision_recs(box_b);

        // Get collision rectangle (only on collision)
        if collision {
            box_collision = box_a.get_collision_rec(box_b).unwrap_or(box_collision);
        }

        // Pause Box A movement
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            pause = !pause;
        }
        viewer.update(&mut rl, &thread);
        //-----------------------------------------------------

        // Draw
        //-----------------------------------------------------
        let screen_w = rl.get_screen_width();
        let measure = rl.measure_text("COLLISION!", 20);
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_rectangle(
            0,
            0,
            screen_width,
            screen_upper_limit,
            if collision { Color::RED } else { Color::BLACK },
        );

        d.draw_rectangle_rec(box_a, Color::GOLD);
        d.draw_rectangle_rec(box_b, Color::BLUE);

        if collision {
            // Draw collision area
            d.draw_rectangle_rec(box_collision, Color::LIME);

            // Draw collision message
            d.draw_text(
                "COLLISION!",
                screen_w / 2 - measure / 2,
                screen_upper_limit / 2 - 10,
                20,
                Color::BLACK,
            );

            // Draw collision area
            d.draw_text(
                &format!(
                    "Collision Area: {}",
                    (box_collision.width as i32) * (box_collision.height as i32)
                ),
                screen_w / 2 - 100,
                screen_upper_limit + 10,
                20,
                Color::BLACK,
            );
        }

        // Draw help instructions
        d.draw_text(
            "Press SPACE to PAUSE/RESUME",
            20,
            screen_height - 35,
            20,
            Color::LIGHTGRAY,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //-----------------------------------------------------
    }

    // De-Initialization
    //---------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //----------------------------------------------------------
}
