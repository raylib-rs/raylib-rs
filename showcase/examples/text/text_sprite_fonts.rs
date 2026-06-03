/*******************************************************************************************
*
*   raylib [text] example - sprite fonts
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: raylib is distributed with some free to use fonts (even for commercial pourposes!)
*         To view details and credits for those fonts, check raylib license file
*
*   Example originally created with raylib 1.7, last time updated with raylib 3.7
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_FONTS: usize = 8;

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
        .title("raylib [text] example - sprite fonts")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)
    // idiomatic: C uses a fixed-size array of Font = {0}; we collect into a Vec to keep RAII
    let font_paths = [
        "resources/text/sprite_fonts/alagard.png",
        "resources/text/sprite_fonts/pixelplay.png",
        "resources/text/sprite_fonts/mecha.png",
        "resources/text/sprite_fonts/setback.png",
        "resources/text/sprite_fonts/romulus.png",
        "resources/text/sprite_fonts/pixantiqua.png",
        "resources/text/sprite_fonts/alpha_beta.png",
        "resources/text/sprite_fonts/jupiter_crash.png",
    ];

    let mut fonts: Vec<Font> = Vec::with_capacity(MAX_FONTS);
    for path in font_paths.iter() {
        fonts.push(rl.load_font(&thread, path).expect("font load"));
    }

    let messages: [&str; MAX_FONTS] = [
        "ALAGARD FONT designed by Hewett Tsoi",
        "PIXELPLAY FONT designed by Aleksander Shevchuk",
        "MECHA FONT designed by Captain Falcon",
        "SETBACK FONT designed by Brian Kent (AEnigma)",
        "ROMULUS FONT designed by Hewett Tsoi",
        "PIXANTIQUA FONT designed by Gerhard Grossmann",
        "ALPHA_BETA FONT designed by Brian Kent (AEnigma)",
        "JUPITER_CRASH FONT designed by Brian Kent (AEnigma)",
    ];

    let spacings: [i32; MAX_FONTS] = [2, 4, 8, 4, 3, 4, 4, 1];

    let mut positions: [Vector2; MAX_FONTS] = [Vector2::new(0.0, 0.0); MAX_FONTS];

    for i in 0..MAX_FONTS {
        positions[i].x = screen_width as f32 / 2.0
            - fonts[i]
                .measure_text(
                    messages[i],
                    fonts[i].base_size() as f32 * 2.0,
                    spacings[i] as f32,
                )
                .x
                / 2.0;
        positions[i].y = 60.0 + fonts[i].base_size() as f32 + 45.0 * i as f32;
    }

    // Small Y position corrections
    positions[3].y += 8.0;
    positions[4].y += 2.0;
    positions[7].y -= 8.0;

    let colors: [Color; MAX_FONTS] = [
        Color::MAROON,
        Color::ORANGE,
        Color::DARKGREEN,
        Color::DARKBLUE,
        Color::DARKPURPLE,
        Color::LIME,
        Color::GOLD,
        Color::RED,
    ];

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

        d.draw_text(
            "free sprite fonts included with raylib",
            220,
            20,
            20,
            Color::DARKGRAY,
        );
        d.draw_line(220, 50, 600, 50, Color::DARKGRAY);

        for i in 0..MAX_FONTS {
            d.draw_text_ex(
                &fonts[i],
                messages[i],
                positions[i],
                fonts[i].base_size() as f32 * 2.0,
                spacings[i] as f32,
                colors[i],
            );
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // Fonts unloading handled by RAII drop of `fonts: Vec<Font>`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
