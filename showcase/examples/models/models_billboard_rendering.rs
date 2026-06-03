/*******************************************************************************************
*
*   raylib [models] example - billboard rendering
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [models] example - billboard rendering")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(5.0, 4.0, 5.0), // Camera position
        Vector3::new(0.0, 2.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        45.0,                        // Camera field-of-view Y
    );

    let bill = rl
        .load_texture(&thread, "resources/models/billboard.png")
        .unwrap(); // Our billboard texture
    let bill_position_static = Vector3::new(0.0, 2.0, 0.0); // Position of static billboard
    let bill_position_rotating = Vector3::new(1.0, 2.0, 1.0); // Position of rotating billboard

    // Entire billboard texture, source is used to take a segment from a larger texture
    let source = Rectangle {
        x: 0.0,
        y: 0.0,
        width: bill.width as f32,
        height: bill.height as f32,
    };

    // NOTE: Billboard locked on axis-Y
    let bill_up = Vector3::new(0.0, 1.0, 0.0);

    // Set the height of the rotating billboard to 1.0 with the aspect ratio fixed
    let size = Vector2::new(source.width / source.height, 1.0);

    // Rotate around origin
    // Here we choose to rotate around the image center
    let origin = size.scale(0.5);

    // Distance is needed for the correct billboard draw order
    // Larger distance (further away from the camera) should be drawn prior to smaller distance
    let mut distance_static;
    let mut distance_rotating;
    let mut rotation: f32 = 0.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        camera.update_camera(CameraMode::CAMERA_ORBITAL);

        rotation += 0.4;
        distance_static = camera.position.distance(bill_position_static);
        distance_rotating = camera.position.distance(bill_position_rotating);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_grid(10, 1.0); // Draw a grid

            // Draw order matters!
            if distance_static > distance_rotating {
                c.draw_billboard(camera, &bill, bill_position_static, 2.0, Color::WHITE);
                c.draw_billboard_pro(
                    camera,
                    *bill,
                    source,
                    bill_position_rotating,
                    bill_up,
                    size,
                    origin,
                    rotation,
                    Color::WHITE,
                );
            } else {
                c.draw_billboard_pro(
                    camera,
                    *bill,
                    source,
                    bill_position_rotating,
                    bill_up,
                    size,
                    origin,
                    rotation,
                    Color::WHITE,
                );
                c.draw_billboard(camera, &bill, bill_position_static, 2.0, Color::WHITE);
            }
        }

        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / CloseWindow are handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
