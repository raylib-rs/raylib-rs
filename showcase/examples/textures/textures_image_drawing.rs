/*******************************************************************************************
*
*   raylib [textures] example - image drawing
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: Images are loaded in CPU memory (RAM); textures are loaded in GPU memory (VRAM)
*
*   Example originally created with raylib 1.4, last time updated with raylib 1.4
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2016-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [textures] example - image drawing")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)

    let mut cat = Image::load_image("resources/textures/cat.png").unwrap(); // Load image in CPU memory (RAM)
    cat.crop(Rectangle::new(100.0, 10.0, 280.0, 380.0)); // Crop an image piece
    cat.flip_horizontal(); // Flip cropped image horizontally
    cat.resize(150, 200); // Resize flipped-cropped image

    let mut parrots = Image::load_image("resources/textures/parrots.png").unwrap(); // Load image in CPU memory (RAM)

    // Draw one image over the other with a scaling of 1.5f
    parrots.draw(
        &cat,
        Rectangle::new(0.0, 0.0, cat.width() as f32, cat.height() as f32),
        Rectangle::new(
            30.0,
            40.0,
            cat.width() as f32 * 1.5,
            cat.height() as f32 * 1.5,
        ),
        Color::WHITE,
    );
    parrots.crop(Rectangle::new(
        0.0,
        50.0,
        parrots.width() as f32,
        parrots.height() as f32 - 100.0,
    )); // Crop resulting image

    // Draw on the image with a few image draw methods
    parrots.draw_pixel(10, 10, Color::RAYWHITE);
    parrots.draw_circle_lines(10, 10, 5, Color::RAYWHITE);
    parrots.draw_rectangle(5, 20, 10, 10, Color::RAYWHITE);

    drop(cat); // Unload image from RAM

    // Load custom font for drawing on image
    let font = rl
        .load_font(&thread, "resources/textures/custom_jupiter_crash.png")
        .unwrap();

    // Draw over image using custom font
    parrots.draw_text_ex(
        &font,
        "PARROTS & CAT",
        Vector2::new(300.0, 230.0),
        font.base_size() as f32,
        -2.0,
        Color::WHITE,
    );

    drop(font); // Unload custom font (already drawn used on image)

    let texture = rl.load_texture_from_image(&thread, &parrots).unwrap(); // Image converted to texture, uploaded to GPU memory (VRAM)
    drop(parrots); // Once image has been converted to texture and uploaded to VRAM, it can be unloaded from RAM

    rl.set_target_fps(60);
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
            screen_height / 2 - texture.height() / 2 - 40,
            Color::WHITE,
        );
        d.draw_rectangle_lines(
            screen_width / 2 - texture.width() / 2,
            screen_height / 2 - texture.height() / 2 - 40,
            texture.width(),
            texture.height(),
            Color::DARKGRAY,
        );

        d.draw_text(
            "We are drawing only one texture from various images composed!",
            240,
            350,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Source images have been cropped, scaled, flipped and copied one over the other.",
            190,
            370,
            10,
            Color::DARKGRAY,
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
