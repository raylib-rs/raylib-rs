/*******************************************************************************************
*
*   raylib [core] example - scissor test
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 3.0
*
*   Example contributed by Chris Dill (@MysteriousSpace) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2019-2025 Chris Dill (@MysteriousSpace)
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
        .title("raylib [core] example - scissor test")
        .build();

    let mut scissor_area = Rectangle::new(0.0, 0.0, 300.0, 300.0);
    let mut scissor_mode = true;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            scissor_mode = !scissor_mode;
        }

        // Centre the scissor area around the mouse position
        scissor_area.x = rl.get_mouse_x() as f32 - scissor_area.width / 2.0;
        scissor_area.y = rl.get_mouse_y() as f32 - scissor_area.height / 2.0;
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        if scissor_mode {
            let mut s = d.begin_scissor_mode(
                scissor_area.x as i32,
                scissor_area.y as i32,
                scissor_area.width as i32,
                scissor_area.height as i32,
            );

            // Draw full screen rectangle and some text
            // NOTE: Only part defined by scissor area will be rendered
            s.draw_rectangle(0, 0, screen_w, screen_h, Color::RED);
            s.draw_text(
                "Move the mouse around to reveal this text!",
                190,
                200,
                20,
                Color::LIGHTGRAY,
            );
        } else {
            d.draw_rectangle(0, 0, screen_w, screen_h, Color::RED);
            d.draw_text(
                "Move the mouse around to reveal this text!",
                190,
                200,
                20,
                Color::LIGHTGRAY,
            );
        }

        d.draw_rectangle_lines_ex(scissor_area, 1.0, Color::BLACK);
        d.draw_text("Press S to toggle scissor test", 10, 10, 20, Color::BLACK);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
