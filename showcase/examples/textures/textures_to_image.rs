/*******************************************************************************************
*
*   raylib [textures] example - to image
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [textures] example - to image")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)

    let image = Image::load_image("resources/textures/raylib_logo.png").unwrap(); // Load image data into CPU memory (RAM)
    let texture = rl.load_texture_from_image(&thread, &image).unwrap(); // Image converted to texture, GPU memory (RAM -> VRAM)
    drop(image); // Unload image data from CPU memory (RAM)

    let image = texture.load_image().unwrap(); // Load image from GPU texture (VRAM -> RAM)
    drop(texture); // Unload texture from GPU memory (VRAM)

    let texture = rl.load_texture_from_image(&thread, &image).unwrap(); // Recreate texture from retrieved image data (RAM -> VRAM)
    drop(image); // Unload retrieved image data from CPU memory (RAM)

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //---------------------------------------------------------------------------------------

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

        d.draw_texture(
            &texture,
            screen_width / 2 - texture.width() / 2,
            screen_height / 2 - texture.height() / 2,
            Color::WHITE,
        );

        d.draw_text(
            "this IS a texture loaded from an image!",
            300,
            370,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `texture`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
