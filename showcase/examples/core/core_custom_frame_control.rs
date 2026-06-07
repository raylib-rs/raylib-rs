/*******************************************************************************************
*
*   raylib [core] example - custom frame control
*
*   Example complexity rating: [★★★★] 4/4
*
*   NOTE: WARNING: This is an example for advanced users willing to have full control over
*   the frame processes. By default, EndDrawing() calls the following processes:
*       1. Draw remaining batch data: rlDrawRenderBatchActive()
*       2. SwapScreenBuffer()
*       3. Frame time control: WaitTime()
*       4. PollInputEvents()
*
*   To avoid steps 2, 3 and 4, flag SUPPORT_CUSTOM_FRAME_CONTROL can be enabled in
*   config.h (it requires recompiling raylib). This way those steps are up to the user
*
*   Note that enabling this flag invalidates some functions:
*       - GetFrameTime()
*       - SetTargetFPS()
*       - GetFPS()
*
*   Example originally created with raylib 4.0, last time updated with raylib 4.0
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2021-2025 Ramon Santamaria (@raysan5)
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
        .title("raylib [core] example - custom frame control")
        .build();

    // Custom timming variables
    let mut previous_time = rl.get_time(); // Previous time measure
    let mut current_time; // Current time measure
    let mut update_draw_time; // Update + Draw time
    let mut wait_time; // Wait time (if target fps required)
    let mut delta_time: f32 = 0.0; // Frame time (Update + Draw + Wait time)

    let mut time_counter: f32 = 0.0; // Accumulative time counter (seconds)
    let mut position: f32 = 0.0; // Circle position
    let mut pause = false; // Pause control flag

    let mut target_fps: i32 = 60; // Our initial target fps
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // NOTE: On non web platforms the PollInputEvents just works before the inputs checks
        rl.poll_input_events(); // Poll input events (SUPPORT_CUSTOM_FRAME_CONTROL)

        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            pause = !pause;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            target_fps += 20;
        } else if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            target_fps -= 20;
        }

        if target_fps < 0 {
            target_fps = 0;
        }

        if !pause {
            position += 200.0 * delta_time; // We move at 200 pixels per second
            if position >= rl.get_screen_width() as f32 {
                position = 0.0;
            }
            time_counter += delta_time; // We count time (seconds)
        }
        viewer.update(&mut rl, &thread);

        // NOTE: On web platform for some reason the PollInputEvents only works after the inputs check,
        // so just call it after check all your inputs (on web)
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        {
            let mut d = rl.begin_drawing(&thread);

            d.clear_background(Color::RAYWHITE);

            for i in 0..(screen_w / 200) {
                d.draw_rectangle(200 * i, 0, 1, screen_h, Color::SKYBLUE);
            }

            d.draw_circle(position as i32, screen_h / 2 - 25, 50.0, Color::RED);

            d.draw_text(
                &format!("{:03.0} ms", time_counter * 1000.0),
                position as i32 - 40,
                screen_h / 2 - 100,
                20,
                Color::MAROON,
            );
            d.draw_text(
                &format!("PosX: {position:03.0}"),
                position as i32 - 50,
                screen_h / 2 + 40,
                20,
                Color::BLACK,
            );

            d.draw_text(
                "Circle is moving at a constant 200 pixels/sec,\nindependently of the frame rate.",
                10,
                10,
                20,
                Color::DARKGRAY,
            );
            d.draw_text(
                "PRESS SPACE to PAUSE MOVEMENT",
                10,
                screen_h - 60,
                20,
                Color::GRAY,
            );
            d.draw_text(
                "PRESS UP | DOWN to CHANGE TARGET FPS",
                10,
                screen_h - 30,
                20,
                Color::GRAY,
            );
            d.draw_text(
                &format!("TARGET FPS: {target_fps}"),
                screen_w - 220,
                10,
                20,
                Color::LIME,
            );
            if delta_time != 0.0 {
                d.draw_text(
                    &format!("CURRENT FPS: {}", (1.0 / delta_time) as i32),
                    screen_w - 220,
                    40,
                    20,
                    Color::GREEN,
                );
            }

            viewer.draw(&mut d);
        }

        // NOTE: In case raylib is configured to SUPPORT_CUSTOM_FRAME_CONTROL,
        // Events polling, screen buffer swap and frame time control must be managed by the user

        rl.swap_screen_buffer(); // Flip the back buffer to screen (front buffer)

        current_time = rl.get_time();
        update_draw_time = current_time - previous_time;

        if target_fps > 0
        // We want a fixed frame rate
        {
            wait_time = (1.0 / target_fps as f64) - update_draw_time;
            if wait_time > 0.0 {
                rl.wait_time(wait_time);
                current_time = rl.get_time();
                delta_time = (current_time - previous_time) as f32;
            }
        } else {
            delta_time = update_draw_time as f32; // Framerate could be variable
        }

        previous_time = current_time;
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
