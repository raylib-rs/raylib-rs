/*******************************************************************************************
*
*   raylib [textures] example - blend modes
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example originally created with raylib 3.5, last time updated with raylib 3.5
*
*   Example contributed by Karlo Licudine (@accidentalrebel) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2020-2025 Karlo Licudine (@accidentalrebel)
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
        .title("raylib [textures] example - blend modes")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)
    let bg_image = Image::load_image("resources/textures/cyberpunk_street_background.png").unwrap(); // Loaded in CPU memory (RAM)
    let bg_texture = rl.load_texture_from_image(&thread, &bg_image).unwrap(); // Image converted to texture, GPU memory (VRAM)

    let fg_image = Image::load_image("resources/textures/cyberpunk_street_foreground.png").unwrap(); // Loaded in CPU memory (RAM)
    let fg_texture = rl.load_texture_from_image(&thread, &fg_image).unwrap(); // Image converted to texture, GPU memory (VRAM)

    // Once image has been converted to texture and uploaded to VRAM, it can be unloaded from RAM
    drop(bg_image);
    drop(fg_image);

    let blend_count_max: i32 = 4;
    let mut blend_mode: i32 = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            if blend_mode >= (blend_count_max - 1) {
                blend_mode = 0;
            } else {
                blend_mode += 1;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture(
            &bg_texture,
            screen_width / 2 - bg_texture.width() / 2,
            screen_height / 2 - bg_texture.height() / 2,
            Color::WHITE,
        );

        // Apply the blend mode and then draw the foreground texture
        {
            // SAFETY: blend_mode is cycled in [0, blend_count_max) which matches valid
            // raylib BlendMode discriminants 0..=3 (ALPHA, ADDITIVE, MULTIPLIED, ADD_COLORS).
            let bm: BlendMode = unsafe { std::mem::transmute::<i32, BlendMode>(blend_mode) };
            let mut b = d.begin_blend_mode(bm);
            b.draw_texture(
                &fg_texture,
                screen_width / 2 - fg_texture.width() / 2,
                screen_height / 2 - fg_texture.height() / 2,
                Color::WHITE,
            );
        }

        // Draw the texts
        d.draw_text(
            "Press SPACE to change blend modes.",
            310,
            350,
            10,
            Color::GRAY,
        );

        match blend_mode {
            0 => d.draw_text(
                "Current: BLEND_ALPHA",
                (screen_width / 2) - 60,
                370,
                10,
                Color::GRAY,
            ),
            1 => d.draw_text(
                "Current: BLEND_ADDITIVE",
                (screen_width / 2) - 60,
                370,
                10,
                Color::GRAY,
            ),
            2 => d.draw_text(
                "Current: BLEND_MULTIPLIED",
                (screen_width / 2) - 60,
                370,
                10,
                Color::GRAY,
            ),
            3 => d.draw_text(
                "Current: BLEND_ADD_COLORS",
                (screen_width / 2) - 60,
                370,
                10,
                Color::GRAY,
            ),
            _ => {}
        }

        d.draw_text(
            "(c) Cyberpunk Street Environment by Luis Zuno (@ansimuz)",
            screen_width - 330,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `fg_texture` and `bg_texture`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
