/*******************************************************************************************
*
*   raylib [models] example - geometric shapes
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 3.5
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
        .title("raylib [models] example - geometric shapes")
        .build();

    // Define the camera to look into our 3d world
    let camera = Camera3D::perspective(
        Vector3::new(0.0, 10.0, 10.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        45.0,
    );

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

            c.draw_cube(Vector3::new(-4.0, 0.0, 2.0), 2.0, 5.0, 2.0, Color::RED);
            c.draw_cube_wires(Vector3::new(-4.0, 0.0, 2.0), 2.0, 5.0, 2.0, Color::GOLD);
            c.draw_cube_wires(Vector3::new(-4.0, 0.0, -2.0), 3.0, 6.0, 2.0, Color::MAROON);

            c.draw_sphere(Vector3::new(-1.0, 0.0, -2.0), 1.0, Color::GREEN);
            c.draw_sphere_wires(Vector3::new(1.0, 0.0, 2.0), 2.0, 16, 16, Color::LIME);

            c.draw_cylinder(
                Vector3::new(4.0, 0.0, -2.0),
                1.0,
                2.0,
                3.0,
                4,
                Color::SKYBLUE,
            );
            c.draw_cylinder_wires(
                Vector3::new(4.0, 0.0, -2.0),
                1.0,
                2.0,
                3.0,
                4,
                Color::DARKBLUE,
            );
            c.draw_cylinder_wires(Vector3::new(4.5, -1.0, 2.0), 1.0, 1.0, 2.0, 6, Color::BROWN);

            c.draw_cylinder(Vector3::new(1.0, 0.0, -4.0), 0.0, 1.5, 3.0, 8, Color::GOLD);
            c.draw_cylinder_wires(Vector3::new(1.0, 0.0, -4.0), 0.0, 1.5, 3.0, 8, Color::PINK);

            c.draw_capsule(
                Vector3::new(-3.0, 1.5, -4.0),
                Vector3::new(-4.0, -1.0, -4.0),
                1.2,
                8,
                8,
                Color::VIOLET,
            );
            c.draw_capsule_wires(
                Vector3::new(-3.0, 1.5, -4.0),
                Vector3::new(-4.0, -1.0, -4.0),
                1.2,
                8,
                8,
                Color::PURPLE,
            );

            c.draw_grid(10, 1.0); // Draw a grid
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
