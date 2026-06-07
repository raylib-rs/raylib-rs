/*******************************************************************************************
*
*   raylib [core] example - delta time
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 6.0
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
        .title("raylib [core] example - delta time")
        .build();

    let mut current_fps: i32 = 60;

    // Store the position for the both of the circles
    let mut delta_circle = Vector2::new(0.0, screen_height as f32 / 3.0);
    let mut frame_circle = Vector2::new(0.0, screen_height as f32 * (2.0 / 3.0));

    // The speed applied to both circles
    let speed: f32 = 10.0;
    let circle_radius: f32 = 32.0;

    rl.set_target_fps(current_fps as u32);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // Adjust the FPS target based on the mouse wheel
        let mouse_wheel = rl.get_mouse_wheel_move();
        if mouse_wheel != 0.0 {
            current_fps += mouse_wheel as i32;
            if current_fps < 0 {
                current_fps = 0;
            }
            rl.set_target_fps(current_fps as u32);
        }

        // GetFrameTime() returns the time it took to draw the last frame, in seconds (usually called delta time)
        // Uses the delta time to make the circle look like it's moving at a "consistent" speed regardless of FPS

        // Multiply by 6.0 (an arbitrary value) in order to make the speed
        // visually closer to the other circle (at 60 fps), for comparison
        delta_circle.x += rl.get_frame_time() * 6.0 * speed;
        // This circle can move faster or slower visually depending on the FPS
        frame_circle.x += 0.1 * speed;

        // If either circle is off the screen, reset it back to the start
        if delta_circle.x > screen_width as f32 {
            delta_circle.x = 0.0;
        }
        if frame_circle.x > screen_width as f32 {
            frame_circle.x = 0.0;
        }

        // Reset both circles positions
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            delta_circle.x = 0.0;
            frame_circle.x = 0.0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let fps = rl.get_fps();
        let frame_time = rl.get_frame_time();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        // Draw both circles to the screen
        d.draw_circle_v(delta_circle, circle_radius, Color::RED);
        d.draw_circle_v(frame_circle, circle_radius, Color::BLUE);

        // Draw the help text
        // Determine what help text to show depending on the current FPS target
        let fps_text = if current_fps <= 0 {
            format!("FPS: unlimited ({fps})")
        } else {
            format!("FPS: {fps} (target: {current_fps})")
        };
        d.draw_text(&fps_text, 10, 10, 20, Color::DARKGRAY);
        // idiomatic: matches the upstream C — GetFrameTime() is in seconds, the "ms" label
        // in the format string is intentional/historical (a known display quirk in raylib's
        // example); we mirror it verbatim instead of "fixing" it to true milliseconds.
        d.draw_text(
            &format!("Frame time: {frame_time:05.2} ms"),
            10,
            30,
            20,
            Color::DARKGRAY,
        );
        d.draw_text(
            "Use the scroll wheel to change the fps limit, r to reset",
            10,
            50,
            20,
            Color::DARKGRAY,
        );

        // Draw the text above the circles
        d.draw_text("FUNC: x += GetFrameTime()*speed", 10, 90, 20, Color::RED);
        d.draw_text("FUNC: x += speed", 10, 240, 20, Color::BLUE);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
