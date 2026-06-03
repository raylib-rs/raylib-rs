/*******************************************************************************************
*
*   raylib [text] example - format text
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.1, last time updated with raylib 3.0
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
        .title("raylib [text] example - format text")
        .build();

    let score = 100020;
    let hiscore = 200450;
    let lives = 5;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // TODO: Update your variables here
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let frame_time = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // idiomatic: upstream uses TextFormat("Score: %08i", score); Rust uses format!
        d.draw_text(&format!("Score: {:08}", score), 200, 80, 20, Color::RED);

        d.draw_text(
            &format!("HiScore: {:08}", hiscore),
            200,
            120,
            20,
            Color::GREEN,
        );

        d.draw_text(&format!("Lives: {:02}", lives), 200, 160, 40, Color::BLUE);

        d.draw_text(
            &format!("Elapsed Time: {:05.2} ms", frame_time * 1000.0),
            200,
            220,
            20,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
