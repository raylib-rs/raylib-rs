/*******************************************************************************************
*
*   raylib [shapes] example - starfield effect
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
*
*   Example contributed by JP Mortiboys (@themushroompirates) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 JP Mortiboys (@themushroompirates)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const STAR_COUNT: usize = 420;

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
        .title("raylib [shapes] example - starfield effect")
        .build();

    let bg_color = Color::DARKBLUE.lerp(Color::BLACK, 0.69);

    // Speed at which we fly forward
    let mut speed: f32 = 10.0 / 9.0;

    // We're either drawing lines or circles
    let mut draw_lines = true;

    let mut stars: [Vector3; STAR_COUNT] = [Vector3::zero(); STAR_COUNT];
    let mut stars_screen_pos: [Vector2; STAR_COUNT] = [Vector2::zero(); STAR_COUNT];

    // Setup the stars with a random position
    for i in 0..STAR_COUNT {
        stars[i].x = rl.get_random_value::<i32>(-screen_width / 2..=screen_width / 2) as f32;
        stars[i].y = rl.get_random_value::<i32>(-screen_height / 2..=screen_height / 2) as f32;
        stars[i].z = 1.0;
    }

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Change speed based on mouse
        let mouse_move = rl.get_mouse_wheel_move();
        if mouse_move as i32 != 0 {
            speed += 2.0 * mouse_move / 9.0;
        }
        if speed < 0.0 {
            speed = 0.1;
        } else if speed > 2.0 {
            speed = 2.0;
        }

        // Toggle lines / points with space bar
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            draw_lines = !draw_lines;
        }

        let dt = rl.get_frame_time();
        for i in 0..STAR_COUNT {
            // Update star's timer
            stars[i].z -= dt * speed;

            // Calculate the screen position
            stars_screen_pos[i] = Vector2::new(
                screen_width as f32 * 0.5 + stars[i].x / stars[i].z,
                screen_height as f32 * 0.5 + stars[i].y / stars[i].z,
            );

            // If the star is too old, or offscreen, it dies and we make a new random one
            if (stars[i].z < 0.0)
                || (stars_screen_pos[i].x < 0.0)
                || (stars_screen_pos[i].y < 0.0)
                || (stars_screen_pos[i].x > screen_width as f32)
                || (stars_screen_pos[i].y > screen_height as f32)
            {
                stars[i].x =
                    rl.get_random_value::<i32>(-screen_width / 2..=screen_width / 2) as f32;
                stars[i].y =
                    rl.get_random_value::<i32>(-screen_height / 2..=screen_height / 2) as f32;
                stars[i].z = 1.0;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(bg_color);

        for i in 0..STAR_COUNT {
            if draw_lines {
                // Get the time a little while ago for this star, but clamp it
                let t = (stars[i].z + 1.0 / 32.0).clamp(0.0, 1.0);

                // If it's different enough from the current time, we proceed
                if (t - stars[i].z) > 1e-3 {
                    // Calculate the screen position of the old point
                    let start_pos = Vector2::new(
                        screen_width as f32 * 0.5 + stars[i].x / t,
                        screen_height as f32 * 0.5 + stars[i].y / t,
                    );

                    // Draw a line connecting the old point to the current point
                    d.draw_line_v(start_pos, stars_screen_pos[i], Color::RAYWHITE);
                }
            } else {
                // Make the radius grow as the star ages
                let radius = lerp(stars[i].z, 1.0, 5.0);

                // Draw the circle
                d.draw_circle_v(stars_screen_pos[i], radius, Color::RAYWHITE);
            }
        }

        d.draw_text(
            &format!("[MOUSE WHEEL] Current Speed: {:.0}", 9.0 * speed / 2.0),
            10,
            40,
            20,
            Color::RAYWHITE,
        );
        d.draw_text(
            &format!(
                "[SPACE] Current draw mode: {}",
                if draw_lines { "Lines" } else { "Circles" }
            ),
            10,
            70,
            20,
            Color::RAYWHITE,
        );

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

// raymath Lerp: linear interpolation, matches C `Lerp(start, end, amount)` order
#[inline]
fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + amount * (end - start)
}
