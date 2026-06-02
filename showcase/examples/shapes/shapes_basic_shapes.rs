/*******************************************************************************************
*
*   raylib [shapes] example - basic shapes
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.0, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2014-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [shapes] example - basic shapes")
        .build();

    let mut rotation: f32 = 0.0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        rotation += 0.2;
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "some basic shapes available on raylib",
            20,
            20,
            20,
            Color::DARKGRAY,
        );

        // Circle shapes and lines
        d.draw_circle(screen_width / 5, 120, 35.0, Color::DARKBLUE);
        d.draw_circle_gradient(screen_width / 5, 220, 60.0, Color::GREEN, Color::SKYBLUE);
        d.draw_circle_lines(screen_width / 5, 340, 80.0, Color::DARKBLUE);
        d.draw_ellipse(screen_width / 5, 120, 25.0, 20.0, Color::YELLOW);
        d.draw_ellipse_lines(screen_width / 5, 120, 30.0, 25.0, Color::YELLOW);

        // Rectangle shapes and lines
        d.draw_rectangle(screen_width / 4 * 2 - 60, 100, 120, 60, Color::RED);
        d.draw_rectangle_gradient_h(
            screen_width / 4 * 2 - 90,
            170,
            180,
            130,
            Color::MAROON,
            Color::GOLD,
        );
        d.draw_rectangle_lines(screen_width / 4 * 2 - 40, 320, 80, 60, Color::ORANGE); // NOTE: Uses QUADS internally, not lines

        // Triangle shapes and lines
        d.draw_triangle(
            Vector2::new(screen_width as f32 / 4.0 * 3.0, 80.0),
            Vector2::new(screen_width as f32 / 4.0 * 3.0 - 60.0, 150.0),
            Vector2::new(screen_width as f32 / 4.0 * 3.0 + 60.0, 150.0),
            Color::VIOLET,
        );

        d.draw_triangle_lines(
            Vector2::new(screen_width as f32 / 4.0 * 3.0, 160.0),
            Vector2::new(screen_width as f32 / 4.0 * 3.0 - 20.0, 230.0),
            Vector2::new(screen_width as f32 / 4.0 * 3.0 + 20.0, 230.0),
            Color::DARKBLUE,
        );

        // Polygon shapes and lines
        d.draw_poly(
            Vector2::new(screen_width as f32 / 4.0 * 3.0, 330.0),
            6,
            80.0,
            rotation,
            Color::BROWN,
        );
        d.draw_poly_lines(
            Vector2::new(screen_width as f32 / 4.0 * 3.0, 330.0),
            6,
            90.0,
            rotation,
            Color::BROWN,
        );
        d.draw_poly_lines_ex(
            Vector2::new(screen_width as f32 / 4.0 * 3.0, 330.0),
            6,
            85.0,
            rotation,
            6.0,
            Color::BEIGE,
        );

        // NOTE: We draw all LINES based shapes together to optimize internal drawing,
        // this way, all LINES are rendered in a single draw pass
        d.draw_line(18, 42, screen_width - 18, 42, Color::BLACK);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
