/*******************************************************************************************
*
*   raylib [core] example - 3d picking
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.0
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
        .title("raylib [core] example - 3d picking")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let cube_position = Vector3::new(0.0, 1.0, 0.0);
    let cube_size = Vector3::new(2.0, 2.0, 2.0);

    let mut ray = Ray {
        position: Vector3::zero(),
        direction: Vector3::zero(),
    }; // Picking line ray
    let mut collision = RayCollision {
        hit: false,
        distance: 0.0,
        point: Vector3::zero(),
        normal: Vector3::zero(),
    }; // Ray collision hit info

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_cursor_hidden() {
            camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);
        }

        // Toggle camera controls
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            if rl.is_cursor_hidden() {
                rl.enable_cursor();
            } else {
                rl.disable_cursor();
            }
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if !collision.hit {
                ray = rl.get_screen_to_world_ray(rl.get_mouse_position(), camera);

                // Check collision between ray and box
                let bbox = BoundingBox::new(
                    Vector3::new(
                        cube_position.x - cube_size.x / 2.0,
                        cube_position.y - cube_size.y / 2.0,
                        cube_position.z - cube_size.z / 2.0,
                    ),
                    Vector3::new(
                        cube_position.x + cube_size.x / 2.0,
                        cube_position.y + cube_size.y / 2.0,
                        cube_position.z + cube_size.z / 2.0,
                    ),
                );
                collision = bbox.get_ray_collision_box(ray);
            } else {
                collision.hit = false;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let measure_box_selected = rl.measure_text("BOX SELECTED", 30);
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            if collision.hit {
                c.draw_cube(
                    cube_position,
                    cube_size.x,
                    cube_size.y,
                    cube_size.z,
                    Color::RED,
                );
                c.draw_cube_wires(
                    cube_position,
                    cube_size.x,
                    cube_size.y,
                    cube_size.z,
                    Color::MAROON,
                );

                c.draw_cube_wires(
                    cube_position,
                    cube_size.x + 0.2,
                    cube_size.y + 0.2,
                    cube_size.z + 0.2,
                    Color::GREEN,
                );
            } else {
                c.draw_cube(
                    cube_position,
                    cube_size.x,
                    cube_size.y,
                    cube_size.z,
                    Color::GRAY,
                );
                c.draw_cube_wires(
                    cube_position,
                    cube_size.x,
                    cube_size.y,
                    cube_size.z,
                    Color::DARKGRAY,
                );
            }

            c.draw_ray(ray, Color::MAROON);
            c.draw_grid(10, 1.0);
        }

        d.draw_text(
            "Try clicking on the box with your mouse!",
            240,
            10,
            20,
            Color::DARKGRAY,
        );

        if collision.hit {
            d.draw_text(
                "BOX SELECTED",
                (screen_width - measure_box_selected) / 2,
                (screen_height as f32 * 0.1) as i32,
                30,
                Color::GREEN,
            );
        }

        d.draw_text(
            "Right click mouse to toggle camera controls",
            10,
            430,
            10,
            Color::GRAY,
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
