/*******************************************************************************************
*
*   raylib [core] example - random values
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.1, last time updated with raylib 1.1
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
        .title("raylib [core] example - random values")
        .build();

    // rl.set_random_seed(0xaabbccff);   // Set a custom random seed if desired, by default: "time(NULL)"

    let mut rand_value: i32 = rl.get_random_value(-8..=5); // Get a random integer number between -8 and 5 (both included)

    let mut frames_counter: u32 = 0; // Variable used to count frames

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        frames_counter += 1;

        // Every two seconds (120 frames) a new random value is generated
        if ((frames_counter / 120) % 2) == 1 {
            rand_value = rl.get_random_value(-8..=5);
            frames_counter = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Every 2 seconds a new random value is generated:",
            130,
            100,
            20,
            Color::MAROON,
        );

        d.draw_text(&format!("{}", rand_value), 360, 180, 80, Color::LIGHTGRAY);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
