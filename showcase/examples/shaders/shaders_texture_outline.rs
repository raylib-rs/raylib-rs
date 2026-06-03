/*******************************************************************************************
*
*   raylib [shaders] example - texture outline
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   Example originally created with raylib 4.0, last time updated with raylib 4.0
*
*   Example contributed by Serenity Skiff (@GoldenThumbs) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Serenity Skiff (@GoldenThumbs) and Ramon Santamaria (@raysan5)
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
        .title("raylib [shaders] example - texture outline")
        .build();

    let texture = rl
        .load_texture(&thread, "resources/shaders/fudesumi.png")
        .unwrap();

    let mut shdr_outline = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/outline.fs",
            GLSL_VERSION
        )),
    );

    let mut outline_size = 2.0f32;
    let outline_color = [1.0f32, 0.0f32, 0.0f32, 1.0f32]; // Normalized RED color
    let texture_size = [texture.width() as f32, texture.height() as f32];

    // Get shader locations
    let outline_size_loc = shdr_outline.get_shader_location("outlineSize");
    let outline_color_loc = shdr_outline.get_shader_location("outlineColor");
    let texture_size_loc = shdr_outline.get_shader_location("textureSize");

    // Set shader values (they can be changed later)
    shdr_outline.set_shader_value(outline_size_loc, outline_size);
    shdr_outline.set_shader_value(outline_color_loc, outline_color);
    shdr_outline.set_shader_value(texture_size_loc, texture_size);

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        outline_size += rl.get_mouse_wheel_move();
        if outline_size < 1.0 {
            outline_size = 1.0;
        }

        shdr_outline.set_shader_value(outline_size_loc, outline_size);
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let tex_w = texture.width();
        let screen_w = rl.get_screen_width();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut s = d.begin_shader_mode(&mut shdr_outline);

            s.draw_texture(&texture, screen_w / 2 - tex_w / 2, -30, Color::WHITE);
        }

        d.draw_text("Shader-based\ntexture\noutline", 10, 10, 20, Color::GRAY);
        d.draw_text(
            "Scroll mouse wheel to\nchange outline size",
            10,
            72,
            20,
            Color::GRAY,
        );
        d.draw_text(
            &format!("Outline size: {} px", outline_size as i32),
            10,
            120,
            20,
            Color::MAROON,
        );

        d.draw_fps(710, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture / UnloadShader / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
