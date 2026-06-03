/*******************************************************************************************
*
*   raylib [shaders] example - ascii rendering
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
*
*   Example contributed by Maicon Santana (@maiconpintoabreu) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Maicon Santana (@maiconpintoabreu)
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
        .title("raylib [shaders] example - ascii rendering")
        .build();

    // Texture to test static drawing
    let fudesumi = rl
        .load_texture(&thread, "resources/shaders/fudesumi.png")
        .unwrap();
    // Texture to test moving drawing
    let raysan = rl
        .load_texture(&thread, "resources/shaders/raysan.png")
        .unwrap();

    // Load shader to be used on postprocessing
    let mut shader = rl.load_shader(
        &thread,
        None,
        Some(&format!(
            "resources/shaders/shaders/glsl{}/ascii.fs",
            GLSL_VERSION
        )),
    );

    // These locations are used to send data to the GPU
    let resolution_loc = shader.get_shader_location("resolution");
    let font_size_loc = shader.get_shader_location("fontSize");

    // Set the character size for the ASCII effect
    // Fontsize should be 9 or more
    let mut font_size: f32 = 9.0;

    // Send the updated values to the shader
    let resolution = Vector2::new(screen_width as f32, screen_height as f32);
    shader.set_shader_value(resolution_loc, resolution);

    let mut circle_pos = Vector2::new(40.0, screen_height as f32 * 0.5);
    let mut circle_speed: f32 = 1.0;

    // RenderTexture to apply the postprocessing later
    let mut target = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .unwrap();

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        circle_pos.x += circle_speed;
        if circle_pos.x > 200.0 || circle_pos.x < 40.0 {
            circle_speed *= -1.0; // Revert speed
        }

        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) && font_size > 9.0 {
            font_size -= 1.0; // Reduce fontSize
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) && font_size < 15.0 {
            font_size += 1.0; // Increase fontSize
        }

        // Set fontsize for the shader
        shader.set_shader_value(font_size_loc, font_size);

        viewer.update(&mut rl, &thread);

        // Draw
        //----------------------------------------------------------------------------------
        let target_tex_w = target.texture().width();
        let target_tex_h = target.texture().height();
        let mut d = rl.begin_drawing(&thread);

        {
            let mut t = d.begin_texture_mode(&thread, &mut target);
            t.clear_background(Color::WHITE);

            // Draw scene in our render texture
            t.draw_texture(&fudesumi, 500, -30, Color::WHITE);
            t.draw_texture_v(&raysan, circle_pos, Color::WHITE);
        }

        d.clear_background(Color::RAYWHITE);

        {
            let mut s = d.begin_shader_mode(&mut shader);
            // Draw the scene texture (that we rendered earlier) to the screen
            // The shader will process every pixel of this texture
            s.draw_texture_rec(
                target.texture(),
                Rectangle::new(0.0, 0.0, target_tex_w as f32, -(target_tex_h as f32)),
                Vector2::new(0.0, 0.0),
                Color::WHITE,
            );
        }

        d.draw_rectangle(0, 0, screen_width, 40, Color::BLACK);
        d.draw_text(
            &format!(
                "Ascii effect - FontSize:{:2.0} - [Left] -1 [Right] +1 ",
                font_size
            ),
            120,
            10,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_fps(10, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture / UnloadShader / UnloadTexture / CloseWindow handled by RAII drops.
    //--------------------------------------------------------------------------------------
}
