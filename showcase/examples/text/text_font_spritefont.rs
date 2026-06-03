/*******************************************************************************************
*
*   raylib [text] example - font spritefont
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: Sprite fonts should be generated following this conventions:
*
*     - Characters must be ordered starting with character 32 (Space)
*     - Every character must be contained within the same Rectangle height
*     - Every character and every line must be separated by the same distance (margin/padding)
*     - Rectangles must be defined by a MAGENTA color background
*
*   Following those constraints, a font can be provided just by an image,
*   this is quite handy to avoid additional font descriptor files (like BMFonts use)
*
*   Example originally created with raylib 1.0, last time updated with raylib 1.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::text::RaylibFont;
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
        .title("raylib [text] example - font spritefont")
        .build();

    let msg1 = "THIS IS A custom SPRITE FONT...";
    let msg2 = "...and this is ANOTHER CUSTOM font...";
    let msg3 = "...and a THIRD one! GREAT! :D";

    // NOTE: Textures/Fonts MUST be loaded after Window initialization (OpenGL context is required)
    let font1 = rl
        .load_font(&thread, "resources/text/custom_mecha.png")
        .expect("custom_mecha"); // Font loading
    let font2 = rl
        .load_font(&thread, "resources/text/custom_alagard.png")
        .expect("custom_alagard"); // Font loading
    let font3 = rl
        .load_font(&thread, "resources/text/custom_jupiter_crash.png")
        .expect("custom_jupiter_crash"); // Font loading

    let font_position1 = Vector2::new(
        screen_width as f32 / 2.0
            - font1.measure_text(msg1, font1.base_size() as f32, -3.0).x / 2.0,
        screen_height as f32 / 2.0 - font1.base_size() as f32 / 2.0 - 80.0,
    );

    let font_position2 = Vector2::new(
        screen_width as f32 / 2.0
            - font2.measure_text(msg2, font2.base_size() as f32, -2.0).x / 2.0,
        screen_height as f32 / 2.0 - font2.base_size() as f32 / 2.0 - 10.0,
    );

    let font_position3 = Vector2::new(
        screen_width as f32 / 2.0 - font3.measure_text(msg3, font3.base_size() as f32, 2.0).x / 2.0,
        screen_height as f32 / 2.0 - font3.base_size() as f32 / 2.0 + 50.0,
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
        // TODO: Update variables here...
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text_ex(
            &font1,
            msg1,
            font_position1,
            font1.base_size() as f32,
            -3.0,
            Color::WHITE,
        );
        d.draw_text_ex(
            &font2,
            msg2,
            font_position2,
            font2.base_size() as f32,
            -2.0,
            Color::WHITE,
        );
        d.draw_text_ex(
            &font3,
            msg3,
            font_position3,
            font3.base_size() as f32,
            2.0,
            Color::WHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(font1..3) handled by RAII drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
