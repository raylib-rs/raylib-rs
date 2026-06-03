/*******************************************************************************************
*
*   raylib [shapes] example - dashed line
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.5
*
*   Example contributed by Luís Almeida (@luis605)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Luís Almeida (@luis605)
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
        .title("raylib [shapes] example - dashed line")
        .build();

    // Line Properties
    let line_start_position = Vector2::new(20.0, 50.0);
    #[allow(unused_assignments)]
    let mut line_end_position = Vector2::new(780.0, 400.0);
    let mut dash_length: f32 = 25.0;
    let mut blank_length: f32 = 15.0;

    // Color selection
    let line_colors = [
        Color::RED,
        Color::ORANGE,
        Color::GOLD,
        Color::GREEN,
        Color::BLUE,
        Color::VIOLET,
        Color::PINK,
        Color::BLACK,
    ];
    let mut color_index: usize = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        line_end_position = rl.get_mouse_position(); // Line endpoint follows the mouse

        // Change Dash Length (UP/DOWN arrows)
        if rl.is_key_down(KeyboardKey::KEY_UP) {
            dash_length += 1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_DOWN) && dash_length > 1.0 {
            dash_length -= 1.0;
        }

        // Change Space Length (LEFT/RIGHT arrows)
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            blank_length += 1.0;
        }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) && blank_length > 1.0 {
            blank_length -= 1.0;
        }

        // Cycle through colors ('C' key)
        if rl.is_key_pressed(KeyboardKey::KEY_C) {
            color_index = (color_index + 1) % line_colors.len();
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Draw the dashed line with the current properties
        d.draw_line_dashed(
            line_start_position,
            line_end_position,
            dash_length as i32,
            blank_length as i32,
            line_colors[color_index],
        );

        // Draw UI and Instructions
        d.draw_rectangle(5, 5, 265, 95, Color::SKYBLUE.alpha(0.5));
        d.draw_rectangle_lines(5, 5, 265, 95, Color::BLUE);

        d.draw_text("CONTROLS:", 15, 15, 10, Color::BLACK);
        d.draw_text("UP/DOWN: Change Dash Length", 15, 35, 10, Color::BLACK);
        d.draw_text("LEFT/RIGHT: Change Space Length", 15, 55, 10, Color::BLACK);
        d.draw_text("C: Cycle Color", 15, 75, 10, Color::BLACK);

        d.draw_text(
            &format!("Dash: {:.0} | Space: {:.0}", dash_length, blank_length),
            15,
            115,
            10,
            Color::DARKGRAY,
        );

        d.draw_fps(screen_width - 80, 10);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
