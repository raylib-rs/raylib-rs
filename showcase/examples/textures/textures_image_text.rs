/*******************************************************************************************
*
*   raylib [textures] example - image text
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.8, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [textures] example - image text")
        .build();

    let mut parrots = Image::load_image("resources/textures/parrots.png").unwrap(); // Load image in CPU memory (RAM)

    // TTF Font loading with custom generation parameters
    let font = rl
        .load_font_ex(&thread, "resources/textures/KAISG.ttf", 64, None)
        .unwrap();

    // Draw over image using custom font
    parrots.draw_text_ex(
        &font,
        "[Parrots font drawing]",
        Vector2::new(20.0, 20.0),
        font.base_size() as f32,
        0.0,
        Color::RED,
    );

    let texture = rl.load_texture_from_image(&thread, &parrots).unwrap(); // Image converted to texture, uploaded to GPU memory (VRAM)
    drop(parrots); // Once image has been converted to texture and uploaded to VRAM, it can be unloaded from RAM

    let position = Vector2::new(
        screen_width as f32 / 2.0 - texture.width() as f32 / 2.0,
        screen_height as f32 / 2.0 - texture.height() as f32 / 2.0 - 20.0,
    );

    let mut show_font;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        #[expect(
            clippy::needless_bool_assign,
            reason = "C-parity: C assigns the bool in if/else"
        )]
        if rl.is_key_down(KeyboardKey::KEY_SPACE) {
            show_font = true;
        } else {
            show_font = false;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if !show_font {
            // Draw texture with text already drawn inside
            d.draw_texture_v(&texture, position, Color::WHITE);

            // Draw text directly using sprite font
            d.draw_text_ex(
                &font,
                "[Parrots font drawing]",
                Vector2::new(position.x + 20.0, position.y + 20.0 + 280.0),
                font.base_size() as f32,
                0.0,
                Color::WHITE,
            );
        } else {
            d.draw_texture(
                font.texture(),
                screen_width / 2 - font.texture().width() / 2,
                50,
                Color::BLACK,
            );
        }

        d.draw_text(
            "PRESS SPACE to SHOW FONT ATLAS USED",
            290,
            420,
            10,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `texture`.
    // UnloadFont is handled by RAII drop of `font`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
