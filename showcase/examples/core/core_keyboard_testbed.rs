/*******************************************************************************************
*
*   raylib [core] example - keyboard testbed
*
*   Example complexity rating: [★★☆☆] 2/4
*
*   NOTE: raylib defined keys refer to ENG-US Keyboard layout,
*   mapping to other layouts is up to the user
*
*   Example originally created with raylib 5.6, last time updated with raylib 5.6
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2026 Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::prelude::*;
use raylib_showcase::SourceViewer;

const KEY_REC_SPACING: f32 = 4.0; // Space in pixels between key rectangles

//------------------------------------------------------------------------------------
// Module Functions Declaration
//------------------------------------------------------------------------------------
// Get keyboard keycode as text (US keyboard)
// NOTE: Mapping for other keyboard layouts can be done here
fn get_key_text(key: i32) -> &'static str {
    use raylib::ffi::KeyboardKey as K;
    match key {
        x if x == K::KEY_APOSTROPHE as i32 => "'",
        x if x == K::KEY_COMMA as i32 => ",",
        x if x == K::KEY_MINUS as i32 => "-",
        x if x == K::KEY_PERIOD as i32 => ".",
        x if x == K::KEY_SLASH as i32 => "/",
        x if x == K::KEY_ZERO as i32 => "0",
        x if x == K::KEY_ONE as i32 => "1",
        x if x == K::KEY_TWO as i32 => "2",
        x if x == K::KEY_THREE as i32 => "3",
        x if x == K::KEY_FOUR as i32 => "4",
        x if x == K::KEY_FIVE as i32 => "5",
        x if x == K::KEY_SIX as i32 => "6",
        x if x == K::KEY_SEVEN as i32 => "7",
        x if x == K::KEY_EIGHT as i32 => "8",
        x if x == K::KEY_NINE as i32 => "9",
        x if x == K::KEY_SEMICOLON as i32 => ";",
        x if x == K::KEY_EQUAL as i32 => "=",
        x if x == K::KEY_A as i32 => "A",
        x if x == K::KEY_B as i32 => "B",
        x if x == K::KEY_C as i32 => "C",
        x if x == K::KEY_D as i32 => "D",
        x if x == K::KEY_E as i32 => "E",
        x if x == K::KEY_F as i32 => "F",
        x if x == K::KEY_G as i32 => "G",
        x if x == K::KEY_H as i32 => "H",
        x if x == K::KEY_I as i32 => "I",
        x if x == K::KEY_J as i32 => "J",
        x if x == K::KEY_K as i32 => "K",
        x if x == K::KEY_L as i32 => "L",
        x if x == K::KEY_M as i32 => "M",
        x if x == K::KEY_N as i32 => "N",
        x if x == K::KEY_O as i32 => "O",
        x if x == K::KEY_P as i32 => "P",
        x if x == K::KEY_Q as i32 => "Q",
        x if x == K::KEY_R as i32 => "R",
        x if x == K::KEY_S as i32 => "S",
        x if x == K::KEY_T as i32 => "T",
        x if x == K::KEY_U as i32 => "U",
        x if x == K::KEY_V as i32 => "V",
        x if x == K::KEY_W as i32 => "W",
        x if x == K::KEY_X as i32 => "X",
        x if x == K::KEY_Y as i32 => "Y",
        x if x == K::KEY_Z as i32 => "Z",
        x if x == K::KEY_LEFT_BRACKET as i32 => "[",
        x if x == K::KEY_BACKSLASH as i32 => "\\",
        x if x == K::KEY_RIGHT_BRACKET as i32 => "]",
        x if x == K::KEY_GRAVE as i32 => "`",
        x if x == K::KEY_SPACE as i32 => "SPACE",
        x if x == K::KEY_ESCAPE as i32 => "ESC",
        x if x == K::KEY_ENTER as i32 => "ENTER",
        x if x == K::KEY_TAB as i32 => "TAB",
        x if x == K::KEY_BACKSPACE as i32 => "BACK",
        x if x == K::KEY_INSERT as i32 => "INS",
        x if x == K::KEY_DELETE as i32 => "DEL",
        x if x == K::KEY_RIGHT as i32 => "RIGHT",
        x if x == K::KEY_LEFT as i32 => "LEFT",
        x if x == K::KEY_DOWN as i32 => "DOWN",
        x if x == K::KEY_UP as i32 => "UP",
        x if x == K::KEY_PAGE_UP as i32 => "PGUP",
        x if x == K::KEY_PAGE_DOWN as i32 => "PGDOWN",
        x if x == K::KEY_HOME as i32 => "HOME",
        x if x == K::KEY_END as i32 => "END",
        x if x == K::KEY_CAPS_LOCK as i32 => "CAPS",
        x if x == K::KEY_SCROLL_LOCK as i32 => "LOCK",
        x if x == K::KEY_NUM_LOCK as i32 => "NUMLOCK",
        x if x == K::KEY_PRINT_SCREEN as i32 => "PRINTSCR",
        x if x == K::KEY_PAUSE as i32 => "PAUSE",
        x if x == K::KEY_F1 as i32 => "F1",
        x if x == K::KEY_F2 as i32 => "F2",
        x if x == K::KEY_F3 as i32 => "F3",
        x if x == K::KEY_F4 as i32 => "F4",
        x if x == K::KEY_F5 as i32 => "F5",
        x if x == K::KEY_F6 as i32 => "F6",
        x if x == K::KEY_F7 as i32 => "F7",
        x if x == K::KEY_F8 as i32 => "F8",
        x if x == K::KEY_F9 as i32 => "F9",
        x if x == K::KEY_F10 as i32 => "F10",
        x if x == K::KEY_F11 as i32 => "F11",
        x if x == K::KEY_F12 as i32 => "F12",
        x if x == K::KEY_LEFT_SHIFT as i32 => "LSHIFT",
        x if x == K::KEY_LEFT_CONTROL as i32 => "LCTRL",
        x if x == K::KEY_LEFT_ALT as i32 => "LALT",
        x if x == K::KEY_LEFT_SUPER as i32 => "WIN",
        x if x == K::KEY_RIGHT_SHIFT as i32 => "RSHIFT",
        x if x == K::KEY_RIGHT_CONTROL as i32 => "RCTRL",
        x if x == K::KEY_RIGHT_ALT as i32 => "ALTGR",
        x if x == K::KEY_RIGHT_SUPER as i32 => "RSUPER",
        x if x == K::KEY_KB_MENU as i32 => "KBMENU",
        x if x == K::KEY_KP_0 as i32 => "KP0",
        x if x == K::KEY_KP_1 as i32 => "KP1",
        x if x == K::KEY_KP_2 as i32 => "KP2",
        x if x == K::KEY_KP_3 as i32 => "KP3",
        x if x == K::KEY_KP_4 as i32 => "KP4",
        x if x == K::KEY_KP_5 as i32 => "KP5",
        x if x == K::KEY_KP_6 as i32 => "KP6",
        x if x == K::KEY_KP_7 as i32 => "KP7",
        x if x == K::KEY_KP_8 as i32 => "KP8",
        x if x == K::KEY_KP_9 as i32 => "KP9",
        x if x == K::KEY_KP_DECIMAL as i32 => "KPDEC",
        x if x == K::KEY_KP_DIVIDE as i32 => "KPDIV",
        x if x == K::KEY_KP_MULTIPLY as i32 => "KPMUL",
        x if x == K::KEY_KP_SUBTRACT as i32 => "KPSUB",
        x if x == K::KEY_KP_ADD as i32 => "KPADD",
        x if x == K::KEY_KP_ENTER as i32 => "KPENTER",
        x if x == K::KEY_KP_EQUAL as i32 => "KPEQU",
        _ => "",
    }
}

// Map a layout-array int to a KeyboardKey, treating the upstream `162` placeholder
// (and `KEY_NULL == 0`) as "no key" so we never transmute a non-discriminant value.
// idiomatic: C just passes the int to IsKeyDown(); Rust's typed API needs us to gate
// the transmute on entries we know are real KEY_* discriminants.
fn key_for_layout(key: i32) -> Option<KeyboardKey> {
    match key {
        // `0` is KEY_NULL, `162` is an upstream placeholder in line06_keys (see raylib's
        // core_keyboard_testbed.c). Neither is a KeyboardKey discriminant — skip both.
        0 | 162 => None,
        v => Some(unsafe {
            // SAFETY: every other entry in the layout arrays (`line0X_keys`) is built from
            // a `KeyboardKey::KEY_*` discriminant via `as i32`, so transmuting back is sound.
            // raylib-sys generates KeyboardKey as #[repr(i32)], so the bit pattern is the
            // same as the original enum value.
            std::mem::transmute::<i32, KeyboardKey>(v)
        }),
    }
}

// Draw keyboard key
fn gui_keyboard_key(d: &mut RaylibDrawHandle, bounds: Rectangle, key: i32) {
    if let Some(key_enum) = key_for_layout(key) {
        if d.is_key_down(key_enum) {
            d.draw_rectangle_lines_ex(bounds, 2.0, Color::MAROON);
            d.draw_text(
                get_key_text(key),
                (bounds.x + 4.0) as i32,
                (bounds.y + 4.0) as i32,
                10,
                Color::MAROON,
            );
        } else {
            d.draw_rectangle_lines_ex(bounds, 2.0, Color::DARKGRAY);
            d.draw_text(
                get_key_text(key),
                (bounds.x + 4.0) as i32,
                (bounds.y + 4.0) as i32,
                10,
                Color::DARKGRAY,
            );
        }
    } else {
        // KEY_NULL or upstream placeholder: just outline the slot like C does for KEY_NULL.
        d.draw_rectangle_lines_ex(bounds, 2.0, Color::LIGHTGRAY);
    }

    if bounds.check_collision_point_rec(d.get_mouse_position()) {
        d.draw_rectangle_rec(bounds, Color::RED.alpha(0.2));
        d.draw_rectangle_lines_ex(bounds, 3.0, Color::RED);
    }
}

//------------------------------------------------------------------------------------
// Program main entry point
//------------------------------------------------------------------------------------
fn main() {
    use raylib::ffi::KeyboardKey as K;
    // Initialization
    //--------------------------------------------------------------------------------------
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [core] example - keyboard testbed")
        .build();
    rl.set_exit_key(None); // Avoid exit on KEY_ESCAPE

    // Keyboard line 01
    let mut line01_key_widths: [i32; 15] = [0; 15];
    for i in 0..15 {
        line01_key_widths[i] = 45;
    }
    line01_key_widths[13] = 62; // PRINTSCREEN
    let line01_keys: [i32; 15] = [
        K::KEY_ESCAPE as i32,
        K::KEY_F1 as i32,
        K::KEY_F2 as i32,
        K::KEY_F3 as i32,
        K::KEY_F4 as i32,
        K::KEY_F5 as i32,
        K::KEY_F6 as i32,
        K::KEY_F7 as i32,
        K::KEY_F8 as i32,
        K::KEY_F9 as i32,
        K::KEY_F10 as i32,
        K::KEY_F11 as i32,
        K::KEY_F12 as i32,
        K::KEY_PRINT_SCREEN as i32,
        K::KEY_PAUSE as i32,
    ];

    // Keyboard line 02
    let mut line02_key_widths: [i32; 15] = [0; 15];
    for i in 0..15 {
        line02_key_widths[i] = 45;
    }
    line02_key_widths[0] = 25; // GRAVE
    line02_key_widths[13] = 82; // BACKSPACE
    let line02_keys: [i32; 15] = [
        K::KEY_GRAVE as i32,
        K::KEY_ONE as i32,
        K::KEY_TWO as i32,
        K::KEY_THREE as i32,
        K::KEY_FOUR as i32,
        K::KEY_FIVE as i32,
        K::KEY_SIX as i32,
        K::KEY_SEVEN as i32,
        K::KEY_EIGHT as i32,
        K::KEY_NINE as i32,
        K::KEY_ZERO as i32,
        K::KEY_MINUS as i32,
        K::KEY_EQUAL as i32,
        K::KEY_BACKSPACE as i32,
        K::KEY_DELETE as i32,
    ];

    // Keyboard line 03
    let mut line03_key_widths: [i32; 15] = [0; 15];
    for i in 0..15 {
        line03_key_widths[i] = 45;
    }
    line03_key_widths[0] = 50; // TAB
    line03_key_widths[13] = 57; // BACKSLASH
    let line03_keys: [i32; 15] = [
        K::KEY_TAB as i32,
        K::KEY_Q as i32,
        K::KEY_W as i32,
        K::KEY_E as i32,
        K::KEY_R as i32,
        K::KEY_T as i32,
        K::KEY_Y as i32,
        K::KEY_U as i32,
        K::KEY_I as i32,
        K::KEY_O as i32,
        K::KEY_P as i32,
        K::KEY_LEFT_BRACKET as i32,
        K::KEY_RIGHT_BRACKET as i32,
        K::KEY_BACKSLASH as i32,
        K::KEY_INSERT as i32,
    ];

    // Keyboard line 04
    let mut line04_key_widths: [i32; 14] = [0; 14];
    for i in 0..14 {
        line04_key_widths[i] = 45;
    }
    line04_key_widths[0] = 68; // CAPS
    line04_key_widths[12] = 88; // ENTER
    let line04_keys: [i32; 14] = [
        K::KEY_CAPS_LOCK as i32,
        K::KEY_A as i32,
        K::KEY_S as i32,
        K::KEY_D as i32,
        K::KEY_F as i32,
        K::KEY_G as i32,
        K::KEY_H as i32,
        K::KEY_J as i32,
        K::KEY_K as i32,
        K::KEY_L as i32,
        K::KEY_SEMICOLON as i32,
        K::KEY_APOSTROPHE as i32,
        K::KEY_ENTER as i32,
        K::KEY_PAGE_UP as i32,
    ];

    // Keyboard line 05
    let mut line05_key_widths: [i32; 14] = [0; 14];
    for i in 0..14 {
        line05_key_widths[i] = 45;
    }
    line05_key_widths[0] = 80; // LSHIFT
    line05_key_widths[11] = 76; // RSHIFT
    let line05_keys: [i32; 14] = [
        K::KEY_LEFT_SHIFT as i32,
        K::KEY_Z as i32,
        K::KEY_X as i32,
        K::KEY_C as i32,
        K::KEY_V as i32,
        K::KEY_B as i32,
        K::KEY_N as i32,
        K::KEY_M as i32,
        K::KEY_COMMA as i32,
        K::KEY_PERIOD as i32, /*KEY_MINUS*/
        K::KEY_SLASH as i32,
        K::KEY_RIGHT_SHIFT as i32,
        K::KEY_UP as i32,
        K::KEY_PAGE_DOWN as i32,
    ];

    // Keyboard line 06
    let mut line06_key_widths: [i32; 11] = [0; 11];
    for i in 0..11 {
        line06_key_widths[i] = 45;
    }
    line06_key_widths[0] = 80; // LCTRL
    line06_key_widths[3] = 208; // SPACE
    line06_key_widths[7] = 60; // RCTRL
    let line06_keys: [i32; 11] = [
        K::KEY_LEFT_CONTROL as i32,
        K::KEY_LEFT_SUPER as i32,
        K::KEY_LEFT_ALT as i32,
        K::KEY_SPACE as i32,
        K::KEY_RIGHT_ALT as i32,
        162,
        K::KEY_NULL as i32,
        K::KEY_RIGHT_CONTROL as i32,
        K::KEY_LEFT as i32,
        K::KEY_DOWN as i32,
        K::KEY_RIGHT as i32,
    ];

    let keyboard_offset = Vector2::new(26.0, 80.0);

    rl.set_target_fps(60);
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        let key = rl.get_key_pressed_number(); // Get pressed keycode
        if let Some(k) = key {
            unsafe {
                raylib::ffi::TraceLog(
                    raylib::ffi::TraceLogLevel::LOG_INFO as i32,
                    c"KEYBOARD TESTBED: KEY PRESSED:    %d".as_ptr(),
                    k as i32,
                );
            }
        }

        let ch = rl.get_char_pressed(); // Get pressed char for text input, using OS mapping
        if let Some(c) = ch {
            let cval = c as u32;
            unsafe {
                raylib::ffi::TraceLog(
                    raylib::ffi::TraceLogLevel::LOG_INFO as i32,
                    c"KEYBOARD TESTBED: CHAR PRESSED:   %c (%d)".as_ptr(),
                    cval as i32,
                    cval as i32,
                );
            }
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_text("KEYBOARD LAYOUT: ENG-US", 26, 38, 20, Color::LIGHTGRAY);

        // Keyboard line 01 - 15 keys
        // ESC, F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, IMP, CLOSE
        let mut rec_offset_x = 0;
        for i in 0..15 {
            gui_keyboard_key(
                &mut d,
                Rectangle::new(
                    keyboard_offset.x + rec_offset_x as f32,
                    keyboard_offset.y,
                    line01_key_widths[i] as f32,
                    30.0,
                ),
                line01_keys[i],
            );
            rec_offset_x += line01_key_widths[i] + KEY_REC_SPACING as i32;
        }

        // Keyboard line 02 - 15 keys
        // `, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, -, =, BACKSPACE, DEL
        let mut rec_offset_x = 0;
        for i in 0..15 {
            gui_keyboard_key(
                &mut d,
                Rectangle::new(
                    keyboard_offset.x + rec_offset_x as f32,
                    keyboard_offset.y + 30.0 + KEY_REC_SPACING,
                    line02_key_widths[i] as f32,
                    38.0,
                ),
                line02_keys[i],
            );
            rec_offset_x += line02_key_widths[i] + KEY_REC_SPACING as i32;
        }

        // Keyboard line 03 - 15 keys
        // TAB, Q, W, E, R, T, Y, U, I, O, P, [, ], \, INS
        let mut rec_offset_x = 0;
        for i in 0..15 {
            gui_keyboard_key(
                &mut d,
                Rectangle::new(
                    keyboard_offset.x + rec_offset_x as f32,
                    keyboard_offset.y + 30.0 + 38.0 + KEY_REC_SPACING * 2.0,
                    line03_key_widths[i] as f32,
                    38.0,
                ),
                line03_keys[i],
            );
            rec_offset_x += line03_key_widths[i] + KEY_REC_SPACING as i32;
        }

        // Keyboard line 04 - 14 keys
        // MAYUS, A, S, D, F, G, H, J, K, L, ;, ', ENTER, REPAG
        let mut rec_offset_x = 0;
        for i in 0..14 {
            gui_keyboard_key(
                &mut d,
                Rectangle::new(
                    keyboard_offset.x + rec_offset_x as f32,
                    keyboard_offset.y + 30.0 + 38.0 * 2.0 + KEY_REC_SPACING * 3.0,
                    line04_key_widths[i] as f32,
                    38.0,
                ),
                line04_keys[i],
            );
            rec_offset_x += line04_key_widths[i] + KEY_REC_SPACING as i32;
        }

        // Keyboard line 05 - 14 keys
        // LSHIFT, Z, X, C, V, B, N, M, ,, ., /, RSHIFT, UP, AVPAG
        let mut rec_offset_x = 0;
        for i in 0..14 {
            gui_keyboard_key(
                &mut d,
                Rectangle::new(
                    keyboard_offset.x + rec_offset_x as f32,
                    keyboard_offset.y + 30.0 + 38.0 * 3.0 + KEY_REC_SPACING * 4.0,
                    line05_key_widths[i] as f32,
                    38.0,
                ),
                line05_keys[i],
            );
            rec_offset_x += line05_key_widths[i] + KEY_REC_SPACING as i32;
        }

        // Keyboard line 06 - 11 keys
        // LCTRL, WIN, LALT, SPACE, ALTGR, \, FN, RCTRL, LEFT, DOWN, RIGHT
        let mut rec_offset_x = 0;
        for i in 0..11 {
            gui_keyboard_key(
                &mut d,
                Rectangle::new(
                    keyboard_offset.x + rec_offset_x as f32,
                    keyboard_offset.y + 30.0 + 38.0 * 4.0 + KEY_REC_SPACING * 5.0,
                    line06_key_widths[i] as f32,
                    38.0,
                ),
                line06_keys[i],
            );
            rec_offset_x += line06_key_widths[i] + KEY_REC_SPACING as i32;
        }

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
