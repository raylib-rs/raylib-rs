/*******************************************************************************************
*
* raylib [shapes] example - Draw a mouse trail (position history)
*
* Example complexity rating: [★☆☆☆] 1/4
*
* Example originally created with raylib 5.6
*
* Example contributed by Balamurugan R (@Bala050814]) and reviewed by Ramon Santamaria (@raysan5)
*
* Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
* BSD-like license that allows static linking with closed source software
*
* Copyright (c) 2025 Balamurugan R (@Bala050814)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Define the maximum number of positions to store in the trail
const MAX_TRAIL_LENGTH: usize = 30;

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
        .title("raylib [shapes] example - mouse trail")
        .build();

    // Array to store the history of mouse positions (our fixed-size queue)
    let mut trail_positions: [Vector2; MAX_TRAIL_LENGTH] = [Vector2::zero(); MAX_TRAIL_LENGTH];

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mouse_position = rl.get_mouse_position();

        // Shift all existing positions backward by one slot in the array
        // The last element (the oldest position) is dropped
        for i in (1..MAX_TRAIL_LENGTH).rev() {
            trail_positions[i] = trail_positions[i - 1];
        }

        // Store the new, current mouse position at the start of the array (Index 0)
        trail_positions[0] = mouse_position;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        // Draw the trail by looping through the history array
        for i in 0..MAX_TRAIL_LENGTH {
            // Ensure we skip drawing if the array hasn't been fully filled on startup
            if (trail_positions[i].x != 0.0) || (trail_positions[i].y != 0.0) {
                // Calculate relative trail strength (ratio is near 1.0 for new, near 0.0 for old)
                let ratio = (MAX_TRAIL_LENGTH - i) as f32 / MAX_TRAIL_LENGTH as f32;

                // Fade effect: oldest positions are more transparent
                // Fade (color, alpha) - alpha is 0.5 to 1.0 based on ratio
                let trail_color = Color::SKYBLUE.alpha(ratio * 0.5 + 0.5);

                // Size effect: oldest positions are smaller
                let trail_radius = 15.0 * ratio;

                d.draw_circle_v(trail_positions[i], trail_radius, trail_color);
            }
        }

        // Draw a distinct white circle for the current mouse position (Index 0)
        d.draw_circle_v(mouse_position, 15.0, Color::WHITE);

        d.draw_text(
            "Move the mouse to see the trail effect!",
            10,
            screen_height - 30,
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
