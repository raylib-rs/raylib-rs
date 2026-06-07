/*******************************************************************************************
*
*   raylib [shaders] example - texture waves
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
*   Example originally created with raylib 2.5, last time updated with raylib 3.7
*
*   Example contributed by Anata (@anatagawa) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Anata (@anatagawa) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::shaders::RaylibShader;
use raylib::core::texture::RaylibTexture2D;
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
        .title("raylib [shaders] example - texture waves")
        .build();

    // Load texture texture to apply shaders
    let texture = rl
        .load_texture(&thread, "resources/shaders/space.png")
        .unwrap();

    // Load shader and setup location points and values
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{GLSL_VERSION}/wave.fs"
        )),
    );

    let seconds_loc = shader.get_shader_location("seconds");
    let freq_x_loc = shader.get_shader_location("freqX");
    let freq_y_loc = shader.get_shader_location("freqY");
    let amp_x_loc = shader.get_shader_location("ampX");
    let amp_y_loc = shader.get_shader_location("ampY");
    let speed_x_loc = shader.get_shader_location("speedX");
    let speed_y_loc = shader.get_shader_location("speedY");

    // Shader uniform values that can be updated at any time
    let freq_x = 25.0f32;
    let freq_y = 25.0f32;
    let amp_x = 5.0f32;
    let amp_y = 5.0f32;
    let speed_x = 8.0f32;
    let speed_y = 8.0f32;

    let screen_size = [rl.get_screen_width() as f32, rl.get_screen_height() as f32];
    let size_loc = shader.get_shader_location("size");
    shader.set_shader_value(size_loc, screen_size);
    shader.set_shader_value(freq_x_loc, freq_x);
    shader.set_shader_value(freq_y_loc, freq_y);
    shader.set_shader_value(amp_x_loc, amp_x);
    shader.set_shader_value(amp_y_loc, amp_y);
    shader.set_shader_value(speed_x_loc, speed_x);
    shader.set_shader_value(speed_y_loc, speed_y);

    let mut seconds = 0.0f32;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    // -------------------------------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        seconds += rl.get_frame_time();

        shader.set_shader_value(seconds_loc, seconds);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let tex_w = texture.width();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut s = d.begin_shader_mode(&mut shader);

            s.draw_texture(&texture, 0, 0, Color::WHITE);
            s.draw_texture(&texture, tex_w, 0, Color::WHITE);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
