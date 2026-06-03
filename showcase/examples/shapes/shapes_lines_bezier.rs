/*******************************************************************************************
*
*   raylib [shapes] example - lines bezier
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 1.7, last time updated with raylib 1.7
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [shapes] example - lines bezier")
        .msaa_4x()
        .build();

    let mut start_point = Vector2::new(30.0, 30.0);
    let mut end_point = Vector2::new(screen_width as f32 - 30.0, screen_height as f32 - 30.0);
    let mut move_start_point = false;
    let mut move_end_point = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let mouse = rl.get_mouse_position();

        if check_collision_point_circle(mouse, start_point, 10.0)
            && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
        {
            move_start_point = true;
        } else if check_collision_point_circle(mouse, end_point, 10.0)
            && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
        {
            move_end_point = true;
        }

        if move_start_point {
            start_point = mouse;
            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                move_start_point = false;
            }
        }

        if move_end_point {
            end_point = mouse;
            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                move_end_point = false;
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("MOVE START-END POINTS WITH MOUSE", 15, 20, 20, Color::GRAY);

        // Draw line Cubic Bezier, in-out interpolation (easing), no control points
        d.draw_line_bezier(start_point, end_point, 4.0, Color::BLUE);

        // Draw start-end spline circles with some details
        d.draw_circle_v(
            start_point,
            if check_collision_point_circle(mouse, start_point, 10.0) {
                14.0
            } else {
                8.0
            },
            if move_start_point {
                Color::RED
            } else {
                Color::BLUE
            },
        );
        d.draw_circle_v(
            end_point,
            if check_collision_point_circle(mouse, end_point, 10.0) {
                14.0
            } else {
                8.0
            },
            if move_end_point {
                Color::RED
            } else {
                Color::BLUE
            },
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
