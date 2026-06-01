/*******************************************************************************************
*
*   raylib [text] example - words alignment
*
*   Example complexity rating: [★☆☆☆] 1/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by JP Mortiboys (@themushroompirates) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 JP Mortiboys (@themushroompirates)
*
********************************************************************************************/

use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

// idiomatic: TextAlignment is collapsed to a plain i32 (0/1/2) to mirror the
// C enum (LEFT/TOP=0, CENTRE/MIDDLE=1, RIGHT/BOTTOM=2).
type TextAlignment = i32;
const TEXT_ALIGN_LEFT: TextAlignment = 0;
const TEXT_ALIGN_CENTRE: TextAlignment = 1;
const TEXT_ALIGN_MIDDLE: TextAlignment = 1;

fn lerp(start: f32, end: f32, amount: f32) -> f32 {
    start + amount * (end - start)
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
        .title("raylib [text] example - words alignment")
        .build();

    // Define the rectangle we will draw the text in
    let text_container_rect = Rectangle::new(
        screen_width as f32 / 2.0 - screen_width as f32 / 4.0,
        screen_height as f32 / 2.0 - screen_height as f32 / 3.0,
        screen_width as f32 / 2.0,
        screen_height as f32 * 2.0 / 3.0,
    );

    // Some text to display the current alignment
    let text_align_name_h = ["Left", "Centre", "Right"];
    let text_align_name_v = ["Top", "Middle", "Bottom"];

    // Define the text we're going to draw in the rectangle
    // idiomatic: upstream uses TextSplit("...", ' ', &wordCount); Rust splits with str::split
    #[allow(unused_assignments)]
    let mut word_index: usize = 0;
    let words: Vec<&str> =
        "raylib is a simple and easy-to-use library to enjoy videogames programming"
            .split(' ')
            .collect();
    let word_count = words.len();

    // Initialize the font size we're going to use
    let font_size: i32 = 40;

    // And of course the font...
    let font = rl.get_font_default();

    // Initialize the alignment variables
    let mut h_align: TextAlignment = TEXT_ALIGN_CENTRE;
    let mut v_align: TextAlignment = TEXT_ALIGN_MIDDLE;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            if h_align > 0 {
                h_align -= 1;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            h_align += 1;
            if h_align > 2 {
                h_align = 2;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_UP) {
            if v_align > 0 {
                v_align -= 1;
            }
        }

        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
            v_align += 1;
            if v_align > 2 {
                v_align = 2;
            }
        }

        // One word per second
        if word_count > 0 {
            word_index = (rl.get_time() as usize) % word_count;
        } else {
            word_index = 0;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::DARKBLUE);

        d.draw_text(
            "Use Arrow Keys to change the text alignment",
            20,
            20,
            20,
            Color::LIGHTGRAY,
        );
        d.draw_text(
            &format!(
                "Alignment: Horizontal = {}, Vertical = {}",
                text_align_name_h[h_align as usize], text_align_name_v[v_align as usize]
            ),
            20,
            40,
            20,
            Color::LIGHTGRAY,
        );

        d.draw_rectangle_rec(text_container_rect, Color::BLUE);

        // Get the size of the text to draw
        let text_size =
            font.measure_text(words[word_index], font_size as f32, font_size as f32 * 0.1);

        // Calculate the top-left text position based on the rectangle and alignment
        let text_pos = Vector2::new(
            text_container_rect.x
                + lerp(
                    0.0,
                    text_container_rect.width - text_size.x,
                    h_align as f32 * 0.5,
                ),
            text_container_rect.y
                + lerp(
                    0.0,
                    text_container_rect.height - text_size.y,
                    v_align as f32 * 0.5,
                ),
        );

        // Draw the text
        d.draw_text_ex(
            &font,
            words[word_index],
            text_pos,
            font_size as f32,
            font_size as f32 * 0.1,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
    let _ = TEXT_ALIGN_LEFT; // silence "unused const" warning under varying compile paths
}
