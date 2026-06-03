/*******************************************************************************************
*
*   raylib [text] example - font loading
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: raylib can load fonts from multiple input file formats:
*
*     - TTF/OTF > Sprite font atlas is generated on loading, user can configure
*                 some of the generation parameters (size, characters to include)
*     - BMFonts > Angel code font fileformat, sprite font image must be provided
*                 together with the .fnt file, font generation can not be configured
*     - XNA Spritefont > Sprite font image, following XNA Spritefont conventions,
*                 Characters in image must follow some spacing and order rules
*
*   Example originally created with raylib 1.4, last time updated with raylib 3.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [text] example - font loading")
        .build();

    // Define characters to draw
    // NOTE: raylib supports UTF-8 encoding, following list is actually codified as UTF8 internally
    let msg = "!#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHI\nJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmn\nopqrstuvwxyz{|}~¿ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏÐÑÒÓ\nÔÕÖ×ØÙÚÛÜÝÞßàáâãäåæçèéêëìíîïðñòóôõö÷\nøùúûüýþÿ";

    // NOTE: Textures/Fonts MUST be loaded after Window initialization (OpenGL context is required)

    // BMFont (AngelCode) : Font data and image atlas have been generated using external program
    let font_bm = rl
        .load_font(&thread, "resources/text/pixantiqua.fnt")
        .expect("font load"); // Requires "resources/pixantiqua.png"

    // TTF font : Font data and atlas are generated directly from TTF
    // NOTE: We define a font base size of 32 pixels tall and up-to 250 characters
    let font_ttf = rl
        .load_font_ex(&thread, "resources/text/pixantiqua.ttf", 32, None)
        .expect("ttf font load");

    rl.set_text_line_spacing(16); // Set line spacing for multiline text (when line breaks are included '\n')

    #[allow(unused_assignments)]
    let mut use_ttf = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            use_ttf = true;
        } else {
            use_ttf = false;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Hold SPACE to use TTF generated font",
            20,
            20,
            20,
            Color::LIGHTGRAY,
        );

        if !use_ttf {
            d.draw_text_ex(
                &font_bm,
                msg,
                Vector2::new(20.0, 100.0),
                font_bm.base_size() as f32,
                2.0,
                Color::MAROON,
            );
            d.draw_text(
                "Using BMFont (Angelcode) imported",
                20,
                screen_h - 30,
                20,
                Color::GRAY,
            );
        } else {
            d.draw_text_ex(
                &font_ttf,
                msg,
                Vector2::new(20.0, 100.0),
                font_ttf.base_size() as f32,
                2.0,
                Color::LIME,
            );
            d.draw_text(
                "Using TTF font generated",
                20,
                screen_h - 30,
                20,
                Color::GRAY,
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(fontBm); UnloadFont(fontTtf) — RAII drop handles both.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
