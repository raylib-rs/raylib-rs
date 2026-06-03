/*******************************************************************************************
*
*   raylib [core] example - world screen
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 1.4
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
        .title("raylib [core] example - world screen")
        .build();

    // Define the camera to look into our 3d world
    let mut camera = Camera3D::perspective(
        Vector3::new(10.0, 10.0, 10.0), // Camera position
        Vector3::new(0.0, 0.0, 0.0),    // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0),    // Camera up vector (rotation towards target)
        45.0,                           // Camera field-of-view Y
    );

    let cube_position = Vector3::new(0.0, 0.0, 0.0);
    let mut cube_screen_position = Vector2::new(0.0, 0.0);

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
        camera.update_camera(CameraMode::CAMERA_THIRD_PERSON);

        // Calculate cube screen space position (with a little offset to be in top)
        cube_screen_position = rl.get_world_to_screen(
            Vector3::new(cube_position.x, cube_position.y + 2.5, cube_position.z),
            camera,
        );
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut m = d.begin_mode3D(camera);

            m.draw_cube(cube_position, 2.0, 2.0, 2.0, Color::RED);
            m.draw_cube_wires(cube_position, 2.0, 2.0, 2.0, Color::MAROON);

            m.draw_grid(10, 1.0);
        }

        d.draw_text(
            "Enemy: 100/100",
            cube_screen_position.x as i32 - measure_text("Enemy: 100/100", 20) / 2,
            cube_screen_position.y as i32,
            20,
            Color::BLACK,
        );

        d.draw_text(
            &format!(
                "Cube position in screen space coordinates: [{}, {}]",
                cube_screen_position.x as i32, cube_screen_position.y as i32
            ),
            10,
            10,
            20,
            Color::LIME,
        );
        d.draw_text(
            "Text 2d should be always on top of the cube",
            10,
            40,
            20,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

fn measure_text(text: &str, font_size: i32) -> i32 {
    let c = std::ffi::CString::new(text).unwrap();
    // SAFETY: pure raylib FFI taking a borrowed C-string pointer and a primitive,
    // returning a primitive; the CString outlives the call so its pointer is valid.
    unsafe { raylib::ffi::MeasureText(c.as_ptr(), font_size) }
}
