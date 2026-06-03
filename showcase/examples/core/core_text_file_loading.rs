/*******************************************************************************************
*
*   raylib [core] example - text file loading
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 5.5, last time updated with raylib 5.6
*
*   Example contributed by Aanjishnu Bhattacharyya (@NimComPoo-04) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 0 Aanjishnu Bhattacharyya (@NimComPoo-04)
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
        .title("raylib [core] example - text file loading")
        .build();

    // Setting up the camera
    let mut cam = Camera2D {
        offset: Vector2::new(0.0, 0.0),
        target: Vector2::new(0.0, 0.0),
        rotation: 0.0,
        zoom: 1.0,
    };

    // Loading text file from resources/core/text_file.txt
    // idiomatic: upstream C uses LoadFileText/LoadTextLines; Rust's std::fs handles file IO
    // and `.lines()` already gives us the line split.
    let file_name = "resources/core/text_file.txt";
    let text = std::fs::read_to_string(file_name).unwrap_or_default();

    // Loading all the text lines
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let line_count = lines.len();

    // Stylistic choises
    let font_size: i32 = 20;
    let text_top: i32 = 25 + font_size; // Top of the screen from where the text is rendered
    let wrap_width: i32 = screen_width - 20;

    // Wrap the lines as needed
    // idiomatic: we mirror the C in-place wrap by inserting '\n' characters. Rust strings are
    // UTF-8, but the upstream operates on ASCII spaces — keep working in bytes for parity.
    for i in 0..line_count {
        let mut bytes: Vec<u8> = lines[i].clone().into_bytes();
        let len = bytes.len();
        let mut j: usize = 0;
        let mut last_space: usize = 0;
        let mut last_wrap_start: usize = 0;

        while j <= len {
            let cur = if j < len { bytes[j] } else { 0 };
            if cur == b' ' || cur == 0 {
                let before = cur;
                // Making a C Style string by adding a '\0' at the required location so that we can use the MeasureText function
                if j < len {
                    bytes[j] = 0;
                }
                // Checking if the text has crossed the wrapWidth, then going back and inserting a newline
                let nul = bytes[last_wrap_start..]
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(bytes.len() - last_wrap_start);
                let sub = std::str::from_utf8(&bytes[last_wrap_start..last_wrap_start + nul])
                    .unwrap_or("");
                let mw = unsafe {
                    // SAFETY: pure raylib FFI taking a borrowed C-string pointer and a primitive,
                    // returning a primitive; the CString lives for the duration of the block so
                    // its pointer is valid for the call, and no aliasing/lifetime escapes.
                    let c = std::ffi::CString::new(sub).unwrap();
                    raylib::ffi::MeasureText(c.as_ptr(), font_size)
                };
                if mw > wrap_width {
                    bytes[last_space] = b'\n';

                    // Since we added a newline the place of wrap changed so we update our lastWrapStart
                    last_wrap_start = last_space + 1;
                }

                if before != 0 && j < len {
                    bytes[j] = b' ';
                } // Resetting the space back
                last_space = j; // Since we encountered a new space we update our last encountered space location
            }

            j += 1;
        }
        lines[i] = String::from_utf8_lossy(&bytes).to_string();
    }

    // Calculating the total height so that we can show a scrollbar
    let mut text_height: i32 = 0;

    for i in 0..line_count {
        let c = std::ffi::CString::new(lines[i].as_str()).unwrap();
        let size = unsafe {
            // SAFETY: pure raylib FFI; GetFontDefault returns a raylib-owned Font by value
            // and MeasureTextEx takes that Font by value plus a borrowed C-string pointer.
            // The CString outlives the call, so the pointer is valid; no aliasing or lifetime escapes.
            raylib::ffi::MeasureTextEx(
                raylib::ffi::GetFontDefault(),
                c.as_ptr(),
                font_size as f32,
                2.0,
            )
        };
        text_height += size.y as i32 + 10;
    }

    // A simple scrollbar on the side to show how far we have read into the file
    let mut scroll_bar = Rectangle::new(
        screen_width as f32 - 5.0,
        0.0,
        5.0,
        screen_height as f32 * 100.0 / (text_height - screen_height) as f32, // Scrollbar height is just a percentage
    );

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let scroll = rl.get_mouse_wheel_move();
        cam.target.y -= scroll * font_size as f32 * 1.5; // Choosing an arbitrary speed for scroll

        if cam.target.y < 0.0 {
            cam.target.y = 0.0;
        } // Snapping to 0 if we go too far back

        // Ensuring that the camera does not scroll past all text
        if cam.target.y > (text_height - screen_height + text_top) as f32 {
            cam.target.y = (text_height - screen_height + text_top) as f32;
        }

        // Computing the position of the scrollBar depending on the percentage of text covered
        scroll_bar.y = lerp(
            text_top as f32,
            screen_height as f32 - scroll_bar.height,
            (cam.target.y - text_top as f32) / (text_height - screen_height) as f32,
        );
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        {
            let mut m = d.begin_mode2D(cam);
            // Going through all the read lines
            let mut t: i32 = text_top;
            for i in 0..line_count {
                // Each time we go through and calculate the height of the text to move the cursor appropriately
                let size = if !lines[i].is_empty() {
                    // Fix for empty line in the text file
                    let c = std::ffi::CString::new(lines[i].as_str()).unwrap();
                    unsafe {
                        // SAFETY: pure raylib FFI; GetFontDefault returns a raylib-owned Font by
                        // value and MeasureTextEx reads through the borrowed C-string pointer (which
                        // is valid for the duration of this call); no aliasing or lifetime escapes.
                        raylib::ffi::MeasureTextEx(
                            raylib::ffi::GetFontDefault(),
                            c.as_ptr(),
                            font_size as f32,
                            2.0,
                        )
                    }
                } else {
                    let c = std::ffi::CString::new(" ").unwrap();
                    unsafe {
                        // SAFETY: pure raylib FFI; same justification as the non-empty branch above.
                        raylib::ffi::MeasureTextEx(
                            raylib::ffi::GetFontDefault(),
                            c.as_ptr(),
                            font_size as f32,
                            2.0,
                        )
                    }
                };

                m.draw_text(&lines[i], 10, t, font_size, Color::RED);

                // Inserting extra space for real newlines,
                // wrapped lines are rendered closer together
                t += size.y as i32 + 10;
            }
        }

        // Header displaying which file is being read currently
        d.draw_rectangle(0, 0, screen_width, text_top - 10, Color::BEIGE);
        d.draw_text(
            &format!("File: {}", file_name),
            10,
            10,
            font_size,
            Color::MAROON,
        );

        d.draw_rectangle_rec(scroll_bar, Color::MAROON);

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadTextLines / UnloadFileText handled by Rust Vec/String drop.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + amount * (end - start)
}
