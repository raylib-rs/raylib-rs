/*******************************************************************************************
*
*   raylib [core] example - 3d camera first person
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 1.3
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_COLUMNS: usize = 20;
const DEG2RAD: f32 = std::f32::consts::PI / 180.0;

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
        .title("raylib [core] example - 3d camera first person")
        .build();

    // Define the camera to look into our 3d world (position, target, up vector)
    let mut camera = Camera3D::perspective(
        Vector3::new(0.0, 2.0, 4.0), // Camera position
        Vector3::new(0.0, 2.0, 0.0), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        60.0,                        // Camera field-of-view Y
    );

    let mut camera_mode = CameraMode::CAMERA_FIRST_PERSON;

    // Generates some random columns
    let mut heights = [0.0_f32; MAX_COLUMNS];
    let mut positions = [Vector3::new(0.0, 0.0, 0.0); MAX_COLUMNS];
    let mut colors = [Color::new(0, 0, 0, 0); MAX_COLUMNS];

    for i in 0..MAX_COLUMNS {
        heights[i] = rl.get_random_value::<i32>(1..=12) as f32;
        positions[i] = Vector3::new(
            rl.get_random_value::<i32>(-15..=15) as f32,
            heights[i] / 2.0,
            rl.get_random_value::<i32>(-15..=15) as f32,
        );
        colors[i] = Color::new(
            rl.get_random_value::<i32>(20..=255) as u8,
            rl.get_random_value::<i32>(10..=55) as u8,
            30,
            255,
        );
    }

    rl.disable_cursor(); // Limit cursor to relative movement inside the window

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Switch camera mode
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            camera_mode = CameraMode::CAMERA_FREE;
            camera.up = Vector3::new(0.0, 1.0, 0.0); // Reset roll
        }

        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            camera_mode = CameraMode::CAMERA_FIRST_PERSON;
            camera.up = Vector3::new(0.0, 1.0, 0.0); // Reset roll
        }

        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            camera_mode = CameraMode::CAMERA_THIRD_PERSON;
            camera.up = Vector3::new(0.0, 1.0, 0.0); // Reset roll
        }

        if rl.is_key_pressed(KeyboardKey::KEY_FOUR) {
            camera_mode = CameraMode::CAMERA_ORBITAL;
            camera.up = Vector3::new(0.0, 1.0, 0.0); // Reset roll
        }

        // Switch camera projection
        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            if camera.projection == CameraProjection::CAMERA_PERSPECTIVE {
                // Create isometric view
                camera_mode = CameraMode::CAMERA_THIRD_PERSON;
                // Note: The target distance is related to the render distance in the orthographic projection
                camera.position = Vector3::new(0.0, 2.0, -100.0);
                camera.target = Vector3::new(0.0, 2.0, 0.0);
                camera.up = Vector3::new(0.0, 1.0, 0.0);
                camera.projection = CameraProjection::CAMERA_ORTHOGRAPHIC;
                camera.fovy = 20.0; // near plane width in CAMERA_ORTHOGRAPHIC
                camera.yaw(-135.0 * DEG2RAD, true);
                camera.pitch(-45.0 * DEG2RAD, true, true, false);
            } else if camera.projection == CameraProjection::CAMERA_ORTHOGRAPHIC {
                // Reset to default view
                camera_mode = CameraMode::CAMERA_THIRD_PERSON;
                camera.position = Vector3::new(0.0, 2.0, 10.0);
                camera.target = Vector3::new(0.0, 2.0, 0.0);
                camera.up = Vector3::new(0.0, 1.0, 0.0);
                camera.projection = CameraProjection::CAMERA_PERSPECTIVE;
                camera.fovy = 60.0;
            }
        }

        // Update camera computes movement internally depending on the camera mode
        // Some default standard keyboard/mouse inputs are hardcoded to simplify use
        // For advanced camera controls, it's recommended to compute camera movement manually
        camera.update_camera(camera_mode); // Update camera

        /*
                // Camera PRO usage example (EXPERIMENTAL)
                // This new camera function allows custom movement/rotation values to be directly provided
                // as input parameters, with this approach, rcamera module is internally independent of raylib inputs
                UpdateCameraPro(&camera,
                    (Vector3){
                        (IsKeyDown(KEY_W) || IsKeyDown(KEY_UP))*0.1f -      // Move forward-backward
                        (IsKeyDown(KEY_S) || IsKeyDown(KEY_DOWN))*0.1f,
                        (IsKeyDown(KEY_D) || IsKeyDown(KEY_RIGHT))*0.1f -   // Move right-left
                        (IsKeyDown(KEY_A) || IsKeyDown(KEY_LEFT))*0.1f,
                        0.0f                                                // Move up-down
                    },
                    (Vector3){
                        GetMouseDelta().x*0.05f,                            // Rotation: yaw
                        GetMouseDelta().y*0.05f,                            // Rotation: pitch
                        0.0f                                                // Rotation: roll
                    },
                    GetMouseWheelMove()*2.0f);                              // Move to target (zoom)
        */
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut c = d.begin_mode3D(camera);

            c.draw_plane(
                Vector3::new(0.0, 0.0, 0.0),
                Vector2::new(32.0, 32.0),
                Color::LIGHTGRAY,
            ); // Draw ground
            c.draw_cube(Vector3::new(-16.0, 2.5, 0.0), 1.0, 5.0, 32.0, Color::BLUE); // Draw a blue wall
            c.draw_cube(Vector3::new(16.0, 2.5, 0.0), 1.0, 5.0, 32.0, Color::LIME); // Draw a green wall
            c.draw_cube(Vector3::new(0.0, 2.5, 16.0), 32.0, 5.0, 1.0, Color::GOLD); // Draw a yellow wall

            // Draw some cubes around
            for i in 0..MAX_COLUMNS {
                c.draw_cube(positions[i], 2.0, heights[i], 2.0, colors[i]);
                c.draw_cube_wires(positions[i], 2.0, heights[i], 2.0, Color::MAROON);
            }

            // Draw player cube
            if camera_mode == CameraMode::CAMERA_THIRD_PERSON {
                c.draw_cube(camera.target, 0.5, 0.5, 0.5, Color::PURPLE);
                c.draw_cube_wires(camera.target, 0.5, 0.5, 0.5, Color::DARKPURPLE);
            }
        }

        // Draw info boxes
        d.draw_rectangle(5, 5, 330, 100, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(5, 5, 330, 100, Color::BLUE);

        d.draw_text("Camera controls:", 15, 15, 10, Color::BLACK);
        d.draw_text(
            "- Move keys: W, A, S, D, Space, Left-Ctrl",
            15,
            30,
            10,
            Color::BLACK,
        );
        d.draw_text(
            "- Look around: arrow keys or mouse",
            15,
            45,
            10,
            Color::BLACK,
        );
        d.draw_text("- Camera mode keys: 1, 2, 3, 4", 15, 60, 10, Color::BLACK);
        d.draw_text(
            "- Zoom keys: num-plus, num-minus or mouse scroll",
            15,
            75,
            10,
            Color::BLACK,
        );
        d.draw_text("- Camera projection key: P", 15, 90, 10, Color::BLACK);

        d.draw_rectangle(600, 5, 195, 100, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(600, 5, 195, 100, Color::BLUE);

        d.draw_text("Camera status:", 610, 15, 10, Color::BLACK);
        let mode_str = match camera_mode {
            CameraMode::CAMERA_FREE => "FREE",
            CameraMode::CAMERA_FIRST_PERSON => "FIRST_PERSON",
            CameraMode::CAMERA_THIRD_PERSON => "THIRD_PERSON",
            CameraMode::CAMERA_ORBITAL => "ORBITAL",
            _ => "CUSTOM",
        };
        d.draw_text(&format!("- Mode: {mode_str}"), 610, 30, 10, Color::BLACK);
        let proj_str = match camera.projection {
            CameraProjection::CAMERA_PERSPECTIVE => "PERSPECTIVE",
            CameraProjection::CAMERA_ORTHOGRAPHIC => "ORTHOGRAPHIC",
        };
        d.draw_text(
            &format!("- Projection: {proj_str}"),
            610,
            45,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!(
                "- Position: ({:06.3}, {:06.3}, {:06.3})",
                camera.position.x, camera.position.y, camera.position.z
            ),
            610,
            60,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!(
                "- Target: ({:06.3}, {:06.3}, {:06.3})",
                camera.target.x, camera.target.y, camera.target.z
            ),
            610,
            75,
            10,
            Color::BLACK,
        );
        d.draw_text(
            &format!(
                "- Up: ({:06.3}, {:06.3}, {:06.3})",
                camera.up.x, camera.up.y, camera.up.z
            ),
            610,
            90,
            10,
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
