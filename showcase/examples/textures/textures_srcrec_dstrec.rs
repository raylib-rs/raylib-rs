/*******************************************************************************************
*
*   raylib [textures] example - srcrec dstrec
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 1.3, last time updated with raylib 1.3
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
        .title("raylib [textures] example - srcrec dstrec")
        .build();

    // NOTE: Textures MUST be loaded after Window initialization (OpenGL context is required)

    let scarfy = rl
        .load_texture(&thread, "resources/textures/scarfy.png")
        .unwrap(); // Texture loading

    let frame_width = scarfy.width() / 6;
    let frame_height = scarfy.height();

    // Source rectangle (part of the texture to use for drawing)
    let source_rec = Rectangle::new(0.0, 0.0, frame_width as f32, frame_height as f32);

    // Destination rectangle (screen rectangle where drawing part of texture)
    let dest_rec = Rectangle::new(
        screen_width as f32 / 2.0,
        screen_height as f32 / 2.0,
        frame_width as f32 * 2.0,
        frame_height as f32 * 2.0,
    );

    // Origin of the texture (rotation/scale point), it's relative to destination rectangle size
    let origin = Vector2::new(frame_width as f32, frame_height as f32);

    let mut rotation: i32 = 0;

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        rotation += 1;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // NOTE: Using DrawTexturePro() we can easily rotate and scale the part of the texture we draw
        // sourceRec defines the part of the texture we use for drawing
        // destRec defines the rectangle where our texture part will fit (scaling it to fit)
        // origin defines the point of the texture used as reference for rotation and scaling
        // rotation defines the texture rotation (using origin as rotation point)
        d.draw_texture_pro(
            &scarfy,
            source_rec,
            dest_rec,
            origin,
            rotation as f32,
            Color::WHITE,
        );

        d.draw_line(
            dest_rec.x as i32,
            0,
            dest_rec.x as i32,
            screen_height,
            Color::GRAY,
        );
        d.draw_line(
            0,
            dest_rec.y as i32,
            screen_width,
            dest_rec.y as i32,
            Color::GRAY,
        );

        d.draw_text(
            "(c) Scarfy sprite by Eiden Marsal",
            screen_width - 200,
            screen_height - 20,
            10,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTexture is handled by RAII drop of `scarfy`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
