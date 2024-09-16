use raylib::prelude::*;

fn main() {
    // Initialization
    let screen_width = 800;
    let screen_height = 450;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("raylib [text] example - draw text inside a rectangle")
        .build();

    let text = "Text cannot escape\tthis container\t...word wrap also works when active so here's \
a long text for testing.\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
tempor incididunt ut labore et dolore magna aliqua. Nec ullamcorper sit amet risus nullam eget felis eget.";

    let mut resizing = false;
    let mut word_wrap = true;

    let mut container = Rectangle {
        x: 25.0,
        y: 25.0,
        width: screen_width as f32 - 50.0,
        height: screen_height as f32 - 250.0,
    };

    let mut resizer = Rectangle {
        x: container.x + container.width - 17.0,
        y: container.y + container.height - 17.0,
        width: 14.0,
        height: 14.0,
    };

    // Minimum width and heigh for the container rectangle
    let min_width = 60.0;
    let min_height = 60.0;
    let max_width = screen_width as f32 - 50.0;
    let max_height = screen_height as f32 - 160.0;

    let mut last_mouse = Vector2::zero(); // Stores last mouse coordinates
    let mut border_color = Color::MAROON; // Container border color
    let font = rl.get_font_default(); // Get default system font

    rl.set_target_fps(60); // Set our game to run at 60 frames-per-second

    // Main game loop
    // Detect window close button or ESC key
    while !rl.window_should_close() {
        // Update
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            word_wrap = !word_wrap;
        }

        let mouse = rl.get_mouse_position();

        // Check if the mouse is inside the container and toggle border color
        if check_collision_point_rec(mouse, container) {
            border_color = rl.gui_fade(Color::MAROON, 0.4);
        } else if !resizing {
            border_color = Color::MAROON;
        }

        // Container resizing logic
        if resizing {
            if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
                resizing = false;
            }

            let width = container.width + (mouse.x - last_mouse.x);
            container.width = f32::clamp(width, min_width, max_width);

            let height = container.height + (mouse.y - last_mouse.y);
            container.height = f32::clamp(height, min_height, max_height);
        } else {
            // Check if we're resizing
            if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT)
                && check_collision_point_rec(mouse, resizer)
            {
                resizing = true;
            }
        }

        // Move resizer rectangle properly
        resizer.x = container.x + container.width - 17.0;
        resizer.y = container.y + container.height - 17.0;

        last_mouse = mouse; // Update mouse

        // Draw
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::RAYWHITE);

        d.draw_rectangle_lines_ex(container, 3.0, border_color); // Draw container border

        // Draw text in container (add some padding)
        draw_text_boxed(
            &mut d,
            &font,
            text,
            Rectangle {
                x: container.x + 4.0,
                y: container.y + 4.0,
                width: container.width - 4.0,
                height: container.height - 4.0,
            },
            20.,
            2.0,
            word_wrap,
            Color::GRAY,
        );

        d.draw_rectangle_rec(resizer, border_color); // Draw the resize box

        // Draw bottom info
        d.draw_rectangle(0, screen_height - 54, screen_width, 54, Color::GRAY);
        d.draw_rectangle(382, screen_height - 34, 12, 12, Color::MAROON);

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
        )

        // No need for end_drawing, it'll end once `d` goes out of scope
    }

    // Window gets closed on drop
}

