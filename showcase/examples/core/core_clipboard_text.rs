/*******************************************************************************************
*
*   raylib [core] example - clipboard text
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Ananth S (@Ananth1839) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Ananth S (@Ananth1839)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const MAX_TEXT_SAMPLES: usize = 5;

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
        .title("raylib [core] example - clipboard text")
        .build();

    // Define some sample texts
    let sample_texts: [&str; MAX_TEXT_SAMPLES] = [
        "Hello from raylib!",
        "The quick brown fox jumps over the lazy dog",
        "Clipboard operations are useful!",
        "raylib is a simple and easy-to-use library",
        "Copy and paste me!",
    ];

    let mut clipboard_text: String = String::new();
    // idiomatic: an owned String replaces the fixed-size `char inputBuffer[256]`.
    // We pre-reserve 256 bytes so raygui's text-box has the same edit headroom as the C version.
    let mut input_buffer = String::with_capacity(256);
    input_buffer.push_str("Hello from raylib!"); // Random initial string

    // UI required variables
    let mut text_box_edit_mode = false;

    let mut btn_cut_pressed;
    let mut btn_copy_pressed;
    let mut btn_paste_pressed;
    let mut btn_clear_pressed;
    let mut btn_random_pressed;

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Set UI style (after window init; raygui functions need a live context)
    {
        let mut d = rl.begin_drawing(&thread);
        d.gui_set_style(GuiControl::DEFAULT, GuiDefaultProperty::TEXT_SIZE, 20);
        d.gui_set_icon_scale(2);
    }

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        // (Button-press handling is moved to AFTER drawing — buttons are immediate-mode,
        // so they're sampled inside the Draw block below in the same frame.)

        // Quick cut/copy/paste with keyboard shortcuts
        if rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        {
            if rl.is_key_pressed(KeyboardKey::KEY_X) {
                rl.set_clipboard_text(&input_buffer).ok();
                input_buffer.clear(); // Quick solution to clear text
            }

            if rl.is_key_pressed(KeyboardKey::KEY_C) {
                rl.set_clipboard_text(&input_buffer).ok();
            }

            if rl.is_key_pressed(KeyboardKey::KEY_V) {
                if let Ok(s) = rl.get_clipboard_text() {
                    clipboard_text = s.clone();
                    input_buffer.clear();
                    input_buffer.push_str(&clipboard_text);
                }
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        {
            let mut d = rl.begin_drawing(&thread);

            d.clear_background(Color::RAYWHITE);

            // Draw instructions
            d.gui_label(
                Rectangle::new(50.0, 20.0, 700.0, 36.0),
                "Use the BUTTONS or KEY SHORTCUTS:",
            );
            d.draw_text(
                "[CTRL+X] - CUT | [CTRL+C] COPY | [CTRL+V] | PASTE",
                50,
                60,
                20,
                Color::MAROON,
            );

            // Draw text box
            if d.gui_text_box(
                Rectangle::new(50.0, 120.0, 652.0, 40.0),
                &mut input_buffer,
                text_box_edit_mode,
            ) {
                text_box_edit_mode = !text_box_edit_mode;
            }

            // Random text button
            btn_random_pressed = d.gui_button(
                Rectangle::new(50.0 + 652.0 + 8.0, 120.0, 40.0, 40.0),
                "#77#",
            );

            // Draw buttons
            btn_cut_pressed = d.gui_button(Rectangle::new(50.0, 180.0, 158.0, 40.0), "#17#CUT");
            btn_copy_pressed =
                d.gui_button(Rectangle::new(50.0 + 165.0, 180.0, 158.0, 40.0), "#16#COPY");
            btn_paste_pressed = d.gui_button(
                Rectangle::new(50.0 + 165.0 * 2.0, 180.0, 158.0, 40.0),
                "#18#PASTE",
            );
            btn_clear_pressed = d.gui_button(
                Rectangle::new(50.0 + 165.0 * 3.0, 180.0, 158.0, 40.0),
                "#143#CLEAR",
            );

            // Draw clipboard status
            d.gui_set_state(GuiState::STATE_DISABLED);
            d.gui_label(
                Rectangle::new(50.0, 260.0, 700.0, 40.0),
                "Clipboard current text data:",
            );
            // SAFETY: `GuiTextBoxProperty` is missing from the safe `GuiProperty` impl list as of
            // raylib-rs 6.0-rc; call the FFI directly until that gap is filled.
            unsafe {
                raylib::ffi::GuiSetStyle(
                    GuiControl::TEXTBOX as i32,
                    GuiTextBoxProperty::TEXT_READONLY as i32,
                    1,
                );
            }
            // idiomatic: read-only display uses a throwaway buffer; the wrapper requires &mut String.
            let mut clip_display = clipboard_text.clone();
            clip_display.reserve(256);
            d.gui_text_box(
                Rectangle::new(50.0, 300.0, 700.0, 40.0),
                &mut clip_display,
                false,
            );
            // SAFETY: see comment above; restore TEXT_READONLY to 0.
            unsafe {
                raylib::ffi::GuiSetStyle(
                    GuiControl::TEXTBOX as i32,
                    GuiTextBoxProperty::TEXT_READONLY as i32,
                    0,
                );
            }
            d.gui_label(
                Rectangle::new(50.0, 360.0, 700.0, 40.0),
                "Try copying text from other applications and pasting here!",
            );
            d.gui_set_state(GuiState::STATE_NORMAL);

            viewer.draw(&mut d);
        }
        //----------------------------------------------------------------------------------

        // Handle button interactions (post-Draw: sampled from immediate-mode buttons above)
        if btn_cut_pressed {
            rl.set_clipboard_text(&input_buffer).ok();
            if let Ok(s) = rl.get_clipboard_text() {
                clipboard_text = s;
            }
            input_buffer.clear(); // Quick solution to clear text
            //input_buffer = String::with_capacity(256); // Clear full buffer properly
        }

        if btn_copy_pressed {
            rl.set_clipboard_text(&input_buffer).ok(); // Copy text to clipboard
            if let Ok(s) = rl.get_clipboard_text() {
                clipboard_text = s; // Get text from clipboard
            }
        }

        if btn_paste_pressed {
            // Paste text from clipboard
            if let Ok(s) = rl.get_clipboard_text() {
                clipboard_text = s.clone();
                input_buffer.clear();
                input_buffer.push_str(&clipboard_text);
            }
        }

        if btn_clear_pressed {
            input_buffer.clear(); // Quick solution to clear text
        }

        if btn_random_pressed {
            // Get random text from sample list
            let idx = rl.get_random_value::<i32>(0..=(MAX_TEXT_SAMPLES as i32 - 1)) as usize;
            input_buffer.clear();
            input_buffer.push_str(sample_texts[idx]);
        }
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
