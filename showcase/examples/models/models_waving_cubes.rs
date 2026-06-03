/*******************************************************************************************
*
*   raylib [models] example - waving cubes
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.7
*
*   Example contributed by Codecat (@codecat) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Codecat (@codecat) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ffi;
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
        .title("raylib [models] example - waving cubes")
        .build();

    // Initialize the camera
    let mut camera = Camera3D::perspective(
        Vector3::new(30.0, 20.0, 30.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        70.0,                           // Camera field-of-view Y
    );

    // Specify the amount of blocks in each direction
    let num_blocks: i32 = 15;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let time = rl.get_time();

        // Calculate time scale for cube position and size
        let scale = (2.0 + time.sin() as f32) * 0.7;

        // Move camera around the scene
        let camera_time = time * 0.3;
        camera.position.x = camera_time.cos() as f32 * 40.0;
        camera.position.z = camera_time.sin() as f32 * 40.0;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_grid(10, 5.0);

            for x in 0..num_blocks {
                for y in 0..num_blocks {
                    for z in 0..num_blocks {
                        // Scale of the blocks depends on x/y/z positions
                        let block_scale = (x + y + z) as f32 / 30.0;

                        // Scatter makes the waving effect by adding blockScale over time
                        let scatter = (block_scale * 20.0 + (time as f32 * 4.0)).sin();

                        // Calculate the cube position
                        let cube_pos = Vector3::new(
                            (x as f32 - num_blocks as f32 / 2.0) * (scale * 3.0) + scatter,
                            (y as f32 - num_blocks as f32 / 2.0) * (scale * 2.0) + scatter,
                            (z as f32 - num_blocks as f32 / 2.0) * (scale * 3.0) + scatter,
                        );

                        // Pick a color with a hue depending on cube position for the rainbow color effect
                        // NOTE: This function is quite costly to be done per cube and frame,
                        // pre-catching the results into a separate array could improve performance
                        // SAFETY: pure raylib FFI taking primitive args; no aliasing or lifetime concerns.
                        let cube_color = unsafe {
                            ffi::ColorFromHSV(((x + y + z) * 18 % 360) as f32, 0.75, 0.9)
                        };

                        // Calculate cube size
                        let cube_size = (2.4 - scale) * block_scale;

                        // And finally, draw the cube!
                        c.draw_cube(cube_pos, cube_size, cube_size, cube_size, cube_color);
                    }
                }
            }
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow handled by RAII drop.
    //--------------------------------------------------------------------------------------
}
