/*******************************************************************************************
*
*   raylib [shaders] example - raymarching rendering
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: This example requires raylib OpenGL 3.3 for shaders support and only #version 330
*         is currently supported. OpenGL ES 2.0 platforms are not supported at the moment
*
*   Example originally created with raylib 2.0, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

#[cfg(target_family = "wasm")]
const GLSL_VERSION: i32 = 100;
#[cfg(not(target_family = "wasm"))]
const GLSL_VERSION: i32 = 330;

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    // SetConfigFlags(FLAG_WINDOW_RESIZABLE)
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - raymarching rendering")
        .resizable()
        .build();

    let mut camera = Camera3D::perspective(
        Vector3::new(2.5, 2.5, 3.0), // Camera position
        Vector3::new(0.0, 0.0, 0.7), // Camera looking at point
        Vector3::new(0.0, 1.0, 0.0), // Camera up vector (rotation towards target)
        65.0,                        // Camera field-of-view Y
    );

    // Load raymarching shader
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/raymarching.fs",
            GLSL_VERSION
        )),
    );

    // Get shader locations for required uniforms
    let view_eye_loc = shader.get_shader_location("viewEye");
    let view_center_loc = shader.get_shader_location("viewCenter");
    let run_time_loc = shader.get_shader_location("runTime");
    let resolution_loc = shader.get_shader_location("resolution");

    let mut resolution = [screen_width as f32, screen_height as f32];
    shader.set_shader_value(resolution_loc, resolution);

    let mut run_time = 0.0f32;

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
        camera.update_camera(CameraMode::CAMERA_FIRST_PERSON);

        let camera_pos = camera.position;
        let camera_target = camera.target;

        let delta_time = rl.get_frame_time();
        run_time += delta_time;

        // Set shader required uniform values
        shader.set_shader_value(view_eye_loc, camera_pos);
        shader.set_shader_value(view_center_loc, camera_target);
        shader.set_shader_value(run_time_loc, run_time);

        // Check if screen is resized
        if rl.is_window_resized() {
            resolution[0] = rl.get_screen_width() as f32;
            resolution[1] = rl.get_screen_height() as f32;
            shader.set_shader_value(resolution_loc, resolution);
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // We only draw a white full-screen rectangle,
        // frame is generated in shader using raymarching
        {
            let mut s = d.begin_shader_mode(&mut shader);
            s.draw_rectangle(0, 0, screen_w, screen_h, Color::WHITE);
        }

        d.draw_text(
            "(c) Raymarching shader by Iñigo Quilez. MIT License.",
            screen_w - 280,
            screen_h - 20,
            10,
            Color::BLACK,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
