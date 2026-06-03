/*******************************************************************************************
*
*   raylib [shaders] example - multi sample2d
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
*   Example originally created with raylib 3.5, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Ramon Santamaria (@raysan5)
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

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [shaders] example - multi sample2d")
        .build();

    // SAFETY: ffi::GenImageColor returns an owned Image; wrap into our RAII Image.
    let im_red = unsafe {
        Image::from_raw(raylib::ffi::GenImageColor(
            800,
            450,
            Color::new(255, 0, 0, 255).into(),
        ))
    };
    let tex_red = rl.load_texture_from_image(&thread, &im_red).unwrap();
    drop(im_red);

    // SAFETY: ffi::GenImageColor returns an owned Image; wrap into our RAII Image.
    let im_blue = unsafe {
        Image::from_raw(raylib::ffi::GenImageColor(
            800,
            450,
            Color::new(0, 0, 255, 255).into(),
        ))
    };
    let tex_blue = rl.load_texture_from_image(&thread, &im_blue).unwrap();
    drop(im_blue);

    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/color_mix.fs",
            GLSL_VERSION
        )),
    );

    // Get an additional sampler2D location to be enabled on drawing
    let tex_blue_loc = shader.get_shader_location("texture1");

    // Get shader uniform for divider
    let divider_loc = shader.get_shader_location("divider");
    let mut divider_value = 0.5f32;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            divider_value += 0.01;
        } else if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            divider_value -= 0.01;
        }

        if divider_value < 0.0 {
            divider_value = 0.0;
        } else if divider_value > 1.0 {
            divider_value = 1.0;
        }

        shader.set_shader_value(divider_loc, divider_value);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // WARNING: Additional textures (sampler2D) are enabled for ALL draw calls in the batch,
        // but EndShaderMode() forces batch drawing and resets active textures, this way
        // other textures (sampler2D) can be activated on consequent drawings (if required)
        // The downside of this approach is that SetShaderValue() must be called inside the loop,
        // to be set again after every EndShaderMode() reset
        shader.set_shader_value_texture(tex_blue_loc, &tex_blue);
        {
            let mut s = d.begin_shader_mode(&mut shader);

            // We are drawing texRed using default [sampler2D texture0] but
            // an additional texture units is enabled for texBlue [sampler2D texture1]
            s.draw_texture(&tex_red, 0, 0, Color::WHITE);
        } // Texture sampler2D is reseted, needs to be set again for next frame

        d.draw_text(
            "Use KEY_LEFT/KEY_RIGHT to move texture mixing in shader!",
            80,
            screen_h - 40,
            20,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
