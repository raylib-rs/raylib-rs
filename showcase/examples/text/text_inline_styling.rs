/*******************************************************************************************
*
*   raylib [text] example - inline styling
*
*   Example complexity rating: [★★★☆] 3/4
*
*   Example originally created with raylib 6.0, last time updated with raylib 6.0
*
*   Example contributed by Wagner Barongello (@SultansOfCode) and reviewed by Ramon Santamaria (@raysan5)
*
*   Example licensed under an unmodified zlib/libpng license, which is an OSI-certified,
*   BSD-like license that allows static linking with closed source software
*
*   Copyright (c) 2025 Wagner Barongello (@SultansOfCode) and Ramon Santamaria (@raysan5)
*
********************************************************************************************/

use raylib::core::drawing::RaylibDraw;
use raylib::core::text::RaylibFont;
use raylib::prelude::*;
use raylib_showcase::SourceViewer;

//----------------------------------------------------------------------------------
// Module Functions Declaration
//----------------------------------------------------------------------------------

// Draw text using inline styling
// PARAM: color is the default text color, background color is BLANK by default
// NOTE: Using input color as the base alpha multiplied to inline styles
fn draw_text_styled<D: RaylibDraw>(
    d: &mut D,
    font: &WeakFont,
    text: &str,
    position: Vector2,
    font_size: f32,
    spacing: f32,
    color: Color,
) {
    // Text inline styling strategy used: [ ] delimiters for format
    // - Define foreground color:      [cRRGGBBAA]
    // - Define background color:      [bRRGGBBAA]
    // - Reset formating:              [r]
    // Example: [bAA00AAFF][cFF0000FF]red text on gray background[r] normal text

    // Note: We use the provided font (default font fallback is handled by caller).

    let bytes = text.as_bytes();
    let text_len = bytes.len() as i32;

    let mut col_front = color;
    let mut col_back = Color::BLANK;
    let back_rec_padding: f32 = 4.0; // Background rectangle padding

    let mut text_offset_y: f32 = 0.0;
    let mut text_offset_x: f32 = 0.0;
    let text_line_spacing: f32 = 0.0;
    let scale_factor = font_size / font.base_size() as f32;

    let mut i: i32 = 0;
    while i < text_len {
        let (codepoint, codepoint_byte_count) = next_codepoint(bytes, i as usize);

        if codepoint == '\n' {
            text_offset_y += font_size + text_line_spacing;
            text_offset_x = 0.0;
            i += codepoint_byte_count;
            continue;
        }

        if codepoint == '[' {
            // Process pipe styling
            if i + 2 < text_len
                && bytes[(i + 1) as usize] == b'r'
                && bytes[(i + 2) as usize] == b']'
            {
                // Reset styling
                col_front = color;
                col_back = Color::BLANK;

                i += 3; // Skip "[r]"
                continue; // Do not draw characters
            } else if i + 1 < text_len
                && (bytes[(i + 1) as usize] == b'c' || bytes[(i + 1) as usize] == b'b')
            {
                i += 2; // Skip "[c" or "[b" to start parsing color

                // Parse following color
                let mut col_hex_text = [0u8; 9];
                let mut col_hex_count: i32 = 0;
                while (i + col_hex_count) < text_len
                    && bytes[(i + col_hex_count) as usize] != 0
                    && bytes[(i + col_hex_count) as usize] != b']'
                {
                    let c = bytes[(i + col_hex_count) as usize];
                    if c.is_ascii_digit()
                        || (b'A'..=b'F').contains(&c)
                        || (b'a'..=b'f').contains(&c)
                    {
                        col_hex_text[col_hex_count as usize] = c;
                        col_hex_count += 1;
                        if col_hex_count >= 8 {
                            break;
                        }
                    } else {
                        break; // Only affects while loop
                    }
                }

                // Convert hex color text into actual Color
                let hex_str =
                    std::str::from_utf8(&col_hex_text[..col_hex_count as usize]).unwrap_or("");
                let col_hex_value = u32::from_str_radix(hex_str, 16).unwrap_or(0);
                if bytes[(i - 1) as usize] == b'c' {
                    col_front = Color::get_color(col_hex_value);
                    //colFront.a *= (unsigned char)(colFront.a*(float)color.a/255.0f); // TODO: Review
                } else if bytes[(i - 1) as usize] == b'b' {
                    col_back = Color::get_color(col_hex_value);
                    //colBack.a *= (unsigned char)(colFront.a*(float)color.a/255.0f);
                }

                i += col_hex_count + 1; // Skip color value retrieved and ']'
                continue; // Do not draw characters
            }
        }

        let index = font.get_glyph_index(codepoint);
        let advance_x = font.chars()[index as usize].advanceX;
        let increase_x: f32 = if advance_x == 0 {
            font_recs(font)[index as usize].width * scale_factor + spacing
        } else {
            advance_x as f32 * scale_factor + spacing
        };

        // Draw background rectangle color (if required)
        if col_back.a > 0 {
            d.draw_rectangle_rec(
                Rectangle::new(
                    position.x + text_offset_x,
                    position.y + text_offset_y - back_rec_padding,
                    increase_x,
                    font_size + 2.0 * back_rec_padding,
                ),
                col_back,
            );
        }

        if codepoint != ' ' && codepoint != '\t' {
            d.draw_text_codepoint(
                font,
                codepoint as i32,
                Vector2::new(position.x + text_offset_x, position.y + text_offset_y),
                font_size,
                col_front,
            );
        }

        text_offset_x += increase_x;

        i += codepoint_byte_count;
    }
}

