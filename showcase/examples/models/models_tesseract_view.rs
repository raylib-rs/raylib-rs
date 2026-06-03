/*******************************************************************************************
*
*   raylib [models] example - tesseract view
*
*   NOTE: This example only works on platforms that support drag & drop (Windows, Linux, OSX, Html5?)
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Timothy van der Valk (@arceryz) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2024-2025 Timothy van der Valk (@arceryz) and Ramon Santamaria (@raysan5)
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
        .title("raylib [models] example - tesseract view")
        .build();

    // Define the camera to look into our 3d world
    let camera = Camera3D::perspective(
        Vector3::new(4.0, 4.0, 4.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 0.0, 1.0), // Camera up vector (rotation towards target)
        50.0,                        // Camera field-of-view Y
    );

    // Find the coordinates by setting XYZW to +-1
    let tesseract: [Vector4; 16] = [
        Vector4::new(1.0, 1.0, 1.0, 1.0),
        Vector4::new(1.0, 1.0, 1.0, -1.0),
        Vector4::new(1.0, 1.0, -1.0, 1.0),
        Vector4::new(1.0, 1.0, -1.0, -1.0),
        Vector4::new(1.0, -1.0, 1.0, 1.0),
        Vector4::new(1.0, -1.0, 1.0, -1.0),
        Vector4::new(1.0, -1.0, -1.0, 1.0),
        Vector4::new(1.0, -1.0, -1.0, -1.0),
        Vector4::new(-1.0, 1.0, 1.0, 1.0),
        Vector4::new(-1.0, 1.0, 1.0, -1.0),
        Vector4::new(-1.0, 1.0, -1.0, 1.0),
        Vector4::new(-1.0, 1.0, -1.0, -1.0),
        Vector4::new(-1.0, -1.0, 1.0, 1.0),
        Vector4::new(-1.0, -1.0, 1.0, -1.0),
        Vector4::new(-1.0, -1.0, -1.0, 1.0),
        Vector4::new(-1.0, -1.0, -1.0, -1.0),
    ];

    let mut rotation: f32;
    let mut transformed: [Vector3; 16] = [Vector3::new(0.0, 0.0, 0.0); 16];
    let mut w_values: [f32; 16] = [0.0; 16];

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        rotation = ffi::DEG2RAD as f32 * 45.0 * rl.get_time() as f32;

        for i in 0..16 {
            let mut p = tesseract[i];

            // Rotate the XW part of the vector
            // SAFETY: pure raymath FFI taking primitive args.
            let rot_xw = unsafe { ffi::Vector2Rotate(ffi::Vector2 { x: p.x, y: p.w }, rotation) };
            p.x = rot_xw.x;
            p.w = rot_xw.y;

            // Projection from XYZW to XYZ from perspective point (0, 0, 0, 3)
            // NOTE: Trace a ray from (0, 0, 0, 3) > p and continue until W = 0
            let c = 3.0 / (3.0 - p.w);
            p.x = c * p.x;
            p.y = c * p.y;
            p.z = c * p.z;

            // Split XYZ coordinate and W values later for drawing
            transformed[i] = Vector3::new(p.x, p.y, p.z);
            w_values[i] = p.w;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);
            for i in 0..16 {
                // Draw spheres to indicate the W value
                c.draw_sphere(transformed[i], (w_values[i] * 0.1).abs(), Color::RED);

                for j in 0..16 {
                    // Two lines are connected if they differ by 1 coordinate
                    // This way we dont have to keep an edge list
                    let v1 = tesseract[i];
                    let v2 = tesseract[j];
                    let diff = (v1.x == v2.x) as i32
                        + (v1.y == v2.y) as i32
                        + (v1.z == v2.z) as i32
                        + (v1.w == v2.w) as i32;

                    // Draw only differing by 1 coordinate and the lower index only (duplicate lines)
                    if diff == 3 && i < j {
                        c.draw_line3D(transformed[i], transformed[j], Color::MAROON);
                    }
                }
            }
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow handled by RAII drop.
    //--------------------------------------------------------------------------------------
}
