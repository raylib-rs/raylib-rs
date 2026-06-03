/*******************************************************************************************
*
*   raylib [textures] example - image rotate
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 1.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const NUM_TEXTURES: usize = 3;

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
        .title("raylib [textures] example - image rotate")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)
    let mut image45 = Image::load_image("resources/textures/raylib_logo.png").unwrap();
    let mut image90 = Image::load_image("resources/textures/raylib_logo.png").unwrap();
    let mut image_neg90 = Image::load_image("resources/textures/raylib_logo.png").unwrap();

    image45.rotate(45);
    image90.rotate(90);
    image_neg90.rotate(-90);

    let textures: [Texture2D; NUM_TEXTURES] = [
        rl.load_texture_from_image(&thread, &image45).unwrap(),
        rl.load_texture_from_image(&thread, &image90).unwrap(),
        rl.load_texture_from_image(&thread, &image_neg90).unwrap(),
    ];

    let mut current_texture: usize = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            || rl.is_key_pressed(KeyboardKey::KEY_RIGHT)
        {
            current_texture = (current_texture + 1) % NUM_TEXTURES; // Cycle between the textures
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_texture(
            &textures[current_texture],
            screen_width / 2 - textures[current_texture].width() / 2,
            screen_height / 2 - textures[current_texture].height() / 2,
            Color::WHITE,
        );

        d.draw_text(
            "Press LEFT MOUSE BUTTON to rotate the image clockwise",
            250,
            420,
            10,
            Color::DARKGRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of textures array.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
