/*******************************************************************************************
*
*   raylib [shaders] example - eratosthenes sieve
*
*   Example complexity rating: [★★★☆] 3/4
*
*   NOTE: Sieve of Eratosthenes, the earliest known (ancient Greek) prime number sieve
*
*       "Sift the twos and sift the threes,
*        The Sieve of Eratosthenes.
*        When the multiples sublime,
*        the numbers that are left are prime."
*
*   NOTE: This example requires raylib OpenGL 3.3 or ES2 versions for shaders support,
*         OpenGL 1.1 does not support shaders, recompile raylib to OpenGL 3.3 version
*
*   NOTE: Shaders used in this example are #version 330 (OpenGL 3.3)
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example contributed by ProfJski (@ProfJski) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 ProfJski (@ProfJski) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::texture::RaylibTexture2D;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// We always run on PLATFORM_DESKTOP via raylib-rs; mirror the GLSL_VERSION fork's desktop value.
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
        .title("raylib [shaders] example - eratosthenes sieve")
        .build();

    let mut target = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .unwrap();

    // Load Eratosthenes shader
    // NOTE: Defining 0 (NULL) for vertex shader forces usage of internal default vertex shader
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/eratosthenes.fs",
            GLSL_VERSION
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
        // Nothing to do here, everything is happening in the shader
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let target_tex_w = target.texture().width();
        let target_tex_h = target.texture().height();
        let mut d = rl.begin_drawing(&thread);

        {
            let mut t = d.begin_texture_mode(&thread, &mut target); // Enable drawing to texture
            t.clear_background(Color::BLACK); // Clear the render texture

            // Draw a rectangle in shader mode to be used as shader canvas
            // NOTE: Rectangle uses font white character texture coordinates,
            // so shader can not be applied here directly because input vertexTexCoord
            // do not represent full screen coordinates (space where want to apply shader)
            t.draw_rectangle(0, 0, screen_w, screen_h, Color::BLACK);
        } // End drawing to texture (now we have a blank texture available for the shader)

        d.clear_background(Color::RAYWHITE); // Clear screen background

        {
            let mut s = d.begin_shader_mode(&mut shader);
            // NOTE: Render texture must be y-flipped due to default OpenGL coordinates (left-bottom)
            s.draw_texture_rec(
                target.texture(),
                Rectangle::new(0.0, 0.0, target_tex_w as f32, -(target_tex_h as f32)),
                Vector2::new(0.0, 0.0),
                Color::WHITE,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadShader / UnloadRenderTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
