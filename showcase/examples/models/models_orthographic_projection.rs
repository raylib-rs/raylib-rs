/*******************************************************************************************
*
*   raylib [models] example - orthographic projection
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 2.0, last time updated with raylib 3.7
*
*   Example contributed by Max Danielsson (@autious) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Max Danielsson (@autious) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::CameraProjection;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const FOVY_PERSPECTIVE: f32 = 45.0;
const WIDTH_ORTHOGRAPHIC: f32 = 10.0;

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
        .title("raylib [models] example - orthographic projection")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(0.0, 10.0, 10.0),
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 1.0, 0.0),
        FOVY_PERSPECTIVE,
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
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if camera.projection == CameraProjection::CAMERA_PERSPECTIVE {
                camera.fovy = WIDTH_ORTHOGRAPHIC;
                camera.projection = CameraProjection::CAMERA_ORTHOGRAPHIC;
            } else {
                camera.fovy = FOVY_PERSPECTIVE;
                camera.projection = CameraProjection::CAMERA_PERSPECTIVE;
            }
        }
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

            c.draw_grid(10, 1.0); // Draw a grid
        }

        let sh = d.get_screen_height();
        d.draw_text(
            "Press Spacebar to switch camera type",
            10,
            sh - 30,
            20,
            Color::DARKGRAY,
        );

        if camera.projection == CameraProjection::CAMERA_ORTHOGRAPHIC {
            d.draw_text("ORTHOGRAPHIC", 10, 40, 20, Color::BLACK);
        } else if camera.projection == CameraProjection::CAMERA_PERSPECTIVE {
            d.draw_text("PERSPECTIVE", 10, 40, 20, Color::BLACK);
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
