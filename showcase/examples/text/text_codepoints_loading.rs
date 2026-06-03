/*******************************************************************************************
*
*   raylib [text] example - codepoints loading
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 4.2, last time updated with raylib 4.2
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2022-2025 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::consts::TextureFilter;
use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// Text to be displayed, must be UTF-8 (this file is saved as UTF-8)
// NOTE: It can contain all the required text for the game,
// this text will be scanned to get all the required codepoints
const TEXT: &str = "いろはにほへと\u{3000}ちりぬるを\nわかよたれそ\u{3000}つねならむ\nうゐのおくやま\u{3000}けふこえて\nあさきゆめみし\u{3000}ゑひもせす";

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Remove codepoint duplicates if requested
fn codepoint_remove_duplicates(codepoints: &[i32]) -> Vec<i32> {
    let mut result: Vec<i32> = codepoints.to_vec();

    // Remove duplicates
    let mut i = 0;
    while i < result.len() {
        let mut j = i + 1;
        while j < result.len() {
            if result[i] == result[j] {
                result.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }

    // NOTE: The size of result is no larger than the original; duplicates have been removed.
    result
}

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
        .title("raylib [text] example - codepoints loading")
        .build();

    // Convert each utf-8 character into its
    // corresponding codepoint in the font file
    // SAFETY: ffi LoadCodepoints returns a raylib-allocated int* + count; we copy into a Vec
    // and free with UnloadCodepoints before returning.
    let (codepoints, codepoint_count) = unsafe {
        let c = std::ffi::CString::new(TEXT).unwrap();
        let mut len: i32 = 0;
        let raw = raylib::ffi::LoadCodepoints(c.as_ptr(), &mut len);
        let v: Vec<i32> = std::slice::from_raw_parts(raw, len as usize).to_vec();
        raylib::ffi::UnloadCodepoints(raw);
        (v, len)
    };

    // Removed duplicate codepoints to generate smaller font atlas
    let mut codepoints_no_dups = codepoint_remove_duplicates(&codepoints);
    let codepoints_no_dups_count = codepoints_no_dups.len() as i32;
    drop(codepoints); // free original list

    // Load font containing all the provided codepoint glyphs
    // A texture font atlas is automatically generated
    // SAFETY: LoadFontEx takes an i32* + count we own; the loaded Font then owns its glyphs/recs/texture.
    let font = unsafe {
        let c = std::ffi::CString::new("resources/text/DotGothic16-Regular.ttf").unwrap();
        let f = raylib::ffi::LoadFontEx(
            c.as_ptr(),
            36,
            codepoints_no_dups.as_mut_ptr(),
            codepoints_no_dups_count,
        );
        Font::from_raw(f)
    };

    // Set bilinear scale filter for better font scaling
    // SAFETY: state-setter on the GPU texture owned by `font`.
    unsafe {
        raylib::ffi::SetTextureFilter(
            {
                let f: &raylib::ffi::Font = std::convert::AsRef::as_ref(&font);
                f.texture
            },
            TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
        );
    }

    rl.set_text_line_spacing(20); // Set line spacing for multiline text (when line breaks are included '\n')

    // Free codepoints, atlas has already been generated
    drop(codepoints_no_dups);

    let mut show_font_atlas = false;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            show_font_atlas = !show_font_atlas;
        }

        // Testing code: getting next and previous codepoints on provided text
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            // Get next codepoint in string and move pointer
            // idiomatic: C uses GetCodepointNext(ptr, &codepointSize); Rust strings handle UTF-8 natively.
        } else if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            // Get previous codepoint in string and move pointer
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let screen_w = rl.get_screen_width();
        let screen_h = rl.get_screen_height();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_rectangle(0, 0, screen_w, 70, Color::BLACK);
        d.draw_text(
            &format!(
                "Total codepoints contained in provided text: {}",
                codepoint_count
            ),
            10,
            10,
            20,
            Color::GREEN,
        );
        d.draw_text(
            &format!(
                "Total codepoints required for font atlas (duplicates excluded): {}",
                codepoints_no_dups_count
            ),
            10,
            40,
            20,
            Color::GREEN,
        );

        if show_font_atlas {
            // Draw generated font texture atlas containing provided codepoints
            d.draw_texture(font.texture(), 150, 100, Color::BLACK);
            d.draw_rectangle_lines(
                150,
                100,
                font.texture().width,
                font.texture().height,
                Color::BLACK,
            );
        } else {
            // Draw provided text with loaded font, containing all required codepoint glyphs
            d.draw_text_ex(
                &font,
                TEXT,
                Vector2::new(160.0, 110.0),
                48.0,
                5.0,
                Color::BLACK,
            );
        }

        d.draw_text(
            "Press SPACE to toggle font atlas view!",
            10,
            screen_h - 30,
            20,
            Color::GRAY,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // UnloadFont(font) handled by RAII drop of `font`.
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
