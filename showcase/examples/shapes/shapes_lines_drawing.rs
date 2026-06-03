/*******************************************************************************************
*
*   raylib [shapes] example - lines drawing
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 5.6
*
*   Example contributed by Robin (@RobinsAviary) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Robin (@RobinsAviary)
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
        .title("raylib [shapes] example - lines drawing")
        .build();

    // Hint text that shows before you click the screen
    let mut start_text = true;

    // The mouse's position on the previous frame
    let mut mouse_position_previous = rl.get_mouse_position();

    // The canvas to draw lines on
    let mut canvas = rl
        .load_render_texture(&thread, screen_width as u32, screen_height as u32)
        .expect("Failed to load render texture");

    // The line's thickness
    let mut line_thickness: f32 = 8.0;
    // The lines hue (in HSV, from 0-360)
    let mut line_hue: f32 = 0.0;

    // Clear the canvas to the background color
    {
        let mut d = rl.begin_drawing(&thread);
        let mut t = d.begin_texture_mode(&thread, &mut canvas);
        t.clear_background(Color::RAYWHITE);
    }

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Disable the hint text once the user clicks
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && start_text {
            start_text = false;
        }

        // Clear the canvas when the user middle-clicks
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_MIDDLE) {
            let mut d = rl.begin_drawing(&thread);
            let mut t = d.begin_texture_mode(&thread, &mut canvas);
            t.clear_background(Color::RAYWHITE);
        }

        // Store whether the left and right buttons are down
        let left_button_down = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let right_button_down = rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);

        if left_button_down || right_button_down {
            // The color for the line
            let mut draw_color = Color::WHITE;

            if left_button_down {
                // Increase the hue value by the distance our cursor has moved since the last frame (divided by 3)
                line_hue += mouse_position_previous.distance(rl.get_mouse_position()) / 3.0;

                // While the hue is >=360, subtract it to bring it down into the range 0-360
                // This is more visually accurate than resetting to zero
                while line_hue >= 360.0 {
                    line_hue -= 360.0;
                }

                // Create the final color
                draw_color = Color::color_from_hsv(line_hue, 1.0, 1.0);
            } else if right_button_down {
                draw_color = Color::RAYWHITE; // Use the background color as an "eraser"
            }

            // Draw the line onto the canvas
            let mouse_pos = rl.get_mouse_position();
            let mut d = rl.begin_drawing(&thread);
            let mut t = d.begin_texture_mode(&thread, &mut canvas);
            // Circles act as "caps", smoothing corners
            t.draw_circle_v(mouse_position_previous, line_thickness / 2.0, draw_color);
            t.draw_circle_v(mouse_pos, line_thickness / 2.0, draw_color);
            t.draw_line_ex(
                mouse_position_previous,
                mouse_pos,
                line_thickness,
                draw_color,
            );
        }

        // Update line thickness based on mousewheel
        line_thickness += rl.get_mouse_wheel_move();
        line_thickness = line_thickness.clamp(1.0, 500.0);

        // Update mouse's previous position
        mouse_position_previous = rl.get_mouse_position();
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mouse_pos = rl.get_mouse_position();
        let mut d = rl.begin_drawing(&thread);

        // Draw the render texture to the screen, flipped vertically to make it appear top-side up
        d.draw_texture_rec(
            canvas.texture(),
            Rectangle::new(
                0.0,
                0.0,
                canvas.texture().width as f32,
                -(canvas.texture().height as f32),
            ),
            Vector2::zero(),
            Color::WHITE,
        );

        // Draw the preview circle
        if !left_button_down {
            d.draw_circle_lines_v(
                mouse_pos,
                line_thickness / 2.0,
                Color::new(127, 127, 127, 127),
            );
        }

        // Draw the hint text
        if start_text {
            d.draw_text("try clicking and dragging!", 275, 215, 20, Color::LIGHTGRAY);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadRenderTexture(canvas) is handled by RAII drop of `canvas`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
