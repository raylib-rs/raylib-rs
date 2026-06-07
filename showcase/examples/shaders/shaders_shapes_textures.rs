/*******************************************************************************************
*
*   raylib [shaders] example - shapes textures
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3), to test this example
*         on OpenGL ES 2.0 platforms (Android, Raspberry Pi, HTML5), use #version 100 shaders
*         raylib comes with shaders ready for both versions, check raylib/shaders install folder
*
*   Example originally created with raylib 1.7, last time updated with raylib 3.7
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

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

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - shapes textures")
        .build();

    let fudesumi = rl
        .load_texture(&thread, "resources/shaders/fudesumi.png")
        .unwrap();

    // Load shader to be used on some parts drawing
    // NOTE 1: Using GLSL 330 shader version, on OpenGL ES 2.0 use GLSL 100 shader version
    // NOTE 2: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/grayscale.fs"
        )),
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

        // Start drawing with default shader

        d.draw_text("USING DEFAULT SHADER", 20, 40, 10, Color::RED);

        d.draw_circle(80, 120, 35.0, Color::DARKBLUE);
        d.draw_circle_gradient(80, 220, 60.0, Color::GREEN, Color::SKYBLUE);
        d.draw_circle_lines(80, 340, 80.0, Color::DARKBLUE);

        // Activate our custom shader to be applied on next shapes/textures drawings
        {
            let mut s = d.begin_shader_mode(&mut shader);

            s.draw_text("USING CUSTOM SHADER", 190, 40, 10, Color::RED);

            s.draw_rectangle(250 - 60, 90, 120, 60, Color::RED);
            s.draw_rectangle_gradient_h(250 - 90, 170, 180, 130, Color::MAROON, Color::GOLD);
            s.draw_rectangle_lines(250 - 40, 320, 80, 60, Color::ORANGE);

            // Activate our default shader for next drawings
        }

        d.draw_text("USING DEFAULT SHADER", 370, 40, 10, Color::RED);

        d.draw_triangle(
            Vector2::new(430.0, 80.0),
            Vector2::new(430.0 - 60.0, 150.0),
            Vector2::new(430.0 + 60.0, 150.0),
            Color::VIOLET,
        );

        d.draw_triangle_lines(
            Vector2::new(430.0, 160.0),
            Vector2::new(430.0 - 20.0, 230.0),
            Vector2::new(430.0 + 20.0, 230.0),
            Color::DARKBLUE,
        );

        d.draw_poly(Vector2::new(430.0, 320.0), 6, 80.0, 0.0, Color::BROWN);

        // Activate our custom shader to be applied on next shapes/textures drawings
        {
            let mut s = d.begin_shader_mode(&mut shader);

            s.draw_texture(&fudesumi, 500, -30, Color::WHITE); // Using custom shader

            // Activate our default shader for next drawings
        }

        d.draw_text(
            "(c) Fudesumi sprite by Eiden Marsal",
            380,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
