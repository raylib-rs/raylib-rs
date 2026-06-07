/*******************************************************************************************
*
*   raylib [text] example - font filters
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: After font loading, font texture atlas filter could be configured for a softer
*   display of the font when scaling it to different sizes, that way, it's not required
*   to generate multiple fonts at multiple sizes (as long as the scaling is not very different)
*
*   Example originally created with raylib 1.3, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2015-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::TextureFilter;
use raylib::core::text::RaylibFont;
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
        .title("raylib [text] example - font filters")
        .build();

    let msg = "Loaded Font";

    // NOTE: Textures/Fonts MUST be loaded after Window initialization (OpenGL context is required)

    // TTF Font loading with custom generation parameters
    let mut font = rl
        .load_font_ex(&thread, "resources/text/KAISG.ttf", 96, None)
        .expect("font load");

    // Generate mipmap levels to use trilinear filtering
    // NOTE: On 2D drawing it won't be noticeable, it looks like FILTER_BILINEAR
    // SAFETY: font owns its texture; raylib's GenTextureMipmaps takes a `*mut Texture2D`
    // and mutates id/mipmaps in-place. We never use the texture concurrently.
    unsafe {
        let f: &mut raylib::ffi::Font = font.as_raw_mut();
        raylib::ffi::GenTextureMipmaps(&mut f.texture);
    }

    let mut font_size = font.base_size() as f32;
    let mut font_position = Vector2::new(40.0, screen_height as f32 / 2.0 - 80.0);
    #[allow(unused_assignments)]
    let mut text_size = Vector2::new(0.0, 0.0);

    // Setup texture scaling filter
    // SAFETY: ffi call mirroring SetTextureFilter — pure state-setter on the font's GPU texture.
    unsafe {
        raylib::ffi::SetTextureFilter(
            {
                let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                f.texture
            },
            TextureFilter::TEXTURE_FILTER_POINT as i32,
        );
    }
    let mut current_font_filter = 0; // TEXTURE_FILTER_POINT

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        font_size += rl.get_mouse_wheel_move() * 4.0;

        // Choose font texture filter method
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            // SAFETY: see above.
            unsafe {
                raylib::ffi::SetTextureFilter(
                    {
                        let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                        f.texture
                    },
                    TextureFilter::TEXTURE_FILTER_POINT as i32,
                );
            }
            current_font_filter = 0;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            // SAFETY: see above.
            unsafe {
                raylib::ffi::SetTextureFilter(
                    {
                        let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                        f.texture
                    },
                    TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
                );
            }
            current_font_filter = 1;
        } else if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            // NOTE: Trilinear filter won't be noticed on 2D drawing
            // SAFETY: see above.
            unsafe {
                raylib::ffi::SetTextureFilter(
                    {
                        let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                        f.texture
                    },
                    TextureFilter::TEXTURE_FILTER_TRILINEAR as i32,
                );
            }
            current_font_filter = 2;
        }

        text_size = font.measure_text(msg, font_size, 0.0);

        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            font_position.x -= 10.0;
        } else if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            font_position.x += 10.0;
        }

        // Load a dropped TTF file dynamically (at current fontSize)
        if rl.is_file_dropped() {
            let dropped_files = rl.load_dropped_files();
            let paths = dropped_files.paths();

            // NOTE: We only support first ttf file dropped
            if !paths.is_empty() && rl.is_file_extension(paths[0], ".ttf") {
                font = rl
                    .load_font_ex(&thread, paths[0], font_size as i32, None)
                    .expect("ttf font load");
            }

            // UnloadDroppedFiles handled by Drop of `dropped_files`.
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text(
            "Use mouse wheel to change font size",
            20,
            20,
            10,
            Color::GRAY,
        );
        d.draw_text(
            "Use KEY_RIGHT and KEY_LEFT to move text",
            20,
            40,
            10,
            Color::GRAY,
        );
        d.draw_text(
            "Use 1, 2, 3 to change texture filter",
            20,
            60,
            10,
            Color::GRAY,
        );
        d.draw_text(
            "Drop a new TTF font for dynamic loading",
            20,
            80,
            10,
            Color::DARKGRAY,
        );

        d.draw_text_ex(&font, msg, font_position, font_size, 0.0, Color::BLACK);

        // TODO: It seems texSize measurement is not accurate due to chars offsets...
        //d.draw_rectangle_lines(font_position.x, font_position.y, text_size.x, text_size.y, Color::RED);

        d.draw_rectangle(0, screen_height - 80, screen_width, 80, Color::LIGHTGRAY);
        d.draw_text(
            &format!("Font size: {font_size:05.2}"),
            20,
            screen_height - 50,
            10,
            Color::DARKGRAY,
        );
        d.draw_text(
            &format!("Text size: [{:05.2}, {:05.2}]", text_size.x, text_size.y),
            20,
            screen_height - 30,
            10,
            Color::DARKGRAY,
        );
        d.draw_text("CURRENT TEXTURE FILTER:", 250, 400, 20, Color::GRAY);

        if current_font_filter == 0 {
            d.draw_text("POINT", 570, 400, 20, Color::BLACK);
        } else if current_font_filter == 1 {
            d.draw_text("BILINEAR", 570, 400, 20, Color::BLACK);
        } else if current_font_filter == 2 {
            d.draw_text("TRILINEAR", 570, 400, 20, Color::BLACK);
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(font) handled by RAII drop of `font`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