// Measure inline styled text
// NOTE: Measuring styled text requires skipping styling data
// WARNING: Not considering line breaks
fn measure_text_styled(font: &WeakFont, text: &str, font_size: f32, spacing: f32) -> Vector2 {
    let mut text_size = Vector2::new(0.0, 0.0);

    if text.is_empty() {
        return text_size; // Security check
    }

    let bytes = text.as_bytes();
    let text_len = bytes.len() as i32;
    //float textLineSpacing = fontSize*1.5f; // Not used...

    let mut text_width: f32 = 0.0;
    let text_height: f32 = font_size;
    let scale_factor = font_size / font.base_size() as f32;

    let mut valid_codepoint_counter: i32 = 0;

    let mut i: i32 = 0;
    while i < text_len {
        let (codepoint, codepoint_byte_count) = next_codepoint(bytes, i as usize);

        if codepoint == '[' {
            // Ignore pipe inline styling
            if i + 2 < text_len
                && bytes[(i + 1) as usize] == b'r'
                && bytes[(i + 2) as usize] == b']'
            {
                i += 3; // Skip "[r]"
                continue; // Do not measure characters
            } else if i + 1 < text_len
                && (bytes[(i + 1) as usize] == b'c' || bytes[(i + 1) as usize] == b'b')
            {
                i += 2; // Skip "[c" or "[b" to start parsing color

                let mut col_hex_count: i32 = 0;
                while (i + col_hex_count) < text_len
                    && bytes[(i + col_hex_count) as usize] != 0
                    && bytes[(i + col_hex_count) as usize] != b']'
                {
                    let c = bytes[(i + col_hex_count) as usize];
                    if c.is_ascii_digit()
                        || (b'A'..=b'F').contains(&c)
                        || (b'a'..=b'f').contains(&c)
                    {
                        col_hex_count += 1;
                    } else {
                        break;
                    }
                }

                i += col_hex_count + 1; // Skip color value retrieved and ']'
                continue; // Do not measure characters
            }
        } else if codepoint != '\n' {
            let index = font.get_glyph_index(codepoint);
            let g = &font.chars()[index as usize];
            if g.advanceX > 0 {
                text_width += g.advanceX as f32;
            } else {
                text_width += font_recs(font)[index as usize].width + g.offsetX as f32;
            }

            valid_codepoint_counter += 1;
            i += codepoint_byte_count;
        } else {
            i += codepoint_byte_count;
        }
    }

    text_size.x = text_width * scale_factor + (valid_codepoint_counter - 1) as f32 * spacing;
    text_size.y = text_height;

    text_size
}