fn draw_text_boxed(
    d: &mut RaylibDrawHandle,
    font: &impl RaylibFont,
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

fn draw_text_boxed_selectable(
    d: &mut RaylibDrawHandle,
    font: &impl RaylibFont,
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
    let mut text_offset_y = 0.0; // Offset between lines (on line break '\n')
    let mut text_offset_x = 0.0; // Offset X to next character to draw

    let scale_factor = font_size / font.base_size() as f32; // Character rectangle scaling factor

    // Word/character wrapping mechanism variables
    enum MeasureState {
        Measure,
        Draw,
    }

    let mut state = if word_wrap {
        MeasureState::Measure
    } else {
        MeasureState::Draw
    };

    let mut start_line = -1;
    let mut end_line = -1;
    let mut last_k: i32 = -1;

    // PORTING NOTE: This loop is very c-like and doesn't really use much of
    // rusts strings, could be improved. But it is correct.
    let mut i: i32 = -1;
    loop {
        i += 1;
        if i as usize >= text.len() {
            break;
        }

        // Get next codepoint from byte string and glyph index in font
        let codepoint = text[i as usize..].chars().next().unwrap();
        i += codepoint.len_utf8() as i32 - 1;
        let mut k = i as i32;

        let glyph_index = font.get_glyph_index(codepoint) as usize;
        let glyph = &font.chars()[glyph_index];
        let mut glyph_width = 0.0;
        if codepoint != '\n' {
            if glyph.advanceX == 0 {
                glyph_width = glyph.image.width as f32 * scale_factor;
            } else {
                glyph_width = glyph.advanceX as f32 * scale_factor;
            }

            if (i as usize + 1) < text.len() {
                glyph_width += spacing;
            }
        }

        match state {
            MeasureState::Measure => {
                // NOTE: When wordWrap is ON we first measure how much of the text we can draw
                // before going outside of the rec container.
                // We store this info in startLine and endLine, then we change states, draw the text
                // between those two variables and change states again and again recursively until
                // the end of the text (or until we get outside of the container).
                // When wordWrap is OFF we don't need the measure state so we go to the drawing state
                // immediately and begin drawing on the next line before we can get outside the container.

                if codepoint.is_whitespace() {
                    end_line = i as i32;
                }

                if text_offset_x + glyph_width > rec.width {
                    end_line = if end_line < 1 { i as i32 } else { end_line };
                    if i == end_line {
                        end_line -= codepoint.len_utf8() as i32;
                    }

                    if (start_line + codepoint.len_utf8() as i32) == end_line {
                        end_line = i as i32 - codepoint.len_utf8() as i32;
                    }

                    state = MeasureState::Draw;
                } else if i as usize + 1 == text.len() {
                    end_line = i as i32;
                    state = MeasureState::Draw;
                } else if codepoint == '\n' {
                    state = MeasureState::Draw;
                }

                if matches!(state, MeasureState::Draw) {
                    text_offset_x = 0.0;
                    i = start_line;
                    glyph_width = 0.0;

                    // Save character position when we switch states
                    k = last_k;
                    last_k = k - 1;
                }
            }
            MeasureState::Draw => {
                if codepoint == '\n' {
                    if !word_wrap {
                        text_offset_y += (font.base_size() as f32 + font.base_size() as f32 / 2.0)
                            * scale_factor;
                        text_offset_x = 0.0;
                    }
                } else {
                    if !word_wrap && (text_offset_x + glyph_width > rec.width) {
                        text_offset_y += (font.base_size() as f32 + font.base_size() as f32 / 2.0)
                            * scale_factor;
                        text_offset_x = 0.0;
                    }

                    // When text overflows rectangle height limit, just stop drawing
                    if text_offset_y + font.base_size() as f32 * scale_factor > rec.height {
                        break;
                    }

                    // Draw selection background
                    let is_glyph_selected = if select_start >= 0
                        && k as i32 >= select_start
                        && (k as i32) < (select_start + select_length)
                    {
                        d.draw_rectangle_rec(
                            Rectangle {
                                x: rec.x + text_offset_x - 1.0,
                                y: rec.y + text_offset_y,
                                width: glyph_width,
                                height: font.base_size() as f32 * scale_factor,
                            },
                            select_back_tint,
                        );
                        true
                    } else {
                        false
                    };

                    // Draw current character glyph
                    if codepoint != ' ' && codepoint != '\t' {
                        d.draw_text_codepoint(
                            font,
                            codepoint as i32,
                            Vector2 {
                                x: rec.x + text_offset_x,
                                y: rec.y + text_offset_y,
                            },
                            font_size,
                            if is_glyph_selected { select_tint } else { tint },
                        );
                    }
                }

                if word_wrap && i as i32 == end_line {
                    text_offset_y +=
                        (font.base_size() as f32 + font.base_size() as f32 / 2.0) * scale_factor;
                    text_offset_x = 0.0;
                    start_line = end_line;
                    end_line = -1;
                    select_start += last_k - k as i32;
                    state = MeasureState::Measure;
                }
            }
        }

        if text_offset_x != 0.0 || codepoint != ' ' {
            text_offset_x += glyph_width; // Avoid leading spaces
        }
    }
}

// FIXME: No bindings for CheckCollisionPointRec?
fn check_collision_point_rec(point: Vector2, rec: Rectangle) -> bool {
    (point.x >= rec.x)
        && (point.x <= (rec.x + rec.width))
        && (point.y >= rec.y)
        && (point.y <= (rec.y + rec.height))
}
