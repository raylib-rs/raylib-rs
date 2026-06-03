/*******************************************************************************************
*
*   raylib [core] example - 3d camera mode
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 1.0
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
        .title("raylib [core] example - 3d camera mode")
        .build();

    // Define the camera to look into our 3d world
    let camera = Camera3D::perspective(
        Vector3::new(0.0, 10.0, 10.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),   // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),   // Camera up vector (rotation towards target)
        45.0,                          // Camera field-of-view Y
    );

    let cube_position = Vector3::new(0.0, 0.0, 0.0);

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
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_cube(cube_position, 2.0, 2.0, 2.0, Color::RED);
            c.draw_cube_wires(cube_position, 2.0, 2.0, 2.0, Color::MAROON);

            c.draw_grid(10, 1.0);
        }

        d.draw_text(
            "Welcome to the third dimension!",
            10,
            40,
            20,
            Color::DARKGRAY,
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