fn font_recs(font: &WeakFont) -> &[Rectangle] {
    // SAFETY: WeakFont owns a glyphCount-long recs array for its lifetime.
    let inner: &raylib::ffi::Font = std::convert::AsRef::as_ref(font);
    unsafe { std::slice::from_raw_parts(inner.recs, inner.glyphCount as usize) }
}

// Minimal UTF-8 codepoint decode mirroring raylib's GetCodepointNext
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
        .title("raylib [text] example - inline styling")
        .build();

    #[allow(unused_assignments)]
    let mut text_size = Vector2::new(0.0, 0.0); // Measure text box for provided font and text
    let mut col_random = Color::RED; // Random color used on text
    let mut frame_counter: i32 = 0; // Used to generate a new random color every certain frames

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second
    let mut viewer = SourceViewer::for_current_example();
    //--------------------------------------------------------------------------------------

    // Main game loop
    while !rl.window_should_close()
    // Detect window close button or ESC key
    {
        // Update
        //----------------------------------------------------------------------------------
        frame_counter += 1;

        if (frame_counter % 20) == 0 {
            col_random.r = rl.get_random_value::<i32>(0..=255) as u8;
            col_random.g = rl.get_random_value::<i32>(0..=255) as u8;
            col_random.b = rl.get_random_value::<i32>(0..=255) as u8;
            col_random.a = 255;
        }
        viewer.update(&mut rl, &thread);
        //----------------------------------------------------------------------------------

        // Draw
        //----------------------------------------------------------------------------------
        let default_font = rl.get_font_default();
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        // Text inline styling strategy used: [ ] delimiters for format
        // - Define foreground color:      [cRRGGBBAA]
        // - Define background color:      [bRRGGBBAA]
        // - Reset formating:              [r]
        // Colors defined with [cRRGGBBAA] or [bRRGGBBAA] are multiplied by the base color alpha
        // This allows global transparency control while keeping per-section styling (ex. text fade effects)
        // Example: [bAA00AAFF][cFF0000FF]red text on gray background[r] normal text

        draw_text_styled(
            &mut d,
            &default_font,
            "This changes the [cFF0000FF]foreground color[r] of provided text!!!",
            Vector2::new(100.0, 80.0),
            20.0,
            2.0,
            Color::BLACK,
        );

        draw_text_styled(
            &mut d,
            &default_font,
            "This changes the [bFF00FFFF]background color[r] of provided text!!!",
            Vector2::new(100.0, 120.0),
            20.0,
            2.0,
            Color::BLACK,
        );

        draw_text_styled(
            &mut d,
            &default_font,
            "This changes the [c00ff00ff][bff0000ff]foreground and background colors[r]!!!",
            Vector2::new(100.0, 160.0),
            20.0,
            2.0,
            Color::BLACK,
        );

        draw_text_styled(
            &mut d,
            &default_font,
            "This changes the [c00ff00ff]alpha[r] relative [cffffffff][b000000ff]from source[r] [cff000088]color[r]!!!",
            Vector2::new(100.0, 200.0),
            20.0,
            2.0,
            Color::new(0, 0, 0, 100),
        );

        // Get pointer to formated text
        let text = format!(
            "Let's be [c{:02x}{:02x}{:02x}FF]CREATIVE[r] !!!",
            col_random.r, col_random.g, col_random.b
        );
        draw_text_styled(
            &mut d,
            &default_font,
            &text,
            Vector2::new(100.0, 240.0),
            40.0,
            2.0,
            Color::BLACK,
        );

        text_size = measure_text_styled(&default_font, &text, 40.0, 2.0);
        d.draw_rectangle_lines(
            100,
            240,
            text_size.x as i32,
            text_size.y as i32,
            Color::GREEN,
        );

        viewer.draw(&mut d);
        //----------------------------------------------------------------------------------
    }

    // De-Initialization
    //--------------------------------------------------------------------------------------
    // CloseWindow() is handled by RAII drop of `rl`.
    //--------------------------------------------------------------------------------------
}
