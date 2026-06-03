/*******************************************************************************************
*
*   raylib [textures] example - background scrolling
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 2.0, last time updated with raylib 2.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [textures] example - background scrolling")
        .build();

    // NOTE: Be careful, background width must be equal or bigger than screen width
    // if not, texture should be draw more than two times for scrolling effect
    let background = rl
        .load_texture(
            &thread,
            "resources/textures/cyberpunk_street_background.png",
        )
        .unwrap();
    let midground = rl
        .load_texture(&thread, "resources/textures/cyberpunk_street_midground.png")
        .unwrap();
    let foreground = rl
        .load_texture(
            &thread,
            "resources/textures/cyberpunk_street_foreground.png",
        )
        .unwrap();

    let mut scrolling_back: f32 = 0.0;
    let mut scrolling_mid: f32 = 0.0;
    let mut scrolling_fore: f32 = 0.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        scrolling_back -= 0.1;
        scrolling_mid -= 0.5;
        scrolling_fore -= 1.0;

        // NOTE: Texture is scaled twice its size, so it sould be considered on scrolling
        if scrolling_back <= -(background.width() as f32 * 2.0) {
            scrolling_back = 0.0;
        }
        if scrolling_mid <= -(midground.width() as f32 * 2.0) {
            scrolling_mid = 0.0;
        }
        if scrolling_fore <= -(foreground.width() as f32 * 2.0) {
            scrolling_fore = 0.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::get_color(0x052c46ff));

        // Draw background image twice
        // NOTE: Texture is scaled twice its size
        d.draw_texture_ex(
            &background,
            Vector2::new(scrolling_back, 20.0),
            0.0,
            2.0,
            Color::WHITE,
        );
        d.draw_texture_ex(
            &background,
            Vector2::new(background.width() as f32 * 2.0 + scrolling_back, 20.0),
            0.0,
            2.0,
            Color::WHITE,
        );

        // Draw midground image twice
        d.draw_texture_ex(
            &midground,
            Vector2::new(scrolling_mid, 20.0),
            0.0,
            2.0,
            Color::WHITE,
        );
        d.draw_texture_ex(
            &midground,
            Vector2::new(midground.width() as f32 * 2.0 + scrolling_mid, 20.0),
            0.0,
            2.0,
            Color::WHITE,
        );

        // Draw foreground image twice
        d.draw_texture_ex(
            &foreground,
            Vector2::new(scrolling_fore, 70.0),
            0.0,
            2.0,
            Color::WHITE,
        );
        d.draw_texture_ex(
            &foreground,
            Vector2::new(foreground.width() as f32 * 2.0 + scrolling_fore, 70.0),
            0.0,
            2.0,
            Color::WHITE,
        );

        d.draw_text("BACKGROUND SCROLLING & PARALLAX", 10, 10, 20, Color::RED);
        d.draw_text(
            "(c) Cyberpunk Street Environment by Luis Zuno (@ansimuz)",
            screen_width - 330,
            screen_height - 20,
            10,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `background`, `midground`, `foreground`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
