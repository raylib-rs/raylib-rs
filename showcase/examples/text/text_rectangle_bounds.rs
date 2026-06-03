/*******************************************************************************************
*
*   raylib [text] example - rectangle bounds
*
*   Example complexity rating: [★★★★] 4/4
*
*   Example originally created with raylib 2.5, last time updated with raylib 4.0
*
*   Example contributed by Vlad Adrian (@demizdor) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2018-2025 Vlad Adrian (@demizdor) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::drawing::RaylibDraw;
use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------
// Draw text using font inside rectangle limits
#[expect(
    clippy::too_many_arguments,
    reason = "C-parity: mirrors the C function signature"
)]
fn draw_text_boxed<D: RaylibDraw>(
    d: &mut D,
    font: &WeakFont,
    text: &str,
    rec: Rectangle,
    font_size: f32,
    spacing: f32,
    word_wrap: bool,
    tint: Color,
) {
    draw_text_boxed_selectable(
        d,
        font,
        text,
        rec,
        font_size,
        spacing,
        word_wrap,
        tint,
        0,
        0,
        Color::WHITE,
        Color::WHITE,
    );
}

// Draw text using font inside rectangle limits with support for text selection
#[allow(clippy::too_many_arguments)]
fn draw_text_boxed_selectable<D: RaylibDraw>(
    d: &mut D,
    font: &WeakFont,
    text: &str,
    rec: Rectangle,
    font_size: f32,
    spacing: f32,
    word_wrap: bool,
    tint: Color,
    mut select_start: i32,
    select_length: i32,
    select_tint: Color,
    select_back_tint: Color,
) {
    // idiomatic: C uses GetCodepoint/GetGlyphIndex on a byte string; we iterate UTF-8 bytes
    // by codepoint via char_indices to match the C byte-level book-keeping.
    let bytes = text.as_bytes();
    let length = bytes.len() as i32;

    let mut text_offset_y: f32 = 0.0;
    let mut text_offset_x: f32 = 0.0;

    let scale_factor = font_size / font.base_size() as f32;

    // Word/character wrapping mechanism variables
    const MEASURE_STATE: i32 = 0;
    const DRAW_STATE: i32 = 1;
    let mut state = if word_wrap { MEASURE_STATE } else { DRAW_STATE };

    let mut start_line: i32 = -1; // Index where to begin drawing (where a line begins)
    let mut end_line: i32 = -1; // Index where to stop drawing (where a line ends)
    let mut lastk: i32 = -1; // Holds last value of the character position

    let mut i: i32 = 0;
    let mut k: i32 = 0;
    while i < length {
        // Get next codepoint from byte string and glyph index in font
        let (codepoint, codepoint_byte_count) = next_codepoint(bytes, i as usize);
        let mut codepoint_byte_count = codepoint_byte_count;
        let index = font.get_glyph_index(codepoint);

        // NOTE: Normally we exit the decoding sequence as soon as a bad byte is found (and return 0x3f)
        // but we need to draw all of the bad bytes using the '?' symbol moving one byte
        if codepoint as i32 == 0x3f {
            codepoint_byte_count = 1;
        }
        i += codepoint_byte_count - 1;

        let mut glyph_width: f32 = 0.0;
        if codepoint != '\n' {
            let advance_x = font.chars()[index as usize].advanceX;
            glyph_width = if advance_x == 0 {
                font.recs()[index as usize].width * scale_factor
            } else {
                advance_x as f32 * scale_factor
            };

            if i + 1 < length {
                glyph_width += spacing;
            }
        }

        // NOTE: When wordWrap is ON we first measure how much of the text we can draw before going outside of the rec container
        // We store this info in startLine and endLine, then we change states, draw the text between those two variables
        // and change states again and again recursively until the end of the text (or until we get outside of the container)
        // When wordWrap is OFF we don't need the measure state so we go to the drawing state immediately
        // and begin drawing on the next line before we can get outside the container
        if state == MEASURE_STATE {
            // TODO: There are multiple types of spaces in UNICODE, maybe it's a good idea to add support for more
            if codepoint == ' ' || codepoint == '\t' || codepoint == '\n' {
                end_line = i;
            }

            if (text_offset_x + glyph_width) > rec.width {
                end_line = if end_line < 1 { i } else { end_line };
                if i == end_line {
                    end_line -= codepoint_byte_count;
                }
                if (start_line + codepoint_byte_count) == end_line {
                    end_line = i - codepoint_byte_count;
                }

                state = 1 - state;
            } else if (i + 1) == length {
                end_line = i;
                state = 1 - state;
            } else if codepoint == '\n' {
                state = 1 - state;
            }

            if state == DRAW_STATE {
                text_offset_x = 0.0;
                i = start_line;
                glyph_width = 0.0;

                // Save character position when we switch states
                let tmp = lastk;
                lastk = k - 1;
                k = tmp;
            }
        } else {
            if codepoint == '\n' {
                if !word_wrap {
                    text_offset_y +=
                        (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                    text_offset_x = 0.0;
                }
            } else {
                if !word_wrap && (text_offset_x + glyph_width) > rec.width {
                    text_offset_y +=
                        (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                    text_offset_x = 0.0;
                }

                // When text overflows rectangle height limit, just stop drawing
                if (text_offset_y + font.base_size() as f32 * scale_factor) > rec.height {
                    break;
                }

                // Draw selection background
                let mut is_glyph_selected = false;
                if select_start >= 0 && k >= select_start && k < (select_start + select_length) {
                    d.draw_rectangle_rec(
                        Rectangle::new(
                            rec.x + text_offset_x - 1.0,
                            rec.y + text_offset_y,
                            glyph_width,
                            font.base_size() as f32 * scale_factor,
                        ),
                        select_back_tint,
                    );
                    is_glyph_selected = true;
                }

                // Draw current character glyph
                if codepoint != ' ' && codepoint != '\t' {
                    d.draw_text_codepoint(
                        font,
                        codepoint as i32,
                        Vector2::new(rec.x + text_offset_x, rec.y + text_offset_y),
                        font_size,
                        if is_glyph_selected { select_tint } else { tint },
                    );
                }
            }

            if word_wrap && i == end_line {
                text_offset_y +=
                    (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                text_offset_x = 0.0;
                start_line = end_line;
                end_line = -1;
                glyph_width = 0.0;
                select_start += lastk - k;
                k = lastk;

                state = 1 - state;
            }
        }

        if text_offset_x != 0.0 || codepoint != ' ' {
            text_offset_x += glyph_width; // avoid leading spaces
        }

        i += 1;
        k += 1;
    }
}

// Minimal UTF-8 codepoint decode mirroring raylib's GetCodepoint
fn next_codepoint(bytes: &[u8], i: usize) -> (char, i32) {
    if i >= bytes.len() {
        return ('\u{3f}', 1);
    }
    let b0 = bytes[i];
    let (cp, n) = if b0 < 0x80 {
        (b0 as u32, 1)
    } else if (b0 & 0xE0) == 0xC0 && i + 1 < bytes.len() {
        let b1 = bytes[i + 1];
        (((b0 as u32 & 0x1F) << 6) | (b1 as u32 & 0x3F), 2)
    } else if (b0 & 0xF0) == 0xE0 && i + 2 < bytes.len() {
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        (
            ((b0 as u32 & 0x0F) << 12) | ((b1 as u32 & 0x3F) << 6) | (b2 as u32 & 0x3F),
            3,
        )
    } else if (b0 & 0xF8) == 0xF0 && i + 3 < bytes.len() {
        let b1 = bytes[i + 1];
        let b2 = bytes[i + 2];
        let b3 = bytes[i + 3];
        (
            ((b0 as u32 & 0x07) << 18)
                | ((b1 as u32 & 0x3F) << 12)
                | ((b2 as u32 & 0x3F) << 6)
                | (b3 as u32 & 0x3F),
            4,
        )
    } else {
        (0x3f, 1)
    };
    (char::from_u32(cp).unwrap_or('\u{3f}'), n)
}

trait FontRecs {
    fn recs(&self) -> &[Rectangle];
}

impl FontRecs for WeakFont {
    fn recs(&self) -> &[Rectangle] {
        // SAFETY: WeakFont's underlying ffi::Font owns `recs` (a glyphCount-long
        // raylib-allocated array) for the lifetime of the font.
        let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(self);
        unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) }
    }
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
        .title("raylib [text] example - rectangle bounds")
        .build();

    let text = "Text cannot escape\tthis container\t...word wrap also works when active so here's a long text for testing.\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Nec ullamcorper sit amet risus nullam eget felis eget.";

    let mut resizing = false;
    let mut word_wrap = true;

    let mut container = Rectangle::new(
        25.0,
        25.0,
        screen_width as f32 - 50.0,
        screen_height as f32 - 250.0,
    );
    let mut resizer = Rectangle::new(
        container.x + container.width - 17.0,
        container.y + container.height - 17.0,
        14.0,
        14.0,
    );

    // Minimum width and heigh for the container rectangle
    let min_width: f32 = 60.0;
    let min_height: f32 = 60.0;
    let max_width: f32 = screen_width as f32 - 50.0;
    let max_height: f32 = screen_height as f32 - 160.0;

    let mut last_mouse = Vector2::new(0.0, 0.0); // Stores last mouse coordinates
    let mut border_color = Color::MAROON; // Container border color
    let font = rl.get_font_default(); // Get default system font

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
            word_wrap = !word_wrap;
        }

        let mouse = rl.get_mouse_position();

        // Check if the mouse is inside the container and toggle border color
        if container.check_collision_point_rec(mouse) {
            border_color = Color::MAROON.alpha(0.4);
        } else if !resizing {
            border_color = Color::MAROON;
        }

        // Container resizing logic
        if resizing {
            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                resizing = false;
            }

            let width = container.width + (mouse.x - last_mouse.x);
            container.width = if width > min_width {
                if width < max_width { width } else { max_width }
            } else {
                min_width
            };

            let height = container.height + (mouse.y - last_mouse.y);
            container.height = if height > min_height {
                if height < max_height {
                    height
                } else {
                    max_height
                }
            } else {
                min_height
            };
        } else {
            // Check if we're resizing
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
                && resizer.check_collision_point_rec(mouse)
            {
                resizing = true;
            }
        }

        // Move resizer rectangle properly
        resizer.x = container.x + container.width - 17.0;
        resizer.y = container.y + container.height - 17.0;

        last_mouse = mouse; // Update mouse
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_rectangle_lines_ex(container, 3.0, border_color); // Draw container border

        // Draw text in container (add some padding)
        draw_text_boxed(
            &mut d,
            &font,
            text,
            Rectangle::new(
                container.x + 4.0,
                container.y + 4.0,
                container.width - 4.0,
                container.height - 4.0,
            ),
            20.0,
            2.0,
            word_wrap,
            Color::GRAY,
        );

        d.draw_rectangle_rec(resizer, border_color); // Draw the resize box

        // Draw bottom info
        d.draw_rectangle(0, screen_height - 54, screen_width, 54, Color::GRAY);
        d.draw_rectangle_rec(
            Rectangle::new(382.0, screen_height as f32 - 34.0, 12.0, 12.0),
            Color::MAROON,
        );

        d.draw_text("Word Wrap: ", 313, screen_height - 115, 20, Color::BLACK);
        if word_wrap {
            d.draw_text("ON", 447, screen_height - 115, 20, Color::RED);
        } else {
            d.draw_text("OFF", 447, screen_height - 115, 20, Color::BLACK);
        }

        d.draw_text(
            "Press [SPACE] to toggle word wrap",
            218,
            screen_height - 86,
            20,
            Color::GRAY,
        );

        d.draw_text(
            "Click hold & drag the    to resize the container",
            155,
            screen_height - 38,
            20,
            Color::RAYWHITE,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
