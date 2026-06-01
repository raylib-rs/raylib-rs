/*******************************************************************************************
*
*   raylib [text] example - input box
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 1.7, last time updated with raylib 3.5
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2017-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::ffi::MouseCursor;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_INPUT_CHARS: usize = 9;

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
        .title("raylib [text] example - input box")
        .build();

    // NOTE: One extra space required for null terminator char '\0' (Rust uses String, not fixed buffer)
    let mut name = String::new();

    let text_box = Rectangle::new(screen_width as f32 / 2.0 - 100.0, 180.0, 225.0, 50.0);
    #[allow(unused_assignments)]
    let mut mouse_on_text = false;

    let mut frames_counter = 0;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if text_box.check_collision_point_rec(rl.get_mouse_position()) {
            mouse_on_text = true;
        } else {
            mouse_on_text = false;
        }

        if mouse_on_text {
            // Set the window's cursor to the I-Beam
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_IBEAM);

            // Get char pressed (unicode character) on the queue
            let mut key = rl.get_char_pressed();

            // Check if more characters have been pressed on the same frame
            while let Some(ch) = key {
                // NOTE: Only allow keys in range [32..125]
                if (ch as u32) >= 32 && (ch as u32) <= 125 && name.len() < MAX_INPUT_CHARS {
                    name.push(ch);
                }

                key = rl.get_char_pressed(); // Check next character in the queue
            }

            if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
                name.pop();
            }
        } else {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_DEFAULT);
        }

        if mouse_on_text {
            frames_counter += 1;
        } else {
            frames_counter = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let name_measure = rl.measure_text(&name, 40);
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("PLACE MOUSE OVER INPUT BOX!", 240, 140, 20, Color::GRAY);

        d.draw_rectangle_rec(text_box, Color::LIGHTGRAY);
        if mouse_on_text {
            d.draw_rectangle_lines(
                text_box.x as i32,
                text_box.y as i32,
                text_box.width as i32,
                text_box.height as i32,
                Color::RED,
            );
        } else {
            d.draw_rectangle_lines(
                text_box.x as i32,
                text_box.y as i32,
                text_box.width as i32,
                text_box.height as i32,
                Color::DARKGRAY,
            );
        }

        d.draw_text(
            &name,
            text_box.x as i32 + 5,
            text_box.y as i32 + 8,
            40,
            Color::MAROON,
        );

        d.draw_text(
            &format!("INPUT CHARS: {}/{}", name.len(), MAX_INPUT_CHARS),
            315,
            250,
            20,
            Color::DARKGRAY,
        );

        if mouse_on_text {
            if name.len() < MAX_INPUT_CHARS {
                // Draw blinking underscore char
                if ((frames_counter / 20) % 2) == 0 {
                    d.draw_text(
                        "_",
                        text_box.x as i32 + 8 + name_measure,
                        text_box.y as i32 + 12,
                        40,
                        Color::MAROON,
                    );
                }
            } else {
                d.draw_text(
                    "Press BACKSPACE to delete chars...",
                    230,
                    300,
                    20,
                    Color::GRAY,
                );
            }
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
